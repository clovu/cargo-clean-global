use std::fs;
use std::io;

use crate::config::resolve_target_dir;
use crate::manifest::looks_like_cargo_manifest;
use crate::paths::path_error;
use crate::types::{
    CargoProject, CleanedProject, CleanupReport, CleanupStatus, PathError, SkippedProject,
};

pub(crate) fn clean_projects<F>(
    projects: &[CargoProject],
    dry_run: bool,
    mut on_project_finished: F,
) -> CleanupReport
where
    F: FnMut(),
{
    let mut report = CleanupReport::default();

    for project in projects {
        match clean_project(project, dry_run) {
            Ok(CleanupStatus::Cleaned(entry)) => report.cleaned.push(entry),
            Ok(CleanupStatus::DryRun(entry)) => report.dry_runs.push(entry),
            Ok(CleanupStatus::MissingTarget) => report.skipped_missing_target += 1,
            Ok(CleanupStatus::UnsafeSkip(skipped)) => report.skipped_unsafe.push(skipped),
            Err(error) => report.errors.push(error),
        }

        on_project_finished();
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

    let cleaned = CleanedProject {
        root: project.root.clone(),
        target: canonical_target,
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
