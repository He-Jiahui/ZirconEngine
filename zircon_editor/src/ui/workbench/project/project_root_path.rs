use std::ffi::OsStr;
use std::path::PathBuf;

use zircon_runtime::asset::project::ProjectPaths;
use zircon_runtime::scene::world::SceneProjectError;

pub(crate) fn project_root_path(
    path: impl AsRef<std::path::Path>,
) -> Result<PathBuf, SceneProjectError> {
    let candidate = path.as_ref();
    let root = if candidate
        .file_name()
        .is_some_and(|name| name == OsStr::new("zircon-project.toml"))
    {
        candidate.parent().unwrap_or(candidate)
    } else {
        candidate
    };
    Ok(ProjectPaths::resolve_existing_path(root)?)
}

#[cfg(test)]
mod tests {
    use zircon_runtime::scene::world::SceneProjectError;

    use super::project_root_path;

    #[cfg(windows)]
    #[test]
    fn project_root_path_rejects_drive_relative_paths() {
        let result = project_root_path(r"C:ambiguous-project-root");

        assert!(matches!(
            result,
            Err(SceneProjectError::Io(error)) if error.kind() == std::io::ErrorKind::InvalidInput
        ));
    }
}
