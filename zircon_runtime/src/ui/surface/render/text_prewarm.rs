use crate::core::runtime::tasks::{TaskPool, TaskPools};
use crate::ui::text::{
    resolve_text_layout, UiTextLayoutRequest, UiTextMeasureCache, UiTextShapePrewarmRequest,
};
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{UiRenderCommand, UiRichTextFormat};

const UI_TEXT_SHAPE_PREWARM_CHUNK_SIZE: usize = 8;

pub(super) fn prewarm_render_command_text(
    commands: &[UiRenderCommand],
    text_measure_cache: &mut UiTextMeasureCache,
) {
    let requests = commands
        .iter()
        .filter_map(|command| {
            if !command_text_can_use_shape_prewarm(command) {
                return None;
            }
            UiTextShapePrewarmRequest::from_layout_source(
                command.text.as_deref()?,
                command.style.clone(),
            )
        })
        .collect::<Vec<_>>();

    if requests.is_empty() {
        return;
    }

    let pool = ui_text_shape_prewarm_pool();
    text_measure_cache.prewarm_horizontal_paragraphs(
        &pool,
        &requests,
        UI_TEXT_SHAPE_PREWARM_CHUNK_SIZE,
    );
}

pub(super) fn resolve_missing_render_command_text_layouts(
    commands: &mut [UiRenderCommand],
    mut text_measure_cache: Option<&mut UiTextMeasureCache>,
) {
    for command in commands {
        if !command_text_needs_layout(command) {
            continue;
        }
        let Some(text) = command.text.as_deref() else {
            continue;
        };
        let request =
            UiTextLayoutRequest::new(text, &command.style, command.frame, command.clip_frame);
        let layout = match text_measure_cache.as_deref_mut() {
            Some(cache) => cache.resolve_or_shape(&request).layout,
            None => resolve_text_layout(&request).layout,
        };
        command.text_layout = Some(layout);
    }
}

fn command_text_can_use_shape_prewarm(command: &UiRenderCommand) -> bool {
    command_text_needs_layout(command)
}

fn command_text_needs_layout(command: &UiRenderCommand) -> bool {
    command.text_layout.is_none()
        && command
            .text
            .as_ref()
            .is_some_and(|text| !text.trim().is_empty())
        && valid_text_frame(command.frame)
}

fn valid_text_frame(frame: UiFrame) -> bool {
    frame.width.is_finite() && frame.height.is_finite() && frame.width > 0.0 && frame.height > 0.0
}

fn ui_text_shape_prewarm_pool() -> TaskPool {
    TaskPools::process_default().compute().clone()
}

#[cfg(test)]
mod tests {
    use super::{prewarm_render_command_text, ui_text_shape_prewarm_pool};
    use crate::core::runtime::tasks::TaskPools;
    use crate::ui::text::UiTextMeasureCache;
    use zircon_runtime_interface::ui::{
        event_ui::UiNodeId,
        layout::UiFrame,
        surface::{
            UiRenderCommand, UiRenderCommandKind, UiResolvedStyle, UiRichTextFormat,
            UiTextWritingMode,
        },
    };

    #[test]
    fn prewarm_render_command_text_skips_rich_and_vertical_contract_mismatches() {
        let mut cache = UiTextMeasureCache::default();
        cache.begin_frame();

        prewarm_render_command_text(
            &[
                text_command(
                    "**sample base.zui**",
                    UiResolvedStyle {
                        rich_text_format: UiRichTextFormat::Markdown,
                        font_size: 10.0,
                        line_height: 12.0,
                        ..UiResolvedStyle::default()
                    },
                ),
                text_command(
                    "folder-open-outline.svg",
                    UiResolvedStyle {
                        text_writing_mode: UiTextWritingMode::VerticalRl,
                        font_size: 10.0,
                        line_height: 12.0,
                        ..UiResolvedStyle::default()
                    },
                ),
                text_command(
                    "**sample base.zui**",
                    UiResolvedStyle {
                        rich_text_format: UiRichTextFormat::Markdown,
                        font_size: 10.0,
                        line_height: 12.0,
                        ..UiResolvedStyle::default()
                    },
                ),
            ],
            &mut cache,
        );

        let report = cache.frame_shape_prewarm_report();
        assert_eq!(report.requested_count, 0);
        assert_eq!(report.cache_miss_count, 0);
        assert_eq!(report.shaped_count, 0);
    }

    #[test]
    fn text_prewarm_uses_one_shared_compute_pool_join_after_command_collection() {
        let prewarm_pool = ui_text_shape_prewarm_pool();
        let process_pools = TaskPools::process_default();
        let extract_source = include_str!("extract.rs");

        assert!(prewarm_pool.shares_execution_owner_with(process_pools.compute()));
        assert_eq!(
            extract_source
                .matches("prewarm_render_command_text(&commands, cache)")
                .count(),
            1
        );
        assert!(!extract_source.contains("prewarm_visible_owner_text"));
    }

    fn text_command(text: &str, style: UiResolvedStyle) -> UiRenderCommand {
        UiRenderCommand {
            node_id: UiNodeId::new(1),
            kind: UiRenderCommandKind::Text,
            frame: UiFrame::new(0.0, 0.0, 180.0, 20.0),
            clip_frame: None,
            z_index: 0,
            style,
            text_layout: None,
            text: Some(text.to_string()),
            image: None,
            opacity: 1.0,
        }
    }
}
