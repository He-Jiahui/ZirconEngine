import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
WORKBENCH_WINDOW = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/windows/workbench_window.zui"
)
PANE_MENU_PROJECTION = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/"
    "pane_menu_projection.rs"
)
ASSET_CREATION_MENU = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/asset_creation_menu.rs"
)
MODULE_OVERFLOW_MENU = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/module_overflow_menu.rs"
)
RUNTIME_POPUP_MENU = REPO_ROOT / (
    "zircon_runtime/src/ui/surface/render/popup_menu.rs"
)
WORKBENCH_COMPONENTS = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench"
)
WORKBENCH_CONTEXT_MENU_PROVIDER = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/"
    "workbench_context_menu/provider.rs"
)


EXPECTED_ACTIONS = {
    "toolbar_main_menu": {
        "Asset Browser": "menu.item.asset_browser",
        "Open Project": "menu.item.open_project",
        "Save Project": "menu.item.save_project",
        "Command Palette": "menu.item.command_palette",
    },
    "toolbar_run_mode_menu": {
        "Play In Editor": "menu.item.play_in_editor",
        "Simulate": "menu.item.simulate",
        "Standalone": "menu.item.standalone",
        "Network Preview": "menu.item.network_preview",
    },
    "toolbar_layout_menu": {
        "Default Layout": "menu.item.default_layout",
        "Gameplay Layout": "menu.item.gameplay_layout",
        "Rendering Layout": "menu.item.rendering_layout",
        "Reset Layout": "menu.item.reset_layout",
    },
    "toolbar_module_overflow_menu": {
        "Behavior": "menu.item.behavior",
        "Render": "menu.item.render",
        "Assets": "menu.item.assets",
        "VFX": "menu.item.v_f_x",
        "HUD": "menu.item.h_u_d",
    },
}

EXPECTED_GENERIC_POPUP_ACTIONS = {
    "New": "menu.item.new",
    "Open": "menu.item.open",
    "Save": "menu.item.save",
    "Delete": "menu.item.delete",
    "More Tools": "menu.item.more_tools",
}

EXPECTED_CONTEXT_ACTIONS = {
    "menu.item.open",
    "menu.item.rename",
    "menu.item.duplicate",
    "menu.item.delete",
    "menu.item.open_module",
    "menu.item.pin_module",
    "menu.item.reset_module",
    "menu.item.inspect",
    "menu.item.copy_id",
    "menu.item.reveal_in_workbench",
}


def menu_item_identity(raw: str):
    if raw == "---":
        return None
    parts = raw.split("|", 2)
    label = parts[0].strip()
    flags = parts[1].split(",") if len(parts) > 1 else []
    action = next(
        (
            flag.split("=", 1)[1].strip()
            for flag in flags
            if flag.strip().startswith("action=")
        ),
        None,
    )
    return label, action


class EditorToolbarMenuActionIdentityContractTests(unittest.TestCase):
    def test_product_toolbar_menu_labels_do_not_own_action_identity(self):
        document = tomllib.loads(WORKBENCH_WINDOW.read_text(encoding="utf-8-sig"))
        nodes = document["nodes"]

        for node_id, expected in EXPECTED_ACTIONS.items():
            actual = dict(
                identity
                for raw in nodes[node_id]["props"]["menu_items"]
                if (identity := menu_item_identity(raw)) is not None
            )
            self.assertEqual(expected, actual, node_id)

    def test_workbench_product_menu_defaults_do_not_derive_identity_from_labels(self):
        missing = []
        for path in sorted(WORKBENCH_COMPONENTS.rglob("*.zui")):
            document = tomllib.loads(path.read_text(encoding="utf-8-sig"))
            for node_id, node in document.get("nodes", {}).items():
                items = node.get("props", {}).get("menu_items")
                if not isinstance(items, list):
                    continue
                actual = dict(
                    identity
                    for raw in items
                    if (identity := menu_item_identity(raw)) is not None
                )
                for label, action_id in actual.items():
                    if action_id is None:
                        missing.append(f"{path.relative_to(REPO_ROOT)}::{node_id}::{label}")

        self.assertEqual([], missing)

        primitive = tomllib.loads(
            (WORKBENCH_COMPONENTS / "primitives/feedback/workbench_popup_menu.zui")
            .read_text(encoding="utf-8-sig")
        )
        actual = dict(
            menu_item_identity(raw)
            for raw in primitive["nodes"]["root"]["props"]["menu_items"]
        )
        self.assertEqual(EXPECTED_GENERIC_POPUP_ACTIONS, actual)

        provider_source = WORKBENCH_CONTEXT_MENU_PROVIDER.read_text(encoding="utf-8")
        for action_id in EXPECTED_CONTEXT_ACTIONS:
            self.assertIn(f"action={action_id}", provider_source)

    def test_editor_projection_prefers_explicit_action_over_display_label(self):
        source = PANE_MENU_PROJECTION.read_text(encoding="utf-8")
        runtime_source = RUNTIME_POPUP_MENU.read_text(encoding="utf-8")

        self.assertIn('explicit_menu_action_id(flags)', source)
        self.assertIn('.unwrap_or_else(|| menu_item_action_id(label))', source)
        self.assertIn('let id = flag_value(flags, "action").unwrap_or(raw_label);', runtime_source)
        self.assertIn('.unwrap_or(raw_label)', runtime_source)

    def test_dynamic_toolbar_menu_rebuilds_preserve_explicit_action_ids(self):
        asset_menu = ASSET_CREATION_MENU.read_text(encoding="utf-8")
        overflow_menu = MODULE_OVERFLOW_MENU.read_text(encoding="utf-8")

        for action_id in EXPECTED_ACTIONS["toolbar_main_menu"].values():
            self.assertIn(f"action={action_id}", asset_menu)
        self.assertIn('format!("action={}", self.menu_action_id)', overflow_menu)


if __name__ == "__main__":
    unittest.main()
