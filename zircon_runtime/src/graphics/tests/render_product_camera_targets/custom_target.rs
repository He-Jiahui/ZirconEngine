use super::assertions::{
    dominant_blue_pixels, dominant_blue_pixels_in_region, dominant_blue_pixels_in_rgba_region,
    dominant_green_pixels, dominant_green_pixels_in_region, dominant_green_pixels_in_rgba_region,
    dominant_red_pixels, dominant_red_pixels_in_region, dominant_red_pixels_in_rgba_region,
    is_dominant_green, is_dominant_red, rgba_pixel_at, RenderViewportRegion,
};
use super::camera::{
    camera_target_product_profile, primary_surface_camera_descriptor, texture_camera_descriptor,
    CameraDescriptorTestExt,
};
use super::fixture::RenderFixture;
use super::mesh::{
    overlay_mesh, sampled_fullscreen_mesh, sampled_fullscreen_mesh_on_layer, sampled_mesh,
};

use crate::core::framework::render::{
    CameraRenderType, RenderCameraClear, RenderCameraTargetKind, RenderCaptureSource,
    RenderFramework, RenderLayerSet, RenderViewportRect,
};
use crate::core::math::{UVec2, Vec3, Vec4};

#[path = "../custom_target/composite.rs"]
mod composite;
#[path = "../custom_target/material_sampling.rs"]
mod material_sampling;
#[path = "../custom_target/ordering.rs"]
mod ordering;
#[path = "../custom_target/viewport.rs"]
mod viewport;
