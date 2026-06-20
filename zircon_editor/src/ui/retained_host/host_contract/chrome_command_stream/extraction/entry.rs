use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};
use crate::ui::retained_host::host_contract::paint_recording::record_host_frame_commands;

use super::command::chrome_command_from_recorded;
use super::model::ChromeCommandExtraction;

pub(in crate::ui::retained_host::host_contract) fn extract_chrome_commands(
    presentation: &HostWindowPresentationData,
    surface_size: (u32, u32),
    damage: Option<&FrameRect>,
    include_image_bytes: bool,
) -> ChromeCommandExtraction {
    let (recorded_commands, clipped_damage) =
        record_host_frame_commands(surface_size.0, surface_size.1, presentation, damage);
    let full_rebuild = clipped_damage.is_none();
    let commands = recorded_commands
        .into_iter()
        .filter_map(|command| {
            chrome_command_from_recorded(command, full_rebuild, include_image_bytes)
        })
        .collect();
    ChromeCommandExtraction {
        commands,
        clipped_damage,
    }
}
