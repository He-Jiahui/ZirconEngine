use super::{assert_contains_all, read_repo, read_runtime_src};

#[path = "render_shader_template_assembly/assembly_assertions.rs"]
mod assembly_assertions;
#[path = "render_shader_template_assembly/deferred_lighting_include.rs"]
mod deferred_lighting_include;
#[path = "render_shader_template_assembly/depth_prepass_cache.rs"]
mod depth_prepass_cache;
#[path = "render_shader_template_assembly/docs_anchors.rs"]
mod docs_anchors;
#[path = "render_shader_template_assembly/gbuffer_cache.rs"]
mod gbuffer_cache;
#[path = "render_shader_template_assembly/sources.rs"]
mod sources;
#[path = "render_shader_template_assembly/wgsl_contracts.rs"]
mod wgsl_contracts;

const PARENT_OWNER: &str = "tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly.rs";
const ASSERTIONS_OWNER: &str = "tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions.rs";
const ASSERTION_TEMPLATE_CONTRACTS_OWNER: &str = "tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions/template_contracts.rs";
const ASSERTION_MESH_CACHE_CONTRACTS_OWNER: &str = "tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions/mesh_cache_contracts.rs";
const ASSERTION_MESH_PIPELINE_SHADOW_GRAPH_CONTRACTS_OWNER: &str = "tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions/mesh_pipeline_shadow_graph_contracts.rs";
const ASSERTION_OWNER_BUDGET_OWNER: &str = "tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions/owner_budget.rs";
const DOCS_OWNER: &str = "tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly/docs_anchors.rs";
const SOURCES_OWNER: &str = "tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly/sources.rs";
const DEPTH_PREPASS_OWNER: &str = "tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly/depth_prepass_cache.rs";
const DEFERRED_LIGHTING_INCLUDE_OWNER: &str = "tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly/deferred_lighting_include.rs";
const GBUFFER_OWNER: &str = "tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly/gbuffer_cache.rs";
const WGSL_CONTRACTS_OWNER: &str = "tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly/wgsl_contracts.rs";

#[test]
fn runtime_15_render_shader_template_assembly_is_folder_backed() {
    let sources = sources::read_render_shader_template_assembly_sources();
    assembly_assertions::assert_render_shader_template_assembly_is_folder_backed(&sources);
    docs_anchors::assert_render_shader_template_assembly_docs_are_anchored();
}

