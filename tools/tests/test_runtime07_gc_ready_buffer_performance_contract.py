from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/script/vm/runtime/hot_reload_coordinator/gc_schedule.rs"


def function_body(source: str, signature: str) -> str:
    start = source.index(signature)
    opening = source.index("{", start)
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index]
    raise AssertionError(f"unterminated function: {signature}")


class GcReadyBufferPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.projection = function_body(
            cls.source,
            "pub(super) fn take_due(&mut self, frame_index: u64) -> Vec<PluginSlotId>",
        )

    def test_due_slots_use_one_contiguous_ready_buffer(self) -> None:
        self.assertIn("let mut ready = Vec::with_capacity(ready_capacity);", self.projection)
        self.assertIn(".map(|(_, slots)| slots.len())", self.projection)
        self.assertIn("ready.push(slot);", self.projection)
        self.assertNotIn("let mut ready = BTreeSet::new();", self.projection)
        self.assertNotIn("ready.insert(slot)", self.projection)

    def test_ready_buffer_preserves_stable_slot_order(self) -> None:
        self.assertIn("ready.sort_unstable();", self.projection)


if __name__ == "__main__":
    unittest.main()
