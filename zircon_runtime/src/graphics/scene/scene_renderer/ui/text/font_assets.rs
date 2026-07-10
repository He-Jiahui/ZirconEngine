use std::collections::HashMap;

use glyphon::FontSystem;
use zircon_runtime_interface::ui::surface::UiTextRenderMode;

use super::super::font_asset::{load_ui_font_manifest_with_asset_manager, LoadedUiFontManifest};
use super::DEFAULT_FONT_ASSET;
use crate::asset::ProjectAssetManager;
use crate::graphics::text::font::publish_shared_font_database;
use crate::graphics::text::font::FontDatabase;
use crate::ui::text::shaper::resolve_text_render_mode;

#[derive(Clone, Debug, Default)]
pub(super) struct LoadedUiFontAsset {
    pub(super) family: Option<String>,
    pub(super) render_mode: Option<UiTextRenderMode>,
}

pub(super) fn resolve_font_asset_record<'a>(
    font_system: &mut FontSystem,
    font_database: &mut FontDatabase,
    font_assets: &'a mut HashMap<String, LoadedUiFontAsset>,
    asset_manager: &ProjectAssetManager,
    font_asset: Option<&str>,
) -> Option<&'a LoadedUiFontAsset> {
    let asset = font_asset
        .filter(|asset| !asset.trim().is_empty())
        .unwrap_or(DEFAULT_FONT_ASSET);
    Some(ensure_font_asset_record(
        font_system,
        font_database,
        font_assets,
        asset_manager,
        asset,
    ))
}

pub(super) fn effective_text_render_mode(
    requested_mode: UiTextRenderMode,
    font_asset: Option<&LoadedUiFontAsset>,
) -> UiTextRenderMode {
    resolve_text_render_mode(
        requested_mode,
        font_asset.and_then(|asset| asset.render_mode),
    )
}

pub(super) fn load_font_asset_record(
    font_system: &mut FontSystem,
    font_database: &mut FontDatabase,
    asset_ref: &str,
    asset_manager: &ProjectAssetManager,
) -> Option<LoadedUiFontAsset> {
    let manifest = load_ui_font_manifest_with_asset_manager(asset_ref, Some(asset_manager))?;
    let face = register_loaded_font_manifest(font_database, &manifest)?;
    let _ = font_database.load_face_into_font_system(face, font_system);
    publish_shared_font_database(font_database);
    Some(LoadedUiFontAsset {
        family: manifest.family,
        render_mode: manifest.render_mode,
    })
}

fn register_loaded_font_manifest(
    font_database: &mut FontDatabase,
    manifest: &LoadedUiFontManifest,
) -> Option<crate::core::framework::render::FontFaceId> {
    if let Some(asset) = &manifest.asset {
        return font_database
            .register_font_asset(asset, &manifest.source_path)
            .ok()
            .and_then(|faces| faces.first().copied());
    }

    font_database
        .register_font_file(
            &manifest.source_path,
            manifest.family.as_deref(),
            manifest.face_index,
        )
        .ok()
}

pub(super) fn ensure_font_asset_record<'a>(
    font_system: &mut FontSystem,
    font_database: &mut FontDatabase,
    font_assets: &'a mut HashMap<String, LoadedUiFontAsset>,
    asset_manager: &ProjectAssetManager,
    asset_ref: &str,
) -> &'a LoadedUiFontAsset {
    font_assets.entry(asset_ref.to_string()).or_insert_with(|| {
        load_font_asset_record(font_system, font_database, asset_ref, asset_manager)
            .unwrap_or_default()
    })
}
