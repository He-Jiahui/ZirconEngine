#[path = "../loading/async_loader.rs"]
mod async_loader;
#[path = "loading/cache.rs"]
mod cache;
#[path = "loading/image.rs"]
mod image;
#[path = "loading/key.rs"]
mod key;
#[path = "loading/missing.rs"]
mod missing;
#[path = "loading/pixels.rs"]
mod pixels;

pub(in crate::ui::retained_host) use async_loader::{
    bind_visual_asset_loader, take_visual_asset_completion, unbind_visual_asset_loader,
    VisualAssetLoadCompletion,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use cache::visual_asset_source_is_current;
pub(super) use cache::{
    cached_visual_asset_pixels, cached_visual_asset_variant, store_visual_asset_pixels,
};
pub(in crate::ui::retained_host) use cache::{
    clear_visual_asset_pixels_cache, invalidate_visual_asset_pixel_paths,
    reconcile_visual_asset_pixel_sources,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use image::load_image_from_candidates;
pub(super) use key::image_pixels_cache_key;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use missing::missing_icon_pixels;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use pixels::load_pixels_from_candidates;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use pixels::{
    load_pixels_from_candidates_with_status, CandidatePixelsLoad,
};
