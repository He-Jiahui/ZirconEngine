import argparse
import hashlib
import json
import subprocess
from pathlib import Path
from typing import Any


SCHEMA = "zircon.editor.hierarchy_sparse_publication_pressure.v1"

CRITICAL_SOURCE_CONTRACTS = (
    (
        "zircon_editor/src/ui/retained_host/app/host_lifecycle/"
        "scene_hierarchy_refresh.rs",
        (
            "fn publish_sparse_hierarchy_host_nodes",
            "build_presented_hierarchy_pane_patch",
            "HostPresentationPatch::new",
            ".with_workbench_nodes",
            ".patch_host_presentation",
        ),
    ),
    (
        "zircon_editor/src/ui/retained_host/app/host_lifecycle/"
        "scene_hierarchy_refresh/hierarchy_row_patch.rs",
        (
            "fn build_presented_hierarchy_pane_patch",
            "rows.with_row_patches(materialized_patches)",
            "HostPanePresentationLocation::NativeFloating",
            "next.hierarchy.hierarchy_nodes = rows.clone()",
            'if window.active_pane.kind.as_str() != "Hierarchy"',
        ),
    ),
    (
        "zircon_editor/src/ui/retained_host/host_contract/data/"
        "pane_presentation_patch.rs",
        (
            "pub(crate) struct HostPresentationPatch",
            "pub(crate) fn with_workbench_nodes",
            "pub(crate) fn paint_model_replacements",
            "presentation.workbench_window_nodes = replacement.nodes",
        ),
    ),
    (
        "zircon_editor/src/ui/retained_host/host_contract/globals/state.rs",
        (
            "pub(crate) fn patch_host_presentation(&mut self, patch: HostPresentationPatch)",
            ".rebind_workbench_nodes(",
            ".rebind_paint_models(&model_replacements)",
            "ui.sparse_presentation_transaction.commit_count",
            "ui.sparse_presentation_transaction.pane_validation_fallback_count",
            "ui.sparse_presentation_transaction.workbench_rebind_fallback_count",
            "ui.sparse_presentation_transaction.paint_rebind_fallback_count",
        ),
    ),
    (
        "zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/"
        "template_node/index.rs",
        (
            "pub(crate) fn rebind_workbench_nodes(",
            "previous_nodes.row_count() != next_nodes.row_count()",
            "!same_index_membership(previous, next)",
            "let mut paint_indices = self.paint_indices.clone();",
            "ui.workbench_hit_index.rebind_paint_index_handle_clone_count",
        ),
    ),
    (
        "dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/"
        "SlateInvalidationRoot.cpp",
        (
            "bool FSlateInvalidationRoot::PaintFastPath",
            "bool FSlateInvalidationRoot::ProcessInvalidation",
            "FinalUpdateList",
        ),
    ),
    (
        "dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/"
        "WidgetProxy.cpp",
        (
            "FWidgetProxy::ProcessLayoutInvalidation",
            "EInvalidateWidgetReason::Paint",
            "EWidgetUpdateFlags::NeedsRenderTransform",
        ),
    ),
)


class SourceContractError(RuntimeError):
    """Raised when the pressure model no longer describes current source."""


