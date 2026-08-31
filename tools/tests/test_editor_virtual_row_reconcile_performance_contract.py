from pathlib import Path
import unittest

from tools.editor_virtual_row_reconcile_pressure import run


REPO_ROOT = Path(__file__).resolve().parents[2]
VIRTUAL_ROWS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/"
    "template_bridge/virtual_rows.rs"
)
COMPONENT_ROWS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/"
    "template_bridge/workbench/component_property_rows.rs"
)
DATA_SYNC = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/"
    "template_bridge/workbench/data_sync.rs"
)
COMPONENTIZED_WINDOW = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/"
    "template_bridge/workbench/componentized_window.rs"
)
POINTER_DISPATCH = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/workbench/pointer.rs"
)
PRODUCT_INSPECTOR_SCROLL = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/app/detail_scroll_pointer/inspector.rs"
)
RUNTIME_EVENT_ROUTING = REPO_ROOT / (
    "zircon_runtime/src/ui/surface/surface/event_routing.rs"
)
INSPECTOR_ASSET = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/shell/"
    "workbench_inspector_panel.zui"
)
PROFILE_MANIFEST = REPO_ROOT / "tools/profile-capture-manifest.ps1"


class EditorVirtualRowReconcilePerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = VIRTUAL_ROWS.read_text(encoding="utf-8")
        cls.implementation = cls.source.split("#[cfg(test)]", 1)[0]
        cls.reconcile = cls.implementation.split(
            "pub(crate) fn reconcile_with_keys", 1
        )[1]
        cls.reconcile = cls.reconcile.split("pub(crate) fn bindings", 1)[0]

    def test_reconcile_delegates_to_surface_owned_bounded_materialization(self) -> None:
        self.assertIn("reconcile_virtual_list_materialization_with_keys", self.reconcile)
        self.assertIn("ensure_virtual_list_prototype_slots", self.reconcile)
        self.assertIn("UiVirtualListMaterializationChange", self.implementation)
        self.assertNotIn("required_virtual_count", self.reconcile)
        self.assertNotIn("0..total_row_count", self.reconcile)

    def test_editor_bridge_has_no_retained_tree_inventory_or_logical_growth_loop(self) -> None:
        self.assertNotIn("surface.tree.nodes.iter()", self.implementation)
        self.assertNotIn("surface.tree.nodes.values()", self.implementation)
        self.assertNotIn("fn inventory(", self.implementation)
        self.assertNotIn("fn next_node_id(", self.implementation)
        self.assertNotIn("insert_or_reuse_pooled_child", self.implementation)

    def test_rust_regression_guards_runtime_materialization_authority(self) -> None:
        self.assertIn(
            "virtual_row_reconcile_uses_runtime_materialization_authority",
            self.source,
        )

    def test_inspector_product_path_uses_one_virtualized_prototype(self) -> None:
        asset = INSPECTOR_ASSET.read_text(encoding="utf-8")
        rows = COMPONENT_ROWS.read_text(encoding="utf-8")
        data_sync = DATA_SYNC.read_text(encoding="utf-8")

        self.assertIn('control_id = "WorkbenchInspectorMeshProperties"', asset)
        self.assertIn(
            'virtualization = { item_extent = 28.0, overscan = 2 }', asset
        )
        self.assertIn('authored_count = 1', asset)
        self.assertIn(
            'children = [{ node = "component_property_slot_04_row" }]', asset
        )
        self.assertIn("component_property_row_bindings", rows)
        self.assertIn("component_property_item_key", rows)
        self.assertIn("let index = binding.logical_index;", data_sync)
        self.assertNotIn("component_property_row_control_ids", data_sync)

    def test_scroll_routes_full_event_and_rebinds_only_changed_slots(self) -> None:
        component_rows = COMPONENT_ROWS.read_text(encoding="utf-8")
        componentized_window = COMPONENTIZED_WINDOW.read_text(encoding="utf-8")
        pointer_dispatch = POINTER_DISPATCH.read_text(encoding="utf-8")
        runtime_event_routing = RUNTIME_EVENT_ROUTING.read_text(encoding="utf-8")

        self.assertIn("route_pointer_input_event(event)", componentized_window)
        self.assertIn("apply_default_pointer_scroll(&route)", componentized_window)
        self.assertIn(
            "refresh_component_property_rows_after_scroll(&route)",
            pointer_dispatch,
        )
        self.assertIn("bindings_for_changes", component_rows)
        self.assertIn("bindings_for_changes", self.implementation)
        scroll_rebind = component_rows.split(
            "refresh_component_property_rows_after_scroll", 1
        )[1].split("fn reconcile_component_property_rows", 1)[0]
        self.assertIn("component_property_keys.clone()", scroll_rebind)
        self.assertNotIn("component_property_item_key(", scroll_rebind)
        self.assertIn("pub fn route_pointer_input_event", runtime_event_routing)
        self.assertIn("event.scroll_delta", runtime_event_routing)
        self.assertIn("pub fn apply_default_pointer_scroll", runtime_event_routing)

        product_scroll = PRODUCT_INSPECTOR_SCROLL.read_text(encoding="utf-8")
        self.assertIn("route_workbench_inspector_scroll", product_scroll)
        self.assertIn(".route_pointer_event(event)", product_scroll)
        self.assertIn(
            "refresh_component_property_rows_after_scroll", product_scroll
        )
        self.assertIn("self.apply_dispatch_effects(effects)", product_scroll)

    def test_pressure_model_is_bounded_by_physical_slots(self) -> None:
        result = run(
            reconcile_count=4096,
            surface_node_count=16384,
            physical_slot_capacity=41,
        )

        self.assertEqual(
            result["retired_shared_inventory"]["retained_tree_node_visits"],
            134217728,
        )
        self.assertEqual(
            result["prototype_slot_pool"]["retained_tree_node_visits"],
            0,
        )
        self.assertEqual(
            result["prototype_slot_pool"]["physical_slot_binding_visits"],
            167936,
        )
        self.assertEqual(
            result["prototype_slot_pool"]["changed_slot_metadata_rebinds"],
            4096,
        )
        self.assertEqual(
            result["delta"]["avoided_retained_tree_node_visits"],
            134217728,
        )
        self.assertEqual(result["delta"]["logical_growth_nodes_created"], 0)
        self.assertEqual(
            result["delta"]["avoided_unchanged_slot_metadata_rebinds"],
            163840,
        )
        self.assertGreater(result["delta"]["retained_tree_to_slot_visit_ratio"], 799.0)

    def test_profile_manifest_binds_virtual_row_reconciliation(self) -> None:
        manifest = PROFILE_MANIFEST.read_text(encoding="utf-8")
        for path in (
            "zircon_editor/src/ui/retained_host/callback_dispatch/"
            "template_bridge/virtual_rows.rs",
            "zircon_editor/src/ui/retained_host/callback_dispatch/"
            "template_bridge/workbench/component_property_rows.rs",
            "zircon_editor/src/ui/retained_host/app/detail_scroll_pointer/inspector.rs",
            "zircon_editor/assets/ui/editor/components/workbench/shell/"
            "workbench_inspector_panel.zui",
            "zircon_runtime/src/ui/surface/virtual_list_materialization.rs",
            "zircon_runtime/src/ui/surface/virtual_list_prototype_pool.rs",
            "zircon_runtime/src/ui/layout/pass/virtual_list_layout.rs",
        ):
            self.assertIn(f'"{path}"', manifest)


if __name__ == "__main__":
    unittest.main()
