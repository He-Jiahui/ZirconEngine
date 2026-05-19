use std::rc::Rc;

use crate::ui::retained_host::primitives::{ModelRc, VecModel};
use crate::ui::retained_host::{
    paint_template_nodes_for_test, paint_template_nodes_for_test_with_background,
    TemplateNodeFrameData, TemplatePaneNodeData,
};
use zircon_runtime_interface::ui::style::{
    ButtonColor, ButtonInteractionState, ButtonVariant, ResolvedButtonStyle,
};

const BACKGROUND: [u8; 4] = [0, 0, 0, 255];
const SURFACE_INSET: [u8; 4] = [18, 24, 30, 255];
const SURFACE_HOVER: [u8; 4] = [47, 70, 80, 255];
const SURFACE_PRESSED: [u8; 4] = [16, 60, 74, 255];
const SURFACE_SELECTED: [u8; 4] = [15, 101, 116, 255];
const SURFACE_DISABLED: [u8; 4] = [25, 29, 34, 255];
const ACCENT: [u8; 4] = [53, 199, 208, 255];
const ACCENT_SOFT: [u8; 4] = [15, 101, 116, 255];
const BORDER: [u8; 4] = [75, 98, 109, 255];
const FOCUS_RING: [u8; 4] = [128, 234, 255, 255];
const BORDER_DISABLED: [u8; 4] = [51, 72, 82, 255];
const WARNING: [u8; 4] = [242, 184, 75, 255];
const WARNING_CONTAINER: [u8; 4] = [70, 49, 18, 255];
const ERROR: [u8; 4] = [239, 112, 102, 255];
const ERROR_CONTAINER: [u8; 4] = [76, 36, 39, 255];
const SUCCESS: [u8; 4] = [92, 190, 122, 255];
const SUCCESS_CONTAINER: [u8; 4] = [29, 71, 47, 255];
const INFO: [u8; 4] = [99, 179, 255, 255];
const INFO_CONTAINER: [u8; 4] = [24, 57, 91, 255];
const TEXT_DISABLED: [u8; 4] = [88, 101, 108, 255];
const SHADOW_BACKGROUND: [u8; 4] = [100, 100, 100, 255];
const SHADOW_ON_BACKGROUND: [u8; 4] = [71, 71, 71, 255];

#[test]
fn native_template_painter_uses_material_state_palette_for_controls() {
    let nodes = model_rc(vec![
        material_node("Default", 4.0, 4.0),
        TemplatePaneNodeData {
            control_id: "Hovered".into(),
            node_id: "Hovered.node".into(),
            hovered: true,
            frame: frame(4.0, 32.0),
            ..material_button_base()
        },
        TemplatePaneNodeData {
            control_id: "Pressed".into(),
            node_id: "Pressed.node".into(),
            pressed: true,
            frame: frame(4.0, 60.0),
            ..material_button_base()
        },
        TemplatePaneNodeData {
            control_id: "Selected".into(),
            node_id: "Selected.node".into(),
            selected: true,
            frame: frame(4.0, 88.0),
            ..material_button_base()
        },
        TemplatePaneNodeData {
            control_id: "Disabled".into(),
            node_id: "Disabled.node".into(),
            disabled: true,
            frame: frame(4.0, 116.0),
            ..material_button_base()
        },
        TemplatePaneNodeData {
            control_id: "Primary".into(),
            node_id: "Primary.node".into(),
            button_variant: "primary".into(),
            frame: frame(4.0, 144.0),
            ..material_button_base()
        },
        TemplatePaneNodeData {
            control_id: "PrimaryHovered".into(),
            node_id: "PrimaryHovered.node".into(),
            button_variant: "primary".into(),
            hovered: true,
            frame: frame(4.0, 172.0),
            ..material_button_base()
        },
        TemplatePaneNodeData {
            control_id: "Checked".into(),
            node_id: "Checked.node".into(),
            checked: true,
            frame: frame(4.0, 200.0),
            ..material_button_base()
        },
    ]);

    let bytes = paint_template_nodes_for_test(96, 232, nodes);

    assert_eq!(pixel(&bytes, 96, 8, 8), SURFACE_INSET);
    assert_eq!(pixel(&bytes, 96, 8, 36), SURFACE_HOVER);
    assert_eq!(pixel(&bytes, 96, 8, 64), SURFACE_PRESSED);
    assert_eq!(pixel(&bytes, 96, 8, 92), SURFACE_SELECTED);
    assert_eq!(pixel(&bytes, 96, 8, 120), SURFACE_DISABLED);
    assert_eq!(pixel(&bytes, 96, 8, 148), ACCENT);
    assert_eq!(pixel(&bytes, 96, 40, 144), FOCUS_RING);
    assert_eq!(pixel(&bytes, 96, 8, 176), ACCENT_SOFT);
    assert_eq!(pixel(&bytes, 96, 40, 172), FOCUS_RING);
    assert_eq!(pixel(&bytes, 96, 8, 204), SURFACE_SELECTED);
    assert_eq!(pixel(&bytes, 96, 40, 200), FOCUS_RING);
    assert_eq!(pixel(&bytes, 96, 40, 32), FOCUS_RING);
    assert_eq!(pixel(&bytes, 96, 40, 88), FOCUS_RING);
}

