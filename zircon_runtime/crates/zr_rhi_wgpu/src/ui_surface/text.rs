use glyphon::{
    Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping, Style,
    SwashCache, TextArea, TextAtlas, TextRenderer, Viewport, Weight, Wrap,
};
use zircon_runtime_interface::ui::surface::UiResolvedStyle;

use zr_rhi::{
    UiSurfaceCommand, UiSurfaceDrawList, UiSurfaceResolvedCommandKind, UiSurfaceTextStyle,
};

use super::batching::DrawOp;
use super::geometry::{
    command_effective_rect, full_projection_effective_rect, text_bounds_from_rect,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct WgpuUiTextPrepareStats {
    pub(super) text_shape_count: u64,
    pub(super) text_renderer_build_count: u64,
    pub(super) text_renderer_cache_hit_count: u64,
    pub(super) text_prepare_failure_count: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TextBatchCacheKey {
    generation: u64,
    projection_size: (u32, u32),
}

pub(super) struct WgpuUiTextRenderer {
    _cache: Cache,
    viewport: Viewport,
    atlas: TextAtlas,
    font_system: FontSystem,
    swash_cache: SwashCache,
    batch_cache_key: Option<TextBatchCacheKey>,
    batches: Vec<WgpuUiTextBatch>,
}

struct WgpuUiTextBatch {
    renderer: Option<TextRenderer>,
}

impl WgpuUiTextRenderer {
    pub(super) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_format: wgpu::TextureFormat,
    ) -> Self {
        let cache = Cache::new(device);
        let viewport = Viewport::new(device, &cache);
        let atlas = TextAtlas::new(device, queue, &cache, target_format);
        Self {
            _cache: cache,
            viewport,
            atlas,
            font_system: FontSystem::new(),
            swash_cache: SwashCache::new(),
            batch_cache_key: None,
            batches: Vec::new(),
        }
    }

    pub(super) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        projection_size: (u32, u32),
        draw_list: &UiSurfaceDrawList,
        draw_ops: &[DrawOp],
    ) -> WgpuUiTextPrepareStats {
        let cache_key = text_batch_cache_key(draw_list, projection_size);
        if let Some(cache_key) = cache_key {
            if self.batch_cache_key == Some(cache_key) {
                return WgpuUiTextPrepareStats {
                    text_renderer_cache_hit_count: self
                        .batches
                        .iter()
                        .filter(|batch| batch.renderer.is_some())
                        .count() as u64,
                    ..WgpuUiTextPrepareStats::default()
                };
            }
        }

        self.viewport.update(
            queue,
            Resolution {
                width: projection_size.0.max(1),
                height: projection_size.1.max(1),
            },
        );
        self.batches.clear();
        self.batch_cache_key = None;
        let mut stats = WgpuUiTextPrepareStats::default();
        for op in draw_ops {
            let DrawOp::Text(text_draw) = op else {
                continue;
            };
            let mut buffers = Vec::new();
            let mut text_commands = Vec::new();
            let mut text_clips = Vec::new();
            for command_index in &text_draw.command_indices {
                let Some(command) = draw_list.commands.get(*command_index) else {
                    continue;
                };
                let Some(UiSurfaceResolvedCommandKind::Text {
                    text,
                    font_family,
                    font_weight,
                    font_size,
                    line_height,
                    style,
                    ..
                }) = draw_list.resolved_kind(command)
                else {
                    continue;
                };
                if !text_has_visible_content(text) {
                    continue;
                }
                let clip = if cache_key.is_some() {
                    full_projection_effective_rect(command, draw_list)
                } else {
                    command_effective_rect(command, draw_list)
                };
                let Some(clip) = clip else {
                    continue;
                };
                let mut buffer = Buffer::new(
                    &mut self.font_system,
                    Metrics::new(font_size.max(1.0), line_height.max(1.0)),
                );
                prepare_buffer(
                    &mut self.font_system,
                    &mut buffer,
                    command,
                    text,
                    font_family,
                    font_weight,
                    style,
                );
                stats.text_shape_count = stats.text_shape_count.saturating_add(1);
                buffers.push(buffer);
                text_commands.push(command);
                text_clips.push(clip);
            }
            let has_visible_glyphs = buffers
                .iter()
                .any(|buffer| buffer.layout_runs().any(|run| !run.glyphs.is_empty()));
            let renderer = if has_visible_glyphs {
                let text_areas = text_commands
                    .iter()
                    .zip(buffers.iter())
                    .zip(text_clips.iter())
                    .map(|((command, buffer), clip)| TextArea {
                        buffer,
                        left: command.frame.x,
                        top: command.frame.y,
                        scale: 1.0,
                        bounds: text_bounds_from_rect(*clip),
                        default_color: text_color(command, draw_list),
                        custom_glyphs: &[],
                    })
                    .collect::<Vec<_>>();
                let mut renderer = TextRenderer::new(
                    &mut self.atlas,
                    device,
                    wgpu::MultisampleState::default(),
                    None,
                );

                let prepared = renderer.prepare(
                    device,
                    queue,
                    &mut self.font_system,
                    &mut self.atlas,
                    &self.viewport,
                    text_areas,
                    &mut self.swash_cache,
                );
                match prepared {
                    Ok(()) => Some(renderer),
                    Err(_) => {
                        stats.text_prepare_failure_count =
                            stats.text_prepare_failure_count.saturating_add(1);
                        None
                    }
                }
            } else {
                None
            };
            let renderer_built = renderer.is_some();
            debug_assert_eq!(self.batches.len(), text_draw.batch_index);
            // Preserve the compiled batch index even when this draw produces no glyph vertices.
            self.batches.push(WgpuUiTextBatch { renderer });
            stats.text_renderer_build_count = stats
                .text_renderer_build_count
                .saturating_add(u64::from(renderer_built));
        }
        self.batch_cache_key =
            committed_text_batch_cache_key(cache_key, stats.text_prepare_failure_count);
        stats
    }

    pub(super) fn render_batch<'pass>(
        &'pass mut self,
        batch_index: usize,
        pass: &mut wgpu::RenderPass<'pass>,
    ) -> bool {
        let Some(batch) = self.batches.get_mut(batch_index) else {
            return false;
        };
        let Some(renderer) = batch.renderer.as_mut() else {
            return false;
        };
        renderer.render(&self.atlas, &self.viewport, pass).is_ok()
    }
}

