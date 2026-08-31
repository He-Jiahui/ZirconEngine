mod async_loader;
mod cache;
mod image;
mod key;
mod missing;
mod pixels;

pub(in crate::ui::retained_host) use async_loader::{
    bind_visual_asset_loader, take_visual_asset_completion, unbind_visual_asset_loader,
    VisualAssetLoadCompletion,
};
pub(super) use cache::{cached_visual_asset_pixels, store_visual_asset_pixels};
pub(in crate::ui::retained_host) use cache::{
    clear_visual_asset_pixels_cache, invalidate_visual_asset_pixel_paths,
    reconcile_visual_asset_pixel_sources,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use image::load_image_from_candidates;
pub(super) use key::image_pixels_cache_key;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use missing::missing_icon_pixels;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use pixels::load_pixels_from_candidates;
pub(super) use pixels::{load_pixels_from_candidates_with_status, CandidatePixelsLoad};
