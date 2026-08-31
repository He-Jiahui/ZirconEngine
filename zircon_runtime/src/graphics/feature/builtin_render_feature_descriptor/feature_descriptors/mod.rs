pub(super) mod advanced_slot;
pub(super) mod anti_alias;
pub(super) mod bloom;
pub(super) mod clustered_lighting;
pub(super) mod color_grading;
pub(super) mod compute_workload;
pub(super) mod debug_overlay;
pub(super) mod deferred_geometry;
pub(super) mod deferred_lighting;
pub(super) mod hzb;
pub(super) mod mesh;
pub(super) mod neural_compute;
pub(super) mod post_process;
pub(super) mod ray_tracing;
pub(super) mod screen_space_ambient_occlusion;
pub(super) mod shadows;
pub(super) mod sprite;
pub(super) mod temporal;
pub(super) mod ui;

pub(crate) use screen_space_ambient_occlusion::SsaoParams;
pub(super) use screen_space_ambient_occlusion::configure_for_profile as configure_screen_space_ambient_occlusion_for_profile;
pub(super) use screen_space_ambient_occlusion::descriptor as screen_space_ambient_occlusion_descriptor;

use crate::render_graph::{RenderResourceSchema, RenderTextureExtentPolicy, RenderTextureSchema};
use crate::rhi::{TextureFormat, TextureUsage};

pub(super) const fn final_output_resource_schema() -> RenderResourceSchema {
    RenderResourceSchema::texture(
        RenderTextureSchema::new(
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED | TextureUsage::COPY_SRC,
        )
        .with_extent(RenderTextureExtentPolicy::View),
    )
}
