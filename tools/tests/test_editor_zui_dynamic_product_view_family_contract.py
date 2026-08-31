import re
import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
ASSET_ROOT = REPO_ROOT / "zircon_editor/assets/ui/editor"
STRICT_THEME = "res://ui/theme/editor_workbench_strict.zui"
STRICT_THEME_SOURCE = REPO_ROOT / "zircon_editor/assets/ui/theme/editor_workbench_strict.zui"
VIEW_PROJECTION_ROOT = REPO_ROOT / "zircon_editor/src/ui/layouts/views/view_projection"
DYNAMIC_VIEW_ROOT = REPO_ROOT / "zircon_editor/src/ui/layouts/views"
INSPECTOR_PROJECTION_SOURCE = DYNAMIC_VIEW_ROOT / "inspector.rs"
NUMERIC_CORNER_RADIUS = re.compile(r"corner_radius:\s*([0-9]+(?:\.[0-9]+)?)")
NUMERIC_FONT_SIZE_CONSTANT = re.compile(
    r"const\s+[A-Z0-9_]*FONT_SIZE:\s*f32\s*=\s*[0-9]+(?:\.[0-9]+)?"
)

WORKBENCH_WIDGETS = {
    "WorkbenchButton": (
        "res://ui/editor/components/workbench/primitives/inputs/"
        "workbench_button.zui#WorkbenchButton"
    ),
    "WorkbenchField": (
        "res://ui/editor/components/workbench/primitives/inputs/"
        "workbench_field.zui#WorkbenchField"
    ),
    "WorkbenchIconButton": (
        "res://ui/editor/components/workbench/primitives/inputs/"
        "workbench_icon_button.zui#WorkbenchIconButton"
    ),
    "WorkbenchDropdown": (
        "res://ui/editor/components/workbench/primitives/inputs/"
        "workbench_dropdown.zui#WorkbenchDropdown"
    ),
    "WorkbenchSearchInput": (
        "res://ui/editor/components/workbench/primitives/inputs/"
        "workbench_search_input.zui#WorkbenchSearchInput"
    ),
    "WorkbenchTab": (
        "res://ui/editor/components/workbench/primitives/inputs/"
        "workbench_tab.zui#WorkbenchTab"
    ),
    "WorkbenchTableRow": (
        "res://ui/editor/components/workbench/primitives/data/"
        "workbench_table_row.zui#WorkbenchTableRow"
    ),
    "WorkbenchPropertyEditorRow": (
        "res://ui/editor/components/workbench/composites/inputs/"
        "workbench_property_editor_row.zui#WorkbenchPropertyEditorRow"
    ),
}

PRODUCT_VIEWS = {
    "asset_browser.zui": {
        "WorkbenchButton": {"import_button"},
        "WorkbenchDropdown": {"toolbar_kind_filter_dropdown"},
        "WorkbenchField": {"import_path_field"},
        "WorkbenchIconButton": {
            "toolbar_locate_button",
            "toolbar_view_mode_list_button",
            "toolbar_view_mode_thumb_button",
        },
        "WorkbenchSearchInput": {"toolbar_search_field"},
        "WorkbenchTab": {
            "utility_preview_button",
            "utility_references_button",
            "utility_metadata_button",
            "utility_plugins_button",
        },
        "WorkbenchTableRow": {
            "content_asset_table_header",
            "content_asset_row_01",
            "content_asset_row_02",
            "content_asset_row_03",
            "content_asset_row_04",
        },
    },
    "assets_activity.zui": {
        "WorkbenchDropdown": {"toolbar_kind_filter_dropdown"},
        "WorkbenchIconButton": {
            "toolbar_open_browser_button",
            "toolbar_view_mode_list_button",
            "toolbar_view_mode_thumb_button",
        },
        "WorkbenchSearchInput": {"toolbar_search_field"},
        "WorkbenchTab": {
            "utility_preview_button",
            "utility_references_button",
        },
    },
    "project_overview.zui": {
        "WorkbenchButton": {
            "catalog_open_assets_button",
            "catalog_open_browser_button",
        },
    },
    "hierarchy.zui": {
        "WorkbenchSearchInput": {"search_field"},
    },
    "inspector.zui": {
        "WorkbenchField": {
            "name_value",
            "parent_value",
            "position_value",
            "components_value",
        },
        "WorkbenchPropertyEditorRow": {
            "name_row",
            "parent_row",
            "position_row",
            "actions_row",
        },
    },
    "console.zui": {},
}

PRODUCT_VIEW_SURFACE_CLASSES = {
    "asset_browser.zui": {
        "asset_browser_root": "workbench-shell-root",
    },
    "assets_activity.zui": {
        "assets_activity_root": "workbench-shell-root",
    },
    "project_overview.zui": {
        "project_overview_root": "workbench-shell-root",
    },
    "hierarchy.zui": {
        "hierarchy_root": "workbench-shell-root",
    },
    "inspector.zui": {
        "inspector_root": "workbench-shell-root",
        "separator_row": "workbench-divider",
    },
    "console.zui": {
        "console_root": "workbench-shell-root",
    },
}

