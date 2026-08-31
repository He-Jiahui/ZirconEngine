import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
ASSET_ROOT = REPO_ROOT / "zircon_editor/assets/ui/editor"
ASSET_BROWSER = ASSET_ROOT / "asset_browser.zui"
ASSETS_ACTIVITY = ASSET_ROOT / "assets_activity.zui"
EDITOR_TOKENS = ASSET_ROOT / "theme/editor_tokens.zui"
HOST_ASSET_CONTROLS = ASSET_ROOT / "host/asset_surface_controls.zui"
ASSET_POINTER_EVENTS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/app/asset_content_pointer/events/click.rs"
)
ASSET_POINTER_DISPATCH = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/shared_pointer/"
    "asset_content.rs"
)
BUILTIN_TEMPLATE_BINDINGS = REPO_ROOT / (
    "zircon_editor/src/ui/template_runtime/builtin/template_bindings.rs"
)
ASSET_EVENT_EXECUTION_COMMON = REPO_ROOT / (
    "zircon_editor/src/ui/host/editor_event_execution/common.rs"
)
WORKBENCH_BUTTON_TAB_IDENTITY = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "style_selector/workbench_button/tab_like.rs"
)
POPUP_ROW_HIT_ROUTING = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/"
    "template_node/popup_rows/hit.rs"
)
KEYBOARD_OPTION_TARGET = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/native_keyboard/"
    "target/options.rs"
)
KEYBOARD_POPUP_ACTIONS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/native_keyboard/"
    "dispatch/actions.rs"
)
ASSET_ACTIVATION = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/"
    "template_activation_semantics/asset.rs"
)
ASSET_CONTROL_IDS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/asset_control_ids.rs"
)
ASSET_TEXT_INPUT_DISPATCH = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/window/text_input/edit/dispatch.rs"
)


def load_nodes(path: Path) -> dict:
    with path.open("rb") as source:
        return tomllib.load(source)["nodes"]


def single_event(nodes: dict, node_name: str) -> dict:
    events = nodes[node_name].get("events", [])
    if len(events) != 1:
        raise AssertionError(f"{node_name} must expose exactly one event: {events!r}")
    return events[0]


