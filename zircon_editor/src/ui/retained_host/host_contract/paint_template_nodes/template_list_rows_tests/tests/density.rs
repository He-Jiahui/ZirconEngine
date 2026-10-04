use crate::ui::retained_host::host_contract::paint_theme::METRICS;

use super::super::super::super::data::{FrameRect, TemplateNodeFrameData, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_property_rows::push_property_row_text_commands;
use super::super::super::template_tree_rows::push_tree_row_commands;
use super::super::push_list_row_commands;
use super::support::list_node;

#[test]
fn list_tree_and_property_rows_use_shared_density_metrics() {
    let clip = frame(0.0, 0.0, 320.0, 80.0);
    let list_rect = frame(8.0, 6.0, 184.0, METRICS.row_height);
    let mut list_commands = Vec::new();
    assert!(push_list_row_commands(
        &mut list_commands,
        &list_node(true, false),
        &list_rect,
        &clip,
        10,
        1.0
    ));
    let list_text = text_command(&list_commands, "Selected item");
    assert_eq!(list_text.font_size, METRICS.font_body);
    assert_eq!(
        list_text.line_height,
        METRICS.line_height(METRICS.font_body)
    );
    assert_eq!(list_text.frame.x, list_rect.x + METRICS.gap_m);
    assert_eq!(list_text.frame.y, list_rect.y + METRICS.gap_s);

    let tree_rect = frame(8.0, 8.0, 260.0, METRICS.row_height);
    let mut tree_commands = Vec::new();
    assert!(push_tree_row_commands(
        &mut tree_commands,
        &tree_node(),
        &tree_rect,
        &clip,
        20,
        1.0
    ));
    let tree_text = text_command(&tree_commands, "SceneRoot");
    assert_eq!(tree_text.font_size, METRICS.font_body);
    assert_eq!(
        tree_text.line_height,
        METRICS.line_height(METRICS.font_body)
    );

    let property_rect = frame(8.0, 8.0, 260.0, METRICS.row_height);
    let mut property_commands = Vec::new();
    assert!(push_property_row_text_commands(
        &mut property_commands,
        &property_node(),
        &property_rect,
        &clip,
        30,
        1.0
    ));
    let property_label = text_command(&property_commands, "Visible");
    let property_value = text_command(&property_commands, "true");
    let property_field = first_quad_with_background(&property_commands);
    assert_eq!(property_label.font_size, METRICS.font_body);
    assert_eq!(property_value.font_size, METRICS.font_body);
    assert_eq!(property_field.corner_radius, METRICS.radius_control);
}

fn text_command<'a>(commands: &'a [HostPaintCommand], text: &str) -> &'a HostPaintCommand {
    commands
        .iter()
        .find(|command| command.text.as_deref() == Some(text))
        .expect("expected row text command")
}

fn first_quad_with_background(commands: &[HostPaintCommand]) -> &HostPaintCommand {
    commands
        .iter()
        .find(|command| command.text.is_none() && command.background_color.is_some())
        .expect("expected row field surface command")
}

fn tree_node() -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: "WorkbenchSceneRootItem".into(),
        role: "TreeRow".into(),
        component_role: "tree-row".into(),
        text: "SceneRoot".into(),
        tree_depth: 1,
        expanded: true,
        selected: true,
        checked: true,
        frame: TemplateNodeFrameData {
            x: 8.0,
            y: 8.0,
            width: 260.0,
            height: METRICS.row_height,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn property_node() -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: "MeshVisibleProperty".into(),
        role: "PropertyRow".into(),
        component_role: "property-row".into(),
        text: "Visible".into(),
        value_text: "true".into(),
        frame: TemplateNodeFrameData {
            x: 8.0,
            y: 8.0,
            width: 260.0,
            height: METRICS.row_height,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn frame(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    FrameRect {
        x,
        y,
        width,
        height,
    }
}