def _sha256(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest().upper()


def _git_output(repo_root: Path, *args: str) -> str | None:
    try:
        completed = subprocess.run(
            ["git", *args],
            cwd=repo_root,
            check=True,
            capture_output=True,
            text=True,
        )
    except (OSError, subprocess.CalledProcessError):
        return None
    return completed.stdout.strip()


def _sparse_publication_body(source: str) -> str:
    try:
        return source.split("fn publish_sparse_hierarchy_host_nodes", 1)[1].split(
            "\n    }\n}", 1
        )[0]
    except IndexError as error:
        raise SourceContractError("sparse hierarchy publication body is missing") from error


def source_binding_report(repo_root: Path) -> dict[str, Any]:
    repo_root = repo_root.resolve()
    critical_sources = []
    source_set = hashlib.sha256()
    relative_paths = []
    for relative_path, required_tokens in CRITICAL_SOURCE_CONTRACTS:
        path = repo_root / relative_path
        try:
            payload = path.read_bytes()
        except OSError as error:
            raise SourceContractError(f"missing critical source: {relative_path}") from error
        source = payload.decode("utf-8")
        missing = [token for token in required_tokens if token not in source]
        if missing:
            raise SourceContractError(
                f"critical source contract changed: {relative_path}: {missing}"
            )
        if relative_path.endswith("scene_hierarchy_refresh.rs"):
            sparse = _sparse_publication_body(source)
            obsolete = [
                token
                for token in ("get_host_presentation", "set_host_presentation")
                if token in sparse
            ]
            if obsolete:
                raise SourceContractError(
                    f"sparse publication restored full presentation work: {obsolete}"
                )
        digest = _sha256(payload)
        critical_sources.append(
            {
                "relative_path": relative_path,
                "sha256": digest,
                "byte_length": len(payload),
            }
        )
        relative_paths.append(relative_path)
        source_set.update(relative_path.encode("utf-8"))
        source_set.update(b"\0")
        source_set.update(bytes.fromhex(digest))

    dirty = _git_output(repo_root, "status", "--short", "--", *relative_paths) or ""
    return {
        "ready": True,
        "git_revision": _git_output(repo_root, "rev-parse", "HEAD") or "unavailable",
        "critical_source_dirty": bool(dirty),
        "critical_source_dirty_entry_count": len(dirty.splitlines()) if dirty else 0,
        "source_set_sha256": source_set.hexdigest().upper(),
        "critical_sources": critical_sources,
    }


def run(
    *,
    interaction_count: int = 65_536,
    workbench_node_count: int = 32_768,
    paint_model_count: int = 64,
    hierarchy_pane_count: int = 6,
    floating_window_count: int = 16,
    changed_workbench_rows: int = 1,
    changed_hierarchy_rows: int = 1,
) -> dict[str, Any]:
    inputs = {
        "interaction_count": interaction_count,
        "workbench_node_count": workbench_node_count,
        "paint_model_count": paint_model_count,
        "hierarchy_pane_count": hierarchy_pane_count,
        "floating_window_count": floating_window_count,
        "changed_workbench_rows": changed_workbench_rows,
        "changed_hierarchy_rows": changed_hierarchy_rows,
    }
    if any(value <= 0 for value in inputs.values()):
        raise ValueError("all pressure inputs must be positive")

    retired_node_visits = interaction_count * workbench_node_count
    retired_model_identity_visits = interaction_count * paint_model_count
    current_row_validations = interaction_count * changed_workbench_rows
    current_pane_validations = interaction_count * hierarchy_pane_count
    current_paint_index_handle_clones = interaction_count * paint_model_count
    current_hierarchy_row_copies = interaction_count * changed_hierarchy_rows
    shared_floating_window_visits = interaction_count * floating_window_count * 2
    retired_modeled_work = (
        retired_node_visits
        + retired_model_identity_visits
        + shared_floating_window_visits
    )
    current_modeled_work = (
        current_row_validations
        + current_pane_validations
        + current_paint_index_handle_clones
        + current_hierarchy_row_copies
        + shared_floating_window_visits
    )

    return {
        "schema": SCHEMA,
        "inputs": inputs,
        "retired_full_publication": {
            "presentation_snapshot_count": interaction_count,
            "presentation_set_count": interaction_count,
            "paint_model_identity_visit_count": retired_model_identity_visits,
            "full_workbench_hit_index_rebuild_count": interaction_count,
            "modeled_workbench_node_visit_count": retired_node_visits,
            "floating_window_visit_count": shared_floating_window_visits,
            "modeled_work_units": retired_modeled_work,
            "complexity": "O(I * (N + M)) before paint-model row indexing",
        },
        "atomic_sparse_transaction": {
            "presentation_snapshot_count": 0,
            "presentation_set_count": 0,
            "full_workbench_hit_index_rebuild_count": 0,
            "exact_workbench_row_validation_count": current_row_validations,
            "hierarchy_pane_validation_count": current_pane_validations,
            "paint_index_handle_clone_count": current_paint_index_handle_clones,
            "hierarchy_row_copy_count": current_hierarchy_row_copies,
            "floating_window_visit_count": shared_floating_window_visits,
            "modeled_work_units": current_modeled_work,
            "complexity": "O(I * (K + P + M)); K changed rows, P hierarchy panes, M paint-index handles",
        },
        "delta": {
            "avoided_presentation_snapshot_count": interaction_count,
            "avoided_full_hit_index_rebuild_count": interaction_count,
            "avoided_modeled_workbench_node_visits": retired_node_visits,
            "modeled_work_reduction_units": retired_modeled_work - current_modeled_work,
            "modeled_work_ratio": round(retired_modeled_work / current_modeled_work, 2),
        },
        "interpretation": {
            "is_product_timing": False,
            "included": "source-proven aggregate publication, model identity visits, main workbench indexing, exact row checks, pane checks, floating-window lookup visits, and paint-index handle clones",
            "excluded": "paint-model row indexing, allocator/cache effects, CPU/GPU time, redraw damage, RSS, and input-to-present latency",
            "required_product_evidence": "sparse transaction counters, full hit-index builds, node visits, allocation counts, damage area, and input-to-present p50/p95/p99",
        },
    }


def write_result(output: Path, result: dict[str, Any]) -> None:
    if output.drive.casefold() == "c:":
        raise ValueError("profile artifacts must not be written to the C drive")
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--interaction-count", type=int, default=65_536)
    parser.add_argument("--workbench-node-count", type=int, default=32_768)
    parser.add_argument("--paint-model-count", type=int, default=64)
    parser.add_argument("--hierarchy-pane-count", type=int, default=6)
    parser.add_argument("--floating-window-count", type=int, default=16)
    parser.add_argument("--changed-workbench-rows", type=int, default=1)
    parser.add_argument("--changed-hierarchy-rows", type=int, default=1)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        interaction_count=args.interaction_count,
        workbench_node_count=args.workbench_node_count,
        paint_model_count=args.paint_model_count,
        hierarchy_pane_count=args.hierarchy_pane_count,
        floating_window_count=args.floating_window_count,
        changed_workbench_rows=args.changed_workbench_rows,
        changed_hierarchy_rows=args.changed_hierarchy_rows,
    )
    result["source_binding"] = source_binding_report(Path(__file__).resolve().parents[4])
    if args.output is not None:
        write_result(args.output, result)
    print(json.dumps(result, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
