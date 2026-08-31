import unittest
import tomllib
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
WELCOME_PROJECTION = REPO_ROOT / (
    "zircon_editor/src/ui/workbench/startup/"
    "editor_startup_session_document_welcome_pane_snapshot.rs"
)
WELCOME_LAYOUT = REPO_ROOT / "zircon_editor/assets/ui/editor/welcome.zui"
EDITOR_TOKENS = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/theme/editor_tokens.zui"
)
WORKBENCH_BUTTON = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/"
    "workbench_button.zui"
)
WORKBENCH_FIELD = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/"
    "workbench_field.zui"
)
WELCOME_BOOTSTRAP_TEST = REPO_ROOT / (
    "zircon_editor/src/tests/ui/welcome/bootstrap_assets.rs"
)
WELCOME_FRAME_RESOLVE = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/"
    "paint_workbench_renderer/welcome/main_column/frames/resolve.rs"
)
WELCOME_TOP_SEQUENCE = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/"
    "paint_workbench_renderer/welcome/main_column/frames/sequence/top.rs"
)
WELCOME_FORM_SEQUENCE = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/"
    "paint_workbench_renderer/welcome/main_column/frames/sequence/form.rs"
)
WELCOME_MAIN_COLUMN = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/"
    "paint_workbench_renderer/welcome/main_column.rs"
)
WELCOME_NATIVE_PAINT = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/"
    "paint_workbench_renderer/welcome.rs"
)
WELCOME_RECENT_HEADER = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/"
    "paint_workbench_renderer/welcome/recent_projects/header.rs"
)
WELCOME_RECENT_GEOMETRY = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/welcome_recent_geometry.rs"
)
WELCOME_RECENT_POINTER_LAYOUT = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/welcome_recent_pointer/"
    "welcome_recent_pointer_layout.rs"
)
WELCOME_RECENT_POINTER_HELPER = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/welcome_recent_pointer/helper.rs"
)
WELCOME_RECENT_APP_LAYOUT = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/app/pointer_layout/welcome_recent.rs"
)
WELCOME_PROFILE_PANE_FRAMES = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/"
    "geometry/pane_frames/pane.rs"
)


