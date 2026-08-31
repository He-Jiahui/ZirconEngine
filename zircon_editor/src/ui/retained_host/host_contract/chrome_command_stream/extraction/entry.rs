use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};
use crate::ui::retained_host::host_contract::paint_recording::record_host_frame_commands;

use super::super::command::{ChromeCommand, ChromeCommandKind, ChromeCommandLayer};
use super::command::chrome_command_from_recorded;
use super::model::ChromeCommandExtraction;

pub(in crate::ui::retained_host::host_contract) fn extract_chrome_commands(
    presentation: &HostWindowPresentationData,
    surface_size: (u32, u32),
    damage: Option<&FrameRect>,
    include_image_bytes: bool,
) -> ChromeCommandExtraction {
    let (recorded_frame, clipped_damage) =
        record_host_frame_commands(surface_size.0, surface_size.1, presentation, damage);
    let full_rebuild = clipped_damage.is_none();
    let damage_clip = clipped_damage.as_ref().map(damage_clip_command);
    let commands = {
        zircon_runtime::profile_scope!("editor", "host_painter", "chrome_extract_commands");
        damage_clip
            .into_iter()
            .chain(recorded_frame.commands.into_iter().filter_map(|command| {
                chrome_command_from_recorded(command, full_rebuild, include_image_bytes)
            }))
            .collect()
    };
    ChromeCommandExtraction {
        commands,
        clipped_damage,
        render_sources: recorded_frame.render_sources,
    }
}

fn damage_clip_command(frame: &FrameRect) -> ChromeCommand {
    ChromeCommand {
        layer: ChromeCommandLayer::Dynamic,
        z_index: 0,
        frame: frame.clone(),
        clip: Some(frame.clone()),
        source: None,
        kind: ChromeCommandKind::Clip,
    }
}
