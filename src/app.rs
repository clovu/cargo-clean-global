use indicatif::ProgressBar;
use indicatif::ProgressDrawTarget;
use indicatif::ProgressStyle;
use std::cell::RefCell;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::io::IsTerminal;
use std::io::Write;
use std::io::{self};
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;
use std::rc::Rc;
use std::time::Duration;

use crate::cleanup::clean_projects;
use crate::cli::Cli;
use crate::config::resolve_target_dir;
use crate::discovery::discover_projects;
use crate::paths::normalize_existing_directory;
use crate::types::CargoProject;
use crate::types::CleanupReport;

pub(crate) fn run(options: Cli) -> ExitCode {
    let scan_roots = match prepare_scan_roots(&options) {
        Ok(roots) if !roots.is_empty() => roots,
        Ok(_) => {
            eprintln!("error: no readable scan roots were found");
            return ExitCode::from(2);
        }
        Err(message) => {
            eprintln!("error: {message}");
            return ExitCode::from(2);
        }
    };

    println!("cargo-clean-global {}\n", env!("CARGO_PKG_VERSION"));

    if options.dry_run {
        // println!("Dry run enabled: target directories will be reported but not deleted.\n");
        println!("🧪 Dry run mode — no files will be deleted\n")
    }

    println!("Scanning root(s):");
    for root in &scan_roots {
        println!("  {}\n", root.display());
    }

    let scan_progress = scan_spinner();
    scan_progress.enable_steady_tick(Duration::from_millis(80));

    let scan_state = Rc::new(RefCell::new(ScanProgressState::default()));
    let scan_state_for_found = Rc::clone(&scan_state);
    let mut discovery = discover_projects(
        &scan_roots,
        |root, path, found_count| {
            let root_label = scan_roots
                .iter()
                .position(|candidate| candidate == root)
                .map(|index| {
                    format!(
                        "[root {}/{}] {}",
                        index + 1,
                        scan_roots.len(),
                        root.display()
                    )
                })
                .unwrap_or_else(|| root.display().to_string());
            let target_total = format_bytes(scan_state.borrow().target_total_bytes);
            scan_progress.set_message(format!(
                "{root_label} Scanning crates...  found: {}  target: {} \n {} ",
                found_count,
                target_total,
                path.display()
            ));
        },
        |project| {
            if let Some(target_size) = target_size_bytes_for_project(project) {
                scan_state_for_found.borrow_mut().target_total_bytes += target_size;
            }
        },
    );
    discovery
        .projects
        .sort_by(|left, right| left.root.cmp(&right.root));
    discovery
        .errors
        .sort_by(|left, right| left.path.cmp(&right.path));
    finish_scan_progress(
        &scan_progress,
        discovery.projects.len(),
        discovery.errors.len(),
        scan_state.borrow().target_total_bytes,
    );

    if !discovery.errors.is_empty() {
        println!(
            "{}",
            console::style(format!("\n⚠️ Errors ({}):", discovery.errors.len())).red()
        );
    }

    discovery.errors.iter().for_each(|item| {
        eprintln!("  - {}: {}", item.path.display(), item.message);
    });

    match confirm_cleanup(&options, discovery.projects.len()) {
        Ok(true) => {}
        Ok(false) => {
            println!("Cleanup cancelled.");
            return ExitCode::SUCCESS;
        }
        Err(message) => {
            eprintln!("error: {message}");
            return ExitCode::from(2);
        }
    }

    println!("\n🧹 Cleaning projects...\n");
    let clean_progress = cleanup_progress(discovery.projects.len(), options.dry_run);
    let cleanup = clean_projects(&discovery.projects, options.dry_run, |project, report| {
        clean_progress.inc(1);
        let action_label = if options.dry_run {
            "would clean"
        } else {
            "cleaned"
        };
        let cleaned_count = if options.dry_run {
            report.dry_runs.len()
        } else {
            report.cleaned.len()
        };
        let freed_bytes = cleaned_size_bytes(report, options.dry_run);
        let target = resolve_target_dir(project)
            .map(|path| path.display().to_string())
            .unwrap_or_else(|_| String::from("<target unresolved>"));

        clean_progress.set_message(format!(
            "{action_label}: {cleaned_count}  freed: {}\n  - project: {}\n  - target: {target}",
            format_bytes(freed_bytes),
            project.root.display(),
        ));
    });
    finish_cleanup_progress(&clean_progress, &cleanup, options.dry_run);

    if !cleanup.skipped_unsafe.is_empty() {
        println!(
            "{} {}",
            console::style("⚠️ Skipped unsafe cleanups:").yellow(),
            cleanup.skipped_unsafe.len()
        );
    }

    cleanup.skipped_unsafe.iter().for_each(|item| {
        eprintln!("  - {} -> {}", item.root.display(), item.reason);
    });

    if options.dry_run {
        if !cleanup.dry_runs.is_empty() {
            println!("{}", console::style("🧪 Dry-run results").green());
        }

        cleanup.dry_runs.iter().for_each(|item| {
            eprintln!("  - {} -> {}", item.root.display(), item.target.display());
        });
    } else {
        cleanup.cleaned.iter().for_each(|item| {
            println!("  - {} -> {}", item.root.display(), item.target.display());
        });
    }

    if !cleanup.errors.is_empty() {
        println!("{}", console::style("❌ Cleanup Errors:").red());
        for error in &cleanup.errors {
            eprintln!("  - {} -> {}", error.path.display(), error.message);
        }
    }

    let total_errors = discovery.errors.len() + cleanup.errors.len();
    let effective_cleaned = if options.dry_run {
        cleanup.dry_runs.len()
    } else {
        cleanup.cleaned.len()
    };

    println!();
    println!("Summary:");
    println!("  Found: {} projects", discovery.projects.len());
    if options.dry_run {
        println!(
            "  Would clean: {} ({})",
            effective_cleaned,
            format_bytes(cleaned_size_bytes(&cleanup, options.dry_run))
        );
    } else {
        println!(
            "  Cleaned: {} ({})",
            effective_cleaned,
            format_bytes(cleaned_size_bytes(&cleanup, options.dry_run))
        );
    }
    println!(
        "  Skipped (missing target): {}",
        cleanup.skipped_missing_target
    );
    println!("  Skipped (safety rules): {}", cleanup.skipped_unsafe.len());
    println!("  Errors: {}", total_errors);

    if total_errors == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];

    let mut value = bytes as f64;
    let mut unit_index = 0;
    while value >= 1024.0 && unit_index < UNITS.len() - 1 {
        value /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{bytes} {}", UNITS[unit_index])
    } else {
        format!("{value:.1} {}", UNITS[unit_index])
    }
}

