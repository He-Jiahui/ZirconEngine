use super::super::super::data::{FrameRect, HostTextInputFocusData};
use super::super::super::paint_theme::{HostControlMetrics, HostMaterialPalette};
use super::super::render_commands::HostPaintCommand;
use super::field_control::push_settings_field_control;

#[allow(clippy::too_many_arguments)]
pub(super) fn push_chord_control(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    setting_key: &str,
    value_text: &str,
    input_focus: Option<&HostTextInputFocusData>,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    palette: HostMaterialPalette,
    metrics: HostControlMetrics,
) {
    let focused = input_focus.is_some_and(|focus| {
        focus.control_id.as_str() == setting_key && focus.captures_keyboard_chord()
    });
    push_settings_field_control(
        commands,
        rect,
        value_text.to_owned(),
        focused,
        clip,
        order,
        opacity,
        palette,
        metrics,
    );
}
