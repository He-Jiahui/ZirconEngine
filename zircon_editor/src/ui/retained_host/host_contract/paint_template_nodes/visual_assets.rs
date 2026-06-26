mod asset;
mod candidates;
mod keys;
mod loading;
mod mui_icons;
mod pixels;
mod retained;
mod svg;
mod target;
mod template;
mod tint;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use asset::{
    load_existing_icon_asset_pixels_for_size, load_visual_asset_pixels,
    load_visual_asset_pixels_for_size,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use pixels::HostPaintImagePixels;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use retained::retained_image_pixels;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use target::{
    raster_size_from_frame, RasterTargetSize, MAX_VECTOR_RASTER_EDGE,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use template::template_image_pixels;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use tint::{
    template_image_tint, tint_non_transparent_pixels, ICON_TINT, ICON_TINT_ACTIVE,
    ICON_TINT_DISABLED, ICON_TINT_ERROR, ICON_TINT_WARNING,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use keys::retained_image_resource_key;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use target::MUI_ICON_DEFAULT_EDGE;

#[cfg(test)]
#[path = "visual_assets_tests/mod.rs"]
mod tests;
