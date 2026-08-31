use zircon_runtime_interface::ui::surface::{
    UiRenderResourceKey, UiRenderResourceKind, UiVisualAssetRef,
};

use super::super::super::data::FrameRect;
use super::super::super::paint_geometry::intersect;
use super::super::render_commands::HostPaintCommand;
use super::super::visual_assets::{
    load_vector_visual_asset_pixels_for_size, load_visual_asset_pixels_for_size,
    raster_size_from_frame,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_image_resource_command(
    output: &mut Vec<HostPaintCommand>,
    resource: &UiRenderResourceKey,
    physical_pixel_size: Option<(f32, f32)>,
    frame: FrameRect,
    clip_frame: Option<FrameRect>,
    z_index: i32,
    opacity: f32,
) {
    if let Some(asset) = visual_asset_from_resource(resource) {
        let damage_frame = match clip_frame.as_ref() {
            Some(clip_frame) => intersect(&frame, clip_frame),
            None => Some(frame.clone()),
        };
        if let (Some((target_width, target_height)), Some(damage_frame)) = (
            raster_target_for_resource(physical_pixel_size, &frame),
            damage_frame,
        ) {
            let image = match resource.kind {
                UiRenderResourceKind::Vector => load_vector_visual_asset_pixels_for_size(
                    &asset,
                    target_width,
                    target_height,
                    Some(damage_frame),
                ),
                UiRenderResourceKind::Icon | UiRenderResourceKind::Image => {
                    load_visual_asset_pixels_for_size(
                        &asset,
                        target_width,
                        target_height,
                        Some(damage_frame),
                    )
                }
                _ => None,
            };
            if let Some(image) = image {
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

fn raster_target_for_resource(
    physical_pixel_size: Option<(f32, f32)>,
    frame: &FrameRect,
) -> Option<(u32, u32)> {
    physical_pixel_size
        .and_then(|(width, height)| raster_size_from_frame(width, height))
        .or_else(|| raster_size_from_frame(frame.width, frame.height))
}

fn visual_asset_from_resource(resource: &UiRenderResourceKey) -> Option<UiVisualAssetRef> {
    match resource.kind {
        UiRenderResourceKind::Icon => Some(UiVisualAssetRef::Icon(resource.id.clone())),
        UiRenderResourceKind::Image => Some(UiVisualAssetRef::Image(resource.id.clone())),
        UiRenderResourceKind::Vector => Some(UiVisualAssetRef::Image(resource.id.clone())),
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

#[cfg(test)]
mod tests {
    use super::{raster_target_for_resource, FrameRect};

    #[test]
    fn runtime_resource_physical_size_precedes_the_logical_frame() {
        let frame = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 24.0,
            height: 16.0,
        };

        assert_eq!(
            raster_target_for_resource(Some((36.0, 24.0)), &frame),
            Some((36, 24))
        );
        assert_eq!(raster_target_for_resource(None, &frame), Some((24, 16)));
    }
}
