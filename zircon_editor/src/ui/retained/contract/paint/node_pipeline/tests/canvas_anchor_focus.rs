//! Consumer regressions, not native-window, source-codec or GUI acceptance.
use super::*;
use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::host_contract::data::{
    ui_asset_canvas_anchor_opener_control_id, TemplateNodeFrameData,
};
use crate::ui::retained_host::host_contract::paint_frame::{
    HostRecordedPaintCommand, HostRecordedPaintKind,
};
use std::sync::Arc;

fn opener(instance: &str) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: format!("fixture/{instance}/canvas-anchor-opener").into(),
        source_path: "fixture://canvas-focus-consumer".into(),
        source_node_id: "slot-details".into(),
        instance_path: format!("fixture/{instance}/slot-details").into(),
        control_id: ui_asset_canvas_anchor_opener_control_id(instance),
        role: "Button".into(),
        component_role: "button".into(),
        button_variant: "secondary".into(),
        action_id: "canvas.anchor.open".into(),
        dispatch_kind: "ui_asset_anchor_popup_open".into(),
        frame: TemplateNodeFrameData {
            x: 5.0,
            y: 7.0,
            width: 80.0,
            height: 24.0,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn origin(x: f32, y: f32) -> FrameRect {
    FrameRect {
        x,
        y,
        width: 180.0,
        height: 90.0,
    }
}

fn returned_focus(node: &TemplatePaneNodeData, origin: &FrameRect) -> HostPaneInteractionStateData {
    HostPaneInteractionStateData {
        focused_template_control_id: node.control_id.clone(),
        template_button_frame: FrameRect {
            x: origin.x + node.frame.x,
            y: origin.y + node.frame.y,
            width: node.frame.width,
            height: node.frame.height,
        },
        template_focus_visible: true,
        template_focus_observed: true,
        ..HostPaneInteractionStateData::default()
    }
}

fn record(
    node: &TemplatePaneNodeData,
    origin: &FrameRect,
    interaction: Option<&HostPaneInteractionStateData>,
    transform: Option<&dyn TemplateNodePaintTransform>,
) -> Vec<HostRecordedPaintCommand> {
    let nodes = model_rc(vec![node.clone()]);
    let mut frame = HostRgbaFrame::recording_only(256, 128);
    if let Some(interaction) = interaction {
        frame.set_pane_interaction_state(Arc::new(interaction.clone()));
    }
    let clip = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 256.0,
        height: 128.0,
    };
    draw_template_nodes_with_transform(&mut frame, &nodes, origin, &clip, None, transform);
    assert!(
        !nodes.get(0).expect("original row").focused,
        "drawing must not materialize focus into the shared node model"
    );
    let commands = frame.into_recorded_commands();
    assert!(
        !commands.is_empty(),
        "the real shared Button painter must produce commands"
    );
    commands
}

// An authored focus oracle intentionally uses no host interaction. It goes
// through the same production painter and shared theme, not a copied palette.
fn focused_commands(
    node: &TemplatePaneNodeData,
    origin: &FrameRect,
) -> Vec<HostRecordedPaintCommand> {
    let mut focused = node.clone();
    focused.focused = true;
    focused.focus_visible = true;
    focused.focus_visible_known = true;
    let nodes = model_rc(vec![focused]);
    let mut frame = HostRgbaFrame::recording_only(256, 128);
    let clip = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 256.0,
        height: 128.0,
    };
    draw_template_nodes(&mut frame, &nodes, origin, &clip, None);
    frame.into_recorded_commands()
}

#[test]
fn returned_canvas_focus_reaches_existing_button_commands_without_changing_source_rows() {
    let node = opener("view-a");
    let origin = origin(20.0, 10.0);
    let baseline = record(&node, &origin, None, None);
    let actual = record(&node, &origin, Some(&returned_focus(&node, &origin)), None);
    assert_ne!(
        actual, baseline,
        "semantic focus alone must not pass without a changed painted border"
    );
    assert!(actual.iter().any(|command| matches!(command.kind, HostRecordedPaintKind::Border { .. })),
        "the focus consumer assertion must include the existing Button border, not merely a state flag");
    assert_eq!(actual, focused_commands(&node, &origin));
    assert!(!node.focused);
    assert!(!node.focus_visible);
}

#[test]
fn returned_canvas_focus_does_not_leak_to_another_split_document_at_the_same_rectangle() {
    let first = opener("view-a");
    let second = opener("view-b");
    assert_ne!(
        first.control_id, second.control_id,
        "the actual shared ID helper must scope by view"
    );
    let origin = origin(20.0, 10.0);
    let focus = returned_focus(&first, &origin);
    assert_eq!(
        record(&second, &origin, Some(&focus), None),
        record(&second, &origin, None, None)
    );
    assert_eq!(
        record(&first, &origin, Some(&focus), None),
        focused_commands(&first, &origin)
    );
}

#[test]
fn moved_canvas_opener_rejects_an_old_absolute_focus_rectangle() {
    let node = opener("view-a");
    let old_origin = origin(20.0, 10.0);
    let current_origin = origin(60.0, 30.0);
    let old_focus = returned_focus(&node, &old_origin);
    assert_eq!(
        record(&node, &current_origin, Some(&old_focus), None),
        record(&node, &current_origin, None, None)
    );
    let current_focus = returned_focus(&node, &current_origin);
    assert_eq!(
        record(&node, &current_origin, Some(&current_focus), None),
        focused_commands(&node, &current_origin)
    );
}

#[test]
fn invisible_focus_disabled_or_other_command_cannot_acquire_canvas_return_focus_paint() {
    let node = opener("view-a");
    let origin = origin(20.0, 10.0);
    let mut hidden_focus = returned_focus(&node, &origin);
    hidden_focus.template_focus_visible = false;
    assert_eq!(
        record(&node, &origin, Some(&hidden_focus), None),
        record(&node, &origin, None, None)
    );
    let focus = returned_focus(&node, &origin);
    let mut disabled = node.clone();
    disabled.disabled = true;
    assert_eq!(
        record(&disabled, &origin, Some(&focus), None),
        record(&disabled, &origin, None, None)
    );
    let mut other = node.clone();
    other.dispatch_kind = "surface".into();
    other.action_id = "some.other.command".into();
    assert_eq!(
        record(&other, &origin, Some(&focus), None),
        record(&other, &origin, None, None)
    );
}

struct MovedOpener;
impl TemplateNodePaintTransform for MovedOpener {
    fn patch_row(
        &self,
        _row: usize,
        node: &TemplatePaneNodeData,
        _clip: &FrameRect,
    ) -> Option<TemplatePaneNodeData> {
        let mut moved = node.clone();
        moved.frame.x += 16.0;
        moved.frame.y += 8.0;
        Some(moved)
    }
}

#[test]
fn transformed_canvas_opener_focus_uses_its_actual_visible_rectangle() {
    let node = opener("view-a");
    let origin = origin(20.0, 10.0);
    let clip = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 256.0,
        height: 128.0,
    };
    let moved = MovedOpener
        .patch_row(0, &node, &clip)
        .expect("actual transform");
    let current = returned_focus(&moved, &origin);
    let actual = record(&node, &origin, Some(&current), Some(&MovedOpener));
    assert_eq!(actual, focused_commands(&moved, &origin));
    let old = returned_focus(&node, &origin);
    assert_eq!(
        record(&node, &origin, Some(&old), Some(&MovedOpener)),
        record(&moved, &origin, None, None)
    );
}
