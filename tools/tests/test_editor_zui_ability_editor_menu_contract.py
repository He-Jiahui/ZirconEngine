import re
import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
ABILITY_WORKSPACE = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/modules/core/gameplay/"
    "workbench_ability_workspace.zui"
)
WORKBENCH_WINDOW = REPO_ROOT / "zircon_editor/assets/ui/editor/windows/workbench_window.zui"
ABILITY_EDITOR_MENU = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/ability_editor_menu.rs"
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
    "workbench/extension_module_navigation/specs/gameplay_animation.rs"
)
MODULE_BINDINGS = REPO_ROOT / (
    "zircon_editor/src/ui/template_runtime/builtin/"
    "workbench_module_template_bindings.rs"
)
EXTENSION_BINDINGS = REPO_ROOT / (
    "zircon_editor/src/ui/template_runtime/builtin/"
    "workbench_extension_module_template_bindings/gameplay_animation.rs"
)


TOOLS = [
    ("Sequencer", "sequencer", "sequencer"),
    ("Montage Editor", "montage_editor", "montage_editor"),
    ("Blend Space", "blend_space", "blend_space"),
    ("Pose Library", "pose_library", "pose_library"),
    ("Retarget", "retarget", "retarget"),
    ("Control Rig", "control_rig", "control_rig"),
    ("Motion Matching", "motion_matching", "motion_matching"),
    ("Animation Compression", "animation_compression", "animation_compression"),
]

OLD_BUTTON_PATTERN = re.compile(
    r"WorkbenchAbility(?:Sequencer|MontageEditor|BlendSpace|PoseLibrary|Retarget|"
    r"ControlRig|MotionMatching|AnimationCompression)Button"
)


def load_zui(path):
    with path.open("rb") as source:
        return tomllib.load(source)


def menu_item_identity(raw):
    label, flags = raw.split("|", 1)
    action = next(flag[7:] for flag in flags.split(",") if flag.startswith("action="))
    return label, action


class EditorZuiAbilityEditorMenuContractTests(unittest.TestCase):
    def test_ability_details_use_one_animation_tools_trigger(self):
        document = load_zui(ABILITY_WORKSPACE)
        nodes = document["nodes"]
        self.assertEqual(
            [
                "ability_name_property_row",
                "ability_net_policy_property_row",
                "ability_cooldown_property_row",
                "ability_animation_tools_button",
            ],
            [child["node"] for child in nodes["ability_right_content"]["children"]],
        )
        trigger = nodes["ability_animation_tools_button"]
        self.assertEqual("WorkbenchAbilityAnimationTools", trigger["control_id"])
        self.assertEqual("Animation Tools", trigger["props"]["text"])
        self.assertEqual(
            "workbench.module.ability.animation_tools.open",
            trigger["events"][0]["route"],
        )
        self.assertIsNone(OLD_BUTTON_PATTERN.search(ABILITY_WORKSPACE.read_text("utf-8")))

    def test_window_overlay_owns_the_anchored_animation_tools_menu(self):
        document = load_zui(WORKBENCH_WINDOW)
        nodes = document["nodes"]
        self.assertIn(
            "ability_animation_tools_menu",
            [child["node"] for child in nodes["root"]["children"]],
        )
        menu = nodes["ability_animation_tools_menu"]
        self.assertEqual("WorkbenchPopupMenu", menu["component"])
        self.assertEqual("WorkbenchAbilityAnimationToolsMenu", menu["control_id"])
        self.assertEqual(
            {
                "popup_anchor": {
                    "kind": "control",
                    "control_id": "WorkbenchAbilityAnimationTools",
                },
                "open_property": "popup_open",
            },
            menu["widget"],
        )
        self.assertEqual("collapsed", menu["props"]["visibility"])
        self.assertFalse(menu["props"]["popup_open"])
        self.assertEqual(
            [
                (label, f"menu.item.ability.{menu_action}")
                for label, menu_action, _ in TOOLS
            ],
            [menu_item_identity(raw) for raw in menu["props"]["menu_items"]],
        )

    def test_rust_menu_authority_maps_every_item_to_existing_extension_action(self):
        source = ABILITY_EDITOR_MENU.read_text("utf-8")
        for _, menu_action, extension_action in TOOLS:
            self.assertIn(f'menu_action_id: "menu.item.ability.{menu_action}"', source)
            self.assertIn(
                f'extension_action_id: "workbench.extension.{extension_action}.open"',
                source,
            )
        self.assertIn("self.apply_reference_menu_action(", source)
        self.assertIn("EditorUiBindingPayload::menu_action(command.extension_action_id)", source)
        self.assertIn(
            "dispatch_workbench_ability_editor_menu_item_state",
            CONTROL_DISPATCH.read_text("utf-8"),
        )

    def test_menu_trigger_and_extension_navigation_have_one_reachable_identity(self):
        state = WINDOW_MENU_STATE.read_text("utf-8")
        self.assertIn('trigger_control_id: "WorkbenchAbilityAnimationTools"', state)
        self.assertIn('menu_control_id: "WorkbenchAbilityAnimationToolsMenu"', state)
        self.assertIn('"workbench.module.ability.animation_tools.open"', state)

        module_bindings = MODULE_BINDINGS.read_text("utf-8")
        self.assertIn('"AbilityAnimationToolsOpen"', module_bindings)
        self.assertIn('"workbench.module.ability.animation_tools.open"', module_bindings)

        navigation = NAVIGATION_SPEC.read_text("utf-8")
        self.assertIsNone(OLD_BUTTON_PATTERN.search(navigation))
        self.assertEqual(16, navigation.count('"WorkbenchAbilityAnimationToolsMenu"'))

        extension_bindings = EXTENSION_BINDINGS.read_text("utf-8")
        for _, _, extension_action in TOOLS:
            self.assertNotIn(
                f'"workbench.extension.{extension_action}.open"', extension_bindings
            )


if __name__ == "__main__":
    unittest.main()
