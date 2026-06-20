use super::super::*;
use std::fs;
use std::path::Path;

pub(crate) fn stage_model_source(
    paths: &ProjectPaths,
    source: &Path,
) -> Result<(ResourceLocator, String), String> {
    if let Ok(relative) = source.strip_prefix(paths.assets_root()) {
        let uri = asset_uri_from_relative_path(relative)?;
        return Ok((uri, source.to_string_lossy().into_owned()));
    }

    let extension = source
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if extension == "gltf" {
        return Err(
            "External .gltf import is not supported yet; copy the model folder into Project/assets or use .glb".to_string(),
        );
    }

    let destination = paths.assets_root().join("models").join(
        source
            .file_name()
            .ok_or_else(|| format!("model path has no file name: {}", source.display()))?,
    );
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    if source != destination {
        fs::copy(source, &destination).map_err(|error| {
            format!(
                "failed to copy model {} into project assets: {error}",
                source.display()
            )
        })?;
        if extension == "obj" {
            let sibling_mtl = source.with_extension("mtl");
            if sibling_mtl.exists() {
                let _ = fs::copy(sibling_mtl, destination.with_extension("mtl"));
            }
        }
    }

    Ok((
        asset_uri_from_relative_path(
            Path::new("models").join(destination.file_name().ok_or_else(|| {
                format!("model path has no file name: {}", destination.display())
            })?),
        )?,
        destination.to_string_lossy().into_owned(),
    ))
}

pub(super) fn asset_uri_from_relative_path(
    relative: impl AsRef<Path>,
) -> Result<ResourceLocator, String> {
    let normalized = relative
        .as_ref()
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/");
    ResourceLocator::parse(&format!("res://{normalized}")).map_err(|error| error.to_string())
}
