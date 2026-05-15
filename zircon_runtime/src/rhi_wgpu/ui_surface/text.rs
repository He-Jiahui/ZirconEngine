use glyphon::{
    Attrs, Buffer, Cache, Color, FontSystem, Metrics, Resolution, Shaping, Style, SwashCache,
    TextArea, TextAtlas, TextRenderer, Viewport, Weight, Wrap,
};

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
                prepare_buffer(&mut self.font_system, &mut buffer, command, text, *style);
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
        &text_attrs(style),
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

fn text_attrs(style: UiSurfaceTextStyle) -> Attrs<'static> {
    let mut attrs = Attrs::new();
    if matches!(
        style,
        UiSurfaceTextStyle::Strong | UiSurfaceTextStyle::StrongEmphasis
    ) {
        attrs = attrs.weight(Weight::BOLD);
    }
    if matches!(
        style,
        UiSurfaceTextStyle::Emphasis | UiSurfaceTextStyle::StrongEmphasis
    ) {
        attrs = attrs.style(Style::Italic);
    }
    attrs
}
