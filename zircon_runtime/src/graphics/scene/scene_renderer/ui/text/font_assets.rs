use std::collections::HashMap;

use zircon_runtime_interface::ui::surface::{resolve_ui_text_render_mode, UiTextRenderMode};

use super::super::font_asset::load_ui_font_manifest_with_asset_manager;
use crate::asset::ProjectAssetManager;
use crate::text::{CompositeFontDescriptor, TextRenderState};

#[derive(Clone, Debug, Default)]
pub(super) struct LoadedUiFontAsset {
    pub(super) family: Option<String>,
    pub(super) render_mode: Option<UiTextRenderMode>,
    pub(super) composite_font: Option<CompositeFontDescriptor>,
}

pub(super) struct EnsuredUiFontAsset<'a> {
    pub(super) record: Option<&'a LoadedUiFontAsset>,
    pub(super) loaded: bool,
    pub(super) faces_changed: bool,
}

pub(super) fn effective_text_render_mode(
    requested_mode: UiTextRenderMode,
    font_asset: Option<&LoadedUiFontAsset>,
) -> UiTextRenderMode {
    resolve_ui_text_render_mode(
        requested_mode,
        font_asset.and_then(|asset| asset.render_mode),
    )
}

pub(super) fn load_font_asset_record(
    text_state: &mut TextRenderState,
    asset_ref: &str,
    asset_manager: &ProjectAssetManager,
) -> Option<LoadedUiFontAsset> {
    let manifest = load_ui_font_manifest_with_asset_manager(asset_ref, Some(asset_manager))?;
    text_state
        .register_font_source(
            &manifest.source_path,
            manifest.asset.as_ref(),
            manifest.family.as_deref(),
            manifest.face_index,
        )
        .then_some(())?;
    let composite_font = manifest
        .asset
        .as_ref()
        .and_then(|asset| asset.composite_font.clone());
    Some(LoadedUiFontAsset {
        family: manifest.family,
        render_mode: manifest.render_mode,
        composite_font,
    })
}

pub(super) fn ensure_font_asset_record<'a>(
    text_state: &mut TextRenderState,
    font_assets: &'a mut HashMap<String, LoadedUiFontAsset>,
    asset_manager: &ProjectAssetManager,
    asset_ref: &str,
) -> EnsuredUiFontAsset<'a> {
    let face_count_before_load = text_state.face_count();
    let mut loaded = false;
    if !font_assets.contains_key(asset_ref) {
        if let Some(record) = load_font_asset_record(text_state, asset_ref, asset_manager) {
            font_assets.insert(asset_ref.to_string(), record);
            text_state.publish_font_database();
            loaded = true;
        }
    }

    EnsuredUiFontAsset {
        record: font_assets.get(asset_ref),
        loaded,
        faces_changed: text_state.face_count() != face_count_before_load,
    }
}
