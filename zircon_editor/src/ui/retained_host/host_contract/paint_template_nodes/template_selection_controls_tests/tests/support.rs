use super::super::super::super::data::{TemplateNodeFrameData, TemplatePaneNodeData};
use zircon_runtime_interface::ui::style::{
    ResolvedButtonStyle, UiResolvedElementStyle, UiRgbaColor, UiStyleColor,
};

pub(super) const SELECTION_MARK_IDLE_FILL: [u8; 4] = super::super::SELECTION_MARK_IDLE_FILL;

pub(super) fn node_with_role(
    role: &str,
    component_role: &str,
    control_id: &str,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: role.into(),
        component_role: component_role.into(),
        frame: TemplateNodeFrameData {
            x: 0.0,
            y: 0.0,
            width: 80.0,
            height: 28.0,
        },
        ..TemplatePaneNodeData::default()
    }
}

pub(super) fn checkbox_node() -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: "WorkbenchCheckboxOn".into(),
        role: "Checkbox".into(),
        component_role: "checkbox".into(),
        text: "Checkbox".into(),
        checked: true,
        selected: true,
        frame: TemplateNodeFrameData {
            x: 0.0,
            y: 0.0,
            width: 96.0,
            height: 28.0,
        },
        ..TemplatePaneNodeData::default()
    }
}

pub(super) fn unchecked_checkbox_node() -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: "WorkbenchCheckboxOff".into(),
        role: "Checkbox".into(),
        component_role: "checkbox".into(),
        text: "Checkbox".into(),
        frame: TemplateNodeFrameData {
            x: 0.0,
            y: 0.0,
            width: 96.0,
            height: 28.0,
        },
        ..TemplatePaneNodeData::default()
    }
}

pub(super) fn changed_pixel_count(
    bytes: &[u8],
    frame_width: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> usize {
    let mut changed = 0;
    for py in y..(y + height) {
        for px in x..(x + width) {
            let index = ((py as usize * frame_width as usize) + px as usize) * 4;
            if bytes[index..index + 4] != [0, 0, 0, 255] {
                changed += 1;
            }
        }
    }
    changed
}

pub(super) fn pixel_at(bytes: &[u8], frame_width: u32, x: u32, y: u32) -> [u8; 4] {
    let index = ((y as usize * frame_width as usize) + x as usize) * 4;
    [
        bytes[index],
        bytes[index + 1],
        bytes[index + 2],
        bytes[index + 3],
    ]
}

pub(super) fn resolved_background_and_border(
    background: [u8; 4],
    border: [u8; 4],
) -> ResolvedButtonStyle {
    resolved_background_foreground_and_border(background, [0, 0, 0, 0], border)
}

pub(super) fn resolved_background_foreground_and_border(
    background: [u8; 4],
    foreground: [u8; 4],
    border: [u8; 4],
) -> ResolvedButtonStyle {
    ResolvedButtonStyle {
        element: UiResolvedElementStyle {
            background_color: Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(
                background[0],
                background[1],
                background[2],
                background[3],
            ))),
            border_color: Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(
                border[0], border[1], border[2], border[3],
            ))),
            foreground_color: Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(
                foreground[0],
                foreground[1],
                foreground[2],
                foreground[3],
            ))),
            ..UiResolvedElementStyle::default()
        },
        ..ResolvedButtonStyle::default()
    }
}
