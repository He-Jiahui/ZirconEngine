#[path = "visual_assets/asset.rs"]
mod asset;
#[path = "visual_assets/candidates.rs"]
mod candidates;
#[path = "visual_assets/keys.rs"]
mod keys;
#[path = "visual_assets/loading.rs"]
mod loading;
#[path = "visual_assets/mui_icons.rs"]
mod mui_icons;
#[path = "visual_assets/pixels.rs"]
mod pixels;
#[path = "visual_assets/preview_artifact.rs"]
mod preview_artifact;
#[path = "visual_assets/retained.rs"]
mod retained;
#[path = "visual_assets/svg.rs"]
mod svg;
#[path = "visual_assets/target.rs"]
mod target;
#[path = "visual_assets/template.rs"]
mod template;
#[path = "visual_assets/tint.rs"]
mod tint;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use asset::{
    load_existing_icon_asset_pixels_for_size, load_vector_visual_asset_pixels_for_size,
    load_visual_asset_pixels, load_visual_asset_pixels_for_size,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use loading::visual_asset_source_is_current;
pub(in crate::ui::retained_host) use loading::{
    bind_visual_asset_loader, clear_visual_asset_pixels_cache, invalidate_visual_asset_pixel_paths,
    reconcile_visual_asset_pixel_sources, take_visual_asset_completion, unbind_visual_asset_loader,
    VisualAssetLoadCompletion,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use pixels::HostPaintImagePixels;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use preview_artifact::preview_artifact_image_pixels;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use retained::retained_image_pixels;
pub(in crate::ui::retained_host) use svg::{
    clear_svg_tree_cache, invalidate_svg_tree_paths, reconcile_svg_tree_sources,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use target::{
    raster_size_from_frame, RasterTargetSize, MAX_VECTOR_RASTER_EDGE,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use template::{
    template_image_pixels, template_vector_image_pixels,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use tint::{
    template_image_tint, tint_non_transparent_pixels, ICON_TINT, ICON_TINT_ACTIVE,
    ICON_TINT_DISABLED, ICON_TINT_ERROR, ICON_TINT_WARNING,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use keys::retained_image_resource_key;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use target::MUI_ICON_DEFAULT_EDGE;

#[cfg(test)]
#[path = "visual_assets_tests/tests/mod.rs"]
mod tests;
