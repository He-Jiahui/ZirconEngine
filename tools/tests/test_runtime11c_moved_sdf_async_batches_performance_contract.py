from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
ASYNC_BATCH = ROOT / "zircon_runtime/src/text/sdf/font_bake/async_batch.rs"


def function_region(source: str, start: str, end: str) -> str:
    offset = source.index(start)
    return source[offset : source.index(end, offset)]


class Runtime11cMovedSdfAsyncBatchesPerformanceContractTests(unittest.TestCase):
    def test_submission_moves_group_entries_instead_of_cloning_each_batch(self) -> None:
        source = ASYNC_BATCH.read_text(encoding="utf-8")
        prepare = function_region(
            source,
            "    pub(super) fn prepare_missing_glyphs_async(",
            "    pub(super) fn cancel_async_generation(",
        )

        batch_size = prepare.index(
            "let batch_size = scheduler.max_glyphs_per_batch();"
        )
        moved_entries = prepare.index("let mut group_entries = group.entries.into_iter();")
        submit = prepare.index("match scheduler.try_submit(")
        self.assertLess(batch_size, moved_entries)
        self.assertLess(moved_entries, submit)
        self.assertNotIn(".chunks(", prepare)
        self.assertNotIn("let entries = entries.to_vec();", prepare)

    def test_batch_helper_collects_a_bounded_prefix_from_the_owned_iterator(self) -> None:
        source = ASYNC_BATCH.read_text(encoding="utf-8")
        helper = function_region(
            source,
            "fn take_async_batch(",
            "#[cfg(test)]",
        )

        self.assertIn("assert!(max_entries > 0", helper)
        self.assertIn("entries.by_ref()", helper)
        self.assertIn(".take(max_entries)", helper)
        self.assertIn(".collect::<Vec<_>>()", helper)
        self.assertNotIn(".cloned()", helper)
        self.assertNotIn(".to_vec()", helper)

    def test_batch_boundaries_and_order_are_covered_by_rust(self) -> None:
        source = ASYNC_BATCH.read_text(encoding="utf-8")

        self.assertIn(
            "fn moved_async_batches_preserve_boundaries_and_entry_order()",
            source,
        )
        self.assertIn("assert_eq!(glyph_ids(first), vec![1, 2]);", source)
        self.assertIn("assert_eq!(glyph_ids(second), vec![3, 4]);", source)
        self.assertIn("assert_eq!(glyph_ids(third), vec![5]);", source)
        self.assertIn("assert!(take_async_batch(&mut entries, 2).is_empty());", source)


if __name__ == "__main__":
    unittest.main()
