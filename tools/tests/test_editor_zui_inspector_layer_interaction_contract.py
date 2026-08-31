import re
import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
INSPECTOR_PANEL = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/shell/"
    "workbench_inspector_panel.zui"
)
INSPECTOR_SNAPSHOT = REPO_ROOT / (
    "zircon_editor/src/ui/workbench/snapshot/data/inspector_snapshot.rs"
)
SNAPSHOT_BUILD = REPO_ROOT / (
    "zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs"
)
DATA_SYNC = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/data_sync.rs"
)
LAYER_EDIT = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/render_layer_edit.rs"
)
APP_EDIT = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/app/pane_surface_actions/"
    "workbench_surface/edit.rs"
)
TEMPLATE_BINDINGS = REPO_ROOT / (
    "zircon_editor/src/ui/template_runtime/builtin/"
    "workbench_window_template_bindings.rs"
)
PREVIEW_ACTIONS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/workbench_preview_actions.rs"
)


class EditorZuiInspectorLayerInteractionContractTests(unittest.TestCase):
    def test_fake_tag_and_named_layer_dropdowns_are_removed(self):
        with INSPECTOR_PANEL.open("rb") as source:
            document = tomllib.load(source)

        nodes = document["nodes"]
        self.assertNotIn("inspector_tag", nodes)
        self.assertNotIn("inspector_layer", nodes)
        self.assertFalse(
            any(
                reference.endswith("#WorkbenchDropdown")
                for reference in document["imports"]["widgets"]
            )
        )

        metadata = nodes["inspector_metadata"]
        self.assertEqual(
            ["inspector_layer_label", "inspector_layer_mask"],
            [child["node"] for child in metadata["children"]],
        )
        self.assertEqual("WorkbenchLabel", nodes["inspector_layer_label"]["component"])
        self.assertEqual("Layer mask", nodes["inspector_layer_label"]["props"]["text"])

    def test_layer_mask_has_edit_and_transactional_submit_routes(self):
        with INSPECTOR_PANEL.open("rb") as source:
            nodes = tomllib.load(source)["nodes"]

        layer = nodes["inspector_layer_mask"]
        self.assertEqual("WorkbenchAxisValueField", layer["component"])
        self.assertEqual("WorkbenchInspectorRenderLayerMask", layer["control_id"])
        self.assertEqual(
            [
                {
                    "id": "Inspector/RenderLayerMaskEdit",
                    "event": "Change",
                    "route": "workbench.inspector.render_layer_mask.edit",
                },
                {
                    "id": "Inspector/RenderLayerMaskCommit",
                    "event": "Submit",
                    "route": "workbench.inspector.render_layer_mask.commit",
                },
            ],
            layer["events"],
        )

    def test_enabled_inspector_controls_do_not_expose_dead_interaction(self):
        with INSPECTOR_PANEL.open("rb") as source:
            nodes = tomllib.load(source)["nodes"]

        interactive_components = {
            "WorkbenchAxisValueField",
            "WorkbenchButton",
            "WorkbenchComponentPropertyRow",
            "WorkbenchDropdown",
            "WorkbenchField",
            "WorkbenchTab",
        }
        dead_controls = []
        for node_name, node in nodes.items():
            if node.get("component") not in interactive_components:
                continue
            props = node.get("props", {})
            explicitly_inert = props.get("disabled") is True or all(
                props.get(flag) is False
                for flag in (
                    "input_interactive",
                    "input_clickable",
                    "input_hoverable",
                    "input_focusable",
                )
            )
            if not explicitly_inert and not node.get("events"):
                dead_controls.append(node_name)

        self.assertEqual([], dead_controls)

    def test_snapshot_and_bridge_share_the_runtime_render_layer_authority(self):
        snapshot = INSPECTOR_SNAPSHOT.read_text(encoding="utf-8")
        build = SNAPSHOT_BUILD.read_text(encoding="utf-8")
        sync = DATA_SYNC.read_text(encoding="utf-8")

        self.assertIn("pub render_layer_mask: u32", snapshot)
        self.assertRegex(build, r"scene\s*\.render_layer_mask\(id\)")
        self.assertIn("default_render_layer_mask", build)
        self.assertIn("inspector.render_layer_mask.to_string()", sync)
        self.assertIn('"WorkbenchInspectorRenderLayerMask"', sync)

    def test_submit_uses_a_typed_unsigned_reflected_field_transaction(self):
        layer_edit = LAYER_EDIT.read_text(encoding="utf-8")
        app_edit = APP_EDIT.read_text(encoding="utf-8")
        bindings = TEMPLATE_BINDINGS.read_text(encoding="utf-8")
        preview_actions = PREVIEW_ACTIONS.read_text(encoding="utf-8")

        self.assertIn(
            '"zircon_runtime::scene::components::RenderLayerMask.mask"',
            layer_edit,
        )
        self.assertIn("UiBindingValue::Unsigned", layer_edit)
        self.assertIn("parse_render_layer_mask", layer_edit)
        self.assertIn(
            "dispatch_componentized_workbench_render_layer_mask_commit",
            app_edit,
        )
        self.assertIn('"RenderLayerMaskEdit"', bindings)
        self.assertIn('"RenderLayerMaskCommit"', bindings)
        self.assertIn('"inspector.render_layer_mask.edit"', preview_actions)
        self.assertIn('"inspector.render_layer_mask.commit"', preview_actions)


if __name__ == "__main__":
    unittest.main()
