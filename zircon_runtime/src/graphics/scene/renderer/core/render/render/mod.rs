#[path = "../../../../../core/render/assign_execution_owned_indirect_args.rs"]
mod assign_execution_owned_indirect_args;
#[path = "../../../../../core/render/bind_compiled_scene_graph_resources.rs"]
mod bind_compiled_scene_graph_resources;
#[path = "../../../../../core/render/bind_environment_ibl_graph_resources.rs"]
mod bind_environment_ibl_graph_resources;
#[path = "../../../../../core/render/bind_execution_owned_graph_resources.rs"]
mod bind_execution_owned_graph_resources;
#[path = "../../../../core/render/bind_frame_graph_resources.rs"]
mod bind_frame_graph_resources;
#[path = "../../../../core/render/bind_history_graph_resources.rs"]
mod bind_history_graph_resources;
#[path = "../../../../core/render/bind_plugin_graph_resources.rs"]
mod bind_plugin_graph_resources;
#[path = "../../../../../core/render/bind_ssao_compute_graph_resources.rs"]
mod bind_ssao_compute_graph_resources;
#[path = "../../../../../core/render/bind_taa_reactive_mask_graph_resource.rs"]
mod bind_taa_reactive_mask_graph_resource;
#[path = "../../../../core/render/build_compiled_scene_draws.rs"]
mod build_compiled_scene_draws;
#[path = "../../../../../core/render/commit_compiled_scene_frame_success.rs"]
mod commit_compiled_scene_frame_success;
#[path = "../../../../core/render/compiled_scene_frame_foundation.rs"]
mod compiled_scene_frame_foundation;
#[path = "../../../../../core/render/execute_compiled_scene_graph_stages.rs"]
mod execute_compiled_scene_graph_stages;
#[cfg(test)]
#[path = "../../../../../core/tests/execute_compiled_scene_graph_stages.rs"]
mod execute_compiled_scene_graph_stages_tests;
#[path = "../../../../core/render/render/execute_graph_stage.rs"]
pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) mod execute_graph_stage;
#[path = "../../../../core/render/render/final_target_output.rs"]
mod final_target_output;
#[path = "frame_lifecycle.rs"]
mod frame_lifecycle;
#[path = "../../../../core/render/irradiance_volume_selection.rs"]
mod irradiance_volume_selection;
#[path = "../../../../core/render/render/pipeline_resource_usage.rs"]
mod pipeline_resource_usage;
#[path = "../../../../../core/render/prepare_compiled_scene_graph_frame.rs"]
mod prepare_compiled_scene_graph_frame;
#[path = "../../../../../render/prepare_compiled_scene_mesh_submission.rs"]
mod prepare_compiled_scene_mesh_submission;
#[path = "../../../../core/render/render/prepare_overlay_buffers.rs"]
mod prepare_overlay_buffers;
#[path = "render.rs"]
mod render;
#[path = "../../../../core/render/graph_pass_frame_services.rs"]
mod render_graph_pass_frame_services;
#[path = "../../../../core/render/render/sprite_stage_selection.rs"]
mod sprite_stage_selection;
#[path = "../../../../core/render/render/terminal_frame_packet.rs"]
mod terminal_frame_packet;

pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) use render_graph_pass_frame_services::RenderGraphPassFrameServices;
#[path = "../../../../core/render/submit_compiled_scene_frame.rs"]
mod submit_compiled_scene_frame;
