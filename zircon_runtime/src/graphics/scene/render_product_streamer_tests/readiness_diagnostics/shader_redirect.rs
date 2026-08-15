use super::*;

use std::borrow::Cow;

use crate::asset::{ShaderDependencyAsset, ShaderImportRedirectAsset};
use crate::core::framework::render::{
    builtin_geometry_source_descriptor, ShaderPassType, GEOMETRY_SOURCE_ID_STATIC_MESH,
};
use crate::graphics::shader::{
    assemble_material_shader_template, validate_material_shader_template_wgsl_with_segments,
    MaterialShaderTemplateRequest, ShaderTemplateAssemblyError,
};
use crate::plugin::PluginShaderModuleSource;

#[test]
fn render_product_streamer_resolves_plugin_imports_into_validated_module_sources() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let surface_uri = locator("res://shaders/plugin-resolved-surface.zshader");
    let surface_id = ResourceId::from_locator(&surface_uri);
    let material_uri = locator("res://materials/plugin-resolved.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    let plugin_source = PluginShaderModuleSource::new(
        "com.zircon.fixture",
        "zircon_plugin::lighting",
        "fn plugin_lighting() -> vec3f { return vec3f(0.2, 0.4, 0.8); }",
        "fixture plugin shaders/lighting.wgsl",
    );
    let expected_content_hash = plugin_source.content_hash.clone();

    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(surface_id, ResourceKind::Shader, surface_uri),
            surface_shader_with_plugin_import(
                "res://shaders/plugin-resolved-surface.zshader",
                "zircon_plugin::lighting",
            ),
        )
        .expect("surface shader insert");
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material_with_refs("res://shaders/plugin-resolved-surface.zshader", None),
        )
        .expect("material insert");
    let mut streamer = ResourceStreamer::new_for_test_with_plugin_shader_module_sources(
        asset_manager,
        &device,
        &queue,
        &texture_layout,
        [plugin_source],
    )
    .expect("plugin module source binding registers");

    streamer
        .ensure_material(
            &device,
            &queue,
            &texture_layout,
            ResourceHandle::<MaterialMarker>::new(material_id),
        )
        .expect("plugin import prepares material");

    let includes = streamer.shader_module_include_sources(&surface_id);
    assert_eq!(includes.len(), 1);
    assert_eq!(includes[0].token, "zircon_plugin::lighting");
    assert_eq!(includes[0].owner_id, "com.zircon.fixture");
    assert_eq!(
        includes[0].diagnostic_origin,
        "fixture plugin shaders/lighting.wgsl"
    );
    assert_eq!(includes[0].source_hash, expected_content_hash);
    assert!(includes[0].source.contains("plugin_lighting"));
    assert_eq!(includes[0].content_hash, expected_content_hash);
    let assembly = material_template_assembly(&streamer, &surface_id, includes);
    assert!(assembly
        .wgsl_source
        .contains("fn plugin_lighting() -> vec3f"));
    assert!(assembly
        .include_content_hashes
        .contains(&expected_content_hash));
    validate_material_shader_template_wgsl_with_segments(&assembly.wgsl_source, &assembly.segments)
        .expect("plugin product assembly is valid WGSL");
    validate_wgpu_shader_module(
        &device,
        "zircon-product-plugin-module-source",
        &assembly.wgsl_source,
    );
}

