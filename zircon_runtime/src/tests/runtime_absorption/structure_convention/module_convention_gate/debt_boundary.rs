use super::super::assert_contains_all;
use super::helpers::{read_repo, AUDIT_ROOT, CORE_DOCS, CORE_DOCS_WITH_SESSION};

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

    for doc in CORE_DOCS {
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

    for doc in CORE_DOCS_WITH_SESSION {
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

    for doc in CORE_DOCS_WITH_SESSION {
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
