from __future__ import annotations

from pathlib import Path

from runtime_structure_audits.runtime_plan_status_anchor_inventory import (
    BACKLOG_GAPS,
    CORE_GUARD_ANCHORS,
    DOC_ANCHORS,
    PENDING_GATE_ANCHORS,
)
from runtime_structure_audits.runtime_plan_status_output_anchors import (
    CARGO_ATTEMPT_STATUS_ANCHORS,
    CARGO_ATTEMPT_STATUS_EVIDENCE_ANCHORS,
    RUNTIME_02_GENERATED_STATUS_DOC_ANCHORS,
    RUNTIME_02_GENERATED_STATUS_GUARD_ANCHORS,
    RUNTIME_02_GENERATED_STATUS_INDEX_ANCHORS,
    RUNTIME_03_MODULE_DOC_STATUS_DOC_ANCHORS,
    RUNTIME_03_MODULE_DOC_STATUS_GUARD_ANCHORS,
    RUNTIME_03_MODULE_DOC_STATUS_INDEX_ANCHORS,
    RUNTIME_07_OWNER_BUDGET_STATUS_DOC_ANCHORS,
    RUNTIME_07_OWNER_BUDGET_STATUS_GUARD_ANCHORS,
    RUNTIME_07_OWNER_BUDGET_STATUS_INDEX_ANCHORS,
    RUNTIME_07_SCENE_STATUS_DOC_ANCHORS,
    RUNTIME_07_SCENE_STATUS_GUARD_ANCHORS,
    RUNTIME_07_SCENE_STATUS_INDEX_ANCHORS,
    RUNTIME_10_BEHAVIOR_STATUS_DOC_ANCHORS,
    RUNTIME_10_BEHAVIOR_STATUS_GUARD_ANCHORS,
    RUNTIME_10_BEHAVIOR_STATUS_INDEX_ANCHORS,
    STATUS_OUTPUT_TABLE_GUARD_ANCHORS,
)
from runtime_structure_audits.runtime_plan_status_sources import (
    archive_status_rows,
    file_entries,
    file_line_count,
    frontmatter_value,
    max_iso_date,
    missing_snippets,
    read_text,
    runtime_index_backlog_rows,
    runtime_index_problem_rows,
    runtime_index_subplan_rows,
    runtime_numbered_archives,
    runtime_subplans,
)
from runtime_structure_audits.runtime_plan_status_support_inventory import (
    EXPECTED_PLAN_STATUS_SUPPORT_FILE_COUNT,
    PLAN_STATUS_SUPPORT_FILES,
)


EXPECTED_SUBPLAN_COUNT = 15
EXPECTED_PROBLEM_ROW_COUNT = 17
EXPECTED_BACKLOG_ROW_COUNT = 7
EXPECTED_CORE_GUARD_COUNT = 15
EXPECTED_PENDING_GATE_COUNT = 15
EXPECTED_DOC_ANCHOR_COUNT = 11
MAX_PLAN_STATUS_BOUNDARY_LINES = 900
MAX_STATUS_OUTPUT_ANCHOR_MODULE_LINES = 300
PLAN_STATUS_BOUNDARY_SCRIPT = ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_plan_status_boundary.py"
PLAN_STATUS_OUTPUT_ANCHOR_MODULE = ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_plan_status_output_anchors.py"


