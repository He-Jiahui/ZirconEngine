import tomllib
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
EXTENSION_MODULES = ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/modules/extensions"
)
SCAN_ONLY_UI_ASSET_EDITOR = (
    EXTENSION_MODULES / "ui/workbench_extension_ui_asset_editor_workspace.zui"
)


class EditorZuiExtensionModuleDensityContract(unittest.TestCase):
    def test_writable_side_geometry_preserves_the_center_at_compact_widths(self) -> None:
        expected_tiers = {
            "_rail_gap": "narrow",
            "_left": "narrow",
            "_right": "wide",
        }
        covered = {suffix: [] for suffix in expected_tiers}

        for path in sorted(EXTENSION_MODULES.rglob("*.zui")):
            if path == SCAN_ONLY_UI_ASSET_EDITOR:
                continue
            with path.open("rb") as source:
                document = tomllib.load(source)
            for node_id, node in document.get("nodes", {}).items():
                for suffix, expected_tier in expected_tiers.items():
                    if not node_id.endswith(suffix):
                        continue

                    location = (
                        f"{path.relative_to(EXTENSION_MODULES).as_posix()}:{node_id}"
                    )
                    covered[suffix].append(location)
                    self.assertEqual(
                        expected_tier,
                        node.get("props", {}).get("responsive_min_tier"),
                        f"{location} must yield space to the center workspace",
                    )

        for suffix, locations in covered.items():
            self.assertEqual(
                43,
                len(locations),
                f"all writable extension module {suffix} nodes must be covered",
            )

    def test_writable_activity_rail_gaps_follow_the_shell_chrome_token(self) -> None:
        rail_gaps = []

        for path in sorted(EXTENSION_MODULES.rglob("*.zui")):
            if path == SCAN_ONLY_UI_ASSET_EDITOR:
                continue
            with path.open("rb") as source:
                document = tomllib.load(source)
            for node_id, node in document.get("nodes", {}).items():
                if not node_id.endswith("_rail_gap"):
                    continue

                width = node.get("layout", {}).get("width", {})
                rail_gaps.append(
                    f"{path.relative_to(EXTENSION_MODULES).as_posix()}:{node_id}"
                )
                self.assertEqual("Fixed", width.get("stretch"))
                for constraint in ("min", "preferred", "max"):
                    self.assertEqual(
                        "$editor.chrome.activity_rail.width",
                        width.get(constraint),
                        f"{rail_gaps[-1]} must track the shell activity rail",
                    )

        self.assertEqual(
            43,
            len(rail_gaps),
            "all writable extension module rail gaps must be covered",
        )

    def test_writable_positive_container_gaps_use_shared_tokens(self) -> None:
        literal_gaps = []
        token_gaps = 0

        for path in sorted(EXTENSION_MODULES.rglob("*.zui")):
            if path == SCAN_ONLY_UI_ASSET_EDITOR:
                continue
            with path.open("rb") as source:
                document = tomllib.load(source)
            for node_id, node in document.get("nodes", {}).items():
                gap = node.get("layout", {}).get("container", {}).get("gap")
                if isinstance(gap, str) and gap.startswith("$editor."):
                    token_gaps += 1
                elif isinstance(gap, (int, float)) and gap > 0.0:
                    literal_gaps.append(
                        f"{path.relative_to(EXTENSION_MODULES).as_posix()}:{node_id}={gap}"
                    )

        self.assertGreater(
            token_gaps,
            0,
            "writable extension modules must exercise shared spacing tokens",
        )
        self.assertEqual(
            [],
            literal_gaps,
            "positive extension-module gaps must not create a private spacing scale",
        )


if __name__ == "__main__":
    unittest.main()
