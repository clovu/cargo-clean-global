use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use toml::Value;

use crate::paths::TARGET_DIR_NAME;
use crate::paths::cargo_home_dir;
use crate::paths::expand_tilde;
use crate::paths::path_error;
use crate::types::CargoProject;
use crate::types::PathError;

const CARGO_CONFIG_DIR_NAME: &str = ".cargo";
const CARGO_CONFIG_LEGACY_FILE_NAME: &str = "config";
const CARGO_CONFIG_FILE_NAME: &str = "config.toml";
const CLEAN_GLOBAL_TABLE: &str = "cargo-clean-global";
const CLEAN_GLOBAL_ROOTS_FIELD: &str = "roots";

pub(crate) fn resolve_target_dir(project: &CargoProject) -> Result<PathBuf, PathError> {
    let configured_target = resolve_target_dir_from_configs(project, cargo_home_dir().as_deref())?;
    Ok(configured_target.unwrap_or_else(|| project.root.join(TARGET_DIR_NAME)))
}

pub(crate) fn resolve_default_scan_roots_from_global_config() -> Result<Vec<PathBuf>, PathError> {
    let Some(config_path) = resolve_cargo_config_path()? else {
        return Ok(vec![]);
    };

    let content = read_config_to_string(&config_path)?;
    let parsed: Value = toml::from_str(&content).map_err(|error| {
        path_error(
            &config_path,
            format!("failed to parse Cargo config: {error}"),
        )
    })?;

    parse_default_scan_roots(&config_path, &parsed)
}

fn parse_default_scan_roots(config_path: &Path, parsed: &Value) -> Result<Vec<PathBuf>, PathError> {
    let Some(roots_value) = parsed
        .get(CLEAN_GLOBAL_TABLE)
        .and_then(Value::as_table)
        .and_then(|table| table.get(CLEAN_GLOBAL_ROOTS_FIELD))
    else {
        return Ok(vec![]);
    };

    match roots_value {
        Value::String(path) => {
            let root = resolve_root_path_from_config(config_path, path);
            Ok(root.into_iter().collect())
        }
        Value::Array(values) => parse_root_array(config_path, values),
        _ => Err(path_error(
            config_path,
            "cargo-clean-global.roots must be a string or an array of strings",
        )),
    }
}

fn parse_root_array(config_path: &Path, values: &[Value]) -> Result<Vec<PathBuf>, PathError> {
    let mut roots = Vec::with_capacity(values.len());

    for item in values {
        let Some(path) = item.as_str() else {
            return Err(path_error(
                config_path,
                "cargo-clean-global.roots must be a string or an array of strings",
            ));
        };

        if let Some(path) = resolve_root_path_from_config(config_path, path) {
            roots.push(path);
        }
    }

    Ok(roots)
}

fn resolve_root_path_from_config(config_path: &Path, path: &str) -> Option<PathBuf> {
    let path = path.trim();
    if path.is_empty() {
        return None;
    }

    let path = expand_tilde(path);

    if path.is_absolute() {
        Some(path)
    } else {
        config_path.parent().map(|dir| dir.join(&path))
    }
}

fn resolve_cargo_config_path() -> Result<Option<PathBuf>, PathError> {
    let Some(cargo_home_dir) = cargo_home_dir() else {
        return Ok(None);
    };

    select_cargo_config_path(&cargo_home_dir)
}

fn resolve_target_dir_from_configs(
    project: &CargoProject,
    cargo_home: Option<&Path>,
) -> Result<Option<PathBuf>, PathError> {
    let project_config_dir = project.root.join(CARGO_CONFIG_DIR_NAME);
    if let Some(target_dir) = read_configured_target_dir_from_config_dir(&project_config_dir)? {
        return Ok(Some(target_dir));
    }

    let Some(cargo_home) = cargo_home else {
        return Ok(None);
    };

    read_configured_target_dir_from_config_dir(cargo_home)
}

fn read_configured_target_dir_from_config_dir(
    config_dir: &Path,
) -> Result<Option<PathBuf>, PathError> {
    let Some(config_path) = select_cargo_config_path(config_dir)? else {
        return Ok(None);
    };

    read_configured_target_dir_from_file(&config_path)
}

fn select_cargo_config_path(config_dir: &Path) -> Result<Option<PathBuf>, PathError> {
    for file_name in [CARGO_CONFIG_LEGACY_FILE_NAME, CARGO_CONFIG_FILE_NAME] {
        let config_path = config_dir.join(file_name);
        let metadata = match fs::metadata(&config_path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(path_error(
                    &config_path,
                    format!("failed to inspect Cargo config: {error}"),
                ));
            }
        };

        if !metadata.is_file() {
            return Err(path_error(
                &config_path,
                "Cargo config exists but is not a regular file",
            ));
        }

        return Ok(Some(config_path));
    }

    Ok(None)
}

