use std::collections::HashMap;
use std::sync::Arc;

use glyphon::{
    Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping, Style,
    SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer, Viewport, Weight, Wrap,
};

use super::font_asset::load_ui_font_manifest_with_asset_manager;
use super::render::ScreenSpaceUiTextBatch;
use crate::asset::ProjectAssetManager;
use glyphon::cosmic_text::Align;
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiTextAlign, UiTextRenderMode, UiTextRunPaintStyle, UiTextWrap,
};

use super::sdf_atlas::ScreenSpaceUiSdfAtlas;
use super::sdf_render::ScreenSpaceUiSdfRenderer;

const DEFAULT_FONT_ASSET: &str = "res://fonts/default.font.toml";

pub(super) struct ScreenSpaceUiTextSystem {
    asset_manager: Arc<ProjectAssetManager>,
    font_system: FontSystem,
    swash_cache: SwashCache,
    font_assets: HashMap<String, LoadedUiFontAsset>,
    native: ScreenSpaceUiTextBackend,
    sdf_atlas: ScreenSpaceUiSdfAtlas,
    sdf_renderer: ScreenSpaceUiSdfRenderer,
}

struct ScreenSpaceUiTextBackend {
    _cache: Cache,
    viewport: Viewport,
    atlas: TextAtlas,
    renderer: TextRenderer,
}

#[derive(Clone, Debug, Default)]
struct ResolvedScreenSpaceUiTextBatches {
    native_texts: Vec<ScreenSpaceUiTextBatch>,
    sdf_texts: Vec<ScreenSpaceUiTextBatch>,
}

impl ResolvedScreenSpaceUiTextBatches {
    fn from_explicit_batches(
        native_texts: &[ScreenSpaceUiTextBatch],
        sdf_texts: &[ScreenSpaceUiTextBatch],
    ) -> Self {
        Self {
            native_texts: native_texts.to_vec(),
            sdf_texts: sdf_texts.to_vec(),
        }
    }

    fn push_resolved_auto_text(
        &mut self,
        text: ScreenSpaceUiTextBatch,
        resolved_mode: UiTextRenderMode,
    ) {
        match resolved_mode {
            UiTextRenderMode::Auto | UiTextRenderMode::Native => self.native_texts.push(text),
            UiTextRenderMode::Sdf => self.sdf_texts.push(text),
        }
    }

    fn native_texts(&self) -> &[ScreenSpaceUiTextBatch] {
        &self.native_texts
    }

    fn sdf_texts(&self) -> &[ScreenSpaceUiTextBatch] {
        &self.sdf_texts
    }

    fn sdf_atlas_texts(&self) -> &[ScreenSpaceUiTextBatch] {
        &self.sdf_texts
    }
}

#[derive(Clone, Debug, Default)]
struct LoadedUiFontAsset {
    family: Option<String>,
    render_mode: Option<UiTextRenderMode>,
}

impl ScreenSpaceUiTextSystem {
    pub(super) fn new(
        asset_manager: Arc<ProjectAssetManager>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_format: wgpu::TextureFormat,
    ) -> Self {
        let mut font_system = FontSystem::new();
        let mut font_assets = HashMap::new();
        let default_font =
            load_font_asset_record(&mut font_system, DEFAULT_FONT_ASSET, &asset_manager);
        if let Some(record) = default_font.as_ref() {
            if let Some(family) = record.family.as_deref() {
                font_system
                    .db_mut()
                    .set_sans_serif_family(family.to_string());
                font_system
                    .db_mut()
                    .set_monospace_family(family.to_string());
            }
            font_assets.insert(DEFAULT_FONT_ASSET.to_string(), record.clone());
        }

        Self {
            asset_manager,
            font_system,
            swash_cache: SwashCache::new(),
            font_assets,
            native: ScreenSpaceUiTextBackend::new(device, queue, target_format),
            sdf_atlas: ScreenSpaceUiSdfAtlas::new(),
            sdf_renderer: ScreenSpaceUiSdfRenderer::new(device, target_format),
        }
    }

