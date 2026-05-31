use super::*;

#[test]
fn render_product_pbr_streamer_records_missing_material_fallback_runtime() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let missing_material_id = ResourceId::from_stable_label("res://materials/not-registered");
    let mut streamer =
        ResourceStreamer::new_for_test(asset_manager, &device, &queue, &texture_layout);

    streamer
        .ensure_material(
            &device,
            &queue,
            &texture_layout,
            ResourceHandle::<MaterialMarker>::new(missing_material_id),
        )
        .expect("missing material uses fallback runtime");

    let report = streamer
        .material_readiness_report(&missing_material_id)
        .expect("fallback material readiness report");
    assert!(!report.is_ready());
    assert!(report.uses_fallback());
    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::UnresolvedMaterialReference { material }
            if *material == missing_material_id
    )));
    assert!(report.fallback_usages.iter().any(|usage| matches!(
        &usage.reason,
        RenderMaterialFallbackReason::Material { material }
            if *material == missing_material_id
    )));
}

#[test]
fn render_product_streamer_material_report_exposes_missing_shader_and_texture_fallbacks() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = locator("res://materials/streamer-missing.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            missing_refs_material(),
        )
        .expect("material insert");
    let mut streamer =
        ResourceStreamer::new_for_test(asset_manager, &device, &queue, &texture_layout);

    streamer
        .ensure_material(
            &device,
            &queue,
            &texture_layout,
            ResourceHandle::<MaterialMarker>::new(material_id),
        )
        .expect("material prepares with fallback resources");

    let report = streamer
        .material_readiness_report(&material_id)
        .expect("streamer readiness report");

    assert!(!report.is_ready());
    assert!(report.uses_fallback());
    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::UnresolvedShaderReference { reference }
            if reference.locator == locator("res://shaders/missing-streamer.wgsl")
    )));
    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::UnresolvedTextureReference { slot, reference }
            if slot == "base_color_texture"
                && reference.locator == locator("res://textures/missing-streamer.png")
    )));
    assert!(report.fallback_usages.iter().any(|usage| matches!(
        &usage.reason,
        RenderMaterialFallbackReason::Shader { reference }
            if reference.locator == locator("res://shaders/missing-streamer.wgsl")
    )));
    assert!(report.fallback_usages.iter().any(|usage| matches!(
        &usage.reason,
        RenderMaterialFallbackReason::Texture { slot, reference }
            if slot == "base_color_texture"
                && reference.locator == locator("res://textures/missing-streamer.png")
    )));
}

#[test]
fn render_product_streamer_reports_wrong_kind_shader_and_texture_refs() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = locator("res://materials/wrong-kind.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(
                ResourceId::from_locator(&locator("res://shaders/not-a-shader.wgsl")),
                ResourceKind::Texture,
                locator("res://shaders/not-a-shader.wgsl"),
            ),
            rgba_texture("res://shaders/not-a-shader.wgsl"),
        )
        .expect("wrong-kind shader texture insert");
    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(
                ResourceId::from_locator(&locator("res://textures/not-a-texture.png")),
                ResourceKind::Shader,
                locator("res://textures/not-a-texture.png"),
            ),
            wgsl_shader("res://textures/not-a-texture.png"),
        )
        .expect("wrong-kind texture shader insert");
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material_with_refs(
                "res://shaders/not-a-shader.wgsl",
                Some("res://textures/not-a-texture.png"),
            ),
        )
        .expect("material insert");
    let mut streamer =
        ResourceStreamer::new_for_test(asset_manager, &device, &queue, &texture_layout);

    streamer
        .ensure_material(
            &device,
            &queue,
            &texture_layout,
            ResourceHandle::<MaterialMarker>::new(material_id),
        )
        .expect("wrong-kind dependencies use fallbacks");

    let report = streamer
        .material_readiness_report(&material_id)
        .expect("streamer readiness report");

    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::UnresolvedShaderReference { reference }
            if reference.locator == locator("res://shaders/not-a-shader.wgsl")
    )));
    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::UnresolvedTextureReference { slot, reference }
            if slot == "base_color_texture"
                && reference.locator == locator("res://textures/not-a-texture.png")
    )));
}

#[test]
fn render_product_streamer_stores_missing_runtime_wgsl_before_returning_error() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = locator("res://materials/missing-wgsl.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(
                ResourceId::from_locator(&locator("res://shaders/source-only.glsl")),
                ResourceKind::Shader,
                locator("res://shaders/source-only.glsl"),
            ),
            glsl_without_runtime_wgsl("res://shaders/source-only.glsl"),
        )
        .expect("shader insert");
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material_with_refs("res://shaders/source-only.glsl", None),
        )
        .expect("material insert");
    let mut streamer =
        ResourceStreamer::new_for_test(asset_manager, &device, &queue, &texture_layout);

    let error = streamer
        .ensure_material(
            &device,
            &queue,
            &texture_layout,
            ResourceHandle::<MaterialMarker>::new(material_id),
        )
        .expect_err("missing runtime WGSL blocks material readiness");
    let report = streamer
        .material_readiness_report(&material_id)
        .expect("stored readiness report after blocking error");

    assert!(error.to_string().contains("not render-ready"));
    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::MissingRuntimeShaderSource
    )));
    assert!(report.fallback_usages.iter().any(|usage| matches!(
        &usage.reason,
        RenderMaterialFallbackReason::Shader { reference }
            if reference.locator == locator("res://shaders/source-only.glsl")
    )));
}