fn text_has_visible_content(text: &str) -> bool {
    text.chars().any(|character| !character.is_whitespace())
}

fn text_batch_cache_key(
    draw_list: &UiSurfaceDrawList,
    projection_size: (u32, u32),
) -> Option<TextBatchCacheKey> {
    draw_list.generation().map(|generation| TextBatchCacheKey {
        generation,
        projection_size,
    })
}

fn committed_text_batch_cache_key(
    cache_key: Option<TextBatchCacheKey>,
    prepare_failure_count: u64,
) -> Option<TextBatchCacheKey> {
    (prepare_failure_count == 0).then_some(cache_key).flatten()
}

fn prepare_buffer(
    font_system: &mut FontSystem,
    buffer: &mut Buffer,
    command: &UiSurfaceCommand,
    text: &str,
    font_family: Option<&str>,
    font_weight: u16,
    style: UiSurfaceTextStyle,
) {
    buffer.set_size(
        font_system,
        Some(command.frame.width.max(1.0)),
        Some(command.frame.height.max(1.0)),
    );
    buffer.set_wrap(font_system, Wrap::None);
    buffer.set_text(
        font_system,
        text,
        &text_attrs(font_family, font_weight, style),
        Shaping::Advanced,
        None,
    );
    buffer.shape_until_scroll(font_system, false);
}

fn text_color(command: &UiSurfaceCommand, draw_list: &UiSurfaceDrawList) -> Color {
    match draw_list.resolved_kind(command) {
        Some(UiSurfaceResolvedCommandKind::Text { color, .. }) => {
            Color::rgba(color[0], color[1], color[2], color[3])
        }
        _ => Color::rgb(255, 255, 255),
    }
}

