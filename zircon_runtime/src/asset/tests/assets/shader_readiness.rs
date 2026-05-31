use crate::asset::{
    AssetReference, AssetUri, ShaderAsset, ShaderAssetManagementRecord,
    ShaderAssetManagementRecordSet, ShaderAssetReadinessSummary, ShaderDependencyAsset,
    ShaderEntryPointAsset, ShaderImportRedirectAsset, ShaderRuntimeSourceKind,
    ShaderSourceLanguage,
};
use crate::core::framework::render::{
    RenderShaderBindGroupLayoutDescriptor, RenderShaderBindingDescriptor,
    RenderShaderBindingResourceType, RenderShaderDefinitionValue,
    RenderShaderPipelineLayoutDescriptor, RenderShaderStage,
};
use crate::core::resource::{ResourceId, ResourceKind};

#[test]
fn shader_readiness_reports_runtime_source_kinds() {
    let mut emitted = base_shader("res://shaders/emitted.shader");
    emitted.source_language = ShaderSourceLanguage::Glsl;
    emitted.source = "void main() {}".to_string();
    emitted.wgsl_source =
        "@fragment fn fs_main() -> @location(0) vec4f { return vec4f(); }".to_string();

    let fallback = base_shader("res://shaders/raw.wgsl");

    let mut unavailable = base_shader("res://shaders/raw.glsl");
    unavailable.source_language = ShaderSourceLanguage::Glsl;
    unavailable.source = "void main() {}".to_string();
    unavailable.wgsl_source.clear();

    let emitted_report = emitted.readiness_report();
    let fallback_report = fallback.readiness_report();
    let unavailable_report = unavailable.readiness_report();

    assert_eq!(
        emitted_report.runtime_source.source_kind,
        ShaderRuntimeSourceKind::EmittedWgsl
    );
    assert_eq!(
        fallback_report.runtime_source.source_kind,
        ShaderRuntimeSourceKind::RawWgslSource
    );
    assert_eq!(
        unavailable_report.runtime_source.source_kind,
        ShaderRuntimeSourceKind::Unavailable
    );
    assert!(emitted_report.uses_runtime_wgsl());
    assert!(fallback_report.uses_runtime_wgsl());
    assert!(!unavailable_report.uses_runtime_wgsl());
    assert!(emitted_report.is_ready());
    assert!(fallback_report.is_ready());
    assert!(!unavailable_report.is_ready());
    assert!(unavailable_report
        .runtime_source
        .diagnostic
        .as_deref()
        .unwrap()
        .contains("does not provide emitted WGSL"));
}

#[test]
fn shader_readiness_reports_import_rows_without_blocking_source_only_imports() {
    let mut shader = base_shader("res://shaders/imports.shader");
    let redirect = asset_reference("res://shaders/shared_lighting");
    shader.imports = vec![
        ShaderImportRedirectAsset {
            source: "zircon::lighting".to_string(),
            redirect: Some(redirect.clone()),
        },
        ShaderImportRedirectAsset {
            source: "naga_oil::math".to_string(),
            redirect: None,
        },
    ];
    shader.dependencies = vec![ShaderDependencyAsset {
        kind: ResourceKind::Shader,
        reference: redirect.clone(),
    }];

    let report = shader.readiness_report();

    assert!(report.is_ready());
    assert!(report.has_redirected_import_dependencies());
    assert_eq!(report.dependency_count, 1);
    assert_eq!(report.imports.len(), 2);
    assert_eq!(report.imports[0].source, "zircon::lighting");
    assert_eq!(report.imports[0].redirect, Some(redirect));
    assert!(report.imports[0].contributes_dependency);
    assert_eq!(report.imports[1].source, "naga_oil::math");
    assert!(report.imports[1].redirect.is_none());
    assert!(!report.imports[1].contributes_dependency);
}

