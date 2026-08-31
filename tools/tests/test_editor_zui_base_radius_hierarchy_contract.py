import re
import tomllib
import unittest
import xml.etree.ElementTree as ElementTree
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
ASSET_ROOT = REPO_ROOT / "zircon_editor/assets"
RUNTIME_ASSET_ROOT = REPO_ROOT / "zircon_runtime/assets"
BASE_THEME = REPO_ROOT / "zircon_editor/assets/ui/theme/editor_base.zui"
EDITOR_TOKENS = REPO_ROOT / "zircon_editor/assets/ui/editor/theme/editor_tokens.zui"
WORKBENCH_THEME = REPO_ROOT / "zircon_editor/assets/ui/theme/editor_workbench_strict.zui"
WORKBENCH_SPATIAL_THEME = REPO_ROOT / (
    "zircon_editor/assets/ui/theme/editor_workbench_spatial.zui"
)
MATERIAL_THEME = REPO_ROOT / "zircon_editor/assets/ui/theme/editor_material.zui"
MATERIAL_COMPONENTS = ASSET_ROOT / "ui/editor/material_components"
SHOWCASE_COMPONENTS = ASSET_ROOT / "ui/editor/components/showcase"
SCAN_ONLY_UI_ASSET_EDITOR = ASSET_ROOT / (
    "ui/editor/components/workbench/modules/extensions/"
    "ui/workbench_extension_ui_asset_editor_workspace.zui"
)
PRODUCT_BINDING_FIXTURE = ASSET_ROOT / "ui/editor/product_binding_fixture.zui"
NON_PRODUCT_SPECIMENS = {
    ASSET_ROOT / "ui/editor/component_showcase.zui",
    ASSET_ROOT / "ui/editor/fyrox_panel_demo_window.zui",
    ASSET_ROOT / "ui/editor/layout_demo_window.zui",
    ASSET_ROOT / "ui/editor/material_demo_window.zui",
}
EDITOR_TOKEN_REFERENCE = "res://ui/editor/theme/editor_tokens.zui"
WORKBENCH_THEME_REFERENCES = {
    "res://ui/theme/editor_workbench_strict.zui",
    "res://ui/theme/editor_workbench_spatial.zui",
}
THEME_ROOT = ASSET_ROOT / "ui/theme"
WORKBENCH_WINDOW = ASSET_ROOT / "ui/editor/windows/workbench_window.zui"
WELCOME = ASSET_ROOT / "ui/editor/welcome.zui"
VIEWPORT_PANEL = ASSET_ROOT / (
    "ui/editor/components/workbench/shell/workbench_viewport_panel.zui"
)
WORKBENCH_PANEL_HEADER = ASSET_ROOT / (
    "ui/editor/components/workbench/composites/chrome/workbench_panel_header.zui"
)
FLOATING_COMMAND_PALETTE = ASSET_ROOT / (
    "ui/editor/components/workbench/floating/workbench_command_palette.zui"
)
WORKBENCH_PRIMITIVES = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/primitives"
)
HEX_COLOR = re.compile(r"#[0-9A-Fa-f]{6}(?:[0-9A-Fa-f]{2})?")
NATIVE_INTERACTIVE_COMPONENTS = {
    "Button",
    "Checkbox",
    "Dropdown",
    "IconButton",
    "InputField",
    "ListRow",
    "NumberField",
    "ProgressBar",
    "Radio",
    "RangeSlider",
    "SearchInput",
    "Select",
    "Slider",
    "Switch",
    "Tab",
    "TabStrip",
    "TableRow",
    "TextField",
    "Toggle",
    "Tooltip",
}


def load_document(path: Path) -> dict:
    with path.open("rb") as source:
        return tomllib.load(source)


def radius_for_selector(document: dict, selector: str) -> str:
    rules = [
        rule
        for stylesheet in document["stylesheets"]
        for rule in stylesheet.get("rules", [])
    ]
    matching = [rule for rule in rules if rule["selector"] == selector]
    if len(matching) != 1:
        raise AssertionError(f"expected one rule for {selector}, found {len(matching)}")
    return matching[0]["set"]["self"]["border"]["radius"]


