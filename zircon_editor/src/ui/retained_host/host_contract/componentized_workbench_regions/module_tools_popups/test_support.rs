//! Actual source-bound painter assertions shared with retained-host native input tests.
use crate::ui::retained_host::host_contract::data::{
    FrameRect, HostWindowPresentationData, TemplatePaneNodeData,
};
use crate::ui::retained_host::host_contract::{
    componentized_workbench_regions::{module_tools_popups, ModuleToolsPopup},
    paint_frame::HostRecordedPaintKind,
    paint_frame::HostRgbaFrame,
    paint_workbench::draw_workbench_presentation_commands,
    surface_hit_test::{hit_test_workbench_window_template_node_with_index, HostWorkbenchHitIndex},
    template_popup_layout::menu_item_row_frame,
};
use std::sync::Arc;
use zircon_runtime_interface::ui::surface::{UiRenderFrameCommandRef, UiSurfaceFrame};

fn bounds(presentation: &HostWindowPresentationData) -> FrameRect {
    let root = presentation
        .workbench_window_nodes
        .iter()
        .find(|node| node.control_id.as_str() == "WorkbenchWindowRoot")
        .unwrap();
    FrameRect {
        x: root.frame.x,
        y: root.frame.y,
        width: root.frame.width,
        height: root.frame.height,
    }
}

fn same_source(
    command: &crate::ui::retained_host::host_contract::paint_frame::HostRecordedPaintCommand,
    recorded: &crate::ui::retained_host::host_contract::paint_frame::HostRecordedFrame,
    node: &TemplatePaneNodeData,
) -> bool {
    command.source.is_some_and(|source| {
        Some(source.command_ref) == node.surface_render_command_ref
            && recorded
                .render_sources
                .resolve(source.surface_key)
                .zip(node.source_surface_frame.as_ref())
                .is_some_and(|(published, actual)| Arc::ptr_eq(published, actual))
    })
}

pub(in crate::ui::retained_host) fn assert_module_tools_menu_paint_once(
    presentation: &HostWindowPresentationData,
    control_id: &str,
) -> ModuleToolsPopup {
    let bounds = bounds(presentation);
    let popup = module_tools_popups(presentation, &bounds)
        .into_iter()
        .find(|popup| {
            presentation
                .workbench_window_nodes
                .get(popup.row)
                .is_some_and(|node| node.control_id.as_str() == control_id)
        })
        .expect("actual selected source owns this foreground popup");
    let node = presentation.workbench_window_nodes.get(popup.row).unwrap();
    assert!(node.source_surface_frame.is_some());
    assert!(node.surface_render_command_ref.is_some());
    let mut frame =
        HostRgbaFrame::recording_only(bounds.width.ceil() as u32, bounds.height.ceil() as u32);
    draw_workbench_presentation_commands(&mut frame, presentation);
    let recorded = frame.into_recorded_frame();
    let expected = crate::ui::retained_host::host_contract::paint_template_nodes::record_popup_menu_rows_for_test(
        node, &popup.frame, &popup.clip, bounds.width.ceil() as u32, bounds.height.ceil() as u32,
    );
    let visible = popup
        .visible_frame()
        .expect("the actual popup has visible pixels");
    assert!(
        recorded.commands.iter().any(|command| {
            same_source(command, &recorded, node)
                && matches!(command.kind, HostRecordedPaintKind::Quad { .. })
                && crate::ui::retained_host::host_contract::paint_geometry::intersect(
                    &command.frame,
                    &visible,
                )
                .is_some()
        }),
        "the real foreground background covers its visible source-bound region"
    );
    for row in 0..node.structured_menu_items.row_count() {
        let item = node.structured_menu_items.get(row).unwrap();
        let row_frame = menu_item_row_frame(
            node,
            &popup.frame,
            node.structured_menu_items.row_count(),
            row,
        )
        .unwrap();
        let labels = recorded.commands.iter().filter(|command| {
            same_source(command, &recorded, node)
                && matches!(&command.kind, HostRecordedPaintKind::Text { text, .. } if text == item.label.as_str())
        }).count();
        let expected_labels = expected.commands.iter().filter(|command| {
            matches!(&command.kind, HostRecordedPaintKind::Text { text, .. } if text == item.label.as_str())
        }).count();
        assert_eq!(
            labels, expected_labels,
            "row {} keeps the actual label eligibility and exact count, without a duplicate pane pass",
            item.action_id
        );
        for border in [false, true] {
            let is_surface = |kind: &HostRecordedPaintKind| match kind {
                HostRecordedPaintKind::Quad { .. } => !border,
                HostRecordedPaintKind::Border { .. } => border,
                _ => false,
            };
            let actual_surfaces = recorded
                .commands
                .iter()
                .filter(|command| {
                    same_source(command, &recorded, node)
                        && command.frame == row_frame
                        && is_surface(&command.kind)
                })
                .count();
            let expected_surfaces = expected
                .commands
                .iter()
                .filter(|command| command.frame == row_frame && is_surface(&command.kind))
                .count();
            assert_eq!(
                actual_surfaces, expected_surfaces,
                "row {} keeps the exact source-bound surface count from its actual style",
                item.action_id
            );
        }
    }
    for command in recorded
        .commands
        .iter()
        .filter(|command| same_source(command, &recorded, node))
    {
        let clip = command
            .clip_frame
            .as_ref()
            .expect("foreground commands keep their effective clip");
        assert!(clip.x >= popup.clip.x && clip.y >= popup.clip.y);
        assert!(clip.x + clip.width <= popup.clip.x + popup.clip.width);
        assert!(clip.y + clip.height <= popup.clip.y + popup.clip.height);
    }
    popup
}

pub(in crate::ui::retained_host) fn assert_control_source_paint_and_hit(
    presentation: &HostWindowPresentationData,
    control_id: &str,
) {
    let node = presentation
        .workbench_window_nodes
        .iter()
        .find(|node| node.control_id.as_str() == control_id)
        .unwrap();
    let bounds = bounds(presentation);
    let mut frame =
        HostRgbaFrame::recording_only(bounds.width.ceil() as u32, bounds.height.ceil() as u32);
    draw_workbench_presentation_commands(&mut frame, presentation);
    let recorded = frame.into_recorded_frame();
    assert!(
        recorded
            .commands
            .iter()
            .any(|command| command.frame.width > 0.0
                && command.frame.height > 0.0
                && same_source(command, &recorded, node)),
        "{control_id} must emit actual live source paint"
    );
    let index = HostWorkbenchHitIndex::from_presentation(presentation);
    let hit = hit_test_workbench_window_template_node_with_index(
        presentation,
        &index,
        node.frame.x + node.frame.width * 0.5,
        node.frame.y + node.frame.height * 0.5,
    )
    .unwrap();
    assert_eq!(hit.control_id.as_str(), control_id);
}

pub(in crate::ui::retained_host) fn assert_retired_source_has_no_paint(
    presentation: &HostWindowPresentationData,
    previous: &Arc<UiSurfaceFrame>,
    reference: UiRenderFrameCommandRef,
) {
    let bounds = bounds(presentation);
    let mut frame =
        HostRgbaFrame::recording_only(bounds.width.ceil() as u32, bounds.height.ceil() as u32);
    draw_workbench_presentation_commands(&mut frame, presentation);
    let recorded = frame.into_recorded_frame();
    assert!(!recorded
        .commands
        .iter()
        .any(|command| command
            .source
            .is_some_and(|source| source.command_ref == reference
                && recorded
                    .render_sources
                    .resolve(source.surface_key)
                    .is_some_and(|published| Arc::ptr_eq(published, previous)))));
}