fn read_configured_target_dir_from_file(config_path: &Path) -> Result<Option<PathBuf>, PathError> {
    let content = read_config_to_string(config_path)?;

    let parsed: Value = toml::from_str(&content).map_err(|error| {
        path_error(
            config_path,
            format!("failed to parse Cargo config: {error}"),
        )
    })?;

    let target_dir = parsed
        .get("build")
        .and_then(Value::as_table)
        .and_then(|build| build.get("target-dir"))
        .and_then(Value::as_str);

    let Some(target_dir) = target_dir else {
        return Ok(None);
    };

    if target_dir.trim().is_empty() {
        return Err(path_error(
            config_path,
            "build.target-dir is configured but empty",
        ));
    }

    resolve_config_relative_path(config_path, Path::new(target_dir)).map(Some)
}

fn read_config_to_string(config_path: &Path) -> Result<String, PathError> {
    let content = fs::read_to_string(config_path).map_err(|error| {
        path_error(config_path, format!("failed to read Cargo config: {error}"))
    })?;
    Ok(content)
}

fn resolve_config_relative_path(config_path: &Path, path: &Path) -> Result<PathBuf, PathError> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }

    let base_dir = config_base_dir(config_path)?;
    Ok(base_dir.join(path))
}

fn config_base_dir(config_path: &Path) -> Result<PathBuf, PathError> {
    let Some(config_dir) = config_path.parent() else {
        return Err(path_error(
            config_path,
            "failed to determine Cargo config directory",
        ));
    };

    let Some(base_dir) = config_dir.parent() else {
        return Err(path_error(
            config_path,
            "failed to determine Cargo config base directory",
        ));
    };

    Ok(base_dir.to_path_buf())
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::Mutex;
    use std::sync::OnceLock;
    use std::time::SystemTime;
    use std::time::UNIX_EPOCH;

    use super::read_configured_target_dir_from_config_dir;
    use super::resolve_default_scan_roots_from_global_config;
    use super::resolve_target_dir_from_configs;
    use crate::types::CargoProject;

    #[test]
    fn reads_target_dir_from_cargo_config() {
        let root = unique_test_dir("reads_target_dir_from_cargo_config");
        let cargo_dir = root.join(".cargo");
        fs::create_dir_all(&cargo_dir).expect("should create .cargo directory");
        fs::write(
            cargo_dir.join("config.toml"),
            "[build]\ntarget-dir = \"build-output\"\n",
        )
        .expect("should write Cargo config");

        let target_dir = read_configured_target_dir_from_config_dir(&cargo_dir)
            .expect("config should parse")
            .expect("target-dir should be present");

        assert_eq!(target_dir, root.join("build-output"));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn returns_none_when_target_dir_is_missing() {
        let root = unique_test_dir("returns_none_when_target_dir_is_missing");
        let cargo_dir = root.join(".cargo");
        fs::create_dir_all(&cargo_dir).expect("should create .cargo directory");
        fs::write(cargo_dir.join("config.toml"), "[build]\njobs = 4\n")
            .expect("should write Cargo config");

        let target_dir =
            read_configured_target_dir_from_config_dir(&cargo_dir).expect("config should parse");

        assert!(target_dir.is_none());

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn falls_back_to_cargo_home_when_project_has_no_target_dir() {
        let base = unique_test_dir("falls_back_to_cargo_home_when_project_has_no_target_dir");
        let project_root = base.join("project");
        let cargo_home = base.join("home").join(".cargo");

        fs::create_dir_all(project_root.join(".cargo")).expect("should create project .cargo");
        fs::create_dir_all(&cargo_home).expect("should create cargo home");
        fs::write(
            project_root.join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
        )
        .expect("should write Cargo.toml");
        fs::write(
            project_root.join(".cargo").join("config.toml"),
            "[build]\njobs = 4\n",
        )
        .expect("should write project config");
        fs::write(
            cargo_home.join("config.toml"),
            "[build]\ntarget-dir = \"shared-target\"\n",
        )
        .expect("should write global config");

        let project = CargoProject {
            root: project_root.clone(),
            manifest: project_root.join("Cargo.toml"),
        };

        let target_dir = resolve_target_dir_from_configs(&project, Some(&cargo_home))
            .expect("resolution should succeed")
            .expect("global target-dir should be used");

        assert_eq!(target_dir, base.join("home").join("shared-target"));

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn prefers_legacy_config_file_over_config_toml() {
        let root = unique_test_dir("prefers_legacy_config_file_over_config_toml");
        let cargo_dir = root.join(".cargo");
        fs::create_dir_all(&cargo_dir).expect("should create .cargo directory");
        fs::write(
            cargo_dir.join("config"),
            "[build]\ntarget-dir = \"legacy-output\"\n",
        )
        .expect("should write legacy config");
        fs::write(
            cargo_dir.join("config.toml"),
            "[build]\ntarget-dir = \"toml-output\"\n",
        )
        .expect("should write config.toml");

        let target_dir = read_configured_target_dir_from_config_dir(&cargo_dir)
            .expect("config should parse")
            .expect("target-dir should be present");

        assert_eq!(target_dir, root.join("legacy-output"));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn reads_default_roots_from_global_config_string() {
        let _guard = env_lock().lock().expect("env lock should not be poisoned");
        let cargo_home = unique_test_dir("reads_default_roots_from_global_config_string");
        fs::create_dir_all(&cargo_home).expect("should create cargo home directory");
        fs::write(
            cargo_home.join("config.toml"),
            "[cargo-clean-global]\nroots = \"/tmp/rust-projects\"\n",
        )
        .expect("should write global Cargo config");

        let original = std::env::var_os("CARGO_HOME");
        unsafe {
            std::env::set_var("CARGO_HOME", &cargo_home);
        }

        let roots = resolve_default_scan_roots_from_global_config().expect("config should parse");

        restore_cargo_home(original);
        let _ = fs::remove_dir_all(&cargo_home);

        assert_eq!(roots, vec![PathBuf::from("/tmp/rust-projects")]);
    }

    #[test]
    fn reads_default_roots_from_global_config_array() {
        let _guard = env_lock().lock().expect("env lock should not be poisoned");
        let cargo_home = unique_test_dir("reads_default_roots_from_global_config_array");
        fs::create_dir_all(&cargo_home).expect("should create cargo home directory");
        fs::write(
            cargo_home.join("config.toml"),
            "[cargo-clean-global]\nroots = [\"/tmp/a\", \"/tmp/b\"]\n",
        )
        .expect("should write global Cargo config");

        let original = std::env::var_os("CARGO_HOME");
        unsafe {
            std::env::set_var("CARGO_HOME", &cargo_home);
        }

        let roots = resolve_default_scan_roots_from_global_config().expect("config should parse");

        restore_cargo_home(original);
        let _ = fs::remove_dir_all(&cargo_home);

        assert_eq!(
            roots,
            vec![PathBuf::from("/tmp/a"), PathBuf::from("/tmp/b")]
        );
    }

    #[test]
    fn returns_empty_for_blank_string_root() {
        let _guard = env_lock().lock().expect("env lock should not be poisoned");
        let cargo_home = unique_test_dir("returns_empty_for_blank_string_root");
        fs::create_dir_all(&cargo_home).expect("should create cargo home directory");
        fs::write(
            cargo_home.join("config.toml"),
            "[cargo-clean-global]\nroots = \"   \"\n",
        )
        .expect("should write global Cargo config");

        let original = std::env::var_os("CARGO_HOME");
        unsafe {
            std::env::set_var("CARGO_HOME", &cargo_home);
        }

        let roots = resolve_default_scan_roots_from_global_config().expect("config should parse");

        restore_cargo_home(original);
        let _ = fs::remove_dir_all(&cargo_home);

        assert!(roots.is_empty());
    }

    #[test]
    fn rejects_invalid_root_types() {
        let _guard = env_lock().lock().expect("env lock should not be poisoned");
        let cargo_home = unique_test_dir("rejects_invalid_root_types");
        fs::create_dir_all(&cargo_home).expect("should create cargo home directory");
        fs::write(
            cargo_home.join("config.toml"),
            "[cargo-clean-global]\nroots = [\"/tmp/a\", 1]\n",
        )
        .expect("should write global Cargo config");

        let original = std::env::var_os("CARGO_HOME");
        unsafe {
            std::env::set_var("CARGO_HOME", &cargo_home);
        }

        let error =
            resolve_default_scan_roots_from_global_config().expect_err("config should be rejected");

        restore_cargo_home(original);
        let _ = fs::remove_dir_all(&cargo_home);

        assert!(
            error
                .message
                .contains("cargo-clean-global.roots must be a string or an array of strings")
        );
    }

    #[test]
    fn returns_empty_when_global_config_missing() {
        let _guard = env_lock().lock().expect("env lock should not be poisoned");
        let cargo_home = unique_test_dir("returns_empty_when_global_config_missing");
        fs::create_dir_all(&cargo_home).expect("should create cargo home directory");

        let original = std::env::var_os("CARGO_HOME");
        unsafe {
            std::env::set_var("CARGO_HOME", &cargo_home);
        }

        let roots = resolve_default_scan_roots_from_global_config()
            .expect("missing config should not fail");

        restore_cargo_home(original);
        let _ = fs::remove_dir_all(&cargo_home);

        assert!(roots.is_empty());
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

    fn unique_test_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos();
        std::env::temp_dir().join(format!("cargo-clean-global-{name}-{nanos}"))
    }
}
