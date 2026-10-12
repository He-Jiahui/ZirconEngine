use super::super::assert_contains_all;
use super::helpers::{read_repo, AUDIT_ROOT, CORE_DOCS};

const STATUS: &str =
    "runtime_15_module_convention_gate_output_contract_static_passed_cargo_deferred";
const SLICE: &str = "Runtime 15 M3 module convention gate output contract";
const GUARD: &str =
    "runtime_15_module_convention_gate_output_contract_is_backed_by_structure_audit";

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

    for doc in CORE_DOCS {
        let source = read_repo(doc);
        assert_contains_all(doc, &source, &[SLICE, STATUS, GUARD]);
    }
}
