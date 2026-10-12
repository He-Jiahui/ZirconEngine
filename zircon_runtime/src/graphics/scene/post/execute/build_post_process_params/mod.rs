#[path = "../../../../post/build_post_process_params/baked_lighting.rs"]
mod baked_lighting;
#[path = "../../build_post_process_params/build.rs"]
mod build;
#[path = "../../../../post/build_post_process_params/color_grading.rs"]
mod color_grading;

pub(in crate::graphics::scene::scene_renderer::post_process::resources) use build::{
    build_post_process_params, build_post_process_params_with_hybrid_gi_policy,
};
