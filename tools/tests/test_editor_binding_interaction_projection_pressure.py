import unittest

from tools.editor_binding_interaction_projection_pressure import model_pressure


class EditorBindingInteractionProjectionPressureTests(unittest.TestCase):
    def test_selected_authority_eliminates_full_inspector_projection_work(self) -> None:
        result = model_pressure(
            interactions_per_family=4096,
            binding_count=256,
            payload_entry_count=128,
            schema_item_count=256,
            event_option_count=16,
            action_kind_count=3,
            route_suggestion_count=4,
            action_suggestion_count=3,
            payload_suggestion_count=8,
        )

        self.assertEqual(result["retired"]["materialized_items"], 23_056_384)
        self.assertEqual(result["selected_authority"]["materialized_items"], 598_016)
        self.assertEqual(result["delta"]["eliminated_materialized_items"], 22_458_368)
        self.assertGreater(result["delta"]["work_reduction_ratio"], 38.0)

    def test_empty_interaction_set_has_finite_zero_work(self) -> None:
        result = model_pressure(interactions_per_family=0)

        self.assertEqual(result["retired"]["materialized_items"], 0)
        self.assertEqual(result["selected_authority"]["materialized_items"], 0)
        self.assertEqual(result["delta"]["work_reduction_ratio"], 0.0)


if __name__ == "__main__":
    unittest.main()