#[test]
fn native_template_painter_uses_material_validation_and_text_state_palette() {
    let nodes = model_rc(vec![
        TemplatePaneNodeData {
            control_id: "Warning".into(),
            node_id: "Warning.node".into(),
            validation_level: "warning".into(),
            text: "".into(),
            frame: frame(4.0, 4.0),
            ..material_button_base()
        },
        TemplatePaneNodeData {
            control_id: "Error".into(),
            node_id: "Error.node".into(),
            validation_level: "error".into(),
            text: "".into(),
            frame: frame(4.0, 32.0),
            ..material_button_base()
        },
        TemplatePaneNodeData {
            control_id: "DisabledText".into(),
            node_id: "DisabledText.node".into(),
            disabled: true,
            text: "Disabled".into(),
            frame: frame(4.0, 60.0),
            ..material_button_base()
        },
        TemplatePaneNodeData {
            control_id: "Success".into(),
            node_id: "Success.node".into(),
            validation_level: "success".into(),
            text: "".into(),
            frame: frame(4.0, 88.0),
            ..material_button_base()
        },
        TemplatePaneNodeData {
            control_id: "Info".into(),
            node_id: "Info.node".into(),
            surface_variant: "info".into(),
            text: "".into(),
            frame: frame(4.0, 116.0),
            ..material_button_base()
        },
    ]);

    let bytes = paint_template_nodes_for_test(96, 144, nodes);

    assert_eq!(pixel(&bytes, 96, 8, 8), WARNING_CONTAINER);
    assert_eq!(pixel(&bytes, 96, 40, 4), WARNING);
    assert_eq!(pixel(&bytes, 96, 8, 36), ERROR_CONTAINER);
    assert_eq!(pixel(&bytes, 96, 40, 32), ERROR);
    assert!(
        region_contains_color(&bytes, 96, 8, 64, 56, 12, TEXT_DISABLED),
        "disabled text should use the Material disabled text color"
    );
    assert_eq!(pixel(&bytes, 96, 8, 92), SUCCESS_CONTAINER);
    assert_eq!(pixel(&bytes, 96, 40, 88), SUCCESS);
    assert_eq!(pixel(&bytes, 96, 8, 120), INFO_CONTAINER);
    assert_eq!(pixel(&bytes, 96, 40, 116), INFO);
}

#[test]
fn native_template_painter_applies_rounded_material_corners() {
    let nodes = model_rc(vec![material_node("Rounded", 4.0, 4.0)]);

    let bytes = paint_template_nodes_for_test(96, 32, nodes);

    assert_eq!(pixel(&bytes, 96, 4, 4), BACKGROUND);
    assert_eq!(pixel(&bytes, 96, 40, 4), BORDER);
    assert_eq!(pixel(&bytes, 96, 8, 8), SURFACE_INSET);
}

#[test]
fn native_template_painter_uses_resolved_button_style_for_material_buttons() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "ResolvedContained".into(),
        node_id: "ResolvedContained.node".into(),
        button_style: ResolvedButtonStyle {
            variant: ButtonVariant::Contained,
            color: ButtonColor::Primary,
            ..ResolvedButtonStyle::default()
        },
        frame: frame(4.0, 4.0),
        ..material_button_base()
    }]);

    let bytes = paint_template_nodes_for_test(96, 32, nodes);

    assert_eq!(pixel(&bytes, 96, 8, 8), ACCENT);
    assert_eq!(pixel(&bytes, 96, 40, 4), FOCUS_RING);
}

