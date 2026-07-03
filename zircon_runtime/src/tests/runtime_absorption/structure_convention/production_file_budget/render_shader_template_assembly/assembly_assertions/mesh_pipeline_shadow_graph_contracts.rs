use super::super::{assert_contains_all, sources::RenderShaderTemplateAssemblySources};

pub(super) fn assert_mesh_pipeline_shadow_graph_contracts(
    sources: &RenderShaderTemplateAssemblySources,
) {
    let RenderShaderTemplateAssemblySources {
        mesh_pipeline_mod,
        mesh_pipeline_test_support,
        mesh_pipeline_velocity,
        mesh_pipeline_taa,
        mesh_pipeline_shadow,
        shadow_processor,
        non_material_rebuild,
        shadow_renderer,
        shadow_mod,
        graph_gpu_context,
        graph_stage_execution,
        ..
    } = sources;
    let mesh_pipeline_shadow_production = mesh_pipeline_shadow
        .split("#[cfg(test)]")
        .next()
        .expect("shadow mesh pipeline production section");

    assert_contains_all(
        "velocity mesh pipeline consumes template entry names",
        &mesh_pipeline_velocity,
        &[
            "entry_point: Some(\"vs_main\")",
            "entry_point: Some(\"fs_main\")",
            "GpuMeshVertex::previous_position_layout()",
            "velocity_mesh_pipeline_declares_template_entries_and_previous_position_vertex_slot",
            "velocity_mesh_pipeline_creates_on_wgpu_device_with_template_shader",
            "push_error_scope(wgpu::ErrorFilter::Validation)",
            "create_standard_mesh_pipeline_layout",
        ],
    );
    assert_contains_all(
        "mesh pipeline WGPU test support owns shared standard layout fixture",
        &mesh_pipeline_test_support,
        &[
            "pub(crate) fn create_standard_mesh_pipeline_layout",
            "create_test_scene_layout",
            "create_empty_shadow_receiver_layout",
            "create_test_material_layout",
            "GPU_MATERIAL_UNIFORM_MIN_SIZE",
            "GpuScene::new",
        ],
    );
    assert_contains_all(
        "mesh pipeline root mounts shared WGPU test support only for tests",
        &mesh_pipeline_mod,
        &["#[cfg(test)]", "mod test_support;"],
    );
    assert_contains_all(
        "shadow mesh pipeline consumes template entry names",
        &mesh_pipeline_shadow,
        &[
            "entry_point: Some(\"vs_main\")",
            "entry_point: Some(\"fs_main\")",
            "targets: &[]",
            "GpuMeshVertex::layout()",
            "SHADOW_DEPTH_BIAS_CONSTANT",
            "shadow_mesh_pipeline_declares_template_entries_static_layout_and_depth_bias",
            "shadow_mesh_pipeline_creates_on_wgpu_device_with_template_shader",
            "push_error_scope(wgpu::ErrorFilter::Validation)",
            "GpuScene::new",
        ],
    );
    assert!(
        !mesh_pipeline_shadow_production.contains("GpuMeshVertex::previous_position_layout()"),
        "shadow mesh pipeline should keep the static mesh vertex ABI and not consume the Velocity-only previous-position slot"
    );
    assert_contains_all(
        "shadow replay resolves cache-backed variants at atlas execution time",
        &shadow_renderer,
        &[
            "ensure_shadow_pipeline_for_variant",
            "command.pipeline_variant_id",
            "bind_standard_material_if_needed",
            "record_depth_only_pass",
            "MeshPipelineCache",
            "record_atlas_commands_with_attachment_ops",
        ],
    );
    let forbidden_shadow_renderer_tokens = [
        ["SHADOW", "_MAP", "_SHADER"].concat(),
        ["fixed", "_shadow", "_variant"].concat(),
        ["alpha", "_mask", "_pipeline"].concat(),
        ["fs", "_alpha", "_mask"].concat(),
    ];
    for forbidden in &forbidden_shadow_renderer_tokens {
        assert!(
            !shadow_renderer.contains(forbidden.as_str()),
            "shadow renderer should not retain the legacy inline shadow shader path token {forbidden}"
        );
    }
    let legacy_shadow_source_module = ["shadow", "_map", "_shader", "_source"].concat();
    assert!(
        !shadow_mod.contains(legacy_shadow_source_module.as_str()),
        "shadow module should not mount the deleted inline shadow-map shader source owner"
    );
    assert_contains_all(
        "shadow command producers resolve real variant ids",
        &shadow_processor,
        &["context.pipeline_variant_id(pipeline_kind, batch)"],
    );
    assert!(
        !shadow_processor.contains("MeshPipelineVariantId::new(0)"),
        "shadow pass processor should not assign the fixed base variant id"
    );
    assert_contains_all(
        "pre-mesh shadow rebuild resolves real variant ids",
        &non_material_rebuild,
        &[
            "context.pipeline_variant_id(pipeline_kind, batch)",
            "rebuilds_opaque_shadow_command_without_material_handles",
        ],
    );
    assert_contains_all(
        "shadow graph execution carries mesh pipeline context",
        &graph_gpu_context,
        &[
            "record_shadow_atlas_to_resources",
            "shadow atlas graph executor for pass `{pass_name}` requires mesh pipeline context",
            "record_atlas_commands_with_attachment_ops",
            "self.device",
            "mesh_pipelines,",
        ],
    );
    assert_contains_all(
        "early shadow graph stage receives mesh pipeline context",
        &graph_stage_execution,
        &[
            "let uses_mesh_pipeline_context = is_depth_prepass || is_shadow;",
            "stage_streamer = uses_mesh_pipeline_context.then_some(streamer)",
            "stage_mesh_pipelines = if uses_mesh_pipeline_context",
            "uses_mesh_pipeline_context.then_some(mesh_draw_lists)",
            "is_shadow.then_some(&self.shadow_map_renderer)",
            "is_shadow.then_some(shadow_frame_plan)",
        ],
    );

    assert_contains_all(
        "taa reactive mask mesh pipeline consumes template entry names",
        &mesh_pipeline_taa,
        &[
            "entry_point: Some(\"vs_main\")",
            "\"fs_taa_reactive_mask\"",
            "\"fs_taa_reactive_material_mask\"",
            "GpuMeshVertex::layout()",
            "taa_reactive_mask_pipeline_declares_template_entries_and_static_vertex_layout",
            "taa_reactive_mask_mesh_pipeline_creates_on_wgpu_device_with_template_shader",
            "push_error_scope(wgpu::ErrorFilter::Validation)",
            "create_standard_mesh_pipeline_layout",
        ],
    );
}
