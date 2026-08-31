import tomllib
import unittest
from pathlib import Path

from tools.tests.test_editor_zui_base_radius_hierarchy_contract import (
    reachable_workbench_documents,
)


REPO_ROOT = Path(__file__).resolve().parents[2]
ASSET_ROOT = REPO_ROOT / "zircon_editor/assets"
TOP_TOOLBAR = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/shell/"
    "workbench_top_toolbar.zui"
)
HOST_WORKBENCH_SHELL = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/host/workbench_shell.zui"
)
HIERARCHY_SURFACE = REPO_ROOT / "zircon_editor/assets/ui/editor/hierarchy.zui"
VIEWPORT_PANEL = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/shell/"
    "workbench_viewport_panel.zui"
)
EDITOR_TOKENS = REPO_ROOT / "zircon_editor/assets/ui/editor/theme/editor_tokens.zui"
ACTIVITY_RAIL = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/shell/"
    "workbench_activity_rail.zui"
)
SCENE_TREE_PANEL = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/shell/"
    "workbench_scene_tree_panel.zui"
)
INSPECTOR_PANEL = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/shell/"
    "workbench_inspector_panel.zui"
)
TRANSPORT_CONTROLS = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/composites/animation/"
    "workbench_transport_controls.zui"
)
WORKBENCH_BRIDGE = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/componentized_window.rs"
)
WORKBENCH_TEMPLATE_BINDINGS = REPO_ROOT / (
    "zircon_editor/src/ui/template_runtime/builtin/"
    "workbench_window_template_bindings.rs"
)
BUILTIN_TEMPLATE_BINDINGS = REPO_ROOT / (
    "zircon_editor/src/ui/template_runtime/builtin/template_bindings.rs"
)
WORKBENCH_TOOLBAR_BREAKPOINT_TESTS = REPO_ROOT / (
    "zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/"
    "workbench_toolbar_breakpoints/mod.rs"
)
WORKBENCH_MAIN_MENU_BINDING_TESTS = REPO_ROOT / (
    "zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/"
    "workbench_window_menus/main_menu_bindings.rs"
)
WORKBENCH_MODULE_NAVIGATION_TESTS = REPO_ROOT / (
    "zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/"
    "workbench_module_navigation.rs"
)
BLEND_SPACE_TRANSPORT = WORKBENCH_BRIDGE.parent / "blend_space_transport.rs"
REFERENCE_MENU_ACTIONS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/reference_menu_actions.rs"
)
WORKBENCH_WINDOW = REPO_ROOT / "zircon_editor/assets/ui/editor/windows/workbench_window.zui"
COMPONENT_DRAWER = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/shell/"
    "workbench_component_drawer.zui"
)
WINDOW_MENU_STATE = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/window_menu_state.rs"
)
WORKBENCH_REFRESH_LAYOUT = WORKBENCH_BRIDGE.parent / (
    "componentized_window/refresh_layout.rs"
)
MODULE_ASSET_ROOT = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/modules"
)
CORE_MODULE_ASSET_ROOT = MODULE_ASSET_ROOT / "core"
EXTENSION_MODULE_ASSET_ROOT = MODULE_ASSET_ROOT / "extensions"
OVERLAY_PRIMITIVE_ROOT = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/primitives/feedback"
)
WORKBENCH_CONTEXT_MENU = OVERLAY_PRIMITIVE_ROOT / "workbench_context_menu.zui"
WORKBENCH_COMMAND_PALETTE = OVERLAY_PRIMITIVE_ROOT / "workbench_command_palette.zui"
WORKBENCH_DIALOG = OVERLAY_PRIMITIVE_ROOT / "workbench_dialog.zui"
WORKBENCH_CONFIRM_DIALOG = OVERLAY_PRIMITIVE_ROOT / "workbench_confirm_dialog.zui"
WORKBENCH_DROPDOWN_POPUP = OVERLAY_PRIMITIVE_ROOT / "workbench_dropdown_popup.zui"
WORKBENCH_COMMAND_PALETTE_BRIDGE = WORKBENCH_BRIDGE.parent / "command_palette.rs"
RUNTIME_KEYBOARD_REDUCER = REPO_ROOT / (
    "zircon_runtime/src/ui/component/state_reducer/keyboard.rs"
)
MODULE_NAVIGATION = WORKBENCH_BRIDGE.parent / "module_navigation.rs"
EXTENSION_MODULE_NAVIGATION = (
    WORKBENCH_BRIDGE.parent / "extension_module_navigation.rs"
)

