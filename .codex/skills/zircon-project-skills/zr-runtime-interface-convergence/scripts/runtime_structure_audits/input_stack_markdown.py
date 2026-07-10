from __future__ import annotations


def render_input_stack_boundary_markdown(boundary: dict[str, object]) -> list[str]:
    runtime_modules = boundary["runtime_modules"]
    framework_modules = boundary["framework_modules"]
    test_modules = boundary["test_modules"]
    lines = [
        "## Runtime 12 Input Stack Boundary",
        "- input runtime owner modules "
        f"({len(runtime_modules)}/{boundary['expected_runtime_module_count']}): "
        f"{', '.join(module['path'] for module in runtime_modules) if runtime_modules else 'none'}",
        "- framework input contract modules "
        f"({len(framework_modules)}/{boundary['expected_framework_module_count']}): "
        f"{len(framework_modules)} files",
        "- input test owner modules "
        f"({len(test_modules)}/{boundary['expected_test_module_count']}): "
        f"{', '.join(module['path'] for module in test_modules) if test_modules else 'none'}",
        "- public surface anchors: "
        f"{boundary['public_surface_anchor_count'] - len(boundary['missing_public_surface'])}/"
        f"{boundary['public_surface_anchor_count']}",
        "- Runtime 12 guard anchors: "
        f"{boundary['guard_anchor_count'] - len(boundary['missing_runtime_12_guards'])}/"
        f"{boundary['guard_anchor_count']}",
        "- behavior test anchors: "
        f"{boundary['behavior_test_anchor_count'] - len(boundary['missing_behavior_test_anchors'])}/"
        f"{boundary['behavior_test_anchor_count']}",
        "- mirror-doc aggregate guard: "
        f"{'present' if boundary['mirror_docs_guard_present'] else 'missing'}",
    ]

    if boundary["missing_runtime_modules"]:
        lines.append(
            "- missing input runtime modules: "
            f"{', '.join(boundary['missing_runtime_modules'])}"
        )
    if boundary["unexpected_runtime_modules"]:
        lines.append(
            "- unexpected input runtime modules: "
            f"{', '.join(boundary['unexpected_runtime_modules'])}"
        )
    if boundary["missing_framework_modules"]:
        lines.append(
            "- missing framework input modules: "
            f"{', '.join(boundary['missing_framework_modules'])}"
        )
    if boundary["unexpected_framework_modules"]:
        lines.append(
            "- unexpected framework input modules: "
            f"{', '.join(boundary['unexpected_framework_modules'])}"
        )
    if boundary["missing_public_surface"]:
        lines.append(
            "- missing public surface anchors: "
            f"{', '.join(boundary['missing_public_surface'])}"
        )
    if boundary["missing_action_evaluator_anchors"]:
        lines.append(
            "- missing action evaluator anchors: "
            f"{', '.join(boundary['missing_action_evaluator_anchors'])}"
        )
    if boundary["missing_gamepad_abi_anchors"]:
        lines.append(
            "- missing gamepad ABI anchors: "
            f"{', '.join(boundary['missing_gamepad_abi_anchors'])}"
        )
    if boundary["missing_cursor_host_request_anchors"]:
        lines.append(
            "- missing cursor host-request anchors: "
            f"{', '.join(boundary['missing_cursor_host_request_anchors'])}"
        )
    if boundary["missing_doc_anchors"]:
        lines.append(
            "- missing Runtime 12 doc anchors: "
            f"{', '.join(boundary['missing_doc_anchors'])}"
        )
    if boundary["missing_test_anchors"]:
        lines.append(
            "- missing Runtime 12 test anchors: "
            f"{', '.join(boundary['missing_test_anchors'])}"
        )
    if boundary["missing_behavior_test_anchors"]:
        lines.append(
            "- missing Runtime 12 behavior test anchors: "
            f"{', '.join(boundary['missing_behavior_test_anchors'])}"
        )
    if boundary["missing_cargo_gate_anchors"]:
        lines.append(
            "- missing pending Cargo gate anchors: "
            f"{', '.join(boundary['missing_cargo_gate_anchors'])}"
        )

    oversized_modules = boundary["oversized_modules"]
    if oversized_modules:
        lines.append("- oversized Runtime 12 input owner modules:")
        for module in oversized_modules:
            lines.append(f"  - `{module['path']}` ({module['lines']} lines)")
    else:
        lines.append(
            "- oversized Runtime 12 input owner modules: none "
            f"(production>{boundary['max_production_module_lines']}, "
            f"tests>{boundary['max_test_module_lines']})"
        )

    for risk in boundary["risks"]:
        lines.append(f"- risk: {risk}")

    return lines