#[test]
fn render_product_streamer_repeated_blocking_material_ensure_remains_blocking() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = locator("res://materials/repeated-missing-wgsl.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(
                ResourceId::from_locator(&locator("res://shaders/repeated-source-only.glsl")),
                ResourceKind::Shader,
                locator("res://shaders/repeated-source-only.glsl"),
            ),
            glsl_without_runtime_wgsl("res://shaders/repeated-source-only.glsl"),
        )
        .expect("shader insert");
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material_with_refs("res://shaders/repeated-source-only.glsl", None),
        )
        .expect("material insert");
    let mut streamer =
        ResourceStreamer::new_for_test(asset_manager, &device, &queue, &texture_layout);

    for _ in 0..2 {
        let error = streamer
            .ensure_material(
                &device,
                &queue,
                &texture_layout,
                ResourceHandle::<MaterialMarker>::new(material_id),
            )
            .expect_err("cached blocking material must remain blocking");
        assert!(error.to_string().contains("not render-ready"));
    }
}

#[test]
fn render_product_streamer_material_report_includes_shader_readiness_diagnostics() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = locator("res://materials/shader-readiness.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    let shader_uri = locator("res://shaders/shader-readiness.zshader");
    let mut shader = wgsl_shader("res://shaders/shader-readiness.zshader");
    shader.entry_points = vec![ShaderEntryPointAsset {
        name: "fs_main".to_string(),
        stage: "pixel".to_string(),
    }];
    shader.shader_defs = vec![
        RenderShaderDefinitionValue::from("USE_UNLIT"),
        RenderShaderDefinitionValue::from(" USE_UNLIT "),
    ];
    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(
                ResourceId::from_locator(&shader_uri),
                ResourceKind::Shader,
                shader_uri,
            ),
            shader,
        )
        .expect("shader insert");
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material_with_refs("res://shaders/shader-readiness.zshader", None),
        )
        .expect("material insert");
    let mut streamer =
        ResourceStreamer::new_for_test(asset_manager, &device, &queue, &texture_layout);

    streamer
        .ensure_material(
            &device,
            &queue,
            &texture_layout,
            ResourceHandle::<MaterialMarker>::new(material_id),
        )
        .expect("shader readiness diagnostics are non-blocking material report rows");

    let report = streamer
        .material_readiness_report(&material_id)
        .expect("streamer readiness report");
    assert!(!report.is_ready());
    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::ShaderReadinessDiagnostic {
            source,
            path,
            diagnostic,
        } if *source == RenderMaterialDiagnosticSource::ShaderReadiness
            && path == "entry_points.fs_main"
            && diagnostic.contains("unsupported stage `pixel`")
    )));
    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::ShaderReadinessDiagnostic {
            source,
            path,
            diagnostic,
        } if *source == RenderMaterialDiagnosticSource::ShaderReadiness
            && path == "shader_defs.USE_UNLIT"
            && diagnostic.contains("duplicated")
    )));
}

#[test]
fn render_product_streamer_reports_shader_material_contract_diagnostics() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = locator("res://materials/material-contract.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    let shader_uri = locator("res://shaders/material-contract.zshader");
    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(
                ResourceId::from_locator(&shader_uri),
                ResourceKind::Shader,
                shader_uri.clone(),
            ),
            shader_with_required_material_contract("res://shaders/material-contract.zshader"),
        )
        .expect("shader insert");
    let mut material = material_with_refs("res://shaders/material-contract.zshader", None);
    material.property_values.insert(
        "ghost_property".to_string(),
        toml::Value::String("orphan".to_string()),
    );
    material.property_values.insert(
        "roughness_bias".to_string(),
        toml::Value::String("not-a-float".to_string()),
    );
    material.texture_slots.insert(
        "required_mask".to_string(),
        MaterialTextureSlotValue {
            reference: None,
            fallback: Some("white".to_string()),
        },
    );
    material.texture_slots.insert(
        "unused_overlay".to_string(),
        MaterialTextureSlotValue {
            reference: None,
            fallback: Some("black".to_string()),
        },
    );
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material,
        )
        .expect("material insert");
    let mut streamer =
        ResourceStreamer::new_for_test(asset_manager, &device, &queue, &texture_layout);

    streamer
        .ensure_material(
            &device,
            &queue,
            &texture_layout,
            ResourceHandle::<MaterialMarker>::new(material_id),
        )
        .expect("shader/material contract diagnostics are stored on readiness report");

    let report = streamer
        .material_readiness_report(&material_id)
        .expect("streamer readiness report");
    assert!(!report.is_ready());
    assert!(report.fallback_usages.is_empty());
    assert_eq!(report.validation_errors.len(), 5);
    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::UnknownPropertyOverride {
            source,
            path,
            name,
        } if *source == RenderMaterialDiagnosticSource::MaterialOverride
            && path == "overrides.ghost_property"
            && name == "ghost_property"
    )));
    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::PropertyOverrideTypeMismatch {
            source,
            path,
            name,
            expected,
        } if *source == RenderMaterialDiagnosticSource::ShaderSchema
            && path == "overrides.roughness_bias"
            && name == "roughness_bias"
            && expected == "float"
    )));
    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::MissingRequiredProperty {
            source,
            path,
            name,
        } if *source == RenderMaterialDiagnosticSource::ShaderSchema
            && path == "overrides.required_gain"
            && name == "required_gain"
    )));
    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::MissingRequiredTextureSlot {
            source,
            path,
            slot,
        } if *source == RenderMaterialDiagnosticSource::ShaderSchema
            && path == "textures.required_mask"
            && slot == "required_mask"
    )));
    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::UnknownTextureSlot {
            source,
            path,
            slot,
        } if *source == RenderMaterialDiagnosticSource::TextureSlot
            && path == "textures.unused_overlay"
            && slot == "unused_overlay"
    )));
}

