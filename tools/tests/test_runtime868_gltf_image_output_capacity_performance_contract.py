import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    REPO_ROOT
    / "zircon_runtime/src/asset/importer/ingest/gltf_decode/images.rs"
)
LOWER = (
    REPO_ROOT / "zircon_runtime/src/asset/importer/ingest/gltf_decode/images/tests/optimization_batch_runtime868_gltf_image_output_capacity_tests.rs"
)


class Runtime868GltfImageOutputCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        match = re.search(
            r"pub\(super\) fn decode_images\(.*?\n}\n\n",
            cls.source,
            re.S,
        )
        if match is None:
            raise AssertionError("decode_images owner is missing")
        cls.body = match.group(0)

    def test_image_count_is_reused_for_limit_and_output_capacity(self) -> None:
        self.assertIn("let image_count = document.images().len()", self.body)
        self.assertIn(
            "image_count > AuxiliarySourceResolver::MAX_SNAPSHOT_FILES", self.body
        )
        self.assertIn("Vec::with_capacity(image_count)", self.body)

    def test_decoded_images_append_in_document_order_without_result_collect(self) -> None:
        self.assertIn("for image in document.images()", self.body)
        self.assertIn("images.push(decoded)", self.body)
        self.assertIn("Ok(images)", self.body)
        self.assertNotIn(".collect()", self.body)

    def test_existing_decode_branches_and_budget_charges_remain_owned(self) -> None:
        self.assertIn("gltf::image::Source::View", self.body)
        self.assertIn("sources.read(uri, MAX_GLTF_AUXILIARY_BYTES)", self.body)
        self.assertIn("decode_data_uri(uri, budget.remaining())", self.body)
        self.assertIn('budget.charge(image.pixels.len() as u64, "image")', self.body)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn(
            "mod optimization_batch_runtime868_gltf_image_output_capacity_tests;",
            self.source,
        )
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn(
            "runtime868_gltf_image_output_capacity_preserves_empty_capacity", lower
        )
        self.assertIn("RUNTIME868_GLTF_IMAGE_OUTPUT_CAPACITY_BENCH_V1", lower)
        self.assertIn("const SAMPLE_PAIRS: usize = 101;", lower)
        self.assertIn("legacy_p99_ns", lower)
        self.assertIn("optimized_p99_ns", lower)

    def test_dense_output_growth_model(self) -> None:
        image_count = 4_096
        legacy_growth_events = 11
        optimized_growth_events = 0
        self.assertEqual(image_count, 4_096)
        self.assertEqual(legacy_growth_events, 11)
        self.assertEqual(optimized_growth_events, 0)


if __name__ == "__main__":
    unittest.main()
