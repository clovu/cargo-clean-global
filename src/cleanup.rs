use std::fs;
use std::io;
use std::path::Path;

use crate::config::resolve_target_dir;
use crate::manifest::looks_like_cargo_manifest;
use crate::paths::path_error;
use crate::types::CargoProject;
use crate::types::CleanedProject;
use crate::types::CleanupReport;
use crate::types::CleanupStatus;
use crate::types::PathError;
use crate::types::SkippedProject;

type ProjectFinishedCallback<'a> = &'a mut dyn FnMut(&CargoProject, &CleanupReport);

pub(crate) fn clean_projects(
    projects: &[CargoProject],
    dry_run: bool,
    mut on_project_finished: Option<ProjectFinishedCallback<'_>>,
) -> CleanupReport {
    let mut report = CleanupReport::default();

    for project in projects {
        match clean_project(project, dry_run) {
            Ok(CleanupStatus::Cleaned(entry)) => report.cleaned.push(entry),
            Ok(CleanupStatus::DryRun(entry)) => report.dry_runs.push(entry),
            Ok(CleanupStatus::MissingTarget) => report.skipped_missing_target += 1,
            Ok(CleanupStatus::UnsafeSkip(skipped)) => report.skipped_unsafe.push(skipped),
            Err(error) => report.errors.push(error),
        }

        if let Some(callback) = on_project_finished.as_deref_mut() {
            callback(project, &report);
        }
    }

    report
}

