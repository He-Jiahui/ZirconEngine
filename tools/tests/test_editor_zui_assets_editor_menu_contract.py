import re
import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
ASSETS_WORKSPACE = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/modules/core/assets/"
    "workbench_assets_workspace.zui"
)
WORKBENCH_WINDOW = REPO_ROOT / "zircon_editor/assets/ui/editor/windows/workbench_window.zui"
ASSET_EDITOR_MENU = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/asset_editor_menu.rs"
)
WINDOW_MENU_STATE = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/window_menu_state.rs"
)
CONTROL_DISPATCH = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/workbench/control.rs"
)
NAVIGATION_SPECS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/extension_module_navigation/specs"
)


CATEGORIES = {
    "world": {
        "trigger": "WorkbenchAssetsWorldTools",
        "menu": "WorkbenchAssetsWorldToolsMenu",
        "open_action": "workbench.module.assets.world_tools.open",
        "items": [
            ("Terrain Editor", "terrain_editor", "terrain_editor"),
            ("Foliage Editor", "foliage_editor", "foliage_editor"),
            ("Level Streaming", "level_streaming", "level_streaming"),
            ("Level Variant", "level_variant", "level_variant"),
            ("Prefab Editor", "prefab_editor", "prefab_editor"),
            ("Scatter Editor", "scatter_editor", "scatter_editor"),
            ("Volume Editor", "volume_editor", "volume_editor"),
            ("Weather Editor", "weather_editor", "weather_editor"),
        ],
    },
    "gameplay": {
        "trigger": "WorkbenchAssetsGameplayTools",
        "menu": "WorkbenchAssetsGameplayToolsMenu",
        "open_action": "workbench.module.assets.gameplay_tools.open",
        "items": [
            ("Spawn Rules", "spawn_rules", "spawn_rules"),
            ("World State", "world_state", "world_state"),
            ("Collision Proxy", "collision_proxy", "collision_proxy"),
            ("Physics Collision", "physics_collision", "physics_collision"),
            ("Navmesh AI", "navmesh_ai", "navmesh_ai"),
            ("Lobby Editor", "lobby_editor", "lobby_editor"),
            ("Matchmaking Editor", "matchmaking_editor", "matchmaking_editor"),
        ],
    },
    "production": {
        "trigger": "WorkbenchAssetsProductionTools",
        "menu": "WorkbenchAssetsProductionToolsMenu",
        "open_action": "workbench.module.assets.production_tools.open",
        "items": [
            ("Data Table", "data_table", "data_table"),
            ("Source Control", "source_control", "source_control"),
            ("Build Export", "build_export", "build_export"),
            ("Automation Report", "automation_report", "automation_report"),
            ("Project Overview", "project_overview", "project_overview"),
            ("Plugin Manager", "plugin_manager", "plugin_manager"),
            ("Save Data", "save_data", "save_data"),
        ],
    },
}


OLD_BUTTON_PATTERN = re.compile(
    r"WorkbenchAssets(?:TerrainEditor|FoliageEditor|LevelStreaming|LevelVariant|"
    r"PrefabEditor|ScatterEditor|VolumeEditor|WeatherEditor|SpawnRules|WorldState|"
    r"CollisionProxy|PhysicsCollision|NavmeshAi|LobbyEditor|MatchmakingEditor|"
    r"DataTable|SourceControl|BuildExport|AutomationReport|ProjectOverview|"
    r"PluginManager|SaveData)Button"
)


def load_zui(path):
    with path.open("rb") as source:
        return tomllib.load(source)


def menu_item_identity(raw):
    label, flags = raw.split("|", 1)
    action = next(flag[7:] for flag in flags.split(",") if flag.startswith("action="))
    return label, action