PRODUCT_VIEW_DENSITY_SPACERS = {
    "hierarchy.zui": {
        "search_left_inset": ("width", "$editor.density.gap.medium"),
        "search_right_inset": ("width", "$editor.density.gap.medium"),
        "header_list_gap": ("height", "$editor.density.gap.small"),
        "search_list_gap": ("height", "$editor.density.gap.small"),
    },
    "console.zui": {
        "console_header_body_gap": ("height", "$editor.density.gap.small"),
        "console_body_top_space": ("height", "$editor.density.gap.small"),
        "console_text_left_space": ("width", "$editor.density.gap.medium"),
        "console_text_right_space": ("width", "$editor.density.gap.medium"),
    },
    "inspector.zui": {
        "header_name_gap": ("height", "$editor.density.gap.small"),
    },
}

LEGACY_SURFACE_CLASSES = {"editor-shell", "editor-pane", "panel", "inset"}

PRODUCT_VIEW_LOADERS = {
    "asset_browser.zui": (
        REPO_ROOT / "zircon_editor/src/ui/layouts/views/asset_browser.rs",
        2,
    ),
    "assets_activity.zui": (
        REPO_ROOT / "zircon_editor/src/ui/layouts/views/assets_activity.rs",
        1,
    ),
    "project_overview.zui": (
        REPO_ROOT / "zircon_editor/src/ui/layouts/views/project_overview.rs",
        1,
    ),
    "welcome.zui": (
        REPO_ROOT / "zircon_editor/src/ui/layouts/views/welcome.rs",
        1,
    ),
    "hierarchy.zui": (
        REPO_ROOT / "zircon_editor/src/ui/layouts/views/hierarchy.rs",
        1,
    ),
    "inspector.zui": (
        REPO_ROOT / "zircon_editor/src/ui/layouts/views/inspector.rs",
        1,
    ),
    "console.zui": (
        REPO_ROOT / "zircon_editor/src/ui/layouts/views/console.rs",
        1,
    ),
}

LEGACY_THEME_SOURCES = {
    "editor_base.zui",
    "editor_material.zui",
    "editor_tokens.zui",
}

EXPECTED_EVENT_IDENTITIES = {
    "asset_browser.zui": {
        (
            "AssetsView/MeshImportPathEdited",
            "workbench.asset.mesh_import.path.set",
        ),
        ("Workbench/AssetBrowserImportModel", "workbench.asset_browser.import_model"),
        ("Workbench/AssetBrowserLocateSelected", "workbench.asset_browser.locate_selected"),
        ("Workbench/AssetBrowserSearchEdited", "workbench.asset_browser.search.edit"),
        ("Workbench/AssetBrowserKindFilterChanged", "workbench.asset_browser.kind_filter.change"),
        ("Workbench/AssetBrowserViewModeList", "workbench.asset_browser.view_mode.list"),
        ("Workbench/AssetBrowserViewModeThumbnail", "workbench.asset_browser.view_mode.thumbnail"),
        ("Workbench/AssetBrowserUtilityPreview", "workbench.asset_browser.utility.preview"),
        ("Workbench/AssetBrowserUtilityReferences", "workbench.asset_browser.utility.references"),
        ("Workbench/AssetBrowserUtilityMetadata", "workbench.asset_browser.utility.metadata"),
        ("Workbench/AssetBrowserUtilityPlugins", "workbench.asset_browser.utility.plugins"),
    },
    "assets_activity.zui": {
        ("Workbench/AssetsActivityOpenBrowser", "view.asset_browser.open"),
        ("Workbench/AssetsActivitySearchEdited", "workbench.assets_activity.search.edit"),
        ("Workbench/AssetsActivityKindFilterChanged", "workbench.assets_activity.kind_filter.change"),
        ("Workbench/AssetsActivityViewModeList", "workbench.assets_activity.view_mode.list"),
        ("Workbench/AssetsActivityViewModeThumbnail", "workbench.assets_activity.view_mode.thumbnail"),
        ("Workbench/AssetsActivityUtilityPreview", "workbench.assets_activity.utility.preview"),
        ("Workbench/AssetsActivityUtilityReferences", "workbench.assets_activity.utility.references"),
    },
    "project_overview.zui": set(),
    "hierarchy.zui": {
        (
            "Workbench/SceneSearchEdit",
            "workbench.hierarchy.search.edit",
        ),
        (
            "Workbench/SceneSearchCommit",
            "workbench.hierarchy.search.commit",
        ),
    },
    "inspector.zui": set(),
    "console.zui": set(),
}