#[derive(Default)]
struct ScanProgressState {
    target_total_bytes: u64,
}

fn cleaned_size_bytes(report: &CleanupReport, dry_run: bool) -> u64 {
    let entries = if dry_run {
        &report.dry_runs
    } else {
        &report.cleaned
    };

    entries
        .iter()
        .map(|entry| entry.size_bytes)
        .fold(0_u64, |total, value| total.saturating_add(value))
}

fn target_size_bytes_for_project(project: &CargoProject) -> Option<u64> {
    let target = resolve_target_dir(project).ok()?;
    let metadata = fs::symlink_metadata(&target).ok()?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return None;
    }

    let canonical_target = fs::canonicalize(&target).ok()?;
    directory_size_bytes(&canonical_target).ok()
}

fn directory_size_bytes(path: &Path) -> io::Result<u64> {
    let mut total = 0_u64;
    let mut stack = vec![path.to_path_buf()];

    while let Some(current) = stack.pop() {
        let entries = fs::read_dir(&current)?;
        for entry in entries {
            let entry = entry?;
            let metadata = entry.metadata()?;
            if metadata.is_dir() {
                stack.push(entry.path());
            } else if metadata.is_file() {
                total = total.saturating_add(metadata.len());
            }
        }
    }

    Ok(total)
}

fn prepare_scan_roots(options: &Cli) -> Result<Vec<PathBuf>, String> {
    let raw_roots = if options.roots.is_empty() {
        default_scan_roots()
    } else {
        options.roots.clone()
    };

    let mut seen = HashSet::new();
    let mut roots = Vec::new();

    for root in raw_roots {
        let normalized = match normalize_existing_directory(&root) {
            Ok(path) => path,
            Err(error) if !options.roots.is_empty() => {
                return Err(format!("invalid scan root {}: {}", root.display(), error));
            }
            Err(_) => continue,
        };

        if seen.insert(normalized.clone()) {
            roots.push(normalized);
        }
    }

    roots.sort();
    Ok(roots)
}

