use super::{assert_contains_all, read_repo};

const SLICE: &str =
    "Runtime 15 M3 editor retained-host pane data conversion projection owner guard";
const STATUS: &str =
    "runtime_15_editor_retained_host_pane_data_conversion_owner_guard_static_passed_cargo_deferred";
const DATE: &str = "2026-06-27";
const GUARD: &str =
    "runtime_15_editor_retained_host_pane_data_conversion_uses_child_projection_owners";
const PANE_DATA_MOD: &str = "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs";
const TEMPLATE_NODE_PROJECTION: &str =
    "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/template_node_projection.rs";
const ANIMATION_PROJECTION: &str =
    "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/animation_projection.rs";
const APPLY_PANE_CONVERSION: &str =
    "zircon_editor/src/ui/retained_host/ui/apply_presentation/pane_conversion.rs";
const FILE_BUDGET: usize = 800;

#[test]
fn runtime_15_editor_retained_host_pane_data_conversion_uses_child_projection_owners() {
    let pane_mod = read_repo(PANE_DATA_MOD);
    let template_node_projection = read_repo(TEMPLATE_NODE_PROJECTION);
    let animation_projection = read_repo(ANIMATION_PROJECTION);
    let apply_pane_conversion = read_repo(APPLY_PANE_CONVERSION);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let editor_workbench_doc = read_repo("docs/editor-and-tooling/editor-workbench-shell.md");
    let status_rows = read_repo(
        "zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget/hub_editor_support.rs",
    );
    let status_map = read_repo(
        "zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/hub_editor_maps.rs",
    );
    let date_map = read_repo(
        "zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/hub_editor_maps.rs",
    );

    assert_contains_all(
        "pane data conversion root delegates projection owners",
        &pane_mod,
        &[
            "mod animation_projection;",
            "mod template_node_projection;",
            "pub(crate) use self::animation_projection::",
            "use self::template_runtime_projection::",
        ],
    );
    for moved_owner in [
        "fn animation_template_projection(",
        "fn to_host_contract_animation_editor_pane(",
        "fn to_host_contract_pane(",
        "pub(super) fn project_nodes<",
    ] {
        assert!(
            !pane_mod.contains(moved_owner),
            "{PANE_DATA_MOD} should stay a thin projection module root and must not regain `{moved_owner}`"
        );
    }

    assert_contains_all(
        "template node projection owns shared node mapping",
        &template_node_projection,
        &[
            "pub(super) fn project_nodes<T, F>(",
            "pub(super) fn project_node_vec<T, F>(",
            "nodes.iter()",
            ".map(&mut map)",
        ],
    );
    assert_contains_all(
        "animation projection owns animation payload conversion",
        &animation_projection,
        &[
            "fn animation_template_projection(",
            "PanePayload::AnimationSequenceV1",
            "PanePayload::AnimationGraphV1",
            "project_nodes(&data.nodes, to_host_contract_template_node)",
        ],
    );
    assert_contains_all(
        "apply presentation pane conversion owns pane routing",
        &apply_pane_conversion,
        &[
            "pub(super) fn to_host_contract_pane(",
            "has_animation_payload",
            "to_host_contract_animation_editor_pane(&data, pane_size, component_showcase_runtime)",
            "pane_data_conversion::refresh_runtime_diagnostics_debug_reflector_from_body_surface",
        ],
    );

    for (path, source) in [
        (PANE_DATA_MOD, pane_mod.as_str()),
        (TEMPLATE_NODE_PROJECTION, template_node_projection.as_str()),
        (ANIMATION_PROJECTION, animation_projection.as_str()),
        (APPLY_PANE_CONVERSION, apply_pane_conversion.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < FILE_BUDGET,
            "{path} should stay below {FILE_BUDGET} lines after the F15 pane projection owner split; got {line_count}"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("editor workbench doc", editor_workbench_doc.as_str()),
        ("status row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                SLICE,
                STATUS,
                PANE_DATA_MOD,
                TEMPLATE_NODE_PROJECTION,
                ANIMATION_PROJECTION,
                APPLY_PANE_CONVERSION,
                GUARD,
            ],
        );
    }
    assert_contains_all("status map", &status_map, &[SLICE, STATUS]);
    assert_contains_all("date map", &date_map, &[SLICE, DATE]);
}
