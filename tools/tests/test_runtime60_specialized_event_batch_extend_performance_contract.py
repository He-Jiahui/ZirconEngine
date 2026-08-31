from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/scene/ecs/events/queue.rs"


def function_body(source: str, name: str) -> str:
    match = re.search(rf"\bfn\s+{re.escape(name)}\b", source)
    if match is None:
        raise AssertionError(f"missing function {name}")
    start = source.find("{", match.end())
    if start < 0:
        raise AssertionError(f"missing body for {name}")
    depth = 0
    for index in range(start, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[start : index + 1]
    raise AssertionError(f"unterminated body for {name}")


class SpecializedEventBatchExtendPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.send_batch = function_body(cls.source, "send_batch")

    def test_batch_write_uses_vec_extend_specialization(self) -> None:
        self.assertIn("let before = self.next.len();", self.send_batch)
        self.assertIn("self.next.extend(events);", self.send_batch)
        self.assertIn("let written = self.next.len() - before;", self.send_batch)

    def test_batch_write_no_longer_manually_reserves_and_pushes(self) -> None:
        self.assertNotIn("size_hint()", self.send_batch)
        self.assertNotIn("self.next.reserve", self.send_batch)
        self.assertNotIn("for event in events", self.send_batch)

    def test_rust_regressions_cover_batch_semantics(self) -> None:
        self.assertIn(
            "fn runtime60_batch_specialized_batch_extend_preserves_order_and_count()",
            self.source,
        )
        self.assertIn(
            "fn runtime60_batch_specialized_batch_extend_accepts_non_exact_iterators()",
            self.source,
        )
        self.assertIn(
            "fn runtime60_batch_empty_batch_does_not_raise_the_high_water_mark()",
            self.source,
        )


if __name__ == "__main__":
    unittest.main()
