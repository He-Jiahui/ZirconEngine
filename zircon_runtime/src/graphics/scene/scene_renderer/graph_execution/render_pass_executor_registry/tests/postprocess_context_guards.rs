use super::*;

#[test]
fn taa_reactive_mask_clear_executor_requires_graph_resources_instead_of_nooping() {
    let mut extract = test_extract();
    extract.view.anti_alias = AntiAliasSettings::taa();
    extract
        .post_process
        .rebuild_graph_with_anti_alias(true, true, &AntiAliasSettings::taa());

    let error = execute_gpu_executor_without_specialized_context_for_extract(
        "taa-reactive-mask-clear",
        "temporal.taa-reactive-mask-clear",
        extract,
    )
    .unwrap_err();

    assert_eq!(
        error,
        "TAA reactive mask clear graph executor for pass `taa-reactive-mask-clear` requires post-process stack context"
    );
}

#[test]
fn taa_reactive_mask_mesh_executor_requires_graph_resources_instead_of_nooping() {
    let mut extract = test_extract();
    extract.view.anti_alias = AntiAliasSettings::taa();
    extract
        .post_process
        .rebuild_graph_with_anti_alias(true, true, &AntiAliasSettings::taa());

    let error = execute_gpu_executor_without_specialized_context_for_extract(
        "taa-reactive-mask-mesh",
        "temporal.taa-reactive-mask-mesh",
        extract,
    )
    .unwrap_err();

    assert_eq!(
        error,
        "TAA reactive mask mesh graph executor for pass `taa-reactive-mask-mesh` requires mesh draw context"
    );
}

#[test]
fn taa_resolve_executor_requires_graph_resources_instead_of_nooping() {
    let mut extract = test_extract();
    extract.view.anti_alias = AntiAliasSettings::taa();
    extract
        .post_process
        .rebuild_graph_with_anti_alias(true, true, &AntiAliasSettings::taa());

    let error = execute_gpu_executor_without_specialized_context_for_extract(
        "taa-resolve",
        "temporal.taa-resolve",
        extract,
    )
    .unwrap_err();

    assert_eq!(
        error,
        "TAA resolve graph executor for pass `taa-resolve` requires post-process stack context"
    );
}

#[test]
fn uber_executor_requires_post_process_context_instead_of_nooping() {
    let error = execute_gpu_executor_without_specialized_context("uber", "post.uber");

    assert_eq!(
        error,
        "post-process stack graph executor for pass `uber` requires post-process stack context"
    );
}

#[test]
fn ssao_executor_requires_post_process_context_instead_of_nooping() {
    let error =
        execute_gpu_executor_without_specialized_context("ssao-evaluate", "ao.ssao-evaluate");

    assert_eq!(
        error,
        "SSAO graph executor for pass `ssao-evaluate` requires post-process stack context"
    );
}

#[test]
fn clustered_lighting_executor_requires_post_process_context_instead_of_nooping() {
    let error =
        execute_gpu_executor_without_specialized_context("light-grid-build", "lighting.light-grid");

    assert_eq!(
        error,
        "light grid graph executor for pass `light-grid-build` requires post-process stack context"
    );
}

#[test]
fn bloom_extract_executor_requires_post_process_context_instead_of_nooping() {
    let error =
        execute_gpu_executor_without_specialized_context("bloom-extract", "post.bloom-extract");

    assert_eq!(
        error,
        "bloom graph executor for pass `bloom-extract` requires post-process stack context"
    );
}

#[test]
fn velocity_camera_executor_requires_post_process_context_instead_of_nooping() {
    let error = execute_gpu_executor_without_specialized_context_with_effect_stack(
        "velocity-camera",
        "temporal.velocity-camera",
        effect_stack_with_motion_vectors(),
    );

    assert_eq!(
        error,
        "velocity camera graph executor for pass `velocity-camera` requires post-process stack context"
    );
}

#[test]
fn velocity_object_executor_requires_graph_target_instead_of_nooping() {
    let error = execute_gpu_executor_without_specialized_context_with_effect_stack(
        "velocity-object",
        "temporal.velocity-object",
        effect_stack_with_motion_vectors(),
    );

    assert_eq!(
        error,
        "render graph execution texture resource `scene-velocity` is not bound"
    );
}

#[test]
fn motion_vector_tile_max_executor_requires_post_process_context_instead_of_nooping() {
    let error = execute_gpu_executor_without_specialized_context_with_effect_stack(
        "motion-vector-tile-max",
        "post.motion-vector-tile-max",
        effect_stack_with_motion_vectors(),
    );

    assert_eq!(
        error,
        "motion-vector tile-max graph executor for pass `motion-vector-tile-max` requires post-process stack context"
    );
}

#[test]
fn motion_vector_tile_max_coarse_executor_requires_post_process_context_instead_of_nooping() {
    let error = execute_gpu_executor_without_specialized_context_with_effect_stack(
        "motion-vector-tile-max-coarse",
        "post.motion-vector-tile-max-coarse",
        effect_stack_with_motion_vectors(),
    );

    assert_eq!(
        error,
        "motion-vector tile-max graph executor for pass `motion-vector-tile-max-coarse` requires post-process stack context"
    );
}

#[test]
fn motion_vector_neighbor_max_executor_requires_post_process_context_instead_of_nooping() {
    let error = execute_gpu_executor_without_specialized_context_with_effect_stack(
        "motion-vector-neighbor-max",
        "post.motion-vector-neighbor-max",
        effect_stack_with_motion_vectors(),
    );

    assert_eq!(
        error,
        "motion-vector neighbor-max graph executor for pass `motion-vector-neighbor-max` requires post-process stack context"
    );
}