fn text_attrs<'a>(
    font_family: Option<&'a str>,
    font_weight: u16,
    style: UiSurfaceTextStyle,
) -> Attrs<'a> {
    let mut attrs = font_family
        .filter(|family| !family.trim().is_empty())
        .map(|family| Attrs::new().family(Family::Name(family)))
        .unwrap_or_else(Attrs::new);
    let resolved_weight = UiResolvedStyle::normalized_font_weight(font_weight);
    let resolved_weight = if matches!(
        style,
        UiSurfaceTextStyle::Strong | UiSurfaceTextStyle::StrongEmphasis
    ) {
        resolved_weight.max(Weight::BOLD.0)
    } else {
        resolved_weight
    };
    attrs = attrs.weight(Weight(resolved_weight));
    if matches!(
        style,
        UiSurfaceTextStyle::Strong | UiSurfaceTextStyle::StrongEmphasis
    ) {
        debug_assert!(attrs.weight.0 >= Weight::BOLD.0);
    }
    if matches!(
        style,
        UiSurfaceTextStyle::Emphasis | UiSurfaceTextStyle::StrongEmphasis
    ) {
        attrs = attrs.style(Style::Italic);
    }
    attrs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_surface_text_attrs_preserve_requested_family_and_weight() {
        let attrs = text_attrs(Some("Zircon Sans"), 500, UiSurfaceTextStyle::Regular);

        assert_eq!(attrs.family, Family::Name("Zircon Sans"));
        assert_eq!(attrs.weight, Weight(500));

        let strong_attrs = text_attrs(Some("Zircon Sans"), 500, UiSurfaceTextStyle::Strong);
        assert_eq!(strong_attrs.family, Family::Name("Zircon Sans"));
        assert_eq!(strong_attrs.weight, Weight::BOLD);

        let emphasis_attrs = text_attrs(None, 450, UiSurfaceTextStyle::Emphasis);
        assert_eq!(emphasis_attrs.weight, Weight(450));
        assert_eq!(emphasis_attrs.style, Style::Italic);
    }

    #[test]
    fn text_batch_cache_key_allows_a_versioned_damage_projection() {
        let versioned = UiSurfaceDrawList::with_generation((64, 32), None, Vec::new(), 9);
        let damaged = UiSurfaceDrawList::with_generation(
            (64, 32),
            Some(zr_rhi::UiSurfaceRect::new(0.0, 0.0, 8.0, 8.0)),
            Vec::new(),
            9,
        );
        let legacy = UiSurfaceDrawList::new((64, 32), None, Vec::new());

        assert!(text_batch_cache_key(&versioned, (64, 32)).is_some());
        assert!(text_batch_cache_key(&damaged, (64, 32)).is_some());
        assert_eq!(text_batch_cache_key(&legacy, (64, 32)), None);
    }

    #[test]
    fn text_batch_cache_key_ignores_target_only_resize() {
        let mut draw_list = UiSurfaceDrawList::with_generation((64, 32), None, Vec::new(), 9);
        let original = text_batch_cache_key(&draw_list, draw_list.projection_size());

        draw_list.retarget_surface_size_preserving_projection((32, 16));

        assert_eq!(
            text_batch_cache_key(&draw_list, draw_list.projection_size()),
            original
        );
    }

    #[test]
    fn text_preparation_skips_content_that_cannot_produce_visible_glyphs() {
        assert!(!text_has_visible_content(""));
        assert!(!text_has_visible_content(" \t\r\n"));
        assert!(text_has_visible_content("Zircon"));
    }

    #[test]
    fn text_prepare_failure_does_not_publish_the_generation_cache_key() {
        let cache_key = Some(TextBatchCacheKey {
            generation: 7,
            projection_size: (320, 240),
        });

        assert_eq!(committed_text_batch_cache_key(cache_key, 0), cache_key);
        assert_eq!(committed_text_batch_cache_key(cache_key, 1), None);
    }
}
