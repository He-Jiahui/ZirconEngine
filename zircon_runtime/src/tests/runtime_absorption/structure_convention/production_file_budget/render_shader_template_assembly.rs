use super::{assert_contains_all, read_repo, read_runtime_src};

#[path = "render_shader_template_assembly/assembly_assertions.rs"]
mod assembly_assertions;
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

const PARENT_OWNER: &str =
    "tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly.rs";
const ASSERTIONS_OWNER: &str = "tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions.rs";
const DOCS_OWNER: &str = "tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly/docs_anchors.rs";
const SOURCES_OWNER: &str = "tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly/sources.rs";
const DEPTH_PREPASS_OWNER: &str = "tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly/depth_prepass_cache.rs";
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
    let docs = read_runtime_src(DOCS_OWNER);
    let sources = read_runtime_src(SOURCES_OWNER);
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support.rs",
    );
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support.rs",
    );

    assert_contains_all(
        "render shader template assembly parent only mounts support children",
        &parent,
        &[
            "mod assembly_assertions;",
            "mod depth_prepass_cache;",
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
        "render shader template assembly assertion owner keeps Rust/cache checks",
        &assertions,
        &[
            "pub(super) fn assert_render_shader_template_assembly_is_folder_backed",
            "template assembler owns request/output contract",
            "mesh pipeline shader source owner consumes standard material template source",
            "shadow replay resolves cache-backed variants at atlas execution time",
            "template unit tests cover geometry and pass dimensions",
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
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");
    for (label, source) in [
        ("status rows", status_rows.as_str()),
        ("status map", status_map.as_str()),
        ("date map", date_map.as_str()),
        ("Runtime 15 plan", runtime_plan.as_str()),
        ("runtime index", runtime_index.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("review findings", review_findings.as_str()),
        ("module convention", module_convention.as_str()),
        ("session note", session_note.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                slice,
                status,
                "structure_convention/production_file_budget/render_shader_template_assembly.rs",
                "structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions.rs",
                "structure_convention/production_file_budget/render_shader_template_assembly/docs_anchors.rs",
                "structure_convention/production_file_budget/render_shader_template_assembly/sources.rs",
                guard,
            ],
        );
    }
    assert_contains_all(
        "date map records the current slice date",
        &date_map,
        &[slice, "Some(\"2026-06-27\")"],
    );

    for owner in [
        PARENT_OWNER,
        ASSERTIONS_OWNER,
        DOCS_OWNER,
        SOURCES_OWNER,
        DEPTH_PREPASS_OWNER,
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
