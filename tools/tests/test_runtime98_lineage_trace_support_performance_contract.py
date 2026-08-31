from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SCENE_TRACE_SUPPORT = (
    ROOT
    / "zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/scene_trace_support.rs"
)
PERFORMANCE_TESTS = (
    ROOT
    / "zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/scene_trace_support/performance_tests.rs"
)


def function_body(source: str, start: str, end: str) -> str:
    offset = source.index(start)
    return source[offset : source.index(end, offset)]


class Runtime98LineageTraceSupportPerformanceContractTests(unittest.TestCase):
    def test_refresh_resolves_scheduled_regions_once_per_frame(self) -> None:
        source = SCENE_TRACE_SUPPORT.read_text(encoding="utf-8")
        refresh = function_body(
            source,
            "pub(in crate::hybrid_gi) fn refresh_recent_lineage_trace_support(",
            "pub(in crate::hybrid_gi) fn effective_lineage_trace_support_score(",
        )

        self.assertEqual(refresh.count("resolve_scheduled_scene_trace_regions()"), 1)
        self.assertIn("&scheduled_trace_regions", refresh)
        self.assertIn("current_lineage_trace_support_score(probe_id,", refresh)

    def test_probe_scoring_borrows_the_resolved_region_slice(self) -> None:
        source = SCENE_TRACE_SUPPORT.read_text(encoding="utf-8")
        scoring = function_body(
            source,
            "fn single_probe_scene_trace_support(",
            "fn resolve_scheduled_scene_trace_regions(",
        )
        compact_scoring = "".join(scoring.split())

        self.assertIn("scheduled_trace_regions:", scoring)
        self.assertIn("scheduled_trace_regions.iter()", compact_scoring)
        self.assertNotIn("scheduled_scene_trace_regions()", scoring)
        self.assertNotIn("collect::<Vec", scoring)

    def test_release_benchmark_tracks_region_resolution_work(self) -> None:
        self.assertTrue(PERFORMANCE_TESTS.is_file())
        source = PERFORMANCE_TESTS.read_text(encoding="utf-8")

        self.assertIn("RUNTIME98_LINEAGE_TRACE_SUPPORT_PERF", source)
        self.assertIn("legacy_region_resolutions=12286", source)
        self.assertIn("optimized_region_resolutions=1", source)
        self.assertIn("SAMPLE_PAIRS: usize = 21", source)
        self.assertIn("sample_order=alternating_legacy_first_even", source)
        self.assertIn("percentile_method=nearest_rank", source)
        self.assertIn("threshold_percent=40", source)
        self.assertIn("legacy_ns={}", source)
        self.assertIn("optimized_ns={}", source)


if __name__ == "__main__":
    unittest.main()