LIVE_STATE_KEYS = {
    "selected",
    "checked",
    "pressed",
    "value",
    "button_interaction_state",
}
MODULE_TAB_NODES = {
    "module_scene",
    "module_effect",
    "module_ability",
    "module_tags",
    "module_perception",
    "module_material",
    "module_behavior",
    "module_render",
    "module_assets",
    "module_vfx",
    "module_hud",
}
OVERLAY_PRIMITIVES = (
    "workbench_command_palette.zui",
    "workbench_confirm_dialog.zui",
    "workbench_context_menu.zui",
    "workbench_dialog.zui",
    "workbench_drag_overlay.zui",
    "workbench_dropdown_popup.zui",
    "workbench_notification_center.zui",
    "workbench_popup_menu.zui",
)
TOP_TOOLBAR_COMMAND_ACTIONS = {
    "toolbar_assets": ("Workbench/OpenAssetBrowserFromToolbar", "view.asset_browser.open"),
    "toolbar_open": ("MenuAction/OpenProject", "file.project.open"),
    "toolbar_save": ("MenuAction/SaveProject", "file.project.save"),
    "run_play": ("Run/Play", "runtime.play_mode.enter"),
    "run_stop": ("Run/Stop", "runtime.play_mode.exit"),
}
HOST_WORKBENCH_COMMAND_ACTIONS = {
    "open_project": ("WorkbenchMenuBar/OpenProject", "file.project.open"),
    "save_project": ("WorkbenchMenuBar/SaveProject", "file.project.save"),
}
def load_document(path: Path) -> dict:
    with path.open("rb") as source:
        return tomllib.load(source)


