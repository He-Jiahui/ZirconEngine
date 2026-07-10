from __future__ import annotations


def render_schedule_frame_loop_boundary_markdown(boundary: dict[str, object]) -> list[str]:
    source_files = boundary["source_files"]
    guard_files = boundary["guard_files"]
    lines = [
        "## Runtime 03 Schedule Frame-Loop Boundary",
        "- audited schedule/frame-loop source files "
        f"({len(source_files)}/{boundary['expected_source_file_count']}): "
        f"{len(source_files)} files",
        "- audited Runtime 03 guard/test files "
        f"({len(guard_files)}/{boundary['expected_guard_file_count']}): "
        f"{len(guard_files)} files",
        "- SystemStage authority: "
        f"COUNT={boundary['stage_count']}/{boundary['expected_stage_count']}, "
        f"variants={boundary['declared_stage_variant_count']}/"
        f"{boundary['expected_stage_count']}, "
        f"fixed_loop={boundary['fixed_loop_stage_count']}/"
        f"{boundary['expected_fixed_loop_stage_count']}",
        "- dynamic-session time advance calls: "
        f"{boundary['dynamic_session_tick_time_call_count']}/"
        f"{boundary['expected_dynamic_session_tick_time_call_count']}",
        "- Runtime 03 guard anchors: "
        f"{boundary['test_anchor_count'] - len(boundary['missing_test_anchors'])}/"
        f"{boundary['test_anchor_count']}",
        "- Runtime 03 behavior test anchors: "
        f"{boundary['behavior_test_anchor_count'] - len(boundary['missing_behavior_test_anchors'])}/"
        f"{boundary['behavior_test_anchor_count']}",
        "- mirror-doc aggregate guard: "
        f"{'present' if boundary['mirror_docs_guard_present'] else 'missing'}",
        "- Runtime 03 doc anchors: "
        f"{boundary['doc_anchor_count'] - len(boundary['missing_doc_anchors'])}/"
        f"{boundary['doc_anchor_count']}",
        "- frame schedule module-doc anchors: "
        f"{boundary['frame_schedule_doc_anchor_count'] - len(boundary['missing_frame_schedule_doc_anchors'])}/"
        f"{boundary['frame_schedule_doc_anchor_count']}",
        "- WorldDriver second time-advance references: "
        f"{len(boundary['world_driver_second_advance_references'])}",
        "- raw-delta LevelSystem tick references from dynamic session: "
        f"{len(boundary['session_raw_delta_tick_references'])}",
    ]

    if boundary["missing_source_files"]:
        lines.append(
            "- missing Runtime 03 source files: "
            f"{', '.join(boundary['missing_source_files'])}"
        )
    if boundary["missing_guard_files"]:
        lines.append(
            "- missing Runtime 03 guard/test files: "
            f"{', '.join(boundary['missing_guard_files'])}"
        )
    if boundary["missing_stage_variants"]:
        lines.append(
            "- missing SystemStage variants: "
            f"{', '.join(boundary['missing_stage_variants'])}"
        )
    if boundary["missing_system_stage_anchors"]:
        lines.append(
            "- missing SystemStage authority anchors: "
            f"{', '.join(boundary['missing_system_stage_anchors'])}"
        )
    if boundary["missing_session_tick_anchors"]:
        lines.append(
            "- missing dynamic-session tick anchors: "
            f"{', '.join(boundary['missing_session_tick_anchors'])}"
        )
    if boundary["missing_time_handoff_anchors"]:
        lines.append(
            "- missing RuntimeTimeAdvance handoff anchors: "
            f"{', '.join(boundary['missing_time_handoff_anchors'])}"
        )
    if boundary["missing_fixed_plan_anchors"]:
        lines.append(
            "- missing FixedStepPlan anchors: "
            f"{', '.join(boundary['missing_fixed_plan_anchors'])}"
        )
    if boundary["missing_ui_extract_anchors"]:
        lines.append(
            "- missing UI extract side-path anchors: "
            f"{', '.join(boundary['missing_ui_extract_anchors'])}"
        )
    if boundary["ui_extract_render_stage_references"]:
        lines.append("- UI extract RenderExtract references:")
        for reference in boundary["ui_extract_render_stage_references"]:
            lines.append(
                f"  - `{reference['path']}:{reference['line']}` {reference['snippet']}"
            )
    if boundary["missing_schedule_order_anchors"]:
        lines.append(
            "- missing explicit ordering anchors: "
            f"{', '.join(boundary['missing_schedule_order_anchors'])}"
        )
    if boundary["missing_schedule_runner_anchors"]:
        lines.append(
            "- missing schedule-runner anchors: "
            f"{', '.join(boundary['missing_schedule_runner_anchors'])}"
        )
    if boundary["missing_parallel_executor_anchors"]:
        lines.append(
            "- missing parallel-executor anchors: "
            f"{', '.join(boundary['missing_parallel_executor_anchors'])}"
        )
    if boundary["missing_test_anchors"]:
        lines.append(
            "- missing Runtime 03 test anchors: "
            f"{', '.join(boundary['missing_test_anchors'])}"
        )
    if boundary["missing_behavior_test_anchors"]:
        lines.append(
            "- missing Runtime 03 behavior test anchors: "
            f"{', '.join(boundary['missing_behavior_test_anchors'])}"
        )
    if boundary["missing_frame_schedule_doc_anchors"]:
        lines.append(
            "- missing frame schedule module-doc anchors: "
            f"{', '.join(boundary['missing_frame_schedule_doc_anchors'])}"
        )
    if boundary["missing_doc_anchors"]:
        lines.append(
            "- missing Runtime 03 doc anchors: "
            f"{', '.join(boundary['missing_doc_anchors'])}"
        )
    if boundary["missing_cargo_gate_anchors"]:
        lines.append(
            "- missing pending Cargo gate anchors: "
            f"{', '.join(boundary['missing_cargo_gate_anchors'])}"
        )
    if boundary["world_driver_second_advance_references"]:
        lines.append("- WorldDriver second time-advance references:")
        for reference in boundary["world_driver_second_advance_references"]:
            lines.append(
                f"  - `{reference['path']}:{reference['line']}` {reference['snippet']}"
            )
    if boundary["session_raw_delta_tick_references"]:
        lines.append("- dynamic-session raw-delta tick references:")
        for reference in boundary["session_raw_delta_tick_references"]:
            lines.append(
                f"  - `{reference['path']}:{reference['line']}` {reference['snippet']}"
            )

    for risk in boundary["risks"]:
        lines.append(f"- risk: {risk}")

    return lines
