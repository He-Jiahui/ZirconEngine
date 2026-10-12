use super::super::super::super::data::{
    TemplateNodeFrameData, TemplatePaneNodeData, TemplatePaneOptionData,
};
use zircon_runtime_interface::ui::style::{
    ResolvedButtonStyle, UiResolvedElementStyle, UiRgbaColor, UiStyleColor,
};

pub(super) fn dropdown_node(focused: bool) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: "WorkbenchInputDropdown".into(),
        role: "Dropdown".into(),
        component_role: "dropdown".into(),
        value_text: "Dropdown".into(),
        focused,
        frame: TemplateNodeFrameData {
            x: 12.0,
            y: 8.0,
            width: 104.0,
            height: 32.0,
        },
        ..TemplatePaneNodeData::default()
    }
}

pub(super) fn resolved_background_and_border(
    background: [u8; 4],
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
            ..UiResolvedElementStyle::default()
        },
        ..ResolvedButtonStyle::default()
    }
}

pub(super) fn option(
    id: &str,
    selected: bool,
    hovered: bool,
    special: bool,
    disabled: bool,
) -> TemplatePaneOptionData {
    TemplatePaneOptionData {
        id: id.into(),
        label: id.into(),
        selected,
        hovered,
        special,
        disabled,
        ..TemplatePaneOptionData::default()
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

pub(super) fn scaled_test_color(color: [u8; 4], brightness: f32) -> [u8; 4] {
    [
        (f32::from(color[0]) * brightness).round().clamp(0.0, 255.0) as u8,
        (f32::from(color[1]) * brightness).round().clamp(0.0, 255.0) as u8,
        (f32::from(color[2]) * brightness).round().clamp(0.0, 255.0) as u8,
        color[3],
    ]
}
