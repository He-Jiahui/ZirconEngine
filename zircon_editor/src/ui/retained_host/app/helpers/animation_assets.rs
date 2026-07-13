mod clip;
mod paths;
mod skeleton;

#[cfg(test)]
#[path = "animation_assets/tests.rs"]
mod tests;

use std::path::Path;

use super::asset_uri_from_relative_path;
use clip::derive_clip_asset;
use paths::{sanitize_animation_asset_segment, sibling_relative_path, write_animation_asset_bytes};
use skeleton::{derive_skeleton_asset, node_parent_indices};
use zircon_runtime::asset::project::ProjectManager;
use zircon_runtime_interface::resource::ResourceLocator;

pub(crate) fn derive_animation_assets_from_model_source(
    project: &ProjectManager,
    model_source: &Path,
) -> Result<Vec<ResourceLocator>, String> {
    let extension = model_source
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if extension != "gltf" && extension != "glb" {
        return Ok(Vec::new());
    }

    let assets_root = project
        .project_asset_root_for_source_path(model_source)
        .map_err(|error| error.to_string())?;
    let relative_model_path = model_source
        .strip_prefix(assets_root)
        .map_err(|error| error.to_string())?;
    let base_name = relative_model_path
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("model source has no file stem: {}", model_source.display()))?;

    let (document, buffers, _) = gltf::import(model_source).map_err(|error| {
        format!(
            "parse gltf animation data from {}: {error}",
            model_source.display()
        )
    })?;
    let Some(skin) = document.skins().next() else {
        return Ok(Vec::new());
    };

    let parent_indices = node_parent_indices(&document);
    let skeleton_file_name = format!("{base_name}.skeleton.zranim");
    let skeleton_relative_path =
        sibling_relative_path(relative_model_path, Path::new(&skeleton_file_name));
    let skeleton_locator = asset_uri_from_relative_path(&skeleton_relative_path)?;
    let derived_skeleton =
        derive_skeleton_asset(&skin, &parent_indices, &skeleton_locator, base_name)?;
    write_animation_asset_bytes(
        &assets_root.join(&skeleton_relative_path),
        derived_skeleton
            .asset
            .to_bytes()
            .map_err(|error| error.to_string())?,
    )?;

    let mut generated = vec![skeleton_locator.clone()];
    for (animation_index, animation) in document.animations().enumerate() {
        let clip_segment = sanitize_animation_asset_segment(animation.name(), animation_index);
        let clip_file_name = format!("{base_name}.{clip_segment}.clip.zranim");
        let clip_relative_path =
            sibling_relative_path(relative_model_path, Path::new(&clip_file_name));
        let clip_locator = asset_uri_from_relative_path(&clip_relative_path)?;
        let clip_asset =
            derive_clip_asset(&animation, &buffers, &derived_skeleton, &skeleton_locator)?;
        write_animation_asset_bytes(
            &assets_root.join(&clip_relative_path),
            clip_asset.to_bytes().map_err(|error| error.to_string())?,
        )?;
        generated.push(clip_locator);
    }

    generated.sort_by_key(|locator| locator.to_string());
    Ok(generated)
}
