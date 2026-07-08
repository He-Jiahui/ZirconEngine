type Slice = super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 module convention gate output contract",
        &[
            "runtime_15_module_convention_gate_output_contract_static_passed_cargo_deferred",
            ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py",
            ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_convention_gate.py",
            "module_convention_gate",
            "runtime_15_module_convention_gate_output_contract_is_backed_by_structure_audit",
        ],
    ),
    (
        "Runtime 15 M3 module convention non-render debt guard",
        &[
            "runtime_15_module_convention_non_render_debt_guard_static_passed_cargo_deferred",
            ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_convention_gate.py",
            ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_convention_gate_markdown.py",
            "render_scoped_migration_debt_count",
            "non_render_migration_debt_count",
            "runtime_15_module_convention_gate_reports_non_render_debt_boundary",
        ],
    ),
    (
        "Runtime 15 M3 render-scoped migration debt handoff gate",
        &[
            "runtime_15_render_scoped_migration_debt_handoff_static_passed_cargo_deferred",
            ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_convention_gate.py",
            "render-scoped migration debt 0",
            "non-render migration debt 0",
            "module_convention_gate classified-and-clear",
            "runtime_15_render_scoped_migration_debt_handoff_is_status_locked",
        ],
    ),
    (
        "Runtime 15 M3 hard-cutover allowed Hyper policy risk cleanup",
        &[
            "runtime_15_hard_cutover_allowed_hyper_policy_risk_cleanup_static_passed_cargo_deferred",
            ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/hard_cutover_migration_smells.py",
            "external-hyper-http1-client-policy",
            "risk_count=0",
            "risks=[]",
            "runtime_15_hard_cutover_allowed_hyper_policy_does_not_report_risk",
        ],
    ),
    (
        "Runtime 15 M3 module convention gate audit-clear status mirror",
        &[
            "runtime_15_module_convention_gate_audit_clear_status_mirror_core_min_cargo_passed_full_sweep_pending",
            ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_convention_gate.py",
            "module_convention_gate audit clear",
            "migration_debt_count=0",
            "risk_count=0",
            "risks=[]",
            "runtime_15_module_convention_gate_audit_clear_is_status_locked",
        ],
    ),
    (
        "Runtime 15 M3 module convention zero-debt revalidation",
        &[
            "runtime_15_module_convention_zero_debt_revalidation_static_passed_cargo_timeout_no_result",
            ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py",
            ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_convention_gate.py",
            ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/large_file_ownership.py",
            "module_convention_gate classified-and-clear",
            "migration_debt_count=0",
            "render_scoped_migration_debt_count=0",
            "non_render_migration_debt_count=0",
            "large_file_ownership_gate classified-and-clear",
            "hotspot_count=0",
            "runtime_15_module_convention_zero_debt_revalidation_is_status_locked",
        ],
    ),
    (
        "Runtime 15 M3 module convention audit script family naming cleanup",
        &[
            "runtime_15_module_convention_audit_script_family_naming_core_min_cargo_passed_full_sweep_pending",
            ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_convention_gate.py",
            ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_convention_gate_markdown.py",
            "module_convention_boundary.py zero hits",
            "module_convention_markdown.py zero hits",
            "runtime_15_module_convention_audit_script_family_uses_gate_names",
        ],
    ),
];
