use std::collections::HashMap;
use std::sync::Arc;

use glyphon::{
    Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache,
    TextArea, TextAtlas, TextBounds, TextRenderer, Viewport, Wrap,
};

use super::font_asset::load_ui_font_manifest_with_asset_manager;
use super::render::ScreenSpaceUiTextBatch;
use crate::asset::ProjectAssetManager;
use glyphon::cosmic_text::Align;

const DEFAULT_FONT_ASSET: &str = "res://fonts/default.font.toml";

pub(super) struct ScreenSpaceUiTextSystem {
    asset_manager: Arc<ProjectAssetManager>,
    font_system: FontSystem,
    swash_cache: SwashCache,
    font_assets: HashMap<String, LoadedUiFontAsset>,
    native: ScreenSpaceUiTextBackend,
    sdf: ScreenSpaceUiTextBackend,
}

struct ScreenSpaceUiTextBackend {
    _cache: Cache,
    viewport: Viewport,
    atlas: TextAtlas,
    renderer: TextRenderer,
}

#[derive(Clone, Debug, Default)]
struct LoadedUiFontAsset {
    family: Option<String>,
    render_mode: Option<crate::ui::surface::UiTextRenderMode>,
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
            sdf: ScreenSpaceUiTextBackend::new(device, queue, target_format),
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
        let (native_texts, sdf_texts) = resolve_text_batches(
            &mut self.font_system,
            &mut self.font_assets,
            self.asset_manager.as_ref(),
            auto_texts,
            native_texts,
            sdf_texts,
        );
        self.native.prepare(
            device,
            queue,
            viewport_size,
            &native_texts,
            &mut self.font_system,
            &mut self.swash_cache,
            &mut self.font_assets,
            self.asset_manager.as_ref(),
        );
        self.sdf.prepare(
            device,
            queue,
            viewport_size,
            &sdf_texts,
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
        let _ = self
            .sdf
            .renderer
            .render(&self.sdf.atlas, &self.sdf.viewport, pass);
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
            let attrs = family_name
                .as_deref()
                .map(|family| Attrs::new().family(Family::Name(family)))
                .unwrap_or_else(Attrs::new);
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
                    crate::ui::surface::UiTextWrap::None => Wrap::None,
                    crate::ui::surface::UiTextWrap::Word => Wrap::Word,
                    crate::ui::surface::UiTextWrap::Glyph => Wrap::Glyph,
                },
            );
            buffer.set_text(
                font_system,
                &text.text,
                &attrs,
                Shaping::Advanced,
                Some(match text.text_align {
                    crate::ui::surface::UiTextAlign::Left => Align::Left,
                    crate::ui::surface::UiTextAlign::Center => Align::Center,
                    crate::ui::surface::UiTextAlign::Right => Align::Right,
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
) -> (Vec<ScreenSpaceUiTextBatch>, Vec<ScreenSpaceUiTextBatch>) {
    let mut resolved_native = native_texts.to_vec();
    let mut resolved_sdf = sdf_texts.to_vec();

    for text in auto_texts {
        let font_asset = resolve_font_asset_record(
            font_system,
            font_assets,
            asset_manager,
            text.font.as_deref(),
        );
        match effective_text_render_mode(crate::ui::surface::UiTextRenderMode::Auto, font_asset) {
            crate::ui::surface::UiTextRenderMode::Auto
            | crate::ui::surface::UiTextRenderMode::Native => {
                resolved_native.push(text.clone());
            }
            crate::ui::surface::UiTextRenderMode::Sdf => resolved_sdf.push(text.clone()),
        }
    }

    (resolved_native, resolved_sdf)
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
    requested_mode: crate::ui::surface::UiTextRenderMode,
    font_asset: Option<&LoadedUiFontAsset>,
) -> crate::ui::surface::UiTextRenderMode {
    match requested_mode {
        crate::ui::surface::UiTextRenderMode::Native => {
            crate::ui::surface::UiTextRenderMode::Native
        }
        crate::ui::surface::UiTextRenderMode::Sdf => crate::ui::surface::UiTextRenderMode::Sdf,
        crate::ui::surface::UiTextRenderMode::Auto => font_asset
            .and_then(|asset| asset.render_mode)
            .filter(|mode| !matches!(mode, crate::ui::surface::UiTextRenderMode::Auto))
            .unwrap_or(crate::ui::surface::UiTextRenderMode::Native),
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
    let clip = text.clip_frame.unwrap_or_else(|| {
        crate::ui::layout::UiFrame::new(0.0, 0.0, viewport_size.x as f32, viewport_size.y as f32)
    });
    let clip = clip
        .intersection(crate::ui::layout::UiFrame::new(
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
    fn auto_text_mode_uses_font_asset_default_when_present() {
        let resolved = effective_text_render_mode(
            crate::ui::surface::UiTextRenderMode::Auto,
            Some(&LoadedUiFontAsset {
                family: Some("Fira Mono".to_string()),
                render_mode: Some(crate::ui::surface::UiTextRenderMode::Sdf),
            }),
        );

        assert_eq!(resolved, crate::ui::surface::UiTextRenderMode::Sdf);
    }

    #[test]
    fn explicit_text_mode_overrides_font_asset_default() {
        let resolved = effective_text_render_mode(
            crate::ui::surface::UiTextRenderMode::Native,
            Some(&LoadedUiFontAsset {
                family: Some("Fira Mono".to_string()),
                render_mode: Some(crate::ui::surface::UiTextRenderMode::Sdf),
            }),
        );

        assert_eq!(resolved, crate::ui::surface::UiTextRenderMode::Native);
    }

    #[test]
    fn auto_text_mode_falls_back_to_native_without_font_asset_default() {
        let resolved = effective_text_render_mode(crate::ui::surface::UiTextRenderMode::Auto, None);

        assert_eq!(resolved, crate::ui::surface::UiTextRenderMode::Native);
    }
}
