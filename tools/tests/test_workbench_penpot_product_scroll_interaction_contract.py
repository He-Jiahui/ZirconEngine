import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SHELL = REPO_ROOT / "zircon_editor/assets/ui/editor/components/workbench/shell"


def load_document(name: str) -> dict:
    with (SHELL / name).open("rb") as source:
        return tomllib.load(source)


class WorkbenchPenpotProductScrollInteractionContractTests(unittest.TestCase):
    def test_scrollable_shell_surfaces_declare_hover_input(self) -> None:
        targets = (
            ("workbench_activity_rail.zui", "activity_rail", "Vertical"),
            ("workbench_top_toolbar.zui", "toolbar_command_row", "Horizontal"),
            ("workbench_viewport_panel.zui", "viewport_toolbar", "Horizontal"),
        )

        for asset, node_id, axis in targets:
            with self.subTest(asset=asset, node=node_id):
                node = load_document(asset)["nodes"][node_id]
                container = node["layout"]["container"]
                self.assertEqual("ScrollableBox", container["kind"])
                self.assertEqual(axis, container["axis"])
                if node_id != "activity_rail":
                    self.assertEqual("Receive", node["layout"].get("input_policy"))
                self.assertIs(
                    node.get("props", {}).get("input_hoverable"),
                    True,
                    f"{node_id} must opt into pointer hover input like other scroll surfaces",
                )


if __name__ == "__main__":
    unittest.main()