class EditorZuiLiveStateAuthorityContractTests(unittest.TestCase):
    def test_componentized_scene_search_has_unique_identity_and_shared_actions(self):
        shell_search = load_document(SCENE_TREE_PANEL)["nodes"]["scene_search_field"]
        host_search = load_document(HIERARCHY_SURFACE)["nodes"]["search_field"]
        self.assertEqual(
            [
                {
                    "id": "Workbench/SceneSearchEditFromShell",
                    "event": "Change",
                    "component_event": "ValueChanged",
                    "route": "workbench.hierarchy.shell_search.edit",
                },
                {
                    "id": "Workbench/SceneSearchCommitFromShell",
                    "event": "Submit",
                    "component_event": "Commit",
                    "route": "workbench.hierarchy.shell_search.commit",
                },
            ],
            shell_search["events"],
        )
        self.assertEqual(
            {"Workbench/SceneSearchEdit", "Workbench/SceneSearchCommit"},
            {event["id"] for event in host_search["events"]},
        )
        self.assertTrue(
            {event["id"] for event in shell_search["events"]}.isdisjoint(
                event["id"] for event in host_search["events"]
            )
        )
        self.assertTrue(
            {event["route"] for event in shell_search["events"]}.isdisjoint(
                event["route"] for event in host_search["events"]
            )
        )

        production = WORKBENCH_TEMPLATE_BINDINGS.read_text(encoding="utf-8").split(
            "#[cfg(test)]", maxsplit=1
        )[0]
        self.assertIn('"SceneSearchEditFromShell"', production)
        self.assertIn('"SceneSearchCommitFromShell"', production)
        self.assertEqual(2, production.count('"workbench.hierarchy.search.edit"'))
        self.assertEqual(2, production.count('"workbench.hierarchy.search.commit"'))

    def test_componentized_document_tabs_have_unique_identity_and_dynamic_payload(self):
        workbench_tabs = load_document(VIEWPORT_PANEL)["nodes"]["document_tabs"]
        host_tabs = load_document(HOST_WORKBENCH_SHELL)["nodes"]["document_tabs"]
        self.assertEqual(
            [
                {
                    "id": "Workbench/ActivateDocumentTab",
                    "event": "Change",
                    "route": "workbench.document_tabs.activate",
                },
                {
                    "id": "Workbench/CloseDocumentTab",
                    "event": "Submit",
                    "route": "workbench.document_tabs.close",
                },
            ],
            workbench_tabs["events"],
        )
        self.assertEqual(
            {"DocumentTabs/ActivateTab", "DocumentTabs/CloseTab"},
            {event["id"] for event in host_tabs["events"]},
        )
        self.assertTrue(
            {event["id"] for event in workbench_tabs["events"]}.isdisjoint(
                event["id"] for event in host_tabs["events"]
            )
        )
        self.assertTrue(
            {event["route"] for event in workbench_tabs["events"]}.isdisjoint(
                event["route"] for event in host_tabs["events"]
            )
        )

        production = BUILTIN_TEMPLATE_BINDINGS.read_text(encoding="utf-8").split(
            "#[cfg(test)]", maxsplit=1
        )[0]
        self.assertIn('"Workbench/ActivateDocumentTab"', production)
        self.assertIn('"Workbench/CloseDocumentTab"', production)
        self.assertEqual(
            4, production.count("DYNAMIC_DOCUMENT_TAB_INSTANCE_ID.to_string()")
        )

    def test_top_toolbar_registered_commands_use_canonical_action_bindings(self):
        nodes = load_document(TOP_TOOLBAR)["nodes"]

        for node_name, (binding_id, command_id) in TOP_TOOLBAR_COMMAND_ACTIONS.items():
            events = nodes[node_name]["events"]
            matching = [event for event in events if event["id"] == binding_id]
            self.assertEqual(1, len(matching), f"{node_name} must retain {binding_id}")
            self.assertNotIn("route", matching[0])
            self.assertEqual({"action": command_id}, matching[0].get("action"))

    def test_host_workbench_registered_commands_use_canonical_action_bindings(self):
        nodes = load_document(HOST_WORKBENCH_SHELL)["nodes"]

        for node_name, (binding_id, command_id) in HOST_WORKBENCH_COMMAND_ACTIONS.items():
            events = nodes[node_name]["events"]
            matching = [event for event in events if event["id"] == binding_id]
            self.assertEqual(1, len(matching), f"{node_name} must retain {binding_id}")
            self.assertNotIn("route", matching[0])
            self.assertEqual({"action": command_id}, matching[0].get("action"))

    def test_project_toolbar_commands_dispatch_through_the_command_registry(self):
        source = WORKBENCH_TEMPLATE_BINDINGS.read_text(encoding="utf-8")
        for command_id, legacy_route in (
            ("file.project.open", "workbench.project.open"),
            ("file.project.save", "workbench.project.save"),
            ("runtime.play_mode.enter", "workbench.play_mode.enter"),
        ):
            self.assertIn(
                f'EditorUiBindingPayload::editor_command("{command_id}")', source
            )
            self.assertNotIn(
                f'EditorUiBindingPayload::menu_action("{legacy_route}")', source
            )

    def test_responsive_menu_regressions_expect_the_same_command_authority(self):
        for path in (
            WORKBENCH_TOOLBAR_BREAKPOINT_TESTS,
            WORKBENCH_MAIN_MENU_BINDING_TESTS,
        ):
            source = " ".join(path.read_text(encoding="utf-8").split())
            for action_id, command_id, legacy_route in (
                ("menu.item.open_project", "file.project.open", "workbench.project.open"),
                ("menu.item.save_project", "file.project.save", "workbench.project.save"),
            ):
                self.assertIn(
                    f'("{action_id}", "{command_id}")',
                    source,
                    f"{path.name} must expect the canonical command used by the toolbar",
                )
                self.assertNotIn(
                    f'("{action_id}", "{legacy_route}")',
                    source,
                    f"{path.name} must not preserve the pre-registry menu route",
                )

        toolbar_source = " ".join(
            WORKBENCH_TOOLBAR_BREAKPOINT_TESTS.read_text(encoding="utf-8").split()
        )
        self.assertIn(
            "EditorUiBindingPayload::EditorCommand { command_id } if command_id == expected_command",
            toolbar_source,
        )

    def test_overlay_primitives_default_to_inert_without_authored_screen_position(self):
        for file_name in OVERLAY_PRIMITIVES:
            document = load_document(OVERLAY_PRIMITIVE_ROOT / file_name)
            component = next(iter(document["components"].values()))
            props = document["nodes"][component["root"]]["props"]

            for state_key in ("open", "popup_open"):
                if state_key in props:
                    self.assertFalse(
                        props[state_key],
                        f"{file_name} must require an explicit {state_key} request",
                    )
            for anchor_key in ("popup_anchor_x", "popup_anchor_y"):
                if anchor_key in props:
                    self.assertEqual(
                        0.0,
                        props[anchor_key],
                        f"{file_name} must receive {anchor_key} from its trigger owner",
                    )

        drag_props = load_document(
            OVERLAY_PRIMITIVE_ROOT / "workbench_drag_overlay.zui"
        )["nodes"]["root"]["props"]
        for state_key in ("open", "dragging", "drop_hovered", "active_drag_target"):
            self.assertFalse(
                drag_props[state_key],
                f"drag overlay must require an explicit {state_key} projection",
            )

    def test_popup_rows_start_without_authored_pointer_or_keyboard_focus(self):
        dropdown_props = load_document(WORKBENCH_DROPDOWN_POPUP)["nodes"]["root"][
            "props"
        ]
        context_props = load_document(WORKBENCH_CONTEXT_MENU)["nodes"]["root"][
            "props"
        ]
        window_context_props = load_document(WORKBENCH_WINDOW)["nodes"][
            "context_menu"
        ]["props"]
        window_notification_props = load_document(WORKBENCH_WINDOW)["nodes"][
            "notification_center"
        ]["props"]
        primitive_notification_props = load_document(
            OVERLAY_PRIMITIVE_ROOT / "workbench_notification_center.zui"
        )["nodes"]["root"]["props"]

        self.assertEqual([], dropdown_props["focused_options"])
        self.assertEqual([], dropdown_props["hovered_options"])
        self.assertEqual([], dropdown_props["pressed_options"])
        for props in (
            dropdown_props,
            context_props,
            window_context_props,
            window_notification_props,
            primitive_notification_props,
        ):
            self.assertEqual(-1, props["focused_index"])

        for props in (dropdown_props, context_props):
            self.assertEqual("", props["hovered_option_id"])
            self.assertEqual("", props["submenu_pending_option_id"])
            self.assertEqual("", props["submenu_open_option_id"])

    def test_workbench_window_authors_only_inert_overlay_boolean_state(self):
        nodes = load_document(WORKBENCH_WINDOW)["nodes"]
        overlay_state_keys = {
            "open",
            "popup_open",
            "focused",
            "selected",
            "hovered",
            "pressed",
        }
        expected_state_owners = {
            "toolbar_main_menu",
            "toolbar_run_mode_menu",
            "toolbar_layout_menu",
            "toolbar_module_overflow_menu",
            "assets_world_tools_menu",
            "assets_gameplay_tools_menu",
            "assets_production_tools_menu",
            "ability_animation_tools_menu",
            "render_tools_menu",
            "hud_tools_menu",
            "command_palette",
            "settings_window",
            "notification_center",
            "toast_overlay",
            "context_menu",
            "icon_button_tooltip",
        }
        actual_state_owners = {
            node_name
            for node_name, node in nodes.items()
            if overlay_state_keys.intersection(node.get("props", {}))
        }

        self.assertEqual(expected_state_owners, actual_state_owners)
        for node_name in sorted(expected_state_owners):
            props = nodes[node_name]["props"]
            for state_key in sorted(overlay_state_keys.intersection(props)):
                self.assertIs(
                    False,
                    props[state_key],
                    f"{node_name}.{state_key} must start inert for Rust projection",
                )

    def test_popup_keyboard_navigation_enters_before_clamping_a_missing_focus(self):
        source = RUNTIME_KEYBOARD_REDUCER.read_text(encoding="utf-8")
        function = source.split("fn next_enabled_index(", 1)[1].split(
            "\nfn option_id_list(", 1
        )[0]

        missing_focus = "if !(0..=max_index).contains(&current)"
        self.assertIn(missing_focus, function)
        self.assertNotIn("let current = current.clamp(0, max_index);", function)
        self.assertIn("UiComponentKeyboardAction::Next", function)
        self.assertIn("UiComponentKeyboardAction::Previous", function)

    def test_module_commands_are_momentary_and_do_not_author_live_state(self):
        command_nodes = []
        for path in sorted(MODULE_ASSET_ROOT.rglob("*.zui")):
            for node_name, node in load_document(path).get("nodes", {}).items():
                if "workbench-module-command" not in node.get("classes", []):
                    continue
                command_nodes.append((path, node_name, node))
                authored = LIVE_STATE_KEYS.intersection(node.get("props", {}))
                self.assertFalse(
                    authored,
                    f"{path}:{node_name} authors momentary command state: "
                    f"{sorted(authored)}",
                )

        self.assertGreaterEqual(len(command_nodes), 100)

        action_source = REFERENCE_MENU_ACTIONS.read_text(encoding="utf-8")
        self.assertIn(
            "workbench_module_panel_command_control_id(action_id).is_some()",
            action_source,
        )
        self.assertIn(
            "workbench_extension_panel_command_control_id(action_id).is_some()",
            action_source,
        )
        self.assertNotIn("MODULE_PANEL_COMMAND_CONTROLS", action_source)
        self.assertNotIn("workbench_extension_panel_command_group", action_source)

        self.assertNotIn(
            "MODULE_PANEL_COMMAND_CONTROLS",
            MODULE_NAVIGATION.read_text(encoding="utf-8"),
        )
        self.assertNotIn(
            "workbench_extension_panel_command_group",
            EXTENSION_MODULE_NAVIGATION.read_text(encoding="utf-8"),
        )

    def test_live_navigation_assets_do_not_author_business_state(self):
        toolbar_nodes = load_document(TOP_TOOLBAR)["nodes"]
        activity_nodes = load_document(ACTIVITY_RAIL)["nodes"]
        scene_nodes = load_document(SCENE_TREE_PANEL)["nodes"]

        for node_name in sorted(MODULE_TAB_NODES):
            authored = LIVE_STATE_KEYS.intersection(
                toolbar_nodes[node_name].get("props", {})
            )
            self.assertFalse(
                authored,
                f"{node_name} duplicates live module state in .zui: {sorted(authored)}",
            )

        for node_name in (
            "module_compile",
            "tool_select",
            "rail_scene",
        ):
            nodes = activity_nodes if node_name == "rail_scene" else toolbar_nodes
            authored = LIVE_STATE_KEYS.intersection(nodes[node_name].get("props", {}))
            self.assertFalse(
                authored,
                f"{node_name} duplicates live control state in .zui: {sorted(authored)}",
            )

        for nodes, node_name in (
            (scene_nodes, "scene_props_item"),
        ):
            authored = LIVE_STATE_KEYS.intersection(nodes[node_name].get("props", {}))
            self.assertFalse(
                authored,
                f"{node_name} duplicates live panel state in .zui: {sorted(authored)}",
            )

    def test_component_drawer_initial_state_is_projected_by_rust(self):
        nodes = load_document(COMPONENT_DRAWER)["nodes"]

        for node_name in ("drawer_tab_components", "drawer_tab_console"):
            authored = LIVE_STATE_KEYS.intersection(
                nodes[node_name].get("props", {})
            )
            self.assertFalse(
                authored,
                f"{node_name} duplicates drawer state in .zui: {sorted(authored)}",
            )
        for node_name in ("component_body", "console_body"):
            self.assertNotIn("visibility", nodes[node_name].get("props", {}))

        source = REFERENCE_MENU_ACTIONS.read_text(encoding="utf-8")
        initialization = source.split(
            "pub(super) fn initialize_panel_live_control_state(", 1
        )[1].split("pub(super) fn apply_reference_menu_action(", 1)[0]
        self.assertIn(
            "self.select_exclusive(\n"
            "            PANEL_COMPONENT_DRAWER_TAB_CONTROLS,\n"
            '            "WorkbenchDrawerTabComponents",\n'
            "        )?;",
            initialization,
        )
        self.assertIn(
            'self.set_visible("WorkbenchComponentDrawerBody", true)?;',
            initialization,
        )
        self.assertIn(
            'self.set_visible("WorkbenchComponentDrawerConsoleBody", false)?;',
            initialization,
        )

    def test_core_module_panel_tabs_do_not_create_live_state_authority(self):
        tab_count = 0
        for path in sorted(CORE_MODULE_ASSET_ROOT.rglob("*.zui")):
            for node in load_document(path).get("nodes", {}).values():
                if node.get("component") != "WorkbenchTab":
                    continue
                tab_count += 1

        self.assertEqual(0, tab_count)

        source = REFERENCE_MENU_ACTIONS.read_text(encoding="utf-8")
        initialization = source.split(
            "pub(super) fn initialize_panel_live_control_state(", 1
        )[1].split("pub(super) fn apply_reference_menu_action(", 1)[0]
        self.assertNotIn("CORE_MODULE_DEFAULT_TAB_ACTIONS", initialization)
        self.assertNotIn("workbench_module_panel_tab", source)

    def test_extension_module_panel_tabs_do_not_create_live_state_authority(self):
        tab_count = 0
        for path in sorted(EXTENSION_MODULE_ASSET_ROOT.rglob("*.zui")):
            tabs = [
                (node_name, node)
                for node_name, node in load_document(path).get("nodes", {}).items()
                if node.get("component") == "WorkbenchTab"
            ]
            tab_count += len(tabs)

        self.assertEqual(0, tab_count)

        navigation = EXTENSION_MODULE_NAVIGATION.read_text(encoding="utf-8")
        self.assertNotIn("workbench_extension_default_panel_tabs", navigation)
        self.assertNotIn("workbench_extension_panel_tab", navigation)

        initialization = REFERENCE_MENU_ACTIONS.read_text(encoding="utf-8").split(
            "pub(super) fn initialize_panel_live_control_state(", 1
        )[1].split("pub(super) fn apply_reference_menu_action(", 1)[0]
        self.assertNotIn("workbench_extension_default_panel_tabs()", initialization)

    def test_row_selection_exclusivity_is_scoped_to_the_layout_parent(self):
        source = WORKBENCH_BRIDGE.read_text(encoding="utf-8")
        selection = source.split("pub(super) fn select_exclusive_selected(", 1)[
            1
        ].split("pub(super) fn apply_workbench_module_workspace(", 1)[0]

        self.assertIn("control_parent_id(selected_control_id)", selection)
        self.assertIn("control_parent_id(control_id)", selection)

        regression = WORKBENCH_MODULE_NAVIGATION_TESTS.read_text(encoding="utf-8")
        self.assertIn(
            "row_selection_is_exclusive_within_its_layout_parent",
            regression,
        )

    def test_workbench_bridge_projects_initial_live_control_state(self):
        source = WORKBENCH_BRIDGE.read_text(encoding="utf-8")

        self.assertIn("bridge.initialize_live_control_state()?;", source)
        for projection in (
            'self.select_exclusive(MODULE_TAB_CONTROLS, "WorkbenchModuleEffect")?;',
            'self.select_exclusive(TOOL_CONTROLS, "WorkbenchToolSelect")?;',
            'self.select_exclusive(RAIL_CONTROLS, "WorkbenchRailScene")?;',
        ):
            self.assertIn(projection, source)
        self.assertNotIn("MODULE_COMMAND_CONTROLS", source)
        self.assertIn("self.initialize_panel_live_control_state()?;", source)

        panel_source = REFERENCE_MENU_ACTIONS.read_text(encoding="utf-8")
        self.assertIn(
            "workbench_module_command_control_id(action_id).is_some()", panel_source
        )
        self.assertNotIn("MODULE_COMMAND_CONTROLS", panel_source)
        for projection in (
            'self.set_selected("WorkbenchScenePropsItem", true)?;',
        ):
            self.assertIn(projection, panel_source)

    def test_animation_transport_initial_state_is_projected_by_rust(self):
        nodes = load_document(TRANSPORT_CONTROLS)["nodes"]
        for node_name in (
            "record",
            "play",
            "pause",
            "previous",
            "next",
            "loop",
        ):
            authored = LIVE_STATE_KEYS.intersection(
                nodes[node_name].get("props", {})
            )
            self.assertFalse(
                authored,
                f"{node_name} duplicates animation transport state in .zui: "
                f"{sorted(authored)}",
            )

        bridge_source = WORKBENCH_BRIDGE.read_text(encoding="utf-8")
        self.assertIn(
            "self.initialize_blend_space_transport_state()?;",
            bridge_source,
        )

        transport_source = BLEND_SPACE_TRANSPORT.read_text(encoding="utf-8")
        self.assertIn("fn initialize_blend_space_transport_state", transport_source)
        for projection in (
            "self.set_control_active(RECORD_CONTROL, false)?;",
            "self.set_control_active(PLAY_CONTROL, true)?;",
            "self.set_control_active(PAUSE_CONTROL, false)?;",
            "self.set_control_active(LOOP_CONTROL, true)?;",
        ):
            self.assertIn(projection, transport_source)

    def test_toolbar_menu_assets_do_not_author_absolute_open_positions(self):
        nodes = load_document(WORKBENCH_WINDOW)["nodes"]

        expected_contracts = {
            "toolbar_main_menu": (
                "WorkbenchToolbarMenu",
                "bottom-start",
                "$editor.chrome.workbench_toolbar.popup.command_offset_y",
            ),
            "toolbar_run_mode_menu": (
                "WorkbenchRunMode",
                "bottom-end",
                "$editor.chrome.workbench_toolbar.popup.command_offset_y",
            ),
            "toolbar_layout_menu": (
                "WorkbenchLayoutGrid",
                "bottom-end",
                "$editor.chrome.workbench_toolbar.popup.command_offset_y",
            ),
            "toolbar_module_overflow_menu": (
                "WorkbenchModuleMore",
                "bottom-start",
                "$editor.chrome.workbench_toolbar.popup.module_offset_y",
            ),
        }
        for node_name, (trigger_id, placement, offset_token) in expected_contracts.items():
            node = nodes[node_name]
            self.assertEqual(
                {"kind": "control", "control_id": trigger_id},
                node["widget"]["popup_anchor"],
            )
            self.assertEqual("popup_open", node["widget"]["open_property"])
            self.assertEqual(placement, node["props"]["placement"])
            self.assertEqual(offset_token, node["props"]["popup_offset_y"])
            self.assertNotIn("popup_anchor_x", node["props"])
            self.assertNotIn("popup_anchor_y", node["props"])
            self.assertEqual(0.0, node["layout"]["position"]["x"])
            self.assertEqual(0.0, node["layout"]["position"]["y"])

        source = WINDOW_MENU_STATE.read_text(encoding="utf-8")
        self.assertIn("set_fixed_control_extent", source)
        for retired_geometry_owner in (
            "apply_toolbar_window_menu_anchor",
            "popup_anchor_metrics",
            "popup_anchor_x",
            "popup_anchor_y",
            "node_position_for_absolute_frame",
            "mark_layout_dirty",
        ):
            self.assertNotIn(retired_geometry_owner, source)

        layout_source = WORKBENCH_REFRESH_LAYOUT.read_text(encoding="utf-8")
        self.assertIn("fn set_fixed_control_extent", layout_source)
        self.assertIn(".mark_layout_dirty(node_id)?;", layout_source)

    def test_component_drawer_popup_uses_its_menu_title_frame_as_anchor(self):
        popup = load_document(COMPONENT_DRAWER)["nodes"]["popup_menu"]

        self.assertEqual(
            {"kind": "control", "control_id": "WorkbenchMenuTitle"},
            popup["widget"]["popup_anchor"],
        )
        self.assertEqual("popup_open", popup["widget"]["open_property"])
        self.assertEqual("bottom-start", popup["props"]["placement"])
        self.assertEqual(
            "$editor.density.gap.small", popup["props"]["popup_offset_y"]
        )
        self.assertNotIn("popup_anchor_x", popup["props"])
        self.assertNotIn("popup_anchor_y", popup["props"])

    def test_product_reachable_graph_freezes_remaining_absolute_popup_anchor_owners(self):
        violations = []
        observed_owners = set()
        documents = reachable_workbench_documents()

        for path, document in documents.items():
            for node_name, node in document.get("nodes", {}).items():
                props = node.get("props", {})
                if any(
                    key in props
                    for key in (
                        "popup_anchor_x",
                        "popup_anchor_y",
                        "popup_anchor_width",
                        "popup_anchor_height",
                    )
                ):
                    observed_owners.add(
                        (path.relative_to(ASSET_ROOT).as_posix(), node_name)
                    )
                for key in ("popup_anchor_x", "popup_anchor_y"):
                    value = props.get(key)
                    if isinstance(value, (int, float)) and value != 0:
                        violations.append(f"{path}:{node_name}.{key}={value}")

        self.assertGreaterEqual(len(documents), 120)
        self.assertEqual(
            set(),
            observed_owners,
        )
        self.assertEqual([], violations)

    def test_command_palette_uses_the_arranged_surface_as_its_anchor(self):
        node = load_document(WORKBENCH_COMMAND_PALETTE)["nodes"]["root"]

        self.assertEqual(
            {
                "behavior": "popup",
                "popup_anchor": {"kind": "surface"},
                "open_property": "popup_open",
            },
            node.get("widget"),
        )
        for property_name in (
            "popup_anchor_x",
            "popup_anchor_y",
            "popup_anchor_width",
            "popup_anchor_height",
        ):
            self.assertNotIn(property_name, node["props"])

        source = WORKBENCH_COMMAND_PALETTE_BRIDGE.read_text(encoding="utf-8")
        for retired_geometry_owner in (
            "refresh_command_palette_popup_anchor",
            "command_palette_anchor_frame",
            "POPUP_ANCHOR_X",
            "POPUP_ANCHOR_Y",
            "POPUP_ANCHOR_WIDTH",
        ):
            self.assertNotIn(retired_geometry_owner, source)

    def test_modal_dialog_primitives_use_the_arranged_surface_as_their_anchor(self):
        for asset in (WORKBENCH_DIALOG, WORKBENCH_CONFIRM_DIALOG):
            node = load_document(asset)["nodes"]["root"]
            self.assertEqual(
                {
                    "behavior": "popup",
                    "popup_anchor": {"kind": "surface"},
                    "open_property": "popup_open",
                },
                node.get("widget"),
                asset,
            )
            self.assertEqual("center", node["props"]["placement"], asset)
            for property_name in (
                "popup_anchor_x",
                "popup_anchor_y",
                "popup_anchor_width",
                "popup_anchor_height",
            ):
                self.assertNotIn(property_name, node["props"], asset)

    def test_context_menu_uses_a_typed_transient_pointer_anchor(self):
        nodes = (
            load_document(WORKBENCH_WINDOW)["nodes"]["context_menu"],
            load_document(WORKBENCH_CONTEXT_MENU)["nodes"]["root"],
        )

        for node in nodes:
            self.assertEqual(
                {
                    "behavior": "popup",
                    "popup_anchor": {
                        "kind": "pointer",
                        "owner_property": "context_target",
                    },
                    "open_property": "popup_open",
                },
                node.get("widget"),
            )
            for property_name in (
                "popup_anchor_x",
                "popup_anchor_y",
                "popup_anchor_width",
                "popup_anchor_height",
            ):
                self.assertNotIn(property_name, node["props"])

        source = (
            REPO_ROOT
            / "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
            "workbench/context_menu.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("set_popup_pointer_anchor", source)
        self.assertNotIn("POPUP_ANCHOR_X", source)
        self.assertNotIn("POPUP_ANCHOR_Y", source)

    def test_icon_tooltip_leaves_geometry_to_its_dynamic_control_anchor(self):
        node = load_document(WORKBENCH_WINDOW)["nodes"]["icon_button_tooltip"]

        self.assertEqual(
            {"behavior": "popup", "open_property": "popup_open"},
            node.get("widget"),
        )
        for property_name in (
            "popup_anchor_x",
            "popup_anchor_y",
            "popup_anchor_width",
            "popup_anchor_height",
        ):
            self.assertNotIn(property_name, node["props"])

        source = (
            REPO_ROOT
            / "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
            "workbench/icon_tooltip.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("set_popup_control_anchor", source)
        self.assertNotIn("POPUP_ANCHOR_X", source)
        self.assertNotIn("POPUP_ANCHOR_Y", source)
        self.assertNotIn("POPUP_ANCHOR_WIDTH", source)
        self.assertNotIn("POPUP_ANCHOR_HEIGHT", source)

    def test_notification_center_authors_no_competing_popup_geometry(self):
        node = load_document(WORKBENCH_WINDOW)["nodes"]["notification_center"]
        props = node["props"]
        primitive_props = load_document(
            OVERLAY_PRIMITIVE_ROOT / "workbench_notification_center.zui"
        )["nodes"]["root"]["props"]

        self.assertEqual(
            {
                "behavior": "popup",
                "popup_anchor": {
                    "kind": "control",
                    "control_id": "WorkbenchWindowTopToolbarRegion",
                },
                "open_property": "popup_open",
            },
            node.get("widget"),
        )
        self.assertEqual(
            {"x": 0.0, "y": 0.0},
            node["layout"]["position"],
        )
        for property_name in (
            "popup_anchor_x",
            "popup_anchor_y",
            "popup_anchor_width",
            "popup_anchor_height",
        ):
            self.assertNotIn(property_name, props)
            self.assertNotIn(property_name, primitive_props)
        self.assertEqual("$editor.density.gap.small", props["popup_offset_y"])
        self.assertEqual(
            "$editor.density.gap.small", primitive_props["popup_offset_y"]
        )

    def test_live_workbench_shell_spacing_uses_shared_density_tokens(self):
        numeric_spacing = []

        for path in (COMPONENT_DRAWER, TOP_TOOLBAR):
            for node_name, node in load_document(path)["nodes"].items():
                gap = node.get("layout", {}).get("container", {}).get("gap")
                if isinstance(gap, (int, float)) and gap != 0:
                    numeric_spacing.append(
                        f"{path.name}:{node_name}.layout.container.gap={gap}"
                    )

                layout_gap = node.get("props", {}).get("layout_gap")
                if isinstance(layout_gap, (int, float)) and layout_gap != 0:
                    numeric_spacing.append(
                        f"{path.name}:{node_name}.props.layout_gap={layout_gap}"
                    )

                for child in node.get("children", []):
                    padding = (
                        child.get("slot", {}).get("layout", {}).get("padding", {})
                    )
                    for edge, value in padding.items():
                        if isinstance(value, (int, float)) and value != 0:
                            numeric_spacing.append(
                                f"{path.name}:{node_name}->{child['node']}."
                                f"slot.padding.{edge}={value}"
                            )

        self.assertEqual([], numeric_spacing)

    def test_toolbar_popup_offsets_follow_the_shared_two_row_chrome_metrics(self):
        document = load_document(EDITOR_TOKENS)
        chrome = document["chrome"]
        controls = document["controls"]
        density = document["density"]

        toolbar_height = chrome["workbench_toolbar_height"]
        command_row_height = chrome["workbench_toolbar_command_row_height"]
        control_height = controls["compact_height"]
        module_row_height = controls["dense_height"]
        row_gap = density["gap_small"]
        placement_gap = density["gap_small"]
        command_trigger_bottom = (command_row_height + control_height) * 0.5
        module_trigger_bottom = (
            command_row_height
            + row_gap
            + (module_row_height + control_height) * 0.5
        )

        self.assertEqual(
            toolbar_height - command_trigger_bottom - placement_gap,
            chrome["workbench_toolbar_popup_command_offset_y"],
        )
        self.assertEqual(
            toolbar_height - module_trigger_bottom - placement_gap,
            chrome["workbench_toolbar_popup_module_offset_y"],
        )

    def test_workbench_toast_action_uses_a_semantic_color_token(self):
        toast_props = load_document(WORKBENCH_WINDOW)["nodes"]["toast_overlay"]["props"]

        self.assertEqual("$editor.semantic.info", toast_props["action_color"])

    def test_workbench_toast_uses_the_runtime_snackbar_identity(self):
        document = load_document(OVERLAY_PRIMITIVE_ROOT / "workbench_toast.zui")
        component = next(iter(document["components"].values()))
        root = document["nodes"][component["root"]]

        self.assertEqual("Snackbar", root["component"])
        self.assertEqual("WorkbenchToastRoot", root["control_id"])

    def test_run_and_layout_menu_indicators_are_initialized_by_rust(self):
        nodes = load_document(WORKBENCH_WINDOW)["nodes"]
        for node_name in ("toolbar_run_mode_menu", "toolbar_layout_menu"):
            props = nodes[node_name]["props"]
            self.assertEqual("", props["value"])
            self.assertEqual("", props["value_text"])
            self.assertFalse(
                any("checked" in item.split("|", 1)[-1].split(",") for item in props["menu_items"]),
                f"{node_name} must not author a checked business-state marker",
            )

        bridge_source = WORKBENCH_BRIDGE.read_text(encoding="utf-8")
        self.assertIn("self.initialize_run_mode_menu_indicator()?;", bridge_source)
        self.assertIn("self.initialize_layout_menu_indicator()?;", bridge_source)

        run_mode_source = (
            WORKBENCH_BRIDGE.parent / "run_mode_menu.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("PlayKind::Play", run_mode_source)
        self.assertIn("sync_run_mode_menu_for_trigger", run_mode_source)

        layout_source = (WORKBENCH_BRIDGE.parent / "layout_menu.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("menu_item_with_checked_state", layout_source)
        self.assertIn('"menu.item.default_layout"', layout_source)


if __name__ == "__main__":
    unittest.main()
