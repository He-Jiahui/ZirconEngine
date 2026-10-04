use super::data::FrameRect;
use super::template_popup_layout::{
    dropdown_option_popup_frame_within, dropdown_option_row_frame_within,
};

pub(in crate::ui::retained_host::host_contract) const SETTINGS_COLOR_CHANNEL_COUNT: usize = 4;

pub(in crate::ui::retained_host::host_contract) struct SettingsColorChannelFrames {
    pub label: FrameRect,
    pub decrement: FrameRect,
    pub value: FrameRect,
    pub increment: FrameRect,
}

pub(in crate::ui::retained_host::host_contract) fn settings_color_popup_frame_within(
    control: &FrameRect,
    bounds: &FrameRect,
) -> Option<FrameRect> {
    dropdown_option_popup_frame_within(control, SETTINGS_COLOR_CHANNEL_COUNT, bounds)
}

pub(in crate::ui::retained_host::host_contract) fn settings_color_channel_frames_within(
    control: &FrameRect,
    channel: usize,
    bounds: &FrameRect,
) -> Option<SettingsColorChannelFrames> {
    let row =
        dropdown_option_row_frame_within(control, SETTINGS_COLOR_CHANNEL_COUNT, channel, bounds)?;
    let label_width = row.height.min(row.width * 0.18);
    let editable_width = row.width - label_width;
    let button_width = row.height.min(editable_width / 3.0);
    let value_width = editable_width - button_width * 2.0;
    Some(SettingsColorChannelFrames {
        label: FrameRect {
            x: row.x,
            y: row.y,
            width: label_width,
            height: row.height,
        },
        decrement: FrameRect {
            x: row.x + label_width,
            y: row.y,
            width: button_width,
            height: row.height,
        },
        value: FrameRect {
            x: row.x + label_width + button_width,
            y: row.y,
            width: value_width,
            height: row.height,
        },
        increment: FrameRect {
            x: row.x + row.width - button_width,
            y: row.y,
            width: button_width,
            height: row.height,
        },
    })
}

#[cfg(test)]
#[path = "tests/settings_color_editor_geometry.rs"]
mod tests;
