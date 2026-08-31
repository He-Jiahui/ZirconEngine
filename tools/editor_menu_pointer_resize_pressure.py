"""Model retained menu-pointer topology versus resize-time full rebuilds."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from pathlib import Path
from typing import Any


SOURCE_PATHS = (
    "zircon_editor/src/ui/retained_host/app/pointer_layout/menu.rs",
    "zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_sync.rs",
    "zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_rebuild_surface.rs",
    "zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/sync.rs",
    "zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/rebuild_surface.rs",
    "dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp",
)

SOURCE_GUARDS = {
    SOURCE_PATHS[0]: (
        "build_host_menu_pointer_layout(",
        "self.menu_pointer_bridge.sync_shared(",
    ),
    SOURCE_PATHS[1]: (
        "let layout_changed =",
        "self.rebuild_surface();",
    ),
    SOURCE_PATHS[2]: (
        "let mut surface = UiSurface::new(",
        "surface.rebuild();",
        "self.surface = surface;",
        "self.dispatcher = dispatcher;",
    ),
    SOURCE_PATHS[3]: (
        "ViewportToolbarSurfaceDelta::Geometry(changes)",
        "topology_changed",
    ),
    SOURCE_PATHS[4]: (
        "publish_authored_geometry(",
        "ViewportToolbarSurfaceDelta::Topology",
    ),
    SOURCE_PATHS[5]: (
        "FSlateInvalidationRoot::PaintFastPath(",
        "FSlateInvalidationRoot::ProcessInvalidation()",
    ),
}


def run(
    *,
    resize_step_count: int = 200,
    menu_button_count: int = 7,
    open_popup_item_count: int = 40,
    open_submenu_depth: int = 2,
    changed_geometry_node_count: int = 3,
) -> dict[str, Any]:
    for name, value in (
        ("resize_step_count", resize_step_count),
        ("menu_button_count", menu_button_count),
        ("open_popup_item_count", open_popup_item_count),
        ("changed_geometry_node_count", changed_geometry_node_count),
    ):
        if value <= 0:
            raise ValueError(f"{name} must be positive")
    if open_submenu_depth < 0:
        raise ValueError("open_submenu_depth must be non-negative")

    popup_layer_count = 1 + open_submenu_depth
    dismiss_node_count = 1
    root_node_count = 1
    surface_node_count = (
        root_node_count
        + menu_button_count
        + dismiss_node_count
        + popup_layer_count
    )
    if changed_geometry_node_count > surface_node_count:
        raise ValueError("changed_geometry_node_count cannot exceed surface_node_count")

    handled_node_count = menu_button_count + dismiss_node_count + popup_layer_count
    full_rebuild_domain_count = 4
    geometry_patch_domain_count = 3
    current_node_domain_visit_units = (
        resize_step_count * surface_node_count * full_rebuild_domain_count
    )
    retained_node_domain_visit_units = (
        resize_step_count
        * changed_geometry_node_count
        * geometry_patch_domain_count
    )
    current_registration_count = resize_step_count * handled_node_count

    return {
        "schema": "zircon.editor.menu_pointer_resize_pressure.v1",
        "inputs": {
            "resize_step_count": resize_step_count,
            "menu_button_count": menu_button_count,
            "open_popup_item_count": open_popup_item_count,
            "open_submenu_depth": open_submenu_depth,
            "changed_geometry_node_count": changed_geometry_node_count,
        },
        "derived": {
            "popup_layer_count": popup_layer_count,
            "surface_node_count": surface_node_count,
            "handled_node_count": handled_node_count,
            "full_rebuild_domain_count": full_rebuild_domain_count,
            "geometry_patch_domain_count": geometry_patch_domain_count,
        },
        "current_full_rebuild": {
            "surface_build_count": resize_step_count,
            "node_domain_visit_units": current_node_domain_visit_units,
            "dispatcher_registration_count": current_registration_count,
            "route_intent_binding_count": current_registration_count,
            "route_path_string_build_count": current_registration_count,
            "popup_item_projection_count": resize_step_count * open_popup_item_count,
            "complexity": "O(R * (surface nodes + popup items))",
        },
        "retained_geometry_patch": {
            "surface_build_count": 0,
            "node_domain_visit_units": retained_node_domain_visit_units,
            "dispatcher_registration_count": 0,
            "route_intent_binding_count": 0,
            "route_path_string_build_count": 0,
            "popup_item_projection_count": 0,
            "complexity": "O(R * changed geometry nodes)",
        },
        "delta": {
            "avoided_surface_build_count": resize_step_count,
            "avoided_node_domain_visit_units": (
                current_node_domain_visit_units - retained_node_domain_visit_units
            ),
            "node_domain_visit_reduction_ratio": (
                current_node_domain_visit_units / retained_node_domain_visit_units
            ),
            "avoided_dispatcher_registration_count": current_registration_count,
            "avoided_route_intent_binding_count": current_registration_count,
            "avoided_route_path_string_build_count": current_registration_count,
            "avoided_popup_item_projection_count": (
                resize_step_count * open_popup_item_count
            ),
        },
        "acceptance_contract": {
            "topology_change": "full surface rebuild is allowed",
            "geometry_only_resize": (
                "retain UiSurface, dispatcher, and route intents; publish only changed "
                "authored geometry"
            ),
            "stable_layout": "zero surface rebuild and zero geometry publication",
            "required_product_counters": [
                "menu_pointer.surface_delta_topology_count",
                "menu_pointer.surface_delta_geometry_count",
                "menu_pointer.surface_geometry_patch_node_count",
                "menu_pointer.surface_geometry_fallback_count",
            ],
        },
        "interpretation": {
            "timing_claim": False,
            "included": (
                "deterministic topology rebuild, registration, route-binding, "
                "popup projection, and node-domain visit units"
            ),
            "excluded": (
                "CPU time, allocator time, hit-test latency, frame latency, RSS, "
                "GPU time, and platform event coalescing"
            ),
        },
    }


def validate_source_contract(sources: dict[str, str]) -> dict[str, Any]:
    blockers: list[dict[str, str]] = []
    for relative_path in SOURCE_PATHS:
        source = sources.get(relative_path)
        if source is None:
            blockers.append(
                {"code": "missing_critical_source", "relative_path": relative_path}
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


def build_source_binding(repo_root: Path) -> dict[str, Any]:
    root = repo_root.resolve()
    sources: dict[str, str] = {}
    critical_source_files: list[dict[str, Any]] = []
    for relative_path in SOURCE_PATHS:
        path = root / relative_path
        if not path.is_file():
            continue
        sources[relative_path] = path.read_text(encoding="utf-8")
        critical_source_files.append(
            {
                "relative_path": relative_path,
                "byte_length": path.stat().st_size,
                "sha256": hashlib.sha256(path.read_bytes()).hexdigest().upper(),
            }
        )
    contract = validate_source_contract(sources)
    revision = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=root,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    manifest = "\n".join(
        f'{entry["relative_path"]}={entry["sha256"]}'
        for entry in critical_source_files
    ).encode("utf-8")
    return {
        "ready": contract["ready"],
        "blockers": contract["blockers"],
        "git_revision": revision,
        "critical_source_files": critical_source_files,
        "source_manifest_sha256": hashlib.sha256(manifest).hexdigest().upper(),
    }


def write_result(output: Path, result: dict[str, Any]) -> None:
    if output.drive.upper() not in {"D:", "E:", "F:"}:
        raise ValueError("profile artifacts must be written to D:, E:, or F:")
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(
        json.dumps(result, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--resize-step-count", type=int, default=200)
    parser.add_argument("--menu-button-count", type=int, default=7)
    parser.add_argument("--open-popup-item-count", type=int, default=40)
    parser.add_argument("--open-submenu-depth", type=int, default=2)
    parser.add_argument("--changed-geometry-node-count", type=int, default=3)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    result = run(
        resize_step_count=args.resize_step_count,
        menu_button_count=args.menu_button_count,
        open_popup_item_count=args.open_popup_item_count,
        open_submenu_depth=args.open_submenu_depth,
        changed_geometry_node_count=args.changed_geometry_node_count,
    )
    result["source_binding"] = build_source_binding(args.repo_root)
    result["ready"] = result["source_binding"]["ready"]
    write_result(args.output, result)
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0 if result["ready"] else 2


if __name__ == "__main__":
    raise SystemExit(main())
