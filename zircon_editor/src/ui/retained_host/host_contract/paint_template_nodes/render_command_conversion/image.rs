use zircon_runtime_interface::ui::surface::{
    UiRenderResourceKey, UiRenderResourceKind, UiVisualAssetRef,
};

use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::super::visual_assets::{load_visual_asset_pixels_for_size, raster_size_from_frame};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_image_resource_command(
    output: &mut Vec<HostPaintCommand>,
    resource: &UiRenderResourceKey,
    frame: FrameRect,
    clip_frame: Option<FrameRect>,
    z_index: i32,
    opacity: f32,
) {
    if let Some(asset) = visual_asset_from_resource(resource) {
        if let Some((target_width, target_height)) =
            raster_size_from_frame(frame.width, frame.height)
        {
            if let Some(image) =
                load_visual_asset_pixels_for_size(&asset, target_width, target_height)
            {
                output.push(HostPaintCommand::image_pixels(
                    frame,
                    clip_frame,
                    z_index,
                    image.resource_key,
                    image.width,
                    image.height,
                    image.rgba,
                    image.atlas,
                    opacity,
                ));
                return;
            }
        }
    }

    output.push(HostPaintCommand::image(
        frame,
        clip_frame,
        z_index,
        resource_image_key(resource),
        opacity,
    ));
}

fn visual_asset_from_resource(resource: &UiRenderResourceKey) -> Option<UiVisualAssetRef> {
    match resource.kind {
        UiRenderResourceKind::Icon => Some(UiVisualAssetRef::Icon(resource.id.clone())),
        UiRenderResourceKind::Image | UiRenderResourceKind::Vector => {
            Some(UiVisualAssetRef::Image(resource.id.clone()))
        }
        _ => None,
    }
}

fn resource_image_key(resource: &UiRenderResourceKey) -> String {
    match resource.kind {
        UiRenderResourceKind::Icon => format!("icon:{}", resource.id),
        UiRenderResourceKind::Image | UiRenderResourceKind::Vector => {
            format!("image:{}", resource.id)
        }
        UiRenderResourceKind::Material => format!("material:{}", resource.id),
        UiRenderResourceKind::Font => format!("font:{}", resource.id),
        UiRenderResourceKind::Texture => format!("texture:{}", resource.id),
    }
}
