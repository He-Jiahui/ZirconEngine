import pathlib
import tomllib
import unittest


REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]
DRAWER_ASSET = (
    REPO_ROOT
    / "zircon_editor/assets/ui/editor/components/workbench/shell/workbench_component_drawer.zui"
)
WORKBENCH_WINDOW = REPO_ROOT / "zircon_editor/assets/ui/editor/windows/workbench_window.zui"


def load_zui(path):
    with path.open("rb") as source:
        return tomllib.load(source)


def child_ids(node):
    return [child["node"] for child in node.get("children", [])]


def descendants(nodes, root_id):
    pending = list(child_ids(nodes[root_id]))
    found = set()
    while pending:
        node_id = pending.pop()
        if node_id in found:
            continue
        found.add(node_id)
        pending.extend(child_ids(nodes[node_id]))
    return found


class EditorZuiWorkbenchProductDrawerAuthorityContractTests(unittest.TestCase):
    def test_live_bottom_drawer_keeps_visible_geometry_authority_anchors(self):
        document = load_zui(DRAWER_ASSET)
        nodes = document["nodes"]

        self.assertEqual(
            child_ids(nodes["component_drawer"]),
            ["drawer_tabs", "component_drawer_content"],
        )
        self.assertEqual(
            nodes["component_drawer"]["control_id"], "WorkbenchComponentDrawer"
        )
        self.assertEqual(nodes["drawer_tabs"]["control_id"], "BottomDrawerHeaderRoot")
        self.assertEqual(
            nodes["component_drawer_content"]["control_id"],
            "BottomDrawerContentRoot",
        )
        for node_id in ("component_drawer", "drawer_tabs", "component_drawer_content"):
            self.assertNotEqual(
                nodes[node_id].get("props", {}).get("visibility"),
                "hidden",
                f"{node_id} is a host geometry anchor and must remain visible",
            )

    def test_legacy_gallery_and_fake_console_are_hidden_from_product_hit_testing(self):
        document = load_zui(DRAWER_ASSET)
        nodes = document["nodes"]
        hidden_roots = {
            "retired_showcase_tabs": ("drawer_tab_components", "drawer_tab_console"),
            "retired_showcase_content": ("component_body", "console_body"),
        }

        self.assertEqual(
            child_ids(nodes["drawer_tabs"]), ["retired_showcase_tabs"]
        )
        self.assertEqual(
            child_ids(nodes["component_drawer_content"]),
            ["retired_showcase_content"],
        )

        hidden_descendants = set()
        for root_id, expected_children in hidden_roots.items():
            root = nodes[root_id]
            self.assertEqual(root.get("props", {}).get("visibility"), "hidden")
            self.assertEqual(tuple(child_ids(root)), expected_children)
            hidden_descendants.add(root_id)
            hidden_descendants.update(descendants(nodes, root_id))

        event_nodes = {
            node_id for node_id, node in nodes.items() if node.get("events")
        }
        self.assertGreater(len(event_nodes), 20)
        self.assertTrue(
            event_nodes <= hidden_descendants,
            "every legacy component-gallery event must inherit Hidden visibility",
        )

    def test_product_window_mounts_drawer_as_geometry_region_not_as_component_lab(self):
        window = load_zui(WORKBENCH_WINDOW)
        nodes = window["nodes"]
        imports = window["imports"]["widgets"]

        self.assertEqual(
            nodes["component_drawer"]["component"], "WorkbenchComponentDrawer"
        )
        self.assertEqual(
            nodes["component_drawer"]["control_id"],
            "WorkbenchWindowComponentDrawerRegion",
        )
        self.assertTrue(
            any("workbench_component_drawer.zui#WorkbenchComponentDrawer" in item for item in imports)
        )
        self.assertFalse(any("material_component_lab.zui" in item for item in imports))


if __name__ == "__main__":
    unittest.main()
