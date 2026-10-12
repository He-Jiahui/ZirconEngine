use super::*;

#[test]
fn render_sss_setup_workload_matches_tile_classifier_owner_constants() {
    let workload = setup_compute_workload();
    assert_eq!(workload.pipeline_label, SSS_SETUP_PIPELINE_LABEL);
    assert_eq!(workload.workgroup_size, SSS_TILE_SIZE);
}

#[test]
fn render_sss_scatter_workload_is_gpu_indirect() {
    let workload = scatter_compute_workload();
    assert_eq!(workload.pipeline_label, SSS_SCATTER_PIPELINE_LABEL);
    assert_eq!(workload.workgroup_size, SSS_TILE_SIZE);
    assert_eq!(
        workload.dispatch_extent,
        crate::render_graph::RenderGraphComputeDispatchExtent::IndirectArgs
    );
}

#[test]
fn render_sss_shaders_keep_tile_indirect_and_recombine_contracts() {
    assert!(pipelines::SETUP_SHADER.contains("atomicAdd(&indirect_args.group_count_x, 1u)"));
    assert!(pipelines::SETUP_SHADER.contains("indirect_args.group_count_y = 1u"));
    assert!(pipelines::SETUP_SHADER.contains("indirect_args.group_count_z = 1u"));
    assert!(pipelines::SCATTER_SHADER.contains("BURLEY_SAMPLE_COUNT: u32 = 64u"));
    assert!(pipelines::SCATTER_SHADER.contains("tile_list[workgroup_id.x]"));
    assert!(pipelines::SCATTER_SHADER.contains("mix(center_diffuse, scattered, falloff)"));
    assert!(pipelines::RECOMBINE_SHADER.contains("scattered_sample.rgb + specular_sample.rgb"));
}

#[test]
fn render_sss_descriptor_extends_deferred_lighting_with_diffuse_and_retained_mrts() {
    let descriptor = render_feature_descriptor();
    let extensions = descriptor.resource_extensions().collect::<Vec<_>>();

    assert_eq!(extensions.len(), 2);
    assert!(extensions.iter().all(|extension| {
        extension.target_pass_name == "deferred-lighting"
            && extension.resource.access == crate::graphics::RenderFeatureResourceAccess::Write
    }));
}

#[test]
fn render_sss_descriptor_versions_prepared_uniforms_from_setup_to_scatter() {
    use crate::graphics::{RenderFeatureResourceAccess, RenderFeatureResourceKind};

    let descriptor = render_feature_descriptor();
    let setup = descriptor
        .stage_passes
        .iter()
        .find(|pass| pass.pass_name == SSS_SETUP_EXECUTOR_ID)
        .expect("SSS setup pass");
    let scatter = descriptor
        .stage_passes
        .iter()
        .find(|pass| pass.pass_name == SSS_SCATTER_EXECUTOR_ID)
        .expect("SSS scatter pass");

    for name in [
        PostProcessGraphResourceNames::SSS_PARAMS,
        PostProcessGraphResourceNames::SSS_PROFILES,
    ] {
        assert!(setup.resources.iter().any(|resource| {
            resource.name == name
                && resource.kind == RenderFeatureResourceKind::Buffer
                && resource.access == RenderFeatureResourceAccess::Write
        }));
        assert!(scatter.resources.iter().any(|resource| {
            resource.name == name
                && resource.kind == RenderFeatureResourceKind::Buffer
                && resource.access == RenderFeatureResourceAccess::Read
        }));
    }
    assert_eq!(SSS_PARAMS_BUFFER_SIZE_BYTES, 80);
    assert_eq!(SSS_PROFILE_TABLE_BUFFER_SIZE_BYTES, 512);
    assert_eq!(
        prepared_frame::PreparedSubsurfaceFrame::uploaded_byte_len(),
        592
    );
}

#[test]
fn render_sss_prepares_once_and_consumes_graph_owned_uniforms() {
    let prepared = include_str!("../prepared_frame.rs");
    let executors = include_str!("../executors.rs");
    let pipelines = include_str!("../pipelines.rs");

    assert_eq!(
        prepared
            .matches("resolve_subsurface_profile_table(")
            .count(),
        1
    );
    assert_eq!(prepared.matches(".inverse();").count(), 1);
    assert!(executors.contains("PreparedSubsurfaceFrame::prepare("));
    assert!(executors.contains("append_pre_submit_buffer_uploads("));
    assert!(!executors.contains(".subsurface_profiles.clone()"));
    assert!(pipelines.contains("context.command_encoder().clear_buffer("));
    assert!(!pipelines.contains("queue.write_buffer("));
    assert!(!pipelines.contains("create_buffer_init("));
}