#[test]
fn render_product_streamer_reports_shader_material_layout_abi_diagnostics() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = locator("res://materials/material-layout-abi.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    let shader_uri = locator("res://shaders/material-layout-abi.zshader");
    let mut shader = wgsl_shader("res://shaders/material-layout-abi.zshader");
    shader.pipeline_layout = RenderShaderPipelineLayoutDescriptor {
        bind_groups: vec![RenderShaderBindGroupLayoutDescriptor {
            group: 3,
            label: Some("material".to_string()),
            bindings: vec![
                RenderShaderBindingDescriptor {
                    binding: 0,
                    label: Some("material_texture".to_string()),
                    resource_type: RenderShaderBindingResourceType::Texture,
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
        push_constant_ranges: Vec::new(),
    };
    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(
                ResourceId::from_locator(&shader_uri),
                ResourceKind::Shader,
                shader_uri,
            ),
            shader,
        )
        .expect("shader insert");
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material_with_refs("res://shaders/material-layout-abi.zshader", None),
        )
        .expect("material insert");
    let mut streamer =
        ResourceStreamer::new_for_test(asset_manager, &device, &queue, &texture_layout);

    streamer
        .ensure_material(
            &device,
            &queue,
            &texture_layout,
            ResourceHandle::<MaterialMarker>::new(material_id),
        )
        .expect("shader material ABI diagnostics are non-blocking readiness rows");

    let report = streamer
        .material_readiness_report(&material_id)
        .expect("streamer readiness report");
    assert!(!report.is_ready());
    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::ShaderReadinessDiagnostic {
            source,
            path,
            diagnostic,
        } if *source == RenderMaterialDiagnosticSource::RendererMaterialAbi
            && path == "pipeline_layout.group3.binding0"
            && diagnostic.contains("uniform buffer")
    )));
    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::ShaderReadinessDiagnostic {
            source,
            path,
            diagnostic,
        } if *source == RenderMaterialDiagnosticSource::RendererMaterialAbi
            && path == "pipeline_layout.group3.binding1"
            && diagnostic.contains("supports only group 3 binding 0")
    )));
}

#[test]
fn render_product_streamer_dependency_readiness_change_invalidates_material_cache() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let texture_uri = locator("res://textures/cache-change.png");
    let texture_id = ResourceId::from_locator(&texture_uri);
    let material_uri = locator("res://materials/cache-change.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(texture_id, ResourceKind::Texture, texture_uri.clone()),
            rgba_texture("res://textures/cache-change.png"),
        )
        .expect("texture insert");
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material_with_refs(
                "builtin://shader/pbr.wgsl",
                Some("res://textures/cache-change.png"),
            ),
        )
        .expect("material insert");
    let mut streamer =
        ResourceStreamer::new_for_test(asset_manager.clone(), &device, &queue, &texture_layout);

    streamer
        .ensure_material(
            &device,
            &queue,
            &texture_layout,
            ResourceHandle::<MaterialMarker>::new(material_id),
        )
        .expect("initial material prepare");
    assert!(streamer
        .material_readiness_report(&material_id)
        .expect("initial report")
        .validation_errors
        .is_empty());

    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(texture_id, ResourceKind::Texture, texture_uri),
            container_texture("res://textures/cache-change.png"),
        )
        .expect("texture update");
    streamer
        .ensure_material(
            &device,
            &queue,
            &texture_layout,
            ResourceHandle::<MaterialMarker>::new(material_id),
        )
        .expect("non-blocking texture fallback remains allowed");

    let report = streamer
        .material_readiness_report(&material_id)
        .expect("updated report");
    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::TextureNotUploadReady { slot, reference, .. }
            if slot == "base_color_texture"
                && reference.locator == locator("res://textures/cache-change.png")
    )));
}
