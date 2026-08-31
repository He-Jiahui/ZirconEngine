import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SHELF = REPO_ROOT / "zircon_runtime/src/text/atlas/shelf_allocator.rs"


class RuntimeTextAtlasShelfAdmissionContractTests(unittest.TestCase):
    def test_shelf_bounds_use_checked_coordinate_arithmetic(self) -> None:
        source = SHELF.read_text(encoding="utf-8")

        self.assertIn("checked_add(size.x)", source)
        self.assertIn("checked_add(size.y)", source)
        self.assertIn("checked_add(self.shelf_height)", source)
        self.assertIn("checked_add(self.padding_px)", source)

    def test_shelf_has_vertical_overflow_regression(self) -> None:
        source = SHELF.read_text(encoding="utf-8")

        self.assertIn(
            "render_text_atlas_shelf_rejects_vertical_coordinate_overflow",
            source,
        )


if __name__ == "__main__":
    unittest.main()
