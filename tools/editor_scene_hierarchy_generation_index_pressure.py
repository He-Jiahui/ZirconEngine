import argparse
import json
import math
from pathlib import Path


def run(
    *,
    row_count: int = 100_000,
    average_display_name_utf8_bytes: int = 24,
    selection_only_patch_count: int = 65_536,
) -> dict[str, object]:
    if row_count < 1:
        raise ValueError("row_count must be positive")
    if average_display_name_utf8_bytes < 0:
        raise ValueError("average_display_name_utf8_bytes must be non-negative")
    if selection_only_patch_count < 0:
        raise ValueError("selection_only_patch_count must be non-negative")

    ordered_index_work_per_row = math.ceil(math.log2(row_count + 1))
    old_index_work = row_count * ordered_index_work_per_row
    new_index_work = row_count
    full_reflow_name_payload = row_count * average_display_name_utf8_bytes
    selection_name_payload = (
        selection_only_patch_count * average_display_name_utf8_bytes
    )

    return {
        "schema_version": 1,
        "inputs": {
            "row_count": row_count,
            "average_display_name_utf8_bytes": average_display_name_utf8_bytes,
            "selection_only_patch_count": selection_only_patch_count,
        },
        "retired_ordered_duplicate_projection": {
            "modeled_full_reflow_index_work_units": old_index_work,
            "projection_owned_display_name_count": row_count,
            "projection_owned_display_name_utf8_payload_bytes": full_reflow_name_payload,
            "selection_only_patch_name_clone_count": selection_only_patch_count,
            "selection_only_patch_name_utf8_payload_bytes": selection_name_payload,
        },
        "generation_identity_index": {
            "modeled_full_reflow_index_work_units": new_index_work,
            "projection_owned_display_name_count": 0,
            "projection_owned_display_name_utf8_payload_bytes": 0,
            "selection_only_patch_name_clone_count": 0,
            "selection_only_patch_name_utf8_payload_bytes": 0,
        },
        "delta": {
            "modeled_index_work_ratio": round(old_index_work / new_index_work, 2),
            "avoided_projection_display_name_count": row_count,
            "avoided_projection_display_name_utf8_payload_bytes": full_reflow_name_payload,
            "avoided_selection_only_name_clone_count": selection_only_patch_count,
            "avoided_selection_only_name_utf8_payload_bytes": selection_name_payload,
        },
        "model_notes": [
            "Ordered-index work uses ceil(log2(N + 1)) units per row; it is an algorithm model, not measured BTree comparisons.",
            "Hash-index work uses one expected probe unit per row; adversarial collision cost is not modeled.",
            "String figures count UTF-8 payload only and exclude String headers, allocator metadata, capacity slack, and SharedString internals.",
            "The model is not CPU time, RSS, allocator traffic, input latency, or GPU evidence.",
        ],
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--row-count", type=int, default=100_000)
    parser.add_argument("--average-display-name-utf8-bytes", type=int, default=24)
    parser.add_argument("--selection-only-patch-count", type=int, default=65_536)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        row_count=args.row_count,
        average_display_name_utf8_bytes=args.average_display_name_utf8_bytes,
        selection_only_patch_count=args.selection_only_patch_count,
    )
    encoded = json.dumps(result, indent=2, sort_keys=True) + "\n"
    if args.output is None:
        print(encoded, end="")
    else:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded, encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
