struct PipelineAdmissionContract<'a> {
    cache_source: &'a str,
    consumer_source: &'a str,
    admission_api: &'a str,
    ready_pipeline_api: &'a str,
}

#[test]
fn synchronous_mesh_pass_consumers_handle_typed_pipeline_admission_without_panicking() {
    let mesh_recording =
        include_str!("../../graph_execution/render_pass_execution_context/gpu/mesh_recording.rs");
    let contracts = [
        PipelineAdmissionContract {
            cache_source: include_str!("ensure_depth_prepass_pipeline.rs"),
            consumer_source: mesh_recording,
            admission_api: "ensure_depth_prepass_pipeline_admission_for_variant",
            ready_pipeline_api: "depth_prepass_pipeline_for_ready_variant",
        },
        PipelineAdmissionContract {
            cache_source: include_str!("ensure_shadow_pipeline.rs"),
            consumer_source: include_str!("../../shadow/shadow_map_renderer.rs"),
            admission_api: "ensure_shadow_pipeline_admission_for_variant",
            ready_pipeline_api: "shadow_pipeline_for_ready_variant",
        },
        PipelineAdmissionContract {
            cache_source: include_str!("ensure_velocity_pipeline.rs"),
            consumer_source: include_str!("../../temporal/velocity/execute_velocity_object.rs"),
            admission_api: "ensure_velocity_pipeline_admission_for_variant",
            ready_pipeline_api: "velocity_pipeline_for_ready_variant",
        },
        PipelineAdmissionContract {
            cache_source: include_str!("ensure_taa_reactive_mask_pipeline.rs"),
            consumer_source: mesh_recording,
            admission_api: "ensure_taa_reactive_pipeline_admission_for_variant",
            ready_pipeline_api: "taa_reactive_pipeline_for_ready_variant",
        },
        PipelineAdmissionContract {
            cache_source: include_str!("ensure_oit_pipeline.rs"),
            consumer_source: include_str!(
                "../../graph_execution/render_pass_execution_context/gpu/oit.rs"
            ),
            admission_api: "ensure_oit_pipeline_admission_for_base_variant",
            ready_pipeline_api: "oit_pipeline_for_ready_base_variant",
        },
    ];

    for contract in contracts {
        assert!(contract.cache_source.contains("PipelineAdmission<()>"));
        assert!(contract.cache_source.contains(contract.admission_api));
        assert!(contract.cache_source.contains(contract.ready_pipeline_api));
        assert!(contract.consumer_source.contains(contract.admission_api));
        assert!(
            contract
                .consumer_source
                .contains(contract.ready_pipeline_api)
        );
        assert!(
            contract
                .consumer_source
                .contains("PipelineAdmission::Ready")
        );
        assert!(
            contract
                .consumer_source
                .contains("PipelineAdmission::Deferred")
        );
        assert!(
            contract
                .consumer_source
                .contains("PipelineAdmission::Failed")
        );
        assert!(
            contract
                .consumer_source
                .contains("record_pipeline_fallback_for_command_variant")
        );
        assert!(
            contract
                .consumer_source
                .contains("invalidate_state_after_external_pipeline")
        );
    }
}

#[test]
fn legacy_option_pipeline_admission_apis_are_removed() {
    let cache_sources = [
        include_str!("ensure_depth_prepass_pipeline.rs"),
        include_str!("ensure_shadow_pipeline.rs"),
        include_str!("ensure_velocity_pipeline.rs"),
        include_str!("ensure_taa_reactive_mask_pipeline.rs"),
        include_str!("ensure_oit_pipeline.rs"),
    ]
    .join("\n");

    for removed_api in [
        "ensure_depth_prepass_pipeline_for_variant",
        "ensure_shadow_pipeline_for_variant",
        "ensure_velocity_pipeline_for_variant",
        "ensure_taa_reactive_mask_pipeline_for_variant",
        "ensure_oit_pipeline_for_base_variant",
    ] {
        assert!(!cache_sources.contains(removed_api), "{removed_api}");
    }
}
