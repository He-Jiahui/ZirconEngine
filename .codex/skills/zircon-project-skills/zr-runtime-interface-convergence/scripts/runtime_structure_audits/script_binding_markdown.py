from __future__ import annotations


def render_script_binding_boundary_markdown(boundary: dict[str, object]) -> list[str]:
    source_files = boundary["source_files"]
    test_files = boundary["test_files"]
    lines = [
        "## Runtime 13 Script Binding Boundary",
        "- audited script binding source files "
        f"({len(source_files)}/{boundary['expected_source_file_count']}): "
        f"{len(source_files)} files",
        "- audited Runtime 13 guard/test files "
        f"({len(test_files)}/{boundary['expected_test_file_count']}): "
        f"{', '.join(file['path'] for file in test_files) if test_files else 'none'}",
        "- fixed host ledger: "
        f"{boundary['fixed_host_module_count']} modules, "
        f"{boundary['fixed_host_function_count']} functions, "
        f"{boundary['type_descriptor_count']} type descriptors",
        "- callback counts: "
        f"builtin={boundary['builtin_callback_count']}/"
        f"{boundary['expected_builtin_callback_count']}, "
        f"gameplay={boundary['gameplay_callback_count']}/"
        f"{boundary['expected_gameplay_callback_count']}, "
        f"macro={boundary['macro_host_function_count']}/"
        f"{boundary['expected_macro_host_function_count']}",
        "- host capability anchors: "
        f"{boundary['host_capability_count'] - len(boundary['missing_capabilities'])}/"
        f"{boundary['expected_host_capability_count']}",
        "- Runtime 13 guard anchors: "
        f"{boundary['guard_anchor_count'] - len(boundary['missing_runtime_13_guards'])}/"
        f"{boundary['guard_anchor_count']}",
        "- native ECS ABI references in script source: "
        f"{len(boundary['native_ecs_abi_references'])}",
        "- mirror-doc aggregate guard: "
        f"{'present' if boundary['mirror_docs_guard_present'] else 'missing'}",
    ]

    if boundary["missing_source_files"]:
        lines.append(
            "- missing Runtime 13 source files: "
            f"{', '.join(boundary['missing_source_files'])}"
        )
    if boundary["missing_test_files"]:
        lines.append(
            "- missing Runtime 13 test files: "
            f"{', '.join(boundary['missing_test_files'])}"
        )
    if boundary["missing_fixed_modules"]:
        lines.append(
            "- missing fixed host module doc anchors: "
            f"{', '.join(boundary['missing_fixed_modules'])}"
        )
    if boundary["missing_capabilities"]:
        lines.append(
            "- missing host capability doc anchors: "
            f"{', '.join(boundary['missing_capabilities'])}"
        )
    if boundary["missing_ledger_doc_anchors"]:
        lines.append(
            "- missing Runtime 13 ledger/doc anchors: "
            f"{', '.join(boundary['missing_ledger_doc_anchors'])}"
        )
    if boundary["missing_runtime_13_guards"]:
        lines.append(
            "- missing Runtime 13 guard anchors: "
            f"{', '.join(boundary['missing_runtime_13_guards'])}"
        )
    if boundary["missing_bridge_anchors"]:
        lines.append(
            "- missing bridge dynamic-module anchors: "
            f"{', '.join(boundary['missing_bridge_anchors'])}"
        )
    if boundary["missing_gameplay_facade_anchors"]:
        lines.append(
            "- missing gameplay facade anchors: "
            f"{', '.join(boundary['missing_gameplay_facade_anchors'])}"
        )
    if boundary["missing_cargo_gate_anchors"]:
        lines.append(
            "- missing pending Cargo gate anchors: "
            f"{', '.join(boundary['missing_cargo_gate_anchors'])}"
        )
    if boundary["native_ecs_abi_references"]:
        lines.append("- native ECS ABI references:")
        for reference in boundary["native_ecs_abi_references"]:
            lines.append(
                f"  - `{reference['path']}:{reference['line']}` "
                f"{reference['symbol']}: {reference['snippet']}"
            )

    oversized_test_files = boundary["oversized_test_files"]
    if oversized_test_files:
        lines.append("- oversized Runtime 13 guard/test files:")
        for file in oversized_test_files:
            lines.append(f"  - `{file['path']}` ({file['lines']} lines)")
    else:
        lines.append(
            "- oversized Runtime 13 guard/test files: none "
            f"(ledger>{boundary['max_script_ledger_test_lines']}, "
            f"gameplay_tests>{boundary['max_gameplay_test_lines']})"
        )

    for risk in boundary["risks"]:
        lines.append(f"- risk: {risk}")

    return lines
