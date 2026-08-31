from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
PLAN = (
    ROOT
    / "zircon_runtime/src/graphics/scene/scene_renderer/shadow/plan.rs"
)
TESTS = (
    ROOT
    / "zircon_runtime/src/graphics/scene/scene_renderer/shadow/plan/tests.rs"
)


class Runtime95AdaptiveShadowSlotRequestCapacityPerformanceContractTests(
    unittest.TestCase
):
    def test_dense_shadow_sets_use_an_exact_preallocated_capacity(self) -> None:
        source = PLAN.read_text(encoding="utf-8")
        production = source.split("#[cfg(test)]", 1)[0]
        normalized = " ".join(production.split())

        self.assertIn(
            "const SHADOW_REQUEST_PREALLOCATION_SAMPLE_LIGHTS_PER_KIND: usize = 64;",
            production,
        )
        self.assertIn(
            "const SHADOW_REQUEST_PREALLOCATION_MINIMUM_SAMPLED_REQUESTS: usize = 32;",
            production,
        )
        self.assertIn("fn shadow_slot_request_capacity_if_dense(", production)
        self.assertEqual(
            production.count("SHADOW_REQUEST_PREALLOCATION_SAMPLE_LIGHTS_PER_KIND"),
            3,
        )
        self.assertIn(".map_or_else(Vec::new, Vec::with_capacity)", normalized)
        self.assertNotIn("let mut requests = Vec::new();", production)

    def test_density_gate_and_exact_count_share_the_shadow_enabled_predicate(self) -> None:
        source = PLAN.read_text(encoding="utf-8")
        production = source.split("#[cfg(test)]", 1)[0]
        helper = production.split(
            "fn shadow_slot_request_capacity_if_dense", 1
        )[1].split("fn append_shadow_slot_requests", 1)[0]

        self.assertEqual(helper.count("shadow_enabled(light.shadow).is_some()"), 4)
        self.assertIn(
            "< SHADOW_REQUEST_PREALLOCATION_MINIMUM_SAMPLED_REQUESTS",
            helper,
        )
        self.assertIn("return None;", helper)
        self.assertIn("Some(", helper)

    def test_rust_coverage_checks_dense_and_sparse_capacity_paths(self) -> None:
        source = TESTS.read_text(encoding="utf-8")

        self.assertIn(
            "shadow_slot_request_capacity_preallocates_dense_shadow_sets",
            source,
        )
        self.assertIn("assert_eq!(requests.capacity(), requests.len());", source)
        self.assertIn(
            "shadow_slot_request_capacity_preserves_sparse_growth_path",
            source,
        )
        self.assertIn(
            "assert_eq!(shadow_slot_request_capacity_if_dense(&lighting), None);",
            source,
        )


if __name__ == "__main__":
    unittest.main()
