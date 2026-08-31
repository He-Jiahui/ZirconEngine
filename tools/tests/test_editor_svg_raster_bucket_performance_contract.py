import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
TARGET = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "visual_assets/target.rs"
)
ASSET = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "visual_assets/asset.rs"
)
TEMPLATE = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "visual_assets/template.rs"
)
CONVERSION = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "render_command_conversion/image.rs"
)
BRUSH_CONVERSION = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "render_command_conversion/brush.rs"
)
TOOLTIP_ARROW = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_tooltip_glyphs/arrows.rs"
)
TOOLTIP_ARROW_ASSET = (
    REPO_ROOT
    / "zircon_editor/assets/icons/zircon_editor_shell/controls/tooltip-arrow.svg"
)
TOOLTIP_INFO = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_tooltip_glyphs/icons.rs"
)
TOOLTIP_INFO_ASSET = (
    REPO_ROOT / "zircon_editor/assets/icons/zircon_editor_shell/status/info.svg"
)
SECTION_TITLE_GLYPHS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_section_title_glyphs.rs"
)
SECTION_TITLE_SHAPES = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_section_title_glyphs/shapes.rs"
)
SECTION_TITLE_DISPATCH = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_section_title_glyphs/shapes/entry.rs"
)
SECTION_TITLE_VECTOR_ASSETS = tuple(
    REPO_ROOT / f"zircon_editor/assets/icons/{relative_path}"
    for relative_path in (
        "zircon_editor_shell/activity/cube.svg",
        "zircon_editor_shell/inspector/transform.svg",
        "zircon_editor_shell/inspector/mesh-renderer.svg",
    )
)
TREE_ROW_GLYPHS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_tree_row_glyphs.rs"
)
TREE_ROW_ACTION_GLYPHS = tuple(
    REPO_ROOT
    / (
        "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
        f"template_tree_row_glyphs/actions/{action}.rs"
    )
    for action in ("eye", "kebab", "lock")
)
TREE_ROW_DISCLOSURE = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_tree_row_glyphs/disclosure.rs"
)
TREE_ROW_OBJECT = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_tree_row_glyphs/object.rs"
)
TREE_ROW_OBJECT_DISPATCH = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_tree_row_glyphs/object/dispatch.rs"
)
TABLE_ROW_ACTIONS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_table_rows/actions.rs"
)
TABLE_ROW_ACTION_ENTRY = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_table_rows/actions/entry.rs"
)
CHIP_GLYPHS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_chip_glyphs.rs"
)
CHIP_CHEVRON = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_chip_glyphs/chevron.rs"
)
DROPDOWN_GLYPHS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_dropdown_glyphs.rs"
)
DROPDOWN_CHEVRON = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_dropdown_glyphs/chevron.rs"
)
FIELD_STEPPER = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_field_stepper.rs"
)
FIELD_STEPPER_COMMAND = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_field_stepper/command.rs"
)
FIELD_STEPPER_ASSET = (
    REPO_ROOT / "zircon_editor/assets/icons/zircon_editor_shell/controls/field-stepper.svg"
)
LIST_ROW_GLYPHS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_list_row_glyphs.rs"
)
CHECKBOX_TICK = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_selection_controls/checkbox/tick.rs"
)
SEARCH_FIELD_GLYPH = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_fields/search/glyph.rs"
)
INSPECTOR_ROW_GLYPHS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_inspector_row_glyphs.rs"
)
INSPECTOR_ROW_VECTOR_OWNERS = tuple(
    REPO_ROOT
    / (
        "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
        f"template_inspector_row_glyphs/{owner}.rs"
    )
    for owner in ("checks", "chevrons", "cubes", "swatches")
)
BUTTON_GLYPHS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_button_glyphs.rs"
)
BUTTON_GLYPH_SHAPES = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_button_glyphs/shapes.rs"
)
BUTTON_VECTOR_ASSETS = tuple(
    REPO_ROOT / f"zircon_editor/assets/icons/{relative_path}"
    for relative_path in (
        "zircon_editor_shell/controls/add.svg",
        "zircon_editor_shell/controls/delete.svg",
        "zircon_editor_shell/toolbar/dropdown.svg",
    )
)
ICON_BUTTON_GLYPHS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_icon_button_glyphs.rs"
)
ALERT_MARKS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_alert_glyphs/marks.rs"
)
ALERT_CLOSE = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_alert_glyphs/close.rs"
)
ALERT_VECTOR_ASSETS = tuple(
    REPO_ROOT
    / f"zircon_editor/assets/icons/zircon_editor_shell/status/alert-{tone}.svg"
    for tone in ("info", "success", "warning", "error")
)
POPUP_ADORNMENT_GLYPHS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_popup_row_adornments/glyphs.rs"
)
POPUP_ADORNMENT_DISPATCH = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_popup_row_adornments/glyphs/dispatch.rs"
)
POPUP_ADORNMENT_SELECTION = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_popup_row_adornments/selection.rs"
)
STATUS_GLYPHS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_status_glyphs.rs"
)
STATUS_ICON_GLYPHS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_status_glyphs/icon_glyphs.rs"
)
ICON_ALIASES = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "visual_assets/candidates/aliases.rs"
)
EDITOR_UI_ROOT = REPO_ROOT / "zircon_editor/assets/ui/editor"
EDITOR_ICON_ROOT = REPO_ROOT / "zircon_editor/assets/icons"


