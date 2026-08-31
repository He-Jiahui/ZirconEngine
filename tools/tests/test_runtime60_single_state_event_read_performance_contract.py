from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/scene/ecs/events/cursor.rs"


def function_body(source: str, name: str) -> str:
    match = re.search(rf"\bfn\s+{re.escape(name)}\b", source)
    if match is None:
        raise AssertionError(f"missing function {name}")
    start = source.find("{", match.end())
    depth = 0
    for index in range(start, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[start : index + 1]
    raise AssertionError(f"unterminated body for {name}")


class SingleStateEventReadPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.next_body = function_body(cls.source, "next")

    def test_iterator_uses_one_internal_state_discriminant(self) -> None:
        self.assertIn("state: EventReadState<'events, T>", self.source)
        self.assertIn("enum EventReadState<'events, T>", self.source)
        self.assertIn("Empty,", self.source)
        self.assertIn("Events {", self.source)
        self.assertNotIn("inner: Option<std::slice::Iter", self.source)
        self.assertNotIn("cursor: Option<&'events mut EventCursor", self.source)

    def test_next_matches_once_and_commits_without_saturation(self) -> None:
        self.assertIn("let EventReadState::Events { inner, cursor } = &mut self.state", self.next_body)
        self.assertIn("cursor.cursor += 1;", self.next_body)
        self.assertNotIn("expect(", self.next_body)
        self.assertNotIn("saturating_add", self.next_body)
        self.assertNotIn("as_deref_mut", self.next_body)

    def test_rust_regressions_cover_empty_partial_and_exhausted_states(self) -> None:
        self.assertIn(
            "fn runtime60_batch_empty_event_read_iterator_stays_exhausted()",
            self.source,
        )
        self.assertIn(
            "fn runtime60_batch_partial_event_read_commits_each_yield_exactly_once()",
            self.source,
        )
        self.assertIn(
            "fn runtime60_batch_event_read_iterator_stays_exhausted_after_tail()",
            self.source,
        )


if __name__ == "__main__":
    unittest.main()
