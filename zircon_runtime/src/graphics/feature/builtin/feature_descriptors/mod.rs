#[path = "advanced_slot.rs"]
pub(super) mod advanced_slot;
#[path = "anti_alias.rs"]
pub(super) mod anti_alias;
#[path = "bloom.rs"]
pub(super) mod bloom;
#[path = "../../../builtin/feature_descriptors/clustered_lighting.rs"]
pub(super) mod clustered_lighting;
#[path = "color_grading.rs"]
pub(super) mod color_grading;
#[path = "../../../builtin/feature_descriptors/compute_workload.rs"]
pub(super) mod compute_workload;
#[path = "debug_overlay.rs"]
pub(super) mod debug_overlay;
#[path = "../../../builtin/feature_descriptors/deferred_geometry.rs"]
pub(super) mod deferred_geometry;
#[path = "../../../builtin/feature_descriptors/deferred_lighting.rs"]
pub(super) mod deferred_lighting;
#[path = "hzb.rs"]
pub(super) mod hzb;
#[path = "mesh.rs"]
pub(super) mod mesh;
#[path = "../../../builtin/feature_descriptors/neural_compute.rs"]
pub(super) mod neural_compute;
#[path = "post.rs"]
pub(super) mod post_process;
#[path = "ray_tracing.rs"]
pub(super) mod ray_tracing;
#[path = "../../../screen_space_ambient_occlusion.rs"]
pub(super) mod screen_space_ambient_occlusion;
#[path = "shadows.rs"]
pub(super) mod shadows;
#[path = "sprite.rs"]
pub(super) mod sprite;
#[path = "temporal.rs"]
pub(super) mod temporal;
#[path = "ui.rs"]
pub(super) mod ui;

pub(super) use screen_space_ambient_occlusion::configure_for_profile as configure_screen_space_ambient_occlusion_for_profile;
pub(super) use screen_space_ambient_occlusion::descriptor as screen_space_ambient_occlusion_descriptor;
pub(crate) use screen_space_ambient_occlusion::SsaoParams;

use crate::render_graph::{RenderResourceSchema, RenderTextureExtentPolicy, RenderTextureSchema};
use crate::rhi::{TextureFormat, TextureUsage};

pub(super) fn final_output_resource_schema() -> RenderResourceSchema {
    RenderResourceSchema::texture(
        RenderTextureSchema::new(
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED | TextureUsage::COPY_SRC,
        )
        .with_extent(RenderTextureExtentPolicy::View),
    )
}