class EditorZuiWelcomeVisualDensityContractTests(unittest.TestCase):
    def test_recent_project_projection_uses_a_scannable_sidebar_title(self):
        source = WELCOME_PROJECTION.read_text(encoding="utf-8")

        self.assertIn('subtitle: "Recent projects".to_string()', source)
        self.assertNotIn("Continue from a recent project", source)
        self.assertNotIn("scaffold a renderable empty project", source)

    def test_recent_project_header_is_a_single_compact_panel_header(self):
        with WELCOME_LAYOUT.open("rb") as source:
            nodes = tomllib.load(source)["nodes"]
        header_height = nodes["recent_header_panel"]["layout"]["height"]
        for bound in ("min", "preferred", "max"):
            self.assertEqual(
                "$editor.chrome.panel_header.height",
                header_height[bound],
            )

        source = WELCOME_RECENT_HEADER.read_text(encoding="utf-8")
        self.assertIn("const RECENT_HEADER_HEIGHT: f32 = 30.0;", source)
        self.assertIn('"Recent Projects"', source)
        self.assertNotIn("Pinned startup workspace", source)
        self.assertIn("assert_eq!(commands.len(), 1);", source)

    def test_wide_welcome_content_is_centered_between_equal_safe_margins(self):
        with WELCOME_LAYOUT.open("rb") as source:
            nodes = tomllib.load(source)["nodes"]

        left_width = nodes["left_space"]["layout"]["width"]
        right_width = nodes["right_space"]["layout"]["width"]
        self.assertEqual(left_width, right_width)
        self.assertEqual("Stretch", left_width["stretch"])
        self.assertNotIn("max", left_width)

        outer_width = nodes["outer_panel"]["layout"]["width"]
        self.assertEqual(0.0, outer_width["min"])
        self.assertEqual(1000.0, outer_width["preferred"])
        self.assertEqual(1000.0, outer_width["max"])
        self.assertEqual("Stretch", outer_width["stretch"])

        self.assertEqual(
            ["left_space", "outer_panel", "right_space"],
            [child["node"] for child in nodes["content_row"]["children"]],
        )

    def test_product_welcome_uses_shared_workbench_control_recipes(self):
        with WELCOME_LAYOUT.open("rb") as source:
            document = tomllib.load(source)

        imports = document["imports"]
        self.assertEqual(
            ["res://ui/theme/editor_workbench_strict.zui"], imports["styles"]
        )
        self.assertIn(
            "res://ui/editor/components/workbench/primitives/inputs/"
            "workbench_button.zui#WorkbenchButton",
            imports["widgets"],
        )
        self.assertIn(
            "res://ui/editor/components/workbench/primitives/inputs/"
            "workbench_field.zui#WorkbenchField",
            imports["widgets"],
        )
        self.assertNotIn("stylesheets", document)

        nodes = document["nodes"]
        button_nodes = (
            "startup_chooser_button_workbench",
            "startup_chooser_button_demo",
            "startup_chooser_button_asset",
            "startup_chooser_button_ui_layout",
            "open_existing_button",
            "create_project_button",
        )
        for node_name in button_nodes:
            self.assertEqual("WorkbenchButton", nodes[node_name]["component"])
        for node_name in ("project_name_field", "location_field"):
            self.assertEqual("WorkbenchField", nodes[node_name]["component"])

        self.assertEqual("Open", nodes["open_existing_button"]["props"]["text"])
        with WORKBENCH_BUTTON.open("rb") as source:
            button_root = tomllib.load(source)["nodes"]["root"]
        with WORKBENCH_FIELD.open("rb") as source:
            field_root = tomllib.load(source)["nodes"]["root"]
        for root in (button_root, field_root):
            self.assertEqual(
                "$editor.control.radius.control",
                root["props"]["corner_radius"],
            )
        self.assertEqual(
            "$editor.text.disabled",
            button_root["props"]["disabled_foreground_color"],
        )

        authored_classes = {
            class_name
            for node in nodes.values()
            for class_name in node.get("classes", [])
        }
        self.assertFalse(
            any(
                class_name.startswith(("showcase-", "material-"))
                for class_name in authored_classes
            )
        )

    def test_welcome_density_prioritizes_the_project_form_at_compact_widths(self):
        with WELCOME_LAYOUT.open("rb") as source:
            nodes = tomllib.load(source)["nodes"]

        self.assertEqual(
            {
                "min": 136.0,
                "preferred": 184.0,
                "max": 240.0,
                "weight": 1.0,
                "stretch": "Stretch",
            },
            nodes["recent_panel"]["layout"]["width"],
        )
        self.assertEqual(
            {
                "min": 280.0,
                "preferred": 560.0,
                "max": 760.0,
                "weight": 4.0,
                "stretch": "Stretch",
            },
            nodes["main_panel"]["layout"]["width"],
        )
        for field_name in ("project_name_field", "location_field"):
            field = nodes[field_name]
            self.assertEqual(
                "$editor.control.height.default",
                field["props"]["height"],
                field_name,
            )
            self.assertEqual(
                {
                    "min": "$editor.control.height.default",
                    "preferred": "$editor.control.height.default",
                    "max": "$editor.control.height.default",
                    "stretch": "Fixed",
                },
                field["layout"]["height"],
                field_name,
            )

    def test_rust_bootstrap_tracks_shared_control_radius_authority(self):
        source = WELCOME_BOOTSTRAP_TEST.read_text(encoding="utf-8")

        self.assertNotIn("Some(4.0)", source)
        self.assertNotIn("starship_control_density", source)
        self.assertIn("WORKBENCH_BUTTON_TOML", source)
        self.assertIn("WORKBENCH_FIELD_TOML", source)
        self.assertIn('Some("$editor.control.radius.control")', source)
        self.assertIn(
            'assert_eq!(field.component, "WorkbenchField")',
            source,
        )
        self.assertIn(
            'assert_eq!(button.component, "WorkbenchButton")',
            source,
        )

    def test_componentized_actions_own_their_fixed_widths(self):
        with WELCOME_LAYOUT.open("rb") as source:
            nodes = tomllib.load(source)["nodes"]

        for node_name, width in (
            ("open_existing_button", 80.0),
            ("create_project_button", 118.0),
        ):
            self.assertEqual(
                {
                    "min": width,
                    "preferred": width,
                    "max": width,
                    "stretch": "Fixed",
                },
                nodes[node_name]["layout"]["width"],
            )

        for child in nodes["actions_row"]["children"]:
            if child["node"] in {"open_existing_button", "create_project_button"}:
                self.assertNotIn("slot", child)

    def test_short_height_keeps_actions_reachable_through_the_main_scroll_view(self):
        with WELCOME_LAYOUT.open("rb") as source:
            nodes = tomllib.load(source)["nodes"]

        for node_name in (
            "main_panel_top_space",
            "status_preview_gap",
            "preview_panel",
            "main_panel_bottom_space",
        ):
            height = nodes[node_name]["layout"]["height"]
            self.assertEqual(0.0, height["min"], node_name)
            self.assertNotEqual(0.0, height["preferred"], node_name)

        main_panel = nodes["main_panel"]
        self.assertEqual("ScrollableBox", main_panel["component"])
        self.assertEqual({"input_hoverable": True}, main_panel["props"])
        self.assertEqual(
            {
                "kind": "ScrollableBox",
                "axis": "Vertical",
                "gap": 0.0,
                "scrollbar_visibility": "Auto",
            },
            main_panel["layout"]["container"],
        )
        self.assertEqual("Receive", main_panel["layout"]["input_policy"])
        self.assertTrue(main_panel["layout"]["clip"])
        self.assertNotIn("preview_bottom_fill", nodes)
        self.assertNotIn(
            "preview_bottom_fill",
            [child["node"] for child in main_panel["children"]],
        )
        self.assertEqual(
            ["open_existing_button", "create_project_button"],
            [
                child["node"]
                for child in nodes["actions_row"]["children"]
                if child["node"] != "actions_row_fill"
            ],
        )
        self.assertEqual(
            4,
            len(nodes["startup_chooser_row"]["children"]),
        )

    def test_ultra_width_prioritizes_the_project_task_over_recent_history(self):
        with WELCOME_LAYOUT.open("rb") as source:
            nodes = tomllib.load(source)["nodes"]
        with EDITOR_TOKENS.open("rb") as source:
            tokens = tomllib.load(source)

        recent = nodes["recent_panel"]
        main = nodes["main_panel"]
        self.assertEqual({"responsive_min_tier": "narrow"}, recent["props"])

        logical_width = 640.0 / 1.5
        panel_padding = tokens["density"]["panel_padding"]
        recent_min = recent["layout"]["width"]["min"]
        main_min = main["layout"]["width"]["min"]
        self.assertGreater(recent_min + main_min + panel_padding * 2.0, logical_width)
        self.assertLessEqual(main_min + panel_padding * 2.0, logical_width)
        self.assertLessEqual(logical_width, tokens["density"]["breakpoint_ultra_width"])

        paint_source = WELCOME_NATIVE_PAINT.read_text(encoding="utf-8")
        recent_geometry_source = WELCOME_RECENT_GEOMETRY.read_text(encoding="utf-8")
        self.assertIn("resolve_welcome_panel_frames(layout, body)", paint_source)
        self.assertIn("if let Some(recent_panel)", paint_source)
        self.assertIn(
            "None if layout.has_nodes => UiFrame::new(0.0, 0.0, 0.0, 0.0)",
            recent_geometry_source,
        )

    def test_welcome_prioritizes_the_project_task_over_duplicate_presentation(self):
        with WELCOME_LAYOUT.open("rb") as source:
            nodes = tomllib.load(source)["nodes"]

        for node_name in (
            "hero_panel",
            "hero_status_gap",
            "status_panel",
            "actions_optional_gap",
        ):
            self.assertEqual(
                {
                    "min": 0.0,
                    "preferred": 0.0,
                    "max": 0.0,
                    "stretch": "Fixed",
                },
                nodes[node_name]["layout"]["height"],
                node_name,
            )

        self.assertEqual(52.0, nodes["preview_panel"]["layout"]["height"]["preferred"])
        main_children = [child["node"] for child in nodes["main_panel"]["children"]]
        self.assertLess(main_children.index("preview_panel"), main_children.index("validation_panel"))
        self.assertLess(main_children.index("validation_panel"), main_children.index("actions_row"))
        self.assertEqual(
            "$editor.density.gap.large",
            nodes["startup_chooser_gap"]["layout"]["height"]["preferred"],
        )
        self.assertLess(main_children.index("actions_row"), main_children.index("startup_chooser_gap"))
        self.assertLess(
            main_children.index("startup_chooser_gap"),
            main_children.index("startup_chooser_row"),
        )
        self.assertLess(main_children.index("startup_chooser_row"), main_children.index("main_panel_bottom_space"))

    def test_ultra_startup_chooser_uses_compact_labels_with_full_tooltips(self):
        with WELCOME_LAYOUT.open("rb") as source:
            nodes = tomllib.load(source)["nodes"]

        expected = {
            "startup_chooser_button_workbench": (
                "Default",
                "Open default Workbench",
                "workbench.welcome.open_startup_workbench",
            ),
            "startup_chooser_button_demo": (
                "Showcase",
                "Open UI Component Showcase",
                "workbench.welcome.open_startup_demo",
            ),
            "startup_chooser_button_asset": (
                "Assets",
                "Open Asset Window",
                "workbench.welcome.open_startup_asset_window",
            ),
            "startup_chooser_button_ui_layout": (
                "UI Layout",
                "Open UI Layout Editor",
                "workbench.welcome.open_startup_ui_layout_editor",
            ),
        }
        for node_name, (label, tooltip, route) in expected.items():
            node = nodes[node_name]
            self.assertEqual(label, node["props"]["text"], node_name)
            self.assertLessEqual(len(label), 10, node_name)
            self.assertEqual(tooltip, node["props"]["tooltip"], node_name)
            self.assertEqual(route, node["events"][0]["route"], node_name)

    def test_collapsed_asset_frames_are_not_resurrected_by_legacy_fallbacks(self):
        resolve_source = WELCOME_FRAME_RESOLVE.read_text(encoding="utf-8")
        top_source = WELCOME_TOP_SEQUENCE.read_text(encoding="utf-8")
        form_source = WELCOME_FORM_SEQUENCE.read_text(encoding="utf-8")
        paint_source = WELCOME_MAIN_COLUMN.read_text(encoding="utf-8")

        self.assertIn("asset_layout_is_authoritative: bool", resolve_source)
        self.assertIn(
            "None if asset_layout_is_authoritative => FrameRect::default()",
            resolve_source,
        )
        self.assertNotIn(".unwrap_or(fallback)", resolve_source)

        self.assertEqual(3, top_source.count("layout.has_nodes,"))
        self.assertEqual(4, form_source.count("layout.has_nodes,"))
        for frame_name in ("hero", "status", "header", "preview", "validation"):
            self.assertIn(
                f"if is_visible_frame(&frames.{frame_name})",
                paint_source,
                frame_name,
            )

    def test_recent_pointer_and_profile_use_the_projected_list_viewport(self):
        geometry_source = WELCOME_RECENT_GEOMETRY.read_text(encoding="utf-8")
        layout_source = WELCOME_RECENT_POINTER_LAYOUT.read_text(encoding="utf-8")
        helper_source = WELCOME_RECENT_POINTER_HELPER.read_text(encoding="utf-8")
        app_source = WELCOME_RECENT_APP_LAYOUT.read_text(encoding="utf-8")
        profile_source = WELCOME_PROFILE_PANE_FRAMES.read_text(encoding="utf-8")

        self.assertIn("welcome_recent_viewport_for_layout", geometry_source)
        self.assertIn("pub viewport: UiFrame", layout_source)
        self.assertNotIn("pub pane_size: UiSize", layout_source)
        self.assertIn("layout.viewport", helper_source)
        self.assertNotIn("welcome_recent_viewport_with_metrics", helper_source)
        self.assertIn("get_welcome_pane", app_source)
        self.assertIn("welcome_recent_viewport_for_layout", app_source)
        self.assertIn("welcome_recent_viewport_for_layout", profile_source)


if __name__ == "__main__":
    unittest.main()
