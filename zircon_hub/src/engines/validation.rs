use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceEngineValidation {
    Valid,
    MissingRoot,
    MissingWorkspaceManifest,
    MissingBuildTool,
}

pub fn validate_source_engine(path: impl AsRef<Path>) -> SourceEngineValidation {
    let path = path.as_ref();
    if !path.is_dir() {
        return SourceEngineValidation::MissingRoot;
    }
    if !path.join("Cargo.toml").is_file() {
        return SourceEngineValidation::MissingWorkspaceManifest;
    }
    if !path.join("tools").join("zircon_build.py").is_file() {
        return SourceEngineValidation::MissingBuildTool;
    }
    SourceEngineValidation::Valid
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn source_engine_validation_requires_manifest_and_build_tool() {
        let root = std::env::temp_dir().join(format!(
            "zircon_hub_engine_validation_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("tools")).unwrap();

        assert_eq!(
            validate_source_engine(&root),
            SourceEngineValidation::MissingWorkspaceManifest
        );
        fs::write(root.join("Cargo.toml"), "[workspace]").unwrap();
        assert_eq!(
            validate_source_engine(&root),
            SourceEngineValidation::MissingBuildTool
        );
        fs::write(root.join("tools").join("zircon_build.py"), "").unwrap();
        assert_eq!(validate_source_engine(&root), SourceEngineValidation::Valid);

        let _ = fs::remove_dir_all(root);
    }
}