fn clean_project(project: &CargoProject, dry_run: bool) -> Result<CleanupStatus, PathError> {
    let target = resolve_target_dir(project)?;

    let metadata = match fs::symlink_metadata(&target) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(CleanupStatus::MissingTarget);
        }
        Err(error) => {
            return Err(path_error(
                project.root.clone(),
                format!("failed to inspect {}: {}", target.display(), error),
            ));
        }
    };

    if metadata.file_type().is_symlink() {
        return Ok(CleanupStatus::UnsafeSkip(SkippedProject {
            root: project.root.clone(),
            reason: String::from("target is a symlink and was left untouched"),
        }));
    }

    if !metadata.is_dir() {
        return Ok(CleanupStatus::UnsafeSkip(SkippedProject {
            root: project.root.clone(),
            reason: String::from("target exists but is not a directory"),
        }));
    }

    let canonical_target = fs::canonicalize(&target).map_err(|error| {
        path_error(
            project.root.clone(),
            format!("failed to resolve {}: {}", target.display(), error),
        )
    })?;

    if canonical_target == project.root {
        return Ok(CleanupStatus::UnsafeSkip(SkippedProject {
            root: project.root.clone(),
            reason: String::from("resolved target directory is the Cargo project root"),
        }));
    }

    if !canonical_target.starts_with(&project.root) {
        return Ok(CleanupStatus::UnsafeSkip(SkippedProject {
            root: project.root.clone(),
            reason: String::from("configured target-dir resolves outside the Cargo project root"),
        }));
    }

    match looks_like_cargo_manifest(&project.manifest) {
        Ok(true) => {}
        Ok(false) => {
            return Ok(CleanupStatus::UnsafeSkip(SkippedProject {
                root: project.root.clone(),
                reason: String::from("Cargo.toml no longer looks like a Cargo manifest"),
            }));
        }
        Err(error) => {
            return Ok(CleanupStatus::UnsafeSkip(SkippedProject {
                root: project.root.clone(),
                reason: format!("failed to re-read Cargo.toml before cleanup: {error}"),
            }));
        }
    }

    let size_bytes = directory_size_bytes(&canonical_target).map_err(|error| {
        path_error(
            project.root.clone(),
            format!(
                "failed to calculate size of {}: {}",
                canonical_target.display(),
                error
            ),
        )
    })?;

    let cleaned = CleanedProject {
        root: project.root.clone(),
        target: canonical_target,
        size_bytes,
    };

    if dry_run {
        return Ok(CleanupStatus::DryRun(cleaned));
    }

    fs::remove_dir_all(&cleaned.target).map_err(|error| {
        path_error(
            project.root.clone(),
            format!("failed to delete {}: {}", cleaned.target.display(), error),
        )
    })?;

    Ok(CleanupStatus::Cleaned(cleaned))
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

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};
    use std::{env, fs, path::PathBuf};

    #[cfg(unix)]
    use std::os::unix::fs as unix_fs;

    use super::clean_projects;
    use crate::types::CargoProject;

    fn unique_test_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos();

        env::temp_dir().join(format!("cargo-clean-global-cleanup-{name}-{nanos}"))
    }

    #[test]
    fn dry_run_reports_target_without_deleting_it() {
        let root = unique_test_dir("dry_run_reports_target_without_deleting_it");
        fs::create_dir_all(&root).expect("should create root directory");

        let root = fs::canonicalize(root).expect("project root should canonicalize");
        let target = root.join("target");
        let manifest = root.join("Cargo.toml");

        fs::create_dir_all(&target).expect("should create target directory");
        fs::write(
            &manifest,
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
        )
        .expect("should write Cargo.toml");
        fs::write(target.join("artifact.txt"), "compiled output")
            .expect("should write target artifact");

        let project = CargoProject {
            root: root.clone(),
            manifest,
        };

        let report = clean_projects(&[project], true, None);

        assert_eq!(report.dry_runs.len(), 1);
        assert!(report.cleaned.is_empty());
        assert!(target.exists());

        fs::remove_dir_all(&root).expect("should remove test project");
        assert!(!root.exists());
    }

    #[test]
    fn clean_removes_target_directory() {
        let root = unique_test_dir("clean_removes_target_directory");
        fs::create_dir_all(&root).expect("should create root directory");

        let root = fs::canonicalize(root).expect("project root should canonicalize");
        let target = root.join("target");
        let manifest = root.join("Cargo.toml");

        fs::create_dir_all(&target).expect("should create target directory");
        fs::write(
            &manifest,
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
        )
        .expect("should write Cargo.toml");
        fs::write(target.join("artifact.txt"), "compiled output")
            .expect("should write target artifact");

        let project = CargoProject {
            root: root.clone(),
            manifest,
        };

        let report = clean_projects(&[project], false, None);

        assert_eq!(report.cleaned.len(), 1);
        assert!(report.dry_runs.is_empty());
        assert!(!target.exists());

        fs::remove_dir_all(&root).expect("should remove test project");
        assert!(!root.exists());
    }

    #[test]
    fn clean_skips_missing_target_directory() {
        let root = unique_test_dir("clean_skips_missing_target_directory");
        fs::create_dir_all(&root).expect("should create root directory");

        let root = fs::canonicalize(root).expect("project root should canonicalize");
        let manifest = root.join("Cargo.toml");

        fs::write(
            &manifest,
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
        )
        .expect("should write Cargo.toml");

        let project = CargoProject {
            root: root.clone(),
            manifest,
        };

        let report = clean_projects(&[project], false, None);

        assert!(report.cleaned.is_empty());
        assert!(report.dry_runs.is_empty());
        assert_eq!(report.skipped_missing_target, 1);
        assert!(report.errors.is_empty());

        fs::remove_dir_all(&root).expect("should remove test project");
        assert!(!root.exists());
    }

    #[test]
    fn clean_skips_target_when_it_is_not_a_directory() {
        let root = unique_test_dir("clean_skips_target_when_it_is_not_a_directory");
        fs::create_dir_all(&root).expect("should create root directory");

        let root = fs::canonicalize(root).expect("project root should canonicalize");
        let target = root.join("target");
        let manifest = root.join("Cargo.toml");

        fs::write(
            &manifest,
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
        )
        .expect("should write Cargo.toml");
        fs::write(&target, "not a directory").expect("should write target file");

        let project = CargoProject {
            root: root.clone(),
            manifest,
        };

        let report = clean_projects(&[project], false, None);

        assert!(report.cleaned.is_empty());
        assert!(report.dry_runs.is_empty());
        assert_eq!(report.skipped_unsafe.len(), 1);
        assert!(report.errors.is_empty());
        assert!(target.exists());

        fs::remove_dir_all(&root).expect("should remove test project");
        assert!(!root.exists());
    }

    #[cfg(unix)]
    #[test]
    fn clean_skips_target_when_it_is_a_symlink() {
        let root = unique_test_dir("clean_skips_target_when_it_is_a_symlink");
        fs::create_dir_all(&root).expect("should create root directory");

        let root = fs::canonicalize(root).expect("project root should canonicalize");
        let real_target = root.join("real-target");
        let target = root.join("target");
        let manifest = root.join("Cargo.toml");

        fs::create_dir_all(&real_target).expect("should create real target directory");
        fs::write(
            &manifest,
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
        )
        .expect("should write Cargo.toml");
        unix_fs::symlink(&real_target, &target).expect("should create target symlink");

        let project = CargoProject {
            root: root.clone(),
            manifest,
        };

        let report = clean_projects(&[project], false, None);

        assert!(report.cleaned.is_empty());
        assert!(report.dry_runs.is_empty());
        assert_eq!(report.skipped_unsafe.len(), 1);
        assert!(report.errors.is_empty());
        assert!(
            target
                .symlink_metadata()
                .expect("target symlink should still exist")
                .file_type()
                .is_symlink()
        );
        assert!(real_target.exists());

        fs::remove_dir_all(&root).expect("should remove test project");
        assert!(!root.exists());
    }
}
