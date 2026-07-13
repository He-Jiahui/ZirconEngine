use super::*;

const STATUS: &str = "render_plan08_material_custom_shading_model_runtime_registry_material_test_static_guard_passed_cargo_guard_timeout_renderdoc_deferred";
const SELECTED_PLUGIN_STATUS: &str =
    "render_plan08_selected_plugin_shading_model_registration_inputs_static_guard_cargo_deferred";
const INCLUDE_SOURCE_SET_STATUS: &str =
    "render_plan08_shading_model_include_source_set_static_passed_cargo_deferred";
const RUNTIME_HANDOFF_STATUS: &str =
    "render_plan08_shading_model_include_source_runtime_handoff_static_passed_cargo_deferred";
const CUSTOM_WGPU_MODULE_STATUS: &str =
    "render_plan08_custom_shading_model_runtime_wgpu_module_passed_product_renderdoc_deferred";
const DEFERRED_LIGHTING_WGPU_PIPELINE_STATUS: &str = "render_plan08_deferred_lighting_custom_include_wgpu_pipeline_passed_product_renderdoc_deferred";
const PRODUCT_STAGED_CACHE_WGPU_STATUS: &str = "render_plan08_custom_shading_model_product_material_pass_staged_cache_wgpu_passed_renderdoc_deferred";
const PRODUCT_SECOND_LAUNCH_WGPU_STATUS: &str =
    "render_plan08_custom_shading_model_second_launch_staged_cache_wgpu_passed_renderdoc_deferred";
const PRODUCT_READBACK_WGPU_STATUS: &str = "render_plan08_custom_shading_model_deferred_lighting_product_readback_wgpu_passed_renderdoc_deferred";
const PRODUCT_READBACK_PNG_STATUS: &str = "render_plan08_custom_shading_model_deferred_lighting_product_readback_png_passed_renderdoc_deferred";
const PRODUCT_GROUP_DIRECT_BINARY_STATUS: &str =
    "render_plan08_custom_shading_model_product_group_direct_binary_wgpu_passed_renderdoc_deferred";
const PRODUCT_GROUP_CARGO_WRAPPER_STATUS: &str =
    "render_plan08_custom_shading_model_product_group_cargo_wrapper_wgpu_passed_renderdoc_deferred";
const PRODUCT_GROUP_DEFAULT_FEATURES_STATUS: &str = "render_plan08_custom_shading_model_product_group_default_features_wgpu_passed_renderdoc_deferred";
const SOURCE_REGISTRY_CARGO_WRAPPER_STATUS: &str =
    "render_plan08_selected_plugin_source_registry_guards_cargo_wrapper_passed_renderdoc_deferred";

