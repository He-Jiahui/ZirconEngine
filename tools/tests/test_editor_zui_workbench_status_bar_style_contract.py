import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
STATUS_BAR = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/shell/"
    "workbench_status_bar.zui"
)
TOP_TOOLBAR = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/shell/"
    "workbench_top_toolbar.zui"
)
VIEWPORT_TOOLBAR = (
    REPO_ROOT / "zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.zui"
)
WORKBENCH_BINDINGS = REPO_ROOT / (
    "zircon_editor/src/ui/template_runtime/builtin/"
    "workbench_window_template_bindings.rs"
)
THEME = REPO_ROOT / "zircon_editor/assets/ui/theme/editor_workbench_strict.zui"
TOKENS = REPO_ROOT / "zircon_editor/assets/ui/editor/theme/editor_tokens.zui"


def load_document(path: Path) -> dict:
    with path.open("rb") as source:
        return tomllib.load(source)


def load_rules() -> dict[str, dict]:
    theme = load_document(THEME)
    return {
        rule["selector"]: rule["set"]["self"]
        for stylesheet in theme["stylesheets"]
        for rule in stylesheet.get("rules", [])
    }


class EditorZuiWorkbenchStatusBarStyleContractTests(unittest.TestCase):
    def test_active_task_status_bar_fits_the_mid_width_product_target(self):
        document = load_document(STATUS_BAR)
        nodes = document["nodes"]
        tokens = load_document(TOKENS)
        token_values = {
            "$editor.chrome.status_bar.height": tokens["chrome"]["status_bar_height"],
            "$editor.density.gap.medium": tokens["density"]["gap_medium"],
        }

        def resolve(value):
            return token_values.get(value, value)

        ready_width = nodes["status_ready"]["layout"]["width"]
        self.assertEqual(128.0, ready_width["min"])
        self.assertEqual(160.0, ready_width["preferred"])
        self.assertEqual("Stretch", ready_width["stretch"])

        active_task_children = nodes["status_bar"]["children"]
        minimum_width = sum(
            resolve(nodes[child["node"]]["layout"]["width"]["min"])
            for child in active_task_children
        )
        root_props = nodes["status_bar"]["props"]
        minimum_width += resolve(root_props["layout_padding_left"])
        minimum_width += resolve(root_props["layout_padding_right"])

        self.assertLessEqual(minimum_width, 900.0)

    def test_status_shortcuts_keep_unique_identity_and_share_viewport_semantics(self):
        status_nodes = load_document(STATUS_BAR)["nodes"]
        top_nodes = load_document(TOP_TOOLBAR)["nodes"]
        viewport_nodes = load_document(VIEWPORT_TOOLBAR)["nodes"]

        self.assertEqual(
            [
                {
                    "id": "Workbench/ToggleSnapFromStatus",
                    "event": "Click",
                    "route": "workbench.status.toggle_snap",
                }
            ],
            status_nodes["status_snap_icon"]["events"],
        )
        self.assertEqual(
            [
                {
                    "id": "Workbench/FrameSelectionFromStatus",
                    "event": "Click",
                    "route": "workbench.status.frame_selection",
                }
            ],
            status_nodes["status_target_icon"]["events"],
        )
        self.assertEqual("Tool/ToggleSnap", top_nodes["tool_snap"]["events"][0]["id"])
        self.assertEqual(
            "ViewportToolbar/FrameSelection",
            viewport_nodes["frame_selection"]["events"][0]["id"],
        )
        self.assertEqual("Run/Stop", top_nodes["run_stop"]["events"][0]["id"])
        self.assertEqual(
            "ViewportToolbar/ExitPlayMode",
            viewport_nodes["exit_play"]["events"][0]["id"],
        )

        governed_nodes = (
            status_nodes["status_snap_icon"],
            status_nodes["status_target_icon"],
            top_nodes["tool_snap"],
            top_nodes["run_stop"],
            viewport_nodes["frame_selection"],
            viewport_nodes["exit_play"],
        )
        binding_ids = [node["events"][0]["id"] for node in governed_nodes]
        routes = [
            node["events"][0]["route"]
            for node in governed_nodes
            if "route" in node["events"][0]
        ]
        self.assertEqual(len(binding_ids), len(set(binding_ids)))
        self.assertEqual(len(routes), len(set(routes)))

        bindings = WORKBENCH_BINDINGS.read_text(encoding="utf-8")
        production_bindings = bindings.split("#[cfg(test)]", maxsplit=1)[0]
        self.assertIn('"ToggleSnapFromStatus"', production_bindings)
        self.assertIn('"FrameSelectionFromStatus"', production_bindings)
        self.assertEqual(2, production_bindings.count("GridMode::VisibleAndSnap"))
        self.assertIn("ViewportCommand::FrameSelection", production_bindings)

    def test_status_actions_share_the_quiet_action_state_recipe(self):
        nodes = load_document(STATUS_BAR)["nodes"]

        for node_id in ("status_snap_icon", "status_target_icon"):
            self.assertIn("workbench-quiet-action", nodes[node_id]["classes"])
            self.assertIn("workbench-status-right-icon", nodes[node_id]["classes"])

        self.assertNotIn(
            "workbench-quiet-action",
            nodes["status_world_icon"]["classes"],
            "The World glyph is a passive indicator, not a painted action",
        )

    def test_status_readouts_and_passive_indicator_have_no_persistent_tiles(self):
        rules = load_rules()

        for selector in (
            ".workbench-status-right-control",
            ".workbench-status-right-icon",
        ):
            self.assertEqual("transparent", rules[selector]["background_color"])
            self.assertEqual("transparent", rules[selector]["border_color"])

        self.assertEqual(
            "$editor.control.radius.small",
            rules[".workbench-status-right-icon"]["radius"],
        )

    def test_status_actions_reveal_surface_only_for_interaction_or_state(self):
        rules = load_rules()

        for selector, expected_surface in (
            (".workbench-quiet-action:hovered", "$workbench_hover"),
            (".workbench-quiet-action:pressed", "$workbench_active"),
            (".workbench-quiet-action:checked", "$workbench_selected"),
            (".workbench-quiet-action:selected", "$workbench_selected"),
        ):
            self.assertEqual(expected_surface, rules[selector]["background_color"])


if __name__ == "__main__":
    unittest.main()
