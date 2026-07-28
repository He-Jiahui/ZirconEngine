use std::collections::HashSet;

use zircon_runtime_interface::ui::surface::UiTextRenderMode;

use super::super::render::{
    ScreenSpaceUiTextBatch, text_advances::refresh_screen_space_text_batch_glyphs,
};
use super::font_assets::{UiFontAssetCache, effective_text_render_mode, ensure_font_asset_record};
use crate::asset::ProjectAssetManager;
use crate::text::{TextLayoutFallbackReport, TextRenderState};

#[derive(Clone, Debug, Default)]
pub(super) struct ResolvedScreenSpaceUiTextBatches {
    pub(super) native_texts: Vec<ScreenSpaceUiTextBatch>,
    pub(super) sdf_texts: Vec<ScreenSpaceUiTextBatch>,
    font_faces_changed: bool,
}

impl ResolvedScreenSpaceUiTextBatches {
    pub(super) fn from_explicit_batches(
        native_texts: &[ScreenSpaceUiTextBatch],
        sdf_texts: &[ScreenSpaceUiTextBatch],
    ) -> Self {
        Self {
            native_texts: native_texts.to_vec(),
            sdf_texts: sdf_texts.to_vec(),
            font_faces_changed: false,
        }
    }

    pub(super) fn push_resolved_auto_text(
        &mut self,
        text: ScreenSpaceUiTextBatch,
        resolved_mode: UiTextRenderMode,
    ) {
        match resolved_mode {
            UiTextRenderMode::Auto | UiTextRenderMode::Native => self.native_texts.push(text),
            UiTextRenderMode::Sdf | UiTextRenderMode::Msdf | UiTextRenderMode::Mtsdf => {
                self.sdf_texts.push(text)
            }
        }
    }

    pub(super) fn native_texts(&self) -> &[ScreenSpaceUiTextBatch] {
        &self.native_texts
    }

    pub(super) fn sdf_texts(&self) -> &[ScreenSpaceUiTextBatch] {
        &self.sdf_texts
    }

    pub(super) fn font_faces_changed(&self) -> bool {
        self.font_faces_changed
    }

    pub(super) fn layout_fallback_report(&self) -> TextLayoutFallbackReport {
        let mut report = TextLayoutFallbackReport::default();
        for error in self
            .native_texts
            .iter()
            .chain(self.sdf_texts.iter())
            .filter_map(|text| text.layout_error.as_ref())
        {
            report.record(error);
        }
        report
    }

    fn refresh_shaping_after_font_load(&mut self) {
        for text in self
            .native_texts
            .iter_mut()
            .chain(self.sdf_texts.iter_mut())
        {
            refresh_screen_space_text_batch_glyphs(text);
        }
    }
}

pub(super) fn resolve_text_batches(
    text_state: &mut TextRenderState,
    font_assets: &mut UiFontAssetCache,
    asset_manager: &ProjectAssetManager,
    auto_texts: &[ScreenSpaceUiTextBatch],
    native_texts: &[ScreenSpaceUiTextBatch],
    sdf_texts: &[ScreenSpaceUiTextBatch],
) -> ResolvedScreenSpaceUiTextBatches {
    let mut loaded_assets = HashSet::new();
    let mut shaping_changed = text_state.refresh_shared_font_database();
    let mut font_faces_changed = shaping_changed;
    for text in auto_texts
        .iter()
        .chain(native_texts.iter())
        .chain(sdf_texts.iter())
    {
        let asset = text
            .font
            .as_deref()
            .filter(|asset| !asset.trim().is_empty())
            .unwrap_or(super::DEFAULT_FONT_ASSET);
        for asset in
            std::iter::once(asset).chain(text.style.code.then_some(super::DEFAULT_FONT_ASSET))
        {
            if !loaded_assets.insert(asset) {
                continue;
            }
            let ensured = ensure_font_asset_record(text_state, font_assets, asset_manager, asset);
            shaping_changed |= ensured.faces_changed;
            font_faces_changed |= ensured.faces_changed;
        }
    }

    let mut resolved =
        ResolvedScreenSpaceUiTextBatches::from_explicit_batches(native_texts, sdf_texts);
    resolved.font_faces_changed = font_faces_changed;
    for text in auto_texts {
        let asset = text
            .font
            .as_deref()
            .filter(|asset| !asset.trim().is_empty())
            .unwrap_or(super::DEFAULT_FONT_ASSET);
        let font_asset = font_assets
            .get(asset)
            .and_then(|entry| entry.loaded_asset());
        resolved.push_resolved_auto_text(
            text.clone(),
            effective_text_render_mode(UiTextRenderMode::Auto, font_asset),
        );
    }

    if shaping_changed {
        resolved.refresh_shaping_after_font_load();
    }
    resolved
}
