import tomllib
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
CORE_MODULES = ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/modules/core"
)


class EditorZuiCoreModuleDensityContract(unittest.TestCase):
    def test_side_geometry_preserves_the_center_at_compact_widths(self) -> None:
        expected_tiers = {
            "_rail_gap": "narrow",
            "_left": "narrow",
            "_right": "wide",
        }
        covered = {suffix: [] for suffix in expected_tiers}

        for path in sorted(CORE_MODULES.rglob("*.zui")):
            with path.open("rb") as source:
                document = tomllib.load(source)
            for node_id, node in document.get("nodes", {}).items():
                for suffix, expected_tier in expected_tiers.items():
                    if not node_id.endswith(suffix):
                        continue

                    location = (
                        f"{path.relative_to(CORE_MODULES).as_posix()}:{node_id}"
                    )
                    covered[suffix].append(location)
                    self.assertEqual(
                        expected_tier,
                        node.get("props", {}).get("responsive_min_tier"),
                        f"{location} must yield space to the center workspace",
                    )

        for suffix, locations in covered.items():
            self.assertEqual(
                10,
                len(locations),
                f"all core module {suffix} nodes must be covered",
            )

    def test_activity_rail_gaps_follow_the_shell_chrome_token(self) -> None:
        rail_gaps = []

        for path in sorted(CORE_MODULES.rglob("*.zui")):
            with path.open("rb") as source:
                document = tomllib.load(source)
            for node_id, node in document.get("nodes", {}).items():
                if not node_id.endswith("_rail_gap"):
                    continue

                width = node.get("layout", {}).get("width", {})
                rail_gaps.append(
                    f"{path.relative_to(CORE_MODULES).as_posix()}:{node_id}"
                )
                self.assertEqual("Fixed", width.get("stretch"))
                for constraint in ("min", "preferred", "max"):
                    self.assertEqual(
                        "$editor.chrome.activity_rail.width",
                        width.get(constraint),
                        f"{rail_gaps[-1]} must track the shell activity rail",
                    )

        self.assertEqual(10, len(rail_gaps), "all core module rail gaps must be covered")

    def test_positive_container_gaps_use_shared_density_tokens(self) -> None:
        literal_gaps = []
        token_gaps = 0

        for path in sorted(CORE_MODULES.rglob("*.zui")):
            with path.open("rb") as source:
                document = tomllib.load(source)
            for node_id, node in document.get("nodes", {}).items():
                container = node.get("layout", {}).get("container", {})
                gap = container.get("gap")
                if isinstance(gap, str) and gap.startswith("$editor."):
                    token_gaps += 1
                elif isinstance(gap, (int, float)) and gap > 0.0:
                    literal_gaps.append(
                        f"{path.relative_to(CORE_MODULES).as_posix()}:{node_id}={gap}"
                    )

        self.assertGreater(token_gaps, 0, "core modules must exercise shared gap tokens")
        self.assertEqual(
            [],
            literal_gaps,
            "positive core-module container gaps must not create a private spacing scale",
        )


if __name__ == "__main__":
    unittest.main()
