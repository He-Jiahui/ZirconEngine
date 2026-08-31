from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
ALLOCATOR = ROOT / (
    "zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/allocator.rs"
)
ALLOCATOR_TESTS = ROOT / (
    "zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/allocator/tests.rs"
)


def function_region(source: str, start: str, end: str) -> str:
    offset = source.index(start)
    return source[offset : source.index(end, offset)]


class Runtime09EShadowAtlasPreemptionPerformanceContractTests(unittest.TestCase):
    def test_frame_reuses_the_planned_index_for_preemption_and_retention(self) -> None:
        source = ALLOCATOR.read_text(encoding="utf-8")
        allocate_frame = function_region(
            source,
            "    pub(crate) fn allocate_frame(",
            "    fn allocate_slot_generation(",
        )

        planned_sort = allocate_frame.index("planned.sort_by(compare_planned_slots);")
        planned_index = allocate_frame.index("let planned_by_key = planned")
        contention = allocate_frame.index("self.update_preemption_contention(")
        retained_projection = allocate_frame.index("let mut retained =")

        self.assertLess(planned_sort, planned_index)
        self.assertLess(planned_index, contention)
        self.assertLess(contention, retained_projection)
        self.assertEqual(allocate_frame.count("planned.sort_by(compare_planned_slots);"), 1)

    def test_contention_uses_indexed_incumbents_and_a_bounded_priority_prefix(self) -> None:
        source = ALLOCATOR.read_text(encoding="utf-8")
        contention = function_region(
            source,
            "    fn update_preemption_contention(",
            "    fn should_release_for_confirmed_preemption(",
        )

        self.assertIn(
            "planned_by_key: &HashMap<ShadowSlotKey, PlannedShadowSlot>",
            contention,
        )
        self.assertIn("planned_by_key.get(&retained_key)", contention)
        self.assertNotIn("planned.iter().find", contention.replace("\n", ""))
        self.assertIn(
            "if challenger.request.priority_score() < required_priority {",
            contention,
        )
        self.assertIn("break;", contention)

    def test_release_benchmark_requires_at_least_95_percent_lower_p95(self) -> None:
        source = ALLOCATOR_TESTS.read_text(encoding="utf-8")

        self.assertIn("RUNTIME09E_SHADOW_PREEMPTION_BENCH_V1", source)
        self.assertIn(
            "indexed_p95_ns.saturating_mul(20) <= legacy_p95_ns",
            source,
        )
        self.assertIn("legacy_incumbent_linear_comparisons", source)
        self.assertIn("indexed_challenger_visits", source)


if __name__ == "__main__":
    unittest.main()
