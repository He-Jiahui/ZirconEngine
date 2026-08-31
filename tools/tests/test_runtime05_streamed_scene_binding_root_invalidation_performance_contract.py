from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
TOPOLOGY = ROOT / (
    "zircon_runtime/src/scene/world/compiled_binding/scene_binding_topology.rs"
)


def function_region(source: str, start: str, end: str) -> str:
    offset = source.index(start)
    return source[offset : source.index(end, offset)]


class Runtime05StreamedSceneBindingRootInvalidationPerformanceContractTests(
    unittest.TestCase
):
    def test_removal_streams_the_intrinsically_unique_root_chain(self) -> None:
        source = TOPOLOGY.read_text(encoding="utf-8")
        removal = function_region(
            source,
            "fn advance_scene_binding_generations_for_removal(",
            "fn advance_scene_binding_generations_for_new_descendant(",
        )
        helper = function_region(
            source,
            "fn scene_binding_removal_roots(",
            "#[cfg(test)]",
        )

        self.assertIn("scene_binding_removal_roots(entity, ancestors)", removal)
        self.assertIn("std::iter::once(entity).chain", helper)
        self.assertIn("scene_binding_ancestor_chain(previous_parent)", removal)
        self.assertNotIn("let mut roots", removal)
        self.assertNotIn("sort_unstable", removal)
        self.assertNotIn("dedup", removal)

    def test_reparent_keeps_overlap_deduplication(self) -> None:
        source = TOPOLOGY.read_text(encoding="utf-8")
        reparent = function_region(
            source,
            "fn advance_scene_binding_generations_for_reparent(",
            "fn advance_scene_binding_generations_for_removal(",
        )

        self.assertIn("roots.sort_unstable();", reparent)
        self.assertIn("roots.dedup();", reparent)

    def test_removed_entity_and_ancestors_share_one_generation(self) -> None:
        source = TOPOLOGY.read_text(encoding="utf-8")

        self.assertIn(
            "fn streamed_scene_binding_root_invalidation_preserves_one_generation()",
            source,
        )

    def test_release_gate_reports_raw_paired_latency_samples(self) -> None:
        source = TOPOLOGY.read_text(encoding="utf-8")

        self.assertIn("const BENCHMARK_WARMUP_PAIRS: usize = 4;", source)
        self.assertIn("const BENCHMARK_SAMPLE_PAIRS: usize = 21;", source)
        self.assertIn("RUNTIME05_STREAMED_SCENE_BINDING_ROOT_INVALIDATION_PERF", source)
        self.assertIn("legacy_samples_ns={:?}", source)
        self.assertIn("optimized_samples_ns={:?}", source)
        self.assertIn("optimized_sort_calls=0", source)
        self.assertIn(
            "optimized_p50_ns.saturating_mul(100) <= legacy_p50_ns.saturating_mul(90)",
            source,
        )
        self.assertIn("optimized_p95_ns <= legacy_p95_ns", source)


if __name__ == "__main__":
    unittest.main()
