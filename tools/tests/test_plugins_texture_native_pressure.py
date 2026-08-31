import unittest
from pathlib import Path

from tools.plugins_texture_native_pressure import run


ROOT = Path(__file__).resolve().parents[2]
KAISER_SOURCE = ROOT / "zircon_plugins/texture_importer/runtime/src/mipgen/kernel.rs"
NATIVE_SOURCE = ROOT / "zircon_plugins/native_dynamic_fixture/native/src/lib.rs"
NATIVE_TESTS = ROOT / "zircon_plugins/native_dynamic_fixture/native/src/tests.rs"


class PluginsTextureNativePressureTests(unittest.TestCase):
    def test_kaiser_axis_cache_eliminates_more_than_ninety_nine_percent_of_weights(
        self,
    ) -> None:
        kaiser = run()["kaiser_axis_cache"]

        self.assertEqual(kaiser["baseline_target_texel_count"], 16_384)
        self.assertEqual(kaiser["candidate_target_texel_count"], 16_384)
        self.assertEqual(kaiser["baseline_weight_evaluation_count"], 487_305)
        self.assertEqual(kaiser["candidate_weight_evaluation_count"], 1_274)
        self.assertGreater(kaiser["weight_evaluation_reduction_percent"], 99.7)
        self.assertEqual(kaiser["baseline_normalizer_evaluation_count"], 1)
        self.assertEqual(kaiser["candidate_normalizer_evaluation_count"], 1)

    def test_kaiser_source_keeps_cache_and_release_contract(self) -> None:
        source = KAISER_SOURCE.read_text(encoding="utf-8")

        self.assertIn("build_kaiser_axis_weights", source)
        self.assertIn("TEXTURE_KAISER_AXIS_CACHE_BENCH_V1", source)
        self.assertIn("assert_eq!(axis_weight_evaluations, (487_305, 1_274))", source)

    def test_bounded_response_halves_full_buffers_and_eliminates_source_clone(
        self,
    ) -> None:
        native = run()["native_response"]

        self.assertEqual(native["baseline_full_response_buffer_count"], 16)
        self.assertEqual(native["candidate_full_response_buffer_count"], 8)
        self.assertEqual(native["full_response_buffer_reduction_percent"], 50.0)
        self.assertEqual(native["baseline_source_text_clone_bytes"], 1_048_808)
        self.assertEqual(native["candidate_source_text_clone_bytes"], 0)
        self.assertEqual(native["baseline_intermediate_metadata_buffer_count"], 8)
        self.assertEqual(native["candidate_intermediate_metadata_buffer_count"], 0)

    def test_native_source_keeps_checked_budgets_and_bounded_writer(self) -> None:
        source = NATIVE_SOURCE.read_text(encoding="utf-8")
        tests = NATIVE_TESTS.read_text(encoding="utf-8")

        self.assertIn("const MAX_IMPORT_METADATA_BYTES: usize = 64 * 1024", source)
        self.assertIn("const MAX_IMPORT_SOURCE_BYTES: usize = 256 * 1024", source)
        self.assertIn("struct BoundedResponseWriter", source)
        self.assertIn("checked_add(bytes.len())", source)
        self.assertIn("PERF_RESULT plugins20_bounded_native_import_response", tests)

    def test_release_acceptance_is_explicit_and_timing_remains_pending(self) -> None:
        acceptance = run()["acceptance"]

        self.assertEqual(acceptance["sample_order"], "alternating")
        self.assertEqual(acceptance["percentile_method"], "nearest_rank")
        self.assertEqual(
            acceptance["kaiser_candidate_p95_maximum_legacy_ratio"], 0.25
        )
        self.assertEqual(
            acceptance["native_candidate_p95_maximum_legacy_ratio"], 1.10
        )
        self.assertTrue(acceptance["release_timing_pending"])

    def test_model_is_bound_to_current_and_head_sources(self) -> None:
        binding = run()["source_binding"]

        self.assertEqual(
            binding["git_revision"],
            "ca3ac3cc6ad218d04a5cd469447cea2452441321",
        )
        self.assertEqual(len(binding["source_sha256"]), 4)
        self.assertEqual(len(binding["source_manifest_sha256"]), 64)

    def test_rejects_invalid_workloads(self) -> None:
        with self.assertRaises(ValueError):
            run(target_texels=0)
        with self.assertRaises(ValueError):
            run(legacy_kaiser_weight_evaluations=0)
        with self.assertRaises(ValueError):
            run(cached_kaiser_weight_evaluations=0)
        with self.assertRaises(ValueError):
            run(native_source_bytes=0)
        with self.assertRaises(ValueError):
            run(native_encodes_per_sample=0)
        with self.assertRaises(ValueError):
            run(
                legacy_kaiser_weight_evaluations=4,
                cached_kaiser_weight_evaluations=5,
            )


if __name__ == "__main__":
    unittest.main()
