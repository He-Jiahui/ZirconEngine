use std::path::Path;

use super::super::{assert_contains_all, read_repo_text};

#[test]
fn runtime_15_editor_workbench_authority_label_uses_editor_name() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let feedback_source = read_repo_text(
        manifest_root,
        "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/gameplay_state.rs",
    );
    let audit_script = read_repo_text(
        manifest_root,
        ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/non_network_server_naming.py",
    );
    let runtime_15_plan = read_repo_text(
        manifest_root,
        "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
    );
    let runtime_index = read_repo_text(manifest_root, "docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-review-findings-2026-06.md",
    );
    let structure_convention = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-structure-convention.md",
    );
    let module_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/structure/module-convention.md",
    );
    let non_network_doc = read_repo_text(
        manifest_root,
        "docs/engine-architecture/non-network-server-naming-m1.md",
    );
    let editor_commands_doc =
        read_repo_text(manifest_root, "docs/zircon_editor/ui/host/commands.md");
    let status_rows = read_repo_text(
        manifest_root,
        "zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
    );
    let status_slice = read_repo_text(
        manifest_root,
        "zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15.rs",
    );
    let date_slice = read_repo_text(
        manifest_root,
        "zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15.rs",
    );

    assert_contains_all(
        "editor workbench extension feedback",
        &feedback_source,
        &[
            "workbench.extension.spawn_rules.condition_night_table_row.select",
            "Selected Condition_Night   editor authority",
        ],
    );
    assert!(
        !feedback_source.contains("server authority"),
        "editor workbench extension feedback should not use non-network server authority wording"
    );
    assert!(
        !audit_script.contains("editor-workbench-authority-label-debt"),
        "runtime structure audit should not retain the retired editor workbench authority-label debt bucket"
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan),
        ("runtime index", runtime_index),
        ("review findings", review_findings),
        ("structure convention", structure_convention),
        ("module convention doc", module_doc),
        ("non-network server naming doc", non_network_doc),
        ("editor commands doc", editor_commands_doc),
        ("status row data", status_rows),
        ("status slice", status_slice),
        ("date slice", date_slice),
    ] {
        assert_contains_all(
            label,
            &source,
            &[
                "Runtime 15 M2 editor workbench authority-label naming hard cutover",
                "runtime_15_editor_workbench_authority_label_naming_hard_cutover_static_passed_cargo_deferred",
                "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/gameplay_state.rs",
                "Selected Condition_Night   editor authority",
                "runtime_15_editor_workbench_authority_label_uses_editor_name",
            ],
        );
    }
}
