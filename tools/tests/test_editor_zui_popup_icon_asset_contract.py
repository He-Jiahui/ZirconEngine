import re
import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
WORKBENCH_COMPONENTS = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench"
)
WORKBENCH_WINDOW = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/windows/workbench_window.zui"
)
CONTEXT_MENU_PROVIDER = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/"
    "workbench_context_menu/provider.rs"
)
ADORNMENT_SELECTION = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_popup_row_adornments/selection.rs"
)
ADORNMENT_DISPATCH = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_popup_row_adornments/glyphs/dispatch.rs"
)
ICON_ALIASES = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "visual_assets/candidates/aliases.rs"
)
ASSET_CREATION_MENU = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/asset_creation_menu.rs"
)

EXTENSION_MENU_ICONS = {
    "menu.item.assets.terrain_editor": "terrain",
    "menu.item.assets.foliage_editor": "leaf",
    "menu.item.assets.level_streaming": "layers",
    "menu.item.assets.level_variant": "branch",
    "menu.item.assets.prefab_editor": "prefab",
    "menu.item.assets.scatter_editor": "sparkles",
    "menu.item.assets.volume_editor": "trigger-volume",
    "menu.item.assets.weather_editor": "sky",
    "menu.item.assets.spawn_rules": "player-start",
    "menu.item.assets.world_state": "globe",
    "menu.item.assets.collision_proxy": "collider",
    "menu.item.assets.physics_collision": "physics",
    "menu.item.assets.navmesh_ai": "navmesh",
    "menu.item.assets.lobby_editor": "gamepad",
    "menu.item.assets.matchmaking_editor": "git-network-outline",
    "menu.item.assets.data_table": "table",
    "menu.item.assets.source_control": "source-control",
    "menu.item.assets.build_export": "build",
    "menu.item.assets.automation_report": "test",
    "menu.item.assets.project_overview": "folder",
    "menu.item.assets.plugin_manager": "plugin",
    "menu.item.assets.save_data": "save",
    "menu.item.ability.sequencer": "sequence",
    "menu.item.ability.montage_editor": "animation-clip",
    "menu.item.ability.blend_space": "curve-bezier",
    "menu.item.ability.pose_library": "skeleton",
    "menu.item.ability.retarget": "bone",
    "menu.item.ability.control_rig": "constraint",
    "menu.item.ability.motion_matching": "animation",
    "menu.item.ability.animation_compression": "archive",
    "menu.item.render.shader_editor": "shader",
    "menu.item.render.lighting_bake": "light",
    "menu.item.render.post_process": "rendering",
    "menu.item.hud.console_diagnostics": "terminal-outline",
    "menu.item.hud.runtime_diagnostics": "diagnostics",
    "menu.item.hud.telemetry_dashboard": "profiler",
    "menu.item.hud.performance": "frame-time",
    "menu.item.hud.font_atlas": "font",
    "menu.item.hud.menu_flow": "branch",
    "menu.item.hud.accessibility_audit": "eye",
    "menu.item.hud.icon_library": "image",
    "menu.item.hud.ui_binding": "link",
    "menu.item.hud.ui_asset_editor": "ui",
}


def menu_icons_from_document(path: Path):
    document = tomllib.loads(path.read_text(encoding="utf-8-sig"))
    icons = set()
    for node in document.get("nodes", {}).values():
        items = node.get("props", {}).get("menu_items")
        if not isinstance(items, list):
            continue
        for raw in items:
            if not isinstance(raw, str):
                continue
            match = re.search(r"(?:^|,)icon=([^,|]+)", raw)
            if match:
                icons.add(match.group(1).strip())
    return icons