    pub(super) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        viewport_size: crate::core::math::UVec2,
        auto_texts: &[ScreenSpaceUiTextBatch],
        native_texts: &[ScreenSpaceUiTextBatch],
        sdf_texts: &[ScreenSpaceUiTextBatch],
    ) {
        let resolved_texts = resolve_text_batches(
            &mut self.font_system,
            &mut self.font_assets,
            self.asset_manager.as_ref(),
            auto_texts,
            native_texts,
            sdf_texts,
        );
        self.sdf_atlas.prepare(resolved_texts.sdf_atlas_texts());
        self.sdf_renderer.prepare(
            device,
            queue,
            viewport_size,
            resolved_texts.sdf_texts(),
            self.sdf_atlas.plan(),
            self.asset_manager.as_ref(),
        );
        self.native.prepare(
            device,
            queue,
            viewport_size,
            resolved_texts.native_texts(),
            &mut self.font_system,
            &mut self.swash_cache,
            &mut self.font_assets,
            self.asset_manager.as_ref(),
        );
    }

    pub(super) fn render<'pass>(&'pass mut self, pass: &mut wgpu::RenderPass<'pass>) {
        let _ = self
            .native
            .renderer
            .render(&self.native.atlas, &self.native.viewport, pass);
        self.sdf_renderer.render(pass);
    }
}

impl ScreenSpaceUiTextBackend {
    fn new(device: &wgpu::Device, queue: &wgpu::Queue, target_format: wgpu::TextureFormat) -> Self {
        let cache = Cache::new(device);
        let viewport = Viewport::new(device, &cache);
        let mut atlas = TextAtlas::new(device, queue, &cache, target_format);
        let renderer =
            TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);

        Self {
            _cache: cache,
            viewport,
            atlas,
            renderer,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        viewport_size: crate::core::math::UVec2,
        texts: &[ScreenSpaceUiTextBatch],
        font_system: &mut FontSystem,
        swash_cache: &mut SwashCache,
        font_assets: &mut HashMap<String, LoadedUiFontAsset>,
        asset_manager: &ProjectAssetManager,
    ) {
        self.viewport.update(
            queue,
            Resolution {
                width: viewport_size.x.max(1),
                height: viewport_size.y.max(1),
            },
        );

        if texts.is_empty() {
            self.atlas.trim();
            return;
        }

        let mut buffers = Vec::with_capacity(texts.len());
        for text in texts {
            let family_name = resolve_family_name(
                font_system,
                font_assets,
                asset_manager,
                text.font.as_deref(),
                text.font_family.as_deref(),
            );
            let attrs = text_attrs(family_name.as_deref(), text.style);
            let mut buffer =
                Buffer::new(font_system, Metrics::new(text.font_size, text.line_height));
            buffer.set_size(
                font_system,
                Some(text.frame.width.max(1.0)),
                Some(text.frame.height.max(1.0)),
            );
            buffer.set_wrap(
                font_system,
                match text.wrap {
                    UiTextWrap::None => Wrap::None,
                    UiTextWrap::Word => Wrap::Word,
                    UiTextWrap::Glyph => Wrap::Glyph,
                },
            );
            buffer.set_text(
                font_system,
                &text.text,
                &attrs,
                Shaping::Advanced,
                Some(match text.text_align {
                    UiTextAlign::Left => Align::Left,
                    UiTextAlign::Center => Align::Center,
                    UiTextAlign::Right => Align::Right,
                }),
            );
            buffer.shape_until_scroll(font_system, false);
            buffers.push(buffer);
        }

        let text_areas = texts
            .iter()
            .zip(buffers.iter())
            .map(|(text, buffer)| TextArea {
                buffer,
                left: text.frame.x,
                top: text.frame.y,
                scale: 1.0,
                bounds: text_bounds(viewport_size, text),
                default_color: pack_color(text.color),
                custom_glyphs: &[],
            })
            .collect::<Vec<_>>();

        let _ = self.renderer.prepare(
            device,
            queue,
            font_system,
            &mut self.atlas,
            &self.viewport,
            text_areas,
            swash_cache,
        );
    }
}

fn text_attrs<'a>(family_name: Option<&'a str>, style: UiTextRunPaintStyle) -> Attrs<'a> {
    let mut attrs = if style.code {
        Attrs::new().family(Family::Monospace)
    } else {
        family_name
            .map(|family| Attrs::new().family(Family::Name(family)))
            .unwrap_or_else(Attrs::new)
    };
    if style.strong {
        attrs = attrs.weight(Weight::BOLD);
    }
    if style.emphasis {
        attrs = attrs.style(Style::Italic);
    }
    attrs
}