#[test]
fn runtime_15_render_shader_template_assembly_support_children_are_folder_backed() {
    let parent = read_runtime_src(PARENT_OWNER);
    let assertions = read_runtime_src(ASSERTIONS_OWNER);
    let template_contracts = read_runtime_src(ASSERTION_TEMPLATE_CONTRACTS_OWNER);
    let mesh_cache_contracts = read_runtime_src(ASSERTION_MESH_CACHE_CONTRACTS_OWNER);
    let mesh_pipeline_shadow_graph_contracts =
        read_runtime_src(ASSERTION_MESH_PIPELINE_SHADOW_GRAPH_CONTRACTS_OWNER);
    let owner_budget = read_runtime_src(ASSERTION_OWNER_BUDGET_OWNER);
    let docs = read_runtime_src(DOCS_OWNER);
    let sources = read_runtime_src(SOURCES_OWNER);
    let mesh_cache_source =
        read_runtime_src("graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs");
    let mesh_cache_source_tests = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source/tests.rs",
    );
    let runtime_shading_model_source_tests = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source/tests/runtime_shading_model_sources.rs",
    );

    assert_contains_all(
        "render shader template assembly parent only mounts support children",
        &parent,
        &[
            "mod assembly_assertions;",
            "mod depth_prepass_cache;",
            "mod deferred_lighting_include;",
            "mod docs_anchors;",
            "mod gbuffer_cache;",
            "mod sources;",
            "mod wgsl_contracts;",
            "fn runtime_15_render_shader_template_assembly_is_folder_backed",
            "fn runtime_15_render_shader_template_assembly_support_children_are_folder_backed",
        ],
    );
    for moved_anchor in [
        ["let shader", "_mod = read_runtime_src("].concat(),
        ["let plan", "_08 = read_repo("].concat(),
        ["Shader template", " assembly foundation"].concat(),
        ["Velocity/TAA variant-id", " pipeline cache owner"].concat(),
        ["for (path, source)", " in ["].concat(),
    ] {
        assert!(
            !parent.contains(moved_anchor.as_str()),
            "render shader template assembly parent should not retain moved support anchor `{moved_anchor}`"
        );
    }

    assert_contains_all(
        "render shader template assembly assertion parent only mounts contract children",
        &assertions,
        &[
            "mod mesh_cache_contracts;",
            "mod mesh_pipeline_shadow_graph_contracts;",
            "mod owner_budget;",
            "mod template_contracts;",
            "pub(super) fn assert_render_shader_template_assembly_is_folder_backed",
            "template_contracts::assert_shader_template_contracts(sources)",
            "mesh_cache_contracts::assert_mesh_cache_contracts(sources)",
            "mesh_pipeline_shadow_graph_contracts::assert_mesh_pipeline_shadow_graph_contracts(sources)",
            "owner_budget::assert_render_shader_template_assembly_production_owners_stay_below_budget",
        ],
    );
    for moved_anchor in [
        "template assembler owns request/output contract".to_string(),
        "mesh pipeline shader source owner consumes standard material template source".to_string(),
        "shadow replay resolves cache-backed variants at atlas execution time".to_string(),
        "template unit tests cover geometry and pass dimensions".to_string(),
        ["for (path, source)", " in ["].concat(),
    ] {
        assert!(
            !assertions.contains(moved_anchor.as_str()),
            "render shader template assembly assertion parent should not retain moved contract anchor `{moved_anchor}`"
        );
    }
    assert_contains_all(
        "render shader template assertion template owner keeps shader/template checks",
        &template_contracts,
        &[
            "pub(super) fn assert_shader_template_contracts",
            "template assembler owns request/output contract",
            "taa reactive mask template assembler owns auxiliary template source assembly",
            "template include registry owns include_str and hashing",
            "template unit tests cover geometry and pass dimensions",
        ],
    );
    assert_contains_all(
        "render shader template assertion cache owner keeps mesh cache checks",
        &mesh_cache_contracts,
        &[
            "pub(super) fn assert_mesh_cache_contracts",
            "mesh pipeline shader source owner consumes standard material template source",
            "mesh pipeline cache stores non-base pipelines by variant id",
            "velocity pipeline cache consumes variant id and shader variant identity",
            "shadow pipeline cache consumes variant id and shader variant identity",
        ],
    );
    assert_contains_all(
        "render shader template assertion pipeline graph owner keeps runtime pipeline checks",
        &mesh_pipeline_shadow_graph_contracts,
        &[
            "pub(super) fn assert_mesh_pipeline_shadow_graph_contracts",
            "velocity mesh pipeline consumes template entry names",
            "shadow replay resolves cache-backed variants at atlas execution time",
            "shadow graph execution carries mesh pipeline context",
            "taa reactive mask mesh pipeline consumes template entry names",
        ],
    );
    assert_contains_all(
        "render shader template assertion owner budget stays independent",
        &owner_budget,
        &[
            "pub(super) fn assert_render_shader_template_assembly_production_owners_stay_below_budget",
            "graphics/shader/template/assemble.rs",
            "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs",
            "graphics/scene/scene_renderer/shadow/shadow_map_renderer.rs",
            "line_count < 800",
        ],
    );
    assert_contains_all(
        "render shader template assembly source owner keeps bulk source reads",
        &sources,
        &[
            "pub(super) struct RenderShaderTemplateAssemblySources",
            "pub(super) fn read_render_shader_template_assembly_sources",
            "graphics/shader/template/assemble.rs",
            "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs",
            "graphics/scene/scene_renderer/shadow/shadow_map_renderer.rs",
        ],
    );
    let shader_template_foundation = ["Shader template", " assembly foundation"].concat();
    assert_contains_all(
        "render shader template assembly docs owner keeps Plan 08 anchors",
        &docs,
        &[
            "pub(super) fn assert_render_shader_template_assembly_docs_are_anchored",
            shader_template_foundation.as_str(),
            "render_plan08_shader_template_assembly_foundation_static_passed_cargo_timeout_no_result",
            "Mesh pipeline shader source owner split",
            "runtime_15_render_shader_template_assembly_is_folder_backed",
        ],
    );

    let slice = "Runtime 15 M3 render shader template assembly guard support child-owner split";
    let status = "runtime_15_render_shader_template_assembly_guard_support_child_owner_split_static_passed_cargo_deferred";
    let guard = "runtime_15_render_shader_template_assembly_support_children_are_folder_backed";
    let runtime_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let module_convention = read_repo("docs/zircon_runtime/structure/module-convention.md");

    let assertion_slice =
        "Runtime 15 M3 render shader template assembly assertion contract child-owner split";
    let assertion_status = "runtime_15_render_shader_template_assembly_assertion_contract_child_owner_split_static_passed_cargo_deferred";

    assert_contains_all(
        "mesh pipeline shader source keeps tests in child owner",
        &mesh_cache_source,
        &[
            "#[cfg(test)]",
            "#[path = \"shader_source/tests.rs\"]",
            "mod tests;",
        ],
    );
    for moved_test_anchor in [
        "mesh_pipeline_standard_material_template_source_assembles_forward_base_source",
        "mesh_pipeline_standard_material_template_source_uses_requested_geometry_source",
        "mesh_pipeline_template_source_hashes_include_template_revision",
    ] {
        assert!(
            !mesh_cache_source.contains(moved_test_anchor),
            "mesh pipeline shader source production owner should not retain moved test `{moved_test_anchor}`"
        );
    }
    assert_contains_all(
        "mesh pipeline shader source tests child keeps source assembly tests",
        &mesh_cache_source_tests,
        &[
            "#[path = \"tests/runtime_shading_model_sources.rs\"]",
            "mod runtime_shading_model_sources;",
            "mesh_pipeline_standard_material_template_source_assembles_forward_base_source",
            "mesh_pipeline_standard_material_template_source_uses_requested_geometry_source",
            "mesh_pipeline_template_source_hashes_include_template_revision",
        ],
    );
    assert_contains_all(
        "mesh pipeline shader source WGPU validation remains child-owned",
        &runtime_shading_model_source_tests,
        &[
            "runtime_custom_shading_model_sources_compile_as_wgpu_modules",
            "ResourceStreamer::new_for_test_with_plugin_shading_models",
        ],
    );
    let shader_source_tests_slice =
        "Runtime 15 M3 mesh pipeline shader source tests child-owner split";
    let shader_source_tests_status = "runtime_15_mesh_pipeline_shader_source_tests_child_owner_split_static_passed_cargo_deferred";

    for owner in [
        PARENT_OWNER,
        ASSERTIONS_OWNER,
        ASSERTION_TEMPLATE_CONTRACTS_OWNER,
        ASSERTION_MESH_CACHE_CONTRACTS_OWNER,
        ASSERTION_MESH_PIPELINE_SHADOW_GRAPH_CONTRACTS_OWNER,
        ASSERTION_OWNER_BUDGET_OWNER,
        DOCS_OWNER,
        SOURCES_OWNER,
        DEPTH_PREPASS_OWNER,
        DEFERRED_LIGHTING_INCLUDE_OWNER,
        GBUFFER_OWNER,
        WGSL_CONTRACTS_OWNER,
    ] {
        let source = read_runtime_src(owner);
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{owner} should stay below the R4.3 test owner budget after support split; got {line_count}"
        );
    }
}
