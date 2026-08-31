import re
import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
RENDER_WORKSPACE = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/modules/core/rendering/"
    "workbench_render_workspace.zui"
)
WORKBENCH_WINDOW = REPO_ROOT / "zircon_editor/assets/ui/editor/windows/workbench_window.zui"
RENDER_EDITOR_MENU = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/render_editor_menu.rs"
)
WINDOW_MENU_STATE = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/window_menu_state.rs"
)
CONTROL_DISPATCH = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/workbench/control.rs"
)
NAVIGATION_SPEC = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/extension_module_navigation/specs/render_asset_vfx.rs"
)
MODULE_BINDINGS = REPO_ROOT / (
    "zircon_editor/src/ui/template_runtime/builtin/"
    "workbench_module_template_bindings.rs"
)
EXTENSION_BINDINGS = REPO_ROOT / (
    "zircon_editor/src/ui/template_runtime/builtin/"
    "workbench_extension_module_template_bindings/render_asset_vfx.rs"
)


TOOLS = [
    ("Shader Editor", "shader_editor", "shader_editor"),
    ("Lighting Bake", "lighting_bake", "lighting_bake"),
    ("Post Process", "post_process", "post_process"),
]

OLD_BUTTON_PATTERN = re.compile(
    r"WorkbenchRender(?:ShaderEditor|LightingBake|PostProcess)Button"
)


def load_zui(path):
    with path.open("rb") as source:
        return tomllib.load(source)


def menu_item_identity(raw):
    label, flags = raw.split("|", 1)
    action = next(flag[7:] for flag in flags.split(",") if flag.startswith("action="))
    return label, action


class EditorZuiRenderEditorMenuContractTests(unittest.TestCase):
    def test_render_details_use_one_render_tools_trigger(self):
        document = load_zui(RENDER_WORKSPACE)
        nodes = document["nodes"]
        self.assertEqual(
            [
                "render_pipeline_property_row",
                "render_platform_property_row",
                "render_frame_property_row",
                "render_tools_button",
            ],
            [
                child["node"]
                for child in nodes["render_right_content"]["children"]
            ],
        )
        trigger = nodes["render_tools_button"]
        self.assertEqual("WorkbenchRenderTools", trigger["control_id"])
        self.assertEqual("Render Tools", trigger["props"]["text"])
        self.assertEqual(
            "workbench.module.render.tools.open",
            trigger["events"][0]["route"],
        )
        self.assertIsNone(OLD_BUTTON_PATTERN.search(RENDER_WORKSPACE.read_text("utf-8")))

    def test_window_overlay_owns_the_anchored_render_tools_menu(self):
        document = load_zui(WORKBENCH_WINDOW)
        nodes = document["nodes"]
        self.assertIn(
            "render_tools_menu",
            [child["node"] for child in nodes["root"]["children"]],
        )
        menu = nodes["render_tools_menu"]
        self.assertEqual("WorkbenchPopupMenu", menu["component"])
        self.assertEqual("WorkbenchRenderToolsMenu", menu["control_id"])
        self.assertEqual(
            {
                "popup_anchor": {
                    "kind": "control",
                    "control_id": "WorkbenchRenderTools",
                },
                "open_property": "popup_open",
            },
            menu["widget"],
        )
        self.assertEqual("collapsed", menu["props"]["visibility"])
        self.assertFalse(menu["props"]["popup_open"])
        self.assertEqual(
            [
                (label, f"menu.item.render.{menu_action}")
                for label, menu_action, _ in TOOLS
            ],
            [menu_item_identity(raw) for raw in menu["props"]["menu_items"]],
        )

    def test_rust_menu_authority_maps_every_item_to_existing_extension_action(self):
        source = RENDER_EDITOR_MENU.read_text("utf-8")
        for _, menu_action, extension_action in TOOLS:
            self.assertIn(f'menu_action_id: "menu.item.render.{menu_action}"', source)
            self.assertIn(
                f'extension_action_id: "workbench.extension.{extension_action}.open"',
                source,
            )
        self.assertIn("self.apply_reference_menu_action(", source)
        self.assertIn("EditorUiBindingPayload::menu_action(command.extension_action_id)", source)
        self.assertIn(
            "dispatch_workbench_render_editor_menu_item_state",
            CONTROL_DISPATCH.read_text("utf-8"),
        )

    def test_menu_trigger_and_extension_navigation_have_one_reachable_identity(self):
        state = WINDOW_MENU_STATE.read_text("utf-8")
        self.assertIn('trigger_control_id: "WorkbenchRenderTools"', state)
        self.assertIn('menu_control_id: "WorkbenchRenderToolsMenu"', state)
        self.assertIn('"workbench.module.render.tools.open"', state)

        module_bindings = MODULE_BINDINGS.read_text("utf-8")
        self.assertIn('"RenderToolsOpen"', module_bindings)
        self.assertIn('"workbench.module.render.tools.open"', module_bindings)

        navigation = NAVIGATION_SPEC.read_text("utf-8")
        self.assertIsNone(OLD_BUTTON_PATTERN.search(navigation))
        self.assertEqual(6, navigation.count('"WorkbenchRenderToolsMenu"'))

        extension_bindings = EXTENSION_BINDINGS.read_text("utf-8")
        for _, _, extension_action in TOOLS:
            self.assertNotIn(
                f'click("{extension_action.title().replace("_", "")}Open", '
                f'"workbench.extension.{extension_action}.open")',
                extension_bindings,
            )


if __name__ == "__main__":
    unittest.main()
