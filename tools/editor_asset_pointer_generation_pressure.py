#!/usr/bin/env python3
"""Model Asset Browser pointer identity publication work.

This is a deterministic ownership/operation model, not product timing.
"""

import argparse
import json
from pathlib import Path, PureWindowsPath


def pressure_report(
    assets: int,
    folders: int,
    stable_syncs: int,
    routed_item_hits: int,
    surfaces: int = 2,
) -> dict[str, object]:
    if min(assets, folders, stable_syncs, routed_item_hits) < 0:
        raise ValueError("pressure inputs must be non-negative")
    if surfaces <= 0:
        raise ValueError("surfaces must be positive")

    current_uuid_payload_clones = assets * stable_syncs * surfaces
    current_uuid_identity_comparisons = assets * stable_syncs * surfaces
    # AssetWorkspaceItemGeneration currently owns five Arc-backed products.
    target_arc_handle_clones = 5 * stable_syncs * surfaces
    target_identity_comparisons = stable_syncs * surfaces
    current_item_identity_units = (
        current_uuid_payload_clones + current_uuid_identity_comparisons
    )
    target_item_identity_units = target_arc_handle_clones + target_identity_comparisons

    return {
        "model": "asset_pointer_generation_ownership",
        "timing_claim": False,
        "inputs": {
            "assets": assets,
            "folders": folders,
            "stable_syncs": stable_syncs,
            "routed_item_hits": routed_item_hits,
            "surfaces": surfaces,
        },
        "current": {
            "uuid_payload_clones": current_uuid_payload_clones,
            "uuid_identity_comparisons": current_uuid_identity_comparisons,
            "item_identity_operation_units": current_item_identity_units,
            "activity_folder_payload_clones": folders * stable_syncs,
            "routed_hit_uuid_payload_clones": routed_item_hits,
        },
        "target": {
            "uuid_payload_clones": 0,
            "uuid_identity_comparisons": 0,
            "generation_arc_handle_clones": target_arc_handle_clones,
            "generation_identity_comparisons": target_identity_comparisons,
            "item_identity_operation_units": target_item_identity_units,
            "activity_folder_payload_clones": folders * stable_syncs,
            "routed_hit_uuid_payload_clones": routed_item_hits,
        },
        "ratios": {
            "item_identity_operation_units": (
                current_item_identity_units / target_item_identity_units
                if target_item_identity_units
                else None
            )
        },
        "invariants": {
            "pointer_layout_reuses_workspace_item_generation": True,
            "stable_sync_uuid_payload_clones": 0,
            "hit_route_owns_only_the_hit_uuid": True,
            "activity_folder_projection_is_out_of_scope": True,
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
    parser.add_argument("--folders", type=int, default=256)
    parser.add_argument("--stable-syncs", type=int, default=1_000)
    parser.add_argument("--routed-item-hits", type=int, default=10_000)
    parser.add_argument("--surfaces", type=int, default=2)
    parser.add_argument("--output", required=True, type=_output_path)
    args = parser.parse_args()

    report = pressure_report(
        args.assets,
        args.folders,
        args.stable_syncs,
        args.routed_item_hits,
        args.surfaces,
    )
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