def packaged_icon_aliases() -> dict[str, str]:
    source = ICON_ALIASES.read_text(encoding="utf-8")
    arm_pattern = re.compile(
        r'(?P<keys>(?:"[^"]+"\s*(?:\|\s*)?)+)\s*=>\s*'
        r'(?:\{\s*)?Some\("(?P<path>[^"]+)"\)',
        re.DOTALL,
    )
    aliases: dict[str, str] = {}
    for arm in arm_pattern.finditer(source):
        for key in re.findall(r'"([^"]+)"', arm.group("keys")):
            aliases[key] = arm.group("path")
    return aliases


def literal_editor_icon_uses() -> dict[str, set[Path]]:
    uses: dict[str, set[Path]] = {}
    patterns = (
        re.compile(r'\bicon\s*=\s*"([^"]+)"'),
        re.compile(r'(?:^|[,|])icon=([A-Za-z0-9_-]+)'),
    )
    for path in EDITOR_UI_ROOT.rglob("*.zui"):
        source = path.read_text(encoding="utf-8")
        for pattern in patterns:
            for icon_name in pattern.findall(source):
                if not icon_name.startswith("$"):
                    uses.setdefault(icon_name, set()).add(path)
    return uses


def packaged_icon_path(icon_name: str, aliases: dict[str, str]) -> Path | None:
    normalized = icon_name.strip().replace("\\", "/").lstrip("/").lower()
    semantic_key = normalized.rsplit("/", 1)[-1]
    semantic_key = re.sub(r"\.(?:svg|png)$", "", semantic_key)
    if semantic_key in aliases:
        alias_path = EDITOR_ICON_ROOT / aliases[semantic_key]
        if alias_path.is_file():
            return alias_path
    candidates = (
        EDITOR_ICON_ROOT / icon_name,
        EDITOR_ICON_ROOT / f"{icon_name}.svg",
        EDITOR_ICON_ROOT / "ionicons" / icon_name,
        EDITOR_ICON_ROOT / "ionicons" / f"{icon_name}.svg",
    )
    return next((candidate for candidate in candidates if candidate.is_file()), None)