#[test]
fn render_product_streamer_resolves_source_only_imports_into_validated_module_sources() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let surface_uri = locator("res://shaders/source-only-resolved-surface.zshader");
    let surface_id = ResourceId::from_locator(&surface_uri);
    let include_uri = locator("res://shaders/source-only-resolved-shared.zshader");
    let include_id = ResourceId::from_locator(&include_uri);
    let material_uri = locator("res://materials/source-only-resolved.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    let plugin_shadow_source = PluginShaderModuleSource::new(
        "com.zircon.fixture",
        "zircon_product::source_only_lighting",
        "fn source_only_lighting() -> vec3f { return vec3f(0.9, 0.1, 0.1); }",
        "fixture plugin shaders/source_only_lighting.wgsl",
    );

    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(include_id, ResourceKind::Shader, include_uri),
            include_shader(
                "res://shaders/source-only-resolved-shared.zshader",
                "zircon_product::source_only_lighting",
                "fn source_only_lighting() -> vec3f { return vec3f(0.25, 0.5, 0.75); }",
            ),
        )
        .expect("include shader insert");
    let mut surface_record = ResourceRecord::new(surface_id, ResourceKind::Shader, surface_uri);
    surface_record.dependency_ids = vec![include_id];
    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            surface_record,
            surface_shader_with_source_only_import(
                "res://shaders/source-only-resolved-surface.zshader",
                "zircon_product::source_only_lighting",
            ),
        )
        .expect("surface shader insert");
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material_with_refs("res://shaders/source-only-resolved-surface.zshader", None),
        )
        .expect("material insert");
    let mut streamer = ResourceStreamer::new_for_test_with_plugin_shader_module_sources(
        asset_manager,
        &device,
        &queue,
        &texture_layout,
        [plugin_shadow_source],
    )
    .expect("plugin shadow module source binding registers");

    streamer
        .ensure_material(
            &device,
            &queue,
            &texture_layout,
            ResourceHandle::<MaterialMarker>::new(material_id),
        )
        .expect("source-only import prepares material");

    let includes = streamer.shader_module_include_sources(&surface_id);
    assert_eq!(includes.len(), 1);
    assert_eq!(includes[0].token, "zircon_product::source_only_lighting");
    assert_eq!(includes[0].owner_id, format!("project:{include_id}"));
    assert!(includes[0]
        .diagnostic_origin
        .contains("source-only-resolved-shared.zshader"));
    assert!(includes[0].source.contains("vec3f(0.25, 0.5, 0.75)"));
    let assembly = material_template_assembly(&streamer, &surface_id, includes);
    assert!(assembly.wgsl_source.contains("vec3f(0.25, 0.5, 0.75)"));
    assert!(!assembly.wgsl_source.contains("vec3f(0.9, 0.1, 0.1)"));
    assert!(assembly.include_content_hashes.iter().any(|hash| {
        hash == &blake3::hash(
            b"fn source_only_lighting() -> vec3f { return vec3f(0.25, 0.5, 0.75); }",
        )
        .to_hex()
        .to_string()
    }));
    validate_material_shader_template_wgsl_with_segments(&assembly.wgsl_source, &assembly.segments)
        .expect("source-only product assembly is valid WGSL");
}

#[test]
fn render_product_streamer_preserves_module_cycle_diagnostics_after_dependency_preparation() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let surface_uri = locator("res://shaders/cyclic-surface.zshader");
    let surface_id = ResourceId::from_locator(&surface_uri);
    let first_uri = locator("res://shaders/cyclic-first.zshader");
    let first_id = ResourceId::from_locator(&first_uri);
    let second_uri = locator("res://shaders/cyclic-second.zshader");
    let second_id = ResourceId::from_locator(&second_uri);

    let mut first_record = ResourceRecord::new(first_id, ResourceKind::Shader, first_uri);
    first_record.dependency_ids = vec![second_id];
    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            first_record,
            include_shader(
                "res://shaders/cyclic-first.zshader",
                "zircon_product::cyclic_first",
                "#include <zircon_product::cyclic_second>\nfn cyclic_first() -> vec3f { return vec3f(0.25); }",
            ),
        )
        .expect("first cyclic include shader insert");
    let mut second_record = ResourceRecord::new(second_id, ResourceKind::Shader, second_uri);
    second_record.dependency_ids = vec![first_id];
    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            second_record,
            include_shader(
                "res://shaders/cyclic-second.zshader",
                "zircon_product::cyclic_second",
                "#include <zircon_product::cyclic_first>\nfn cyclic_second() -> vec3f { return vec3f(0.5); }",
            ),
        )
        .expect("second cyclic include shader insert");
    let mut surface_record = ResourceRecord::new(surface_id, ResourceKind::Shader, surface_uri);
    surface_record.dependency_ids = vec![first_id];
    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            surface_record,
            surface_shader_with_source_only_import(
                "res://shaders/cyclic-surface.zshader",
                "zircon_product::cyclic_first",
            ),
        )
        .expect("cyclic surface shader insert");
    let mut streamer =
        ResourceStreamer::new_for_test(asset_manager, &device, &queue, &texture_layout);

    let (prepared_id, _, fallback_report) = streamer
        .ensure_shader_source(&asset_reference("res://shaders/cyclic-surface.zshader"))
        .expect("cyclic shader dependencies prepare without recursive re-entry");

    assert_eq!(prepared_id, surface_id);
    assert!(fallback_report.is_none());
    assert!(streamer.shader_source(&surface_id).is_some());
    assert!(streamer.shader_source(&first_id).is_some());
    assert!(streamer.shader_source(&second_id).is_some());
    let modules = streamer.shader_module_include_sources(&surface_id);
    assert!(modules
        .iter()
        .any(|module| module.token == "zircon_product::cyclic_first"));
    assert!(modules
        .iter()
        .any(|module| module.token == "zircon_product::cyclic_second"));
    let error = try_material_template_assembly(&streamer, &surface_id, modules)
        .expect_err("source-only product modules must retain the circular include diagnostic");
    assert_eq!(
        error,
        ShaderTemplateAssemblyError::CircularModuleInclude {
            cycle: vec![
                "zircon_product::cyclic_first".to_string(),
                "zircon_product::cyclic_second".to_string(),
                "zircon_product::cyclic_first".to_string(),
            ],
        }
    );
}

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

