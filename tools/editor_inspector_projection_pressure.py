import argparse
import hashlib
import json
from pathlib import Path
import subprocess
from typing import Any


INSPECTOR_SURFACE_PIPELINE_STAGE_COUNT = 4
SOURCE_PATHS = (
    "zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs",
    "zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/inspector.rs",
    "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/inspector_projection.rs",
    "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/inspector_fields.rs",
    "zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs",
    "zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/shell/builder.rs",
    "zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute.rs",
)
SOURCE_GUARDS = {
    SOURCE_PATHS[0]: (
        "let inspector = selected.map(|id| InspectorSnapshot {",
        "plugin_components: inspector_plugin_components(",
    ),
    SOURCE_PATHS[1]: (
        "fn plugin_component_payload(",
        ".map(plugin_component_property_payload)",
        "value: property.value.clone()",
    ),
    SOURCE_PATHS[2]: (
        ".project_pane_body(",
        ".build_shared_surface(",
        ".compute_layout(",
        ".build_host_model_with_surface(",
        "payload\n            .plugin_components\n            .iter()",
    ),
    SOURCE_PATHS[3]: (
        "plugin_components: data.plugin_components.clone()",
        "for component in &fields.plugin_components",
        "for property in &component.properties",
    ),
    SOURCE_PATHS[4]: ("Arc::as_ptr(&document.compiled) as usize",),
    SOURCE_PATHS[5]: (
        "fn build_window_metrics_shell_snapshot(",
        "let committed = self.committed_shell_state.take()?;",
        "retained_pane_payloads: committed.pane_payloads",
        "retained_shell_presentation: committed.retained_shell_presentation",
    ),
    SOURCE_PATHS[6]: (
        "self.build_window_metrics_shell_snapshot()",
        "apply_window_metrics_geometry_presentation(",
        "shell.retained_pane_payloads.as_ref()",
        "self.committed_shell_state = Some(committed_shell_state::CommittedShellState",
    ),
}

REFERENCE_SOURCE_PATHS = (
    "dev/UnrealEngine/Engine/Source/Editor/PropertyEditor/Private/SDetailsView.cpp",
    "dev/UnrealEngine/Engine/Source/Editor/PropertyEditor/Private/DetailLayoutBuilderImpl.cpp",
    "dev/Fyrox/fyrox-ui/src/inspector/mod.rs",
    "dev/slint/internal/core/model.rs",
)
REFERENCE_SOURCE_GUARDS = {
    REFERENCE_SOURCE_PATHS[0]: (
        ".TreeItemsSource(&RootTreeNodes)",
        "if( bForceRefresh || ShouldSetNewObjects(InObjects) )",
        "SetObjectArrayPrivate(InObjects);",
    ),
    REFERENCE_SOURCE_PATHS[1]: (
        "void FDetailLayoutBuilderImpl::FilterDetailLayout",
        "void FDetailLayoutBuilderImpl::RefreshNodeVisbility()",
        "Node->RefreshVisibility();",
    ),
    REFERENCE_SOURCE_PATHS[2]: (
        "pub struct InspectorContext {",
        "pub entries: Vec<ContextEntry>",
        "pub fn sync(",
        "message.delivery_mode = DeliveryMode::SyncOnly;",
    ),
    REFERENCE_SOURCE_PATHS[3]: (
        "pub trait ModelTracker {",
        "fn track_row_data_changes(&self, row: usize);",
        "ModelNotify::row_changed",
        "ModelNotify::row_added",
        "ModelNotify::row_removed",
    ),
}