#[test]
fn depth_of_field_prepare_executor_requires_post_process_context_instead_of_nooping() {
    let error = execute_gpu_executor_without_specialized_context_with_effect_stack(
        "depth-of-field-prepare",
        "post.depth-of-field-prepare",
        effect_stack_with_depth_of_field(),
    );

    assert_eq!(
        error,
        "depth-of-field prepare graph executor for pass `depth-of-field-prepare` requires post-process stack context"
    );
}

#[test]
fn screen_space_reflection_resolve_executor_requires_post_process_context_instead_of_nooping() {
    let error = execute_gpu_executor_without_specialized_context_with_effect_stack(
        "screen-space-reflection-resolve",
        "post.screen-space-reflection-resolve",
        effect_stack_with_screen_space_reflection(),
    );

    assert_eq!(
        error,
        "screen-space reflection resolve graph executor for pass `screen-space-reflection-resolve` requires post-process stack context"
    );
}

#[test]
fn screen_space_reflection_reflection_pyramid_executor_requires_post_process_context_instead_of_nooping()
 {
    let error = execute_gpu_executor_without_specialized_context_with_effect_stack(
        "screen-space-reflection-reflection-pyramid",
        "post.screen-space-reflection-reflection-pyramid",
        effect_stack_with_screen_space_reflection(),
    );

    assert_eq!(
        error,
        "screen-space reflection reflection-pyramid graph executor for pass `screen-space-reflection-reflection-pyramid` requires post-process stack context"
    );
}

#[test]
fn screen_space_reflection_reflection_pyramid_coarse_executor_requires_post_process_context_instead_of_nooping()
 {
    let error = execute_gpu_executor_without_specialized_context_with_effect_stack(
        "screen-space-reflection-reflection-pyramid-coarse",
        "post.screen-space-reflection-reflection-pyramid-coarse",
        effect_stack_with_screen_space_reflection(),
    );

    assert_eq!(
        error,
        "screen-space reflection reflection-pyramid coarse graph executor for pass `screen-space-reflection-reflection-pyramid-coarse` requires post-process stack context"
    );
}

#[test]
fn optional_postprocess_executors_skip_resource_work_when_effects_are_disabled() {
    for (pass_name, executor_id) in [
        ("velocity-object", "temporal.velocity-object"),
        ("velocity-camera", "temporal.velocity-camera"),
        (
            "taa-reactive-mask-clear",
            "temporal.taa-reactive-mask-clear",
        ),
        ("taa-reactive-mask-mesh", "temporal.taa-reactive-mask-mesh"),
        ("taa-resolve", "temporal.taa-resolve"),
        ("motion-vector-tile-max", "post.motion-vector-tile-max"),
        (
            "motion-vector-tile-max-coarse",
            "post.motion-vector-tile-max-coarse",
        ),
        (
            "motion-vector-neighbor-max",
            "post.motion-vector-neighbor-max",
        ),
        ("depth-of-field-prepare", "post.depth-of-field-prepare"),
        (
            "screen-space-reflection-reflection-pyramid",
            "post.screen-space-reflection-reflection-pyramid",
        ),
        (
            "screen-space-reflection-reflection-pyramid-coarse",
            "post.screen-space-reflection-reflection-pyramid-coarse",
        ),
        (
            "screen-space-reflection-resolve",
            "post.screen-space-reflection-resolve",
        ),
        (
            "screen-space-reflection-specular-occlusion",
            "post.screen-space-reflection-specular-occlusion",
        ),
    ] {
        execute_gpu_executor_without_specialized_context_for_extract(
            pass_name,
            executor_id,
            test_extract(),
        )
        .unwrap_or_else(|error| {
            panic!(
                "disabled optional post-process executor `{executor_id}` should skip before resource work; error={error}"
            )
        });
    }
}

#[test]
fn preview_sky_executor_requires_preview_renderer_context_instead_of_nooping() {
    let error =
        execute_gpu_executor_without_specialized_context("preview-sky", "sky.preview-scene-color");

    assert_eq!(
        error,
        "preview sky graph executor for pass `preview-sky` requires preview sky renderer context"
    );
}

fn execute_gpu_executor_without_specialized_context_with_effect_stack(
    pass_name: &str,
    executor_id: &str,
    effect_stack: RenderPostProcessEffectStackSettings,
) -> String {
    let mut extract = test_extract();
    extract.post_process.effect_stack = effect_stack;
    execute_gpu_executor_without_specialized_context_for_extract(pass_name, executor_id, extract)
        .unwrap_err()
}

fn effect_stack_with_motion_vectors() -> RenderPostProcessEffectStackSettings {
    RenderPostProcessEffectStackSettings {
        motion_blur: RenderMotionBlurSettings {
            shutter_angle: 90.0,
            samples: 8,
        },
        ..Default::default()
    }
}

fn effect_stack_with_depth_of_field() -> RenderPostProcessEffectStackSettings {
    RenderPostProcessEffectStackSettings {
        depth_of_field: RenderDepthOfFieldSettings {
            aperture: 0.75,
            max_blur_radius: 3.0,
            ..Default::default()
        },
        ..Default::default()
    }
}

fn effect_stack_with_screen_space_reflection() -> RenderPostProcessEffectStackSettings {
    RenderPostProcessEffectStackSettings {
        screen_space_reflection: RenderScreenSpaceReflectionSettings {
            intensity: 0.5,
            max_steps: 24,
            ..Default::default()
        },
        ..Default::default()
    }
}
