//! 文本整形与图集准备遵守投影尺寸及批次顺序；字形准备失败时不发布文本批次缓存键，成功的键在呈现提交前已形成。
use glyphon::{
    Attrs, Buffer, Cache, Color, ColorMode, Family, FontSystem, Metrics, Resolution, Shaping,
    Style, SwashCache, TextArea, TextAtlas, TextRenderer, Viewport, Weight, Wrap,
};
use zircon_runtime_interface::ui::surface::UiResolvedStyle;

use zr_rhi::{
    UiSurfaceCommand, UiSurfaceDrawList, UiSurfaceResolvedCommandKind, UiSurfaceTextStyle,
};

mod layout_evidence;
use super::batching::DrawOp;
use super::color_space::{target_color_mode, UiTargetColorMode};
use super::geometry::{
    command_effective_rect, full_projection_effective_rect, text_bounds_from_rect,
};
use layout_evidence::observe_buffer;
use zr_rhi::{UiSurfaceTextLayoutRun, UiSurfaceTextLayoutSnapshot};

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
    prepared_renderer_count: u64,
    batches: Vec<WgpuUiTextBatch>,
    font_face_cache: std::collections::HashMap<String, zr_rhi::UiSurfaceTextFace>,
    pub(super) observe_layout: bool,
    pub(super) layout_snapshot: Option<UiSurfaceTextLayoutSnapshot>,
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
        let atlas = TextAtlas::with_color_mode(
            device,
            queue,
            &cache,
            target_format,
            text_color_mode(target_format),
        );
        Self {
            _cache: cache,
            viewport,
            atlas,
            font_system: FontSystem::new(),
            swash_cache: SwashCache::new(),
            batch_cache_key: None,
            prepared_renderer_count: 0,
            batches: Vec::new(),
            font_face_cache: std::collections::HashMap::new(),
            observe_layout: false,
            layout_snapshot: None,
        }
    }

    pub(super) fn set_layout_observation(&mut self, enabled: bool) {
        if self.observe_layout != enabled {
            self.observe_layout = enabled;
            self.layout_snapshot = None;
            // Existing cached batches lack the newly requested layout observation.
            self.batch_cache_key = None;
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
                    text_renderer_cache_hit_count: self.prepared_renderer_count,
                    ..WgpuUiTextPrepareStats::default()
                };
            }
        }

        self.layout_snapshot = None;
        let mut observed_runs: Vec<UiSurfaceTextLayoutRun> = Vec::new();
        self.viewport.update(
            queue,
            Resolution {
                width: projection_size.0.max(1),
                height: projection_size.1.max(1),
            },
        );
        self.batches.clear();
        self.batch_cache_key = None;
        self.prepared_renderer_count = 0;
        let mut stats = WgpuUiTextPrepareStats::default();
        for op in draw_ops {
            let DrawOp::Text(text_draw) = op else {
                continue;
            };
            let mut batch_observations = Vec::new();
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
                let mut buffer =
                    Buffer::new(&mut self.font_system, text_metrics(font_size, line_height));
                prepare_buffer(
                    &mut self.font_system,
                    &mut buffer,
                    command,
                    text,
                    font_family,
                    font_weight,
                    style,
                );
                if self.observe_layout {
                    batch_observations.push(observe_buffer(
                        &self.font_system,
                        &mut self.font_face_cache,
                        &buffer,
                        command,
                        *command_index,
                        text,
                        clip,
                    ));
                }
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
            if renderer_built {
                observed_runs.extend(batch_observations);
            }
            debug_assert_eq!(self.batches.len(), text_draw.batch_index);
            // Preserve the compiled batch index even when this draw produces no glyph vertices.
            self.batches.push(WgpuUiTextBatch { renderer });
            stats.text_renderer_build_count = stats
                .text_renderer_build_count
                .saturating_add(u64::from(renderer_built));
        }
        self.batch_cache_key =
            committed_text_batch_cache_key(cache_key, stats.text_prepare_failure_count);
        self.prepared_renderer_count = stats.text_renderer_build_count;
        if self.observe_layout && stats.text_prepare_failure_count == 0 {
            self.layout_snapshot = Some(UiSurfaceTextLayoutSnapshot {
                presented_frame_count: 0,
                projection_size,
                damage: draw_list.damage,
                prepared_this_present: false,
                retained_cache_copy_bytes: 0,
                draw_list_generation: draw_list.generation(),
                runs: observed_runs,
            });
        }
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
        let rendered = renderer.render(&self.atlas, &self.viewport, pass).is_ok();
        if !rendered {
            self.layout_snapshot = None;
        }
        rendered
    }
}

fn text_has_visible_content(text: &str) -> bool {
    text.chars().any(|character| !character.is_whitespace())
}

fn text_metrics(font_size: f32, line_height: f32) -> Metrics {
    Metrics::new(font_size.max(1.0), line_height.max(1.0))
}

fn text_color_mode(target_format: wgpu::TextureFormat) -> ColorMode {
    match target_color_mode(target_format) {
        UiTargetColorMode::LinearSrgb => ColorMode::Accurate,
        UiTargetColorMode::ByteEncodedFallback => ColorMode::Web,
    }
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
#[path = "tests/text.rs"]
mod tests;
