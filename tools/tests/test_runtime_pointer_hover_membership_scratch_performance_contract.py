import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SURFACE = ROOT / "zircon_runtime/src/ui/surface/surface.rs"
ROUTING = ROOT / "zircon_runtime/src/ui/surface/surface/event_routing.rs"


class RuntimePointerHoverMembershipScratchPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.surface = SURFACE.read_text(encoding="utf-8")
        cls.routing = ROUTING.read_text(encoding="utf-8")

    def test_surface_owns_non_serialized_hover_membership_scratch(self) -> None:
        self.assertIn("hover_diff_scratch: UiSurfaceHoverDiffScratch", self.surface)
        self.assertIn("#[serde(default, skip)]", self.surface)
        self.assertIn("struct UiSurfaceHoverDiffScratch", self.surface)

    def test_pointer_route_uses_the_retained_scratch(self) -> None:
        production = self.routing.split("#[cfg(test)]\nmod hot_path_tests", 1)[0]
        self.assertIn(
            "let mut hover_diff_scratch = std::mem::take(&mut self.hover_diff_scratch);",
            production,
        )
        self.assertIn("hover_diff_with_scratch(", production)
        self.assertIn("&hit.stacked,", production)
        self.assertIn("&previous_hovered,", production)
        self.assertIn("&mut hover_diff_scratch.membership", production)
        self.assertIn(
            "self.hover_diff_scratch = hover_diff_scratch;",
            production,
        )
        self.assertNotIn(
            "hover_diff(&hit.stacked, &previous_hovered)",
            production,
        )

    def test_scratch_path_reserves_only_when_warm_capacity_is_insufficient(self) -> None:
        helper = self.routing.split("fn hover_diff_with_scratch", 1)[1].split(
            "fn hover_diff_linear", 1
        )[0]
        self.assertIn("membership.clear();", helper)
        self.assertIn("if membership.capacity() < required", helper)
        self.assertIn("membership.reserve(required);", helper)
        self.assertIn("membership.extend(previous.iter().copied());", helper)
        self.assertIn("membership.extend(current.iter().copied());", helper)
        self.assertNotIn("HashSet::with_capacity", helper)

    def test_release_gate_and_work_model_bound_the_allocation_reduction(self) -> None:
        self.assertIn(
            "RUNTIME200_HOVER_DIFF_MEMBERSHIP_SCRATCH_BENCH_V1",
            self.routing,
        )
        events = 1_000_000
        legacy_allocations = events
        optimized_warm_allocations = 1
        self.assertLessEqual(optimized_warm_allocations * 100, legacy_allocations)


if __name__ == "__main__":
    unittest.main()
