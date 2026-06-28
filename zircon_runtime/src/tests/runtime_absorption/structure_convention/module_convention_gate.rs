use super::{assert_contains_all, repo_path};

const STATUS: &str =
    "runtime_15_module_convention_gate_output_contract_static_passed_cargo_deferred";
const SLICE: &str = "Runtime 15 M3 module convention gate output contract";
const GUARD: &str =
    "runtime_15_module_convention_gate_output_contract_is_backed_by_structure_audit";
const NON_RENDER_STATUS: &str =
    "runtime_15_module_convention_non_render_debt_guard_static_passed_cargo_deferred";
const NON_RENDER_SLICE: &str = "Runtime 15 M3 module convention non-render debt guard";
const NON_RENDER_GUARD: &str = "runtime_15_module_convention_gate_reports_non_render_debt_boundary";
const RENDER_HANDOFF_STATUS: &str =
    "runtime_15_render_scoped_migration_debt_handoff_static_passed_cargo_deferred";
const RENDER_HANDOFF_SLICE: &str = "Runtime 15 M3 render-scoped migration debt handoff gate";
const RENDER_HANDOFF_GUARD: &str =
    "runtime_15_render_scoped_migration_debt_handoff_is_status_locked";
const ALLOWED_HYPER_POLICY_RISK_STATUS: &str =
    "runtime_15_hard_cutover_allowed_hyper_policy_risk_cleanup_static_passed_cargo_deferred";
const ALLOWED_HYPER_POLICY_RISK_SLICE: &str =
    "Runtime 15 M3 hard-cutover allowed Hyper policy risk cleanup";
const ALLOWED_HYPER_POLICY_RISK_GUARD: &str =
    "runtime_15_hard_cutover_allowed_hyper_policy_does_not_report_risk";
const AUDIT_CLEAR_STATUS: &str =
    "runtime_15_module_convention_gate_audit_clear_status_mirror_core_min_cargo_passed_full_sweep_pending";
const AUDIT_CLEAR_SLICE: &str = "Runtime 15 M3 module convention gate audit-clear status mirror";
const AUDIT_CLEAR_GUARD: &str = "runtime_15_module_convention_gate_audit_clear_is_status_locked";
const AUDIT_SCRIPT_FAMILY_STATUS: &str =
    "runtime_15_module_convention_audit_script_family_naming_core_min_cargo_passed_full_sweep_pending";
const AUDIT_SCRIPT_FAMILY_SLICE: &str =
    "Runtime 15 M3 module convention audit script family naming cleanup";
const AUDIT_SCRIPT_FAMILY_GUARD: &str =
    "runtime_15_module_convention_audit_script_family_uses_gate_names";
const AUDIT_ROOT: &str =
    ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts";

fn read_repo(relative: &str) -> String {
    std::fs::read_to_string(repo_path(relative))
        .unwrap_or_else(|error| panic!("failed to read repository file `{relative}`: {error}"))
}

fn assert_not_contains(label: &str, source: &str, forbidden: &[&str]) {
    let present: Vec<_> = forbidden
        .iter()
        .copied()
        .filter(|anchor| source.contains(anchor))
        .collect();
    assert!(
        present.is_empty(),
        "{label} contains forbidden stale anchors: {present:?}"
    );
}

