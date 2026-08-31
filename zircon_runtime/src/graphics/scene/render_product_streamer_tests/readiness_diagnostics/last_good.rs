use super::*;

use crate::asset::ShaderImportRedirectAsset;

#[test]
fn render_product_streamer_blocking_reload_keeps_the_published_material_bundle() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let device = &backend.device;
    let queue = &backend.queue;
    let texture_layout = texture_bind_group_layout(device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = locator("res://materials/last-good.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    let ready_shader_uri = locator("res://shaders/last-good.zshader");
    let rejected_shader_uri = locator("res://shaders/rejected-source-only.glsl");
    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(
                ResourceId::from_locator(&ready_shader_uri),
                ResourceKind::Shader,
                ready_shader_uri,
            ),
            material_surface_shader("res://shaders/last-good.zshader"),
        )
        .expect("ready shader insert");
    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(
                ResourceId::from_locator(&rejected_shader_uri),
                ResourceKind::Shader,
                rejected_shader_uri,
            ),
            glsl_without_runtime_wgsl("res://shaders/rejected-source-only.glsl"),
        )
        .expect("rejected shader insert");
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri.clone()),
            material_with_refs("res://shaders/last-good.zshader", None),
        )
        .expect("ready material insert");
    let mut streamer =
        ResourceStreamer::new_for_test(asset_manager.clone(), device, queue, &texture_layout);
    let handle = ResourceHandle::<MaterialMarker>::new(material_id);
    streamer
        .ensure_material(&backend, device, queue, &texture_layout, handle)
        .expect("initial material publication");
    let published_revision = streamer
        .material_revision(&material_id)
        .expect("published material revision");
    let published_pipeline_key = streamer
        .material(&material_id)
        .expect("published material runtime")
        .pipeline_key
        .clone();
    let published_uniform = streamer.material_uniform(&material_id);
    let published_standard_uniform = streamer.standard_material_uniform(&material_id);

    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material_with_refs("res://shaders/rejected-source-only.glsl", None),
        )
        .expect("blocking material candidate insert");
    streamer
        .ensure_material(&backend, device, queue, &texture_layout, handle)
        .expect("blocking reload must keep the published material drawable");

    assert_eq!(
        streamer.material_revision(&material_id),
        Some(published_revision)
    );
    assert_ne!(
        streamer
            .rejected_material_candidate_revision(&material_id)
            .expect("rejected candidate revision"),
        published_revision,
        "the rejected candidate identity must remain distinct from the published revision"
    );
    assert_eq!(
        streamer
            .material(&material_id)
            .expect("last-good material runtime")
            .pipeline_key,
        published_pipeline_key
    );
    assert!(Arc::ptr_eq(
        &streamer.material_uniform(&material_id),
        &published_uniform
    ));
    assert!(Arc::ptr_eq(
        &streamer.standard_material_uniform(&material_id),
        &published_standard_uniform
    ));
    assert!(
        streamer
            .material_readiness_report(&material_id)
            .expect("rejected candidate readiness report")
            .validation_errors
            .iter()
            .any(|error| matches!(
                error,
                RenderMaterialValidationError::MissingRuntimeShaderSource
            ))
    );
}