fn resolve_family_name(
    font_system: &mut FontSystem,
    font_assets: &mut HashMap<String, LoadedUiFontAsset>,
    asset_manager: &ProjectAssetManager,
    font_asset: Option<&str>,
    preferred_family: Option<&str>,
) -> Option<String> {
    if let Some(family) = preferred_family.filter(|family| !family.trim().is_empty()) {
        if let Some(asset) = font_asset.filter(|asset| !asset.trim().is_empty()) {
            ensure_font_asset_record(font_system, font_assets, asset_manager, asset);
        }
        return Some(family.to_string());
    }

    let asset = font_asset
        .filter(|asset| !asset.trim().is_empty())
        .unwrap_or(DEFAULT_FONT_ASSET);
    ensure_font_asset_record(font_system, font_assets, asset_manager, asset)
        .family
        .clone()
}

fn resolve_text_batches(
    font_system: &mut FontSystem,
    font_assets: &mut HashMap<String, LoadedUiFontAsset>,
    asset_manager: &ProjectAssetManager,
    auto_texts: &[ScreenSpaceUiTextBatch],
    native_texts: &[ScreenSpaceUiTextBatch],
    sdf_texts: &[ScreenSpaceUiTextBatch],
) -> ResolvedScreenSpaceUiTextBatches {
    let mut resolved =
        ResolvedScreenSpaceUiTextBatches::from_explicit_batches(native_texts, sdf_texts);

    for text in auto_texts {
        let font_asset = resolve_font_asset_record(
            font_system,
            font_assets,
            asset_manager,
            text.font.as_deref(),
        );
        resolved.push_resolved_auto_text(
            text.clone(),
            effective_text_render_mode(UiTextRenderMode::Auto, font_asset),
        );
    }

    resolved
}

fn resolve_font_asset_record<'a>(
    font_system: &mut FontSystem,
    font_assets: &'a mut HashMap<String, LoadedUiFontAsset>,
    asset_manager: &ProjectAssetManager,
    font_asset: Option<&str>,
) -> Option<&'a LoadedUiFontAsset> {
    let asset = font_asset
        .filter(|asset| !asset.trim().is_empty())
        .unwrap_or(DEFAULT_FONT_ASSET);
    Some(ensure_font_asset_record(
        font_system,
        font_assets,
        asset_manager,
        asset,
    ))
}

fn effective_text_render_mode(
    requested_mode: UiTextRenderMode,
    font_asset: Option<&LoadedUiFontAsset>,
) -> UiTextRenderMode {
    match requested_mode {
        UiTextRenderMode::Native => UiTextRenderMode::Native,
        UiTextRenderMode::Sdf => UiTextRenderMode::Sdf,
        UiTextRenderMode::Auto => font_asset
            .and_then(|asset| asset.render_mode)
            .filter(|mode| !matches!(mode, UiTextRenderMode::Auto))
            .unwrap_or(UiTextRenderMode::Native),
    }
}

fn load_font_asset_record(
    font_system: &mut FontSystem,
    asset_ref: &str,
    asset_manager: &ProjectAssetManager,
) -> Option<LoadedUiFontAsset> {
    let manifest = load_ui_font_manifest_with_asset_manager(asset_ref, Some(asset_manager))?;
    let _ = font_system.db_mut().load_font_file(manifest.source_path);
    Some(LoadedUiFontAsset {
        family: manifest.family,
        render_mode: manifest.render_mode,
    })
}

fn ensure_font_asset_record<'a>(
    font_system: &mut FontSystem,
    font_assets: &'a mut HashMap<String, LoadedUiFontAsset>,
    asset_manager: &ProjectAssetManager,
    asset_ref: &str,
) -> &'a LoadedUiFontAsset {
    font_assets.entry(asset_ref.to_string()).or_insert_with(|| {
        load_font_asset_record(font_system, asset_ref, asset_manager).unwrap_or_default()
    })
}

fn text_bounds(
    viewport_size: crate::core::math::UVec2,
    text: &ScreenSpaceUiTextBatch,
) -> TextBounds {
    let clip = text
        .clip_frame
        .unwrap_or_else(|| UiFrame::new(0.0, 0.0, viewport_size.x as f32, viewport_size.y as f32));
    let clip = clip
        .intersection(UiFrame::new(
            0.0,
            0.0,
            viewport_size.x as f32,
            viewport_size.y as f32,
        ))
        .unwrap_or_default();
    TextBounds {
        left: clip.x.max(0.0).floor() as i32,
        top: clip.y.max(0.0).floor() as i32,
        right: clip.right().max(0.0).ceil() as i32,
        bottom: clip.bottom().max(0.0).ceil() as i32,
    }
}

