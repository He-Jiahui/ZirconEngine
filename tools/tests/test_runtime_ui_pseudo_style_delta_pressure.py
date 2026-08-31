from pathlib import Path
import unittest

from tools.runtime_ui_pseudo_style_delta_pressure import (
    pressure_report,
    validate_output_path,
)


ROOT = Path(__file__).resolve().parents[2]
STYLE = ROOT / "zircon_runtime/src/ui/v2/style.rs"
STATE_INVALIDATION = ROOT / (
    "zircon_runtime/src/ui/surface/surface/pointer_component_events/"
    "state_invalidation.rs"
)
PROPERTY_TRANSACTION = ROOT / (
    "zircon_runtime/src/ui/surface/surface/property_transaction.rs"
)
UNREAL_INVALIDATION_ROOT = ROOT / (
    "dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/"
    "SlateInvalidationRoot.cpp"
)


class RuntimeUiPseudoStyleDeltaPressureTests(unittest.TestCase):
    def test_self_hover_counts_complete_map_reconstruction(self):
        report = pressure_report(1_000, 1, 1, 24, 8, 8, 4, 3)

        current = report["current_runtime_style_recompute"]
        target = report["published_pseudo_style_delta"]
        self.assertEqual(current["subtree_or_node_visits"], 1_000)
        self.assertEqual(current["full_map_entry_copies"], 46_000)
        self.assertEqual(current["operation_units"], 51_000)
        self.assertEqual(target["operation_units"], 4_000)
        self.assertEqual(target["full_map_entry_copies"], 0)
        self.assertEqual(report["comparison"]["operation_reduction_ratio"], 12.75)
        self.assertFalse(report["is_product_timing"])

    def test_sparse_ancestor_dependency_avoids_unaffected_subtree(self):
        report = pressure_report(1_000, 10_000, 64, 24, 8, 8, 4, 3)

        current = report["current_runtime_style_recompute"]
        target = report["published_pseudo_style_delta"]
        self.assertEqual(current["subtree_or_node_visits"], 10_000_000)
        self.assertEqual(current["full_map_entry_copies"], 460_000_000)
        self.assertEqual(current["operation_units"], 510_000_000)
        self.assertEqual(target["affected_dependency_visits"], 64_000)
        self.assertEqual(target["operation_units"], 256_000)
        self.assertGreater(
            report["comparison"]["operation_reduction_ratio"], 1_900.0
        )

    def test_dense_descendant_case_remains_linear_in_real_affected_nodes(self):
        report = pressure_report(1_000, 10_000, 10_000, 24, 8, 8, 4, 3)

        self.assertEqual(
            report["published_pseudo_style_delta"]["affected_dependency_visits"],
            10_000_000,
        )
        self.assertEqual(
            report["published_pseudo_style_delta"]["operation_units"],
            40_000_000,
        )
        self.assertEqual(report["comparison"]["operation_reduction_ratio"], 12.75)

    def test_rejects_invalid_inputs(self):
        for values in (
            (0, 1, 1, 1, 1, 1, 1, 1),
            (1, 0, 1, 1, 1, 1, 1, 1),
            (1, 1, 0, 1, 1, 1, 1, 1),
            (1, 1, 2, 1, 1, 1, 1, 1),
        ):
            with self.subTest(values=values):
                with self.assertRaises(ValueError):
                    pressure_report(*values)

    def test_artifact_output_rejects_the_system_drive(self):
        with self.assertRaises(ValueError):
            validate_output_path(r"C:\zircon-profiles\pseudo-style.json")
        self.assertEqual(
            validate_output_path(r"E:\zircon-profiles\pseudo-style.json").drive.upper(),
            "E:",
        )

    def test_model_is_bound_to_current_zircon_style_recompute(self):
        style = STYLE.read_text(encoding="utf-8")
        invalidation = STATE_INVALIDATION.read_text(encoding="utf-8")
        transaction = PROPERTY_TRANSACTION.read_text(encoding="utf-8")

        self.assertIn("apply_to_tree_subtree", style)
        self.assertIn("let mut stack = vec![RuntimeStyleFrame", style)
        self.assertIn("let mut next_attributes = base_attributes.clone()", style)
        self.assertIn("node_style.self_values.clone()", style)
        self.assertIn("base_style_overrides", style)
        self.assertIn("base_style_tokens", style)
        self.assertIn("node_state_can_affect_descendants", invalidation)
        self.assertIn("apply_runtime_state_style_subtree", invalidation)
        self.assertIn("mark_component_state_render_dirty(node_id)", transaction)

    def test_unreal_keeps_reasoned_widget_invalidation_update_lists(self):
        unreal = UNREAL_INVALIDATION_ROOT.read_text(encoding="utf-8")
        self.assertIn("FSlateInvalidationRoot::InvalidateWidget", unreal)
        self.assertIn("EInvalidateWidgetReason::Paint", unreal)
        self.assertIn("FSlateInvalidationRoot::ProcessAttributeUpdate", unreal)
        self.assertIn("FinalUpdateList", unreal)


if __name__ == "__main__":
    unittest.main()
