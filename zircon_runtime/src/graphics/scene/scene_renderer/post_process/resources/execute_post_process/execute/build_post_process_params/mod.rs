mod baked_lighting;
mod build;
mod color_grading;

pub(in crate::graphics::scene::scene_renderer::post_process::resources) use build::{
    build_post_process_params, build_post_process_params_with_hybrid_gi_policy,
};
