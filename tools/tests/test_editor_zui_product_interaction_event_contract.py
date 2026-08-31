import unittest
from pathlib import Path

from tools.tests.test_editor_zui_base_radius_hierarchy_contract import (
    WELCOME,
    imported_zui_path,
    load_document,
    reachable_workbench_documents,
)


REPO_ROOT = Path(__file__).resolve().parents[2]
APP_EDIT = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/app/pane_surface_actions/edit.rs"
)
COMPONENT_LAB_EDIT = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/component_lab_field_edit.rs"
)
CONTROL_DISPATCH = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/workbench/control.rs"
)
TEMPLATE_BINDINGS = REPO_ROOT / (
    "zircon_editor/src/ui/template_runtime/builtin/"
    "workbench_window_template_bindings.rs"
)
BUILTIN_TEMPLATE_BINDINGS = REPO_ROOT / (
    "zircon_editor/src/ui/template_runtime/builtin/template_bindings.rs"
)
PREVIEW_ACTIONS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/workbench_preview_actions.rs"
)
WELCOME_PRESENTATION = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/ui/apply_presentation.rs"
)
PROJECT_OVERVIEW = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/project_overview.zui"
)
PROJECT_OVERVIEW_WORKSPACE = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/modules/extensions/"
    "production/workbench_extension_project_overview_workspace.zui"
)
PROJECT_OVERVIEW_FEEDBACK = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/extension_module_feedback/data_production.rs"
)
PROJECT_OVERVIEW_NAVIGATION = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/extension_module_navigation/specs/data_production.rs"
)
PROJECT_OVERVIEW_TEMPLATE_BINDINGS = REPO_ROOT / (
    "zircon_editor/src/ui/template_runtime/builtin/"
    "workbench_extension_module_template_bindings/render_asset_vfx.rs"
)
EXTENSION_PREVIEW_ACTIONS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/workbench_preview_actions/extensions.rs"
)
HIERARCHY = REPO_ROOT / "zircon_editor/assets/ui/editor/hierarchy.zui"
INSPECTOR_PANE_BODY = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/host/inspector_body.zui"
)
PERFORMANCE_TIMELINE_PANE_BODY = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/host/performance_timeline_body.zui"
)
PANE_SURFACE_CONTROLS = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/host/pane_surface_controls.zui"
)
VIEW_TEMPLATE_PROJECTION = REPO_ROOT / (
    "zircon_editor/src/ui/layouts/views/view_projection/materialization.rs"
)
TEMPLATE_NODE_CONVERSION = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/ui/template_node_conversion.rs"
)
TEMPLATE_ACTIVATION_ROUTE = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/"
    "template_activation_semantics/route.rs"
)
TEMPLATE_ACTIVATION_DISPATCH = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/"
    "template_activation_semantics/dispatch.rs"
)
ASSET_ACTIVATION = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/"
    "template_activation_semantics/asset.rs"
)
ASSET_CONTROL_IDS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/asset_control_ids.rs"
)
PANE_SURFACE_CLICK = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/app/pane_surface_actions/click.rs"
)
MENU_ACTION_FROM_ID = REPO_ROOT / (
    "zircon_editor/src/ui/workbench/event/menu_action_from_id.rs"
)
DYNAMIC_PRODUCT_VIEW_NAMES = (
    "asset_browser.zui",
    "assets_activity.zui",
    "project_overview.zui",
    "welcome.zui",
    "hierarchy.zui",
    "inspector.zui",
    "console.zui",
)


NATIVE_INTERACTIVE_COMPONENTS = {
    "Button",
    "Checkbox",
    "CommandPalette",
    "Dropdown",
    "IconButton",
    "InputField",
    "ListRow",
    "NumberField",
    "Radio",
    "RangeField",
    "RangeSlider",
    "SearchField",
    "SearchInput",
    "SegmentedControl",
    "Slider",
    "Tab",
    "TableRow",
    "Toggle",
    "ToggleButton",
    "TreeRow",
}


