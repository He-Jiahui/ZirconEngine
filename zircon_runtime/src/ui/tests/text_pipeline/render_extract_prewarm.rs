use super::fixtures::{
    button_metadata, rich_text_metadata, text_layout_command_count, text_metadata,
    vertical_text_metadata, visible_text_state,
};
use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::UiFrame,
    surface::{UiRenderCommandKind, UiTextWritingMode},
    tree::UiTreeNode,
};

#[test]
fn render_extract_automatically_prewarms_visible_owner_text_before_layout() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.text.prewarm.surface"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 260.0, 72.0))
            .with_state_flags(visible_text_state(true)),
    );
    for (node_id, path, frame, text, visible) in [
        (
            UiNodeId::new(2),
            "root/first",
            UiFrame::new(0.0, 0.0, 220.0, 16.0),
            "editor base.zui",
            true,
        ),
        (
            UiNodeId::new(3),
            "root/second",
            UiFrame::new(0.0, 16.0, 220.0, 16.0),
            "folder-open-outline.svg",
            true,
        ),
        (
            UiNodeId::new(4),
            "root/duplicate",
            UiFrame::new(0.0, 32.0, 220.0, 16.0),
            "editor base.zui",
            true,
        ),
        (
            UiNodeId::new(5),
            "root/hidden",
            UiFrame::new(0.0, 48.0, 220.0, 16.0),
            "hidden-row-should-not-prewarm",
            false,
        ),
    ] {
        surface
            .tree
            .insert_child(
                UiNodeId::new(1),
                UiTreeNode::new(node_id, UiNodePath::new(path))
                    .with_frame(frame)
                    .with_state_flags(visible_text_state(visible))
                    .with_template_metadata(text_metadata(text)),
            )
            .expect("text child should be inserted");
    }

    surface.rebuild();

    let prewarm_report = surface.text_measure_cache.frame_shape_prewarm_report();
    let shaped_report = surface.text_measure_cache.frame_shaped_run_report();

    assert_eq!(text_layout_command_count(&surface), 3);
    assert_eq!(
        prewarm_report.requested_count, 3,
        "only visible owner text should be collected for automatic prewarm"
    );
    assert_eq!(prewarm_report.cache_hit_count, 0);
    assert_eq!(
        prewarm_report.cache_miss_count, 2,
        "duplicate visible labels should share one pending shape miss"
    );
    assert_eq!(prewarm_report.batch_duplicate_count, 1);
    assert_eq!(prewarm_report.shaped_count, 2);
    assert_eq!(prewarm_report.inserted_count, 2);
    assert_eq!(
        shaped_report
            .miss_count
            .saturating_sub(prewarm_report.inserted_count as u64),
        1,
        "layout should only add the shared metrics run after visible source text prewarm"
    );
    assert!(
        shaped_report.hit_count >= prewarm_report.requested_count as u64,
        "layout should consume the source runs inserted by automatic prewarm"
    );
}

