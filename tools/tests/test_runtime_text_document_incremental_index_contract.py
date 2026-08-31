import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
INDEX = REPO_ROOT / "zircon_runtime/src/text/document/index.rs"
STORAGE = REPO_ROOT / "zircon_runtime/src/text/document/storage.rs"
DOCUMENT_TESTS = REPO_ROOT / "zircon_runtime/src/text/document/tests.rs"
PROFILE = REPO_ROOT / "zircon_runtime/src/text/document/index_profile.rs"


def owner_body(source: str, signature: str, end_marker: str) -> str:
    start = source.index(signature)
    end = source.index(end_marker, start)
    return source[start:end]


class RuntimeTextDocumentIncrementalIndexContract(unittest.TestCase):
    def test_incremental_admission_does_not_flatten_context(self) -> None:
        index = INDEX.read_text(encoding="utf-8")
        prepare = owner_body(
            index,
            "pub(super) fn prepare_incremental_edit(",
            "pub(super) fn apply_incremental_edit(",
        )

        self.assertIn("range_is_ascii_grapheme_edit", prepare)
        self.assertNotIn("snapshot_range_unchecked", prepare)
        self.assertIn("ascii_grapheme_edit(replacement.as_bytes())", prepare)

    def test_piece_preflight_is_checked_and_allocation_free(self) -> None:
        storage = STORAGE.read_text(encoding="utf-8")
        helper = owner_body(
            storage,
            "pub(super) fn range_is_ascii_grapheme_edit(",
            "fn is_utf8_boundary(",
        )

        self.assertIn("piece_source_bytes", helper)
        self.assertIn("checked_add", helper)
        self.assertIn("piece_end >= range.end", helper)
        self.assertNotIn("String::with_capacity", helper)
        self.assertNotIn("snapshot_range_unchecked", helper)

    def test_unicode_and_separator_fallbacks_remain_explicit(self) -> None:
        tests = DOCUMENT_TESTS.read_text(encoding="utf-8")
        self.assertIn(
            "ascii_incremental_preflight_checks_piece_bytes_without_crossing_line_breaks",
            tests,
        )
        self.assertIn("source_index_rejects_incremental_edits_next_to_unicode_context", tests)
        self.assertIn("source_index_rejects_incremental_edits_next_to_crlf_context", tests)

    def test_incremental_profile_names_are_fixed_and_scoped(self) -> None:
        profile = PROFILE.read_text(encoding="utf-8")
        names = re.findall(r'"(text_document_grapheme_[^"]+)"', profile)

        self.assertEqual(len(names), 12)
        self.assertEqual(len(set(names)), 12)
        self.assertTrue(
            all(name.startswith("text_document_grapheme_") for name in names)
        )
        self.assertIn("text_document_grapheme_index_incremental_update_count", names)
        self.assertIn("text_document_grapheme_index_incremental_update_nanos", names)


if __name__ == "__main__":
    unittest.main()
