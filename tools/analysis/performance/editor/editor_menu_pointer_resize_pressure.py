"""Model retained shell-pointer topology versus resize-time full rebuilds."""

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
    "zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_surface_delta.rs",
    "zircon_editor/src/ui/retained_host/menu_pointer/build_host_menu_pointer_layout.rs",
    "zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_layout.rs",
    "zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/sync.rs",
    "zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/rebuild_surface.rs",
    "zircon_editor/src/ui/retained_host/activity_rail_pointer/sync.rs",
    "zircon_editor/src/ui/retained_host/activity_rail_pointer/rebuild_surface.rs",
    "zircon_editor/src/ui/retained_host/activity_rail_pointer/surface_delta.rs",
    "zircon_editor/src/ui/retained_host/app/pointer_layout/shell_chrome.rs",
    "zircon_editor/src/ui/retained_host/activity_rail_pointer/build_host_activity_rail_pointer_layout.rs",
    "zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute_viewport.rs",
    "zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/pointer_surfaces.rs",
    "zircon_editor/src/ui/retained_host/app/pointer_layout/welcome_recent.rs",
    "dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp",
)

SOURCE_GUARDS = {
    SOURCE_PATHS[0]: (
        "build_host_menu_pointer_geometry_layout(",
        "window_metrics_target",
        "self.menu_pointer_bridge.sync_shared(",
    ),
    SOURCE_PATHS[1]: (
        "let layout_changed =",
        "popup_semantics_changed",
        "self.apply_surface_delta();",
    ),
    SOURCE_PATHS[2]: (
        "rebuild_surface_from_scratch",
        "let mut surface = UiSurface::new(",
        "surface.rebuild_authored_frames(",
        "self.surface = surface;",
        "self.dispatcher = dispatcher;",
    ),
    SOURCE_PATHS[3]: (
        "MenuPointerSurfaceDelta::NoChange",
        "MenuPointerSurfaceDelta::Geometry",
        "MenuPointerSurfaceDelta::Topology",
        "publish_authored_geometry(",
        "expected_nodes.len() != self.surface.tree.nodes.len()",
    ),
    SOURCE_PATHS[4]: (
        "pub(crate) fn build_host_menu_pointer_geometry_layout(",
        "Arc::clone(&previous.menus)",
        "Arc::clone(&previous.intrinsic_popup_widths)",
        "ui.menu_pointer.geometry_product_build_count",
    ),
    SOURCE_PATHS[5]: (
        "Arc<[Vec<MenuItemSpec>]>",
        "intrinsic_popup_widths: Arc<[f32]>",
        "popup_semantics_equal",
    ),
    SOURCE_PATHS[6]: (
        "ViewportToolbarSurfaceDelta::Geometry(changes)",
        "topology_changed",
    ),
    SOURCE_PATHS[7]: (
        "publish_authored_geometry(",
        "ViewportToolbarSurfaceDelta::Topology",
    ),
    SOURCE_PATHS[8]: (
        "surface_delta_for_layout",
        "ActivityRailSurfaceDelta::Geometry(changes)",
    ),
    SOURCE_PATHS[9]: (
        "publish_authored_geometry(",
        "rebuild_authored_frames(",
        "surface_geometry_authority_miss_count",
    ),
    SOURCE_PATHS[10]: (
        "ActivityRailSurfaceDelta::NoChange",
        "ActivityRailSurfaceDelta::Geometry",
        "ActivityRailSurfaceDelta::Topology",
        "strip_node_count",
    ),
    SOURCE_PATHS[11]: (
        "sync_activity_rail_pointer_layout_for_target",
        "build_host_activity_rail_pointer_geometry_layout(",
        "window_metrics_target",
    ),
    SOURCE_PATHS[12]: (
        "pub(crate) fn build_host_activity_rail_pointer_geometry_layout(",
        "Arc::clone(&previous.left_tabs)",
        "Arc::clone(&previous.right_tabs)",
        "ui.activity_rail.geometry_product_build_count",
    ),
    SOURCE_PATHS[13]: (
        "window_metrics_target: bool",
        "sync_activity_rail_pointer_layout_for_target(model, window_metrics_target)",
        "ui.window_metrics.semantic_pointer_receipt_reuse_count",
    ),
    SOURCE_PATHS[14]: (
        "sync_welcome_recent_pointer_size()",
        "ui.window_metrics.welcome_recent_semantic_reuse_count",
        "if window_metrics_target {",
    ),
    SOURCE_PATHS[15]: (
        "fn sync_welcome_recent_pointer_size",
        "sync_viewport(viewport)",
    ),
    SOURCE_PATHS[16]: (
        "FSlateInvalidationRoot::PaintFastPath(",
        "FSlateInvalidationRoot::ProcessInvalidation()",
    ),
}


