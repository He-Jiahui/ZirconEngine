import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
ASSET_BROWSER = REPO_ROOT / "zircon_editor" / "assets" / "ui" / "editor" / "asset_browser.zui"


class EditorAssetBrowserDetailsScrollContractTests(unittest.TestCase):
    def test_details_body_is_the_scroll_authority_below_a_fixed_header(self):
        source = ASSET_BROWSER.read_text(encoding="utf-8")
        body = source.split("[nodes.details_scroll_body]", 1)[1].split("[nodes.details_panel]", 1)[0]
        panel = source.split("[nodes.details_panel]", 1)[1].split("[nodes.main_panel]", 1)[0]

        self.assertIn('component = "ScrollableBox"', body)
        self.assertIn('control_id = "AssetBrowserDetailsScrollBody"', body)
        self.assertIn('kind = "ScrollableBox"', body)
        self.assertIn('axis = "Vertical"', body)
        self.assertIn('scrollbar_visibility = "Auto"', body)
        self.assertIn('input_policy = "Receive"', body)
        self.assertIn('{ node = "details_header_panel" }', panel)
        self.assertIn('{ node = "details_scroll_body" }', panel)

    def test_details_content_remains_a_single_vertical_document(self):
        source = ASSET_BROWSER.read_text(encoding="utf-8")
        content = source.split("[nodes.details_content_panel]", 1)[1].split("[nodes.details_right_gutter]", 1)[0]

        self.assertIn('component = "VerticalBox"', content)
        self.assertIn('height = { min = 538.0, preferred = 538.0, max = 538.0, stretch = "Fixed" }', content)
        self.assertEqual(content.count("{ node = "), 6)


if __name__ == "__main__":
    unittest.main()
