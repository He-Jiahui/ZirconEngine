mod command;
mod geometry;
mod text_style;

use zircon_runtime::rhi::{UiSurfaceDrawList, UiSurfaceImageResource, UiSurfaceImageResourceTable};

use self::command::{ui_surface_command_from_chrome, ui_surface_command_from_owned_chrome};
use self::geometry::ui_rect;
use super::ChromeCommandStream;

pub(in crate::ui::retained_host::host_contract) fn ui_surface_draw_list_from_stream(
    stream: &ChromeCommandStream,
) -> UiSurfaceDrawList {
    ui_surface_draw_list_from_stream_with_residency(stream, |_, _| false)
}

pub(in crate::ui::retained_host::host_contract) fn ui_surface_draw_list_from_stream_with_residency(
    stream: &ChromeCommandStream,
    mut is_resident: impl FnMut(&str, u64) -> bool,
) -> UiSurfaceDrawList {
    UiSurfaceDrawList::with_compact_styles_and_image_resources(
        stream.surface_size(),
        stream.damage().map(ui_rect),
        stream
            .commands()
            .iter()
            .map(|command| {
                let image_pixels_are_in_resource_table = matches!(
                    &command.kind,
                    super::ChromeCommandKind::Image { payload }
                        if stream
                            .image_resource(
                                payload.resource_key.as_str(),
                                payload.resource_generation,
                            )
                            .is_some()
                );
                ui_surface_command_from_chrome(command, image_pixels_are_in_resource_table)
            })
            .collect(),
        ui_surface_image_resources_from_borrowed_stream(stream.image_resources(), &mut is_resident),
    )
}

pub(in crate::ui::retained_host::host_contract) fn ui_surface_draw_list_from_owned_stream(
    stream: ChromeCommandStream,
) -> UiSurfaceDrawList {
    ui_surface_draw_list_from_owned_stream_with_optional_generation(stream, None, |_, _| false)
}

pub(in crate::ui::retained_host::host_contract) fn ui_surface_draw_list_from_owned_stream_with_generation(
    stream: ChromeCommandStream,
    producer_generation: u64,
) -> UiSurfaceDrawList {
    ui_surface_draw_list_from_owned_stream_with_optional_generation(
        stream,
        Some(producer_generation),
        |_, _| false,
    )
}

pub(in crate::ui::retained_host::host_contract) fn ui_surface_draw_list_from_owned_stream_with_generation_and_residency(
    stream: ChromeCommandStream,
    producer_generation: u64,
    is_resident: impl FnMut(&str, u64) -> bool,
) -> UiSurfaceDrawList {
    ui_surface_draw_list_from_owned_stream_with_optional_generation(
        stream,
        Some(producer_generation),
        is_resident,
    )
}

fn ui_surface_draw_list_from_owned_stream_with_optional_generation(
    mut stream: ChromeCommandStream,
    generation: Option<u64>,
    is_resident: impl FnMut(&str, u64) -> bool,
) -> UiSurfaceDrawList {
    stream.compact_image_resources_with_residency(is_resident);
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
    image_resources: super::ChromeImageResources,
) -> UiSurfaceImageResourceTable {
    let mut runtime_resources = UiSurfaceImageResourceTable::default();
    for (resource_key, resource) in image_resources.into_entries() {
        runtime_resources.insert(
            resource_key,
            UiSurfaceImageResource {
                generation: resource.generation,
                width: resource.width,
                height: resource.height,
                upload_bytes: resource.upload_bytes,
                rgba: resource.rgba,
            },
        );
    }
    runtime_resources
}

fn ui_surface_image_resources_from_borrowed_stream(
    resources: &super::ChromeImageResources,
    is_resident: &mut impl FnMut(&str, u64) -> bool,
) -> UiSurfaceImageResourceTable {
    let mut runtime_resources = UiSurfaceImageResourceTable::default();
    for (resource_key, generation, resource) in resources.iter() {
        if is_resident(resource_key, generation) {
            continue;
        }
        runtime_resources.insert(
            resource_key.to_string(),
            UiSurfaceImageResource {
                generation: resource.generation,
                width: resource.width,
                height: resource.height,
                upload_bytes: resource.upload_bytes,
                rgba: resource.rgba.clone(),
            },
        );
    }
    runtime_resources
}

#[cfg(test)]
mod tests;
