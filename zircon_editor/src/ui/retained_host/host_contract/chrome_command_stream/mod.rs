use super::data::{FrameRect, HostWindowPresentationData};
pub(in crate::ui::retained_host::host_contract) use command::{
    ChromeCommand, ChromeCommandKind, ChromeCommandLayer, ChromeImagePayload, ChromeImageUvRect,
};
use extraction::extract_chrome_commands;
pub(in crate::ui::retained_host) use icon_atlas::invalidate_editor_icon_atlas;
pub(in crate::ui::retained_host::host_contract) use replay::{
    paint_chrome_command_stream_to_frame, repaint_chrome_command_stream_region,
};
pub(in crate::ui::retained_host::host_contract) use runtime_draw_list::{
    ui_surface_draw_list_from_owned_stream, ui_surface_draw_list_from_owned_stream_with_generation,
    ui_surface_draw_list_from_owned_stream_with_generation_and_residency,
    ui_surface_draw_list_from_owned_stream_with_residency, ui_surface_draw_list_from_stream,
    ui_surface_draw_list_from_stream_with_residency,
};
use stream::clamp_surface_size;
pub(in crate::ui::retained_host::host_contract) use stream::{
    ChromeCommandStream, ChromeImageResource, ChromeImageResources,
};

#[cfg(test)]
use extraction::chrome_command_from_recorded_for_test;

mod atlas;
mod command;
mod extraction;
mod icon_atlas;
mod replay;
mod runtime_draw_list;
mod stats;
mod stream;

pub(in crate::ui::retained_host::host_contract) fn build_chrome_command_stream(
    presentation: &HostWindowPresentationData,
    surface_size: (u32, u32),
    damage: Option<&FrameRect>,
    include_image_bytes: bool,
) -> ChromeCommandStream {
    build_chrome_command_stream_with_residency(
        presentation,
        surface_size,
        damage,
        include_image_bytes,
        |_, _| false,
    )
}

pub(in crate::ui::retained_host::host_contract) fn build_chrome_command_stream_with_residency(
    presentation: &HostWindowPresentationData,
    surface_size: (u32, u32),
    damage: Option<&FrameRect>,
    include_image_bytes: bool,
    is_resident: impl FnMut(&str, u64) -> bool,
) -> ChromeCommandStream {
    let surface_size = clamp_surface_size(surface_size);
    let mut extraction =
        extract_chrome_commands(presentation, surface_size, damage, include_image_bytes);
    icon_atlas::pack_editor_icons_into_atlas(&mut extraction.commands);
    let mut stream = if let Some(damage) = extraction.clipped_damage.clone() {
        ChromeCommandStream::patch(surface_size, damage.clone())
    } else {
        ChromeCommandStream::full_rebuild(surface_size)
    };
    if let Some(damage) = extraction.clipped_damage {
        stream.push_clip(ChromeCommandLayer::Dynamic, 0, damage);
    }
    stream.extend_commands(extraction.commands);
    stream.compact_image_resources_with_residency(is_resident);
    stream
}

#[cfg(test)]
mod atlas_tests;

#[cfg(test)]
mod tests;