def run(
    *,
    plugin_property_count: int,
    authored_node_count: int,
    physical_slot_count: int,
    stable_recompute_count: int,
    resize_step_count: int,
    delta_update_count: int,
    changed_fields_per_delta: int,
    cache_entry_limit: int,
    resize_fallback_count: int = 0,
) -> dict[str, Any]:
    if plugin_property_count <= 0:
        raise ValueError("plugin_property_count must be positive")
    if authored_node_count <= 0:
        raise ValueError("authored_node_count must be positive")
    if not 0 < physical_slot_count <= plugin_property_count:
        raise ValueError(
            "physical_slot_count must be positive and no larger than plugin_property_count"
        )
    if stable_recompute_count < 0 or resize_step_count < 0 or delta_update_count < 0:
        raise ValueError("operation counts must be non-negative")
    if not 0 <= changed_fields_per_delta <= plugin_property_count:
        raise ValueError("changed_fields_per_delta must be within plugin_property_count")
    if delta_update_count > 0 and changed_fields_per_delta == 0:
        raise ValueError("delta updates must identify at least one changed field")
    if cache_entry_limit <= 0:
        raise ValueError("cache_entry_limit must be positive")
    if not 0 <= resize_fallback_count <= resize_step_count:
        raise ValueError("resize_fallback_count must be within resize_step_count")

    baseline_initial_property_materialization_count = plugin_property_count
    baseline_stable_property_materialization_count = (
        stable_recompute_count * plugin_property_count
    )
    baseline_resize_property_materialization_count = (
        resize_fallback_count * plugin_property_count
    )
    rejected_resize_full_rebuild_property_materialization_count = (
        resize_step_count * plugin_property_count
    )
    baseline_delta_property_materialization_count = delta_update_count * plugin_property_count
    baseline_total_property_materialization_count = sum(
        (
            baseline_initial_property_materialization_count,
            baseline_stable_property_materialization_count,
            baseline_resize_property_materialization_count,
            baseline_delta_property_materialization_count,
        )
    )
    current_pane_payload_property_copy_count = (
        baseline_total_property_materialization_count
    )
    current_two_stage_property_record_work = (
        baseline_total_property_materialization_count
        + current_pane_payload_property_copy_count
    )

    retained_initial_property_materialization_count = plugin_property_count
    retained_stable_property_materialization_count = 0
    retained_resize_property_materialization_count = 0
    retained_delta_property_materialization_count = (
        delta_update_count * changed_fields_per_delta
    )
    retained_total_property_materialization_count = (
        retained_initial_property_materialization_count
        + retained_delta_property_materialization_count
    )

    baseline_stable_surface_build_count = stable_recompute_count
    baseline_resize_surface_build_count = resize_fallback_count
    rejected_resize_full_rebuild_surface_build_count = resize_step_count
    baseline_delta_surface_build_count = delta_update_count
    baseline_total_surface_build_count = (
        1
        + baseline_stable_surface_build_count
        + baseline_resize_surface_build_count
        + baseline_delta_surface_build_count
    )
    retained_stable_surface_build_count = 0
    retained_resize_surface_build_count = 0
    retained_delta_surface_build_count = 0
    retained_total_surface_build_count = 1

    baseline_nodes_per_surface = authored_node_count + plugin_property_count
    retained_nodes_per_surface = authored_node_count + physical_slot_count
    baseline_surface_node_stage_visit_count = (
        baseline_total_surface_build_count
        * baseline_nodes_per_surface
        * INSPECTOR_SURFACE_PIPELINE_STAGE_COUNT
    )
    retained_initial_surface_node_stage_visit_count = (
        retained_nodes_per_surface * INSPECTOR_SURFACE_PIPELINE_STAGE_COUNT
    )
    retained_resize_geometry_visit_count = resize_step_count * retained_nodes_per_surface
    retained_delta_field_patch_count = retained_delta_property_materialization_count
    current_metrics_fast_path_count = resize_step_count - resize_fallback_count
    two_stage_property_record_work_ratio = (
        current_two_stage_property_record_work
        / retained_total_property_materialization_count
    )

    return {
        "model_scope": (
            "deterministic Inspector projection/materialization operation counts; "
            "not elapsed time, allocator bytes, or RSS"
        ),
        "inspector_surface_pipeline_stage_count": INSPECTOR_SURFACE_PIPELINE_STAGE_COUNT,
        "plugin_property_count": plugin_property_count,
        "authored_node_count": authored_node_count,
        "physical_slot_count": physical_slot_count,
        "stable_recompute_count": stable_recompute_count,
        "resize_step_count": resize_step_count,
        "current_metrics_fast_path_count": current_metrics_fast_path_count,
        "current_metrics_fallback_count": resize_fallback_count,
        "delta_update_count": delta_update_count,
        "changed_fields_per_delta": changed_fields_per_delta,
        "cache_entry_limit": cache_entry_limit,
        "baseline_initial_property_materialization_count": (
            baseline_initial_property_materialization_count
        ),
        "baseline_stable_property_materialization_count": (
            baseline_stable_property_materialization_count
        ),
        "baseline_resize_property_materialization_count": (
            baseline_resize_property_materialization_count
        ),
        "rejected_resize_full_rebuild_property_materialization_count": (
            rejected_resize_full_rebuild_property_materialization_count
        ),
        "baseline_delta_property_materialization_count": (
            baseline_delta_property_materialization_count
        ),
        "baseline_total_property_materialization_count": (
            baseline_total_property_materialization_count
        ),
        "retained_initial_property_materialization_count": (
            retained_initial_property_materialization_count
        ),
        "retained_stable_property_materialization_count": (
            retained_stable_property_materialization_count
        ),
        "retained_resize_property_materialization_count": (
            retained_resize_property_materialization_count
        ),
        "retained_delta_property_materialization_count": (
            retained_delta_property_materialization_count
        ),
        "retained_total_property_materialization_count": (
            retained_total_property_materialization_count
        ),
        "eliminated_property_materialization_count": (
            baseline_total_property_materialization_count
            - retained_total_property_materialization_count
        ),
        "property_materialization_reduction_ratio": (
            baseline_total_property_materialization_count
            / retained_total_property_materialization_count
        ),
        "current_pane_payload_property_copy_count": (
            current_pane_payload_property_copy_count
        ),
        "current_two_stage_property_record_work": current_two_stage_property_record_work,
        "two_stage_property_record_work_ratio": two_stage_property_record_work_ratio,
        "baseline_stable_surface_build_count": baseline_stable_surface_build_count,
        "baseline_resize_surface_build_count": baseline_resize_surface_build_count,
        "rejected_resize_full_rebuild_surface_build_count": (
            rejected_resize_full_rebuild_surface_build_count
        ),
        "baseline_delta_surface_build_count": baseline_delta_surface_build_count,
        "baseline_total_surface_build_count": baseline_total_surface_build_count,
        "retained_stable_surface_build_count": retained_stable_surface_build_count,
        "retained_resize_surface_build_count": retained_resize_surface_build_count,
        "retained_delta_surface_build_count": retained_delta_surface_build_count,
        "retained_total_surface_build_count": retained_total_surface_build_count,
        "baseline_surface_node_stage_visit_count": baseline_surface_node_stage_visit_count,
        "retained_initial_surface_node_stage_visit_count": (
            retained_initial_surface_node_stage_visit_count
        ),
        "retained_resize_geometry_visit_count": retained_resize_geometry_visit_count,
        "retained_delta_field_patch_count": retained_delta_field_patch_count,
        "retained_logical_property_copy_count": plugin_property_count,
        "retained_cached_payload_property_capacity": plugin_property_count,
        "retained_surface_node_capacity": cache_entry_limit * retained_nodes_per_surface,
    }


