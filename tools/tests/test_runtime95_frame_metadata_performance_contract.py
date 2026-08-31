from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
LIGHT_BUFFER = (
    ROOT
    / "zircon_runtime/src/graphics/scene/scene_renderer/lighting/light_buffer.rs"
)
LIGHT_BUFFER_PERFORMANCE = (
    ROOT
    / "zircon_runtime/src/graphics/scene/scene_renderer/lighting/light_buffer/performance_tests.rs"
)
COOKIE_PLAN = (
    ROOT
    / "zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/light_cookie/frame_plan.rs"
)
COOKIE_PLAN_PERFORMANCE = (
    ROOT
    / "zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/light_cookie/frame_plan/performance_tests.rs"
)


def function_body(source: str, start: str, end: str) -> str:
    offset = source.index(start)
    return source[offset : source.index(end, offset)]


class Runtime95FrameMetadataPerformanceContractTests(unittest.TestCase):
    def test_large_volumetric_membership_uses_a_prebuilt_index(self) -> None:
        source = LIGHT_BUFFER.read_text(encoding="utf-8")
        packing = function_body(
            source,
            "pub(crate) fn pack_light_slices_with_advanced_metadata(",
            "fn apply_cookie_metadata(",
        )

        self.assertIn("enum VolumetricLightIdIndex", source)
        self.assertIn("HashSet", source)
        self.assertIn("VolumetricLightIdIndex::new(volumetric_light_ids)", packing)
        self.assertIn("volumetric_light_ids.contains(light_id)", packing)
        self.assertNotIn("volumetric_light_ids.contains(&light_id)", packing)

    def test_cookie_plan_uses_one_contiguous_sorted_index(self) -> None:
        source = COOKIE_PLAN.read_text(encoding="utf-8")
        planning = function_body(
            source,
            "pub(crate) fn build_cookie_frame_plan(",
            "fn projection_metadata(",
        )

        self.assertIn("Vec::with_capacity(cookies.len())", planning)
        self.assertIn("sort_unstable_by_key", planning)
        self.assertIn("entries.len() < COOKIE_ATLAS_MAX_ENTRIES", planning)
        self.assertIn("indexed[group_end - 1].1", planning)
        self.assertNotIn("collect::<BTreeMap", planning)

    def test_release_benchmarks_cover_both_metadata_indexes(self) -> None:
        self.assertTrue(LIGHT_BUFFER_PERFORMANCE.is_file())
        self.assertTrue(COOKIE_PLAN_PERFORMANCE.is_file())
        light_benchmark = LIGHT_BUFFER_PERFORMANCE.read_text(encoding="utf-8")
        cookie_benchmark = COOKIE_PLAN_PERFORMANCE.read_text(encoding="utf-8")

        self.assertIn("RUNTIME95_VOLUMETRIC_MEMBERSHIP_PERF", light_benchmark)
        self.assertIn("legacy_comparisons=268435456", light_benchmark)
        self.assertIn("optimized_probes=40960", light_benchmark)
        self.assertIn("RUNTIME95_COOKIE_CANDIDATE_PERF", cookie_benchmark)
        self.assertIn("legacy_tree_nodes=65536", cookie_benchmark)
        self.assertIn("optimized_tree_nodes=0", cookie_benchmark)
        self.assertIn("optimized_index_entries=65536", cookie_benchmark)


if __name__ == "__main__":
    unittest.main()
