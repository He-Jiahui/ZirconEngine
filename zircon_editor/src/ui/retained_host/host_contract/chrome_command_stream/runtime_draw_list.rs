mod command;
mod geometry;
mod text_style;

use zircon_runtime::rhi::UiSurfaceDrawList;

use self::command::{ui_surface_command_from_chrome, ui_surface_command_from_owned_chrome};
use self::geometry::ui_rect;
use super::ChromeCommandStream;

pub(in crate::ui::retained_host::host_contract) fn ui_surface_draw_list_from_stream(
    stream: &ChromeCommandStream,
) -> UiSurfaceDrawList {
    UiSurfaceDrawList::new(
        stream.surface_size(),
        stream.damage().map(ui_rect),
        stream
            .commands()
            .iter()
            .map(ui_surface_command_from_chrome)
            .collect(),
    )
}

pub(in crate::ui::retained_host::host_contract) fn ui_surface_draw_list_from_owned_stream(
    stream: ChromeCommandStream,
) -> UiSurfaceDrawList {
    ui_surface_draw_list_from_owned_stream_with_optional_generation(stream, None)
}

pub(in crate::ui::retained_host::host_contract) fn ui_surface_draw_list_from_owned_stream_with_generation(
    stream: ChromeCommandStream,
    producer_generation: u64,
) -> UiSurfaceDrawList {
    ui_surface_draw_list_from_owned_stream_with_optional_generation(
        stream,
        Some(producer_generation),
    )
}

fn ui_surface_draw_list_from_owned_stream_with_optional_generation(
    stream: ChromeCommandStream,
    generation: Option<u64>,
) -> UiSurfaceDrawList {
    let surface_size = stream.surface_size();
    let damage = stream.damage().map(ui_rect);
    let commands = stream
        .into_commands()
        .into_iter()
        .map(ui_surface_command_from_owned_chrome)
        .collect();
    match generation {
        Some(generation) => {
            UiSurfaceDrawList::with_generation(surface_size, damage, commands, generation)
        }
        None => UiSurfaceDrawList::new(surface_size, damage, commands),
    }
}

#[cfg(test)]
mod tests;
