use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use crate::types::PathError;

const CARGO_CONFIG_DIR_NAME: &str = ".cargo";
pub(crate) const TARGET_DIR_NAME: &str = "target";

pub(crate) fn normalize_existing_directory(path: &Path) -> io::Result<PathBuf> {
    let metadata = fs::metadata(path)?;
    if !metadata.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "path is not a directory",
        ));
    }
    fs::canonicalize(path)
}

pub(crate) fn path_error(path: impl Into<PathBuf>, message: impl Into<String>) -> PathError {
    PathError {
        path: path.into(),
        message: message.into(),
    }
}

pub(crate) fn should_skip_dir(name: &OsStr) -> bool {
    let Some(name) = name.to_str() else {
        return false;
    };

    const COMMON_SKIPS: &[&str] = &[
        ".git",
        ".hg",
        ".svn",
        ".idea",
        ".vscode",
        ".cargo",
        ".rustup",
        "node_modules",
        TARGET_DIR_NAME,
    ];

    const COMMON_SKIPS_PREFIXES: &[&str] = &[
        // Common hidden directories on Unix-like systems
        ".",
    ];

    if COMMON_SKIPS
        .iter()
        .any(|candidate| name.eq_ignore_ascii_case(candidate))
    {
        return true;
    }

    if COMMON_SKIPS_PREFIXES
        .iter()
        .any(|prefix| name.starts_with(prefix))
    {
        return true;
    }

    if cfg!(windows) && name.eq_ignore_ascii_case("AppData") {
        return true;
    }

    if cfg!(target_os = "macos") && name == "Library" {
        return true;
    }

    if cfg!(unix) && name == ".cache" {
        return true;
    }

    false
}

pub(crate) fn cargo_home_dir() -> Option<PathBuf> {
    let cargo_home = env::var_os("CARGO_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);

    match cargo_home {
        Some(path) if path.is_absolute() => Some(path),
        Some(path) => env::current_dir()
            .ok()
            .map(|current_dir| current_dir.join(path)),
        None => dirs_next::home_dir().map(|home_dir| home_dir.join(CARGO_CONFIG_DIR_NAME)),
    }
}

#[cfg(test)]
mod tests {
    use crate::paths::should_skip_dir;

    use super::cargo_home_dir;
    use std::ffi::OsString;
    use std::path::PathBuf;
    use std::sync::Mutex;
    use std::sync::OnceLock;

    #[test]
    fn should_skip_specified_prefix_dir() {
        assert!(should_skip_dir(&OsString::from(".Trash")));
    }

    #[test]
    fn cargo_home_dir_prefers_env_var() {
        let _guard = env_lock().lock().expect("env lock should not be poisoned");
        let relative_home = PathBuf::from("relative-cargo-home");
        let expected = std::env::current_dir()
            .expect("current dir should exist")
            .join(&relative_home);
        let original = std::env::var_os("CARGO_HOME");

        unsafe {
            std::env::set_var("CARGO_HOME", &relative_home);
        }
        let resolved = cargo_home_dir().expect("cargo home should resolve");
        restore_cargo_home(original);

        assert_eq!(resolved, expected);
    }

    fn restore_cargo_home(original: Option<OsString>) {
        unsafe {
            match original {
                Some(value) => std::env::set_var("CARGO_HOME", value),
                None => std::env::remove_var("CARGO_HOME"),
            }
        }
    }

    fn env_lock() -> &'static Mutex<()> {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        ENV_LOCK.get_or_init(|| Mutex::new(()))
    }
}
