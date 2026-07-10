from __future__ import annotations

from runtime_structure_audits.runtime_plan_status_output_anchors import (
    CARGO_ATTEMPT_STATUS_ANCHORS,
)

def render_runtime_plan_status_boundary_markdown(
    boundary: dict[str, object],
) -> list[str]:
    lines = [
        "## Runtime 05 Plan Status Boundary",
        "- audited plan-status support files "
        f"({len(boundary['support_files'])}/"
        f"{boundary['expected_support_file_count']}): "
        f"{len(boundary['support_files'])} files",
        "- plan-status boundary script lines: "
        f"{boundary['plan_status_boundary_line_count']}/"
        f"{boundary['max_plan_status_boundary_lines']}",
        "- status-output anchor module: "
        f"{'present' if boundary['status_output_anchor_module_present'] else 'missing'}",
        "- status-output anchor module lines: "
        f"{boundary['status_output_anchor_module_line_count']}/"
        f"{boundary['max_status_output_anchor_module_lines']}",
        "- runtime subplans: "
        f"{boundary['subplan_count']}/{boundary['expected_subplan_count']}",
        "- runtime index subplan rows: "
        f"{boundary['subplan_index_row_count']}/"
        f"{boundary['expected_subplan_count']}",
        "- runtime index problem rows: "
        f"{boundary['problem_row_count']}/"
        f"{boundary['expected_problem_row_count']}",
        "- runtime index known-backlog rows: "
        f"{boundary['backlog_row_count']}/"
        f"{boundary['expected_backlog_row_count']}",
        "- Runtime 05 closeout status: " f"{boundary['runtime_05_status']}",
        "- plan-status core guard anchors: "
        f"{boundary['core_guard_count'] - len(boundary['missing_core_guard_anchors'])}/"
        f"{boundary['core_guard_count']}",
        "- pending Cargo gate anchors: "
        f"{boundary['pending_gate_count'] - len(boundary['missing_pending_gate_anchors'])}/"
        f"{boundary['pending_gate_count']}",
        "- plan-status doc anchors: "
        f"{boundary['doc_anchor_count'] - len(boundary['missing_doc_anchors'])}/"
        f"{boundary['doc_anchor_count']}",
        "- status-output table guard anchors: "
        f"{boundary['status_output_table_guard_count'] - len(boundary['missing_status_output_table_guard_anchors'])}/"
        f"{boundary['status_output_table_guard_count']}",
        "- status-output table guard: "
        f"{'present' if boundary['status_output_table_guard_present'] else 'missing'}",
        "- Runtime 03 module-doc status anchors: "
        f"{boundary['runtime_03_module_doc_status_index_anchor_count'] - len(boundary['missing_runtime_03_module_doc_status_index_anchors'])}/"
        f"{boundary['runtime_03_module_doc_status_index_anchor_count']} in runtime index, "
        f"{boundary['runtime_03_module_doc_status_guard_anchor_count'] - len(boundary['missing_runtime_03_module_doc_status_guard_anchors'])}/"
        f"{boundary['runtime_03_module_doc_status_guard_anchor_count']} in status-output guards, "
        f"{boundary['runtime_03_module_doc_status_doc_anchor_count'] - len(boundary['missing_runtime_03_module_doc_status_doc_anchors'])}/"
        f"{boundary['runtime_03_module_doc_status_doc_anchor_count']} in mirror docs",
        "- Runtime 03 module-doc status guard: "
        f"{'present' if boundary['runtime_03_module_doc_status_guard_present'] else 'missing'}",
        "- Runtime 07 scene status anchors: "
        f"{boundary['runtime_07_scene_status_index_anchor_count'] - len(boundary['missing_runtime_07_scene_status_index_anchors'])}/"
        f"{boundary['runtime_07_scene_status_index_anchor_count']} in runtime index, "
        f"{boundary['runtime_07_scene_status_guard_anchor_count'] - len(boundary['missing_runtime_07_scene_status_guard_anchors'])}/"
        f"{boundary['runtime_07_scene_status_guard_anchor_count']} in status-output guards, "
        f"{boundary['runtime_07_scene_status_doc_anchor_count'] - len(boundary['missing_runtime_07_scene_status_doc_anchors'])}/"
        f"{boundary['runtime_07_scene_status_doc_anchor_count']} in mirror docs",
        "- Runtime 07 scene status guard: "
        f"{'present' if boundary['runtime_07_scene_status_guard_present'] else 'missing'}",
        "- Runtime 07 owner-budget status anchors: "
        f"{boundary['runtime_07_owner_budget_status_index_anchor_count'] - len(boundary['missing_runtime_07_owner_budget_status_index_anchors'])}/"
        f"{boundary['runtime_07_owner_budget_status_index_anchor_count']} in runtime index, "
        f"{boundary['runtime_07_owner_budget_status_guard_anchor_count'] - len(boundary['missing_runtime_07_owner_budget_status_guard_anchors'])}/"
        f"{boundary['runtime_07_owner_budget_status_guard_anchor_count']} in status-output guards, "
        f"{boundary['runtime_07_owner_budget_status_doc_anchor_count'] - len(boundary['missing_runtime_07_owner_budget_status_doc_anchors'])}/"
        f"{boundary['runtime_07_owner_budget_status_doc_anchor_count']} in mirror docs",
        "- Runtime 07 owner-budget status guard: "
        f"{'present' if boundary['runtime_07_owner_budget_status_guard_present'] else 'missing'}",
        "- Runtime 02 generated status anchors: "
        f"{boundary['runtime_02_generated_status_index_anchor_count'] - len(boundary['missing_runtime_02_generated_status_index_anchors'])}/"
        f"{boundary['runtime_02_generated_status_index_anchor_count']} in runtime index, "
        f"{boundary['runtime_02_generated_status_guard_anchor_count'] - len(boundary['missing_runtime_02_generated_status_guard_anchors'])}/"
        f"{boundary['runtime_02_generated_status_guard_anchor_count']} in status-output guards, "
        f"{boundary['runtime_02_generated_status_doc_anchor_count'] - len(boundary['missing_runtime_02_generated_status_doc_anchors'])}/"
        f"{boundary['runtime_02_generated_status_doc_anchor_count']} in mirror docs",
        "- Runtime 02 generated status guard: "
        f"{'present' if boundary['runtime_02_generated_status_guard_present'] else 'missing'}",
        "- Cargo attempt status anchors: "
        f"{boundary['cargo_attempt_status_anchor_count'] - len(boundary['missing_cargo_attempt_status_index_anchors'])}/"
        f"{boundary['cargo_attempt_status_anchor_count']} in runtime index, "
        f"{boundary['cargo_attempt_status_anchor_count'] - len(boundary['missing_cargo_attempt_status_runtime_14_anchors'])}/"
        f"{boundary['cargo_attempt_status_anchor_count']} in Runtime 14 plan, "
        f"{len(CARGO_ATTEMPT_STATUS_ANCHORS) - len(boundary['missing_cargo_attempt_status_guard_anchors'])}/"
        f"{len(CARGO_ATTEMPT_STATUS_ANCHORS)} in status-output guards",
        "- Cargo attempt status guard: "
        f"{'present' if boundary['cargo_attempt_status_guard_present'] else 'missing'}",
    ]

    if boundary["status_counts"]:
        status_counts = boundary["status_counts"]
        lines.append(
            "- runtime subplan status counts: "
            + ", ".join(
                f"{status}={count}" for status, count in sorted(status_counts.items())
            )
        )
    if boundary["missing_support_files"]:
        lines.append(
            "- missing plan-status support files: "
            f"{', '.join(boundary['missing_support_files'])}"
        )
    if boundary["missing_backlog_gaps"]:
        lines.append(
            "- missing known backlog gaps: "
            f"{', '.join(boundary['missing_backlog_gaps'])}"
        )
    if boundary["last_refined_violations"]:
        lines.append("- last_refined violations:")
        for violation in boundary["last_refined_violations"]:
            lines.append(
                "  - "
                f"`{violation['path']}` last_refined={violation['last_refined']} "
                f"max_date={violation['max_date']}"
            )
    if boundary["status_table_gaps"]:
        lines.append(
            "- status/evidence table gaps: "
            f"{', '.join(boundary['status_table_gaps'])}"
        )
    if boundary["in_progress_without_gate"]:
        lines.append(
            "- in-progress rows without remaining gate marker: "
            f"{', '.join(boundary['in_progress_without_gate'])}"
        )
    if boundary["missing_core_guard_anchors"]:
        lines.append(
            "- missing core guard anchors: "
            f"{', '.join(boundary['missing_core_guard_anchors'])}"
        )
    if boundary["missing_pending_gate_anchors"]:
        lines.append(
            "- missing pending Cargo gate anchors: "
            f"{', '.join(boundary['missing_pending_gate_anchors'])}"
        )
    if boundary["missing_doc_anchors"]:
        lines.append(
            "- missing plan-status doc anchors: "
            f"{', '.join(boundary['missing_doc_anchors'])}"
        )
    if boundary["missing_status_output_table_guard_anchors"]:
        lines.append(
            "- missing status-output table guard anchors: "
            f"{', '.join(boundary['missing_status_output_table_guard_anchors'])}"
        )
    if boundary["missing_runtime_03_module_doc_status_index_anchors"]:
        lines.append(
            "- missing Runtime 03 module-doc status index anchors: "
            f"{', '.join(boundary['missing_runtime_03_module_doc_status_index_anchors'])}"
        )
    if boundary["missing_runtime_03_module_doc_status_guard_anchors"]:
        lines.append(
            "- missing Runtime 03 module-doc status guard anchors: "
            f"{', '.join(boundary['missing_runtime_03_module_doc_status_guard_anchors'])}"
        )
    if boundary["missing_runtime_03_module_doc_status_doc_anchors"]:
        lines.append(
            "- missing Runtime 03 module-doc status doc anchors: "
            f"{', '.join(boundary['missing_runtime_03_module_doc_status_doc_anchors'])}"
        )
    if boundary["missing_runtime_07_scene_status_index_anchors"]:
        lines.append(
            "- missing Runtime 07 scene status index anchors: "
            f"{', '.join(boundary['missing_runtime_07_scene_status_index_anchors'])}"
        )
    if boundary["missing_runtime_07_scene_status_guard_anchors"]:
        lines.append(
            "- missing Runtime 07 scene status guard anchors: "
            f"{', '.join(boundary['missing_runtime_07_scene_status_guard_anchors'])}"
        )
    if boundary["missing_runtime_07_scene_status_doc_anchors"]:
        lines.append(
            "- missing Runtime 07 scene status doc anchors: "
            f"{', '.join(boundary['missing_runtime_07_scene_status_doc_anchors'])}"
        )
    if boundary["missing_runtime_07_owner_budget_status_index_anchors"]:
        lines.append(
            "- missing Runtime 07 owner-budget status index anchors: "
            f"{', '.join(boundary['missing_runtime_07_owner_budget_status_index_anchors'])}"
        )
    if boundary["missing_runtime_07_owner_budget_status_guard_anchors"]:
        lines.append(
            "- missing Runtime 07 owner-budget status guard anchors: "
            f"{', '.join(boundary['missing_runtime_07_owner_budget_status_guard_anchors'])}"
        )
    if boundary["missing_runtime_07_owner_budget_status_doc_anchors"]:
        lines.append(
            "- missing Runtime 07 owner-budget status doc anchors: "
            f"{', '.join(boundary['missing_runtime_07_owner_budget_status_doc_anchors'])}"
        )
    if boundary["missing_runtime_02_generated_status_index_anchors"]:
        lines.append(
            "- missing Runtime 02 generated status index anchors: "
            f"{', '.join(boundary['missing_runtime_02_generated_status_index_anchors'])}"
        )
    if boundary["missing_runtime_02_generated_status_guard_anchors"]:
        lines.append(
            "- missing Runtime 02 generated status guard anchors: "
            f"{', '.join(boundary['missing_runtime_02_generated_status_guard_anchors'])}"
        )
    if boundary["missing_runtime_02_generated_status_doc_anchors"]:
        lines.append(
            "- missing Runtime 02 generated status doc anchors: "
            f"{', '.join(boundary['missing_runtime_02_generated_status_doc_anchors'])}"
        )
    if boundary["missing_cargo_attempt_status_index_anchors"]:
        lines.append(
            "- missing Cargo attempt status index anchors: "
            f"{', '.join(boundary['missing_cargo_attempt_status_index_anchors'])}"
        )
    if boundary["missing_cargo_attempt_status_runtime_14_anchors"]:
        lines.append(
            "- missing Cargo attempt status Runtime 14 anchors: "
            f"{', '.join(boundary['missing_cargo_attempt_status_runtime_14_anchors'])}"
        )
    if boundary["missing_cargo_attempt_status_guard_anchors"]:
        lines.append(
            "- missing Cargo attempt status guard anchors: "
            f"{', '.join(boundary['missing_cargo_attempt_status_guard_anchors'])}"
        )
    if boundary["runtime_05_closeout_anchors"]:
        lines.append(
            "- missing Runtime 05 closeout anchors: "
            f"{', '.join(boundary['runtime_05_closeout_anchors'])}"
        )

    for risk in boundary["risks"]:
        lines.append(f"- risk: {risk}")

    return lines
