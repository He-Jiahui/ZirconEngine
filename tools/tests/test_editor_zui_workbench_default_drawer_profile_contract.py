import pathlib
import re
import unittest


REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]
PAGE_TEMPLATES = REPO_ROOT / "zircon_editor/assets/ui/editor/layout/page_templates.toml"
LAYOUT_PRESETS = REPO_ROOT / "zircon_editor/assets/ui/editor/layout/presets.toml"
LAYOUT_PRESET = REPO_ROOT / "zircon_editor/src/ui/workbench/layout_preset.rs"
PAGE_LAYOUT = REPO_ROOT / "zircon_editor/src/ui/workbench/page_layout_template.rs"
DESIGN_STACK = REPO_ROOT / "zircon_editor/src/ui/workbench/preset/design_stack.rs"


def source(path):
    return path.read_text(encoding="utf-8")


class EditorZuiWorkbenchDefaultDrawerProfileContractTests(unittest.TestCase):
    def test_authoring_keeps_main_side_work_surfaces_and_collapses_bottom_output(self):
        source_text = source(LAYOUT_PRESET)
        self.assertIn(
            "drawer_states: drawer_states_with_bottom(ActivityDrawerMode::Collapsed)",
            source_text,
        )
        self.assertIn(
            "fn drawer_states_with_bottom(bottom_mode: ActivityDrawerMode)",
            source_text,
        )

    def test_review_and_inspector_profiles_prioritize_right_details(self):
        self.assertRegex(
            source(LAYOUT_PRESET),
            r"drawer_state\(\s*ActivityDrawerSlot::Bottom,\s*"
            r"ActivityDrawerMode::Collapsed,?\s*\)",
        )
        page_source = source(PAGE_LAYOUT)
        self.assertIn("fn right_review_drawers()", page_source)
        self.assertIn("default_drawer_states: right_review_drawers()", page_source)
        self.assertIn(
            "ActivityDrawerSlot::RightTop | ActivityDrawerSlot::RightBottom =>",
            page_source,
        )
        self.assertIn("_ => ActivityDrawerMode::Collapsed", page_source)

    def test_authoring_page_assets_match_the_runtime_default_profile(self):
        asset = source(PAGE_TEMPLATES)
        scene = re.search(
            r'page = "scene"(?P<body>.*?)(?=\n\[\[pages\]\]|\Z)',
            asset,
            re.S,
        )
        self.assertIsNotNone(scene)
        self.assertIn(
            'drawer_modes = { left_top = "pinned", left_bottom = "pinned", '
            'right_top = "pinned", right_bottom = "pinned", bottom = "collapsed" }',
            scene.group("body"),
        )
        self.assertIn(
            "LayoutPresetName::Authoring,\n            authoring_drawers(),",
            source(PAGE_LAYOUT),
        )

    def test_builtin_preset_asset_matches_the_runtime_default_profiles(self):
        asset = source(LAYOUT_PRESETS)
        authoring = re.search(
            r'name = "authoring"(?P<body>.*?)(?=\n\[\[presets\]\]|\Z)',
            asset,
            re.S,
        )
        review = re.search(
            r'name = "review"(?P<body>.*?)(?=\n\[\[presets\]\]|\Z)',
            asset,
            re.S,
        )
        self.assertIsNotNone(authoring)
        self.assertIsNotNone(review)
        self.assertIn('bottom = "collapsed"', authoring.group("body"))
        self.assertIn('bottom = "collapsed"', review.group("body"))

    def test_shell_preset_does_not_make_output_pinned_by_default(self):
        design_stack = source(DESIGN_STACK)
        output_start = design_stack.index("ActivityDrawerSlot::Bottom")
        output_end = design_stack.index("]),", output_start)
        output = design_stack[output_start:output_end]
        self.assertIn(
            ".with_default_mode(ActivityDrawerMode::Collapsed)", output
        )


if __name__ == "__main__":
    unittest.main()
