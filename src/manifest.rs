use std::fs;
use std::io;
use std::path::Path;

pub(crate) fn looks_like_cargo_manifest(path: &Path) -> io::Result<bool> {
    let content = fs::read_to_string(path)?;
    Ok(content.lines().any(is_cargo_manifest_section))
}

pub(crate) fn is_cargo_manifest_section(line: &str) -> bool {
    let trimmed = line.split('#').next().unwrap_or_default().trim();
    trimmed == "[package]" || trimmed == "[workspace]"
}

#[cfg(test)]
mod tests {
    use super::is_cargo_manifest_section;

    #[test]
    fn detects_package_section() {
        assert!(is_cargo_manifest_section("[package]"));
        assert!(is_cargo_manifest_section(" [workspace] "));
        assert!(is_cargo_manifest_section("[package] # comment"));
        assert!(!is_cargo_manifest_section("[dependencies]"));
        assert!(!is_cargo_manifest_section("# [package]"));
    }
}
