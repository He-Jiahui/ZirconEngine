import sys
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_editor/src/ui/template_runtime/runtime/pane_payload_projection.rs"
SURFACE_SOURCE = (
    ROOT
    / "zircon_editor/src/ui/template_runtime/runtime/runtime_host/dynamic_control_state.rs"
)
PROJECTION_SOURCE = ROOT / "zircon_editor/src/ui/template_runtime/runtime/projection.rs"

sys.path.insert(0, str(ROOT / "tools"))
from ui_component_patch_index_pressure import run


class EditorTemplateComponentPatchIndexPerformanceContract(unittest.TestCase):
    def test_component_patches_are_grouped_before_the_projection_walk(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        injector = source.split("fn inject_template_v2_component_patches", 1)[1].split(
            "fn ui_value_to_toml", 1
        )[0]

        self.assertIn("patches_by_control", injector)
        self.assertIn("apply_component_projection_patches", injector)
        self.assertNotIn("apply_component_projection_patch(root, patch)", injector)

    def test_patch_application_keeps_a_single_recursive_walk_helper(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        self.assertIn("patches.remove(control_id)", source)
        self.assertIn("apply_component_projection_patches(child, patches)", source)
        self.assertIn("if patches.is_empty()", source)
        self.assertIn("for child in &mut node.children", source)
        self.assertNotIn("return true;", source)

    def test_surface_control_updates_build_one_control_index(self) -> None:
        source = SURFACE_SOURCE.read_text(encoding="utf-8")
        host_function = source.split(
            "pub(super) fn apply_template_control_attributes_to_host_model", 1
        )[1].split(
            "pub(super) fn apply_template_control_attributes_to_surface", 1
        )[0]
        self.assertIn("control_node_indices", host_function)
        self.assertIn(".get(control_id)", host_function)
        self.assertNotIn("host_model.nodes.iter_mut().filter", host_function)

        function = source.split(
            "pub(super) fn apply_template_control_attributes_to_surface", 1
        )[1].split("fn apply_template_control_property", 1)[0]
        self.assertIn("control_node_ids", function)
        self.assertIn("control_node_ids", function)
        self.assertIn(".get(control_id)", function)
        self.assertNotIn("surface.tree.nodes.iter().filter_map", function)

    def test_rebound_action_bindings_do_not_clone_the_full_control_table(self) -> None:
        source = SURFACE_SOURCE.read_text(encoding="utf-8")
        function = source.split("pub(crate) fn bind_template_actions_for_pane", 1)[1].split(
            "pub(crate) fn update_template_action_control_state", 1
        )[0]
        self.assertIn("BTreeMap::new()", function)
        self.assertNotIn("control_attributes.clone()", function)

    def test_pane_attributes_are_built_once_and_moved_to_the_slot_anchor(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        self.assertIn(") -> BTreeMap<String, Value>", source)
        self.assertIn("append_hybrid_slot_anchor_projection(&mut projection.root, body, pane_attributes)", SURFACE_SOURCE.read_text(encoding="utf-8"))
        anchor = source.split("pub(super) fn append_hybrid_slot_anchor_projection", 1)[1]
        self.assertNotIn("pane_body_attributes(body)", anchor)

    def test_action_resolution_indexes_controls_without_cloning_all_attributes(self) -> None:
        source = PROJECTION_SOURCE.read_text(encoding="utf-8")
        resolver = source.split("fn resolve_template_actions", 1)[1].split(
            "pub(super) fn resolve_template_action", 1
        )[0]
        self.assertIn("control_indices", resolver)
        self.assertIn("let resolved_actions = nodes", resolver)
        self.assertIn("nodes.get(*index)", resolver)
        self.assertNotIn("node.attributes.clone()", resolver)

    def test_pressure_model_preserves_first_match_and_removes_patch_times_tree_scan(self) -> None:
        result = run(node_count=128, patch_count=64)
        self.assertTrue(result["first_match_semantics_match"])
        self.assertGreater(result["scan_reduction_ratio"], 20)
        self.assertEqual(result["unmatched_patch_count"], 6)


if __name__ == "__main__":
    unittest.main()
