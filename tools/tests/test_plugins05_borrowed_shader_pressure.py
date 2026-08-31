import unittest
from pathlib import Path

from tools.plugins05_borrowed_shader_pressure import run


ROOT = Path(__file__).resolve().parents[2]
CONTRACT_SOURCE = ROOT / "zircon_runtime/src/asset/importer/contract.rs"
WGSL_SOURCE = ROOT / "zircon_plugins/shader_wgsl_importer/runtime/src/lib.rs"
FAMILY_SOURCE = ROOT / "zircon_plugins/asset_importers/shader/runtime/src/lib.rs"


class Plugins05BorrowedShaderPressureTests(unittest.TestCase):
    def test_borrowed_validation_eliminates_all_source_clone_bytes(self) -> None:
        validation = run()["validation"]

        self.assertEqual(validation["baseline_source_clone_byte_count"], 33_554_432)
        self.assertEqual(validation["candidate_source_clone_byte_count"], 0)
        self.assertEqual(validation["source_clone_byte_reduction_percent"], 100.0)
        self.assertEqual(validation["baseline_source_clone_allocation_count"], 32)
        self.assertEqual(validation["candidate_source_clone_allocation_count"], 0)

    def test_parse_attempt_and_utf8_view_counts_are_preserved(self) -> None:
        validation = run()["validation"]

        self.assertEqual(validation["baseline_utf8_view_count"], 32)
        self.assertEqual(validation["candidate_utf8_view_count"], 32)
        self.assertEqual(validation["baseline_invalid_parse_attempt_count"], 32)
        self.assertEqual(validation["candidate_invalid_parse_attempt_count"], 32)

    def test_asset_context_success_path_borrows_the_source_buffer(self) -> None:
        source = CONTRACT_SOURCE.read_text(encoding="utf-8")
        body = source.split("pub fn source_str", 1)[1].split(
            "pub fn virtual_geometry_cook_request", 1
        )[0]

        self.assertIn("std::str::from_utf8(&self.source_bytes)", body)
        self.assertIn("Ok(source) => Ok(source)", body)
        self.assertIn("source_str_borrows_the_context_utf8_buffer", source)

    def test_current_wgsl_and_glsl_providers_validate_borrowed_text(self) -> None:
        wgsl = WGSL_SOURCE.read_text(encoding="utf-8")
        family = FAMILY_SOURCE.read_text(encoding="utf-8")

        self.assertIn("let source = context.source_str()?;", wgsl)
        self.assertGreaterEqual(family.count("let source = context.source_str()?;"), 2)
        self.assertNotIn("let source = context.source_text()?;", family)

    def test_release_contract_uses_the_exact_enlarged_workload(self) -> None:
        source = WGSL_SOURCE.read_text(encoding="utf-8")
        acceptance = run()["acceptance"]

        self.assertIn("PERF-MVP-PLUGINS05-BORROWED-SHADER-SOURCE", source)
        self.assertIn("const SOURCE_BYTES: usize = 1_048_576", source)
        self.assertIn("const ITERATIONS: usize = 32", source)
        self.assertEqual(acceptance["borrowed_p95_maximum_owned_ratio"], 0.85)
        self.assertTrue(acceptance["release_timing_pending"])

    def test_model_is_bound_to_current_and_head_sources(self) -> None:
        binding = run()["source_binding"]

        self.assertEqual(
            binding["git_revision"],
            "ca3ac3cc6ad218d04a5cd469447cea2452441321",
        )
        self.assertEqual(len(binding["source_sha256"]), 4)
        self.assertEqual(len(binding["source_manifest_sha256"]), 64)

    def test_rejects_non_positive_workloads(self) -> None:
        with self.assertRaises(ValueError):
            run(source_bytes=0)
        with self.assertRaises(ValueError):
            run(iterations_per_sample=0)


if __name__ == "__main__":
    unittest.main()
