from __future__ import annotations

import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/text/atlas/bitmap_run/staging.rs"


def rust_function_body(source: str, name: str) -> str:
    match = re.search(rf"\bfn\s+{re.escape(name)}\s*[^{{]*{{", source, re.DOTALL)
    if match is None:
        raise AssertionError(f"missing Rust function {name}")
    depth = 1
    index = match.end()
    while index < len(source) and depth:
        depth += source[index] == "{"
        depth -= source[index] == "}"
        index += 1
    if depth:
        raise AssertionError(f"unterminated Rust function {name}")
    return source[match.end() : index - 1]


class StreamingBitmapUploadStagingPerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        cls.entry_body = rust_function_body(
            source,
            "glyph_atlas_bitmap_upload_staging_plan",
        )
        cls.body = rust_function_body(
            source,
            "glyph_atlas_bitmap_upload_staging_plan_for_commands",
        )
        cls.command_loop = cls.body.split("for command in upload_commands", 1)[1]

    def test_command_copy_selection_does_not_materialize_a_temporary_vector(self) -> None:
        self.assertIn("&run.upload_commands", self.entry_body)
        self.assertIn("let mut copies_by_page = HashMap::new()", self.body)
        self.assertIn("let mut copies = copies_by_page", self.command_loop)
        self.assertNotIn("collect::<Vec<_>>()", self.command_loop)
        self.assertNotIn("Vec::with_capacity", self.command_loop)
        self.assertNotIn("copies.is_empty()", self.command_loop)

    def test_first_matching_copy_is_reused_without_a_second_scan(self) -> None:
        self.assertIn("let first_copy = copies.next()", self.command_loop)
        self.assertEqual(
            self.command_loop.count("first_copy.into_iter().chain(copies)"),
            2,
        )
        self.assertEqual(self.command_loop.count("for copy in copies"), 0)

    def test_filter_keeps_page_and_target_rect_membership_contracts(self) -> None:
        self.assertIn(".entry(copy.page_key)", self.body)
        self.assertIn(".get(&command.page_key)", self.command_loop)
        self.assertIn(
            "target_rect_contains(command.rect, copy.atlas_rect)",
            self.command_loop,
        )
        self.assertIn("GlyphAtlasBitmapUploadStagingFailureReason::MissingPage", self.command_loop)
        self.assertIn("pages.push(page_staging)", self.command_loop)


if __name__ == "__main__":
    unittest.main()