def workbench_radius_for_selector(document: dict, selector: str) -> str:
    rules = [
        rule
        for stylesheet in document["stylesheets"]
        for rule in stylesheet.get("rules", [])
    ]
    matching = [rule for rule in rules if rule["selector"] == selector]
    if len(matching) != 1:
        raise AssertionError(f"expected one rule for {selector}, found {len(matching)}")
    return matching[0]["set"]["self"]["radius"]


def primitive_corner_radius(relative_path: str) -> str:
    document = load_document(WORKBENCH_PRIMITIVES / relative_path)
    return document["nodes"]["root"]["props"]["corner_radius"]


def imported_zui_path(reference: str) -> Path | None:
    asset_path = reference.split("#", 1)[0]
    if not asset_path.startswith("res://") or not asset_path.endswith(".zui"):
        return None
    return ASSET_ROOT / asset_path.removeprefix("res://")


def reachable_workbench_documents() -> dict[Path, dict]:
    documents = {}
    pending = [WORKBENCH_WINDOW]
    while pending:
        path = pending.pop()
        if path in documents:
            continue
        document = load_document(path)
        documents[path] = document
        imports = document.get("imports", {})
        for category in ("widgets", "styles"):
            for reference in imports.get(category, []):
                imported_path = imported_zui_path(reference)
                if imported_path is not None and imported_path.exists():
                    pending.append(imported_path)
    return documents


def low_nonzero_radii(value, owner: str):
    if isinstance(value, dict):
        for key, child in value.items():
            child_owner = f"{owner}.{key}"
            if (
                key in {"radius", "corner_radius"}
                and isinstance(child, (int, float))
                and 0.0 < float(child) < 6.0
            ):
                yield child_owner, float(child)
            yield from low_nonzero_radii(child, child_owner)
    elif isinstance(value, list):
        for index, child in enumerate(value):
            yield from low_nonzero_radii(child, f"{owner}[{index}]")


def numeric_typography(value, owner: str):
    if isinstance(value, dict):
        for key, child in value.items():
            child_owner = f"{owner}.{key}"
            if key in {"font_size", "font_weight"} and isinstance(
                child, (int, float)
            ):
                yield child_owner, float(child)
            yield from numeric_typography(child, child_owner)
    elif isinstance(value, list):
        for index, child in enumerate(value):
            yield from numeric_typography(child, f"{owner}[{index}]")


def positive_numeric_spacing(value, owner: str, in_padding: bool = False):
    if isinstance(value, dict):
        for key, child in value.items():
            child_owner = f"{owner}.{key}"
            if (
                key in {"gap", "column_gap", "row_gap"}
                and isinstance(child, (int, float))
                and float(child) > 0.0
            ):
                yield child_owner, float(child)
            if in_padding and isinstance(child, (int, float)) and float(child) > 0.0:
                yield child_owner, float(child)
            yield from positive_numeric_spacing(
                child,
                child_owner,
                in_padding or key in {"padding", "layout_padding"},
            )
    elif isinstance(value, list):
        for index, child in enumerate(value):
            yield from positive_numeric_spacing(
                child,
                f"{owner}[{index}]",
                in_padding,
            )


def string_values(value):
    if isinstance(value, str):
        yield value
    elif isinstance(value, dict):
        for child in value.values():
            yield from string_values(child)
    elif isinstance(value, list):
        for child in value:
            yield from string_values(child)


def packaged_visual_asset_path(reference: str) -> Path | None:
    for candidate in (
        ASSET_ROOT / reference,
        ASSET_ROOT / "icons" / reference,
        RUNTIME_ASSET_ROOT / reference,
        RUNTIME_ASSET_ROOT / "icons" / reference,
    ):
        if candidate.is_file():
            return candidate
    return None