class EditorSvgRasterBucketPerformanceContract(unittest.TestCase):
    def test_workbench_status_icons_use_packaged_vector_assets(self) -> None:
        root = STATUS_GLYPHS.read_text(encoding="utf-8")
        icons = STATUS_ICON_GLYPHS.read_text(encoding="utf-8")

        self.assertIn("push_icon_asset_pixels", icons)
        for icon_name in ("snap", "globe", "target"):
            self.assertIn(f'"{icon_name}"', icons)
        self.assertNotIn("HostPaintCommand::quad", icons)
        self.assertNotIn("mod snap", icons)
        self.assertNotIn("mod target", icons)
        self.assertNotIn("mod world", icons)
        self.assertNotIn("mod segments", root)

    def test_popup_row_adornments_share_the_packaged_vector_asset_path(self) -> None:
        owner = POPUP_ADORNMENT_GLYPHS.read_text(encoding="utf-8")
        dispatch = POPUP_ADORNMENT_DISPATCH.read_text(encoding="utf-8")
        selection = POPUP_ADORNMENT_SELECTION.read_text(encoding="utf-8")

        self.assertIn("push_icon_asset_pixels", dispatch)
        for icon_name in ("checkmark", "chevron-right"):
            self.assertIn(f'"{icon_name}"', dispatch)
        self.assertIn("PopupRowAdornmentKind::Icon(icon_name) => icon_name", dispatch)
        for icon_name in ("add", "folder", "save", "trash"):
            self.assertIn(f'"{icon_name}"', selection)
        self.assertNotIn("HostPaintCommand::quad", dispatch)
        self.assertNotIn("mod assets", owner)
        self.assertNotIn("mod segments", owner)
        self.assertNotIn("mod symbols", owner)

    def test_alert_marks_use_tinted_vector_assets_instead_of_pixel_grid_segments(self) -> None:
        marks = ALERT_MARKS.read_text(encoding="utf-8")
        close = ALERT_CLOSE.read_text(encoding="utf-8")

        self.assertIn("push_icon_asset_pixels", marks)
        self.assertIn("push_icon_asset_pixels", close)
        self.assertNotIn("HostPaintCommand::quad", marks)
        self.assertNotIn("push_segments", marks)
        self.assertNotIn("push_segments", close)
        for asset in ALERT_VECTOR_ASSETS:
            self.assertTrue(asset.is_file(), asset)

    def test_tooltip_arrow_uses_supersampled_vector_pixels_instead_of_row_quads(self) -> None:
        source = TOOLTIP_ARROW.read_text(encoding="utf-8")

        self.assertIn("push_icon_asset_pixels", source)
        self.assertIn("tooltip-arrow.svg", source)
        self.assertNotIn("HostPaintCommand::quad", source)
        self.assertTrue(TOOLTIP_ARROW_ASSET.is_file())

    def test_tooltip_info_uses_packaged_vector_pixels_instead_of_manual_quads(self) -> None:
        source = TOOLTIP_INFO.read_text(encoding="utf-8")

        self.assertIn("push_icon_asset_pixels", source)
        self.assertIn("zircon_editor_shell/status/info.svg", source)
        self.assertNotIn("HostPaintCommand::quad", source)
        self.assertTrue(TOOLTIP_INFO_ASSET.is_file())

    def test_section_title_icons_use_packaged_vectors_without_grid_fallbacks(self) -> None:
        owner = SECTION_TITLE_GLYPHS.read_text(encoding="utf-8")
        shapes = SECTION_TITLE_SHAPES.read_text(encoding="utf-8")
        dispatch = SECTION_TITLE_DISPATCH.read_text(encoding="utf-8")

        self.assertIn("push_icon_asset_pixels", dispatch)
        for asset_name in ("cube.svg", "transform.svg", "mesh-renderer.svg"):
            self.assertIn(asset_name, dispatch)
        self.assertNotIn("HostPaintCommand::quad", dispatch)
        self.assertNotIn("mod segments", owner)
        for module_name in ("cube", "mesh", "transform"):
            self.assertNotIn(f"mod {module_name}", shapes)
        for asset in SECTION_TITLE_VECTOR_ASSETS:
            self.assertTrue(asset.is_file(), asset)

    def test_tree_row_glyphs_use_packaged_vectors_without_grid_fallbacks(self) -> None:
        owner = TREE_ROW_GLYPHS.read_text(encoding="utf-8")
        disclosure = TREE_ROW_DISCLOSURE.read_text(encoding="utf-8")
        object_owner = TREE_ROW_OBJECT.read_text(encoding="utf-8")
        object_dispatch = TREE_ROW_OBJECT_DISPATCH.read_text(encoding="utf-8")

        self.assertNotIn("mod segments", owner)
        self.assertNotIn("push_segments", disclosure)
        self.assertNotIn("mod icons", object_owner)
        self.assertNotIn("push_audio_icon", object_dispatch)
        self.assertNotIn("push_cube_icon", object_dispatch)
        self.assertNotIn("push_player_start_icon", object_dispatch)
        for path in (*TREE_ROW_ACTION_GLYPHS, TREE_ROW_DISCLOSURE, TREE_ROW_OBJECT_DISPATCH):
            source = path.read_text(encoding="utf-8")
            self.assertIn("push_icon_asset_pixels", source)
            self.assertNotIn("push_segments", source)

    def test_table_row_actions_use_packaged_vectors_without_grid_fallbacks(self) -> None:
        owner = TABLE_ROW_ACTIONS.read_text(encoding="utf-8")
        entry = TABLE_ROW_ACTION_ENTRY.read_text(encoding="utf-8")

        self.assertIn("push_icon_asset_pixels", entry)
        self.assertIn("settings.svg", entry)
        self.assertIn("more-horizontal.svg", entry)
        self.assertNotIn("push_table_gear", entry)
        self.assertNotIn("push_table_kebab", entry)
        self.assertNotIn("mod glyphs", owner)

    def test_compact_control_glyphs_never_fall_back_to_pixel_grids(self) -> None:
        chip_owner = CHIP_GLYPHS.read_text(encoding="utf-8")
        chip = CHIP_CHEVRON.read_text(encoding="utf-8")
        dropdown_owner = DROPDOWN_GLYPHS.read_text(encoding="utf-8")
        dropdown = DROPDOWN_CHEVRON.read_text(encoding="utf-8")
        stepper_owner = FIELD_STEPPER.read_text(encoding="utf-8")
        stepper = FIELD_STEPPER_COMMAND.read_text(encoding="utf-8")
        list_row = LIST_ROW_GLYPHS.read_text(encoding="utf-8")
        checkbox = CHECKBOX_TICK.read_text(encoding="utf-8")
        search = SEARCH_FIELD_GLYPH.read_text(encoding="utf-8")

        for source in (chip, dropdown, stepper, list_row, checkbox, search):
            self.assertIn("push_icon_asset_pixels", source)
        for owner in (chip_owner, dropdown_owner, stepper_owner, list_row):
            self.assertNotIn("mod segments", owner)
        self.assertNotIn("mod shapes", list_row)
        self.assertNotIn("push_segments", chip)
        self.assertNotIn("push_segments", dropdown)
        self.assertNotIn("checkbox_tick_segments", checkbox)
        self.assertNotIn("search_icon_ring_rect", search)
        self.assertIn("field-stepper.svg", stepper)
        self.assertTrue(FIELD_STEPPER_ASSET.is_file())

    def test_inspector_row_glyphs_use_packaged_vectors_without_manual_fallbacks(self) -> None:
        owner = INSPECTOR_ROW_GLYPHS.read_text(encoding="utf-8")

        self.assertNotIn("mod segments", owner)
        for path in INSPECTOR_ROW_VECTOR_OWNERS:
            source = path.read_text(encoding="utf-8")
            self.assertIn("push_icon_asset_pixels", source)
            self.assertNotIn("fallback", source)

    def test_button_glyphs_are_vector_only_and_icon_buttons_fail_closed(self) -> None:
        owner = BUTTON_GLYPHS.read_text(encoding="utf-8")
        shapes = BUTTON_GLYPH_SHAPES.read_text(encoding="utf-8")
        icon_buttons = ICON_BUTTON_GLYPHS.read_text(encoding="utf-8")

        self.assertNotIn("mod segments", owner)
        self.assertIn("push_icon_asset_pixels", shapes)
        self.assertNotIn("push_segments", shapes)
        self.assertNotIn("push_icon_button_glyph_shape", icon_buttons)
        self.assertNotIn("icon_button_glyph_kind", icon_buttons)
        self.assertIn("if node.icon_name.trim().is_empty()", icon_buttons)
        for asset in BUTTON_VECTOR_ASSETS:
            self.assertTrue(asset.is_file(), asset)

    def test_live_zui_literal_icons_never_require_manual_pixel_glyph_fallback(self) -> None:
        aliases = packaged_icon_aliases()
        unresolved = {
            icon_name: sorted(
                str(path.relative_to(REPO_ROOT)) for path in referencing_paths
            )
            for icon_name, referencing_paths in literal_editor_icon_uses().items()
            if packaged_icon_path(icon_name, aliases) is None
        }

        self.assertEqual(
            unresolved,
            {},
            "literal Editor .zui icons must resolve from packaged SVG assets: "
            f"{unresolved}",
        )

    def test_vector_target_has_a_bounded_adaptive_bucket_policy(self) -> None:
        source = TARGET.read_text(encoding="utf-8")
        method = source.split("fn vector_cache_bucket", 1)[1].split("fn fit_preserving_aspect", 1)[0]

        self.assertIn("VECTOR_RASTER_CACHE_SMALL_EDGE", source)
        self.assertIn("VECTOR_RASTER_CACHE_MEDIUM_EDGE", source)
        self.assertIn("VECTOR_RASTER_CACHE_LARGE_EDGE", source)
        self.assertIn(
            "if max_edge <= VECTOR_RASTER_CACHE_SMALL_EDGE {\n            1",
            method,
        )
        self.assertIn("self.quantized_up(bucket_edge)", method)
        self.assertIn("continuous_resize_uses_bounded_vector_cache_buckets", source)

    def test_vector_assets_bucket_before_pixel_cache_lookup(self) -> None:
        source = ASSET.read_text(encoding="utf-8")
        sized = source.split("fn load_visual_asset_pixels_for_target", 1)[1]

        self.assertIn("vector_cache_target", sized)
        self.assertIn("target.map(RasterTargetSize::vector_cache_bucket)", source)
        self.assertIn("source_is_svg", source)
        self.assertIn("load_vector_visual_asset_pixels_for_size", source)

    def test_explicit_vector_kind_preserves_the_hint_without_reclassifying_bitmaps(self) -> None:
        source = CONVERSION.read_text(encoding="utf-8")
        brush = BRUSH_CONVERSION.read_text(encoding="utf-8")

        self.assertIn("UiRenderResourceKind::Vector =>", source)
        self.assertIn("load_vector_visual_asset_pixels_for_size", source)
        self.assertIn("UiRenderResourceKind::Image =>", source)
        self.assertIn("resource_state.pixel_size", brush)
        self.assertIn("physical_pixel_size", source)
        self.assertIn("raster_target_for_resource", source)

    def test_template_vector_classification_respects_primary_source_priority(self) -> None:
        source = TEMPLATE.read_text(encoding="utf-8")

        self.assertIn("vector_cache_target", source)
        bucket = source.index("let target = vector_cache_target(")
        key = source.index("let key = template_image_cache_key")
        self.assertLess(bucket, key)
        self.assertIn("template_source_is_vector", source)
        self.assertIn("if !media_source.trim().is_empty()", source)
        self.assertIn("icon_source_is_vector(icon_name)", source)
        self.assertIn("explicit_vector", source)
        self.assertIn("image_candidates(media_source)", source)
        self.assertIn("icon_candidates(icon_name)", source)
        self.assertIn("CandidatePixelsLoad::Missing", source)
        self.assertIn("CandidatePixelsLoad::Deferred", source)
        self.assertNotIn("template_image_candidates", source)

    def test_svg_icon_role_preserves_vector_semantics_and_draw_geometry(self) -> None:
        source = (
            REPO_ROOT
            / (
                "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
                "template_node_images/command.rs"
            )
        ).read_text(encoding="utf-8")

        self.assertIn('node.role.as_str() == "SvgIcon"', source)
        self.assertIn("template_vector_image_pixels", source)
        self.assertIn("vector_raster_bucket_does_not_change_the_command_frame", source)


if __name__ == "__main__":
    unittest.main()
