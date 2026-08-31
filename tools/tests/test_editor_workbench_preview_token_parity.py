import re
import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
EDITOR_TOKENS = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/theme/editor_tokens.zui"
)
PREVIEW_CSS = REPO_ROOT / "tools/editor-workbench-preview/design.css"
PREVIEW_JS = REPO_ROOT / "tools/editor-workbench-preview/design.js"
PREVIEW_VERIFIER = REPO_ROOT / (
    "tools/editor-workbench-preview/verify-designs.mjs"
)


def load_tokens() -> dict:
    with EDITOR_TOKENS.open("rb") as source:
        return tomllib.load(source)


def root_variables(source: str) -> dict[str, str]:
    match = re.search(r":root\s*\{(?P<body>.*?)\n\}", source, re.DOTALL)
    if match is None:
        raise AssertionError("design.css must define a :root token block")
    return {
        name: value.strip()
        for name, value in re.findall(
            r"(--[a-z0-9-]+)\s*:\s*([^;]+);", match.group("body")
        )
    }


class EditorWorkbenchPreviewTokenParityTests(unittest.TestCase):
    def test_preview_foundation_projects_current_editor_palette_and_radii(self) -> None:
        tokens = load_tokens()
        palette = tokens["palette"]
        controls = tokens["controls"]
        variables = root_variables(PREVIEW_CSS.read_text(encoding="utf-8"))

        expected = {
            "--chrome": palette["surface"][0],
            "--chrome-2": palette["surface"][1],
            "--panel": palette["surface"][1],
            "--panel-2": palette["surface"][2],
            "--panel-3": palette["surface"][3],
            "--inset": palette["surface_recessed"],
            "--line": palette["separator_soft"],
            "--line-strong": palette["border"],
            "--text": palette["text_primary"],
            "--muted": palette["text_secondary"],
            "--subtle": palette["text_disabled"],
            "--accent": palette["accent"],
            "--accent-2": palette["info"],
            "--green": palette["success"],
            "--yellow": palette["warning"],
            "--red": palette["error"],
            "--blue": palette["info"],
            "--radius-sm": f'{controls["small_radius"]:g}px',
            "--radius": f'{controls["control_radius"]:g}px',
            "--radius-lg": f'{controls["panel_radius"]:g}px',
        }
        self.assertEqual(expected, {name: variables[name] for name in expected})

    def test_preview_contains_no_retired_teal_palette_authority(self) -> None:
        sources = "\n".join(
            (
                PREVIEW_CSS.read_text(encoding="utf-8"),
                PREVIEW_JS.read_text(encoding="utf-8"),
            )
        ).lower()
        for retired in (
            "#35c7d0",
            "#3cc7d6",
            "#55d6e0",
            "rgba(53, 199, 208",
            "rgba(53,199,208",
            "rgba(60, 199, 214",
            "rgba(60,199,214",
            "accent.teal",
        ):
            self.assertNotIn(retired, sources, retired)

    def test_preview_contains_no_retired_blue_black_foundation(self) -> None:
        sources = "\n".join(
            (
                PREVIEW_CSS.read_text(encoding="utf-8"),
                PREVIEW_JS.read_text(encoding="utf-8"),
            )
        ).lower()
        for retired in (
            "#0b0f12",
            "#11161a",
            "#121619",
            "#12181d",
            "#151a1e",
            "#171d22",
            "#1b2025",
            "#20262b",
            "#20272e",
            "#252d34",
            "#2b343c",
            "#303740",
            "#3d4952",
        ):
            self.assertNotIn(retired, sources, retired)

    def test_visual_profile_uses_current_blue_accent_and_readable_dark_bounds(
        self,
    ) -> None:
        source = PREVIEW_VERIFIER.read_text(encoding="utf-8")

        self.assertNotIn("tealRatio", source)
        self.assertNotIn("teal accent ratio", source)
        self.assertIn("blueAccentRatio", source)
        self.assertIn("const MAX_AVERAGE_LUMA = 60;", source)
        self.assertIn("const MAX_BRIGHT_PIXEL_RATIO = 0.12;", source)

    def test_manifest_sheet_reuses_foundation_tokens(self) -> None:
        source = PREVIEW_CSS.read_text(encoding="utf-8")

        for selector, declarations in (
            ("preview-sheet", ("background: var(--chrome);",)),
            (
                "sheet-summary-card",
                (
                    "border: 1px solid var(--line);",
                    "background: var(--panel);",
                ),
            ),
            (
                "sheet-kind",
                (
                    "color: var(--chrome);",
                    "background: var(--accent);",
                ),
            ),
        ):
            match = re.search(
                rf"\.{re.escape(selector)}\s*\{{(?P<body>.*?)\n\}}",
                source,
                re.DOTALL,
            )
            self.assertIsNotNone(match, selector)
            for declaration in declarations:
                self.assertIn(declaration, match.group("body"), selector)


if __name__ == "__main__":
    unittest.main()
