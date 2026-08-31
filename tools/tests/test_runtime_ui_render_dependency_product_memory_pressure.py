from pathlib import Path
import unittest

from tools.runtime_ui_render_dependency_product_memory_pressure import (
    run,
    validate_output_path,
)


class RuntimeUiRenderDependencyProductMemoryPressureTests(unittest.TestCase):
    def test_default_model_retains_one_source_payload_and_bounded_metadata(self):
        result = run()

        self.assertEqual(
            result["shared_source_payload"]["image_vertex_payload_bytes"], 196_608
        )
        self.assertEqual(
            result["shared_source_payload"]["text_glyph_payload_bytes"], 1_572_864
        )
        self.assertEqual(
            result["shared_source_payload"]["total_payload_bytes"], 1_769_472
        )
        self.assertEqual(
            result["target_dependency_product"]["global_binding_metadata_bytes"],
            10_240,
        )
        self.assertEqual(
            result["target_dependency_product"]["base_generation_metadata_bytes"],
            15_328,
        )
        self.assertEqual(
            result["target_dependency_product"]["delta_generation_metadata_bytes"],
            392,
        )
        self.assertEqual(
            result["target_dependency_product"]["total_metadata_bytes"], 26_352
        )
        self.assertEqual(
            result["target_dependency_product"][
                "retained_delta_source_payload_bytes"
            ],
            55_296,
        )
        self.assertEqual(
            result["target_dependency_product"]["total_retained_bytes"], 1_851_120
        )
        self.assertTrue(result["target_dependency_product"]["within_metadata_budget"])

    def test_retained_memory_is_independent_of_present_count(self):
        baseline = run(present_count=4_096)
        long_run = run(present_count=10_000_000)

        self.assertEqual(
            baseline["target_dependency_product"]["total_retained_bytes"],
            long_run["target_dependency_product"]["total_retained_bytes"],
        )
        self.assertEqual(
            baseline["target_dependency_product"]["total_metadata_bytes"],
            long_run["target_dependency_product"]["total_metadata_bytes"],
        )

    def test_each_retained_delta_generation_adds_only_one_changed_path(self):
        three = run(retained_generation_count=3)
        four = run(retained_generation_count=4, max_retained_generation_count=4)

        self.assertEqual(
            four["target_dependency_product"]["total_metadata_bytes"]
            - three["target_dependency_product"]["total_metadata_bytes"],
            three["target_dependency_product"]["delta_generation_metadata_bytes"],
        )
        self.assertEqual(
            four["target_dependency_product"]["total_retained_bytes"]
            - three["target_dependency_product"]["total_retained_bytes"],
            three["target_dependency_product"]["delta_generation_retained_bytes"],
        )

    def test_multi_segment_delta_bounds_each_changed_payload_and_directory_path(self):
        one = run(changed_segments_per_delta_generation=1)
        two = run(changed_segments_per_delta_generation=2)

        self.assertEqual(
            two["target_dependency_product"]["delta_source_payload_bytes"],
            2 * one["target_dependency_product"]["delta_source_payload_bytes"],
        )
        self.assertEqual(
            two["target_dependency_product"]["delta_directory_path_bytes"],
            2 * one["target_dependency_product"]["delta_directory_path_bytes"],
        )

    def test_single_segment_directory_delta_copies_only_the_leaf(self):
        result = run(
            segment_count=1,
            image_dependencies_per_segment=1,
            unique_image_binding_count=1,
        )

        self.assertEqual(result["inputs"]["directory_depth"], 0)
        self.assertEqual(
            result["target_dependency_product"]["delta_directory_path_bytes"],
            result["inputs"]["persistent_directory_node_bytes"],
        )

    def test_stress_scale_remains_under_explicit_metadata_budget(self):
        result = run(
            segment_count=4_096,
            unique_image_binding_count=1_024,
            retained_generation_count=3,
            metadata_budget_bytes=8 * 1024 * 1024,
        )

        self.assertEqual(
            result["target_dependency_product"]["total_metadata_bytes"], 1_148_016
        )
        self.assertTrue(result["target_dependency_product"]["within_metadata_budget"])
        self.assertGreater(
            result["target_dependency_product"]["metadata_budget_headroom_bytes"],
            6 * 1024 * 1024,
        )

    def test_rejected_full_generation_clone_exposes_payload_multiplication(self):
        result = run()

        self.assertEqual(
            result["rejected_full_generation_clone"]["retained_payload_bytes"],
            5_308_416,
        )
        self.assertEqual(
            result["delta"]["avoided_payload_duplication_bytes"], 3_483_648
        )

    def test_model_rejects_invalid_cardinality_or_budget(self):
        invalid_calls = (
            {"present_count": 0},
            {"segment_count": 0},
            {"image_dependencies_per_segment": 0},
            {"unique_image_binding_count": 0},
            {"segment_count": 2, "image_dependencies_per_segment": 1, "unique_image_binding_count": 3},
            {"image_batches_per_segment": 0},
            {"text_run_spans_per_segment": 0},
            {"text_glyph_instances_per_segment": 0},
            {"retained_generation_count": 0},
            {"retained_generation_count": 4},
            {"changed_segments_per_delta_generation": 0},
            {"segment_count": 2, "changed_segments_per_delta_generation": 3},
            {"metadata_budget_bytes": 0},
        )
        for kwargs in invalid_calls:
            with self.subTest(kwargs=kwargs):
                with self.assertRaises(ValueError):
                    run(**kwargs)

    def test_output_artifacts_are_restricted_to_d_e_or_f(self):
        for path in (
            Path("D:/profiles/memory.json"),
            Path("E:/profiles/memory.json"),
            Path("F:/profiles/memory.json"),
        ):
            with self.subTest(path=path):
                self.assertEqual(validate_output_path(path), path)
        for path in (Path("C:/profiles/memory.json"), Path("memory.json")):
            with self.subTest(path=path):
                with self.assertRaises(ValueError):
                    validate_output_path(path)

    def test_interpretation_keeps_opaque_gpu_and_real_rss_unclaimed(self):
        result = run()

        self.assertFalse(result["interpretation"]["rss_measured"])
        self.assertFalse(result["interpretation"]["gpu_resident_bytes_measured"])
        self.assertIn("wgpu::BindGroup", result["interpretation"]["excluded"])
        self.assertIn("private", result["interpretation"]["required_product_evidence"])


if __name__ == "__main__":
    unittest.main()
