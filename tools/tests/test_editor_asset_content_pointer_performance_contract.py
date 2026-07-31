from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
STATE_WRITEBACK = (
    ROOT
    / "zircon_editor/src/ui/retained_host/app/asset_content_pointer/target/state.rs"
)


class EditorAssetContentPointerPerformanceContractTests(unittest.TestCase):
    def test_unchanged_content_pointer_state_skips_ui_property_writeback(self) -> None:
        source = STATE_WRITEBACK.read_text(encoding="utf-8")

        compare = source.index("surface.content_state == state")
        assignment = source.index("surface.content_state = state")
        writeback = source.index("self.apply_asset_pointer_state_to_ui(surface_mode)")

        self.assertLess(compare, assignment)
        self.assertLess(assignment, writeback)
        self.assertIn("return;", source[compare:assignment])


if __name__ == "__main__":
    unittest.main()
