use glyphon::{
    Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping, Style,
    SwashCache, TextArea, TextAtlas, TextRenderer, Viewport, Weight, Wrap,
};
use zircon_runtime_interface::ui::surface::UiResolvedStyle;

use crate::rhi::{UiSurfaceCommand, UiSurfaceCommandKind, UiSurfaceDrawList, UiSurfaceTextStyle};

use super::batching::DrawOp;
use super::geometry::{command_effective_rect, text_bounds_from_rect};

pub(super) struct WgpuUiTextRenderer {
    _cache: Cache,
    viewport: Viewport,
    atlas: TextAtlas,
    font_system: FontSystem,
    swash_cache: SwashCache,
    batches: Vec<WgpuUiTextBatch>,
}

struct WgpuUiTextBatch {
    renderer: TextRenderer,
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
            batches: Vec::new(),
        }
    }

    pub(super) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_size: (u32, u32),
        draw_list: &UiSurfaceDrawList,
        draw_ops: &[DrawOp],
    ) {
        self.viewport.update(
            queue,
            Resolution {
                width: surface_size.0.max(1),
                height: surface_size.1.max(1),
            },
        );
        self.batches.clear();
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
                let UiSurfaceCommandKind::Text {
                    text,
                    font_family,
                    font_weight,
                    font_size,
                    line_height,
                    style,
                    ..
                } = &command.kind
                else {
                    continue;
                };
                let Some(clip) = command_effective_rect(command, draw_list) else {
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
                    font_family.as_deref(),
                    *font_weight,
                    *style,
                );
                buffers.push(buffer);
                text_commands.push(command);
                text_clips.push(clip);
            }
            if buffers.is_empty() {
                continue;
            }
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
                    default_color: text_color(command),
                    custom_glyphs: &[],
                })
                .collect::<Vec<_>>();
            let mut renderer = TextRenderer::new(
                &mut self.atlas,
                device,
                wgpu::MultisampleState::default(),
                None,
            );

            let _ = renderer.prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                text_areas,
                &mut self.swash_cache,
            );
            debug_assert_eq!(self.batches.len(), text_draw.batch_index);
            self.batches.push(WgpuUiTextBatch { renderer });
        }
    }

    pub(super) fn render_batch<'pass>(
        &'pass mut self,
        batch_index: usize,
        pass: &mut wgpu::RenderPass<'pass>,
    ) {
        let Some(batch) = self.batches.get_mut(batch_index) else {
            return;
        };
        let _ = batch.renderer.render(&self.atlas, &self.viewport, pass);
    }
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

fn text_color(command: &UiSurfaceCommand) -> Color {
    match &command.kind {
        UiSurfaceCommandKind::Text { color, .. } => {
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
}
