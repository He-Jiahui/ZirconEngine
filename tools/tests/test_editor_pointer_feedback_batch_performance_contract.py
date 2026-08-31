from pathlib import Path
import unittest

from tools.editor_pointer_feedback_batch_pressure import run


ROOT = Path(__file__).resolve().parents[2]
POINTER = ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/workbench/pointer.rs"
)
FEEDBACK = ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/pointer_feedback.rs"
)
PRODUCT_TEST = ROOT / (
    "zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/"
    "workbench_pointer_preview.rs"
)


class EditorPointerFeedbackBatchPerformanceContractTests(unittest.TestCase):
    def test_pointer_dispatch_batches_feedback_before_activation(self) -> None:
        pointer = POINTER.read_text(encoding="utf-8")

        self.assertEqual(pointer.count("update_pointer_"), 4)
        self.assertEqual(pointer.count("update_text_input_pointer_feedback("), 1)
        self.assertNotIn("refresh_pointer_hover_feedback", pointer)
        self.assertNotIn("refresh_pointer_press_feedback", pointer)
        self.assertNotIn("refresh_pointer_range_feedback", pointer)
        activation = pointer.split(
            "if let Some((control_id, event_kind)) =", 1
        )[1]
        self.assertIn("if let Some(result) = dispatched", activation)
        self.assertIn("ui.workbench.pointer.transaction_count", pointer)
        self.assertIn(
            "ui.workbench.pointer.activation_coalesced_refresh_count", pointer
        )
        self.assertLess(
            activation.index("return Some(result);"),
            activation.index("let pointer_feedback_dirty = match"),
        )

    def test_feedback_stages_defer_surface_refresh_to_frame_commit(self) -> None:
        feedback = FEEDBACK.read_text(encoding="utf-8")

        self.assertNotIn("refresh_after_state_change(", feedback)
        self.assertNotIn("dirty_flags().any()", feedback)
        self.assertEqual(
            feedback.count("pending_invalidation_changed_node_count()"), 1
        )
        self.assertIn("ui.workbench.pointer.feedback_deferred_count", feedback)
        self.assertNotIn("refresh_dirty_pointer_feedback", feedback)

        search_clear = (FEEDBACK.parent / "search_clear_action.rs").read_text(
            encoding="utf-8"
        )
        self.assertNotIn("refresh_after_state_change(", search_clear)

    def test_slider_press_product_regression_requires_one_commit(self) -> None:
        product_test = PRODUCT_TEST.read_text(encoding="utf-8")

        self.assertIn("generation_before_press", product_test)
        self.assertIn(
            "one pointer event must publish press and range feedback in one invalidation commit",
            product_test,
        )
        self.assertIn(
            "one pointer event must publish release and activation state in one invalidation commit",
            product_test,
        )

    def test_pressure_model_removes_duplicate_dirty_summaries_and_refreshes(self) -> None:
        result = run(
            event_count=65_536,
            changed_stage_count=2,
            trailing_candidate_count=1,
        )

        self.assertEqual(result["old_dirty_summary_count"], 196_608)
        self.assertEqual(result["new_dirty_summary_count"], 0)
        self.assertEqual(result["avoided_dirty_summary_count"], 196_608)
        self.assertEqual(result["dirty_summary_elimination_percent"], 100.0)
        self.assertEqual(result["new_pending_invalidation_count_checks"], 65_536)
        self.assertEqual(result["old_surface_refresh_count"], 131_072)
        self.assertEqual(result["new_surface_refresh_count"], 65_536)
        self.assertEqual(result["avoided_surface_refresh_count"], 65_536)
        self.assertEqual(result["surface_refresh_reduction_ratio"], 2.0)


if __name__ == "__main__":
    unittest.main()
