use super::super::assert_contains_all;
use super::helpers::{assert_not_contains, read_repo, AUDIT_ROOT, CORE_DOCS_WITH_SESSION};

const AUDIT_CLEAR_STATUS: &str =
    "runtime_15_module_convention_gate_audit_clear_status_mirror_core_min_cargo_passed_full_sweep_pending";
const AUDIT_CLEAR_SLICE: &str = "Runtime 15 M3 module convention gate audit-clear status mirror";
const AUDIT_CLEAR_GUARD: &str = "runtime_15_module_convention_gate_audit_clear_is_status_locked";
const ZERO_DEBT_REVALIDATION_STATUS: &str =
    "runtime_15_module_convention_zero_debt_revalidation_static_passed_cargo_timeout_no_result";
const ZERO_DEBT_REVALIDATION_SLICE: &str = "Runtime 15 M3 module convention zero-debt revalidation";
const ZERO_DEBT_REVALIDATION_GUARD: &str =
    "runtime_15_module_convention_zero_debt_revalidation_is_status_locked";
const AUDIT_SCRIPT_FAMILY_STATUS: &str =
    "runtime_15_module_convention_audit_script_family_naming_core_min_cargo_passed_full_sweep_pending";
const AUDIT_SCRIPT_FAMILY_SLICE: &str =
    "Runtime 15 M3 module convention audit script family naming cleanup";
const AUDIT_SCRIPT_FAMILY_GUARD: &str =
    "runtime_15_module_convention_audit_script_family_uses_gate_names";

#[test]
fn runtime_15_module_convention_gate_audit_clear_is_status_locked() {
    let gate_source = read_repo(&format!(
        "{AUDIT_ROOT}/runtime_structure_audits/module_convention_gate.py"
    ));
    assert_contains_all(
        "module convention gate audit-clear fields",
        &gate_source,
        &[
            "\"m1_gate_status\"",
            "\"migration_debt_count\"",
            "\"render_scoped_migration_debt_count\"",
            "\"non_render_migration_debt_count\"",
            "\"risk_count\": len(risks)",
            "\"risks\": risks",
        ],
    );

    for doc in CORE_DOCS_WITH_SESSION {
        let source = read_repo(doc);
        assert_contains_all(
            doc,
            &source,
            &[
                AUDIT_CLEAR_SLICE,
                AUDIT_CLEAR_STATUS,
                AUDIT_CLEAR_GUARD,
                "module_convention_gate audit clear",
                "migration_debt_count=0",
                "risk_count=0",
                "risks=[]",
                "全量 Cargo sweep 仍 pending",
            ],
        );
    }

    for (doc, stale_anchor) in [
        (
            "docs/plans/zircon_runtime/runtime/index.md",
            "完整 `module_convention_gate`、全量 dead-code sweep 与测试组织拆分仍 pending",
        ),
        (
            "docs/zircon_runtime/structure/module-convention.md",
            "完整 `module_convention_boundary.py` 审计计数",
        ),
    ] {
        let source = read_repo(doc);
        assert_not_contains(doc, &source, &[stale_anchor]);
    }
}

#[test]
fn runtime_15_module_convention_zero_debt_revalidation_is_status_locked() {
    let audit_script = read_repo(&format!("{AUDIT_ROOT}/audit_runtime_structure.py"));
    assert_contains_all(
        "runtime audit still builds module convention from gate source inputs",
        &audit_script,
        &[
            "inventory = runtime_inventory(root, args.hotspot_threshold)",
            "large_file_gate = large_file_ownership_gate(",
            "runtime_naming = runtime_naming_boundary_audit(root)",
            "hard_cutover_smells = hard_cutover_migration_smells_audit(root)",
            "non_network_servers = non_network_server_references(",
            "module_gate = module_convention_gate(",
            "\"module_convention_gate\": module_gate",
        ],
    );

    let module_gate_source = read_repo(&format!(
        "{AUDIT_ROOT}/runtime_structure_audits/module_convention_gate.py"
    ));
    assert_contains_all(
        "module convention gate exposes zero-debt audit fields",
        &module_gate_source,
        &[
            "\"m1_gate_status\"",
            "\"migration_debt_count\"",
            "\"render_scoped_migration_debt_count\"",
            "\"non_render_migration_debt_count\"",
            "\"source_gate_statuses\"",
            "\"violation_fields\"",
            "\"risk_count\": len(risks)",
            "\"risks\": risks",
        ],
    );

    let large_file_source = read_repo(&format!(
        "{AUDIT_ROOT}/runtime_structure_audits/large_file_ownership.py"
    ));
    assert_contains_all(
        "large-file ownership gate exposes zero-hotspot fields",
        &large_file_source,
        &[
            "\"hotspot_count\": len(all_hotspots)",
            "\"large_file_migration_debt_count\": len(migration_debt)",
            "\"m1_gate_status\"",
            "\"classified-and-clear\"",
        ],
    );

    for doc in CORE_DOCS_WITH_SESSION {
        let source = read_repo(doc);
        assert_contains_all(
            doc,
            &source,
            &[
                ZERO_DEBT_REVALIDATION_SLICE,
                ZERO_DEBT_REVALIDATION_STATUS,
                ZERO_DEBT_REVALIDATION_GUARD,
                "module_convention_gate classified-and-clear",
                "migration_debt_count=0",
                "render_scoped_migration_debt_count=0",
                "non_render_migration_debt_count=0",
                "risk_count=0",
                "large_file_ownership_gate classified-and-clear",
                "hotspot_count=0",
            ],
        );
    }
}

#[test]
fn runtime_15_module_convention_audit_script_family_uses_gate_names() {
    let structure_convention = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    assert_contains_all(
        "engine code structure convention audit script family names",
        &structure_convention,
        &[
            "module_convention_gate.py",
            "module_convention_gate_markdown.py",
            "audit_runtime_structure.py",
        ],
    );
    assert_not_contains(
        "engine code structure convention audit script family names",
        &structure_convention,
        &[
            "`module_convention_boundary.py` + `_markdown.py`",
            "`module_convention_boundary.py` + `module_convention_markdown.py`",
        ],
    );

    let runtime_15_plan = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
    );
    assert_contains_all(
        "Runtime 15 module convention script inventory uses current gate files",
        &runtime_15_plan,
        &[
            "module_convention_gate.py",
            "module_convention_gate_markdown.py",
            "audit_runtime_structure.py",
        ],
    );
    assert_not_contains(
        "Runtime 15 module convention script inventory uses current gate files",
        &runtime_15_plan,
        &[
            "runtime_structure_audits/module_convention_boundary.py",
            "runtime_structure_audits/module_convention_markdown.py",
            "`module_convention_boundary.py` + `module_convention_markdown.py`",
        ],
    );

    for doc in CORE_DOCS_WITH_SESSION {
        let source = read_repo(doc);
        assert_contains_all(
            doc,
            &source,
            &[
                AUDIT_SCRIPT_FAMILY_SLICE,
                AUDIT_SCRIPT_FAMILY_STATUS,
                AUDIT_SCRIPT_FAMILY_GUARD,
                "module_convention_gate.py",
                "module_convention_gate_markdown.py",
                "module_convention_boundary.py zero hits",
            ],
        );
    }
}