NATIVE_INTERACTIVE_COMPONENTS = {
    "Button",
    "IconButton",
    "InputField",
    "ListRow",
    "SearchField",
    "Table",
    "TextField",
    "ToggleButton",
}
LIVE_STATE_KEYS = {"checked", "focused", "pressed", "selected"}


def load_document(name: str) -> dict:
    with (ASSET_ROOT / name).open("rb") as source:
        return tomllib.load(source)


def event_identity(event: dict) -> tuple[str, str]:
    action = event.get("action", "")
    if isinstance(action, dict):
        action = action.get("action", "")
    return event["id"], event.get("route", action)


class EditorZuiDynamicProductViewFamilyContractTests(unittest.TestCase):
    def test_rust_loaded_product_views_use_the_strict_workbench_family(self):
        for asset_name, expected_families in PRODUCT_VIEWS.items():
            document = load_document(asset_name)
            imports = document["imports"]
            self.assertEqual([STRICT_THEME], imports["styles"], asset_name)

            expected_widgets = {
                WORKBENCH_WIDGETS[family] for family in expected_families
            }
            self.assertEqual(
                expected_widgets, set(imports.get("widgets", [])), asset_name
            )

            nodes = document["nodes"]
            native_bypasses = {
                node_name
                for node_name, node in nodes.items()
                if node.get("component") in NATIVE_INTERACTIVE_COMPONENTS
            }
            self.assertEqual(set(), native_bypasses, asset_name)

            for family, expected_nodes in expected_families.items():
                actual_nodes = {
                    node_name
                    for node_name, node in nodes.items()
                    if node.get("component") == family
                }
                self.assertEqual(expected_nodes, actual_nodes, f"{asset_name}:{family}")
                if family == "WorkbenchTab":
                    for node_name in expected_nodes:
                        props = nodes[node_name].get("props", {})
                        self.assertNotIn("value", props, f"{asset_name}:{node_name}")
                        self.assertIn("value_text", props, f"{asset_name}:{node_name}")
                if family == "WorkbenchSearchInput":
                    for node_name in expected_nodes:
                        props = nodes[node_name].get("props", {})
                        for redundant_prop in (
                            "value",
                            "surface_variant",
                            "border_width",
                            "corner_radius",
                        ):
                            self.assertNotIn(
                                redundant_prop,
                                props,
                                f"{asset_name}:{node_name}",
                            )

    def test_dynamic_product_surfaces_use_workbench_style_roles(self):
        for asset_name, expected_classes in PRODUCT_VIEW_SURFACE_CLASSES.items():
            nodes = load_document(asset_name)["nodes"]
            for node_name, expected_class in expected_classes.items():
                self.assertIn(
                    expected_class,
                    nodes[node_name].get("classes", []),
                    f"{asset_name}:{node_name}",
                )

        for asset_name in PRODUCT_VIEWS:
            nodes = load_document(asset_name)["nodes"]
            legacy_references = {
                f"{node_name}:{class_name}"
                for node_name, node in nodes.items()
                for class_name in node.get("classes", [])
                if class_name in LEGACY_SURFACE_CLASSES
            }
            self.assertEqual(set(), legacy_references, asset_name)

        strict_classes = set(
            re.findall(
                r'^selector\s*=\s*"\.([A-Za-z0-9_-]+)',
                STRICT_THEME_SOURCE.read_text(encoding="utf-8"),
                re.MULTILINE,
            )
        )
        for asset_name in PRODUCT_VIEWS:
            nodes = load_document(asset_name)["nodes"]
            unresolved_classes = {
                f"{node_name}:{class_name}"
                for node_name, node in nodes.items()
                for class_name in node.get("classes", [])
                if class_name not in strict_classes
            }
            self.assertEqual(set(), unresolved_classes, asset_name)

    def test_project_overview_uses_one_unframed_scroll_surface(self):
        nodes = load_document("project_overview.zui")["nodes"]

        outer_panel = nodes["outer_panel"]
        self.assertNotIn("workbench-panel", outer_panel.get("classes", []))
        self.assertEqual([{"node": "content_stack"}], outer_panel["children"])
        for redundant_inset_node in (
            "outer_top_space",
            "outer_bottom_space",
            "content_inset_left",
            "content_inset_right",
            "content_inset_row",
        ):
            self.assertNotIn(redundant_inset_node, nodes)
        for redundant_surface_prop in ("surface_variant", "radius", "border_width"):
            self.assertNotIn(redundant_surface_prop, outer_panel.get("props", {}))
            self.assertNotIn(
                redundant_surface_prop,
                nodes["details_panel"].get("props", {}),
            )
            self.assertNotIn(
                redundant_surface_prop,
                nodes["catalog_panel"].get("props", {}),
            )

        scroll = nodes["content_stack"]
        self.assertEqual("ScrollableBox", scroll["component"])
        self.assertEqual("ProjectOverviewContentScroll", scroll["control_id"])
        self.assertTrue(scroll["props"]["input_hoverable"])
        self.assertTrue(scroll["layout"]["clip"])
        self.assertEqual("Receive", scroll["layout"]["input_policy"])
        self.assertEqual(
            {
                "kind": "ScrollableBox",
                "axis": "Vertical",
                "gap": 0.0,
                "scrollbar_visibility": "Auto",
            },
            scroll["layout"]["container"],
        )
        self.assertNotIn("bottom_fill", nodes)

        self.assertEqual(
            {"min": 144.0, "preferred": 144.0, "max": 144.0, "stretch": "Fixed"},
            nodes["details_panel"]["layout"]["height"],
        )
        self.assertEqual(
            {"min": 132.0, "preferred": 132.0, "max": 132.0, "stretch": "Fixed"},
            nodes["catalog_panel"]["layout"]["height"],
        )

    def test_inspector_uses_continuous_rows_below_a_fixed_header(self):
        document = load_document("inspector.zui")
        nodes = document["nodes"]

        self.assertEqual(
            ["header_panel", "header_name_gap", "body_overlay"],
            [entry["node"] for entry in nodes["inspector_root"]["children"]],
        )
        body = nodes["inspector_scroll_body"]
        self.assertEqual("ScrollableBox", body["component"])
        self.assertEqual("InspectorScrollBody", body["control_id"])
        self.assertTrue(body["props"]["input_hoverable"])
        self.assertEqual(
            {
                "kind": "ScrollableBox",
                "axis": "Vertical",
                "gap": 0.0,
                "scrollbar_visibility": "Auto",
            },
            body["layout"]["container"],
        )
        self.assertEqual("Receive", body["layout"]["input_policy"])
        self.assertEqual(
            ["name_row", "parent_row", "position_row", "separator_row", "actions_row"],
            [entry["node"] for entry in body["children"]],
        )

        for row_name in ("name_row", "parent_row", "position_row", "actions_row"):
            row = nodes[row_name]
            self.assertNotIn("workbench-panel", row.get("classes", []))
            for redundant_surface_prop in ("surface_variant", "radius", "border_width"):
                self.assertNotIn(redundant_surface_prop, row.get("props", {}))

        for row_name, label, value_name, value_control_id in (
            ("name_row", "Name", "name_value", "InspectorNameValue"),
            ("parent_row", "Parent", "parent_value", "InspectorParentValue"),
            ("position_row", "Position", "position_value", "InspectorPositionValue"),
            ("actions_row", "Components", "components_value", "InspectorComponentsValue"),
        ):
            row = nodes[row_name]
            value = nodes[value_name]
            self.assertEqual("WorkbenchPropertyEditorRow", row["component"])
            self.assertEqual(label, row["props"]["text"])
            self.assertEqual(
                [{"node": value_name, "slot": {"name": "value"}}],
                row["children"],
            )
            self.assertEqual("WorkbenchField", value["component"])
            self.assertEqual(value_control_id, value["control_id"])
            self.assertFalse(value["props"]["editable_text"])
            for input_prop in (
                "input_interactive",
                "input_clickable",
                "input_hoverable",
                "input_focusable",
            ):
                self.assertFalse(value["props"][input_prop])

        empty_state = nodes["bottom_fill"]
        self.assertNotIn("workbench-panel", empty_state.get("classes", []))
        self.assertEqual("transparent", empty_state["props"]["surface_variant"])
        self.assertEqual(
            ["inspector_scroll_body", "bottom_fill"],
            [entry["node"] for entry in nodes["body_overlay"]["children"]],
        )

        projection_source = INSPECTOR_PROJECTION_SOURCE.read_text(encoding="utf-8")
        self.assertNotIn("\u2022", projection_source)
        for value_control_id in (
            "InspectorNameValue",
            "InspectorParentValue",
            "InspectorPositionValue",
            "InspectorComponentsValue",
        ):
            self.assertIn(value_control_id, projection_source)
        self.assertNotIn("selected: Some(active)", projection_source)
        self.assertIn("selected: Some(false)", projection_source)
        empty_state_patch = projection_source.split("fn mark_empty_state", 1)[1].split(
            "fn mark_readout", 1
        )[0]
        self.assertIn("if has_selection", empty_state_patch)
        self.assertIn('"transparent"', empty_state_patch)
        self.assertIn('"inset"', empty_state_patch)

    def test_dynamic_product_shell_spacing_uses_shared_density_tokens(self):
        for asset_name, expected_spacers in PRODUCT_VIEW_DENSITY_SPACERS.items():
            nodes = load_document(asset_name)["nodes"]
            for node_name, (axis, token) in expected_spacers.items():
                dimension = nodes[node_name]["layout"][axis]
                self.assertEqual(
                    {
                        "min": token,
                        "preferred": token,
                        "max": token,
                        "stretch": "Fixed",
                    },
                    dimension,
                    f"{asset_name}:{node_name}:{axis}",
                )

    def test_family_migration_preserves_existing_event_identities(self):
        for asset_name, expected_identities in EXPECTED_EVENT_IDENTITIES.items():
            document = load_document(asset_name)
            identities = {
                event_identity(event)
                for node in document["nodes"].values()
                for event in node.get("events", [])
            }
            self.assertEqual(expected_identities, identities, asset_name)

    def test_dynamic_product_views_do_not_author_live_control_state(self):
        for asset_name, expected_families in PRODUCT_VIEWS.items():
            document = load_document(asset_name)
            nodes = document["nodes"]
            violations = []
            migrated_nodes = (
                set().union(*expected_families.values())
                if expected_families
                else set()
            )
            for node_name in migrated_nodes:
                authored = LIVE_STATE_KEYS.intersection(
                    nodes[node_name].get("props", {})
                )
                if isinstance(nodes[node_name].get("props", {}).get("value"), bool):
                    authored.add("value")
                if authored:
                    violations.append(f"{node_name}:{sorted(authored)}")
            self.assertEqual([], violations, asset_name)

    def test_rust_projection_uses_the_document_import_graph_as_authority(self):
        for asset_name, (source_path, expected_empty_imports) in (
            PRODUCT_VIEW_LOADERS.items()
        ):
            source = source_path.read_text(encoding="utf-8")
            for legacy_source in LEGACY_THEME_SOURCES:
                self.assertNotIn(legacy_source, source, asset_name)
            self.assertEqual(expected_empty_imports, source.count("&[]"), asset_name)

    def test_rust_projected_product_surfaces_do_not_reintroduce_sub_token_rounding(self):
        violations = []
        for path in DYNAMIC_VIEW_ROOT.rglob("*.rs"):
            source = path.read_text(encoding="utf-8")
            for match in NUMERIC_CORNER_RADIUS.finditer(source):
                radius = float(match.group(1))
                if 0.0 < radius < 6.0:
                    line = source.count("\n", 0, match.start()) + 1
                    relative_path = path.relative_to(REPO_ROOT).as_posix()
                    violations.append(f"{relative_path}:{line}:{radius:g}")

        self.assertEqual([], violations)

    def test_rust_projected_product_typography_uses_shared_tokens(self):
        violations = []
        for path in DYNAMIC_VIEW_ROOT.rglob("*.rs"):
            source = path.read_text(encoding="utf-8").split("#[cfg(test)]", 1)[0]
            for match in NUMERIC_FONT_SIZE_CONSTANT.finditer(source):
                line = source.count("\n", 0, match.start()) + 1
                relative_path = path.relative_to(REPO_ROOT).as_posix()
                violations.append(f"{relative_path}:{line}:{match.group(0)}")

        self.assertEqual([], violations)

    def test_dynamic_product_fixed_labels_preserve_authored_line_heights(self):
        font_sizes = {
            "$editor.typography.caption.size": 8.0 * 96.0 / 72.0,
            "$editor.typography.overlay.size": 9.0 * 96.0 / 72.0,
            "$editor.typography.body.size": 10.0 * 96.0 / 72.0,
            "$editor.typography.title.size": 14.0 * 96.0 / 72.0,
        }
        violations = []

        for asset_name in PRODUCT_VIEWS:
            for node_name, node in load_document(asset_name)["nodes"].items():
                if node.get("component") != "Label":
                    continue
                height = node.get("layout", {}).get("height", {})
                if height.get("stretch") != "Fixed":
                    continue

                props = node.get("props", {})
                font_size = props.get("font_size")
                if isinstance(font_size, str):
                    requested = font_sizes.get(font_size)
                elif isinstance(font_size, (int, float)):
                    requested = float(font_size)
                elif props.get("text_tone", "") in {
                    "muted",
                    "subtle",
                    "secondary",
                }:
                    requested = font_sizes["$editor.typography.caption.size"]
                else:
                    requested = font_sizes["$editor.typography.body.size"]
                if requested is None:
                    continue

                required = requested * 1.2
                actual = float(height["preferred"])
                if actual + 1.0e-6 < required:
                    violations.append(
                        f"{asset_name}:{node_name}:{actual:g}<{required:g}"
                    )

        self.assertEqual([], violations)

    def test_retained_text_does_not_shrink_authored_fonts_to_short_slots(self):
        metrics = (
            REPO_ROOT
            / "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "template_node_text/metrics.rs"
        ).read_text(encoding="utf-8")

        self.assertNotIn("requested.min(available_height)", metrics)
        self.assertIn(
            "node_font_size_from_host(&body, 10.0, metrics), 15.0",
            metrics,
        )
        self.assertIn(
            "node_font_size_from_host(&caption, 6.0, metrics), 11.0",
            metrics,
        )

    def test_search_field_uses_shared_text_input_projection_semantics(self):
        semantics = (VIEW_PROJECTION_ROOT / "component_semantics.rs").read_text(
            encoding="utf-8"
        )
        materialization = (VIEW_PROJECTION_ROOT / "materialization.rs").read_text(
            encoding="utf-8"
        )
        normalized_materialization = " ".join(materialization.split())

        self.assertEqual(
            1,
            semantics.count(
                '"InputField" | "SearchField" | "TextField" | "NumberField" => '
                '"InputField"'
            ),
        )
        self.assertEqual(
            1,
            semantics.count(
                '"InputField" | "SearchField" | "TextField" => "input-field"'
            ),
        )
        self.assertIn(
            '"Button" | "EditableTable" | "InputField" | "NumberField" '
            '| "SearchField" | "Table" | "TextField" => true',
            normalized_materialization,
        )

    def test_asset_browser_main_columns_use_bounded_weighted_slots(self):
        nodes = load_document("asset_browser.zui")["nodes"]
        expected_widths = {
            "sources_panel": {
                "min": 188.0,
                "preferred": 220.0,
                "max": 250.0,
                "weight": 1.0,
                "stretch": "Stretch",
            },
            "content_panel": {
                "min": 320.0,
                "preferred": 640.0,
                "weight": 4.0,
                "stretch": "Stretch",
            },
            "details_panel": {
                "min": 210.0,
                "preferred": 260.0,
                "max": 310.0,
                "weight": 1.0,
                "stretch": "Stretch",
            },
        }

        children = nodes["main_panel"]["children"]
        actual_widths = {
            child["node"]: child["slot"]["layout"]["width"]
            for child in children
        }
        self.assertEqual(expected_widths, actual_widths)
        for node_name in expected_widths:
            self.assertEqual(
                "Stretch", nodes[node_name]["layout"]["width"]["stretch"]
            )

    def test_asset_browser_details_text_slots_preserve_workbench_line_heights(self):
        nodes = load_document("asset_browser.zui")["nodes"]
        caption_line_height = (8.0 * 96.0 / 72.0) * 1.2
        body_line_height = (10.0 * 96.0 / 72.0) * 1.2
        violations = []

        for node_name, node in nodes.items():
            if node.get("component") != "Label":
                continue
            if not node.get("control_id", "").startswith("AssetBrowserDetails"):
                continue
            height = node.get("layout", {}).get("height", {})
            if height.get("stretch") != "Fixed":
                continue

            tone = node.get("props", {}).get("text_tone", "")
            required = (
                caption_line_height
                if tone in {"muted", "subtle", "secondary"}
                else body_line_height
            )
            actual = float(height["preferred"])
            if actual + 1.0e-6 < required:
                violations.append(f"{node_name}:{actual:g}<{required:g}")

        self.assertEqual([], violations)

    def test_asset_browser_import_path_routes_to_the_existing_draft_action(self):
        node = load_document("asset_browser.zui")["nodes"]["import_path_field"]
        self.assertNotIn("dispatch_kind", node.get("props", {}))
        self.assertEqual(
            [
                {
                    "id": "AssetsView/MeshImportPathEdited",
                    "event": "Change",
                    "route": "workbench.asset.mesh_import.path.set",
                }
            ],
            node.get("events", []),
        )

        snapshot_source = (
            REPO_ROOT
            / "zircon_editor/src/ui/workbench/snapshot/asset/asset_workspace_snapshot.rs"
        ).read_text(encoding="utf-8")
        state_projection = (
            REPO_ROOT
            / "zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs"
        ).read_text(encoding="utf-8")
        browser_projection = (
            REPO_ROOT / "zircon_editor/src/ui/layouts/views/asset_browser.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("pub mesh_import_path: String", snapshot_source)
        self.assertRegex(
            state_projection,
            r"asset_browser\s*\.mesh_import_path\s*"
            r"\.clone_from\(&self\.mesh_import_path\)",
        )
        self.assertIn("mesh_import_path: snapshot.mesh_import_path.clone()", browser_projection)
        self.assertIn(
            '"AssetBrowserImportPathField".to_string(),\n'
            "        snapshot.mesh_import_path.clone()",
            browser_projection,
        )

        compact_layout = (
            REPO_ROOT
            / "zircon_editor/src/ui/layouts/views/asset_browser/compact_layout.rs"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "EditorTypographyTokens::WORKBENCH_CAPTION_SIZE\n"
            "    * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO",
            compact_layout,
        )
        self.assertIn(
            "EditorTypographyTokens::WORKBENCH_BODY_SIZE\n"
            "    * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO",
            compact_layout,
        )
        self.assertNotRegex(
            compact_layout.split("#[cfg(test)]\nmod tests", 1)[0],
            r"compact_line_height\([^\n]+,\s*(?:10\.0|12\.0|14\.0)\)",
        )
        self.assertIn("COMPACT_CONTENT_TITLE_LINE_HEIGHT", compact_layout)
        self.assertIn("COMPACT_CONTENT_PATH_LINE_HEIGHT", compact_layout)
        self.assertNotIn("let title_height = 12.0_f32.min(height);", compact_layout)
        self.assertNotIn("let path_height = 10.0_f32.min(height);", compact_layout)

    def test_assets_activity_main_columns_use_bounded_weighted_slots(self):
        nodes = load_document("assets_activity.zui")["nodes"]
        expected_widths = {
            "tree_panel": {
                "min": 188.0,
                "preferred": 220.0,
                "max": 250.0,
                "weight": 1.0,
                "stretch": "Stretch",
            },
            "content_panel": {
                "min": 320.0,
                "preferred": 640.0,
                "weight": 4.0,
                "stretch": "Stretch",
            },
        }

        children = nodes["main_panel"]["children"]
        actual_widths = {
            child["node"]: child["slot"]["layout"]["width"]
            for child in children
        }
        self.assertEqual(expected_widths, actual_widths)
        for node_name in expected_widths:
            self.assertEqual(
                "Stretch", nodes[node_name]["layout"]["width"]["stretch"]
            )

    def test_assets_activity_compact_projection_uses_the_viewport_breakpoint(self):
        responsive_layout = (
            REPO_ROOT
            / "zircon_editor/src/ui/layouts/views/assets_activity/responsive_layout.rs"
        ).read_text(encoding="utf-8")

        self.assertIn(
            "root.width > density.breakpoint_narrow_width", responsive_layout
        )
        self.assertNotIn(
            "root.width > density.compact_left_drawer_max_width", responsive_layout
        )

    def test_assets_activity_fixed_text_slots_preserve_authored_line_heights(self):
        nodes = load_document("assets_activity.zui")["nodes"]
        font_sizes = {
            "$editor.typography.caption.size": 8.0 * 96.0 / 72.0,
            "$editor.typography.overlay.size": 9.0 * 96.0 / 72.0,
            "$editor.typography.body.size": 10.0 * 96.0 / 72.0,
            "$editor.typography.title.size": 14.0 * 96.0 / 72.0,
        }
        violations = []

        for node_name, node in nodes.items():
            if node.get("component") != "Label":
                continue
            height = node.get("layout", {}).get("height", {})
            if height.get("stretch") != "Fixed":
                continue

            props = node.get("props", {})
            font_size = props.get("font_size")
            if isinstance(font_size, str):
                requested = font_sizes.get(font_size)
            elif isinstance(font_size, (int, float)):
                requested = float(font_size)
            elif props.get("text_tone", "") in {"muted", "subtle", "secondary"}:
                requested = font_sizes["$editor.typography.caption.size"]
            else:
                requested = font_sizes["$editor.typography.body.size"]
            if requested is None:
                continue

            required = requested * 1.2
            actual = float(height["preferred"])
            if actual + 1.0e-6 < required:
                violations.append(f"{node_name}:{actual:g}<{required:g}")

        self.assertEqual([], violations)

        responsive_layout = (
            REPO_ROOT
            / "zircon_editor/src/ui/layouts/views/assets_activity/responsive_layout.rs"
        ).read_text(encoding="utf-8")
        preview_layout = responsive_layout.split("fn layout_preview", 1)[1].split(
            "fn measured_button_width", 1
        )[0]
        self.assertIn("PREVIEW_OVERLAY_LINE_HEIGHT", preview_layout)
        self.assertIn("PREVIEW_CAPTION_LINE_HEIGHT", preview_layout)
        self.assertNotRegex(preview_layout, r"set_node_frame\([^\n]+14\.0")

    def test_inspector_is_one_continuous_docked_property_surface(self):
        document = load_document("inspector.zui")
        nodes = document["nodes"]

        self.assertEqual(
            [
                {"node": "header_panel"},
                {"node": "header_name_gap"},
                {"node": "body_overlay"},
            ],
            nodes["inspector_root"]["children"],
        )
        for obsolete_container in (
            "top_space",
            "bottom_space",
            "left_space",
            "right_space",
            "content_row",
            "content_panel",
        ):
            self.assertNotIn(obsolete_container, nodes)

        header_props = nodes["header_panel"]["props"]
        self.assertEqual("Inspector", header_props["text"])
        self.assertEqual("transparent", header_props["surface_variant"])
        self.assertEqual(0.0, header_props["radius"])
        self.assertEqual(0.0, header_props["border_width"])
        self.assertEqual(
            "$editor.density.gap.medium", header_props["layout_padding_left"]
        )
        self.assertEqual(
            "$editor.density.gap.medium", header_props["layout_padding_right"]
        )

        empty_props = nodes["bottom_fill"]["props"]
        self.assertEqual(0.0, empty_props["radius"])
        self.assertEqual(0.0, empty_props["border_width"])

        production = INSPECTOR_PROJECTION_SOURCE.read_text(
            encoding="utf-8"
        ).split("#[cfg(test)]", 1)[0]
        self.assertNotIn("mark_header", production)
        self.assertNotIn('"InspectorHeaderPanel"', production)

    def test_hierarchy_is_one_continuous_docked_tool_surface(self):
        document = load_document("hierarchy.zui")
        nodes = document["nodes"]

        self.assertEqual(
            [
                {"node": "header_panel"},
                {"node": "header_list_gap"},
                {"node": "search_inset_row"},
                {"node": "search_list_gap"},
                {"node": "list_panel"},
            ],
            nodes["hierarchy_root"]["children"],
        )
        for obsolete_spacer in (
            "top_space",
            "bottom_space",
            "left_space",
            "right_space",
            "content_row",
            "content_panel",
        ):
            self.assertNotIn(obsolete_spacer, nodes)

        header_props = nodes["header_panel"]["props"]
        self.assertEqual("transparent", header_props["surface_variant"])
        self.assertEqual(0.0, header_props["radius"])
        self.assertEqual(0.0, header_props["border_width"])

        list_props = nodes["list_panel"]["props"]
        self.assertEqual("transparent", list_props["surface_variant"])
        self.assertEqual(0.0, list_props["radius"])
        self.assertEqual(0.0, list_props["border_width"])
        self.assertEqual("HierarchyListPanel", nodes["list_panel"]["control_id"])

        self.assertEqual(
            [
                {"node": "search_left_inset"},
                {"node": "search_row"},
                {"node": "search_right_inset"},
            ],
            nodes["search_inset_row"]["children"],
        )
        self.assertEqual(
            "$editor.density.gap.medium",
            nodes["search_left_inset"]["layout"]["width"]["preferred"],
        )
        self.assertEqual(
            "$editor.density.gap.medium",
            nodes["search_right_inset"]["layout"]["width"]["preferred"],
        )

        self.assertEqual("Hierarchy", header_props["text"])
        projection_source = (
            REPO_ROOT / "zircon_editor/src/ui/layouts/views/hierarchy.rs"
        ).read_text(encoding="utf-8")
        production = projection_source.split("#[cfg(test)]", 1)[0]
        self.assertNotIn("entries.is_selected", production)
        self.assertNotIn("if has_selection", production)
        self.assertNotIn(
            'text_overrides.insert(HIERARCHY_HEADER_PANEL.to_string()',
            production,
        )
        self.assertIn('.surface_variant("transparent")', production)

    def test_console_status_does_not_select_or_reframe_the_docked_surface(self):
        document = load_document("console.zui")
        nodes = document["nodes"]

        self.assertEqual(
            [
                {"node": "console_header"},
                {"node": "console_header_body_gap"},
                {"node": "console_body"},
            ],
            nodes["console_root"]["children"],
        )
        for obsolete_container in (
            "top_space",
            "bottom_space",
            "left_space",
            "right_space",
            "content_row",
            "console_content_stack",
        ):
            self.assertNotIn(obsolete_container, nodes)

        header_props = nodes["console_header"]["props"]
        self.assertEqual("Console", header_props["text"])
        self.assertEqual("transparent", header_props["surface_variant"])
        self.assertEqual(0.0, header_props["radius"])
        self.assertEqual(0.0, header_props["border_width"])

        body_props = nodes["console_body"]["props"]
        self.assertEqual("transparent", body_props["surface_variant"])
        self.assertEqual(0.0, body_props["radius"])
        self.assertEqual(0.0, body_props["border_width"])
        self.assertEqual("ConsoleBodySection", nodes["console_body"]["control_id"])
        self.assertEqual("ConsoleTextPanel", nodes["text_panel"]["control_id"])

        projection_source = (
            REPO_ROOT / "zircon_editor/src/ui/layouts/views/console.rs"
        ).read_text(encoding="utf-8")
        production = projection_source.split("#[cfg(test)]", 1)[0]
        self.assertNotIn("mark_console_node", production)
        self.assertNotIn(
            'text_overrides.insert("ConsoleHeader".to_string()',
            production,
        )
        self.assertNotIn("if has_status { \"panel\" }", production)
        self.assertIn(
            'ViewTemplateNodePatch::visual_state(false, false, "transparent"',
            production,
        )


if __name__ == "__main__":
    unittest.main()
