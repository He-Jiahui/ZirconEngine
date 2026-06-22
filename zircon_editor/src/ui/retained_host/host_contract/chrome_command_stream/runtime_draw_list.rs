mod command;
mod geometry;
mod text_style;

use zircon_runtime::rhi::UiSurfaceDrawList;

use self::command::ui_surface_command_from_chrome;
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

#[cfg(test)]
mod tests;