def interactive_component_registry(documents):
    registry = {}
    component_roots = set()
    for path, document in documents.items():
        for component_name, component in document.get("components", {}).items():
            root_name = component["root"]
            registry[component_name] = document["nodes"][root_name]["component"]
            component_roots.add((path, root_name))
    return registry, component_roots


def native_component(component, registry, seen=frozenset()):
    if component in NATIVE_INTERACTIVE_COMPONENTS:
        return component
    if component in seen or component not in registry:
        return None
    return native_component(component=registry[component], registry=registry, seen=seen | {component})


def reachable_documents(entry):
    documents = {}
    pending = [entry]
    while pending:
        path = pending.pop()
        if path in documents:
            continue
        document = load_document(path)
        documents[path] = document
        for category in ("widgets", "styles"):
            for reference in document.get("imports", {}).get(category, []):
                imported_path = imported_zui_path(reference)
                if imported_path is not None and imported_path.exists():
                    pending.append(imported_path)
    return documents


class EditorZuiProductInteractionEventContractTests(unittest.TestCase):
    def test_product_pane_bodies_expose_real_actions_instead_of_fixture_commands(self):
        inspector = load_document(INSPECTOR_PANE_BODY)
        timeline = load_document(PERFORMANCE_TIMELINE_PANE_BODY)
        pane_surface = load_document(PANE_SURFACE_CONTROLS)
        builtin_bindings = BUILTIN_TEMPLATE_BINDINGS.read_text(encoding="utf-8")

        inspector_controls = {
            node.get("control_id") for node in inspector.get("nodes", {}).values()
        }
        timeline_controls = {
            node.get("control_id") for node in timeline.get("nodes", {}).values()
        }
        inspector_binding_ids = {
            event["id"]
            for node in inspector.get("nodes", {}).values()
            for event in node.get("events", [])
        }
        timeline_binding_ids = {
            event["id"]
            for node in timeline.get("nodes", {}).values()
            for event in node.get("events", [])
        }
        pane_surface_controls = {
            node.get("control_id") for node in pane_surface.get("nodes", {}).values()
        }
        pane_surface_binding_ids = {
            event["id"]
            for node in pane_surface.get("nodes", {}).values()
            for event in node.get("events", [])
        }

        self.assertNotIn("ApplyDraft", inspector_controls)
        self.assertNotIn("InspectorPaneBody/ApplyDraft", inspector_binding_ids)
        self.assertNotIn("RefreshTimelineSnapshot", timeline_controls)
        self.assertNotIn(
            "PerformanceTimelinePaneBody/RefreshSnapshot", timeline_binding_ids
        )
        self.assertNotIn("TriggerAction", pane_surface_controls)
        self.assertNotIn("PaneSurface/TriggerAction", pane_surface_binding_ids)
        self.assertIn("InspectorBodySection", inspector_controls)
        self.assertIn("PerformanceTimelineCaptureControls", timeline_controls)
        self.assertNotIn('"InspectorPaneBody/ApplyDraft"', builtin_bindings)
        self.assertNotIn(
            '"PerformanceTimelinePaneBody/RefreshSnapshot"', builtin_bindings
        )
        self.assertNotIn('"PaneSurface/TriggerAction"', builtin_bindings)

    def test_project_overview_does_not_advertise_feedback_only_commands(self):
        workspace = load_document(PROJECT_OVERVIEW_WORKSPACE)
        controls = {
            node.get("control_id") for node in workspace.get("nodes", {}).values()
        }
        event_ids = {
            event["id"]
            for node in workspace.get("nodes", {}).values()
            for event in node.get("events", [])
        }
        event_routes = {
            event.get("route")
            for node in workspace.get("nodes", {}).values()
            for event in node.get("events", [])
        }

        self.assertNotIn("WorkbenchExtensionProjectOverviewRefreshButton", controls)
        self.assertNotIn("WorkbenchExtensionProjectOverviewPublishButton", controls)
        self.assertNotIn("WorkbenchExtension/ProjectOverviewRefresh", event_ids)
        self.assertNotIn("WorkbenchExtension/ProjectOverviewPublish", event_ids)
        self.assertNotIn(
            "workbench.extension.project_overview.refresh", event_routes
        )
        self.assertNotIn(
            "workbench.extension.project_overview.publish", event_routes
        )

        registered_sources = "\n".join(
            path.read_text(encoding="utf-8")
            for path in (
                PROJECT_OVERVIEW_FEEDBACK,
                PROJECT_OVERVIEW_NAVIGATION,
                PROJECT_OVERVIEW_TEMPLATE_BINDINGS,
                EXTENSION_PREVIEW_ACTIONS,
                PREVIEW_ACTIONS,
            )
        )
        self.assertNotIn(
            "workbench.extension.project_overview.refresh.invoke",
            registered_sources,
        )
        self.assertNotIn(
            "workbench.extension.project_overview.publish.invoke",
            registered_sources,
        )
        self.assertNotIn("Project overview refresh queued", registered_sources)
        self.assertNotIn("Project overview publish queued", registered_sources)

    def test_production_event_routes_are_globally_unique_for_trace_authority(self):
        routes = {}
        for asset_root in (
            REPO_ROOT / "zircon_editor/assets/ui",
            REPO_ROOT / "zircon_runtime/assets/ui",
        ):
            for path in asset_root.rglob("*.zui"):
                document = load_document(path)
                for node_name, node in document.get("nodes", {}).items():
                    for event_index, event in enumerate(node.get("events", []), start=1):
                        route = event.get("route")
                        if route is None:
                            continue
                        routes.setdefault(route, []).append(
                            f"{path.relative_to(REPO_ROOT)}::{node_name}#{event_index}"
                        )

        duplicates = {
            route: owners for route, owners in routes.items() if len(owners) > 1
        }
        self.assertEqual({}, duplicates)

    def test_dynamic_product_views_have_no_undocumented_eventless_interactions(self):
        expected_eventless_controls = {
            "asset_browser.zui": set(),
            "assets_activity.zui": set(),
            "project_overview.zui": {"OpenAssetsView", "OpenAssetBrowser"},
            "welcome.zui": {"WelcomeProjectNameField", "WelcomeLocationField"},
            "hierarchy.zui": set(),
            "inspector.zui": set(),
            "console.zui": set(),
        }
        actual_eventless_controls = {}

        for asset_name in DYNAMIC_PRODUCT_VIEW_NAMES:
            entry = REPO_ROOT / "zircon_editor/assets/ui/editor" / asset_name
            documents = reachable_documents(entry)
            registry, component_roots = interactive_component_registry(documents)
            eventless = set()
            for path, document in documents.items():
                for node_name, node in document.get("nodes", {}).items():
                    native = native_component(node.get("component"), registry)
                    if native is None or (path, node_name) in component_roots:
                        continue
                    props = node.get("props", {})
                    if (
                        props.get("disabled") is True
                        or props.get("input_interactive") is False
                        or node.get("events")
                    ):
                        continue
                    eventless.add(node.get("control_id", ""))
            actual_eventless_controls[asset_name] = eventless

        self.assertEqual(expected_eventless_controls, actual_eventless_controls)

    def test_product_reachable_enabled_interaction_consumers_have_events(self):
        documents = reachable_workbench_documents()
        registry, component_roots = interactive_component_registry(documents)
        missing = []

        for path, document in documents.items():
            for node_name, node in document.get("nodes", {}).items():
                native = native_component(node.get("component"), registry)
                if native is None or (path, node_name) in component_roots:
                    continue
                props = node.get("props", {})
                explicitly_inert = (
                    props.get("disabled") is True
                    or props.get("input_interactive") is False
                )
                if not explicitly_inert and not node.get("events"):
                    missing.append((path.name, node_name, node.get("component"), native))

        self.assertEqual([], missing)

    def test_workbench_scene_search_reuses_hierarchy_filter_authority(self):
        app_edit = APP_EDIT.read_text(encoding="utf-8")
        bindings = TEMPLATE_BINDINGS.read_text(encoding="utf-8")
        preview_actions = PREVIEW_ACTIONS.read_text(encoding="utf-8")
        hierarchy = load_document(HIERARCHY)
        hierarchy_search = hierarchy["nodes"]["search_field"]

        self.assertNotIn("WORKBENCH_SCENE_SEARCH_CONTROL_ID", app_edit)
        self.assertNotIn('control_id == "HierarchySearchQuery"', app_edit)
        self.assertIn('"Workbench/SceneSearchEdit"', app_edit)
        self.assertIn('"Workbench/SceneSearchCommit"', app_edit)
        self.assertIn("set_hierarchy_filter_query(value)", app_edit)
        self.assertIn('"SceneSearchEdit"', bindings)
        self.assertIn('"SceneSearchCommit"', bindings)
        self.assertIn('"workbench.hierarchy.search.edit"', preview_actions)
        self.assertIn('"workbench.hierarchy.search.commit"', preview_actions)
        self.assertEqual(
            [
                {
                    "id": "Workbench/SceneSearchEdit",
                    "event": "Change",
                    "component_event": "ValueChanged",
                    "route": "workbench.hierarchy.search.edit",
                },
                {
                    "id": "Workbench/SceneSearchCommit",
                    "event": "Submit",
                    "component_event": "Commit",
                    "route": "workbench.hierarchy.search.commit",
                },
            ],
            hierarchy_search.get("events"),
        )

    def test_welcome_eventless_fields_have_explicit_native_dispatch_authority(self):
        documents = reachable_documents(WELCOME)
        registry, component_roots = interactive_component_registry(documents)
        native_dispatch = WELCOME_PRESENTATION.read_text(encoding="utf-8")
        native_controls = {
            "WelcomeProjectNameField": "welcome.project.name.edit",
            "WelcomeLocationField": "welcome.project.location.edit",
        }
        missing = []

        for path, document in documents.items():
            for node_name, node in document.get("nodes", {}).items():
                native = native_component(node.get("component"), registry)
                if native is None or (path, node_name) in component_roots:
                    continue
                props = node.get("props", {})
                if (
                    props.get("disabled") is True
                    or props.get("input_interactive") is False
                    or node.get("events")
                ):
                    continue
                control_id = node.get("control_id", "")
                action_id = native_controls.get(control_id)
                if (
                    action_id is None
                    or f'"{control_id}"' not in native_dispatch
                    or f'"{action_id}"' not in native_dispatch
                ):
                    missing.append((path.name, node_name, node.get("component"), native))

        self.assertIn('node.dispatch_kind = "welcome_text".into()', native_dispatch)
        self.assertEqual([], missing)

    def test_project_overview_eventless_buttons_have_native_projection_authority(self):
        documents = reachable_documents(PROJECT_OVERVIEW)
        registry, component_roots = interactive_component_registry(documents)
        eventless_controls = {}

        for path, document in documents.items():
            for node_name, node in document.get("nodes", {}).items():
                native = native_component(node.get("component"), registry)
                if native is None or (path, node_name) in component_roots:
                    continue
                props = node.get("props", {})
                if (
                    props.get("disabled") is True
                    or props.get("input_interactive") is False
                    or node.get("events")
                ):
                    continue
                eventless_controls[node.get("control_id", "")] = props

        self.assertEqual(
            {"OpenAssetsView", "OpenAssetBrowser"},
            set(eventless_controls),
        )
        self.assertEqual(
            {
                "dispatch_kind": "surface",
                "action_id": "workbench.view.open.editor.assets",
            },
            {
                key: eventless_controls["OpenAssetsView"].get(key)
                for key in ("dispatch_kind", "action_id")
            },
        )
        self.assertEqual(
            "asset",
            eventless_controls["OpenAssetBrowser"].get("dispatch_kind"),
        )

        materialization = VIEW_TEMPLATE_PROJECTION.read_text(encoding="utf-8")
        conversion = TEMPLATE_NODE_CONVERSION.read_text(encoding="utf-8")
        route = TEMPLATE_ACTIVATION_ROUTE.read_text(encoding="utf-8")
        dispatch = TEMPLATE_ACTIVATION_DISPATCH.read_text(encoding="utf-8")
        asset = ASSET_ACTIVATION.read_text(encoding="utf-8")
        asset_ids = ASSET_CONTROL_IDS.read_text(encoding="utf-8")
        click = PANE_SURFACE_CLICK.read_text(encoding="utf-8")
        menu_action = MENU_ACTION_FROM_ID.read_text(encoding="utf-8")

        self.assertIn('string_attribute(metadata, "dispatch_kind")', materialization)
        self.assertIn('string_attribute(metadata, "action_id")', materialization)
        self.assertIn("dispatch_kind: data.dispatch_kind.clone()", conversion)
        self.assertIn("action_id: data.action_id.clone()", conversion)
        self.assertIn("asset_dispatch_source(kind).is_some()", route)
        self.assertIn("TemplatePrimaryActivationRoute::SurfaceAction", route)
        self.assertIn("dispatch_asset_template_node_primary_press", dispatch)
        self.assertIn(
            "invoke_surface_control_clicked(hit.control_id, hit.action_id)",
            dispatch,
        )
        self.assertIn("invoke_asset_control_clicked", asset)
        self.assertIn(
            '"OpenAssetBrowser" | "workbench.asset_browser.open"',
            asset_ids,
        )
        self.assertIn("callback_dispatch::dispatch_menu_action", click)
        self.assertIn('.strip_prefix("workbench.view.open.")', menu_action)

    def test_component_lab_fields_have_local_state_and_binding_authority(self):
        bridge = COMPONENT_LAB_EDIT.read_text(encoding="utf-8")
        production_bridge = bridge.split("#[cfg(test)]", 1)[0]
        dispatch = CONTROL_DISPATCH.read_text(encoding="utf-8")
        bindings = TEMPLATE_BINDINGS.read_text(encoding="utf-8")
        preview_actions = PREVIEW_ACTIONS.read_text(encoding="utf-8")

        self.assertIn("edit_component_lab_field", dispatch)
        self.assertIn('"value_percent"', bridge)
        self.assertIn('"value_text"', bridge)
        self.assertIn('"query"', bridge)
        self.assertNotIn("let value = value.trim();", bridge)
        self.assertNotIn("SEARCH_CONTROL", production_bridge)
        self.assertNotIn("WorkbenchInputSearch", production_bridge)
        self.assertIn("control_string(control_id, QUERY_PROPERTY)", production_bridge)
        self.assertIn('"component_lab.input_search.edit"', preview_actions)
        self.assertIn('"component_lab.input_search.commit"', preview_actions)
        self.assertIn('&format!("{control}Edit")', bindings)
        self.assertIn('&format!("{control}Commit")', bindings)
        for control, action in (
            ("InputStepper", "input_stepper"),
            ("InputSlider", "input_slider"),
            ("InputRangeSlider", "input_range_slider"),
            ("InputStepsSlider", "input_steps_slider"),
        ):
            self.assertIn(f'("{control}", "{action}")', bindings)
            self.assertIn(f'"component_lab.{action}.edit"', preview_actions)
            self.assertIn(f'"component_lab.{action}.commit"', preview_actions)


if __name__ == "__main__":
    unittest.main()
