use std::sync::OnceLock;

use crate::core::runtime::tasks::{TaskPool, TaskPoolDescriptor};
use crate::ui::surface::{
    component_state::UiSurfaceComponentStateStore, is_arranged_render_visible,
};
use crate::ui::text::{
    resolve_text_layout, UiTextLayoutRequest, UiTextMeasureCache, UiTextShapePrewarmRequest,
};
use zircon_runtime_interface::ui::surface::{UiRenderCommand, UiRichTextFormat};
use zircon_runtime_interface::ui::{layout::UiFrame, surface::UiArrangedTree, tree::UiTree};

use super::{
    buttons::button_suppresses_owner_text, chrome::chrome_suppresses_owner_text,
    collection_rows::collection_row_suppresses_owner_text,
    command_palette::command_palette_suppresses_owner_text, dialog::dialog_suppresses_owner_text,
    drag_overlay::drag_overlay_suppresses_owner_text, dropdowns::dropdown_suppresses_owner_text,
    feedback::feedback_suppresses_owner_text, node_visual_data::UiNodeVisualData,
    notification_center::notification_center_suppresses_owner_text,
    segmented_controls::segmented_control_suppresses_owner_text,
    selection_controls::selection_control_suppresses_owner_text,
    sliders::slider_suppresses_owner_text, text_fields::text_field_suppresses_owner_text,
};

const UI_TEXT_SHAPE_PREWARM_CHUNK_SIZE: usize = 8;
const UI_TEXT_SHAPE_PREWARM_THREADS: usize = 2;

pub(super) fn prewarm_visible_owner_text(
    tree: &UiTree,
    arranged_tree: &UiArrangedTree,
    component_states: Option<&UiSurfaceComponentStateStore>,
    text_measure_cache: &mut UiTextMeasureCache,
) {
    let requests = arranged_tree
        .draw_order
        .iter()
        .copied()
        .filter_map(|node_id| {
            let node = tree.nodes.get(&node_id)?;
            if !is_arranged_render_visible(arranged_tree, node_id).unwrap_or(false) {
                return None;
            }
            if suppresses_owner_text(node.template_metadata.as_ref()) {
                return None;
            }

            let component_state = component_states.and_then(|states| states.get(node_id));
            let visual = UiNodeVisualData::resolve(
                node.template_metadata.as_ref(),
                &node.state_flags,
                component_state,
            );
            let text = visual.text?;
            if text.is_empty() {
                return None;
            }
            UiTextShapePrewarmRequest::from_layout_source(&text, visual.style)
        })
        .collect::<Vec<_>>();

    if requests.is_empty() {
        return;
    }

    text_measure_cache.prewarm_horizontal_paragraphs(
        ui_text_shape_prewarm_pool(),
        &requests,
        UI_TEXT_SHAPE_PREWARM_CHUNK_SIZE,
    );
}

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

    text_measure_cache.prewarm_horizontal_paragraphs(
        ui_text_shape_prewarm_pool(),
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

fn suppresses_owner_text(
    metadata: Option<&zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata>,
) -> bool {
    selection_control_suppresses_owner_text(metadata)
        || slider_suppresses_owner_text(metadata)
        || dropdown_suppresses_owner_text(metadata)
        || text_field_suppresses_owner_text(metadata)
        || button_suppresses_owner_text(metadata)
        || segmented_control_suppresses_owner_text(metadata)
        || collection_row_suppresses_owner_text(metadata)
        || feedback_suppresses_owner_text(metadata)
        || dialog_suppresses_owner_text(metadata)
        || command_palette_suppresses_owner_text(metadata)
        || notification_center_suppresses_owner_text(metadata)
        || drag_overlay_suppresses_owner_text(metadata)
        || chrome_suppresses_owner_text(metadata)
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

fn ui_text_shape_prewarm_pool() -> &'static TaskPool {
    static POOL: OnceLock<TaskPool> = OnceLock::new();
    POOL.get_or_init(|| {
        TaskPool::new(
            TaskPoolDescriptor::compute()
                .with_thread_name("ui-text-shape-prewarm")
                .with_worker_threads(UI_TEXT_SHAPE_PREWARM_THREADS),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::prewarm_render_command_text;
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
    fn prewarm_render_command_text_accepts_rich_and_vertical_commands() {
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
        assert_eq!(report.requested_count, 3);
        assert_eq!(report.cache_miss_count, 2);
        assert_eq!(report.batch_duplicate_count, 1);
        assert_eq!(report.shaped_count, 2);
        assert_eq!(report.inserted_count, 2);
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