def run(
    *,
    resize_step_count: int = 200,
    hover_state_change_count: int = 10_000,
    menu_button_count: int = 7,
    open_popup_item_count: int = 40,
    open_submenu_depth: int = 2,
    changed_geometry_node_count: int = 3,
    activity_rail_tab_count: int = 4,
    activity_rail_changed_geometry_node_count: int = 5,
    host_page_receipt_item_count: int = 5,
    document_tab_receipt_item_count: int = 4,
    drawer_header_receipt_item_count: int = 4,
    welcome_recent_item_count: int = 8,
) -> dict[str, Any]:
    for name, value in (
        ("resize_step_count", resize_step_count),
        ("hover_state_change_count", hover_state_change_count),
        ("menu_button_count", menu_button_count),
        ("open_popup_item_count", open_popup_item_count),
        ("changed_geometry_node_count", changed_geometry_node_count),
        ("activity_rail_tab_count", activity_rail_tab_count),
        (
            "activity_rail_changed_geometry_node_count",
            activity_rail_changed_geometry_node_count,
        ),
    ):
        if value <= 0:
            raise ValueError(f"{name} must be positive")
    if open_submenu_depth < 0:
        raise ValueError("open_submenu_depth must be non-negative")
    for name, value in (
        ("host_page_receipt_item_count", host_page_receipt_item_count),
        ("document_tab_receipt_item_count", document_tab_receipt_item_count),
        ("drawer_header_receipt_item_count", drawer_header_receipt_item_count),
        ("welcome_recent_item_count", welcome_recent_item_count),
    ):
        if value < 0:
            raise ValueError(f"{name} must be non-negative")

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
    full_rebuild_domain_count = 5
    geometry_patch_domain_count = 5
    current_node_domain_visit_units = (
        resize_step_count * surface_node_count * full_rebuild_domain_count
    )
    retained_node_domain_visit_units = (
        resize_step_count
        * changed_geometry_node_count
        * geometry_patch_domain_count
    )
    current_registration_count = resize_step_count * handled_node_count
    activity_rail_strip_count = 2
    activity_rail_surface_node_count = (
        root_node_count + activity_rail_strip_count + activity_rail_tab_count
    )
    if activity_rail_changed_geometry_node_count > activity_rail_surface_node_count:
        raise ValueError(
            "activity_rail_changed_geometry_node_count cannot exceed "
            "activity_rail_surface_node_count"
        )
    activity_rail_handled_node_count = activity_rail_strip_count + activity_rail_tab_count
    activity_rail_legacy_node_domain_visit_units = (
        resize_step_count
        * activity_rail_surface_node_count
        * full_rebuild_domain_count
    )
    activity_rail_retained_node_domain_visit_units = (
        resize_step_count
        * activity_rail_changed_geometry_node_count
        * geometry_patch_domain_count
    )
    activity_rail_legacy_registration_count = (
        resize_step_count * activity_rail_handled_node_count
    )
    semantic_pointer_receipt_type_count = 3
    semantic_pointer_receipt_item_count = (
        host_page_receipt_item_count
        + document_tab_receipt_item_count
        + drawer_header_receipt_item_count
    )

    return {
        "schema": "zircon.editor.menu_pointer_resize_pressure.v7",
        "inputs": {
            "resize_step_count": resize_step_count,
            "hover_state_change_count": hover_state_change_count,
            "menu_button_count": menu_button_count,
            "open_popup_item_count": open_popup_item_count,
            "open_submenu_depth": open_submenu_depth,
            "changed_geometry_node_count": changed_geometry_node_count,
            "activity_rail_tab_count": activity_rail_tab_count,
            "activity_rail_changed_geometry_node_count": (
                activity_rail_changed_geometry_node_count
            ),
            "host_page_receipt_item_count": host_page_receipt_item_count,
            "document_tab_receipt_item_count": document_tab_receipt_item_count,
            "drawer_header_receipt_item_count": drawer_header_receipt_item_count,
            "welcome_recent_item_count": welcome_recent_item_count,
        },
        "derived": {
            "popup_layer_count": popup_layer_count,
            "surface_node_count": surface_node_count,
            "handled_node_count": handled_node_count,
            "full_rebuild_domain_count": full_rebuild_domain_count,
            "geometry_patch_domain_count": geometry_patch_domain_count,
            "activity_rail_surface_node_count": activity_rail_surface_node_count,
            "activity_rail_handled_node_count": activity_rail_handled_node_count,
            "semantic_pointer_receipt_type_count": semantic_pointer_receipt_type_count,
            "semantic_pointer_receipt_item_count": semantic_pointer_receipt_item_count,
        },
        "legacy_full_rebuild": {
            "surface_build_count": resize_step_count,
            "semantic_layout_build_count": resize_step_count,
            "popup_text_row_measurement_count": (
                resize_step_count * open_popup_item_count
            ),
            "node_domain_visit_units": current_node_domain_visit_units,
            "dispatcher_registration_count": current_registration_count,
            "route_intent_binding_count": current_registration_count,
            "route_path_string_build_count": current_registration_count,
            "popup_item_projection_count": resize_step_count * open_popup_item_count,
            "complexity": "O(R * (surface nodes + popup items))",
            "hover_surface_build_count": hover_state_change_count,
            "hover_dispatcher_registration_count": (
                hover_state_change_count * handled_node_count
            ),
            "hover_route_intent_binding_count": (
                hover_state_change_count * handled_node_count
            ),
        },
        "current_retained_geometry_patch": {
            "surface_build_count": 0,
            "semantic_layout_build_count": 0,
            "geometry_layout_build_count": resize_step_count,
            "popup_text_row_measurement_count": 0,
            "node_domain_visit_units": retained_node_domain_visit_units,
            "dispatcher_registration_count": 0,
            "route_intent_binding_count": 0,
            "route_path_string_build_count": 0,
            "popup_item_projection_count": 0,
            "complexity": (
                "O(R * (menu buttons + popup widths + changed geometry nodes))"
            ),
            "hover_surface_build_count": 0,
            "hover_geometry_publication_count": 0,
            "hover_popup_item_comparison_count": 0,
            "hover_complexity": "O(1) surface-state gate",
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
            "avoided_semantic_layout_build_count": resize_step_count,
            "avoided_popup_text_row_measurement_count": (
                resize_step_count * open_popup_item_count
            ),
            "avoided_hover_surface_build_count": hover_state_change_count,
            "avoided_hover_dispatcher_registration_count": (
                hover_state_change_count * handled_node_count
            ),
            "avoided_hover_route_intent_binding_count": (
                hover_state_change_count * handled_node_count
            ),
        },
        "activity_rail_resize": {
            "legacy_full_rebuild": {
                "surface_build_count": resize_step_count,
                "semantic_product_build_count": resize_step_count,
                "semantic_tab_projection_count": (
                    resize_step_count * activity_rail_tab_count
                ),
                "node_domain_visit_units": activity_rail_legacy_node_domain_visit_units,
                "dispatcher_registration_count": activity_rail_legacy_registration_count,
                "route_intent_binding_count": activity_rail_legacy_registration_count,
                "route_path_string_build_count": activity_rail_legacy_registration_count,
                "complexity": "O(R * activity rail surface nodes)",
            },
            "current_retained_geometry_patch": {
                "surface_build_count": 0,
                "semantic_product_build_count": 0,
                "geometry_product_build_count": resize_step_count,
                "semantic_tab_projection_count": 0,
                "node_domain_visit_units": activity_rail_retained_node_domain_visit_units,
                "dispatcher_registration_count": 0,
                "route_intent_binding_count": 0,
                "route_path_string_build_count": 0,
                "complexity": "O(R * changed activity rail geometry nodes)",
            },
            "delta": {
                "avoided_surface_build_count": resize_step_count,
                "avoided_semantic_product_build_count": resize_step_count,
                "avoided_semantic_tab_projection_count": (
                    resize_step_count * activity_rail_tab_count
                ),
                "avoided_node_domain_visit_units": (
                    activity_rail_legacy_node_domain_visit_units
                    - activity_rail_retained_node_domain_visit_units
                ),
                "node_domain_visit_reduction_ratio": (
                    activity_rail_legacy_node_domain_visit_units
                    / activity_rail_retained_node_domain_visit_units
                ),
                "avoided_dispatcher_registration_count": (
                    activity_rail_legacy_registration_count
                ),
                "avoided_route_intent_binding_count": (
                    activity_rail_legacy_registration_count
                ),
                "avoided_route_path_string_build_count": (
                    activity_rail_legacy_registration_count
                ),
            },
        },
        "semantic_pointer_receipt_resize": {
            "legacy_projection": {
                "receipt_product_build_count": (
                    resize_step_count * semantic_pointer_receipt_type_count
                ),
                "receipt_item_projection_count": (
                    resize_step_count * semantic_pointer_receipt_item_count
                ),
                "complexity": "O(R * semantic pointer receipt items)",
            },
            "current_retained_receipts": {
                "receipt_product_build_count": 0,
                "receipt_item_projection_count": 0,
                "reuse_counter_count": (
                    resize_step_count * semantic_pointer_receipt_type_count
                ),
                "complexity": "O(1) WindowMetrics target gate",
            },
            "delta": {
                "avoided_receipt_product_build_count": (
                    resize_step_count * semantic_pointer_receipt_type_count
                ),
                "avoided_receipt_item_projection_count": (
                    resize_step_count * semantic_pointer_receipt_item_count
                ),
            },
        },
        "welcome_recent_resize": {
            "legacy_projection": {
                "recent_path_clone_count": resize_step_count,
                "recent_item_projection_count": resize_step_count * welcome_recent_item_count,
                "complexity": "O(R * recent project paths)",
            },
            "current_viewport_fast_path": {
                "recent_path_clone_count": 0,
                "recent_item_projection_count": 0,
                "viewport_sync_count": resize_step_count,
                "complexity": "O(1) viewport-only update",
            },
            "delta": {
                "avoided_recent_path_clone_count": resize_step_count,
                "avoided_recent_item_projection_count": resize_step_count * welcome_recent_item_count,
            },
        },
        "acceptance_contract": {
            "topology_change": "full surface rebuild is allowed",
            "geometry_only_resize": (
                "retain UiSurface, dispatcher, and route intents; publish only changed "
                "authored geometry"
            ),
            "stable_layout": "zero surface rebuild and zero geometry publication",
            "geometry_only_resize_semantics": (
                "reuse menu, preset, and intrinsic-width Arc products; rebuild only "
                "button/popup geometry"
            ),
            "required_product_counters": [
                "menu_pointer.surface_delta_topology_count",
                "menu_pointer.surface_delta_geometry_count",
                "menu_pointer.surface_geometry_patch_node_count",
                "menu_pointer.surface_geometry_fallback_count",
                "ui.activity_rail.surface_geometry_patch_node_count",
                "ui.activity_rail.surface_geometry_local_publication_count",
                "ui.activity_rail.surface_geometry_fallback_count",
                "ui.activity_rail.semantic_product_build_count",
                "ui.activity_rail.geometry_product_build_count",
                "ui.window_metrics.semantic_pointer_receipt_reuse_count",
                "ui.window_metrics.welcome_recent_semantic_reuse_count",
            ],
        },
        "interpretation": {
            "timing_claim": False,
            "included": (
                "deterministic topology rebuild, registration, route-binding, "
                "semantic projection, popup projection, and node-domain visit units"
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
        payload = path.read_bytes()
        sources[relative_path] = payload.decode("utf-8")
        critical_source_files.append(
            {
                "relative_path": relative_path,
                "byte_length": len(payload),
                "sha256": hashlib.sha256(payload).hexdigest().upper(),
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
    parser.add_argument("--repo-root", type=Path, default=Path(__file__).resolve().parents[4])
    parser.add_argument("--resize-step-count", type=int, default=200)
    parser.add_argument("--hover-state-change-count", type=int, default=10_000)
    parser.add_argument("--menu-button-count", type=int, default=7)
    parser.add_argument("--open-popup-item-count", type=int, default=40)
    parser.add_argument("--open-submenu-depth", type=int, default=2)
    parser.add_argument("--changed-geometry-node-count", type=int, default=3)
    parser.add_argument("--activity-rail-tab-count", type=int, default=4)
    parser.add_argument(
        "--activity-rail-changed-geometry-node-count", type=int, default=5
    )
    parser.add_argument("--host-page-receipt-item-count", type=int, default=5)
    parser.add_argument("--document-tab-receipt-item-count", type=int, default=4)
    parser.add_argument("--drawer-header-receipt-item-count", type=int, default=4)
    parser.add_argument("--welcome-recent-item-count", type=int, default=8)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    result = run(
        resize_step_count=args.resize_step_count,
        hover_state_change_count=args.hover_state_change_count,
        menu_button_count=args.menu_button_count,
        open_popup_item_count=args.open_popup_item_count,
        open_submenu_depth=args.open_submenu_depth,
        changed_geometry_node_count=args.changed_geometry_node_count,
        activity_rail_tab_count=args.activity_rail_tab_count,
        activity_rail_changed_geometry_node_count=(
            args.activity_rail_changed_geometry_node_count
        ),
        host_page_receipt_item_count=args.host_page_receipt_item_count,
        document_tab_receipt_item_count=args.document_tab_receipt_item_count,
        drawer_header_receipt_item_count=args.drawer_header_receipt_item_count,
        welcome_recent_item_count=args.welcome_recent_item_count,
    )
    result["source_binding"] = build_source_binding(args.repo_root)
    result["ready"] = result["source_binding"]["ready"]
    write_result(args.output, result)
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0 if result["ready"] else 2


if __name__ == "__main__":
    raise SystemExit(main())
