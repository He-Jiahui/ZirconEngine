//! 场景资源的装载、候选发布、上传回滚与几何重放由各子模块负责；这里只声明模块并导出渲染侧使用的类型。
#[path = "geometry_replay.rs"]
mod geometry_replay;
#[path = "../../resource_streamer/model_geometry_resolution.rs"]
pub(in crate::graphics::scene::resources) mod model_geometry_resolution;
#[path = "../../../resource_streamer/published_material_draw_proxy.rs"]
mod published_material_draw_proxy;
#[path = "../../resource_streamer/render_scene_geometry.rs"]
mod render_scene_geometry;
#[path = "../../resource_streamer/resource_streamer.rs"]
mod resource_streamer;
#[path = "accessors.rs"]
mod resource_streamer_accessors;
#[path = "../../resource_streamer/advanced_lighting.rs"]
mod resource_streamer_advanced_lighting;
#[path = "construct.rs"]
mod resource_streamer_construction;
#[path = "ensure_material.rs"]
mod resource_streamer_ensure_material;
#[path = "ensure_mesh.rs"]
mod resource_streamer_ensure_mesh;
#[path = "ensure_model.rs"]
mod resource_streamer_ensure_model;
#[path = "../../../resource_streamer/ensure_output_target_texture.rs"]
mod resource_streamer_ensure_output_target_texture;
#[path = "../../../resource_streamer/ensure_post_process_lut_texture.rs"]
mod resource_streamer_ensure_post_process_lut_texture;
#[path = "../../resource_streamer/ensure_scene_resources.rs"]
mod resource_streamer_ensure_scene_resources;
#[path = "../../resource_streamer/ensure_shader_source.rs"]
mod resource_streamer_ensure_shader_source;
#[path = "ensure_texture.rs"]
mod resource_streamer_ensure_texture;
#[path = "../../../resource_streamer/execute_output_target_writeback.rs"]
mod resource_streamer_execute_output_target_writeback;
#[path = "../../../resource_streamer/load_animation_skeleton_asset.rs"]
mod resource_streamer_load_animation_skeleton_asset;
#[path = "../../resource_streamer/load_model_asset.rs"]
mod resource_streamer_load_model_asset;
#[path = "mip_streaming.rs"]
mod resource_streamer_mip_streaming;
#[path = "residency.rs"]
mod resource_streamer_residency;
#[path = "../../resource_streamer/resolve_texture_id.rs"]
mod resource_streamer_resolve_texture_id;
#[path = "revision.rs"]
mod resource_streamer_resource_revision;
#[path = "shading_models.rs"]
mod resource_streamer_shading_models;
#[path = "../../resource_streamer/submission_failure.rs"]
mod resource_streamer_submission_failure;
#[path = "../../../resource_streamer/validate_material_shader_layout.rs"]
mod resource_streamer_validate_material_shader_layout;

pub(crate) use published_material_draw_proxy::{
    MaterialDrawGenerationSelection, PublishedMaterialDrawProxy,
};
pub(in crate::graphics::scene) use published_material_draw_proxy::{
    PublishedMaterialTextureBinding, PublishedMaterialTextureSet,
};
pub(crate) use resource_streamer::ResourceStreamer;
pub(crate) use resource_streamer_advanced_lighting::IrradianceVolumeTextureBinding;
pub(in crate::graphics::scene::resources) use resource_streamer_ensure_texture::TextureSnapshotFramePrepareError;
pub(crate) use resource_streamer_residency::{
    RenderAssetSemanticRoutingBudget, RenderAssetSemanticRoutingFailure,
    RenderAssetSemanticRoutingReport, RenderSceneFrameAdmissionError, RenderSceneWorldReleaseError,
};
