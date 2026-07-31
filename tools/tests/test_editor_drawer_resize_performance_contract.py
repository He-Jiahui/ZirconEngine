from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
MOVEMENT = (
    ROOT
    / "zircon_editor/src/ui/retained_host/app/workspace_docking/drawer_resize/movement.rs"
)


class EditorDrawerResizePerformanceContractTests(unittest.TestCase):
    def test_repeated_pointer_extent_does_not_mark_layout_dirty(self) -> None:
        source = MOVEMENT.read_text(encoding="utf-8")
        update_start = source.index("fn update_drawer_resize_capture")
        finish_start = source.index("fn finish_drawer_resize_capture")
        update = source[update_start:finish_start]

        previous = update.index("let previous_preferred")
        unchanged = update.index("if previous_preferred == preferred")
        insert = update.index(".insert(active.region, preferred)")
        dirty = update.index("self.mark_layout_dirty()")

        self.assertLess(previous, unchanged)
        self.assertLess(unchanged, insert)
        self.assertLess(insert, dirty)
        self.assertIn("return;", update[unchanged:insert])


if __name__ == "__main__":
    unittest.main()
