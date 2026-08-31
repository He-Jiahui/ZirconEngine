from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
CHROME = ROOT / (
    "zircon_editor/src/ui/layouts/windows/workbench_host_window/"
    "chrome_template_projection.rs"
)


def function_body(source: str, name: str) -> str:
    match = re.search(rf"\nfn {name}\([^{{]+\{{", source)
    if match is None:
        match = re.search(rf"\npub\(super\) fn {name}\([^{{]+\{{", source)
    if match is None:
        raise AssertionError(f"missing function {name}")

    depth = 1
    cursor = match.end()
    while depth and cursor < len(source):
        depth += source[cursor] == "{"
        depth -= source[cursor] == "}"
        cursor += 1
    return source[match.end() : cursor - 1]


class EditorChromeTemplateProjectionPerformanceContractTests(unittest.TestCase):
    def setUp(self) -> None:
        self.source = CHROME.read_text(encoding="utf-8")

    def test_multi_control_derivation_builds_one_borrowed_frame_index(self) -> None:
        self.assertIn("struct ControlFrameIndex<'a>", self.source)
        self.assertIn("for node in nodes.iter()", self.source)

        for name in ("control_frames", "tab_frames"):
            body = function_body(self.source, name)
            self.assertEqual(body.count("ControlFrameIndex::from_nodes(nodes)"), 1)
            self.assertNotIn("control_frame(nodes", body)

    def test_page_overflow_uses_index_instead_of_scanning_per_tab(self) -> None:
        body = function_body(self.source, "page_overflow_hidden_tab_indices")

        self.assertEqual(body.count("ControlFrameIndex::from_nodes(nodes)"), 1)
        self.assertNotIn("has_control_frame", body)
        self.assertNotIn("row_data", body)

    def test_read_only_tab_and_node_queries_do_not_clone_model_rows(self) -> None:
        for name in (
            "visible_page_tab_indices",
            "active_tab_row",
            "tab_text_overrides",
            "tab_node_with_state",
            "node_survives_tab_close_filter",
            "control_frame",
        ):
            self.assertNotIn("row_data", function_body(self.source, name), name)


if __name__ == "__main__":
    unittest.main()
