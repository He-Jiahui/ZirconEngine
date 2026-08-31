import unittest

from tools.runtime_ui_segmented_submission_pressure import run


class RuntimeUiSegmentedSubmissionPressureContract(unittest.TestCase):
    def test_model_separates_clone_publication_and_renderer_work(self) -> None:
        result = run()

        self.assertEqual(
            result["legacy_flat_aggregate_submission"]["changed_surface_command_clones"],
            16_777_216,
        )
        self.assertEqual(
            result["legacy_flat_aggregate_submission"]["aggregate_flat_command_clones"],
            1_073_741_824,
        )
        self.assertEqual(
            result["legacy_flat_aggregate_submission"]["combined_command_clone_events"],
            1_090_519_040,
        )
        self.assertEqual(
            result["persistent_command_submission"]["surface_command_clones"],
            262_144,
        )
        self.assertEqual(
            result["persistent_command_submission"]["surface_directory_node_clones"],
            8_192,
        )
        self.assertEqual(
            result["persistent_command_submission"]["runtime_node_id_projection_command_clones"],
            0,
        )
        self.assertEqual(
            result["persistent_command_submission"]["segment_handle_publications"],
            262_144,
        )
        self.assertEqual(
            result["persistent_command_submission"]["required_renderer_command_visits"],
            1_073_741_824,
        )
        self.assertEqual(
            result["delta"]["surface_publication_command_clone_reduction_ratio"],
            64.0,
        )
        self.assertEqual(
            result["delta"]["legacy_total_command_clone_reduction_ratio"],
            4_160.0,
        )
        self.assertEqual(result["delta"]["renderer_command_visit_reduction"], 0)

    def test_model_scales_with_multiple_changed_surfaces(self) -> None:
        result = run(
            update_count=2,
            surface_count=8,
            commands_per_surface=16,
            changed_surface_count=2,
        )

        self.assertEqual(
            result["persistent_command_submission"]["surface_command_clones"], 64
        )
        self.assertEqual(result["delta"]["avoided_command_clone_events"], 256)
        self.assertEqual(
            result["delta"]["surface_publication_command_clone_reduction_ratio"],
            1.0,
        )

    def test_model_rejects_non_positive_or_impossible_inputs(self) -> None:
        with self.assertRaises(ValueError):
            run(update_count=0)
        with self.assertRaises(ValueError):
            run(surface_count=2, changed_surface_count=3)
        with self.assertRaises(ValueError):
            run(commands_per_surface=2, changed_commands_per_surface=3)


if __name__ == "__main__":
    unittest.main()
