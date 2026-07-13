use super::super::*;
use std::fs;
use std::path::Path;
use zircon_runtime::asset::AssetImportError;

pub(crate) fn stage_model_source(
    project: &ProjectManager,
    source: &Path,
) -> Result<(ResourceLocator, String), String> {
    match project.project_uri_for_source_path(source) {
        Ok(uri) => return Ok((uri, source.to_string_lossy().into_owned())),
        Err(AssetImportError::SourceOutsideProjectAssetRoots { .. }) => {}
        Err(error) => return Err(error.to_string()),
    }

    let extension = source
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if extension == "gltf" {
        return Err(
            "External .gltf import is not supported yet; copy the model folder into a configured project asset root or use .glb".to_string(),
        );
    }

    let destination = project
        .primary_project_asset_root()
        .map_err(|error| error.to_string())?
        .join("models")
        .join(
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
