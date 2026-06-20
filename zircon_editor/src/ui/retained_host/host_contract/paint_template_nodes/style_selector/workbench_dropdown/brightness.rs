use super::model::WorkbenchDropdownStyle;
use super::state::is_unavailable_dropdown_state;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn apply_visual_brightness(
    style: WorkbenchDropdownStyle,
    brightness: f32,
) -> WorkbenchDropdownStyle {
    if is_unavailable_dropdown_state(style.state) {
        return style;
    }
    if !brightness.is_finite() || brightness <= 0.0 || (brightness - 1.0).abs() < 0.001 {
        return style;
    }
    let brightness = brightness.clamp(0.0, 4.0);
    WorkbenchDropdownStyle {
        surface: scaled_color(style.surface, brightness),
        border: scaled_color(style.border, brightness),
        text: scaled_color(style.text, brightness),
        chevron: scaled_color(style.chevron, brightness),
        state: style.state,
    }
}

fn scaled_color(color: [u8; 4], brightness: f32) -> [u8; 4] {
    [
        scaled_channel(color[0], brightness),
        scaled_channel(color[1], brightness),
        scaled_channel(color[2], brightness),
        color[3],
    ]
}

fn scaled_channel(value: u8, brightness: f32) -> u8 {
    (f32::from(value) * brightness).round().clamp(0.0, 255.0) as u8
}
