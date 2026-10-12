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
        .expect("initial material candidate");
    assert!(streamer
        .published_material_draw_proxy(&material_id)
        .runtime()
        .is_none());
    assert!(streamer.publish_staged_material_candidate(material_id));
    let published_revision = streamer
        .resource_revision(material_id)
        .expect("material source revision");
    let published = streamer.published_material_draw_proxy(&material_id);
    let published_draw_generation = published.draw_generation().expect("published generation");
    let published_pipeline_key = published
        .runtime()
        .expect("published material runtime")
        .pipeline_key
        .clone();
    let published_uniform = published.uniform();
    let published_standard_uniform = published.standard_uniform();

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

    let retained = streamer.published_material_draw_proxy(&material_id);
    assert_eq!(retained.draw_generation(), Some(published_draw_generation));
    assert_eq!(
        streamer.material_draw_generations(&material_id),
        [Some(published_draw_generation), None, None],
        "rejection must not leave a staged or previous published generation"
    );
    assert_ne!(
        streamer
            .rejected_material_candidate_revision(&material_id)
            .expect("rejected candidate revision"),
        published_revision,
        "the rejected candidate identity must remain distinct from the published revision"
    );
    assert_eq!(
        retained
            .runtime()
            .expect("last-good material runtime")
            .pipeline_key,
        published_pipeline_key
    );
    assert!(Arc::ptr_eq(&retained.uniform(), &published_uniform));
    assert!(Arc::ptr_eq(
        &retained.standard_uniform(),
        &published_standard_uniform
    ));
    assert!(streamer
        .material_readiness_report(&material_id)
        .expect("rejected candidate readiness report")
        .validation_errors
        .iter()
        .any(|error| matches!(
            error,
            RenderMaterialValidationError::MissingRuntimeShaderSource
        )));
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
        .expect("initial material candidate");
    assert!(streamer
        .published_material_draw_proxy(&material_id)
        .runtime()
        .is_none());
    assert!(streamer.publish_staged_material_candidate(material_id));
    let published_material_revision = streamer
        .resource_revision(material_id)
        .expect("material source revision");
    let published = streamer.published_material_draw_proxy(&material_id);
    let published_draw_generation = published.draw_generation().expect("published generation");
    let published_pipeline_key = published
        .runtime()
        .expect("published material runtime")
        .pipeline_key
        .clone();
    let published_uniform = published.uniform();
    let published_standard_uniform = published.standard_uniform();
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
        streamer.resource_revision(material_id).unwrap(),
        published_material_revision
    );
    let retained = streamer.published_material_draw_proxy(&material_id);
    assert_eq!(
        retained.draw_generation(),
        Some(published_draw_generation),
        "a rejected dependency candidate must retain the last-good draw generation"
    );
    assert!(streamer.staged_material_candidate(&material_id).is_none());
    assert_eq!(
        streamer
            .rejected_material_candidate_revision(&material_id)
            .expect("rejected candidate revision"),
        published_material_revision,
        "shader-only reload must not require a synthetic material revision"
    );
    assert_eq!(
        retained
            .runtime()
            .expect("last-good material runtime")
            .pipeline_key,
        published_pipeline_key
    );
    assert!(Arc::ptr_eq(&retained.uniform(), &published_uniform));
    assert!(Arc::ptr_eq(
        &retained.standard_uniform(),
        &published_standard_uniform
    ));
    assert!(streamer
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
        )));

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
        .expect("recovered dependency must stage one replacement material generation");

    let candidate_generation = streamer
        .staged_material_draw_generation(&material_id)
        .expect("recovered material candidate generation");
    assert_ne!(candidate_generation, published_draw_generation);
    let retained = streamer.published_material_draw_proxy(&material_id);
    assert_eq!(retained.draw_generation(), Some(published_draw_generation));
    assert_eq!(
        retained.runtime().unwrap().pipeline_key,
        published_pipeline_key,
        "preparing a recovered candidate must not publish it before the boundary"
    );
    assert!(Arc::ptr_eq(&retained.uniform(), &published_uniform));
    assert!(Arc::ptr_eq(
        &retained.standard_uniform(),
        &published_standard_uniform
    ));
    assert!(streamer.publish_staged_material_candidate(material_id));

    let recovered = streamer.published_material_draw_proxy(&material_id);
    let recovered_runtime = recovered.runtime().expect("recovered material runtime");
    assert_eq!(
        recovered_runtime.pipeline_key.shader_revision, published_pipeline_key.shader_revision,
        "a leaf-only reload must not synthesize a root shader revision"
    );
    assert_ne!(
        recovered_runtime.pipeline_key.shader_dependency_identity,
        published_pipeline_key.shader_dependency_identity,
        "the recovered leaf closure must advance the runtime PSO identity"
    );
    assert_ne!(
        recovered_runtime.pipeline_key, published_pipeline_key,
        "all mesh passes must observe a new shared pipeline key after dependency recovery"
    );
    assert_eq!(recovered.draw_generation(), Some(candidate_generation));
    assert_eq!(
        streamer.material_draw_generations(&material_id),
        [
            Some(candidate_generation),
            Some(published_draw_generation),
            None
        ],
        "publication must retain exactly the previous last-good generation"
    );
    assert_eq!(
        streamer.resource_revision(material_id).unwrap(),
        published_material_revision,
        "a dependency-only recovery must not synthesize a material revision"
    );
    assert!(!Arc::ptr_eq(&recovered.uniform(), &published_uniform));
    assert!(streamer
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
        )));
}

#[test]
fn render_product_streamer_cached_shader_rechecks_removed_dependency_before_hit() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let device = &backend.device;
    let queue = &backend.queue;
    let texture_layout = texture_bind_group_layout(device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let root_uri = locator("res://shaders/cache-root.zshader");
    let root_id = ResourceId::from_locator(&root_uri);
    let child_uri = locator("res://shaders/cache-child.zshader");
    let child_id = ResourceId::from_locator(&child_uri);

    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(child_id, ResourceKind::Shader, child_uri.clone()),
            include_shader(
                "res://shaders/cache-child.zshader",
                "cache_child",
                "fn cache_child() -> vec3f { return vec3f(0.2); }",
            ),
        )
        .expect("cache child insert");
    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(root_id, ResourceKind::Shader, root_uri.clone())
                .with_dependency_ids(vec![child_id]),
            material_surface_shader("res://shaders/cache-root.zshader"),
        )
        .expect("cache root insert");

    let mut streamer =
        ResourceStreamer::new_for_test(asset_manager.clone(), device, queue, &texture_layout);
    let root_reference = asset_reference("res://shaders/cache-root.zshader");
    streamer
        .ensure_shader_source(&root_reference)
        .expect("initial cache root preparation");
    let root_source = streamer
        .shader_source(&root_id)
        .expect("published cache root source")
        .to_string();

    asset_manager
        .assets::<ShaderAsset>()
        .remove_by_locator(&child_uri)
        .expect("remove cache child");
    let error = streamer
        .ensure_shader_source(&root_reference)
        .expect_err("a current root must not hide a removed dependency");
    assert!(error.to_string().contains("shader dependency is not ready"));
    assert_eq!(streamer.shader_source(&root_id), Some(root_source.as_str()));
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
