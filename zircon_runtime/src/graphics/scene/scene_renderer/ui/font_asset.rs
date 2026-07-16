use std::path::PathBuf;

use crate::asset::{AssetUuid, FontAsset, ProjectAssetManager};
use crate::text::font::load_text_font_source;
use zircon_runtime_interface::ui::surface::UiTextRenderMode;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LoadedUiFontManifest {
    pub(crate) source_path: PathBuf,
    pub(crate) asset: Option<FontAsset>,
    pub(crate) family: Option<String>,
    pub(crate) render_mode: Option<UiTextRenderMode>,
    pub(crate) face_index: u32,
    pub(crate) asset_uuid: Option<AssetUuid>,
}

pub(crate) fn load_ui_font_manifest_with_asset_manager(
    asset_ref: &str,
    asset_manager: Option<&ProjectAssetManager>,
) -> Option<LoadedUiFontManifest> {
    let source = load_text_font_source(asset_ref, asset_manager)?;
    let render_mode = source
        .asset
        .as_ref()
        .and_then(effective_ui_font_render_mode);
    Some(LoadedUiFontManifest {
        source_path: source.source_path,
        asset: source.asset,
        family: source.family,
        render_mode,
        face_index: source.face_index,
        asset_uuid: source.asset_uuid,
    })
}

fn effective_ui_font_render_mode(manifest: &FontAsset) -> Option<UiTextRenderMode> {
    manifest.effective_render_mode()
}

#[cfg(test)]
mod tests;
