use super::*;

use crate::asset::{ShaderDependencyAsset, ShaderImportRedirectAsset};

#[test]
fn render_product_streamer_resolves_shader_redirect_imports_into_module_sources() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let surface_uri = locator("res://shaders/redirect-resolved-surface.zshader");
    let surface_id = ResourceId::from_locator(&surface_uri);
    let include_uri = locator("res://shaders/redirect-resolved-shared.zshader");
    let include_id = ResourceId::from_locator(&include_uri);
    let material_uri = locator("res://materials/redirect-resolved.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);

    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(include_id, ResourceKind::Shader, include_uri.clone()),
            include_shader(
                "res://shaders/redirect-resolved-shared.zshader",
                "zircon_product::lighting",
                "fn redirected_lighting() -> vec3f { return vec3f(0.25, 0.5, 0.75); }",
            ),
        )
        .expect("include shader insert");
    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(surface_id, ResourceKind::Shader, surface_uri),
            surface_shader_with_redirect_import(
                "res://shaders/redirect-resolved-surface.zshader",
                "zircon_product::lighting",
                "res://shaders/redirect-resolved-shared.zshader",
            ),
        )
        .expect("surface shader insert");
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material_with_refs("res://shaders/redirect-resolved-surface.zshader", None),
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
        .expect("resolved redirect import prepares material");

    let report = streamer
        .material_readiness_report(&material_id)
        .expect("streamer readiness report");
    assert!(report.validation_errors.is_empty());
    assert!(!report.uses_fallback());
    assert!(streamer
        .shader_source(&include_id)
        .expect("redirect include shader prepared")
        .contains("redirected_lighting"));
    let includes = streamer.shader_module_include_sources(&surface_id);
    assert_eq!(includes.len(), 1);
    assert_eq!(includes[0].token, "zircon_product::lighting");
    assert!(includes[0].source.contains("redirected_lighting"));
}

#[test]
fn render_product_streamer_reports_missing_shader_redirect_import_as_fallback() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let surface_uri = locator("res://shaders/redirect-missing-surface.zshader");
    let surface_id = ResourceId::from_locator(&surface_uri);
    let material_uri = locator("res://materials/redirect-missing.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);

    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(surface_id, ResourceKind::Shader, surface_uri),
            surface_shader_with_redirect_import(
                "res://shaders/redirect-missing-surface.zshader",
                "zircon_product::missing_lighting",
                "res://shaders/redirect-missing-shared.zshader",
            ),
        )
        .expect("surface shader insert");
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material_with_refs("res://shaders/redirect-missing-surface.zshader", None),
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
        .expect("missing redirect import remains a non-blocking material fallback");

    let report = streamer
        .material_readiness_report(&material_id)
        .expect("streamer readiness report");
    assert!(!report.is_ready());
    assert!(report.uses_fallback());
    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::UnresolvedShaderReference { reference }
            if reference.locator == locator("res://shaders/redirect-missing-shared.zshader")
    )));
    assert!(report.fallback_usages.iter().any(|usage| matches!(
        &usage.reason,
        RenderMaterialFallbackReason::Shader { reference }
            if reference.locator == locator("res://shaders/redirect-missing-shared.zshader")
    )));
}

fn surface_shader_with_redirect_import(
    uri: &str,
    import_path: &str,
    redirect_uri: &str,
) -> ShaderAsset {
    let mut shader = wgsl_shader(uri);
    shader.source = format!(
        "#include <{import_path}>\nfn zr_material_surface(input: ZrSurfaceInput) -> ZrMaterialSurface {{\n    var surface = zr_surface_default(input);\n    surface.base_color = vec4f(redirected_lighting(), 1.0);\n    return surface;\n}}"
    );
    let redirect = asset_reference(redirect_uri);
    shader.imports = vec![ShaderImportRedirectAsset {
        source: import_path.to_string(),
        redirect: Some(redirect.clone()),
    }];
    shader.dependencies = vec![ShaderDependencyAsset {
        kind: ResourceKind::Shader,
        reference: redirect,
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
