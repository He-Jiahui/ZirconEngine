import re
import tomllib
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
EDITOR_TOKENS = ROOT / "zircon_editor/assets/ui/editor/theme/editor_tokens.zui"
EDITOR_MATERIAL = ROOT / "zircon_editor/assets/ui/theme/editor_material.zui"
RUNTIME_TOKENS = ROOT / "zircon_runtime_interface/src/ui/design_tokens.rs"
WELCOME_STYLE = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/"
    "paint_workbench_renderer/welcome/style.rs"
)
WORKBENCH_RENDERER_STYLE = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/"
    "paint_workbench_renderer/style.rs"
)
WORKBENCH_SPLITTER = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/"
    "paint_workbench_renderer/scene_layers/resize.rs"
)
ASSET_BROWSER_ICON_TEST = ROOT / (
    "zircon_editor/src/ui/retained_host/ui/"
    "asset_browser_icon_button_painter_tests.rs"
)
WORKBENCH_BUTTON_PALETTE = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "style_selector/workbench_button/palette.rs"
)
WORKBENCH_BUTTON_NORMAL = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "style_selector/workbench_button/states/normal.rs"
)
WORKBENCH_BUTTON_INTERACTIVE = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "style_selector/workbench_button/states/interactive.rs"
)


def rgba_bytes(value: str) -> list[int]:
    encoded = value.removeprefix("#")
    if len(encoded) == 6:
        encoded += "ff"
    return [int(encoded[offset : offset + 2], 16) for offset in range(0, 8, 2)]


def runtime_rgba_constant(source: str, constant_name: str) -> list[int]:
    match = re.search(
        rf"pub const {constant_name}: \[u8; 4\] = \[([^\]]+)\];",
        source,
    )
    if match is None:
        raise AssertionError(f"Runtime is missing {constant_name}")
    return [int(channel.strip()) for channel in match.group(1).split(",")]


def relative_luminance(value: str) -> float:
    channels = [channel / 255.0 for channel in rgba_bytes(value)[:3]]
    linear = [
        channel / 12.92
        if channel <= 0.04045
        else ((channel + 0.055) / 1.055) ** 2.4
        for channel in channels
    ]
    return 0.2126 * linear[0] + 0.7152 * linear[1] + 0.0722 * linear[2]


def contrast_ratio(first: str, second: str) -> float:
    lighter, darker = sorted(
        (relative_luminance(first), relative_luminance(second)), reverse=True
    )
    return (lighter + 0.05) / (darker + 0.05)