#[test]
fn native_template_painter_resolves_button_variants_and_interaction_priority() {
    let nodes = model_rc(vec![
        styled_button_node(
            "ContainedPrimary",
            4.0,
            button_style(
                ButtonVariant::Contained,
                ButtonColor::Primary,
                ButtonInteractionState::Normal,
            ),
        ),
        styled_button_node(
            "ContainedError",
            32.0,
            button_style(
                ButtonVariant::Contained,
                ButtonColor::Error,
                ButtonInteractionState::Normal,
            ),
        ),
        styled_button_node(
            "OutlinedPrimary",
            60.0,
            button_style(
                ButtonVariant::Outlined,
                ButtonColor::Primary,
                ButtonInteractionState::Normal,
            ),
        ),
        styled_button_node(
            "OutlinedPressed",
            88.0,
            button_style(
                ButtonVariant::Outlined,
                ButtonColor::Primary,
                ButtonInteractionState::Pressed,
            ),
        ),
        styled_button_node(
            "OutlinedFocused",
            116.0,
            button_style(
                ButtonVariant::Outlined,
                ButtonColor::Primary,
                ButtonInteractionState::Focused,
            ),
        ),
        TemplatePaneNodeData {
            disabled: true,
            ..styled_button_node(
                "DisabledStyle",
                144.0,
                button_style(
                    ButtonVariant::Contained,
                    ButtonColor::Primary,
                    ButtonInteractionState::Normal,
                ),
            )
        },
        TemplatePaneNodeData {
            button_variant: "text".into(),
            border_width: 0.0,
            ..styled_button_node(
                "TextStyle",
                172.0,
                button_style(
                    ButtonVariant::Text,
                    ButtonColor::Primary,
                    ButtonInteractionState::Normal,
                ),
            )
        },
    ]);

    let bytes = paint_template_nodes_for_test(96, 200, nodes);

    assert_eq!(pixel(&bytes, 96, 8, 8), ACCENT);
    assert_eq!(pixel(&bytes, 96, 40, 4), FOCUS_RING);
    assert_eq!(pixel(&bytes, 96, 8, 36), ERROR_CONTAINER);
    assert_eq!(pixel(&bytes, 96, 40, 32), ERROR);
    assert_eq!(pixel(&bytes, 96, 8, 64), SURFACE_INSET);
    assert_eq!(pixel(&bytes, 96, 40, 60), FOCUS_RING);
    assert_eq!(pixel(&bytes, 96, 8, 92), SURFACE_PRESSED);
    assert_eq!(pixel(&bytes, 96, 40, 88), FOCUS_RING);
    assert_eq!(pixel(&bytes, 96, 8, 120), SURFACE_SELECTED);
    assert_eq!(pixel(&bytes, 96, 40, 116), FOCUS_RING);
    assert_eq!(pixel(&bytes, 96, 8, 148), SURFACE_DISABLED);
    assert_eq!(pixel(&bytes, 96, 40, 144), BORDER_DISABLED);
    assert_eq!(pixel(&bytes, 96, 8, 176), BACKGROUND);
}

#[test]
fn native_template_painter_projects_fab_pill_radius_and_elevation_shadow() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "Fab".into(),
        node_id: "Fab.node".into(),
        role: "Button".into(),
        text: "".into(),
        surface_variant: "elevated".into(),
        corner_radius: 999.0,
        border_width: 0.0,
        elevation: 2.0,
        button_style: button_style(
            ButtonVariant::Contained,
            ButtonColor::Primary,
            ButtonInteractionState::Normal,
        ),
        frame: TemplateNodeFrameData {
            x: 8.0,
            y: 8.0,
            width: 34.0,
            height: 34.0,
        },
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test_with_background(64, 64, SHADOW_BACKGROUND, nodes);

    assert_eq!(pixel(&bytes, 64, 8, 8), SHADOW_BACKGROUND);
    assert_eq!(pixel(&bytes, 64, 24, 24), ACCENT);
    assert_eq!(pixel(&bytes, 64, 42, 36), SHADOW_ON_BACKGROUND);
}

fn material_node(control_id: &str, x: f32, y: f32) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        node_id: format!("{control_id}.node").into(),
        frame: frame(x, y),
        ..material_button_base()
    }
}

fn styled_button_node(
    control_id: &str,
    y: f32,
    button_style: ResolvedButtonStyle,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        node_id: format!("{control_id}.node").into(),
        role: "Button".into(),
        text: "".into(),
        border_width: 1.0,
        corner_radius: 10.0,
        button_style,
        frame: frame(4.0, y),
        ..TemplatePaneNodeData::default()
    }
}

fn button_style(
    variant: ButtonVariant,
    color: ButtonColor,
    interaction_state: ButtonInteractionState,
) -> ResolvedButtonStyle {
    ResolvedButtonStyle {
        variant,
        color,
        interaction_state,
        ..ResolvedButtonStyle::default()
    }
}

fn material_button_base() -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        role: "Button".into(),
        text: "".into(),
        surface_variant: "inset".into(),
        border_width: 1.0,
        corner_radius: 10.0,
        ..TemplatePaneNodeData::default()
    }
}

fn frame(x: f32, y: f32) -> TemplateNodeFrameData {
    TemplateNodeFrameData {
        x,
        y,
        width: 72.0,
        height: 20.0,
    }
}

fn pixel(bytes: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
    let offset = ((y as usize * width as usize) + x as usize) * 4;
    [
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]
}

fn region_contains_color(
    bytes: &[u8],
    width: u32,
    x: u32,
    y: u32,
    region_width: u32,
    region_height: u32,
    expected: [u8; 4],
) -> bool {
    let y1 = y.saturating_add(region_height);
    let x1 = x.saturating_add(region_width);
    (y..y1).any(|row| (x..x1).any(|column| pixel(bytes, width, column, row) == expected))
}

fn model_rc<T: Clone + 'static>(values: Vec<T>) -> ModelRc<T> {
    ModelRc::from(Rc::new(VecModel::from(values)))
}
