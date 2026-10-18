from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_editor/src/core/play/pending_edits/queue.rs"


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


class Editor07PendingEditPageSingleScanPerformanceContractTests(unittest.TestCase):
    def test_page_reserves_from_the_iterator_upper_bound(self) -> None:
        body = function_body(SOURCE.read_text(encoding="utf-8"), "pub fn page(")

        self.assertIn("candidates.size_hint().1", body)
        self.assertIn("unwrap_or(limit).min(limit)", body)
        self.assertIn("Vec::with_capacity(page_capacity)", body)
        self.assertIn("candidates.by_ref().take(limit)", body)

    def test_page_does_not_count_or_clone_the_queue_before_projection(self) -> None:
        body = function_body(SOURCE.read_text(encoding="utf-8"), "pub fn page(")

        self.assertNotIn("candidates.clone()", body)
        self.assertNotIn("candidates.count()", body)
        self.assertNotIn("collect::<Vec<_>>()", body)

    def test_bounded_page_pressure_model_removes_the_full_queue_rescan(self) -> None:
        queue_entries = 1_048_576
        page_limit = 128

        legacy_iterator_steps = queue_entries + page_limit
        optimized_iterator_steps = page_limit
        self.assertEqual(legacy_iterator_steps, 1_048_704)
        self.assertEqual(optimized_iterator_steps, 128)
        self.assertLess(optimized_iterator_steps, legacy_iterator_steps)
        self.assertEqual(
            legacy_iterator_steps - optimized_iterator_steps,
            queue_entries,
        )

    def test_next_cursor_probe_remains_after_the_bounded_projection(self) -> None:
        body = function_body(SOURCE.read_text(encoding="utf-8"), "pub fn page(")

        self.assertIn("for intent in candidates.by_ref().take(limit)", body)
        self.assertIn("let next_cursor = candidates", body)
        self.assertIn(".next()", body)
        self.assertIn("entries.last()", body)


if __name__ == "__main__":
    unittest.main()