class EditorDesignTokenPaletteParityContract(unittest.TestCase):
    def test_workbench_text_and_primary_actions_have_legible_contrast(self) -> None:
        asset = tomllib.loads(EDITOR_TOKENS.read_text(encoding="utf-8"))
        palette = asset["palette"]

        for surface in palette["surface"]:
            self.assertGreaterEqual(contrast_ratio(palette["text_primary"], surface), 4.5)
            self.assertGreaterEqual(contrast_ratio(palette["text_secondary"], surface), 4.5)
            self.assertGreaterEqual(contrast_ratio(palette["focus_ring"], surface), 3.0)

        inverse_text = palette["surface"][0]
        self.assertGreaterEqual(contrast_ratio(inverse_text, palette["accent"]), 4.5)
        self.assertGreaterEqual(contrast_ratio(inverse_text, palette["focus_ring"]), 4.5)
        self.assertGreaterEqual(
            contrast_ratio(palette["accent"], palette["surface_selected"]),
            4.5,
        )
        self.assertGreaterEqual(
            contrast_ratio(palette["text_primary"], palette["surface_selected"]),
            4.5,
        )

        button_palette = WORKBENCH_BUTTON_PALETTE.read_text(encoding="utf-8")
        normal_state = WORKBENCH_BUTTON_NORMAL.read_text(encoding="utf-8")
        interactive_state = WORKBENCH_BUTTON_INTERACTIVE.read_text(encoding="utf-8")
        self.assertEqual(
            2, button_palette.count("primary_text: palette.shell_background")
        )
        self.assertEqual(2, button_palette.count("primary_pressed_text: palette.text"))
        self.assertIn("text: button_palette.primary_text", normal_state)
        self.assertIn("glyph: button_palette.primary_text", normal_state)
        self.assertIn("style.text = button_palette.primary_pressed_text", interactive_state)
        self.assertIn("style.glyph = button_palette.primary_pressed_text", interactive_state)

    def test_workbench_palette_uses_neutral_slate_surface_roles(self) -> None:
        asset = tomllib.loads(EDITOR_TOKENS.read_text(encoding="utf-8"))
        palette = asset["palette"]

        # Match Slate's Background/Panel/Header/Dropdown role ladder. Accent is
        # reserved for interaction state instead of tinting every work surface.
        self.assertEqual(
            ["#151515", "#242424", "#2f2f2f", "#383838"],
            palette["surface"],
        )
        self.assertEqual("#0f0f0f", palette["surface_recessed"])
        self.assertEqual("#454545", palette["surface_hover"])
        self.assertEqual("#60aeff", palette["accent"])
        self.assertEqual("#66b2ff", palette["focus_ring"])

        neutral_roles = [
            *palette["surface"],
            palette["surface_recessed"],
            palette["surface_hover"],
            palette["surface_disabled"],
            palette["border"],
            palette["border_disabled"],
            palette["separator_strong"],
            palette["separator_soft"],
            palette["popup"],
            palette["track"],
        ]
        for role in neutral_roles:
            red, green, blue, _ = rgba_bytes(role)
            self.assertEqual(red, green, role)
            self.assertEqual(green, blue, role)

        welcome_style = WELCOME_STYLE.read_text(encoding="utf-8")
        self.assertRegex(
            welcome_style,
            r"const WELCOME_BACKGROUND: \[u8; 4\] =\s*PALETTE\.shell_background;",
        )
        self.assertRegex(
            welcome_style,
            r"const WELCOME_SUCCESS: \[u8; 4\] =\s*PALETTE\.success;",
        )
        self.assertNotIn("[19, 23, 30, 255]", welcome_style)
        self.assertNotIn("[88, 168, 112, 255]", welcome_style)

        renderer_style = WORKBENCH_RENDERER_STYLE.read_text(encoding="utf-8")
        for role in (
            "PALETTE.surface",
            "PALETTE.surface_inset",
            "PALETTE.shell_background",
            "PALETTE.shadow",
        ):
            self.assertIn(role, renderer_style)
        for legacy_color in (
            "[23, 27, 34, 255]",
            "[13, 16, 22, 255]",
            "[7, 10, 15, 255]",
            "[4, 6, 10, 180]",
        ):
            self.assertNotIn(legacy_color, renderer_style)

        splitter = WORKBENCH_SPLITTER.read_text(encoding="utf-8")
        self.assertIn("current_host_palette().separator_strong", splitter)
        self.assertNotIn("[42, 50, 56, 255]", splitter)

        asset_browser_test = ASSET_BROWSER_ICON_TEST.read_text(encoding="utf-8")
        self.assertIn(
            "EditorPaletteTokens::WORKBENCH_SURFACE[0]",
            asset_browser_test,
        )
        self.assertNotIn("[17, 20, 22, 255]", asset_browser_test)

    def test_authored_asset_and_runtime_defaults_share_one_palette(self) -> None:
        asset = tomllib.loads(EDITOR_TOKENS.read_text(encoding="utf-8"))
        palette = asset["palette"]
        runtime = RUNTIME_TOKENS.read_text(encoding="utf-8")

        surface_match = re.search(
            r"pub const WORKBENCH_SURFACE: \[\[u8; 4\]; 4\] = \[(.*?)\];",
            runtime,
            re.DOTALL,
        )
        self.assertIsNotNone(surface_match, "Runtime is missing WORKBENCH_SURFACE")
        runtime_surfaces = [
            [int(channel.strip()) for channel in entry.split(",")]
            for entry in re.findall(r"\[([^\]]+)\]", surface_match.group(1))
        ]
        self.assertEqual(
            runtime_surfaces,
            [rgba_bytes(value) for value in palette["surface"]],
            "the authored surface ladder and first-frame Runtime defaults must match",
        )

        for field_name in [
            "surface_recessed",
            "surface_hover",
            "surface_selected",
            "surface_disabled",
            "accent",
            "accent_soft",
            "border",
            "border_disabled",
            "separator_strong",
            "separator_soft",
            "text_primary",
            "text_secondary",
            "text_disabled",
            "success",
            "success_container",
            "info",
            "info_container",
            "warning",
            "warning_container",
            "error",
            "error_container",
            "popup",
            "track",
            "focus_ring",
            "shadow",
        ]:
            constant_name = f"WORKBENCH_{field_name.upper()}"
            self.assertEqual(
                runtime_rgba_constant(runtime, constant_name),
                rgba_bytes(palette[field_name]),
                f"{field_name} must not diverge between .zui and Runtime defaults",
            )

    def test_material_theme_interaction_roles_follow_editor_palette(self) -> None:
        palette = tomllib.loads(EDITOR_TOKENS.read_text(encoding="utf-8"))["palette"]
        material = tomllib.loads(EDITOR_MATERIAL.read_text(encoding="utf-8"))["tokens"]

        self.assertEqual("$theme.palette.surface.3", material["material_surface_pressed"])
        self.assertEqual(palette["surface_selected"], material["material_surface_selected"])
        self.assertEqual(palette["accent_soft"], material["material_accent_soft"])
        self.assertEqual("$theme.palette.surface.1", material["material_popup"])
        self.assertEqual(palette["track"], material["material_track"])
        self.assertEqual("$theme.palette.separator", material["material_outline_variant"])
        self.assertEqual("$theme.palette.surface.0", material["material_on_primary"])

        source = EDITOR_MATERIAL.read_text(encoding="utf-8").lower()
        for retired in ("#103c4a", "#0f6574", "#334852", "#0e1217", "#2a343c"):
            self.assertNotIn(retired, source)


if __name__ == "__main__":
    unittest.main()