#[test]
fn shader_readiness_reports_entry_stage_diagnostics() {
    let mut shader = base_shader("res://shaders/entries.shader");
    shader.entry_points = vec![
        ShaderEntryPointAsset {
            name: "vs_main".to_string(),
            stage: "vs".to_string(),
        },
        ShaderEntryPointAsset {
            name: "fs_main".to_string(),
            stage: "pixel".to_string(),
        },
    ];

    let report = shader.readiness_report();

    assert!(!report.is_ready());
    assert_eq!(report.entry_points.len(), 2);
    assert_eq!(
        report.entry_points[0].canonical_stage,
        Some(RenderShaderStage::Vertex)
    );
    assert!(report.entry_points[0].diagnostic.is_none());
    assert!(report.entry_points[1].canonical_stage.is_none());
    assert!(report.entry_points[1]
        .diagnostic
        .as_deref()
        .unwrap()
        .contains("unsupported stage `pixel`"));
}

#[test]
fn shader_readiness_reports_shader_def_diagnostics() {
    let mut shader = base_shader("res://shaders/defs.shader");
    shader.shader_defs = vec![
        RenderShaderDefinitionValue::from("USE_UNLIT"),
        RenderShaderDefinitionValue::from("  "),
        RenderShaderDefinitionValue::uint("ALPHA_CLIP", 1),
        RenderShaderDefinitionValue::bool(" USE_UNLIT ", false),
    ];

    let report = shader.readiness_report();

    assert!(!report.is_ready());
    assert_eq!(report.shader_defs[0].normalized_name, "USE_UNLIT");
    assert_eq!(report.shader_defs[0].value.value_as_string(), "true");
    assert!(report.shader_defs[0].diagnostic.is_none());
    assert_eq!(report.shader_defs[1].normalized_name, "");
    assert!(report.shader_defs[1]
        .diagnostic
        .as_deref()
        .unwrap()
        .contains("empty after trimming"));
    assert_eq!(report.shader_defs[2].normalized_name, "ALPHA_CLIP");
    assert_eq!(report.shader_defs[2].value.value_as_string(), "1");
    assert!(report.shader_defs[2].diagnostic.is_none());
    assert_eq!(report.shader_defs[3].normalized_name, "USE_UNLIT");
    assert_eq!(report.shader_defs[3].value.value_as_string(), "false");
    assert!(report.shader_defs[3]
        .diagnostic
        .as_deref()
        .unwrap()
        .contains("duplicated"));
}

#[test]
fn shader_readiness_copies_validation_diagnostics_and_pipeline_context() {
    let mut shader = base_shader("res://shaders/diagnostics.shader");
    shader.validation_diagnostics =
        vec!["wgsl_capture property `base_color` was not found".to_string()];
    shader.pipeline_layout = RenderShaderPipelineLayoutDescriptor {
        bind_groups: vec![RenderShaderBindGroupLayoutDescriptor {
            group: 3,
            label: Some("material".to_string()),
            bindings: vec![
                RenderShaderBindingDescriptor {
                    binding: 0,
                    label: Some("material_uniforms".to_string()),
                    resource_type: RenderShaderBindingResourceType::UniformBuffer,
                    visibility: vec![RenderShaderStage::Vertex, RenderShaderStage::Fragment],
                },
                RenderShaderBindingDescriptor {
                    binding: 1,
                    label: Some("material_sampler".to_string()),
                    resource_type: RenderShaderBindingResourceType::Sampler,
                    visibility: vec![RenderShaderStage::Fragment],
                },
            ],
        }],
        push_constant_ranges: vec!["draw_index:0..4".to_string()],
    };

    let report = shader.readiness_report();

    assert!(!report.is_ready());
    assert_eq!(
        report.validation_diagnostics,
        vec!["wgsl_capture property `base_color` was not found".to_string()]
    );
    assert!(report.has_pipeline_layout);
    assert!(report.pipeline_layout.has_layout);
    assert_eq!(report.pipeline_layout.bind_group_count, 1);
    assert_eq!(report.pipeline_layout.binding_count, 2);
    assert_eq!(report.pipeline_layout.push_constant_range_count, 1);
    assert_eq!(
        report.pipeline_layout.push_constant_ranges,
        vec!["draw_index:0..4"]
    );
    assert_eq!(report.pipeline_layout.bind_groups[0].group, 3);
    assert_eq!(
        report.pipeline_layout.bind_groups[0].label.as_deref(),
        Some("material")
    );
    assert_eq!(report.pipeline_layout.bind_groups[0].binding_count, 2);
    assert_eq!(
        report.pipeline_layout.bind_groups[0].bindings[0].resource_type,
        RenderShaderBindingResourceType::UniformBuffer
    );
    assert_eq!(
        report.pipeline_layout.bind_groups[0].bindings[0].visibility,
        vec![RenderShaderStage::Vertex, RenderShaderStage::Fragment]
    );
}

