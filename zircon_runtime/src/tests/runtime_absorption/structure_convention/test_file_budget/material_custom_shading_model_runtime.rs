use super::*;

const STATUS: &str =
    "render_plan08_material_custom_shading_model_runtime_registry_material_test_static_guard_passed_cargo_guard_timeout_renderdoc_deferred";

#[test]
fn runtime_15_material_custom_shading_model_runtime_registry_is_wired() {
    let extension_inputs = read_runtime_src("builtin/runtime_modules/assembly/extension_inputs.rs");
    let registration_inputs =
        read_runtime_src("builtin/runtime_modules/assembly/registration_inputs.rs");
    let target_modules = read_runtime_src("builtin/runtime_modules/assembly/target_modules.rs");
    let core_modules = read_runtime_src("builtin/runtime_modules/core_modules.rs");
    let graphics_module = read_runtime_src("graphics/runtime_builtin_graphics/mod.rs");
    let graphics_module_descriptor = read_runtime_src(
        "graphics/runtime_builtin_graphics/host/module_host/module_registration/module_descriptor.rs",
    );
    let graphics_module_create = read_runtime_src(
        "graphics/runtime_builtin_graphics/host/module_host/create/create_render_framework.rs",
    );
    let framework_construct = read_runtime_src(
        "graphics/runtime/render_framework/wgpu_render_framework_construction/construct.rs",
    );
    let scene_construct = read_runtime_src(
        "graphics/scene/scene_renderer/core/scene_renderer_construct/construct.rs",
    );
    let scene_icon_construct = read_runtime_src(
        "graphics/scene/scene_renderer/core/scene_renderer_construct/new_with_icon_source.rs",
    );
    let material_mod = read_runtime_src("graphics/material/mod.rs");
    let shading_builtins = read_runtime_src("graphics/material/shading_models/builtins.rs");
    let resource_streamer =
        read_runtime_src("graphics/scene/resources/resource_streamer/resource_streamer.rs");
    let resource_streamer_construction = read_runtime_src(
        "graphics/scene/resources/resource_streamer/resource_streamer_construction.rs",
    );
    let ensure_material = read_runtime_src(
        "graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs",
    );
    let material_capture = read_runtime_src(
        "graphics/scene/resources/resource_streamer/resource_streamer_accessors/material_capture.rs",
    );
    let material_tests =
        read_runtime_src("graphics/scene/render_product_streamer_tests/material_runtime.rs");
    let plan_08 = read_repo("docs/plans/zircon_runtime/render/08-material-shader-permutation.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/index.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let material_doc = read_repo("docs/zircon_runtime/core/framework/render/material.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");

    assert_contains_all(
        "runtime module assembly carries plugin shading models to graphics modules",
        &(extension_inputs + &registration_inputs + &target_modules + &core_modules),
        &[
            "collect_shading_models",
            "registry.shading_models().iter().cloned()",
            "shading_models: Vec<ShadingModelDescriptor>",
            "inputs.shading_models()",
            "shading_models.iter().cloned()",
        ],
    );
    assert_contains_all(
        "graphics module carries shading descriptors into the WGPU framework factory",
        &(graphics_module + &graphics_module_descriptor + &graphics_module_create),
        &[
            "plugin_shading_models: Vec<ShadingModelDescriptor>",
            "plugin_shading_models(&self)",
            "module_descriptor_with_render_features",
            "plugin_shading_models.to_vec()",
            "new_with_plugin_render_extensions_and_solari_and_compute_task_pool",
        ],
    );
    assert_contains_all(
        "WGPU renderer construction passes plugin shading models to the resource streamer",
        &(framework_construct + &scene_construct + &scene_icon_construct),
        &[
            "new_with_plugin_render_extensions_and_shading_models",
            "new_with_icon_source_and_plugin_render_features_and_shading_models",
            "plugin_shading_models.into_iter().collect::<Vec<_>>()",
            "ResourceStreamer::new_with_plugin_shading_models",
        ],
    );
    assert_contains_all(
        "resource streamer resolves material runtime and capture shading ids from the same registry",
        &(material_mod + &shading_builtins + &resource_streamer + &resource_streamer_construction
            + &ensure_material
            + &material_capture),
        &[
            "shading_model_registry_with_plugin_descriptors",
            "ShadingModelRegistry",
            "shading_model_registry: ShadingModelRegistry",
            "new_with_plugin_shading_models",
            ".resolve_lighting_model(&lighting_model)",
            "self.shading_model_id_for_lighting_model",
            ".resolve_lighting_model(model)",
        ],
    );
    assert_contains_all(
        "focused material runtime test covers plugin custom shading models",
        &material_tests,
        &[
            "render_product_streamer_projects_plugin_custom_shading_model_into_pipeline_key",
            "custom:subsurface",
            "new_for_test_with_plugin_shading_models",
            "plugin_shading_model_id",
            "material.pipeline_key.shading_model_id",
        ],
    );

    for (label, source) in [
        ("Plan 08", plan_08.as_str()),
        ("render index", render_index.as_str()),
        ("shader doc", shader_doc.as_str()),
        ("material doc", material_doc.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Material custom shading-model runtime registry",
                STATUS,
                "render_product_streamer_projects_plugin_custom_shading_model_into_pipeline_key",
                "runtime_15_material_custom_shading_model_runtime_registry_is_wired",
            ],
        );
    }
}
