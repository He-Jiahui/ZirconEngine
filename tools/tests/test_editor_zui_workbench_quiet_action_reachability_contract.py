import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
TRANSPORT = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/composites/animation/"
    "workbench_transport_controls.zui"
)
SCENE_TREE = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/shell/"
    "workbench_scene_tree_panel.zui"
)
COMPONENT_DRAWER = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/shell/"
    "workbench_component_drawer.zui"
)
WORKBENCH_BINDINGS = REPO_ROOT / (
    "zircon_editor/src/ui/template_runtime/builtin/"
    "workbench_window_template_bindings.rs"
)
WORKBENCH_PREVIEW_ACTIONS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/workbench_preview_actions.rs"
)


def load_nodes(path: Path) -> dict:
    with path.open("rb") as source:
        return tomllib.load(source)["nodes"]


class EditorZuiWorkbenchQuietActionReachabilityContractTests(unittest.TestCase):
    def test_transport_controls_use_quiet_chrome_without_forking_routes(self):
        nodes = load_nodes(TRANSPORT)
        expected_routes = {
            "record": "workbench.extension.animation_transport.record.toggle",
            "play": "workbench.extension.animation_transport.play.invoke",
            "pause": "workbench.extension.animation_transport.pause.invoke",
            "previous": "workbench.extension.animation_transport.previous.invoke",
            "next": "workbench.extension.animation_transport.next.invoke",
            "loop": "workbench.extension.animation_transport.loop.toggle",
        }

        for node_id, expected_route in expected_routes.items():
            self.assertIn("workbench-quiet-action", nodes[node_id]["classes"])
            self.assertEqual(expected_route, nodes[node_id]["events"][0]["route"])

    def test_scene_tree_trailing_actions_are_quiet_and_keep_command_identity(self):
        nodes = load_nodes(SCENE_TREE)
        expected_events = {
            "scene_filter_button": {
                "id": "Hierarchy/OpenFilter",
                "event": "Click",
                "route": "workbench.hierarchy.open_filter",
            },
            "scene_add_button": {
                "id": "Hierarchy/AddEntity",
                "event": "Click",
                "route": "workbench.hierarchy.add_entity",
            },
        }

        for node_id, expected_event in expected_events.items():
            self.assertIn("workbench-quiet-action", nodes[node_id]["classes"])
            self.assertEqual(expected_event, nodes[node_id]["events"][0])

    def test_component_lab_icon_button_samples_remain_framed(self):
        nodes = load_nodes(COMPONENT_DRAWER)
        sample_ids = (
            "mini_add",
            "mini_folder",
            "mini_save",
            "mini_delete",
            "mini_eye",
            "mini_eye_off",
            "mini_lock",
            "mini_more",
        )

        for node_id in sample_ids:
            self.assertNotIn("workbench-quiet-action", nodes[node_id]["classes"])

    def test_component_lab_icon_button_samples_dispatch_named_actions(self):
        nodes = load_nodes(COMPONENT_DRAWER)
        bindings = WORKBENCH_BINDINGS.read_text(encoding="utf-8")
        preview_actions = WORKBENCH_PREVIEW_ACTIONS.read_text(encoding="utf-8")
        expected_actions = {
            "mini_add": ("MiniAdd", "component_lab.icon_button.add"),
            "mini_folder": ("MiniOpen", "component_lab.icon_button.open"),
            "mini_save": ("MiniSave", "component_lab.icon_button.save"),
            "mini_delete": ("MiniDelete", "component_lab.icon_button.delete"),
            "mini_eye": ("MiniShow", "component_lab.icon_button.show"),
            "mini_eye_off": ("MiniHide", "component_lab.icon_button.hide"),
            "mini_lock": ("MiniLock", "component_lab.icon_button.lock"),
            "mini_more": ("MiniMore", "component_lab.icon_button.more"),
        }

        for node_id, (binding_name, action) in expected_actions.items():
            self.assertEqual(
                [
                    {
                        "id": f"ComponentLab/{binding_name}",
                        "event": "Click",
                        "route": action,
                    }
                ],
                nodes[node_id].get("events"),
            )
            self.assertIn(f'("{binding_name}", "{action}")', bindings)
            self.assertIn(f'"{action}"', preview_actions)

        self.assertIn("EditorUiBindingPayload::menu_action(action)", bindings)


if __name__ == "__main__":
    unittest.main()