#[test]
fn runtime_15_material_custom_shading_model_runtime_registry_is_wired() {
    let extension_inputs = read_runtime_src("builtin/runtime_modules/assembly/extension_inputs.rs");
    let registration_inputs =
        read_runtime_src("builtin/runtime_modules/assembly/registration_inputs.rs");
    let registration_input_tests =
        read_runtime_src("builtin/runtime_modules/assembly/registration_inputs/tests.rs");
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
    let scene_execute_graph_stage = read_runtime_src(
        "graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs",
    );
    let material_mod = read_runtime_src("graphics/material/mod.rs");
    let shading_builtins = read_runtime_src("graphics/material/shading_models/builtins.rs");
    let shading_include_sources =
        read_runtime_src("graphics/material/shading_models/include_sources.rs");
    let shader_template_assemble = read_runtime_src("graphics/shader/template/assemble.rs");
    let shader_template_gbuffer = read_runtime_src("graphics/shader/template/deferred_gbuffer.rs");
    let deferred_lighting_source = read_runtime_src(
        "graphics/scene/scene_renderer/deferred/lighting_pipeline/shader_source.rs",
    );
    let deferred_lighting_tests =
        read_runtime_src("graphics/scene/scene_renderer/deferred/lighting_pipeline/tests.rs");
    let deferred_lighting_runtime_pipeline_tests = read_runtime_src(
        "graphics/scene/scene_renderer/deferred/lighting_pipeline/tests/runtime_pipeline.rs",
    );
    let deferred_lighting_create =
        read_runtime_src("graphics/scene/scene_renderer/deferred/lighting_pipeline/create.rs");
    let deferred_resources_construct = read_runtime_src(
        "graphics/scene/scene_renderer/deferred/deferred_scene_resources/construct.rs",
    );
    let deferred_gbuffer_record = read_runtime_src(
        "graphics/scene/scene_renderer/deferred/deferred_scene_resources/record_gbuffer_geometry.rs",
    );
    let render_pass_gpu_deferred = read_runtime_src(
        "graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/deferred.rs",
    );
    let render_pass_gpu_context = read_runtime_src(
        "graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs",
    );
    let mesh_shader_source =
        read_runtime_src("graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs");
    let mesh_shader_source_tests = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source/tests.rs",
    );
    let mesh_shader_source_runtime_tests = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source/tests/runtime_shading_model_sources.rs",
    );
    let ensure_gbuffer_pipeline = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_gbuffer_pipeline.rs",
    );
    let resource_streamer =
        read_runtime_src("graphics/scene/resources/resource_streamer/resource_streamer.rs");
    let resource_streamer_mod =
        read_runtime_src("graphics/scene/resources/resource_streamer/mod.rs");
    let resource_streamer_shading_models = read_runtime_src(
        "graphics/scene/resources/resource_streamer/resource_streamer_shading_models.rs",
    );
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
    let dynamic_shader_prewarm = read_runtime_src("dynamic_api/shader_prewarm.rs");
    let material_pass_product_custom = read_runtime_src(
        "graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache/custom_shading_model.rs",
    );
    let material_pass_product_custom_png = read_runtime_src(
        "graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache/custom_product_png.rs",
    );
    let material_pass_product_custom_second_launch = read_runtime_src(
        "graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache/custom_second_launch.rs",
    );
    let material_pass_product_manifest = read_runtime_src(
        "graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache/manifest.rs",
    );
    let material_pass_product_fixture = read_runtime_src(
        "graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache/fixture.rs",
    );
    let material_pass_product_assertions = read_runtime_src(
        "graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache/assertions.rs",
    );
    let plan_08 = read_repo(
        "docs/plans/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let material_doc = read_repo("docs/zircon_runtime/core/framework/render/material.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");

    assert_contains_all(
        "runtime module assembly carries plugin shading models to graphics modules",
        &(extension_inputs
            + &registration_inputs
            + &registration_input_tests
            + &target_modules
            + &core_modules),
        &[
            "collect_shading_models",
            "registry.shading_models().iter().cloned()",
            "shading_models: Vec<ShadingModelDescriptor>",
            "inputs.shading_models()",
            "shading_models.iter().cloned()",
            "plugin_registration_inputs_collect_shading_model_descriptors",
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
        "project plugin shader records export shading-model include source sets to template requests",
        &(shading_include_sources
            + &shader_template_assemble
            + &shader_template_gbuffer
            + &deferred_lighting_source),
        &[
            "ShadingModelIncludeSourceSet",
            "from_project_asset_manager",
            "ready_records_for_kind",
            "ResourceKind::Shader",
            "runtime_wgsl_source",
            "MissingInclude",
            "DuplicateIncludeToken",
            "with_shading_model_forward_include_sources",
            "with_shading_model_gbuffer_include_sources",
            "with_shading_model_deferred_include_sources",
            "exported_include_source_set_feeds_forward_and_gbuffer_template_requests",
        ],
    );
    assert_contains_all(
        "runtime SceneRenderer handoff feeds shading-model descriptors and include sources to mesh and deferred pipelines",
        &(resource_streamer_mod
            + &resource_streamer_shading_models
            + &mesh_shader_source
            + &ensure_gbuffer_pipeline
            + &deferred_gbuffer_record
            + &render_pass_gpu_deferred
            + &render_pass_gpu_context
            + &deferred_lighting_create
            + &deferred_resources_construct
            + &scene_icon_construct
            + &scene_execute_graph_stage),
        &[
            "mod resource_streamer_shading_models;",
            "shading_model_descriptor_for_pipeline_key",
            "shading_model_include_source_set",
            "ShadingModelIncludeSourceSet::from_project_asset_manager",
            "with_runtime_shading_model_sources",
            "with_shading_model_forward_include_sources",
            "with_shading_model_gbuffer_include_sources",
            "ensure_gbuffer_pipeline_for_variant(device, streamer, gbuffer_variant_id)",
            "record_gbuffer_geometry(",
            "streamer: &ResourceStreamer",
            "with_deferred_renderer(deferred, streamer, mesh_draw_lists)",
            "with_deferred_lighting_renderer(deferred, mesh_draw_lists)",
            "assemble_deferred_lighting_shader_source",
            "with_shading_model_deferred_include_sources",
            "create_lighting_pipeline(",
            "plugin_shading_models.iter().cloned()",
        ],
    );
    assert_contains_all(
        "runtime custom shading-model source handoff creates WGPU shader modules",
        &(mesh_shader_source + &mesh_shader_source_tests + &mesh_shader_source_runtime_tests),
        &[
            "mod runtime_shading_model_sources;",
            "runtime_custom_shading_model_sources_compile_as_wgpu_modules",
            "new_for_test_with_plugin_shading_models",
            "mesh_pipeline_shader_source_for_geometry_descriptor",
            "mesh_pipeline_deferred_gbuffer_template_source_for_geometry_descriptor_with_streamer",
            "ZR_SHADING_TOON_DEBUG_ID",
            "ZR_GBUFFER_TOON_DEBUG_ID",
            "device.create_shader_module",
        ],
    );
    assert_contains_all(
        "deferred lighting custom include source creates a WGPU render pipeline through product layouts",
        &(deferred_lighting_tests
            + &deferred_lighting_runtime_pipeline_tests
            + &deferred_lighting_create),
        &[
            "mod runtime_pipeline;",
            "custom_shading_model_deferred_lighting_pipeline_creates_with_project_include_source",
            "ProjectAssetManager::default",
            "ResourceRecord::new",
            "register_ready",
            "create_lighting_bind_group_layout",
            "GpuScene::new",
            "scene_bind_group_layout()",
            "create_lighting_pipeline",
            "device.create_shader_module",
            "device.create_render_pipeline",
            "push_error_scope",
            "pop()",
        ],
    );
    assert_contains_all(
        "custom shading-model product material passes use staged prewarm cache through WGPU product submit",
        &(dynamic_shader_prewarm
            + &material_pass_product_custom
            + &material_pass_product_custom_png
            + &material_pass_product_custom_second_launch
            + &material_pass_product_manifest
            + &material_pass_product_fixture
            + &material_pass_product_assertions
            + &render_pass_gpu_context
            + &scene_execute_graph_stage),
        &[
            "builtin_standard_material_shader_prewarm_manifest_for_geometry_with_plugin_shading_models",
            "PluginShadingModelTemplateSource",
            "registry_material_pass_product_prewarm_manifest_with_plugin_shading_models",
            "render_product_custom_shading_model_registry_material_passes_use_staged_prewarm_without_compile_miss",
            "prewarm_shader_variants_with_wgpu_pipeline_validation",
            "submit_registry_material_passes_with_plugin_shading_model",
            "RegistryMaterialPassPluginShadingModel",
            "register_shader_includes",
            "custom:toon",
            "SHADING_MODEL_PLUGIN_ID_START",
            "assert_registry_material_pass_first_frame_shader_cache_hit_for_shading_model",
            "assert_registry_material_pass_velocity_frame_shader_cache_hit_for_shading_model",
            "render_product_custom_shading_model_second_launch_uses_staged_prewarm_without_compile_miss",
            "render_product_custom_shading_model_deferred_lighting_readback_uses_project_include",
            "export_custom_shading_model_deferred_lighting_product_png",
            PRODUCT_READBACK_PNG_STATUS,
            "runtime_render_plan08_custom_shading_model_deferred_lighting_20260704.png",
            "save_side_by_side_product_frames",
            "assert_runtime_shader_cache_root_empty",
            "second custom shading-model product launch should still stay read-only against staged cache",
            "with_deferred_lighting_renderer",
            "submit_registry_material_passes_with_plugin_shading_model_capture",
            "first_capture",
            "velocity_capture",
            "capture_frame",
            "select_visible_registry_material_pass_camera",
            "registry_material_pass_viewport_size",
            "frame_rgb_summary",
            "max(0.65",
            "custom toon deferred lighting include should tint the product frame green",
        ],
    );
    assert_contains_all(
        "resource streamer resolves material runtime and capture shading ids from the same registry",
        &(material_mod
            + &shading_builtins
            + &resource_streamer
            + &resource_streamer_construction
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
                SELECTED_PLUGIN_STATUS,
                "Project/plugin shading-model include source set",
                INCLUDE_SOURCE_SET_STATUS,
                "Runtime shading-model include source handoff",
                RUNTIME_HANDOFF_STATUS,
                "Custom shading-model runtime WGPU module validation",
                CUSTOM_WGPU_MODULE_STATUS,
                "runtime_custom_shading_model_sources_compile_as_wgpu_modules",
                "Deferred lighting custom include WGPU pipeline validation",
                DEFERRED_LIGHTING_WGPU_PIPELINE_STATUS,
                "custom_shading_model_deferred_lighting_pipeline_creates_with_project_include_source",
                PRODUCT_STAGED_CACHE_WGPU_STATUS,
                "Custom shading-model second-launch staged-cache WGPU validation",
                PRODUCT_SECOND_LAUNCH_WGPU_STATUS,
                "render_product_custom_shading_model_second_launch_uses_staged_prewarm_without_compile_miss",
                "Custom shading-model deferred-lighting product readback",
                PRODUCT_READBACK_WGPU_STATUS,
                "render_product_custom_shading_model_deferred_lighting_readback_uses_project_include",
                "Custom shading-model deferred-lighting product readback PNG",
                PRODUCT_READBACK_PNG_STATUS,
                "export_custom_shading_model_deferred_lighting_product_png",
                "runtime_render_plan08_custom_shading_model_deferred_lighting_20260704.png",
                "submit_registry_material_passes_with_plugin_shading_model_capture",
                "select_visible_registry_material_pass_camera",
                "Custom shading-model product group direct-binary sweep",
                PRODUCT_GROUP_DIRECT_BINARY_STATUS,
                "render_product_custom_shading_model",
                "3/3",
                "Custom shading-model product group Cargo-wrapper WGPU backfill",
                PRODUCT_GROUP_CARGO_WRAPPER_STATUS,
                "5850 filtered",
                "19.04s",
                "Custom shading-model product group default-feature WGPU backfill",
                PRODUCT_GROUP_DEFAULT_FEATURES_STATUS,
                "5932 filtered",
                "21.45s",
                "Selected plugin/source-registry guard Cargo-wrapper backfill",
                SOURCE_REGISTRY_CARGO_WRAPPER_STATUS,
                "shader_reimport_exports_updated_revision_for_prewarm_registry",
                "exported_include_source_set_feeds_forward_and_gbuffer_template_requests",
                "5839 filtered",
                "custom:toon",
                "with_deferred_lighting_renderer",
                "ShadingModelIncludeSourceSet::from_project_asset_manager",
                "with_shading_model_deferred_include_sources",
                "plugin_registration_inputs_collect_shading_model_descriptors",
                "render_product_streamer_projects_plugin_custom_shading_model_into_pipeline_key",
                "runtime_15_material_custom_shading_model_runtime_registry_is_wired",
            ],
        );
    }
}