#[test]
fn shader_readiness_summary_counts_management_panel_fields() {
    let mut shader = base_shader("res://shaders/summary.shader");
    let redirect = asset_reference("res://shaders/shared_summary");
    shader.imports = vec![
        ShaderImportRedirectAsset {
            source: "zircon::shared_summary".to_string(),
            redirect: Some(redirect.clone()),
        },
        ShaderImportRedirectAsset {
            source: "source_only".to_string(),
            redirect: None,
        },
    ];
    shader.dependencies = vec![ShaderDependencyAsset {
        kind: ResourceKind::Shader,
        reference: redirect,
    }];
    shader.entry_points = vec![
        ShaderEntryPointAsset {
            name: "vs_main".to_string(),
            stage: "vertex".to_string(),
        },
        ShaderEntryPointAsset {
            name: "bad_main".to_string(),
            stage: "pixel".to_string(),
        },
    ];
    shader.shader_defs = vec![
        RenderShaderDefinitionValue::bool("USE_LIGHTING", true),
        RenderShaderDefinitionValue::from("  "),
    ];
    shader.validation_diagnostics = vec!["wgsl capture missing `roughness`".to_string()];
    shader.pipeline_layout = RenderShaderPipelineLayoutDescriptor {
        bind_groups: vec![RenderShaderBindGroupLayoutDescriptor {
            group: 3,
            label: Some("material".to_string()),
            bindings: vec![RenderShaderBindingDescriptor {
                binding: 0,
                label: Some("material_uniforms".to_string()),
                resource_type: RenderShaderBindingResourceType::UniformBuffer,
                visibility: vec![RenderShaderStage::Fragment],
            }],
        }],
        push_constant_ranges: vec!["draw_index:0..4".to_string()],
    };

    let summary: ShaderAssetReadinessSummary = shader.readiness_summary();

    assert!(!summary.ready);
    assert!(summary.uses_runtime_wgsl);
    assert_eq!(
        summary.runtime_source_kind,
        ShaderRuntimeSourceKind::RawWgslSource
    );
    assert_eq!(summary.import_count, 2);
    assert_eq!(summary.redirected_import_count, 1);
    assert_eq!(summary.entry_point_count, 2);
    assert_eq!(summary.entry_point_diagnostic_count, 1);
    assert_eq!(summary.shader_definition_count, 2);
    assert_eq!(summary.shader_definition_diagnostic_count, 1);
    assert_eq!(summary.validation_diagnostic_count, 1);
    assert_eq!(summary.dependency_count, 1);
    assert!(summary.has_pipeline_layout);
    assert_eq!(summary.bind_group_count, 1);
    assert_eq!(summary.binding_count, 1);
    assert_eq!(summary.push_constant_range_count, 1);
}

#[test]
fn shader_asset_management_record_wraps_id_summary_and_report() {
    let shader = base_shader("res://shaders/management-record.shader");
    let shader_id = ResourceId::from_locator(&shader.uri);
    let report = shader.readiness_report();
    let summary = report.summary();

    let record: ShaderAssetManagementRecord = shader.management_record(shader_id);

    assert_eq!(record.shader_id, shader_id);
    assert_eq!(record.summary, summary);
    assert_eq!(record.report, report);
}

