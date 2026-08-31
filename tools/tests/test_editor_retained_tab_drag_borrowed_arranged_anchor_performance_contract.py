import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_editor/src/ui/retained_host/tab_drag/strip_hitbox.rs"


def source() -> str:
    return SOURCE.read_text(encoding="utf-8")


def rust_item(text: str, marker: str) -> str:
    start = text.index(marker)
    brace = text.index("{", start)
    depth = 0
    for index in range(brace, len(text)):
        if text[index] == "{":
            depth += 1
        elif text[index] == "}":
            depth -= 1
            if depth == 0:
                return text[start : index + 1]
    raise AssertionError(f"unterminated Rust item: {marker}")


class RetainedTabDragBorrowedAnchorPerformanceContract(unittest.TestCase):
    def test_strip_rows_borrow_identity_title_and_document_host(self):
        text = source()

        row = rust_item(text, "struct StripTabRef<'a>")
        host = rust_item(text, "enum StripTabHost<'a>")
        self.assertIn("instance_id: &'a ViewInstanceId", row)
        self.assertIn("title: &'a str", row)
        self.assertIn("host: StripTabHost<'a>", row)
        self.assertIn("Drawer(ActivityDrawerSlot)", host)
        self.assertIn("target: &'a WorkspaceTarget", host)
        self.assertIn("workspace_path: &'a [usize]", host)

    def test_hit_box_owns_geometry_only(self):
        text = source()

        hit_box = rust_item(text, "struct TabStripHitBox")
        self.assertNotIn("tabs:", hit_box)
        self.assertNotIn("struct StripTabEntry", text)

    def test_release_path_builds_no_temporary_tab_vector(self):
        text = source()

        self.assertNotIn("collect::<Vec<_>>()", text)
        self.assertNotIn("let tabs: Vec<_>", text)

    def test_pane_and_document_row_projection_are_borrowed(self):
        text = source()

        pane = rust_item(text, "fn strip_tab_from_pane")
        document = rust_item(text, "fn strip_tab_from_document")
        self.assertNotIn(".clone()", pane)
        self.assertNotIn(".clone()", document)
        self.assertIn("workspace_path: &tab.workspace_path", document)

    def test_dragged_row_is_skipped_before_any_width_measurement(self):
        precise = rust_item(source(), "fn precise_drop_in_tabs")

        skip = precise.index("if tab.instance_id.0 == dragging_id")
        measure = precise.index("strip.tab_width(tab)")
        self.assertLess(skip, measure)
        self.assertIn("continue;", precise[skip:measure])

    def test_only_selected_row_is_materialized_into_the_result(self):
        text = source()

        precise = rust_item(text, "fn precise_drop_in_tabs")
        result = rust_item(text, "fn resolved_drop_for_tab")
        self.assertGreaterEqual(precise.count("resolved_drop_for_tab("), 3)
        self.assertIn("host: tab.host.materialize()", result)
        self.assertIn("target_id: tab.instance_id.clone()", result)


if __name__ == "__main__":
    unittest.main()