def runtime_plan_status_boundary_audit(root: Path) -> dict[str, object]:
    boundary_script = root / PLAN_STATUS_BOUNDARY_SCRIPT
    output_anchor_module = root / PLAN_STATUS_OUTPUT_ANCHOR_MODULE
    plan_status_boundary_line_count = (
        file_line_count(boundary_script) if boundary_script.exists() else 0
    )
    status_output_anchor_module_line_count = (
        file_line_count(output_anchor_module) if output_anchor_module.exists() else 0
    )

    support_files, missing_support_files = file_entries(root, PLAN_STATUS_SUPPORT_FILES)
    support_sources = tuple(
        read_text(root / file_name)
        for file_name in PLAN_STATUS_SUPPORT_FILES
        if (root / file_name).exists()
    )

    runtime_index = root / "docs/plans/zircon_runtime/runtime/index.md"
    runtime_05_plan = root / "docs/plans/zircon_runtime/runtime/05-scene-editor-boundary-closeout.md"
    runtime_14_plan = root / "docs/plans/zircon_runtime/runtime/14-runtime-module-family-closeout.md"
    review = root / "docs/engine-architecture/runtime-architecture-review-m0.md"
    convergence = root / "docs/engine-architecture/runtime-interface-convergence.md"
    index_source = read_text(runtime_index) if runtime_index.exists() else ""
    runtime_05_source = read_text(runtime_05_plan) if runtime_05_plan.exists() else ""
    runtime_14_source = read_text(runtime_14_plan) if runtime_14_plan.exists() else ""
    review_source = read_text(review) if review.exists() else ""
    convergence_source = read_text(convergence) if convergence.exists() else ""

    subplans = runtime_subplans(root)
    numbered_archives = runtime_numbered_archives(root)
    archive_sources = tuple(
        source
        for entries in numbered_archives.values()
        for _, source in entries
    )
    problem_rows = runtime_index_problem_rows(index_source)
    subplan_rows = runtime_index_subplan_rows(index_source)
    backlog_rows = runtime_index_backlog_rows(index_source)
    status_counts: dict[str, int] = {}
    last_refined_violations: list[dict[str, str]] = []
    status_table_gaps: list[str] = []
    in_progress_without_gate: list[str] = []
    remaining_gate_markers = (
        "Cargo",
        "cargo",
        "待",
        "pending",
        "owner",
        "active lane",
        "阻塞",
        "超时",
        "窗口",
    )

    for filename, source in subplans:
        status = frontmatter_value(source, "status:")
        if status is None:
            status_table_gaps.append(f"{filename}: missing status frontmatter")
        else:
            status_counts[status] = status_counts.get(status, 0) + 1
        last_refined = frontmatter_value(source, "last_refined:")
        max_date = max_iso_date(source)
        if last_refined is None:
            last_refined_violations.append(
                {"path": filename, "last_refined": "", "max_date": max_date or ""}
            )
        elif max_date and last_refined < max_date:
            last_refined_violations.append(
                {
                    "path": filename,
                    "last_refined": last_refined,
                    "max_date": max_date,
                }
            )
        plan_number = filename[:2]
        archive_entries = numbered_archives.get(plan_number, [])
        archive_record_count = sum(
            len(archive_status_rows(archive_source))
            for _, archive_source in archive_entries
        )
        linked_archive = any(
            archive_path in source
            or Path(archive_path).name in source
            for archive_path, _ in archive_entries
        )
        if (
            "## 状态与产出记录" not in source
            or archive_record_count == 0
            or not linked_archive
        ):
            status_table_gaps.append(
                f"{filename}: missing numbered-archive status/evidence records"
            )

        if status == "in_progress":
            matching_row = next(
                (line for line in subplan_rows if filename in line),
                "",
            )
            if not any(marker in matching_row for marker in remaining_gate_markers):
                in_progress_without_gate.append(filename)

    missing_core_guard_anchors = missing_snippets(support_sources, CORE_GUARD_ANCHORS)
    missing_pending_gate_anchors = missing_snippets(
        support_sources + (index_source, runtime_05_source, review_source),
        PENDING_GATE_ANCHORS,
    )
    missing_doc_anchors = missing_snippets(
        (index_source, runtime_05_source, review_source, convergence_source),
        DOC_ANCHORS,
    )
    missing_status_output_table_guard_anchors = missing_snippets(
        support_sources + (index_source, runtime_05_source, review_source, convergence_source),
        STATUS_OUTPUT_TABLE_GUARD_ANCHORS,
    )
    missing_runtime_03_module_doc_status_index_anchors = missing_snippets(
        (index_source,) + archive_sources, RUNTIME_03_MODULE_DOC_STATUS_INDEX_ANCHORS
    )
    missing_runtime_03_module_doc_status_guard_anchors = missing_snippets(
        support_sources, RUNTIME_03_MODULE_DOC_STATUS_GUARD_ANCHORS
    )
    missing_runtime_03_module_doc_status_doc_anchors = missing_snippets(
        (runtime_05_source, review_source, convergence_source) + archive_sources,
        RUNTIME_03_MODULE_DOC_STATUS_DOC_ANCHORS,
    )
    missing_runtime_07_scene_status_index_anchors = missing_snippets(
        (index_source,) + archive_sources, RUNTIME_07_SCENE_STATUS_INDEX_ANCHORS
    )
    missing_runtime_07_scene_status_guard_anchors = missing_snippets(
        support_sources, RUNTIME_07_SCENE_STATUS_GUARD_ANCHORS
    )
    missing_runtime_07_scene_status_doc_anchors = missing_snippets(
        (runtime_05_source, review_source, convergence_source) + archive_sources,
        RUNTIME_07_SCENE_STATUS_DOC_ANCHORS,
    )
    missing_runtime_07_owner_budget_status_index_anchors = missing_snippets(
        (index_source,) + archive_sources, RUNTIME_07_OWNER_BUDGET_STATUS_INDEX_ANCHORS
    )
    missing_runtime_07_owner_budget_status_guard_anchors = missing_snippets(
        support_sources, RUNTIME_07_OWNER_BUDGET_STATUS_GUARD_ANCHORS
    )
    missing_runtime_07_owner_budget_status_doc_anchors = missing_snippets(
        (runtime_05_source, review_source, convergence_source) + archive_sources,
        RUNTIME_07_OWNER_BUDGET_STATUS_DOC_ANCHORS,
    )
    missing_runtime_02_generated_status_index_anchors = missing_snippets(
        (index_source,) + archive_sources, RUNTIME_02_GENERATED_STATUS_INDEX_ANCHORS
    )
    missing_runtime_02_generated_status_guard_anchors = missing_snippets(
        support_sources, RUNTIME_02_GENERATED_STATUS_GUARD_ANCHORS
    )
    missing_runtime_02_generated_status_doc_anchors = missing_snippets(
        (runtime_05_source, review_source, convergence_source) + archive_sources,
        RUNTIME_02_GENERATED_STATUS_DOC_ANCHORS,
    )
    missing_runtime_10_behavior_status_index_anchors = missing_snippets(
        (index_source,) + archive_sources, RUNTIME_10_BEHAVIOR_STATUS_INDEX_ANCHORS
    )
    missing_runtime_10_behavior_status_guard_anchors = missing_snippets(
        support_sources, RUNTIME_10_BEHAVIOR_STATUS_GUARD_ANCHORS
    )
    missing_runtime_10_behavior_status_doc_anchors = missing_snippets(
        (runtime_05_source, review_source, convergence_source) + archive_sources,
        RUNTIME_10_BEHAVIOR_STATUS_DOC_ANCHORS,
    )
    cargo_attempt_status_anchors = (
        CARGO_ATTEMPT_STATUS_ANCHORS + CARGO_ATTEMPT_STATUS_EVIDENCE_ANCHORS
    )
    missing_cargo_attempt_status_index_anchors = missing_snippets(
        (index_source,) + archive_sources, cargo_attempt_status_anchors
    )
    missing_cargo_attempt_status_runtime_14_anchors = missing_snippets(
        (runtime_14_source,)
        + tuple(source for _, source in numbered_archives.get("14", [])),
        (
            "cargo_deferred_active_lane",
            "cargo_blocked_external_compile_drift",
            "cargo_recheck_blocked_external_ui_compile_drift",
            "cargo_recheck_timeout_no_result",
            "Cargo 验证窗口探测",
            "animation Cargo gate 尝试",
            "animation Cargo gate 修复与复验阻塞",
            "animation runtime-status focused recheck timeout",
        )
        + CARGO_ATTEMPT_STATUS_EVIDENCE_ANCHORS,
    )
    missing_cargo_attempt_status_guard_anchors = missing_snippets(
        support_sources, CARGO_ATTEMPT_STATUS_ANCHORS
    )
    missing_backlog_gaps = missing_snippets((index_source,), BACKLOG_GAPS)

    runtime_05_status = frontmatter_value(runtime_05_source, "status:")
    runtime_05_closeout_anchors = missing_snippets(
        (runtime_05_source, index_source, review_source),
        (
            "pending_full_scene_cargo",
            "cargo test -p zircon_runtime --lib scene:: --locked",
            "runtime_05_closeout_status_waits_for_full_scene_cargo_gate",
        ),
    )

    risks: list[str] = []
    if missing_support_files:
        risks.append("Runtime plan-status support file set is incomplete.")
    if not output_anchor_module.exists():
        risks.append("Runtime plan-status output-anchor module is missing.")
    if plan_status_boundary_line_count > MAX_PLAN_STATUS_BOUNDARY_LINES:
        risks.append("Runtime plan-status boundary script exceeded its orchestration line budget.")
    if status_output_anchor_module_line_count > MAX_STATUS_OUTPUT_ANCHOR_MODULE_LINES:
        risks.append("Runtime plan-status output-anchor module exceeded its data-owner line budget.")
    if len(support_files) != EXPECTED_PLAN_STATUS_SUPPORT_FILE_COUNT:
        risks.append("Runtime plan-status support file count changed without audit sync.")
    if len(subplans) != EXPECTED_SUBPLAN_COUNT:
        risks.append("Runtime subplan file count changed without audit sync.")
    if len(subplan_rows) != EXPECTED_SUBPLAN_COUNT:
        risks.append("Runtime index subplan map count changed without audit sync.")
    if len(problem_rows) != EXPECTED_PROBLEM_ROW_COUNT:
        risks.append("Runtime index problem-row count changed without audit sync.")
    if len(backlog_rows) != EXPECTED_BACKLOG_ROW_COUNT:
        risks.append("Runtime index known-backlog count changed without audit sync.")
    if missing_backlog_gaps:
        risks.append("Runtime index known-backlog required gaps are missing.")
    if last_refined_violations:
        risks.append("Runtime subplan last_refined frontmatter is behind recorded dates.")
    if status_table_gaps:
        risks.append("Runtime subplan status/evidence tables are incomplete.")
    if in_progress_without_gate:
        risks.append("Runtime index in-progress rows are missing remaining gate markers.")
    if missing_core_guard_anchors:
        risks.append("Runtime plan-status core guard anchors are missing.")
    if missing_pending_gate_anchors:
        risks.append("Runtime pending Cargo gate anchors are missing.")
    if missing_doc_anchors:
        risks.append("Runtime plan-status mirror docs are missing required anchors.")
    if missing_status_output_table_guard_anchors:
        risks.append("Runtime plan-status output-table guard anchors are missing.")
    if (
        missing_runtime_03_module_doc_status_index_anchors
        or missing_runtime_03_module_doc_status_guard_anchors
        or missing_runtime_03_module_doc_status_doc_anchors
    ):
        risks.append("Runtime 03 module-doc status-output anchors are missing.")
    if (
        missing_runtime_07_scene_status_index_anchors
        or missing_runtime_07_scene_status_guard_anchors
        or missing_runtime_07_scene_status_doc_anchors
    ):
        risks.append("Runtime 07 scene asset status-output anchors are missing.")
    if (
        missing_runtime_07_owner_budget_status_index_anchors
        or missing_runtime_07_owner_budget_status_guard_anchors
        or missing_runtime_07_owner_budget_status_doc_anchors
    ):
        risks.append("Runtime 07 owner-budget status-output anchors are missing.")
    if (
        missing_runtime_02_generated_status_index_anchors
        or missing_runtime_02_generated_status_guard_anchors
        or missing_runtime_02_generated_status_doc_anchors
    ):
        risks.append("Runtime 02 generated status-output anchors are missing.")
    if (
        missing_runtime_10_behavior_status_index_anchors
        or missing_runtime_10_behavior_status_guard_anchors
        or missing_runtime_10_behavior_status_doc_anchors
    ):
        risks.append("Runtime 10 behavior status-output anchors are missing.")
    if (
        missing_cargo_attempt_status_index_anchors
        or missing_cargo_attempt_status_runtime_14_anchors
        or missing_cargo_attempt_status_guard_anchors
    ):
        risks.append("Runtime Cargo attempt status anchors are missing.")
    if runtime_05_status != "in_progress":
        risks.append("Runtime 05 closeout status changed before full scene Cargo gate evidence.")
    if runtime_05_closeout_anchors:
        risks.append("Runtime 05 full-scene closeout gate anchors are missing.")

    return {
        "plan_status_boundary_script": PLAN_STATUS_BOUNDARY_SCRIPT,
        "plan_status_output_anchor_module": PLAN_STATUS_OUTPUT_ANCHOR_MODULE,
        "plan_status_boundary_line_count": plan_status_boundary_line_count,
        "max_plan_status_boundary_lines": MAX_PLAN_STATUS_BOUNDARY_LINES,
        "status_output_anchor_module_line_count": status_output_anchor_module_line_count,
        "max_status_output_anchor_module_lines": MAX_STATUS_OUTPUT_ANCHOR_MODULE_LINES,
        "status_output_anchor_module_present": output_anchor_module.exists(),
        "plan_status_boundary_under_line_budget": (
            0 < plan_status_boundary_line_count <= MAX_PLAN_STATUS_BOUNDARY_LINES
        ),
        "status_output_anchor_module_under_line_budget": (
            output_anchor_module.exists()
            and status_output_anchor_module_line_count <= MAX_STATUS_OUTPUT_ANCHOR_MODULE_LINES
        ),
        "support_files": support_files,
        "expected_support_file_count": EXPECTED_PLAN_STATUS_SUPPORT_FILE_COUNT,
        "missing_support_files": missing_support_files,
        "subplan_count": len(subplans),
        "expected_subplan_count": EXPECTED_SUBPLAN_COUNT,
        "status_counts": status_counts,
        "subplan_index_row_count": len(subplan_rows),
        "problem_row_count": len(problem_rows),
        "expected_problem_row_count": EXPECTED_PROBLEM_ROW_COUNT,
        "backlog_row_count": len(backlog_rows),
        "expected_backlog_row_count": EXPECTED_BACKLOG_ROW_COUNT,
        "missing_backlog_gaps": missing_backlog_gaps,
        "last_refined_violations": last_refined_violations,
        "status_table_gaps": status_table_gaps,
        "in_progress_without_gate": in_progress_without_gate,
        "missing_core_guard_anchors": missing_core_guard_anchors,
        "core_guard_count": len(CORE_GUARD_ANCHORS),
        "missing_pending_gate_anchors": missing_pending_gate_anchors,
        "pending_gate_count": len(PENDING_GATE_ANCHORS),
        "missing_doc_anchors": missing_doc_anchors,
        "doc_anchor_count": len(DOC_ANCHORS),
        "status_output_table_guard_count": len(STATUS_OUTPUT_TABLE_GUARD_ANCHORS),
        "missing_status_output_table_guard_anchors": missing_status_output_table_guard_anchors,
        "status_output_table_guard_present": not missing_status_output_table_guard_anchors,
        "runtime_03_module_doc_status_index_anchor_count": len(
            RUNTIME_03_MODULE_DOC_STATUS_INDEX_ANCHORS
        ),
        "missing_runtime_03_module_doc_status_index_anchors": missing_runtime_03_module_doc_status_index_anchors,
        "runtime_03_module_doc_status_guard_anchor_count": len(
            RUNTIME_03_MODULE_DOC_STATUS_GUARD_ANCHORS
        ),
        "missing_runtime_03_module_doc_status_guard_anchors": missing_runtime_03_module_doc_status_guard_anchors,
        "runtime_03_module_doc_status_doc_anchor_count": len(
            RUNTIME_03_MODULE_DOC_STATUS_DOC_ANCHORS
        ),
        "missing_runtime_03_module_doc_status_doc_anchors": missing_runtime_03_module_doc_status_doc_anchors,
        "runtime_03_module_doc_status_guard_present": not (
            missing_runtime_03_module_doc_status_index_anchors
            or missing_runtime_03_module_doc_status_guard_anchors
            or missing_runtime_03_module_doc_status_doc_anchors
        ),
        "runtime_07_scene_status_index_anchor_count": len(
            RUNTIME_07_SCENE_STATUS_INDEX_ANCHORS
        ),
        "missing_runtime_07_scene_status_index_anchors": missing_runtime_07_scene_status_index_anchors,
        "runtime_07_scene_status_guard_anchor_count": len(
            RUNTIME_07_SCENE_STATUS_GUARD_ANCHORS
        ),
        "missing_runtime_07_scene_status_guard_anchors": missing_runtime_07_scene_status_guard_anchors,
        "runtime_07_scene_status_doc_anchor_count": len(
            RUNTIME_07_SCENE_STATUS_DOC_ANCHORS
        ),
        "missing_runtime_07_scene_status_doc_anchors": missing_runtime_07_scene_status_doc_anchors,
        "runtime_07_scene_status_guard_present": not (
            missing_runtime_07_scene_status_index_anchors
            or missing_runtime_07_scene_status_guard_anchors
            or missing_runtime_07_scene_status_doc_anchors
        ),
        "runtime_07_owner_budget_status_index_anchor_count": len(
            RUNTIME_07_OWNER_BUDGET_STATUS_INDEX_ANCHORS
        ),
        "missing_runtime_07_owner_budget_status_index_anchors": missing_runtime_07_owner_budget_status_index_anchors,
        "runtime_07_owner_budget_status_guard_anchor_count": len(
            RUNTIME_07_OWNER_BUDGET_STATUS_GUARD_ANCHORS
        ),
        "missing_runtime_07_owner_budget_status_guard_anchors": missing_runtime_07_owner_budget_status_guard_anchors,
        "runtime_07_owner_budget_status_doc_anchor_count": len(
            RUNTIME_07_OWNER_BUDGET_STATUS_DOC_ANCHORS
        ),
        "missing_runtime_07_owner_budget_status_doc_anchors": missing_runtime_07_owner_budget_status_doc_anchors,
        "runtime_07_owner_budget_status_guard_present": not (
            missing_runtime_07_owner_budget_status_index_anchors
            or missing_runtime_07_owner_budget_status_guard_anchors
            or missing_runtime_07_owner_budget_status_doc_anchors
        ),
        "runtime_02_generated_status_index_anchor_count": len(
            RUNTIME_02_GENERATED_STATUS_INDEX_ANCHORS
        ),
        "missing_runtime_02_generated_status_index_anchors": missing_runtime_02_generated_status_index_anchors,
        "runtime_02_generated_status_guard_anchor_count": len(
            RUNTIME_02_GENERATED_STATUS_GUARD_ANCHORS
        ),
        "missing_runtime_02_generated_status_guard_anchors": missing_runtime_02_generated_status_guard_anchors,
        "runtime_02_generated_status_doc_anchor_count": len(
            RUNTIME_02_GENERATED_STATUS_DOC_ANCHORS
        ),
        "missing_runtime_02_generated_status_doc_anchors": missing_runtime_02_generated_status_doc_anchors,
        "runtime_02_generated_status_guard_present": not (
            missing_runtime_02_generated_status_index_anchors
            or missing_runtime_02_generated_status_guard_anchors
            or missing_runtime_02_generated_status_doc_anchors
        ),
        "runtime_10_behavior_status_index_anchor_count": len(
            RUNTIME_10_BEHAVIOR_STATUS_INDEX_ANCHORS
        ),
        "missing_runtime_10_behavior_status_index_anchors": missing_runtime_10_behavior_status_index_anchors,
        "runtime_10_behavior_status_guard_anchor_count": len(
            RUNTIME_10_BEHAVIOR_STATUS_GUARD_ANCHORS
        ),
        "missing_runtime_10_behavior_status_guard_anchors": missing_runtime_10_behavior_status_guard_anchors,
        "runtime_10_behavior_status_doc_anchor_count": len(
            RUNTIME_10_BEHAVIOR_STATUS_DOC_ANCHORS
        ),
        "missing_runtime_10_behavior_status_doc_anchors": missing_runtime_10_behavior_status_doc_anchors,
        "runtime_10_behavior_status_guard_present": not (
            missing_runtime_10_behavior_status_index_anchors
            or missing_runtime_10_behavior_status_guard_anchors
            or missing_runtime_10_behavior_status_doc_anchors
        ),
        "cargo_attempt_status_anchor_count": len(cargo_attempt_status_anchors),
        "missing_cargo_attempt_status_index_anchors": missing_cargo_attempt_status_index_anchors,
        "missing_cargo_attempt_status_runtime_14_anchors": missing_cargo_attempt_status_runtime_14_anchors,
        "missing_cargo_attempt_status_guard_anchors": missing_cargo_attempt_status_guard_anchors,
        "cargo_attempt_status_guard_present": not (
            missing_cargo_attempt_status_index_anchors
            or missing_cargo_attempt_status_runtime_14_anchors
            or missing_cargo_attempt_status_guard_anchors
        ),
        "runtime_05_status": runtime_05_status,
        "runtime_05_closeout_anchors": runtime_05_closeout_anchors,
        "risks": risks,
    }