fn pack_color(color: [f32; 4]) -> Color {
    Color::rgba(
        (color[0].clamp(0.0, 1.0) * 255.0) as u8,
        (color[1].clamp(0.0, 1.0) * 255.0) as u8,
        (color[2].clamp(0.0, 1.0) * 255.0) as u8,
        (color[3].clamp(0.0, 1.0) * 255.0) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_backend_routing_keeps_explicit_native_out_of_sdf_atlas_batches() {
        let native = text_batch("Normal", UiTextRenderMode::Native);
        let sdf = text_batch("Signed", UiTextRenderMode::Sdf);

        let routed = ResolvedScreenSpaceUiTextBatches::from_explicit_batches(&[native], &[sdf]);

        assert_eq!(routed.native_texts().len(), 1);
        assert_eq!(routed.native_texts()[0].text, "Normal");
        assert_eq!(routed.sdf_texts().len(), 1);
        assert_eq!(routed.sdf_texts()[0].text, "Signed");
        assert_eq!(routed.sdf_atlas_texts().len(), 1);
        assert_eq!(routed.sdf_atlas_texts()[0].text, "Signed");
    }

    #[test]
    fn text_backend_routing_respects_auto_font_mode_without_crossing_backends() {
        let mut routed = ResolvedScreenSpaceUiTextBatches::default();

        routed.push_resolved_auto_text(
            text_batch("NormalAuto", UiTextRenderMode::Auto),
            UiTextRenderMode::Native,
        );
        routed.push_resolved_auto_text(
            text_batch("SdfAuto", UiTextRenderMode::Auto),
            UiTextRenderMode::Sdf,
        );

        assert_eq!(routed.native_texts().len(), 1);
        assert_eq!(routed.native_texts()[0].text, "NormalAuto");
        assert_eq!(routed.sdf_texts().len(), 1);
        assert_eq!(routed.sdf_texts()[0].text, "SdfAuto");
        assert_eq!(routed.sdf_atlas_texts()[0].text, "SdfAuto");
    }

    #[test]
    fn auto_text_mode_uses_font_asset_default_when_present() {
        let resolved = effective_text_render_mode(
            UiTextRenderMode::Auto,
            Some(&LoadedUiFontAsset {
                family: Some("Fira Mono".to_string()),
                render_mode: Some(UiTextRenderMode::Sdf),
            }),
        );

        assert_eq!(resolved, UiTextRenderMode::Sdf);
    }

    #[test]
    fn explicit_text_mode_overrides_font_asset_default() {
        let resolved = effective_text_render_mode(
            UiTextRenderMode::Native,
            Some(&LoadedUiFontAsset {
                family: Some("Fira Mono".to_string()),
                render_mode: Some(UiTextRenderMode::Sdf),
            }),
        );

        assert_eq!(resolved, UiTextRenderMode::Native);
    }

    #[test]
    fn auto_text_mode_falls_back_to_native_without_font_asset_default() {
        let resolved = effective_text_render_mode(UiTextRenderMode::Auto, None);

        assert_eq!(resolved, UiTextRenderMode::Native);
    }

    #[test]
    fn text_attrs_maps_shared_rich_run_style_to_glyphon_attrs() {
        let attrs = text_attrs(
            Some("Zircon Sans"),
            UiTextRunPaintStyle {
                strong: true,
                emphasis: true,
                code: false,
            },
        );

        assert_eq!(attrs.family, Family::Name("Zircon Sans"));
        assert_eq!(attrs.weight, Weight::BOLD);
        assert_eq!(attrs.style, Style::Italic);

        let code_attrs = text_attrs(
            Some("Zircon Sans"),
            UiTextRunPaintStyle {
                strong: false,
                emphasis: false,
                code: true,
            },
        );

        assert_eq!(code_attrs.family, Family::Monospace);
    }

    fn text_batch(text: &str, _mode: UiTextRenderMode) -> ScreenSpaceUiTextBatch {
        ScreenSpaceUiTextBatch {
            text: text.to_string(),
            frame: UiFrame::new(0.0, 0.0, 128.0, 24.0),
            clip_frame: None,
            color: [1.0, 1.0, 1.0, 1.0],
            font: Some("res://fonts/default.font.toml".to_string()),
            font_family: Some("Zircon Sans".to_string()),
            font_size: 16.0,
            line_height: 20.0,
            text_align: UiTextAlign::Left,
            wrap: UiTextWrap::None,
            style: Default::default(),
        }
    }
}
