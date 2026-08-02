mod command;
mod geometry;
mod text_style;

use zircon_runtime::rhi::{UiSurfaceDrawList, UiSurfaceImageResource};

use self::command::{ui_surface_command_from_chrome, ui_surface_command_from_owned_chrome};
use self::geometry::ui_rect;
use super::ChromeCommandStream;

pub(in crate::ui::retained_host::host_contract) fn ui_surface_draw_list_from_stream(
    stream: &ChromeCommandStream,
) -> UiSurfaceDrawList {
    UiSurfaceDrawList::with_compact_styles_and_image_resources(
        stream.surface_size(),
        stream.damage().map(ui_rect),
        stream
            .commands()
            .iter()
            .map(ui_surface_command_from_chrome)
            .collect(),
        ui_surface_image_resources_from_stream(stream.image_resources().clone()),
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
    mut stream: ChromeCommandStream,
    generation: Option<u64>,
) -> UiSurfaceDrawList {
    stream.compact_image_resources();
    let surface_size = stream.surface_size();
    let damage = stream.damage().map(ui_rect);
    let (commands, image_resources) = stream.into_parts();
    let commands = commands
        .into_iter()
        .map(ui_surface_command_from_owned_chrome)
        .collect();
    let image_resources = ui_surface_image_resources_from_stream(image_resources);
    match generation {
        Some(generation) => {
            UiSurfaceDrawList::with_generation_and_compact_styles_and_image_resources(
                surface_size,
                damage,
                commands,
                generation,
                image_resources,
            )
        }
        None => UiSurfaceDrawList::with_compact_styles_and_image_resources(
            surface_size,
            damage,
            commands,
            image_resources,
        ),
    }
}

fn ui_surface_image_resources_from_stream(
    image_resources: impl IntoIterator<Item = (String, super::ChromeImageResource)>,
) -> std::collections::HashMap<String, UiSurfaceImageResource> {
    image_resources
        .into_iter()
        .map(|(resource_key, resource)| {
            (
                resource_key,
                UiSurfaceImageResource {
                    generation: resource.generation,
                    width: resource.width,
                    height: resource.height,
                    upload_bytes: resource.upload_bytes,
                    rgba: resource.rgba,
                },
            )
        })
        .collect()
}

#[cfg(test)]
mod tests;
