use std::path::PathBuf;

#[derive(Debug, Clone)]
pub(crate) struct CargoProject {
    pub(crate) root: PathBuf,
    pub(crate) manifest: PathBuf,
}

#[derive(Debug, Clone)]
pub(crate) struct PathError {
    pub(crate) path: PathBuf,
    pub(crate) message: String,
}

#[derive(Debug, Clone)]
pub(crate) struct CleanedProject {
    pub(crate) root: PathBuf,
    pub(crate) target: PathBuf,
    pub(crate) size_bytes: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct SkippedProject {
    pub(crate) root: PathBuf,
    pub(crate) reason: String,
}

#[derive(Debug, Default)]
pub(crate) struct DiscoveryResult {
    pub(crate) projects: Vec<CargoProject>,
    pub(crate) errors: Vec<PathError>,
}

#[derive(Debug, Default)]
pub(crate) struct CleanupReport {
    pub(crate) cleaned: Vec<CleanedProject>,
    pub(crate) dry_runs: Vec<CleanedProject>,
    pub(crate) skipped_missing_target: usize,
    pub(crate) skipped_unsafe: Vec<SkippedProject>,
    pub(crate) errors: Vec<PathError>,
}

#[derive(Debug)]
pub(crate) enum CleanupStatus {
    Cleaned(CleanedProject),
    DryRun(CleanedProject),
    MissingTarget,
    UnsafeSkip(SkippedProject),
}
