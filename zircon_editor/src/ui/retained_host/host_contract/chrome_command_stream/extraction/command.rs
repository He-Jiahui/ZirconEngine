mod kind;
mod layer;

use super::super::command::ChromeCommand;
use super::visibility::visible_frame;
use crate::ui::retained_host::host_contract::paint_frame::HostRecordedPaintCommand;
use kind::chrome_command_kind_from_recorded;
use layer::chrome_command_layer_from_recorded;

pub(super) fn chrome_command_from_recorded(
    command: HostRecordedPaintCommand,
    full_rebuild: bool,
    include_image_bytes: bool,
) -> Option<ChromeCommand> {
    if !visible_frame(&command.frame) {
        return None;
    }
    let layer = chrome_command_layer_from_recorded(&command.kind, full_rebuild);
    let kind = chrome_command_kind_from_recorded(command.kind, include_image_bytes);
    Some(ChromeCommand {
        layer,
        z_index: command.z_index,
        frame: command.frame,
        clip: command.clip_frame,
        source: command.source,
        kind,
    })
}

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract) fn chrome_command_from_recorded_for_test(
    command: HostRecordedPaintCommand,
    full_rebuild: bool,
    include_image_bytes: bool,
) -> Option<ChromeCommand> {
    chrome_command_from_recorded(command, full_rebuild, include_image_bytes)
}