#[test]
fn shader_asset_management_record_set_sorts_and_summarizes_records() {
    let ready_shader = base_shader("res://shaders/ready-record.shader");
    let mut invalid_shader = base_shader("res://shaders/invalid-record.shader");
    let redirect = asset_reference("res://shaders/shared_invalid");
    invalid_shader.source_language = ShaderSourceLanguage::Glsl;
    invalid_shader.source = "void main() {}".to_string();
    invalid_shader.wgsl_source.clear();
    invalid_shader.imports = vec![ShaderImportRedirectAsset {
        source: "zircon::shared_invalid".to_string(),
        redirect: Some(redirect.clone()),
    }];
    invalid_shader.dependencies = vec![ShaderDependencyAsset {
        kind: ResourceKind::Shader,
        reference: redirect,
    }];
    invalid_shader.entry_points = vec![ShaderEntryPointAsset {
        name: "bad_main".to_string(),
        stage: "pixel".to_string(),
    }];
    invalid_shader.shader_defs = vec![RenderShaderDefinitionValue::from("  ")];
    invalid_shader.validation_diagnostics = vec!["wgsl capture missing `base_color`".to_string()];
    invalid_shader.pipeline_layout = RenderShaderPipelineLayoutDescriptor {
        bind_groups: vec![RenderShaderBindGroupLayoutDescriptor {
            group: 3,
            label: Some("material".to_string()),
            bindings: vec![
                RenderShaderBindingDescriptor {
                    binding: 0,
                    label: Some("material_uniforms".to_string()),
                    resource_type: RenderShaderBindingResourceType::UniformBuffer,
                    visibility: vec![RenderShaderStage::Fragment],
                },
                RenderShaderBindingDescriptor {
                    binding: 1,
                    label: Some("material_sampler".to_string()),
                    resource_type: RenderShaderBindingResourceType::Sampler,
                    visibility: vec![RenderShaderStage::Fragment],
                },
            ],
        }],
        push_constant_ranges: vec!["draw_index:0..4".to_string()],
    };
    let ready_id = ResourceId::from_locator(&ready_shader.uri);
    let invalid_id = ResourceId::from_locator(&invalid_shader.uri);

    let record_set = ShaderAssetManagementRecordSet::from_records(vec![
        invalid_shader.management_record(invalid_id),
        ready_shader.management_record(ready_id),
    ]);

    let mut expected_ids = vec![invalid_id, ready_id];
    expected_ids.sort();
    let record_ids = record_set
        .records
        .iter()
        .map(|record| record.shader_id)
        .collect::<Vec<_>>();
    assert_eq!(record_ids, expected_ids);
    assert_eq!(record_set.records.len(), 2);
    let summary = &record_set.summary;
    assert_eq!(summary.shader_count, 2);
    assert_eq!(summary.ready_count, 1);
    assert_eq!(summary.not_ready_count, 1);
    assert_eq!(summary.runtime_wgsl_count, 1);
    assert_eq!(summary.unavailable_runtime_source_count, 1);
    assert_eq!(summary.redirected_import_count, 1);
    assert_eq!(summary.dependency_count, 1);
    assert_eq!(summary.entry_point_diagnostic_count, 1);
    assert_eq!(summary.shader_definition_diagnostic_count, 1);
    assert_eq!(summary.validation_diagnostic_count, 1);
    assert_eq!(summary.pipeline_layout_count, 1);
    assert_eq!(summary.bind_group_count, 1);
    assert_eq!(summary.binding_count, 2);
    assert_eq!(summary.push_constant_range_count, 1);
}

fn base_shader(uri: &str) -> ShaderAsset {
    ShaderAsset {
        uri: locator(uri),
        source_language: ShaderSourceLanguage::Wgsl,
        source: "@fragment fn fs_main() -> @location(0) vec4f { return vec4f(); }".to_string(),
        wgsl_source: String::new(),
        import_path: None,
        entry_points: Vec::new(),
        dependencies: Vec::new(),
        source_files: Vec::new(),
        imports: Vec::new(),
        shader_defs: Vec::new(),
        property_schema: Vec::new(),
        texture_slots: Vec::new(),
        editor: Default::default(),
        pipeline_layout: Default::default(),
        validation_diagnostics: Vec::new(),
    }
}

fn locator(uri: &str) -> AssetUri {
    AssetUri::parse(uri).unwrap()
}

fn asset_reference(uri: &str) -> AssetReference {
    AssetReference::from_locator(locator(uri))
}
