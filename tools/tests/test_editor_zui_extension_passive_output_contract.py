import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
EXTENSION_ROOT = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/modules/extensions"
)
SCAN_ONLY_WORKSPACE = "ui_asset_editor"
EXPECTED_PASSIVE_OUTPUT_COUNT = 43
RUNTIME_SOURCE_ROOTS = (
    REPO_ROOT
    / (
        "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
        "workbench/extension_module_navigation/specs"
    ),
    REPO_ROOT
    / (
        "zircon_editor/src/ui/template_runtime/builtin/"
        "workbench_extension_module_template_bindings"
    ),
)
RUNTIME_SOURCE_FILES = (
    REPO_ROOT / "zircon_editor/src/ui/retained_host/workbench_preview_actions.rs",
)


def extension_slug(path: Path) -> str:
    prefix = "workbench_extension_"
    suffix = "_workspace"
    stem = path.stem
    assert stem.startswith(prefix) and stem.endswith(suffix)
    return stem[len(prefix) : -len(suffix)]


def passive_outputs():
    outputs = []
    for path in sorted(EXTENSION_ROOT.rglob("workbench_extension_*_workspace.zui")):
        slug = extension_slug(path)
        if slug == SCAN_ONLY_WORKSPACE:
            continue
        with path.open("rb") as source:
            nodes = tomllib.load(source)["nodes"]
        for node_name, node in nodes.items():
            if node_name.endswith("_output") and node.get("control_id", "").endswith(
                "OutputRow"
            ):
                outputs.append(
                    (
                        path,
                        node_name,
                        f"workbench.extension.{slug}.output.select",
                    )
                )
        if slug == "blend_space":
            outputs.append(
                (
                    path,
                    "blend_space_preview_timeline",
                    "workbench.extension.blend_space.output.select",
                )
            )
    return outputs


class EditorZuiExtensionPassiveOutputContractTests(unittest.TestCase):
    def test_extension_output_readouts_are_passive(self):
        outputs = passive_outputs()
        self.assertEqual(EXPECTED_PASSIVE_OUTPUT_COUNT, len(outputs))
        for path, node_name, _ in outputs:
            with self.subTest(workspace=path.name, node=node_name):
                with path.open("rb") as source:
                    node = tomllib.load(source)["nodes"][node_name]
                self.assertNotIn("events", node)
                props = node["props"]
                for property_name in (
                    "input_interactive",
                    "input_clickable",
                    "input_hoverable",
                    "input_focusable",
                ):
                    self.assertIs(False, props[property_name])

    def test_extension_output_actions_have_no_runtime_registration(self):
        runtime_paths = list(RUNTIME_SOURCE_FILES)
        for root in RUNTIME_SOURCE_ROOTS:
            runtime_paths.extend(root.rglob("*.rs"))
        runtime_source = "\n".join(
            path.read_text(encoding="utf-8") for path in runtime_paths
        )
        for _, _, action_id in passive_outputs():
            with self.subTest(action=action_id):
                self.assertNotIn(action_id, runtime_source)


if __name__ == "__main__":
    unittest.main()
