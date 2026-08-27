use std::path::{Component, Path};

use crate::error::Error;

const MAX_LEN: usize = 64;

pub fn validate_project_name(name: &str) -> Result<(), Error> {
    if name.is_empty() {
        return Err(Error::InvalidName("name must not be empty".into()));
    }
    if name.len() > MAX_LEN {
        return Err(Error::InvalidName(format!(
            "name must be at most {MAX_LEN} characters"
        )));
    }
    if name.contains('\0') {
        return Err(Error::InvalidName(
            "name must not contain null bytes".into(),
        ));
    }
    if name.contains('/') || name.contains('\\') {
        return Err(Error::InvalidName(
            "name must not contain path separators".into(),
        ));
    }
    if name == "." || name == ".." || name.contains("..") {
        return Err(Error::InvalidName(
            "name must not contain path traversal".into(),
        ));
    }

    let first = name.as_bytes()[0];
    if !first.is_ascii_lowercase() {
        return Err(Error::InvalidName(
            "name must start with a lowercase letter".into(),
        ));
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(Error::InvalidName(
            "name must be kebab-case (lowercase letters, digits, hyphens)".into(),
        ));
    }
    if name.starts_with('-') || name.ends_with('-') || name.contains("--") {
        return Err(Error::InvalidName(
            "name must be kebab-case without leading, trailing, or doubled hyphens".into(),
        ));
    }
    Ok(())
}

pub fn crate_name(project_name: &str) -> String {
    project_name.replace('-', "_")
}

pub fn validate_output_path(path: &Path) -> Result<(), Error> {
    if path.as_os_str().is_empty() {
        return Err(Error::InvalidOutput("output path must not be empty".into()));
    }
    if path
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(Error::InvalidOutput(
            "output path must not contain ..".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn accepts_kebab_case() {
        assert!(validate_project_name("payment-service").is_ok());
        assert!(validate_project_name("api").is_ok());
        assert!(validate_project_name("svc2").is_ok());
    }

    #[test]
    fn rejects_empty_and_traversal() {
        assert!(validate_project_name("").is_err());
        assert!(validate_project_name("..").is_err());
        assert!(validate_project_name("../evil").is_err());
        assert!(validate_project_name("foo/bar").is_err());
        assert!(validate_project_name("foo\\bar").is_err());
        assert!(validate_project_name("foo..bar").is_err());
    }

    #[test]
    fn rejects_invalid_kebab() {
        assert!(validate_project_name("Payment").is_err());
        assert!(validate_project_name("-lead").is_err());
        assert!(validate_project_name("trail-").is_err());
        assert!(validate_project_name("doub--le").is_err());
        assert!(validate_project_name("has_underscore").is_err());
    }

    #[test]
    fn crate_name_replaces_hyphens() {
        assert_eq!(crate_name("payment-service"), "payment_service");
    }

    #[test]
    fn rejects_parent_dir_in_output() {
        assert!(validate_output_path(Path::new("../out")).is_err());
        assert!(validate_output_path(Path::new("out/../other")).is_err());
        assert!(validate_output_path(Path::new("out")).is_ok());
    }
}
