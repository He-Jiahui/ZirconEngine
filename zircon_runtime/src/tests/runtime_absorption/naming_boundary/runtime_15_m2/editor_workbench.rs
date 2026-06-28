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
        "zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
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

#[test]
fn runtime_15_editor_workbench_archived_fixtures_use_current_names() {
    const SLICE: &str = "Runtime 15 M2 editor Workbench archived fixture naming hard cutover";
    const STATUS: &str =
        "runtime_15_editor_workbench_archived_fixture_naming_hard_cutover_static_passed_cargo_deferred";
    const GUARD: &str = "runtime_15_editor_workbench_archived_fixtures_use_current_names";

    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_root
        .parent()
        .expect("zircon_runtime manifest should live under repository root");
    let renderer_mod = read_repo_text(
        manifest_root,
        "zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer.rs",
    );
    let host_window = read_repo_text(
        manifest_root,
        "zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/host_window.rs",
    );
    let paint_commands = read_repo_text(
        manifest_root,
        "zircon_editor/src/ui/retained_host/host_contract/paint_workbench/commands.rs",
    );
    let paint_test_frame = read_repo_text(
        manifest_root,
        "zircon_editor/src/ui/retained_host/host_contract/paint_workbench/test_frame.rs",
    );
    let table_cells = read_repo_text(
        manifest_root,
        "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows/cells/text.rs",
    );
    let table_cell_tests = read_repo_text(
        manifest_root,
        "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows_tests/cells.rs",
    );
    let extension_feedback = read_repo_text(
        manifest_root,
        "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs",
    );
    let render_asset_specs = read_repo_text(
        manifest_root,
        "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/render_asset_vfx.rs",
    );
    let ui_specs = read_repo_text(
        manifest_root,
        "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/ui_diagnostics.rs",
    );
    let preview_actions = read_repo_text(
        manifest_root,
        "zircon_editor/src/ui/retained_host/workbench_preview_actions/extensions.rs",
    );
    let render_asset_bindings = read_repo_text(
        manifest_root,
        "zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/render_asset_vfx.rs",
    );
    let ui_bindings = read_repo_text(
        manifest_root,
        "zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/ui_diagnostics.rs",
    );
    let particle_asset = read_repo_text(
        manifest_root,
        "zircon_editor/assets/ui/editor/components/workbench/modules/extensions/rendering/workbench_extension_particle_library_workspace.zui",
    );
    let icon_asset = read_repo_text(
        manifest_root,
        "zircon_editor/assets/ui/editor/components/workbench/modules/extensions/ui/workbench_extension_icon_library_workspace.zui",
    );
    let hard_cutover_doc = read_repo_text(
        manifest_root,
        "docs/engine-architecture/hard-cutover-migration-smells-m1.md",
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
    let status_rows = read_repo_text(
        manifest_root,
        "zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
    );
    let expected_status = read_repo_text(
        manifest_root,
        "zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/naming_boundary.rs",
    );
    let expected_date = read_repo_text(
        manifest_root,
        "zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/naming_boundary.rs",
    );

    assert!(
        !repo_root
            .join(
                "zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/legacy.rs"
            )
            .exists(),
        "retired Workbench renderer legacy.rs module should not exist"
    );
    assert_contains_all(
        "Workbench renderer host-window module",
        &renderer_mod,
        &[
            "mod host_window;",
            "use host_window::{",
            "draw_host_workbench_window",
            "draw_host_workbench_window_profiled",
        ],
    );
    assert_contains_all(
        "Workbench host-window renderer",
        &host_window,
        &[
            "fn draw_host_workbench_window(",
            "fn draw_host_workbench_window_profiled(",
            "resolve_root_frames",
            "draw_root_skeleton",
            "draw_host_scene",
        ],
    );
    assert_contains_all(
        "Workbench host-window callers",
        &format!("{paint_commands}\n{paint_test_frame}"),
        &[
            "workbench::draw_host_workbench_window(",
            "workbench::draw_host_workbench_window_profiled(",
        ],
    );
    assert_contains_all(
        "Workbench table archived text parser",
        &format!("{table_cells}\n{table_cell_tests}"),
        &[
            "split_archived_table_text",
            "table_cells_prefer_declared_options_over_archived_text",
            "archived_table_text_keeps_size_and_modified_units_together",
        ],
    );
    assert_contains_all(
        "Workbench extension archived fixtures",
        &format!(
            "{extension_feedback}\n{render_asset_specs}\n{ui_specs}\n{preview_actions}\n{render_asset_bindings}\n{ui_bindings}\n{particle_asset}\n{icon_asset}"
        ),
        &[
            "workbench.extension.particle_library.archived_row.select",
            "WorkbenchExtensionParticleLibraryArchivedRow",
            "Selected Archived Smoke   Archived   Warning",
            "workbench.extension.icon_library.archived_table_row.select",
            "WorkbenchExtensionIconLibraryArchivedTableRow",
            "Selected icon-archive   Archived   Warning",
        ],
    );
    for retired in [
        "mod legacy;",
        "use legacy::",
        "draw_legacy_workbench_window",
        "split_legacy_table_text",
        "legacy_table_row",
        "LegacyTableRow",
        "deprecated_row",
        "DeprecatedRow",
        "Legacy Smoke",
        "icon-old",
    ] {
        for (label, source) in [
            ("renderer mod", renderer_mod.as_str()),
            ("host window", host_window.as_str()),
            ("paint commands", paint_commands.as_str()),
            ("paint test frame", paint_test_frame.as_str()),
            ("table cells", table_cells.as_str()),
            ("table cell tests", table_cell_tests.as_str()),
            ("extension feedback", extension_feedback.as_str()),
            ("render asset specs", render_asset_specs.as_str()),
            ("ui specs", ui_specs.as_str()),
            ("preview actions", preview_actions.as_str()),
            ("render asset bindings", render_asset_bindings.as_str()),
            ("ui bindings", ui_bindings.as_str()),
            ("particle asset", particle_asset.as_str()),
            ("icon asset", icon_asset.as_str()),
        ] {
            assert!(
                !source.contains(retired),
                "{label} should not retain retired Workbench fixture wording {retired}"
            );
        }
    }
    assert_contains_all(
        "hard-cutover migration smells document",
        &hard_cutover_doc,
        &[
            "`legacy-editor-ui-fixture-debt` was cleared",
            "draw_host_workbench_window",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        (
            "hard-cutover migration smells doc",
            hard_cutover_doc.as_str(),
        ),
        ("status row data", status_rows.as_str()),
        ("expected status map", expected_status.as_str()),
        ("expected date map", expected_date.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                SLICE,
                STATUS,
                "zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/host_window.rs",
                "draw_host_workbench_window",
                GUARD,
            ],
        );
    }
}