def write_result(output: Path, result: dict[str, Any]) -> None:
    if output.drive.upper() not in {"D:", "E:", "F:"}:
        raise ValueError("profile artifacts must be written to D:, E:, or F:")
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(
        json.dumps(result, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )


def validate_source_contract(sources: dict[str, str]) -> dict[str, Any]:
    blockers: list[dict[str, str]] = []
    for relative_path in SOURCE_PATHS:
        source = sources.get(relative_path)
        if source is None:
            blockers.append(
                {
                    "code": "missing_critical_source",
                    "relative_path": relative_path,
                }
            )
            continue
        for anchor in SOURCE_GUARDS[relative_path]:
            if anchor not in source:
                blockers.append(
                    {
                        "code": "missing_current_source_anchor",
                        "relative_path": relative_path,
                        "anchor": anchor,
                    }
                )
    return {"ready": not blockers, "blockers": blockers}


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest().upper()


def build_source_binding(repo_root: Path) -> dict[str, Any]:
    root = repo_root.resolve()
    sources: dict[str, str] = {}
    critical_source_files: list[dict[str, Any]] = []
    reference_source_files: list[dict[str, Any]] = []
    blockers: list[dict[str, str]] = []
    for relative_path in SOURCE_PATHS:
        path = root / relative_path
        if not path.is_file():
            blockers.append(
                {
                    "code": "missing_critical_source",
                    "relative_path": relative_path,
                }
            )
            continue
        sources[relative_path] = path.read_text(encoding="utf-8")
        critical_source_files.append(
            {
                "relative_path": relative_path,
                "byte_length": path.stat().st_size,
                "sha256": _sha256(path),
            }
        )

    blockers.extend(validate_source_contract(sources)["blockers"])
    for relative_path in REFERENCE_SOURCE_PATHS:
        path = root / relative_path
        if not path.is_file():
            blockers.append(
                {
                    "code": "missing_reference_source",
                    "relative_path": relative_path,
                }
            )
            continue
        source = path.read_text(encoding="utf-8", errors="replace")
        for anchor in REFERENCE_SOURCE_GUARDS[relative_path]:
            if anchor not in source:
                blockers.append(
                    {
                        "code": "missing_reference_source_anchor",
                        "relative_path": relative_path,
                        "anchor": anchor,
                    }
                )
        reference_source_files.append(
            {
                "relative_path": relative_path,
                "byte_length": path.stat().st_size,
                "sha256": _sha256(path),
            }
        )
    try:
        revision = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=root,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()
        dirty_lines = subprocess.run(
            ["git", "status", "--short", "--", *SOURCE_PATHS],
            cwd=root,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.splitlines()
    except (OSError, subprocess.CalledProcessError) as error:
        revision = ""
        dirty_lines = []
        blockers.append(
            {
                "code": "git_source_binding_failed",
                "detail": str(error),
            }
        )

    model_path = Path(__file__).resolve()
    source_set = hashlib.sha256()
    for entry in (*critical_source_files, *reference_source_files):
        source_set.update(entry["relative_path"].encode("utf-8"))
        source_set.update(b"\0")
        source_set.update(entry["sha256"].encode("ascii"))
        source_set.update(b"\n")
    return {
        "ready": not blockers,
        "blockers": blockers,
        "repository_root": str(root),
        "git_revision": revision,
        "dirty_paths": [line[3:] for line in dirty_lines if len(line) > 3],
        "critical_source_files": critical_source_files,
        "reference_source_files": reference_source_files,
        "source_set_sha256": source_set.hexdigest().upper(),
        "model_source_sha256": _sha256(model_path),
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--plugin-property-count", type=int, default=10_000)
    parser.add_argument("--authored-node-count", type=int, default=256)
    parser.add_argument("--physical-slot-count", type=int, default=64)
    parser.add_argument("--stable-recompute-count", type=int, default=1_000)
    parser.add_argument("--resize-step-count", type=int, default=200)
    parser.add_argument("--delta-update-count", type=int, default=1_000)
    parser.add_argument("--changed-fields-per-delta", type=int, default=1)
    parser.add_argument("--cache-entry-limit", type=int, default=8)
    parser.add_argument("--resize-fallback-count", type=int, default=0)
    parser.add_argument(
        "--repo-root", type=Path, default=Path(__file__).resolve().parents[1]
    )
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        plugin_property_count=args.plugin_property_count,
        authored_node_count=args.authored_node_count,
        physical_slot_count=args.physical_slot_count,
        stable_recompute_count=args.stable_recompute_count,
        resize_step_count=args.resize_step_count,
        delta_update_count=args.delta_update_count,
        changed_fields_per_delta=args.changed_fields_per_delta,
        cache_entry_limit=args.cache_entry_limit,
        resize_fallback_count=args.resize_fallback_count,
    )
    source_binding = build_source_binding(args.repo_root)
    result["source_binding"] = source_binding
    result["ready"] = source_binding["ready"]
    if args.output is not None:
        write_result(args.output, result)
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0 if source_binding["ready"] else 2


if __name__ == "__main__":
    raise SystemExit(main())
