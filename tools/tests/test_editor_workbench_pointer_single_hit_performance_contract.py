from pathlib import Path
import unittest

from tools.editor_workbench_pointer_single_hit_pressure import run


ROOT = Path(__file__).resolve().parents[2]


class EditorWorkbenchPointerSingleHitPerformanceContract(unittest.TestCase):
    def test_surface_projection_publishes_pointer_identity_and_tooltip_eligibility(self):
        host_nodes = (ROOT / "zircon_editor/src/ui/template_runtime/host_nodes.rs").read_text()
        projection = (
            ROOT / "zircon_editor/src/ui/template_runtime/runtime/projection.rs"
        ).read_text()
        adapter = (
            ROOT / "zircon_editor/src/ui/template_runtime/retained_adapter.rs"
        ).read_text()

        self.assertIn("pub surface_node_id: Option<UiNodeId>", host_nodes)
        self.assertIn("pub has_workbench_icon_tooltip: bool", host_nodes)
        self.assertIn("surface_node_id: Some(node_id)", projection)
        self.assertIn(
            "has_workbench_icon_tooltip: workbench_icon_tooltip_text(metadata).is_some()",
            projection,
        )
        self.assertIn("surface_node_id: node.surface_node_id", adapter)
        self.assertIn(
            "has_workbench_icon_tooltip: node.has_workbench_icon_tooltip", adapter
        )

    def test_move_index_is_a_hover_superset_without_relaxing_press_dispatch(self):
        dispatch = (
            ROOT
            / "zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/surface_frame_builder/dispatch.rs"
        ).read_text()
        index = (
            ROOT
            / "zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/index.rs"
        ).read_text()
        hit = (
            ROOT
            / "zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/hit.rs"
        ).read_text()

        self.assertIn("fn accepts_pointer_move", dispatch)
        self.assertIn("node.has_workbench_icon_tooltip", dispatch)
        self.assertIn("accepts_pointer_move(node)", index)
        self.assertIn("is_dispatchable(node) && template_node_accepts_point", hit)
        self.assertIn("accepts_node(node) && template_node_accepts_point", hit)
        self.assertIn("is_dispatchable)?", hit)
        self.assertIn("accepts_pointer_move,", hit)

    def test_native_move_returns_the_single_hit_identity_to_tooltip_observation(self):
        pointer = (
            ROOT
            / "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/events/pointer.rs"
        ).read_text()
        entry = (
            ROOT
            / "zircon_editor/src/ui/retained_host/host_contract/native_pointer/move_dispatch/entry.rs"
        ).read_text()
        tooltip = (
            ROOT
            / "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/icon_tooltip.rs"
        ).read_text()
        callbacks = (
            ROOT
            / "zircon_editor/src/ui/retained_host/host_contract/globals/callbacks/host.rs"
        ).read_text()

        native_dispatch = pointer.index("dispatch_native_pointer_move")
        tooltip_observation = pointer.index("invoke_workbench_pointer_input")
        self.assertLess(native_dispatch, tooltip_observation)
        self.assertIn("Option<WorkbenchTooltipPointerTarget>", pointer)
        self.assertIn("Option<WorkbenchTooltipPointerTarget>", entry)
        self.assertIn("WorkbenchTooltipPointerTarget::SurfaceNode", tooltip)
        self.assertIn("WorkbenchTooltipPointerTarget::HostChrome", tooltip)
        self.assertIn(
            "Option<Callback2<UiPointerInputEvent, Option<WorkbenchTooltipPointerTarget>>>",
            callbacks,
        )
        self.assertNotIn("surface.hit_test(point)", tooltip)

    def test_pressure_model_counts_only_the_removed_duplicate_query(self):
        result = run(pointer_move_count=65_536)

        self.assertEqual(
            result["retired_double_hit_path"]["total_spatial_queries"], 131_072
        )
        self.assertEqual(
            result["published_identity_single_hit_path"]["total_spatial_queries"],
            65_536,
        )
        self.assertEqual(result["delta"]["avoided_spatial_queries"], 65_536)
        self.assertEqual(result["delta"]["spatial_query_reduction_ratio"], 2.0)
        self.assertTrue(result["scope"]["latest_value_event_coalescing_implemented"])


if __name__ == "__main__":
    unittest.main()