fn default_scan_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    let home = dirs_next::home_dir().and_then(|path| normalize_existing_directory(&path).ok());
    let current = env::current_dir()
        .ok()
        .and_then(|path| normalize_existing_directory(&path).ok());

    if let Some(home) = home {
        roots.push(home.clone());

        if let Some(current) = current
            && current != home
            && !current.starts_with(&home)
        {
            roots.push(current);
        }
    } else if let Some(current) = current {
        roots.push(current);
    }

    roots
}

fn confirm_cleanup(options: &Cli, project_count: usize) -> Result<bool, String> {
    if options.dry_run || options.yes || project_count == 0 {
        return Ok(true);
    }

    if !io::stdin().is_terminal() || !io::stderr().is_terminal() {
        return Err(String::from(
            "cleanup confirmation requires an interactive terminal; rerun with --yes to skip the prompt",
        ));
    }

    prompt_for_confirmation(project_count)
}

fn prompt_for_confirmation(project_count: usize) -> Result<bool, String> {
    eprint!("Proceed to clean {project_count} discovered Cargo project(s)? [y/N]: ");
    io::stderr()
        .flush()
        .map_err(|error| format!("failed to flush confirmation prompt: {error}"))?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|error| format!("failed to read confirmation input: {error}"))?;

    Ok(parse_confirmation_input(&input))
}

fn parse_confirmation_input(input: &str) -> bool {
    matches!(input.trim().to_ascii_lowercase().as_str(), "y" | "yes")
}

fn scan_spinner() -> ProgressBar {
    let progress = progress_bar(ProgressBar::new_spinner());
    progress.set_style(
        ProgressStyle::with_template("{spinner} {msg}")
            .expect("valid scan progress style")
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ "),
    );
    progress.set_message("Scanning Cargo projects...");
    progress.enable_steady_tick(Duration::from_millis(80));
    progress
}

fn cleanup_progress(total_projects: usize, dry_run: bool) -> ProgressBar {
    let progress = progress_bar(ProgressBar::new(total_projects as u64));
    progress.set_style(
        ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
            .expect("valid cleanup progress style")
            .progress_chars("##-"),
    );
    let action = if dry_run {
        "Evaluating target directories..."
    } else {
        "Cleaning target directories..."
    };
    progress.set_message(action);
    progress
}

fn progress_bar(progress: ProgressBar) -> ProgressBar {
    if io::stderr().is_terminal() {
        progress.set_draw_target(ProgressDrawTarget::stderr());
    } else {
        progress.set_draw_target(ProgressDrawTarget::hidden());
    }
    progress
}

fn finish_scan_progress(
    progress: &ProgressBar,
    project_count: usize,
    error_count: usize,
    target_total_bytes: u64,
) {
    let message = format!(
        "{} Scan complete: found {project_count} Cargo project(s), target total {}, errors {error_count}\n",
        console::style("✔").green(),
        format_bytes(target_total_bytes),
    );
    progress.finish_with_message(message);
}

fn finish_cleanup_progress(
    progress: &ProgressBar,
    cleanup: &crate::types::CleanupReport,
    dry_run: bool,
) {
    let completed = if dry_run {
        cleanup.dry_runs.len()
    } else {
        cleanup.cleaned.len()
    };
    let freed_bytes = cleaned_size_bytes(cleanup, dry_run);
    let action = if dry_run { "Evaluation" } else { "Cleanup" };
    let message = format!(
        "{action} complete:\nprocessed {}, cleaned {}, freed {}, errors {}\n",
        cleanup.cleaned.len()
            + cleanup.dry_runs.len()
            + cleanup.skipped_missing_target
            + cleanup.skipped_unsafe.len()
            + cleanup.errors.len(),
        completed,
        format_bytes(freed_bytes),
        cleanup.errors.len()
    );
    progress.finish_with_message(message);
}

#[cfg(test)]
mod tests {
    use super::parse_confirmation_input;

    #[test]
    fn accepts_yes_confirmation() {
        assert!(parse_confirmation_input("y"));
        assert!(parse_confirmation_input("Y"));
        assert!(parse_confirmation_input("yes"));
        assert!(parse_confirmation_input(" Yes \n"));
    }

    #[test]
    fn rejects_non_yes_confirmation() {
        assert!(!parse_confirmation_input(""));
        assert!(!parse_confirmation_input("n"));
        assert!(!parse_confirmation_input("no"));
        assert!(!parse_confirmation_input("maybe"));
    }
}