class EditorZuiAssetsEditorMenuContractTests(unittest.TestCase):
    def test_assets_details_use_three_discoverable_menu_triggers(self):
        document = load_zui(ASSETS_WORKSPACE)
        nodes = document["nodes"]
        self.assertEqual(
            [
                "assets_type_property_row",
                "assets_path_property_row",
                "assets_owner_property_row",
                "assets_tools_title",
                "assets_world_tools",
                "assets_gameplay_tools",
                "assets_production_tools",
            ],
            [
                child["node"]
                for child in nodes["assets_right_content"]["children"]
            ],
        )
        self.assertEqual("Open Tool", nodes["assets_tools_title"]["props"]["text"])

        source = ASSETS_WORKSPACE.read_text(encoding="utf-8")
        self.assertIsNone(OLD_BUTTON_PATTERN.search(source))
        for category, contract in CATEGORIES.items():
            node = nodes[f"assets_{category}_tools"]
            self.assertEqual("WorkbenchButton", node["component"])
            self.assertEqual(contract["trigger"], node["control_id"])
            self.assertEqual("outlined", node["props"]["button_variant"])
            self.assertEqual(
                contract["open_action"], node["events"][0]["route"]
            )

    def test_window_overlay_owns_the_three_anchored_popup_menus(self):
        document = load_zui(WORKBENCH_WINDOW)
        nodes = document["nodes"]
        root_children = [child["node"] for child in nodes["root"]["children"]]

        for category, contract in CATEGORIES.items():
            node_name = f"assets_{category}_tools_menu"
            self.assertIn(node_name, root_children)
            menu = nodes[node_name]
            self.assertEqual("WorkbenchPopupMenu", menu["component"])
            self.assertEqual(contract["menu"], menu["control_id"])
            self.assertEqual(
                {
                    "popup_anchor": {
                        "kind": "control",
                        "control_id": contract["trigger"],
                    },
                    "open_property": "popup_open",
                },
                menu["widget"],
            )
            self.assertEqual("collapsed", menu["props"]["visibility"])
            self.assertFalse(menu["props"]["popup_open"])
            self.assertEqual(
                [
                    (label, f"menu.item.assets.{menu_action}")
                    for label, menu_action, _ in contract["items"]
                ],
                [menu_item_identity(raw) for raw in menu["props"]["menu_items"]],
            )

    def test_rust_menu_authority_maps_every_item_to_the_existing_extension_action(self):
        source = ASSET_EDITOR_MENU.read_text(encoding="utf-8")
        expected = []
        for contract in CATEGORIES.values():
            for _, menu_action, extension_action in contract["items"]:
                expected.append(
                    (
                        contract["menu"],
                        f"menu.item.assets.{menu_action}",
                        f"workbench.extension.{extension_action}.open",
                    )
                )
        self.assertEqual(22, len(expected))
        for menu_control, menu_action, extension_action in expected:
            self.assertIn(f'menu_control_id: "{menu_control}"', source)
            self.assertIn(f'menu_action_id: "{menu_action}"', source)
            self.assertIn(f'extension_action_id: "{extension_action}"', source)

        self.assertIn("self.apply_reference_menu_action(", source)
        self.assertIn("EditorUiBindingPayload::menu_action(command.extension_action_id)", source)
        dispatch = CONTROL_DISPATCH.read_text(encoding="utf-8")
        self.assertIn("dispatch_workbench_asset_editor_menu_item_state", dispatch)

    def test_window_menu_state_and_navigation_specs_use_menu_control_identity(self):
        state = WINDOW_MENU_STATE.read_text(encoding="utf-8")
        for contract in CATEGORIES.values():
            self.assertIn(f'trigger_control_id: "{contract["trigger"]}"', state)
            self.assertIn(f'menu_control_id: "{contract["menu"]}"', state)
            self.assertIn(f'"{contract["open_action"]}"', state)

        specs = "\n".join(
            path.read_text(encoding="utf-8")
            for path in NAVIGATION_SPECS.rglob("*.rs")
        )
        self.assertIsNone(OLD_BUTTON_PATTERN.search(specs))
        for contract in CATEGORIES.values():
            self.assertIn(contract["menu"], specs)


if __name__ == "__main__":
    unittest.main()
