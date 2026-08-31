"""Model retained fallback-reason aggregation for layout diagnostics."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from pathlib import Path


def validate_output_path(path: Path) -> Path:
    if path.drive.upper() not in {"D:", "E:", "F:"}:
        raise ValueError("performance artifacts must be written to D:, E:, or F:")
    return path


CRITICAL_SOURCE_FILES = (
    "zircon_runtime_interface/src/ui/layout/engine.rs",
    "zircon_runtime_interface/src/ui/surface/persistent_sequence.rs",
    "zircon_runtime/src/ui/surface/surface/frame_publication.rs",
    "dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp",
)


def build_source_binding(source_root: Path) -> dict[str, object]:
    revision = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=source_root,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    source_sha256 = {
        path: hashlib.sha256((source_root / path).read_bytes()).hexdigest().upper()
        for path in CRITICAL_SOURCE_FILES
    }
    manifest_payload = "\n".join(
        f"{path}={source_sha256[path]}" for path in CRITICAL_SOURCE_FILES
    ).encode("utf-8")
    return {
        "git_revision": revision,
        "critical_source_files": list(CRITICAL_SOURCE_FILES),
        "source_sha256": source_sha256,
        "source_manifest_sha256": hashlib.sha256(manifest_payload).hexdigest().upper(),
    }


def run(
    selection_count: int = 10_000,
    non_native_selection_count: int = 10_000,
    distinct_reason_count: int = 8,
    recompute_count: int = 1_000,
    changed_selection_count: int = 1,
    selection_segment_size: int = 64,
    directory_fanout: int = 32,
    selection_payload_bytes: int = 128,
) -> dict[str, object]:
    for name, value in (
        ("selection_count", selection_count),
        ("non_native_selection_count", non_native_selection_count),
        ("distinct_reason_count", distinct_reason_count),
        ("recompute_count", recompute_count),
        ("changed_selection_count", changed_selection_count),
        ("selection_segment_size", selection_segment_size),
        ("selection_payload_bytes", selection_payload_bytes),
    ):
        if value <= 0:
            raise ValueError(f"{name} must be positive")
    if non_native_selection_count > selection_count:
        raise ValueError("non_native_selection_count cannot exceed selection_count")
    if distinct_reason_count > non_native_selection_count:
        raise ValueError("distinct_reason_count cannot exceed non_native_selection_count")
    if changed_selection_count > selection_count:
        raise ValueError("changed_selection_count cannot exceed selection_count")
    if directory_fanout <= 1:
        raise ValueError("directory_fanout must be greater than one")

    old_reason_entry_allocations = recompute_count * distinct_reason_count
    retained_reason_vector_allocations = distinct_reason_count
    old_reason_aggregation_operations = recompute_count * non_native_selection_count
    retained_reason_aggregation_operations = old_reason_aggregation_operations
    selection_segment_count = ceil_div(selection_count, selection_segment_size)
    changed_selection_segment_count = min(
        changed_selection_count,
        selection_segment_count,
    )
    selection_directory_depth = persistent_directory_depth_for(
        selection_segment_count,
        directory_fanout,
    )
    historical_selection_clone_work = selection_count * recompute_count
    persistent_selection_clone_work = (
        min(
            selection_count,
            changed_selection_segment_count * selection_segment_size,
        )
        * recompute_count
    )
    persistent_directory_node_clone_work = (
        changed_selection_segment_count
        * selection_directory_depth
        * recompute_count
    )

    return {
        "schema": "zircon.runtime.ui_layout_report_aggregation_pressure.v2",
        "inputs": {
            "selection_count": selection_count,
            "non_native_selection_count": non_native_selection_count,
            "distinct_reason_count": distinct_reason_count,
            "recompute_count": recompute_count,
            "changed_selection_count": changed_selection_count,
            "selection_segment_size": selection_segment_size,
            "directory_fanout": directory_fanout,
            "selection_payload_bytes": selection_payload_bytes,
        },
        "historical_temporary_btree_map": {
            "reason_entry_allocations": old_reason_entry_allocations,
            "aggregation_operations": old_reason_aggregation_operations,
            "complexity": (
                "O(R * N * log(K)) with fresh map entries per recompute"
            ),
        },
        "retained_sorted_reason_vector": {
            "reason_entry_allocations": retained_reason_vector_allocations,
            "aggregation_operations": retained_reason_aggregation_operations,
            "complexity": (
                "O(R * N * log(K)) with one retained vector capacity"
            ),
        },
        "historical_flat_selection_vector": {
            "selection_clone_work": historical_selection_clone_work,
            "selection_payload_clone_bytes": (
                historical_selection_clone_work * selection_payload_bytes
            ),
            "complexity": "O(R * N) selection clones at layout-frame publication",
        },
        "persistent_segmented_selection_sequence": {
            "selection_segment_count": selection_segment_count,
            "changed_selection_segment_count_upper_bound": (
                changed_selection_segment_count
            ),
            "selection_clone_work": persistent_selection_clone_work,
            "selection_payload_clone_bytes": (
                persistent_selection_clone_work * selection_payload_bytes
            ),
            "directory_depth": selection_directory_depth,
            "directory_node_clone_work": persistent_directory_node_clone_work,
            "publication_handle_clone_count": recompute_count,
            "residual_reason_entry_clone_work": (
                distinct_reason_count * recompute_count
            ),
            "residual_reason_vector_allocation_count": recompute_count,
            "complexity": (
                "O(R * (changed segments * segment size + directory depth)) "
                "mutation work, O(R) publication handles, and O(R * K) "
                "bounded fallback-reason copies"
            ),
        },
        "delta": {
            "avoided_reason_entry_allocations": (
                old_reason_entry_allocations - retained_reason_vector_allocations
            ),
            "reason_entry_allocation_reduction_ratio": (
                old_reason_entry_allocations / retained_reason_vector_allocations
            ),
            "aggregation_operation_count_unchanged": (
                old_reason_aggregation_operations
                == retained_reason_aggregation_operations
            ),
            "avoided_selection_clone_work": (
                historical_selection_clone_work - persistent_selection_clone_work
            ),
            "avoided_selection_payload_clone_bytes": (
                (historical_selection_clone_work - persistent_selection_clone_work)
                * selection_payload_bytes
            ),
            "selection_clone_work_reduction_ratio": (
                historical_selection_clone_work / persistent_selection_clone_work
            ),
        },
        "interpretation": {
            "timing_claim": False,
            "included": (
                "temporary fallback-reason container entry allocation and flat "
                "selection publication copies versus retained sorted-vector and "
                "persistent segmented selection capacity"
            ),
            "excluded": (
                "CPU timing, allocator latency, cache locality, RSS, and "
                "product frame latency"
            ),
        },
    }


def ceil_div(value: int, divisor: int) -> int:
    return (value + divisor - 1) // divisor


def persistent_directory_depth_for(segment_count: int, fanout: int) -> int:
    depth = 1
    capacity = fanout
    while capacity < segment_count:
        capacity *= fanout
        depth += 1
    return depth


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--selection-count", type=int, default=10_000)
    parser.add_argument("--non-native-selection-count", type=int, default=10_000)
    parser.add_argument("--distinct-reason-count", type=int, default=8)
    parser.add_argument("--recompute-count", type=int, default=1_000)
    parser.add_argument("--changed-selection-count", type=int, default=1)
    parser.add_argument("--selection-segment-size", type=int, default=64)
    parser.add_argument("--directory-fanout", type=int, default=32)
    parser.add_argument("--selection-payload-bytes", type=int, default=128)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        selection_count=args.selection_count,
        non_native_selection_count=args.non_native_selection_count,
        distinct_reason_count=args.distinct_reason_count,
        recompute_count=args.recompute_count,
        changed_selection_count=args.changed_selection_count,
        selection_segment_size=args.selection_segment_size,
        directory_fanout=args.directory_fanout,
        selection_payload_bytes=args.selection_payload_bytes,
    )
    result["source_binding"] = build_source_binding(Path(__file__).resolve().parents[1])
    encoded = json.dumps(result, indent=2, sort_keys=True) + "\n"
    if args.output is not None:
        validate_output_path(args.output)
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded, encoding="utf-8")
    print(encoded, end="")


if __name__ == "__main__":
    main()