class EditorPopupIconAssetContractTests(unittest.TestCase):
    def test_product_popup_icon_flags_forward_to_the_vector_asset_loader(self):
        icons = menu_icons_from_document(WORKBENCH_WINDOW)
        for path in WORKBENCH_COMPONENTS.rglob("*.zui"):
            icons.update(menu_icons_from_document(path))
        provider = CONTEXT_MENU_PROVIDER.read_text(encoding="utf-8")
        icons.update(re.findall(r"icon=([a-z0-9-]+)", provider))

        self.assertTrue(
            {"copy", "edit", "folder", "grid", "pin", "play", "rotate-ccw", "search", "target"}
            <= icons
        )

        selection = ADORNMENT_SELECTION.read_text(encoding="utf-8")
        dispatch = ADORNMENT_DISPATCH.read_text(encoding="utf-8")
        self.assertRegex(
            selection,
            r"enum\s+PopupRowAdornmentKind<'a>[\s\S]*?Icon\(&'a str\)",
        )
        self.assertIn("Some(PopupRowAdornmentKind::Icon(icon))", selection)
        self.assertIn("PopupRowAdornmentKind::Icon(icon_name) => icon_name", dispatch)

    def test_product_popup_semantic_icons_resolve_to_packaged_svg_assets(self):
        aliases = ICON_ALIASES.read_text(encoding="utf-8")
        expected = {
            "copy": "zircon_editor_shell/controls/copy.svg",
            "pin": "editor_pages/workbench/tabs/pin-tab.svg",
            "rotate-ccw": "zircon_editor_shell/toolbar/undo.svg",
        }

        for semantic_name, relative_path in expected.items():
            self.assertIn(f'"{semantic_name}"', aliases)
            self.assertIn(f'Some("{relative_path}")', aliases)
            self.assertTrue(
                (REPO_ROOT / "zircon_editor/assets/icons" / relative_path).is_file(),
                relative_path,
            )

    def test_extension_tool_menus_use_distinct_semantic_vector_icons(self):
        document = tomllib.loads(WORKBENCH_WINDOW.read_text(encoding="utf-8-sig"))
        actual = {}
        for node in document["nodes"].values():
            for raw in node.get("props", {}).get("menu_items", []):
                if not isinstance(raw, str) or raw == "---":
                    continue
                action = re.search(r"(?:^|[|,])action=([^,|]+)", raw)
                icon = re.search(r"(?:^|[|,])icon=([^,|]+)", raw)
                if action and action.group(1) in EXTENSION_MENU_ICONS:
                    self.assertIsNotNone(icon, action.group(1))
                    actual[action.group(1)] = icon.group(1)

        self.assertEqual(EXTENSION_MENU_ICONS, actual)
        self.assertNotIn("grid", actual.values())
        self.assertGreaterEqual(len(set(actual.values())), 40)

    def test_leaf_commands_do_not_advertise_unimplemented_submenus(self):
        document = tomllib.loads(WORKBENCH_WINDOW.read_text(encoding="utf-8-sig"))
        rows = {}
        for node in document["nodes"].values():
            for raw in node.get("props", {}).get("menu_items", []):
                if not isinstance(raw, str) or raw == "---":
                    continue
                parts = raw.split("|", 2)
                flags = parts[1].split(",") if len(parts) > 1 else []
                action = next(
                    (
                        flag.split("=", 1)[1]
                        for flag in flags
                        if flag.startswith("action=")
                    ),
                    None,
                )
                if action:
                    rows[action] = set(flags)

        self.assertIn("icon=search", rows["menu.item.command_palette"])
        self.assertNotIn("submenu", rows["menu.item.command_palette"])
        self.assertIn("icon=route", rows["menu.item.network_preview"])
        self.assertNotIn("submenu", rows["menu.item.network_preview"])

        for path in [
            WORKBENCH_COMPONENTS / "primitives/feedback/workbench_popup_menu.zui",
            WORKBENCH_COMPONENTS / "shell/workbench_component_drawer.zui",
        ]:
            items = menu_icons_from_document(path)
            self.assertIn("more", items, path.name)
            self.assertNotIn(
                "More Tools|action=menu.item.more_tools,submenu",
                path.read_text(encoding="utf-8-sig"),
            )

        source = ASSET_CREATION_MENU.read_text(encoding="utf-8")
        self.assertIn('"action=menu.item.command_palette,icon=search"', source)
        self.assertNotIn('"action=menu.item.command_palette,submenu"', source)


if __name__ == "__main__":
    unittest.main()
