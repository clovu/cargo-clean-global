use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use crate::manifest::looks_like_cargo_manifest;
use crate::paths::normalize_existing_directory;
use crate::paths::path_error;
use crate::paths::should_skip_dir;
use crate::types::CargoProject;
use crate::types::DiscoveryResult;

const MANIFEST_NAME: &str = "Cargo.toml";

pub(crate) fn discover_projects(
    scan_roots: &[PathBuf],
    mut on_scan: impl FnMut(&PathBuf, &PathBuf, usize),
    mut on_project_found: impl FnMut(&CargoProject),
) -> DiscoveryResult {
    let mut seen_roots = HashSet::new();
    let mut result = DiscoveryResult::default();

    for root in scan_roots {
        scan_directory(
            root,
            &mut seen_roots,
            &mut result,
            &mut on_scan,
            &mut on_project_found,
        );
    }

    result
}

fn scan_directory(
    root: &Path,
    seen_roots: &mut HashSet<PathBuf>,
    result: &mut DiscoveryResult,
    on_scan: &mut impl FnMut(&PathBuf, &PathBuf, usize),
    on_project_found: &mut impl FnMut(&CargoProject),
) {
    let root_path = root.to_path_buf();
    let mut stack = vec![root_path.clone()];

    while let Some(current_dir) = stack.pop() {
        on_scan(&root_path, &current_dir, result.projects.len());
        let entries = match fs::read_dir(&current_dir) {
            Ok(entries) => entries,
            Err(error) => {
                result.errors.push(path_error(
                    current_dir,
                    format!("failed to read directory: {error}"),
                ));
                continue;
            }
        };

        let mut manifest_path = None;
        let mut child_dirs = Vec::new();

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    result.errors.push(path_error(
                        current_dir.clone(),
                        format!("failed to enumerate directory entry: {error}"),
                    ));
                    continue;
                }
            };

            let file_type = match entry.file_type() {
                Ok(file_type) => file_type,
                Err(error) => {
                    result.errors.push(path_error(
                        entry.path(),
                        format!("failed to inspect file type: {error}"),
                    ));
                    continue;
                }
            };

            if file_type.is_symlink() {
                continue;
            }

            let path = entry.path();
            let file_name = entry.file_name();

            if file_type.is_dir() {
                if should_skip_dir(&file_name) {
                    continue;
                }
                child_dirs.push(path);
                continue;
            }

            if file_type.is_file() && file_name == OsStr::new(MANIFEST_NAME) {
                manifest_path = Some(path);
            }
        }

        if let Some(manifest_path) = manifest_path {
            match looks_like_cargo_manifest(&manifest_path) {
                Ok(true) => {
                    let project_root = normalize_existing_directory(&current_dir)
                        .unwrap_or_else(|_| current_dir.clone());
                    if seen_roots.insert(project_root.clone()) {
                        let project = CargoProject {
                            root: project_root,
                            manifest: manifest_path,
                        };
                        on_project_found(&project);
                        result.projects.push(project);
                    }
                }
                Ok(false) => {}
                Err(error) => result.errors.push(path_error(
                    manifest_path,
                    format!("failed to parse manifest: {error}"),
                )),
            }
        }

        child_dirs.sort();
        for child in child_dirs.into_iter().rev() {
            stack.push(child);
        }
    }
}