#[test]
fn runtime_15_module_convention_gate_output_contract_is_backed_by_structure_audit() {
    let audit_script = read_repo(&format!("{AUDIT_ROOT}/audit_runtime_structure.py"));
    assert_contains_all(
        "audit runtime structure module convention gate wiring",
        &audit_script,
        &[
            "from runtime_structure_audits.module_convention_gate import module_convention_gate",
            "from runtime_structure_audits.module_convention_gate_markdown import",
            "module_gate = module_convention_gate(",
            "\"module_convention_gate\": module_gate",
            "render_module_convention_gate_markdown",
        ],
    );

    let gate_source = read_repo(&format!(
        "{AUDIT_ROOT}/runtime_structure_audits/module_convention_gate.py"
    ));
    assert_contains_all(
        "module convention gate output fields",
        &gate_source,
        &[
            "def module_convention_gate(",
            "module_classification: Mapping[str, Mapping[str, object]]",
            "large_file_ownership_gate: Mapping[str, object]",
            "runtime_naming_boundary: Mapping[str, object]",
            "hard_cutover_migration_smells: Mapping[str, object]",
            "non_network_server_references: Mapping[str, object]",
            "\"m1_gate_status\"",
            "\"classification_counts\"",
            "\"migration_debt_count\"",
            "\"exempt\"",
            "\"source_gate_statuses\"",
            "\"violation_fields\"",
        ],
    );

    let markdown_source = read_repo(&format!(
        "{AUDIT_ROOT}/runtime_structure_audits/module_convention_gate_markdown.py"
    ));
    assert_contains_all(
        "module convention gate markdown mirrors audit fields",
        &markdown_source,
        &[
            "## Module Convention Gate",
            "M1 gate status",
            "migration debt count",
            "exempt entries",
            "source gate statuses",
            "violation fields",
            "classification counts",
        ],
    );

    for doc in [
        "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
        "docs/plans/zircon_runtime/runtime/index.md",
        "docs/plans/engine-code-structure-convention.md",
        "docs/plans/engine-code-review-findings-2026-06.md",
        "docs/zircon_runtime/structure/module-convention.md",
    ] {
        let source = read_repo(doc);
        assert_contains_all(doc, &source, &[SLICE, STATUS, GUARD]);
    }
}

#[test]
fn runtime_15_module_convention_gate_reports_non_render_debt_boundary() {
    let gate_source = read_repo(&format!(
        "{AUDIT_ROOT}/runtime_structure_audits/module_convention_gate.py"
    ));
    assert_contains_all(
        "module convention gate non-render debt boundary fields",
        &gate_source,
        &[
            "RENDER_SCOPED_MIGRATION_DEBT_PREFIXES",
            "\"runtime-naming:legacy: legacy-runtime-graphics-debt:\"",
            "\"hard-cutover: legacy-runtime-graphics-debt:\"",
            "def _split_render_scoped_migration_debt(",
            "\"render_scoped_migration_debt\"",
            "\"render_scoped_migration_debt_count\"",
            "\"non_render_migration_debt\"",
            "\"non_render_migration_debt_count\"",
        ],
    );

    let markdown_source = read_repo(&format!(
        "{AUDIT_ROOT}/runtime_structure_audits/module_convention_gate_markdown.py"
    ));
    assert_contains_all(
        "module convention gate markdown exposes non-render debt boundary",
        &markdown_source,
        &[
            "render-scoped migration debt count",
            "non-render migration debt count",
            "non-render migration debt",
        ],
    );

    for doc in [
        "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
        "docs/plans/zircon_runtime/runtime/index.md",
        "docs/plans/engine-code-structure-convention.md",
        "docs/plans/engine-code-review-findings-2026-06.md",
        "docs/zircon_runtime/structure/module-convention.md",
    ] {
        let source = read_repo(doc);
        assert_contains_all(
            doc,
            &source,
            &[NON_RENDER_SLICE, NON_RENDER_STATUS, NON_RENDER_GUARD],
        );
    }
}

#[test]
fn runtime_15_render_scoped_migration_debt_handoff_is_status_locked() {
    let gate_source = read_repo(&format!(
        "{AUDIT_ROOT}/runtime_structure_audits/module_convention_gate.py"
    ));
    assert_contains_all(
        "module convention gate render-scoped migration debt handoff fields",
        &gate_source,
        &[
            "RENDER_SCOPED_MIGRATION_DEBT_PREFIXES",
            "\"runtime-naming:legacy: legacy-runtime-graphics-debt:\"",
            "\"hard-cutover: legacy-runtime-graphics-debt:\"",
            "\"render_scoped_migration_debt_count\"",
            "\"non_render_migration_debt_count\"",
        ],
    );

    let markdown_source = read_repo(&format!(
        "{AUDIT_ROOT}/runtime_structure_audits/module_convention_gate_markdown.py"
    ));
    assert_contains_all(
        "module convention gate markdown keeps render handoff visible",
        &markdown_source,
        &[
            "render-scoped migration debt count",
            "non-render migration debt count",
            "non-render migration debt",
        ],
    );

    for doc in [
        "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
        "docs/plans/zircon_runtime/runtime/index.md",
        "docs/plans/engine-code-structure-convention.md",
        "docs/plans/engine-code-review-findings-2026-06.md",
        "docs/zircon_runtime/structure/module-convention.md",
        ".codex/sessions/20260612-0847-runtime-architecture-implementation.md",
    ] {
        let source = read_repo(doc);
        assert_contains_all(
            doc,
            &source,
            &[
                RENDER_HANDOFF_SLICE,
                RENDER_HANDOFF_STATUS,
                RENDER_HANDOFF_GUARD,
                "render-scoped migration debt 0",
                "non-render migration debt 0",
                "module_convention_gate classified-and-clear",
            ],
        );
    }
}