#[test]
fn render_product_streamer_transitive_dependency_reload_keeps_then_replaces_last_good_bundle() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let device = &backend.device;
    let queue = &backend.queue;
    let texture_layout = texture_bind_group_layout(device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = locator("res://materials/dependency-last-good.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    let shader_uri = locator("res://shaders/dependency-last-good.zshader");
    let shader_id = ResourceId::from_locator(&shader_uri);
    let invalid_dependency_uri = locator("res://shaders/invalid-dependency.zshader");
    let dependency_id = ResourceId::from_locator(&invalid_dependency_uri);
    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(
                dependency_id,
                ResourceKind::Shader,
                invalid_dependency_uri.clone(),
            ),
            include_shader(
                "res://shaders/invalid-dependency.zshader",
                "invalid_dependency",
                "fn dependency_value() -> f32 { return 1.0; }",
            ),
        )
        .expect("ready dependency insert");
    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(shader_id, ResourceKind::Shader, shader_uri)
                .with_dependency_ids(vec![dependency_id]),
            material_surface_shader_with_redirect(
                "res://shaders/dependency-last-good.zshader",
                "res://shaders/invalid-dependency.zshader",
            ),
        )
        .expect("ready shader insert");
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material_with_refs("res://shaders/dependency-last-good.zshader", None),
        )
        .expect("ready material insert");
    let mut streamer =
        ResourceStreamer::new_for_test(asset_manager.clone(), device, queue, &texture_layout);
    let handle = ResourceHandle::<MaterialMarker>::new(material_id);
    streamer
        .ensure_material(&backend, device, queue, &texture_layout, handle)
        .expect("initial material publication");
    let published_material_revision = streamer
        .material_revision(&material_id)
        .expect("published material revision");
    let published_draw_generation = streamer
        .material_draw_generation(&material_id)
        .expect("published material draw generation");
    let published_pipeline_key = streamer
        .material(&material_id)
        .expect("published material runtime")
        .pipeline_key
        .clone();
    let published_uniform = streamer.material_uniform(&material_id);
    let published_standard_uniform = streamer.standard_material_uniform(&material_id);
    let published_shader_source = streamer
        .shader_source(&shader_id)
        .expect("published shader source")
        .to_string();

    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(
                dependency_id,
                ResourceKind::Shader,
                invalid_dependency_uri.clone(),
            ),
            wgsl_shader("res://shaders/invalid-dependency.zshader"),
        )
        .expect("invalid dependency insert");
    streamer
        .ensure_material(&backend, device, queue, &texture_layout, handle)
        .expect("dependency failure must keep the published material drawable");

    assert_eq!(
        streamer.shader_source(&shader_id),
        Some(published_shader_source.as_str()),
        "a shader candidate must publish only after its dependency closure succeeds"
    );
    assert_eq!(
        streamer.material_revision(&material_id),
        Some(published_material_revision)
    );
    assert_eq!(
        streamer.material_draw_generation(&material_id),
        Some(published_draw_generation),
        "a rejected dependency candidate must retain the last-good draw generation"
    );
    assert_eq!(
        streamer
            .rejected_material_candidate_revision(&material_id)
            .expect("rejected candidate revision"),
        published_material_revision,
        "shader-only reload must not require a synthetic material revision"
    );
    assert_eq!(
        streamer
            .material(&material_id)
            .expect("last-good material runtime")
            .pipeline_key,
        published_pipeline_key
    );
    assert!(Arc::ptr_eq(
        &streamer.material_uniform(&material_id),
        &published_uniform
    ));
    assert!(Arc::ptr_eq(
        &streamer.standard_material_uniform(&material_id),
        &published_standard_uniform
    ));
    assert!(
        streamer
            .material_readiness_report(&material_id)
            .expect("rejected dependency report")
            .validation_errors
            .iter()
            .any(|error| matches!(
                error,
                RenderMaterialValidationError::ShaderReadinessDiagnostic {
                    source: RenderMaterialDiagnosticSource::DependencyResolution,
                    path,
                    diagnostic,
                } if path == "dependencies.shader"
                    && diagnostic.contains("invalid surface source contract")
            ))
    );

    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(dependency_id, ResourceKind::Shader, invalid_dependency_uri),
            include_shader(
                "res://shaders/invalid-dependency.zshader",
                "invalid_dependency",
                "fn dependency_value() -> f32 { return 2.0; }",
            ),
        )
        .expect("recovered dependency insert");
    streamer
        .ensure_material(&backend, device, queue, &texture_layout, handle)
        .expect("recovered dependency must publish one replacement material generation");

    let recovered = streamer
        .material(&material_id)
        .expect("recovered material runtime");
    assert_eq!(
        recovered.pipeline_key.shader_revision, published_pipeline_key.shader_revision,
        "a leaf-only reload must not synthesize a root shader revision"
    );
    assert_ne!(
        recovered.pipeline_key.shader_dependency_revision,
        published_pipeline_key.shader_dependency_revision,
        "the recovered leaf closure must advance the runtime PSO identity"
    );
    assert_ne!(
        recovered.pipeline_key, published_pipeline_key,
        "all mesh passes must observe a new shared pipeline key after dependency recovery"
    );
    assert_ne!(
        streamer.material_draw_generation(&material_id),
        Some(published_draw_generation),
        "publishing a recovered dependency closure must invalidate cached draw payloads"
    );
    assert!(!Arc::ptr_eq(
        &streamer.material_uniform(&material_id),
        &published_uniform
    ));
    assert!(
        streamer
            .material_readiness_report(&material_id)
            .expect("recovered material readiness")
            .validation_errors
            .iter()
            .all(|error| !matches!(
                error,
                RenderMaterialValidationError::ShaderReadinessDiagnostic {
                    source: RenderMaterialDiagnosticSource::DependencyResolution,
                    ..
                }
            ))
    );
}

fn material_surface_shader(uri: &str) -> ShaderAsset {
    let mut shader = wgsl_shader(uri);
    shader.source = "fn zr_material_surface(input: ZrSurfaceInput) -> ZrSurfaceOutput {\n    return zr_surface_default(input);\n}"
        .to_string();
    shader
}

fn material_surface_shader_with_redirect(uri: &str, redirect_uri: &str) -> ShaderAsset {
    let mut shader = material_surface_shader(uri);
    shader.source = "#include <invalid_dependency>\nfn zr_material_surface(input: ZrSurfaceInput) -> ZrSurfaceOutput {\n    return zr_surface_default(input);\n}"
        .to_string();
    shader.imports = vec![ShaderImportRedirectAsset {
        source: "invalid_dependency".to_string(),
        redirect: Some(asset_reference(redirect_uri)),
    }];
    shader
}

fn include_shader(uri: &str, import_path: &str, source: &str) -> ShaderAsset {
    let mut shader = wgsl_shader(uri);
    shader.kind = ShaderAssetKind::Include;
    shader.import_path = Some(import_path.to_string());
    shader.source = source.to_string();
    shader
}
