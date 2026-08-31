import argparse
import json
from pathlib import Path
from typing import Any


def run(
    *,
    slot_count: int,
    scroll_update_count: int,
    large_seek_count: int,
) -> dict[str, Any]:
    if slot_count < 0 or scroll_update_count < 0 or large_seek_count < 0:
        raise ValueError("counts must be non-negative")
    if large_seek_count > scroll_update_count:
        raise ValueError("large_seek_count cannot exceed scroll_update_count")

    one_row_scroll_count = scroll_update_count - large_seek_count
    owner_generation_token_refreshes = slot_count * scroll_update_count
    changed_slot_token_refreshes = (
        one_row_scroll_count + large_seek_count * slot_count
    )

    return {
        "schema_version": 1,
        "scope": "virtual_list_assignment_identity_model_only",
        "slot_count": slot_count,
        "scroll_update_count": scroll_update_count,
        "large_seek_count": large_seek_count,
        "one_row_scroll_count": one_row_scroll_count,
        "owner_generation_token_refreshes": owner_generation_token_refreshes,
        "changed_slot_token_refreshes": changed_slot_token_refreshes,
        "preserved_unchanged_slot_tokens": (
            owner_generation_token_refreshes - changed_slot_token_refreshes
        ),
        "token_refresh_reduction_ratio": _ratio(
            owner_generation_token_refreshes, changed_slot_token_refreshes
        ),
        "runtime_cpu_measured": False,
        "allocator_or_rss_measured": False,
        "accesskit_adapter_wired": False,
    }


def _ratio(retired: int, retained: int) -> float | None:
    if retained == 0:
        return None
    return round(retired / retained, 2)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--slot-count", type=int, default=41)
    parser.add_argument("--scroll-update-count", type=int, default=4_096)
    parser.add_argument("--large-seek-count", type=int, default=64)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()

    result = run(
        slot_count=args.slot_count,
        scroll_update_count=args.scroll_update_count,
        large_seek_count=args.large_seek_count,
    )
    encoded = json.dumps(result, indent=2, sort_keys=True) + "\n"
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded, encoding="utf-8")
    print(encoded, end="")


if __name__ == "__main__":
    main()
