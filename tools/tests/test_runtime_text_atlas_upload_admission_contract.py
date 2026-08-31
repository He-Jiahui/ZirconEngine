import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
UPLOAD = REPO_ROOT / "zircon_runtime/src/text/atlas/upload.rs"
UPLOAD_TESTS = REPO_ROOT / "zircon_runtime/src/text/atlas/upload/tests.rs"


class RuntimeTextAtlasUploadAdmissionContractTests(unittest.TestCase):
    def test_upload_math_is_checked_and_zero_pages_are_not_normalized(self) -> None:
        source = UPLOAD.read_text(encoding="utf-8")

        self.assertNotIn("page.size.x.max(1)", source)
        self.assertNotIn("page.size.y.max(1)", source)
        self.assertNotIn("saturating_mul", source)
        self.assertNotIn("clamp_upload_rect", source)
        self.assertIn("checked_mul", source)
        self.assertIn("checked_add", source)
        self.assertIn("admit_upload_rect", source)

    def test_rust_regressions_cover_invalid_page_rect_and_stride(self) -> None:
        source = UPLOAD_TESTS.read_text(encoding="utf-8")

        for test_name in (
            "render_text_atlas_upload_rejects_zero_sized_page",
            "render_text_atlas_upload_rejects_dirty_rect_outside_page",
            "render_text_atlas_upload_rejects_rgba_row_stride_overflow",
            "render_text_atlas_upload_rejects_source_range_overflow",
        ):
            self.assertIn(test_name, source)


if __name__ == "__main__":
    unittest.main()
