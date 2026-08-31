from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
ALLOCATOR = ROOT / (
    "zircon_runtime/src/graphics/scene/scene_renderer/environment/"
    "probe_buffer/slot_allocator.rs"
)
RUST_TEST = ROOT / (
    "zircon_runtime/src/graphics/scene/scene_renderer/environment/"
    "probe_buffer/tests/slot_allocator.rs"
)


def function_region(source: str, start: str, end: str) -> str:
    offset = source.index(start)
    return source[offset : source.index(end, offset)]


class Runtime09f1IntrusiveProbeSlotLruPerformanceContractTests(unittest.TestCase):
    def test_allocator_keeps_intrusive_oldest_and_newest_links(self) -> None:
        source = ALLOCATOR.read_text(encoding="utf-8")

        self.assertIn("previous: Option<ResourceId>", source)
        self.assertIn("next: Option<ResourceId>", source)
        self.assertIn("oldest: Option<ResourceId>", source)
        self.assertIn("newest: Option<ResourceId>", source)
        self.assertNotIn("last_used: u64", source)
        self.assertNotIn("slot_owners: Vec<Option<ResourceId>>", source)
        self.assertNotIn("clock: u64", source)

    def test_pressure_evicts_the_oldest_link_without_scanning_entries(self) -> None:
        source = ALLOCATOR.read_text(encoding="utf-8")
        acquire = function_region(source, "    pub(super) fn acquire(", "    fn touch(")
        eviction = function_region(source, "    fn evict_oldest(", "    fn insert_newest(")

        self.assertIn("self.touch(cubemap, entry);", acquire)
        self.assertIn("self.evict_oldest()", acquire)
        self.assertIn("self.oldest", eviction)
        self.assertIn(".remove(&evicted)", eviction)
        self.assertNotIn(".min_by_key(", acquire)
        self.assertNotIn(".iter().position(", acquire)
        self.assertNotIn("self.entries.iter()", acquire)

    def test_repeated_touch_order_and_slot_reuse_are_covered_by_rust(self) -> None:
        source = RUST_TEST.read_text(encoding="utf-8")

        self.assertIn(
            "fn render_probe_slot_allocator_preserves_lru_order_across_repeated_touches()",
            source,
        )
        self.assertIn("assert_eq!(fifth_slot.evicted, Some(third));", source)
        self.assertIn("assert_eq!(sixth_slot.evicted, Some(fourth));", source)
        self.assertIn("assert_eq!(fifth_slot.slot, third_slot.slot);", source)
        self.assertIn("assert_eq!(sixth_slot.slot, fourth_slot.slot);", source)


if __name__ == "__main__":
    unittest.main()
