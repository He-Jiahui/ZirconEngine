use super::support::assert_contains_all;

pub(super) fn assert_capability_audit_surfaces_are_wired() {
    let capability_audit =
        include_str!("../../../../../../../tools/plugin_structure_audits/capability.py");
    let audit_cli = include_str!("../../../../../../../tools/audit_plugin_structure.py");
    let catalog =
        include_str!("../../../../../../../zircon_plugins/first_party_runtime_catalog/src/lib.rs");

    assert_contains_all(
        "plugin capability audit enforces capability.rs as the runtime root single source",
        capability_audit,
        &[
            "FIRST_PARTY_RUNTIME_CAPABILITY_ROOTS",
            "audited_runtime_root_count",
            "capability_source_mismatches",
            "missing_capability_owner_files",
            "missing_runtime_capability_exports",
            "root_capability_mismatches",
            "module_capability_mismatches",
            "lib_capability_literal_sites",
            "m4_runtime_capability_gate_status",
            "runtime-capability-single-source-clean",
            "sdk_builder_mirror_violations",
            "m4_t2_builder_mirror_gate_status",
            "sdk-builder-mirror-clean",
            "PluginFeatureBundleBuilder",
        ],
    );
    assert_contains_all(
        "plugin structure audit CLI wires D1 capability audit into the JSON result",
        audit_cli,
        &[
            "from plugin_structure_audits.capability import audit_plugin_capability_conformance",
            "capability_conformance = audit_plugin_capability_conformance(root).to_json()",
            "\"capability_conformance\": capability_conformance",
            "\"capability_source_mismatches\": capability_conformance[",
            "\"m4_runtime_capability_gate_status\": capability_conformance[",
            "\"sdk_builder_mirror_violations\": capability_conformance[",
            "\"m4_t2_builder_mirror_gate_status\": capability_conformance[",
        ],
    );
    assert_contains_all(
        "first-party runtime catalog keeps the D1 capability audit as an executable guard",
        catalog,
        &[
            "plugins_12_capability_single_source_conformance",
            r#"\"audited_runtime_root_count\": 15"#,
            r#"\"capability_source_mismatches\": 0"#,
            r#"\"missing_capability_owner_files\": 0"#,
            r#"\"missing_runtime_capability_exports\": 0"#,
            r#"\"root_capability_mismatches\": 0"#,
            r#"\"module_capability_mismatches\": 0"#,
            r#"\"lib_capability_literal_sites\": 0"#,
            r#"\"m4_runtime_capability_gate_status\""#,
            "runtime-capability-single-source-clean",
            r#"\"sdk_builder_mirror_violations\": 0"#,
            r#"\"m4_t2_builder_mirror_gate_status\""#,
            "sdk-builder-mirror-clean",
        ],
    );
}
