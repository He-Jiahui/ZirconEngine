#!/usr/bin/env python3
"""Model Asset Browser selection and context-menu lookup work.

This is a deterministic operation model, not product timing.
"""

import argparse
import json
from pathlib import Path, PureWindowsPath


def pressure_report(
    assets: int,
    selection_lookups: int,
    context_menu_hits: int,
) -> dict[str, object]:
    if min(assets, selection_lookups, context_menu_hits) < 0:
        raise ValueError("pressure inputs must be non-negative")

    current_selection_scan = assets * selection_lookups
    target_selection_index = 3 * selection_lookups
    current_context_scan = assets * context_menu_hits
    target_context_index = 2 * context_menu_hits
    current_total = current_selection_scan + current_context_scan
    target_total = target_selection_index + target_context_index

    return {
        "model": "asset_selection_and_context_lookup",
        "timing_claim": False,
        "inputs": {
            "assets": assets,
            "selection_lookups": selection_lookups,
            "context_menu_hits": context_menu_hits,
        },
        "current": {
            "selection_visible_asset_scan_units": current_selection_scan,
            "context_menu_visible_asset_scan_units": current_context_scan,
            "total_lookup_units": current_total,
        },
        "target": {
            "selection_index_units": target_selection_index,
            "context_menu_index_units": target_context_index,
            "total_lookup_units": target_total,
        },
        "ratios": {
            "total_lookup_units": current_total / target_total
            if target_total
            else None,
        },
        "invariants": {
            "selection_preserves_display_order": True,
            "context_menu_reuses_uuid_index": True,
            "stale_generation_rejects_without_scan": True,
        },
    }


def _output_path(value: str) -> Path:
    windows = PureWindowsPath(value)
    if windows.drive.upper() == "C:":
        raise argparse.ArgumentTypeError("profile artifacts must not be written to C:")
    return Path(value)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--assets", type=int, default=100_000)
    parser.add_argument("--selection-lookups", type=int, default=1_000)
    parser.add_argument("--context-menu-hits", type=int, default=1_000)
    parser.add_argument("--output", required=True, type=_output_path)
    args = parser.parse_args()
    report = pressure_report(
        args.assets,
        args.selection_lookups,
        args.context_menu_hits,
    )
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