class EditorZuiAssetSurfaceInteractionContractTests(unittest.TestCase):
    def test_product_asset_surfaces_use_unique_trace_identities_behind_canonical_host_controls(self):
        host_nodes = load_nodes(HOST_ASSET_CONTROLS)
        activity_nodes = load_nodes(ASSETS_ACTIVITY)
        browser_nodes = load_nodes(ASSET_BROWSER)

        expected_activity = {
            "toolbar_open_browser_button": (
                "Workbench/AssetsActivityOpenBrowser",
                None,
            ),
            "toolbar_search_field": (
                "Workbench/AssetsActivitySearchEdited",
                "workbench.assets_activity.search.edit",
            ),
            "toolbar_kind_filter_dropdown": (
                "Workbench/AssetsActivityKindFilterChanged",
                "workbench.assets_activity.kind_filter.change",
            ),
            "toolbar_view_mode_list_button": (
                "Workbench/AssetsActivityViewModeList",
                "workbench.assets_activity.view_mode.list",
            ),
            "toolbar_view_mode_thumb_button": (
                "Workbench/AssetsActivityViewModeThumbnail",
                "workbench.assets_activity.view_mode.thumbnail",
            ),
            "utility_preview_button": (
                "Workbench/AssetsActivityUtilityPreview",
                "workbench.assets_activity.utility.preview",
            ),
            "utility_references_button": (
                "Workbench/AssetsActivityUtilityReferences",
                "workbench.assets_activity.utility.references",
            ),
        }
        expected_browser = {
            "toolbar_locate_button": (
                "Workbench/AssetBrowserLocateSelected",
                "workbench.asset_browser.locate_selected",
            ),
            "toolbar_search_field": (
                "Workbench/AssetBrowserSearchEdited",
                "workbench.asset_browser.search.edit",
            ),
            "toolbar_kind_filter_dropdown": (
                "Workbench/AssetBrowserKindFilterChanged",
                "workbench.asset_browser.kind_filter.change",
            ),
            "toolbar_view_mode_list_button": (
                "Workbench/AssetBrowserViewModeList",
                "workbench.asset_browser.view_mode.list",
            ),
            "toolbar_view_mode_thumb_button": (
                "Workbench/AssetBrowserViewModeThumbnail",
                "workbench.asset_browser.view_mode.thumbnail",
            ),
            "import_button": (
                "Workbench/AssetBrowserImportModel",
                "workbench.asset_browser.import_model",
            ),
            "utility_preview_button": (
                "Workbench/AssetBrowserUtilityPreview",
                "workbench.asset_browser.utility.preview",
            ),
            "utility_references_button": (
                "Workbench/AssetBrowserUtilityReferences",
                "workbench.asset_browser.utility.references",
            ),
            "utility_metadata_button": (
                "Workbench/AssetBrowserUtilityMetadata",
                "workbench.asset_browser.utility.metadata",
            ),
            "utility_plugins_button": (
                "Workbench/AssetBrowserUtilityPlugins",
                "workbench.asset_browser.utility.plugins",
            ),
        }

        product_identities = []
        product_routes = []
        for nodes, expected in (
            (activity_nodes, expected_activity),
            (browser_nodes, expected_browser),
        ):
            for node_name, (binding_id, route) in expected.items():
                event = single_event(nodes, node_name)
                self.assertEqual(binding_id, event["id"], node_name)
                self.assertEqual(route, event.get("route"), node_name)
                product_identities.append(binding_id)
                if route is not None:
                    product_routes.append(route)

        self.assertEqual(len(product_identities), len(set(product_identities)))
        self.assertEqual(len(product_routes), len(set(product_routes)))
        host_identities = {
            event["id"]
            for node in host_nodes.values()
            for event in node.get("events", [])
        }
        self.assertTrue(host_identities.issuperset({
            "AssetSurface/SearchEdited",
            "AssetSurface/SetKindFilter",
            "AssetSurface/SetViewMode",
            "AssetSurface/SetUtilityTab",
            "AssetSurface/LocateSelectedAsset",
            "AssetSurface/ImportModel",
        }))
        self.assertTrue(host_identities.isdisjoint(product_identities))

        activation_source = ASSET_ACTIVATION.read_text(encoding="utf-8")
        text_input_source = ASSET_TEXT_INPUT_DISPATCH.read_text(encoding="utf-8")
        self.assertIn("asset_dispatch_source(hit.dispatch_kind.as_str())", activation_source)
        self.assertIn("action_or_control_id(hit)", activation_source)
        self.assertIn("hit.value_text", activation_source)
        self.assertIn("asset_dispatch_source(kind).is_some()", text_input_source)
        self.assertIn("invoke_asset_control_changed", text_input_source)

    def test_asset_browser_kind_filter_dropdown_is_not_a_legacy_toolbar_chip(self):
        identity_source = WORKBENCH_BUTTON_TAB_IDENTITY.read_text(encoding="utf-8")

        self.assertNotIn(
            'control_id.starts_with("AssetBrowserKind")\n        ||',
            identity_source,
        )
        self.assertIn('control_id.ends_with("Chip")', identity_source)
        self.assertIn('control_id.ends_with("Button")', identity_source)

    def test_host_asset_kind_filter_emits_the_canonical_parser_value_domain(self):
        nodes = load_nodes(HOST_ASSET_CONTROLS)
        filter_node = nodes["kind_filter"]
        filter_event = single_event(nodes, "kind_filter")

        self.assertEqual("Change", filter_event["event"])
        self.assertEqual("AssetSurface/SetKindFilter", filter_event["id"])
        self.assertEqual("workbench.asset.set_kind_filter", filter_event["route"])
        self.assertEqual("All", filter_node["props"]["value"])
        self.assertEqual("All Types", filter_node["props"]["value_text"])
        expected_options = [
                "All|label=All Types",
                "Texture|label=Textures",
                "Material|label=Materials",
                "Scene|label=Scenes",
                "Model|label=Models",
                "Mesh|label=Meshes",
                "Shader|label=Shaders",
                "PhysicsMaterial|label=Physics Materials",
                "AnimationSkeleton|label=Animation Skeletons",
                "AnimationClip|label=Animation Clips",
                "AnimationSequence|label=Animation Sequences",
                "AnimationGraph|label=Animation Graphs",
                "AnimationStateMachine|label=Animation State Machines",
                "UiLayout|label=UI Layouts",
                "UiWidget|label=UI Widgets",
                "UiStyle|label=UI Styles",
            ]
        self.assertEqual(expected_options, filter_node["props"]["options"])
        parser_source = ASSET_EVENT_EXECUTION_COMMON.read_text(encoding="utf-8")
        for option in expected_options:
            stable_id = option.partition("|")[0]
            self.assertIn(f'"{stable_id}"', parser_source, stable_id)

    def test_asset_browser_kind_filter_uses_one_native_dropdown_authority(self):
        nodes = load_nodes(ASSET_BROWSER)
        filter_node = nodes["toolbar_kind_filter_dropdown"]
        filter_event = single_event(nodes, "toolbar_kind_filter_dropdown")
        self.assertEqual("Change", filter_event["event"])
        self.assertEqual(
            "workbench.asset_browser.kind_filter.change", filter_event["route"]
        )
        self.assertEqual("Workbench/AssetBrowserKindFilterChanged", filter_event["id"])
        self.assertNotIn("component_event", filter_event)
        self.assertNotIn("action_id", filter_node["props"])
        self.assertEqual(
            [
                "All|label=All Types",
                "Texture|label=Textures",
                "Material|label=Materials",
                "Scene|label=Scenes",
                "Model|label=Models",
                "Mesh|label=Meshes",
                "Shader|label=Shaders",
                "PhysicsMaterial|label=Physics Materials",
                "AnimationSkeleton|label=Animation Skeletons",
                "AnimationClip|label=Animation Clips",
                "AnimationSequence|label=Animation Sequences",
                "AnimationGraph|label=Animation Graphs",
                "AnimationStateMachine|label=Animation State Machines",
                "UiLayout|label=UI Layouts",
                "UiWidget|label=UI Widgets",
                "UiStyle|label=UI Styles",
            ],
            filter_node["props"]["options"],
        )
        self.assertFalse(
            any(
                node_name.startswith("toolbar_kind_")
                and node_name.endswith("_chip")
                for node_name in nodes
            )
        )

    def test_asset_dropdown_mouse_and_keyboard_options_keep_asset_dispatch(self):
        pointer_source = POPUP_ROW_HIT_ROUTING.read_text(encoding="utf-8")
        keyboard_target_source = KEYBOARD_OPTION_TARGET.read_text(encoding="utf-8")
        keyboard_action_source = KEYBOARD_POPUP_ACTIONS.read_text(encoding="utf-8")
        activation_source = ASSET_ACTIVATION.read_text(encoding="utf-8")
        control_source = ASSET_CONTROL_IDS.read_text(encoding="utf-8")

        self.assertIn("popup_row_dispatch_kind(node, kind)", pointer_source)
        self.assertIn("option_popup_dispatch_kind(node)", keyboard_target_source)
        self.assertIn("asset_dispatch_source(target.dispatch_kind.as_str())", keyboard_action_source)
        self.assertIn("invoke_asset_control_changed", keyboard_action_source)
        self.assertIn("TemplateComponentFamily::Dropdown", activation_source)
        self.assertIn('"AssetSurface/SetKindFilter"', control_source)

    def test_asset_browser_actions_keep_named_routes_after_primitive_migration(self):
        nodes = load_nodes(ASSET_BROWSER)
        expected = {
            "toolbar_locate_button": ("Click", "workbench.asset_browser.locate_selected"),
            "toolbar_search_field": ("Change", "workbench.asset_browser.search.edit"),
            "import_path_field": ("Change", "workbench.asset.mesh_import.path.set"),
            "import_button": ("Click", "workbench.asset_browser.import_model"),
            "toolbar_view_mode_list_button": (
                "Change",
                "workbench.asset_browser.view_mode.list",
            ),
            "toolbar_view_mode_thumb_button": (
                "Change",
                "workbench.asset_browser.view_mode.thumbnail",
            ),
        }
        for node_name, (event_kind, route) in expected.items():
            event = single_event(nodes, node_name)
            self.assertEqual(event_kind, event["event"], node_name)
            self.assertEqual(route, event["route"], node_name)

        expected_utility_routes = {
            "utility_preview_button": "workbench.asset_browser.utility.preview",
            "utility_references_button": "workbench.asset_browser.utility.references",
            "utility_metadata_button": "workbench.asset_browser.utility.metadata",
            "utility_plugins_button": "workbench.asset_browser.utility.plugins",
        }
        for node_name, route in expected_utility_routes.items():
            event = single_event(nodes, node_name)
            self.assertEqual("Change", event["event"], node_name)
            self.assertEqual(route, event["route"], node_name)

        with EDITOR_TOKENS.open("rb") as source:
            tokens = tomllib.load(source)
        utility_row = nodes["utility_tabs_row"]
        child_names = [child["node"] for child in utility_row["children"]]
        utility_names = list(expected_utility_routes)
        self.assertEqual(utility_names, child_names[: len(utility_names)])

        locator_name = "utility_selection_locator_text"
        self.assertEqual(
            "narrow",
            nodes[locator_name]["props"]["responsive_min_tier"],
        )
        gap = tokens["density"]["gap_small"]
        authored_min = sum(
            nodes[name]["layout"]["width"].get("min", 0.0)
            for name in child_names
        ) + gap * (len(child_names) - 1)
        ultra_names = [name for name in child_names if name != locator_name]
        ultra_min = sum(
            nodes[name]["layout"]["width"].get("min", 0.0)
            for name in ultra_names
        ) + gap * (len(ultra_names) - 1)
        ultra_available = (
            640.0 / 1.5 - tokens["chrome"]["activity_rail_width"]
        )
        self.assertGreater(authored_min, ultra_available)
        self.assertLessEqual(ultra_min, ultra_available)

    def test_assets_activity_keeps_command_and_filter_authority(self):
        nodes = load_nodes(ASSETS_ACTIVITY)
        open_event = single_event(nodes, "toolbar_open_browser_button")
        self.assertEqual("Click", open_event["event"])
        self.assertEqual(
            {"action": "view.asset_browser.open"}, open_event["action"]
        )

        expected_routes = {
            "toolbar_search_field": "workbench.assets_activity.search.edit",
            "toolbar_view_mode_list_button": "workbench.assets_activity.view_mode.list",
            "toolbar_view_mode_thumb_button": "workbench.assets_activity.view_mode.thumbnail",
            "utility_preview_button": "workbench.assets_activity.utility.preview",
            "utility_references_button": "workbench.assets_activity.utility.references",
        }
        for node_name, route in expected_routes.items():
            event = single_event(nodes, node_name)
            self.assertEqual("Change", event["event"], node_name)
            self.assertEqual(route, event["route"], node_name)

        filter_node = nodes["toolbar_kind_filter_dropdown"]
        filter_event = single_event(nodes, "toolbar_kind_filter_dropdown")
        self.assertEqual("Change", filter_event["event"])
        self.assertEqual(
            "workbench.assets_activity.kind_filter.change", filter_event["route"]
        )
        self.assertEqual("Workbench/AssetsActivityKindFilterChanged", filter_event["id"])
        self.assertNotIn("component_event", filter_event)
        self.assertNotIn("action_id", filter_node["props"])
        self.assertEqual(
            [
                "All|label=All Types",
                "Texture|label=Textures",
                "Material|label=Materials",
                "Scene|label=Scenes",
                "Model|label=Models",
                "Mesh|label=Meshes",
                "Shader|label=Shaders",
                "PhysicsMaterial|label=Physics Materials",
                "AnimationSkeleton|label=Animation Skeletons",
                "AnimationClip|label=Animation Clips",
                "AnimationSequence|label=Animation Sequences",
                "AnimationGraph|label=Animation Graphs",
                "AnimationStateMachine|label=Animation State Machines",
                "UiLayout|label=UI Layouts",
                "UiWidget|label=UI Widgets",
                "UiStyle|label=UI Styles",
            ],
            filter_node["props"]["options"],
        )
        self.assertFalse(
            any(
                node_name.startswith("toolbar_kind_")
                and node_name.endswith("_chip")
                for node_name in nodes
            )
        )

    def test_asset_rows_use_one_shared_pointer_selection_authority(self):
        nodes = load_nodes(ASSET_BROWSER)
        row_names = [
            node_name
            for node_name in nodes
            if node_name.startswith("content_asset_row_")
        ]
        self.assertGreaterEqual(len(row_names), 4)
        for node_name in row_names:
            node = nodes[node_name]
            self.assertEqual("WorkbenchTableRow", node["component"])
            self.assertNotIn("events", node)
            self.assertNotIn("selected", node.get("props", {}))
            self.assertNotIn("focused", node.get("props", {}))

        event_source = ASSET_POINTER_EVENTS.read_text(encoding="utf-8")
        dispatch_source = ASSET_POINTER_DISPATCH.read_text(encoding="utf-8")
        binding_source = BUILTIN_TEMPLATE_BINDINGS.read_text(encoding="utf-8")
        for surface in ("activity", "browser"):
            self.assertIn(f'"{surface}" =>', event_source)
        self.assertGreaterEqual(
            event_source.count("dispatch_shared_asset_content_pointer_click("), 2
        )
        self.assertIn("AssetPointerContentRoute::Item", dispatch_source)
        self.assertIn('"SelectItem"', dispatch_source)
        self.assertIn('"AssetSurface/SelectItem"', binding_source)
        self.assertIn("AssetCommand::SelectItem", binding_source)


if __name__ == "__main__":
    unittest.main()