fn surface_shader_with_source_only_import(uri: &str, import_path: &str) -> ShaderAsset {
    let mut shader = wgsl_shader(uri);
    shader.source = format!(
        "#include <{import_path}>\nfn zr_material_surface(input: ZrSurfaceInput) -> ZrMaterialSurface {{\n    var surface = zr_surface_default(input);\n    surface.base_color = vec4f(source_only_lighting(), 1.0);\n    return surface;\n}}"
    );
    shader.imports = vec![ShaderImportRedirectAsset {
        source: import_path.to_string(),
        redirect: None,
    }];
    shader
}

fn surface_shader_with_plugin_import(uri: &str, import_path: &str) -> ShaderAsset {
    let mut shader = wgsl_shader(uri);
    shader.source = format!(
        "#include <{import_path}>\nfn zr_material_surface(input: ZrSurfaceInput) -> ZrMaterialSurface {{\n    var surface = zr_surface_default(input);\n    surface.base_color = vec4f(plugin_lighting(), 1.0);\n    return surface;\n}}"
    );
    shader.imports = vec![ShaderImportRedirectAsset {
        source: import_path.to_string(),
        redirect: None,
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

fn material_template_assembly(
    streamer: &ResourceStreamer,
    surface_id: &ResourceId,
    includes: Vec<crate::graphics::shader::ShaderTemplateInclude>,
) -> crate::graphics::shader::MaterialShaderTemplateAssembly {
    try_material_template_assembly(streamer, surface_id, includes)
        .expect("material module source assembles")
}

fn try_material_template_assembly(
    streamer: &ResourceStreamer,
    surface_id: &ResourceId,
    includes: Vec<crate::graphics::shader::ShaderTemplateInclude>,
) -> Result<crate::graphics::shader::MaterialShaderTemplateAssembly, ShaderTemplateAssemblyError> {
    let geometry_source = builtin_geometry_source_descriptor(GEOMETRY_SOURCE_ID_STATIC_MESH)
        .expect("static mesh geometry source");
    assemble_material_shader_template(
        MaterialShaderTemplateRequest::new(
            geometry_source,
            ShaderPassType::Forward,
            streamer
                .shader_source(surface_id)
                .expect("surface source is prepared"),
            "zr_material_surface",
        )
        .with_module_include_sources(includes),
    )
}

fn validate_wgpu_shader_module(device: &wgpu::Device, label: &'static str, wgsl_source: &str) {
    let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let _shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(wgsl_source)),
    });
    let error = pollster::block_on(error_scope.pop());
    assert!(
        error.is_none(),
        "{label} should create a WGPU shader module: {error:?}"
    );
}
