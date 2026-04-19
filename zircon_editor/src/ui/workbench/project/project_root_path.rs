use std::ffi::OsStr;
use std::path::PathBuf;

use zircon_scene::world::SceneProjectError;

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
    if root.is_absolute() {
        Ok(root.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(root))
    }
}
