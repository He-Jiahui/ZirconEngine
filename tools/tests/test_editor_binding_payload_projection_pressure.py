import unittest

from tools.editor_binding_payload_projection_pressure import model_pressure


class EditorBindingPayloadProjectionPressureTests(unittest.TestCase):
    def test_nested_payload_eliminates_recursive_value_clones(self) -> None:
        result = model_pressure(
            projection_count=4096,
            root_field_count=16,
            branching_factor=4,
            depth=5,
        )

        self.assertEqual(result["payload"]["value_nodes_per_projection"], 21_840)
        self.assertEqual(
            result["retired_owned_projection"][
                "value_node_clone_operations_per_projection"
            ],
            167_473,
        )
        self.assertEqual(
            result["delta"]["eliminated_value_node_clone_operations"],
            685_969_408,
        )
        self.assertGreater(
            result["delta"]["retired_clone_to_borrowed_visit_ratio"], 3.8
        )
        self.assertEqual(
            result["borrowed_projection"]["value_node_clone_operations"], 0
        )

    def test_empty_projection_set_has_finite_zero_work(self) -> None:
        result = model_pressure(projection_count=0, root_field_count=0)

        self.assertEqual(
            result["delta"]["eliminated_value_node_clone_operations"], 0
        )
        self.assertEqual(
            result["delta"]["retired_clone_to_borrowed_visit_ratio"], 0.0
        )


if __name__ == "__main__":
    unittest.main()