#[test]
fn runtime_15_hard_cutover_allowed_hyper_policy_does_not_report_risk() {
    let hard_cutover_source = read_repo(&format!(
        "{AUDIT_ROOT}/runtime_structure_audits/hard_cutover_migration_smells.py"
    ));
    assert_contains_all(
        "hard-cutover allowed Hyper policy risk cleanup",
        &hard_cutover_source,
        &[
            "HARD_CUTOVER_ALLOWED_CLASSIFICATIONS",
            "\"external-hyper-http1-client-policy\"",
            "non_allowed_legacy_count",
            "decision.classification not in HARD_CUTOVER_ALLOWED_CLASSIFICATIONS",
            "\"risks\": risks",
        ],
    );

    let module_gate_source = read_repo(&format!(
        "{AUDIT_ROOT}/runtime_structure_audits/module_convention_gate.py"
    ));
    assert_contains_all(
        "module convention risk aggregation keeps source risks visible",
        &module_gate_source,
        &[
            "risks.extend(str(risk) for risk in source.get(\"risks\", []))",
            "\"risks\": risks",
            "\"risk_count\": len(risks)",
        ],
    );

    for doc in [
        "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
        "docs/plans/zircon_runtime/runtime/index.md",
        "docs/plans/engine-code-structure-convention.md",
        "docs/plans/engine-code-review-findings-2026-06.md",
        "docs/zircon_runtime/structure/module-convention.md",
        ".codex/sessions/20260612-0847-runtime-architecture-implementation.md",
    ] {
        let source = read_repo(doc);
        assert_contains_all(
            doc,
            &source,
            &[
                ALLOWED_HYPER_POLICY_RISK_SLICE,
                ALLOWED_HYPER_POLICY_RISK_STATUS,
                ALLOWED_HYPER_POLICY_RISK_GUARD,
                "external-hyper-http1-client-policy",
                "risk_count=0",
                "risks=[]",
            ],
        );
    }
}

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

    for doc in [
        "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
        "docs/plans/zircon_runtime/runtime/index.md",
        "docs/plans/engine-code-structure-convention.md",
        "docs/plans/engine-code-review-findings-2026-06.md",
        "docs/zircon_runtime/structure/module-convention.md",
        ".codex/sessions/20260612-0847-runtime-architecture-implementation.md",
    ] {
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
fn runtime_15_module_convention_audit_script_family_uses_gate_names() {
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    assert_contains_all(
        "engine code structure convention audit script family names",
        &structure_convention,
        &[
            "runtime_structure_audits/`（`module_convention_gate.py` + `module_convention_gate_markdown.py`",
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

    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    assert_contains_all(
        "Runtime 15 module convention script inventory uses current gate files",
        &runtime_15_plan,
        &[
            ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_convention_gate.py",
            ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_convention_gate_markdown.py",
            "module_convention_gate.py` + `module_convention_gate_markdown.py",
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

    for doc in [
        "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
        "docs/plans/zircon_runtime/runtime/index.md",
        "docs/plans/engine-code-structure-convention.md",
        "docs/plans/engine-code-review-findings-2026-06.md",
        "docs/zircon_runtime/structure/module-convention.md",
        ".codex/sessions/20260612-0847-runtime-architecture-implementation.md",
    ] {
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