#[test]
fn render_extract_prewarms_and_layouts_component_text_commands() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.text.prewarm.component-commands"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 320.0, 72.0))
            .with_state_flags(visible_text_state(true)),
    );
    for (node_id, path, frame, text, visible) in [
        (
            UiNodeId::new(2),
            "root/first_button",
            UiFrame::new(0.0, 0.0, 150.0, 22.0),
            "editor base.zui",
            true,
        ),
        (
            UiNodeId::new(3),
            "root/second_button",
            UiFrame::new(160.0, 0.0, 150.0, 22.0),
            "folder-open-outline.svg",
            true,
        ),
        (
            UiNodeId::new(4),
            "root/duplicate_button",
            UiFrame::new(0.0, 28.0, 150.0, 22.0),
            "editor base.zui",
            true,
        ),
        (
            UiNodeId::new(5),
            "root/hidden_button",
            UiFrame::new(160.0, 28.0, 150.0, 22.0),
            "hidden-row-should-not-prewarm",
            false,
        ),
    ] {
        surface
            .tree
            .insert_child(
                UiNodeId::new(1),
                UiTreeNode::new(node_id, UiNodePath::new(path))
                    .with_frame(frame)
                    .with_state_flags(visible_text_state(visible))
                    .with_template_metadata(button_metadata(text)),
            )
            .expect("button child should be inserted");
    }

    surface.rebuild();

    let text_commands = surface
        .render_extract
        .list
        .commands
        .iter()
        .filter(|command| {
            matches!(command.kind, UiRenderCommandKind::Text)
                && command.text.as_ref().is_some_and(|text| !text.is_empty())
        })
        .collect::<Vec<_>>();
    let prewarm_report = surface.text_measure_cache.frame_shape_prewarm_report();

    assert_eq!(text_commands.len(), 3);
    assert!(
        text_commands
            .iter()
            .all(|command| command.text_layout.is_some()),
        "component generated text commands should be resolved before retained-host fallback"
    );
    assert_eq!(
        prewarm_report.requested_count, 3,
        "only visible component text commands should be collected for automatic prewarm"
    );
    assert_eq!(prewarm_report.cache_hit_count, 0);
    assert_eq!(prewarm_report.cache_miss_count, 2);
    assert_eq!(prewarm_report.batch_duplicate_count, 1);
    assert_eq!(prewarm_report.shaped_count, 2);
    assert_eq!(prewarm_report.inserted_count, 2);
}

#[test]
fn render_extract_prewarms_rich_and_vertical_owner_text_before_layout() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.text.prewarm.rich-vertical"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 260.0, 80.0))
            .with_state_flags(visible_text_state(true)),
    );
    for (node_id, path, frame, metadata) in [
        (
            UiNodeId::new(2),
            "root/rich_first",
            UiFrame::new(0.0, 0.0, 220.0, 16.0),
            rich_text_metadata("**editor base.zui**"),
        ),
        (
            UiNodeId::new(3),
            "root/vertical",
            UiFrame::new(0.0, 18.0, 48.0, 48.0),
            vertical_text_metadata("folder-open-outline.svg"),
        ),
        (
            UiNodeId::new(4),
            "root/rich_duplicate",
            UiFrame::new(0.0, 66.0, 220.0, 16.0),
            rich_text_metadata("**editor base.zui**"),
        ),
    ] {
        surface
            .tree
            .insert_child(
                UiNodeId::new(1),
                UiTreeNode::new(node_id, UiNodePath::new(path))
                    .with_frame(frame)
                    .with_state_flags(visible_text_state(true))
                    .with_template_metadata(metadata),
            )
            .expect("text child should be inserted");
    }

    surface.rebuild();

    let prewarm_report = surface.text_measure_cache.frame_shape_prewarm_report();
    let shaped_report = surface.text_measure_cache.frame_shaped_run_report();
    let rich_command = surface
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == UiNodeId::new(2))
        .expect("rich text command should be present");
    let vertical_command = surface
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == UiNodeId::new(3))
        .expect("vertical text command should be present");

    assert_eq!(text_layout_command_count(&surface), 3);
    assert_eq!(
        rich_command
            .text_layout
            .as_ref()
            .and_then(|layout| layout.lines.first())
            .map(|line| line.text.as_str()),
        Some("editor base.zui"),
        "rich-text prewarm must use the same visible text that layout resolves"
    );
    assert_eq!(
        vertical_command
            .text_layout
            .as_ref()
            .map(|layout| layout.writing_mode),
        Some(UiTextWritingMode::VerticalRl)
    );
    assert_eq!(prewarm_report.requested_count, 3);
    assert_eq!(prewarm_report.cache_miss_count, 2);
    assert_eq!(prewarm_report.batch_duplicate_count, 1);
    assert_eq!(prewarm_report.shaped_count, 2);
    assert_eq!(prewarm_report.inserted_count, 2);
    assert!(
        shaped_report.hit_count >= prewarm_report.requested_count as u64,
        "rich and vertical layout should consume the prewarmed shaped runs"
    );
}