class EditorZuiBaseRadiusHierarchyContractTests(unittest.TestCase):
    def test_zero_radius_is_limited_to_flush_chrome_and_viewport_artwork(self):
        expected_theme_selectors = {
            WORKBENCH_THEME: {
                ".workbench-topbar",
                ".workbench-strip",
                ".workbench-rail",
                ".workbench-component-property-row",
                ".workbench-component-property-row:hovered",
                ".workbench-panel",
                ".workbench-property-section",
                ".workbench-component-drawer",
                ".workbench-status",
                ".workbench-overlay-region",
            },
            WORKBENCH_SPATIAL_THEME: {
                ".workbench-viewport-panel",
                ".workbench-viewport-backdrop",
                ".workbench-viewport-ceiling",
                ".workbench-viewport-wall",
                ".workbench-viewport-side",
                ".workbench-viewport-floor",
                ".workbench-viewport-grid-line",
                ".workbench-viewport-grid-major",
                ".workbench-viewport-floor-seam",
            },
        }
        expected_node_owners = {
            VIEWPORT_PANEL: {
                "viewport_backdrop",
                "viewport_ceiling",
                "viewport_back_wall",
                "viewport_side_left",
                "viewport_side_right",
                "viewport_floor",
                *(f"viewport_floor_grid_h{index}" for index in range(6)),
                *(f"viewport_floor_grid_v{index}" for index in range(7)),
                "viewport_floor_seam_right",
            },
        }

        documents = reachable_workbench_documents()
        actual_theme_selectors = {}
        actual_node_owners = {}
        for path, document in documents.items():
            selectors = {
                rule["selector"]
                for stylesheet in document.get("stylesheets", [])
                for rule in stylesheet.get("rules", [])
                if rule.get("set", {}).get("self", {}).get("radius") == 0
            }
            if selectors:
                actual_theme_selectors[path] = selectors
            nodes = {
                node_name
                for node_name, node in document.get("nodes", {}).items()
                if node.get("props", {}).get("corner_radius") == 0
            }
            if nodes:
                actual_node_owners[path] = nodes

        self.assertEqual(expected_theme_selectors, actual_theme_selectors)
        self.assertEqual(expected_node_owners, actual_node_owners)

    def test_workbench_focus_visible_cue_covers_all_keyboard_controls(self):
        document = load_document(WORKBENCH_THEME)
        rules = {
            rule["selector"]: rule["set"]["self"]
            for stylesheet in document["stylesheets"]
            for rule in stylesheet.get("rules", [])
        }
        selectors = [
            ".workbench-control-button:focus-visible",
            ".workbench-icon-button:focus-visible",
            ".workbench-rail-button:focus-visible",
            ".workbench-tab:focus-visible",
            ".workbench-segmented-control:focus-visible",
            ".workbench-number-field:focus-visible",
            ".workbench-check:focus-visible",
            ".workbench-radio:focus-visible",
            ".workbench-toggle:focus-visible",
            ".workbench-slider:focus-visible",
            ".workbench-tree-item:focus-visible",
            ".workbench-list-row:focus-visible",
            ".workbench-table-row:focus-visible",
        ]
        for selector in selectors:
            self.assertEqual(
                {
                    "border_color": "$editor.focus.ring",
                    "border_width": "$editor.control.border_width",
                },
                rules.get(selector),
                f"{selector} must preserve the independent focus ring recipe",
            )

    def test_editor_radius_tokens_define_a_legible_four_tier_hierarchy(self):
        tokens = load_document(EDITOR_TOKENS)["controls"]

        self.assertEqual(
            [6.0, 8.0, 10.0, 12.0],
            [
                tokens["small_radius"],
                tokens["control_radius"],
                tokens["large_radius"],
                tokens["panel_radius"],
            ],
        )

    def test_ordinary_buttons_use_the_control_radius_tier(self):
        document = load_document(BASE_THEME)

        for selector in ("Button.primary", "Button.secondary"):
            self.assertEqual(
                "$editor.control.radius.control",
                radius_for_selector(document, selector),
                f"{selector} must not collapse ordinary controls into the compact radius tier",
            )

    def test_compact_surfaces_remain_smaller_than_ordinary_controls(self):
        document = load_document(BASE_THEME)
        tokens = load_document(EDITOR_TOKENS)["controls"]

        self.assertLess(tokens["small_radius"], tokens["control_radius"])
        for selector in (".inset", ".chrome-selected"):
            self.assertEqual(
                "$editor.control.radius.small",
                radius_for_selector(document, selector),
                f"{selector} must retain the compact radius tier",
            )

    def test_workbench_buttons_and_fields_use_the_control_radius_tier(self):
        theme = load_document(WORKBENCH_THEME)

        for selector in (
            ".workbench-control-button",
            ".workbench-field",
            ".workbench-dropdown",
        ):
            self.assertEqual(
                "$editor.control.radius.control",
                workbench_radius_for_selector(theme, selector),
            )

        for relative_path in (
            "inputs/workbench_button.zui",
            "inputs/workbench_dropdown.zui",
        ):
            self.assertEqual(
                "$editor.control.radius.control",
                primitive_corner_radius(relative_path),
            )

    def test_workbench_floating_overlays_use_the_panel_radius_tier(self):
        theme = load_document(WORKBENCH_THEME)
        for selector in (".workbench-popup-menu", ".workbench-toast"):
            self.assertEqual(
                "$editor.control.radius.panel",
                workbench_radius_for_selector(theme, selector),
            )

        for relative_path in (
            "feedback/workbench_popup_menu.zui",
            "feedback/workbench_context_menu.zui",
            "feedback/workbench_dropdown_popup.zui",
            "feedback/workbench_toast.zui",
        ):
            self.assertEqual(
                "$editor.control.radius.panel",
                primitive_corner_radius(relative_path),
            )

    def test_component_gallery_cards_use_the_panel_radius_tier(self):
        theme = load_document(WORKBENCH_THEME)
        self.assertEqual(
            "$editor.control.radius.panel",
            workbench_radius_for_selector(theme, ".workbench-component-sample-card"),
        )

    def test_notification_center_keeps_distinct_panel_and_row_radius_tiers(self):
        document = load_document(
            WORKBENCH_PRIMITIVES / "feedback/workbench_notification_center.zui"
        )
        props = document["nodes"]["root"]["props"]

        self.assertEqual("$editor.control.radius.panel", props["panel_radius"])
        self.assertEqual("$editor.control.radius.small", props["row_radius"])

    def test_command_palette_keeps_distinct_panel_and_search_radius_tiers(self):
        document = load_document(
            WORKBENCH_PRIMITIVES / "feedback/workbench_command_palette.zui"
        )
        props = document["nodes"]["root"]["props"]

        self.assertEqual("$editor.control.radius.panel", props["corner_radius"])
        self.assertEqual("$editor.control.radius.small", props["search_radius"])

        floating = load_document(FLOATING_COMMAND_PALETTE)
        self.assertEqual(
            "$editor.control.radius.panel",
            floating["nodes"]["palette"]["style"]["self"]["border"]["radius"],
        )

    def test_confirm_dialog_uses_the_panel_radius_tier(self):
        document = load_document(
            WORKBENCH_PRIMITIVES / "feedback/workbench_confirm_dialog.zui"
        )
        props = document["nodes"]["root"]["props"]

        self.assertEqual("$editor.control.radius.panel", props["corner_radius"])

    def test_tooltip_uses_the_panel_radius_tier(self):
        document = load_document(
            WORKBENCH_PRIMITIVES / "feedback/workbench_tooltip.zui"
        )
        props = document["nodes"]["root"]["props"]

        self.assertEqual("$editor.control.radius.panel", props["corner_radius"])

    def test_alert_uses_the_panel_radius_tier(self):
        document = load_document(
            WORKBENCH_PRIMITIVES / "feedback/workbench_alert.zui"
        )
        props = document["nodes"]["root"]["props"]

        self.assertEqual("$editor.control.radius.panel", props["corner_radius"])

    def test_product_workbench_rejects_legacy_tiny_radius_theme_overrides(self):
        documents = reachable_workbench_documents()
        relative_paths = {path.relative_to(ASSET_ROOT).as_posix() for path in documents}

        self.assertNotIn("ui/theme/editor_unreal_dark.zui", relative_paths)
        self.assertGreaterEqual(len(documents), 100)

        unexpected = []
        for path, document in documents.items():
            if path in {WORKBENCH_THEME, WORKBENCH_SPATIAL_THEME}:
                for stylesheet in document.get("stylesheets", []):
                    for rule in stylesheet.get("rules", []):
                        selector = rule.get("selector", "<missing-selector>")
                        for owner, radius in low_nonzero_radii(
                            rule.get("set", {}), f"selector {selector}"
                        ):
                            if selector.startswith(".workbench-viewport-"):
                                continue
                            unexpected.append(
                                (path.relative_to(ASSET_ROOT).as_posix(), owner, radius)
                            )
                continue
            for owner, radius in low_nonzero_radii(document, "document"):
                if path == VIEWPORT_PANEL:
                    continue
                unexpected.append((path.relative_to(ASSET_ROOT).as_posix(), owner, radius))

        self.assertEqual([], unexpected)

    def test_non_fixture_editor_products_use_tokens_for_positive_shape_and_typography(self):
        unexpected = []
        for path in sorted((ASSET_ROOT / "ui/editor").rglob("*.zui")):
            if (
                path == VIEWPORT_PANEL
                or path in NON_PRODUCT_SPECIMENS
                or path.is_relative_to(MATERIAL_COMPONENTS)
                or path.is_relative_to(SHOWCASE_COMPONENTS)
            ):
                continue
            document = load_document(path)
            relative_path = path.relative_to(ASSET_ROOT).as_posix()
            unexpected.extend(
                (relative_path, owner, radius)
                for owner, radius in low_nonzero_radii(document, "document")
            )
            unexpected.extend(
                (relative_path, owner, value)
                for owner, value in numeric_typography(document, "document")
            )

        self.assertEqual([], unexpected)

    def test_non_fixture_editor_products_use_tokens_for_positive_spacing(self):
        unexpected = []
        for path in sorted((ASSET_ROOT / "ui/editor").rglob("*.zui")):
            if (
                path == VIEWPORT_PANEL
                or path == SCAN_ONLY_UI_ASSET_EDITOR
                or path == PRODUCT_BINDING_FIXTURE
                or path in NON_PRODUCT_SPECIMENS
                or path.is_relative_to(MATERIAL_COMPONENTS)
                or path.is_relative_to(SHOWCASE_COMPONENTS)
            ):
                continue
            document = load_document(path)
            relative_path = path.relative_to(ASSET_ROOT).as_posix()
            unexpected.extend(
                (relative_path, owner, value)
                for owner, value in positive_numeric_spacing(document, "document")
            )

        self.assertEqual([], unexpected)

    def test_product_workbench_path_backed_svg_assets_are_packaged_and_scalable(self):
        references = sorted(
            {
                value
                for document in reachable_workbench_documents().values()
                for value in string_values(document)
                if value.lower().endswith(".svg")
            }
        )
        self.assertEqual(49, len(references))

        missing = []
        missing_view_box = []
        for reference in references:
            asset_path = packaged_visual_asset_path(reference)
            if asset_path is None:
                missing.append(reference)
                continue
            if ElementTree.parse(asset_path).getroot().get("viewBox") is None:
                missing_view_box.append(reference)

        self.assertEqual([], missing)
        self.assertEqual([], missing_view_box)

    def test_product_workbench_components_do_not_author_raw_interaction_colors(self):
        unexpected = []
        for path, document in reachable_workbench_documents().items():
            if (
                path == EDITOR_TOKENS
                or path == VIEWPORT_PANEL
                or path.is_relative_to(THEME_ROOT)
            ):
                continue
            for value in string_values(document):
                if HEX_COLOR.fullmatch(value):
                    unexpected.append(
                        (path.relative_to(ASSET_ROOT).as_posix(), value)
                    )

        self.assertEqual([], unexpected)

    def test_product_pages_consume_native_controls_through_workbench_families(self):
        documents = reachable_workbench_documents()
        documents[WELCOME] = load_document(WELCOME)
        unexpected = []

        for path, document in documents.items():
            if path.is_relative_to(WORKBENCH_PRIMITIVES):
                continue
            for node_name, node in document.get("nodes", {}).items():
                component = node.get("component")
                if component in NATIVE_INTERACTIVE_COMPONENTS:
                    unexpected.append(
                        (
                            path.relative_to(ASSET_ROOT).as_posix(),
                            node_name,
                            component,
                        )
                    )

        self.assertEqual([], unexpected)

    def test_workbench_strict_theme_delegates_spatial_raw_colors(self):
        strict_tokens = load_document(WORKBENCH_THEME)["tokens"]
        strict_raw_colors = {
            name: value
            for name, value in strict_tokens.items()
            if isinstance(value, str) and HEX_COLOR.fullmatch(value)
        }
        self.assertEqual({}, strict_raw_colors)

        spatial_theme = load_document(WORKBENCH_SPATIAL_THEME)
        self.assertEqual(
            ["res://ui/theme/editor_workbench_strict.zui"],
            spatial_theme["imports"]["styles"],
        )
        raw_colors = {
            name: value
            for name, value in spatial_theme["tokens"].items()
            if isinstance(value, str) and HEX_COLOR.fullmatch(value)
        }
        self.assertEqual(40, len(raw_colors))
        self.assertEqual(
            [],
            [
                name
                for name in raw_colors
                if not name.startswith(("workbench_viewport_", "workbench_axis_"))
            ],
        )

        selectors = {
            rule["selector"]
            for stylesheet in spatial_theme["stylesheets"]
            for rule in stylesheet.get("rules", [])
        }
        self.assertGreater(len(selectors), 40)
        self.assertEqual(
            [],
            [
                selector
                for selector in selectors
                if not selector.startswith(
                    (
                        ".workbench-transform-",
                        ".workbench-axis-",
                        ".workbench-mesh-",
                        ".workbench-viewport-",
                    )
                )
            ],
        )

    def test_product_workbench_does_not_import_material_showcase_theme(self):
        strict_imports = load_document(WORKBENCH_THEME)["imports"]["styles"]
        self.assertEqual(
            ["res://ui/editor/theme/editor_tokens.zui"], strict_imports
        )
        self.assertNotIn(MATERIAL_THEME, reachable_workbench_documents())

    def test_product_workbench_uses_only_the_converged_theme_entry_points(self):
        allowed = WORKBENCH_THEME_REFERENCES | {EDITOR_TOKEN_REFERENCE}
        unexpected = []
        for path, document in reachable_workbench_documents().items():
            for reference in document.get("imports", {}).get("styles", []):
                if reference not in allowed:
                    unexpected.append(
                        (path.relative_to(ASSET_ROOT).as_posix(), reference)
                    )

        self.assertEqual([], unexpected)

    def test_workbench_theme_consumers_do_not_reimport_editor_tokens(self):
        unexpected = []
        for path in sorted((ASSET_ROOT / "ui/editor").rglob("*.zui")):
            styles = load_document(path).get("imports", {}).get("styles", [])
            if (
                EDITOR_TOKEN_REFERENCE in styles
                and WORKBENCH_THEME_REFERENCES.intersection(styles)
            ):
                unexpected.append(path.relative_to(ASSET_ROOT).as_posix())

        self.assertEqual([], unexpected)

    def test_product_workbench_typography_uses_editor_tokens(self):
        unexpected = []
        for path, document in reachable_workbench_documents().items():
            for stylesheet in document.get("stylesheets", []):
                for rule in stylesheet.get("rules", []):
                    selector = rule.get("selector", "<missing-selector>")
                    style = rule.get("set", {}).get("self", {})
                    for property_name in ("font_size", "font_weight"):
                        value = style.get(property_name)
                        if value is not None and not (
                            isinstance(value, str)
                            and value.startswith("$editor.typography.")
                        ):
                            unexpected.append(
                                (
                                    path.relative_to(ASSET_ROOT).as_posix(),
                                    selector,
                                    property_name,
                                    value,
                                )
                            )

        self.assertEqual([], unexpected)


if __name__ == "__main__":
    unittest.main()
