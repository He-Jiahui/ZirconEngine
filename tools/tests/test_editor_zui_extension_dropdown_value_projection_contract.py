import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
WORKBENCH_ASSET_ROOT = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench"
)
EXTENSION_WORKSPACE_ROOT = WORKBENCH_ASSET_ROOT / "modules/extensions"
UI_ASSET_EDITOR_MARKERS = ("ui_asset", "asset_editor")


def extension_workspace_assets():
    return sorted(
        path
        for path in EXTENSION_WORKSPACE_ROOT.rglob("*.zui")
        if not any(marker in path.as_posix().lower() for marker in UI_ASSET_EDITOR_MARKERS)
    )


def product_workbench_assets():
    return sorted(
        path
        for path in WORKBENCH_ASSET_ROOT.rglob("*.zui")
        if not any(marker in path.as_posix().lower() for marker in UI_ASSET_EDITOR_MARKERS)
    )


def structured_option(raw_option):
    option_id, _, metadata = raw_option.partition("|")
    label = option_id
    for entry in metadata.split(","):
        key, separator, value = entry.strip().partition("=")
        if separator and key.lower() in {"label", "text"} and value.strip():
            label = value.strip()
            break
    return option_id.strip(), label


class EditorZuiExtensionDropdownValueProjectionContractTests(unittest.TestCase):
    def test_machine_values_with_display_labels_use_structured_options(self):
        labeled_dropdown_count = 0

        for asset_path in product_workbench_assets():
            with asset_path.open("rb") as source:
                nodes = tomllib.load(source).get("nodes", {})

            for node_name, node in nodes.items():
                if node.get("component") != "WorkbenchDropdown":
                    continue
                props = node.get("props", {})
                value = props.get("value")
                value_text = props.get("value_text")
                if value == value_text:
                    continue

                labeled_dropdown_count += 1
                context = f"{asset_path.relative_to(REPO_ROOT)} [{node_name}]"
                options = props.get("options", [])
                projected_options = dict(structured_option(option) for option in options)
                with self.subTest(dropdown=context):
                    self.assertGreater(
                        len(options), 0, f"{context} needs selectable labeled options"
                    )
                    self.assertTrue(
                        all("|" in option for option in options),
                        f"{context} must label every selectable machine value",
                    )
                    self.assertEqual(
                        value_text,
                        projected_options.get(value),
                        f"{context} current value must resolve to its rendered label",
                    )

        self.assertGreater(labeled_dropdown_count, 0)

    def test_product_dropdowns_never_shadow_runtime_selection_text(self):
        dropdown_count = 0

        for asset_path in product_workbench_assets():
            with asset_path.open("rb") as source:
                nodes = tomllib.load(source).get("nodes", {})

            for node_name, node in nodes.items():
                if node.get("component") != "WorkbenchDropdown":
                    continue

                dropdown_count += 1
                props = node.get("props", {})
                context = f"{asset_path.relative_to(REPO_ROOT)} [{node_name}]"
                with self.subTest(dropdown=context):
                    self.assertNotIn(
                        "text",
                        props,
                        f"{context} must not shadow runtime-updated value_text",
                    )
                    self.assertIn("value", props, f"{context} needs a committed value")
                    self.assertIn(
                        "value_text", props, f"{context} needs rendered selection text"
                    )

        self.assertGreater(dropdown_count, 0)

    def test_dropdowns_use_mutable_value_text_instead_of_static_text(self):
        dropdown_count = 0

        for asset_path in extension_workspace_assets():
            with asset_path.open("rb") as source:
                nodes = tomllib.load(source).get("nodes", {})

            for node_name, node in nodes.items():
                if node.get("component") != "WorkbenchDropdown":
                    continue

                dropdown_count += 1
                props = node.get("props", {})
                context = f"{asset_path.relative_to(REPO_ROOT)} [{node_name}]"
                with self.subTest(dropdown=context):
                    self.assertNotIn(
                        "text",
                        props,
                        f"{context} must not shadow runtime-updated value_text",
                    )
                    self.assertIn("value", props, f"{context} needs a committed value")
                    self.assertIn(
                        "value_text", props, f"{context} needs rendered selection text"
                    )
                    self.assertGreater(
                        len(props.get("options", [])),
                        0,
                        f"{context} needs selectable options",
                    )

        self.assertEqual(79, dropdown_count)


if __name__ == "__main__":
    unittest.main()
