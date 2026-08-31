import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
EXTENSION_ROOT = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/modules/extensions"
)
PROPERTY_ROW_IMPORT = (
    "res://ui/editor/components/workbench/composites/inputs/"
    "workbench_property_editor_row.zui#WorkbenchPropertyEditorRow"
)
EXCLUDED_WORKSPACES = {"blend_space", "ui_asset_editor"}
LABEL_ACRONYMS = {
    "ai": "AI",
    "dpi": "DPI",
    "gpu": "GPU",
    "id": "ID",
    "lod": "LOD",
    "uv": "UV",
}


def extension_slug(path: Path) -> str:
    prefix = "workbench_extension_"
    suffix = "_workspace"
    return path.stem[len(prefix) : -len(suffix)]


def editable_nodes(document):
    for node_name, node in document["nodes"].items():
        if any(
            event.get("event") == "Change" for event in node.get("events", [])
        ):
            yield node_name, node


def expected_label(node):
    edit_routes = [
        event["route"]
        for event in node["events"]
        if event.get("event") == "Change"
        and event["route"].endswith(".edit")
    ]
    if len(edit_routes) != 1:
        raise AssertionError(f"expected one .edit route, found {edit_routes}")
    field_name = edit_routes[0].removesuffix(".edit").rsplit(".", 1)[-1]
    return " ".join(
        LABEL_ACRONYMS.get(word, word.capitalize())
        for word in field_name.split("_")
    )


class EditorZuiExtensionDetailsPropertyRowsContractTests(unittest.TestCase):
    def test_extension_command_inputs_use_name_value_property_rows(self):
        controls = 0
        workspaces = 0
        for path in sorted(EXTENSION_ROOT.rglob("workbench_extension_*_workspace.zui")):
            if extension_slug(path) in EXCLUDED_WORKSPACES:
                continue
            workspaces += 1
            with path.open("rb") as source:
                document = tomllib.load(source)
            self.assertIn(PROPERTY_ROW_IMPORT, document["imports"]["widgets"])
            nodes = document["nodes"]
            for node_name, node in editable_nodes(document):
                controls += 1
                wrapper_name = f"{node_name}_property_row"
                self.assertIn(wrapper_name, nodes, path.name)
                wrapper = nodes[wrapper_name]
                self.assertEqual("WorkbenchPropertyEditorRow", wrapper["component"])
                self.assertEqual(expected_label(node), wrapper["props"]["text"])
                self.assertNotIn("events", wrapper)
                self.assertEqual(node["layout"], wrapper["layout"])
                self.assertEqual(
                    [{"node": node_name, "slot": {"name": "value"}}],
                    wrapper["children"],
                )
                parent_names = [
                    parent_name
                    for parent_name, parent in nodes.items()
                    if any(
                        child.get("node") == node_name
                        for child in parent.get("children", [])
                    )
                ]
                self.assertEqual([wrapper_name], parent_names)
                display_value = node.get("props", {}).get("value_text")
                if isinstance(display_value, str):
                    repeated_prefix = f"{expected_label(node)}:"
                    self.assertFalse(
                        display_value.casefold().startswith(
                            repeated_prefix.casefold()
                        ),
                        f"{path.name}:{node_name} repeats its property label",
                    )
                    for option in node["props"].get("options", []):
                        option_label = option.split("|label=", 1)[-1]
                        self.assertFalse(
                            option_label.casefold().startswith(
                                repeated_prefix.casefold()
                            ),
                            f"{path.name}:{node_name} option repeats its property label",
                        )

        self.assertEqual(42, workspaces)
        self.assertEqual(126, controls)


if __name__ == "__main__":
    unittest.main()
