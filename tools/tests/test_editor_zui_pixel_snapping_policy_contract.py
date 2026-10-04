import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
WORKBENCH_WINDOW = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/windows/workbench_window.zui"
)
STATUS_BAR = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/shell/"
    "workbench_status_bar.zui"
)
DIVIDER = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/primitives/data/"
    "workbench_divider.zui"
)
DRAG_OVERLAY = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/primitives/feedback/"
    "workbench_drag_overlay.zui"
)
SLIDER = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/"
    "workbench_slider.zui"
)
RANGE_SLIDER = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/"
    "workbench_range_slider.zui"
)
PROGRESS_BAR = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/primitives/feedback/"
    "workbench_progress_bar.zui"
)
SKELETON = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/primitives/feedback/"
    "workbench_skeleton.zui"
)
DROPDOWN = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/"
    "workbench_dropdown.zui"
)
BUTTON = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/"
    "workbench_button.zui"
)
CHIP = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/primitives/chrome/"
    "workbench_chip.zui"
)
ICON_BUTTON = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/"
    "workbench_icon_button.zui"
)
ACTIVITY_DRAWER_WINDOW = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/shell/"
    "activity_drawer_window.zui"
)
WELCOME = REPO_ROOT / "zircon_editor/assets/ui/editor/welcome.zui"
HOST_SURFACE_DOCUMENTS = tuple(
    sorted((REPO_ROOT / "zircon_editor/assets/ui/editor/host").glob("*.zui"))
)
INDEPENDENT_SURFACE_ROOTS = tuple(
    (REPO_ROOT / path, root_node)
    for path, root_node in (
        ("zircon_editor/assets/ui/editor/host/workbench_shell.zui", "host"),
        ("zircon_editor/assets/ui/editor/host/editor_main_frame.zui", "root"),
        (
            "zircon_editor/assets/ui/editor/host/floating_window_source.zui",
            "root",
        ),
        (
            "zircon_editor/assets/ui/editor/component_showcase.zui",
            "component_showcase_root",
        ),
        ("zircon_editor/assets/ui/editor/material_demo_window.zui", "window"),
        (
            "zircon_editor/assets/ui/editor/material_component_lab.zui",
            "material_lab_root",
        ),
        ("zircon_editor/assets/ui/editor/asset_browser.zui", "asset_browser_root"),
        ("zircon_editor/assets/ui/editor/assets_activity.zui", "assets_activity_root"),
        ("zircon_editor/assets/ui/editor/hierarchy.zui", "hierarchy_root"),
        ("zircon_editor/assets/ui/editor/inspector.zui", "inspector_root"),
        ("zircon_editor/assets/ui/editor/console.zui", "console_root"),
        ("zircon_editor/assets/ui/editor/project_overview.zui", "project_overview_root"),
        (
            "zircon_editor/assets/ui/editor/ui_asset_editor.zui",
            "ui_asset_editor_root",
        ),
        (
            "zircon_editor/assets/ui/editor/workbench_menu_chrome.zui",
            "workbench_menu_chrome_root",
        ),
        (
            "zircon_editor/assets/ui/editor/workbench_menu_popup.zui",
            "workbench_menu_popup_root",
        ),
        (
            "zircon_editor/assets/ui/editor/workbench_dock_header.zui",
            "dock_header_root",
        ),
        (
            "zircon_editor/assets/ui/editor/workbench_page_chrome.zui",
            "workbench_page_chrome_root",
        ),
        (
            "zircon_editor/assets/ui/editor/workbench_status_bar.zui",
            "workbench_status_bar_root",
        ),
        (
            "zircon_editor/assets/ui/editor/workbench_activity_rail.zui",
            "activity_rail_root",
        ),
    )
)
RUNTIME_POLICY_BOUNDARY_FILES = tuple(
    REPO_ROOT / path
    for path in (
        "zircon_runtime/src/ui/surface/render/divider.rs",
        "zircon_runtime/src/ui/surface/render/progress.rs",
        "zircon_runtime/src/ui/surface/render/skeleton.rs",
        "zircon_runtime/src/ui/surface/render/sliders.rs",
        "zircon_runtime/src/ui/surface/render/dropdowns.rs",
    )
)
NATIVE_SLIDER_CONTEXT = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_sliders/commands/context.rs"
)
NATIVE_FRACTIONAL_CONTROL_GEOMETRY_FILES = tuple(
    REPO_ROOT / path
    for path in (
        "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
        "template_buttons/geometry.rs",
        "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
        "template_dropdowns/geometry.rs",
        "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
        "template_fields/geometry.rs",
        "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
        "template_fields/search.rs",
        "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
        "material_primitives/text_field/geometry.rs",
        "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
        "template_chips/geometry.rs",
    )
)


def load_zui(path: Path) -> dict:
    with path.open("rb") as source:
        return tomllib.load(source)


class EditorZuiPixelSnappingPolicyContractTests(unittest.TestCase):
    def test_every_host_surface_root_preserves_fractional_geometry(self):
        for path in HOST_SURFACE_DOCUMENTS:
            with self.subTest(path=path.name):
                document = load_zui(path)
                root_node = document["root"]["node"]
                self.assertEqual(
                    document["nodes"][root_node].get("pixel_snapping"),
                    "disabled",
                )

    def test_product_root_preserves_fractional_controls_and_static_chrome_opts_in(self):
        workbench = load_zui(WORKBENCH_WINDOW)
        status_bar = load_zui(STATUS_BAR)
        divider = load_zui(DIVIDER)
        dropdown = load_zui(DROPDOWN)
        button = load_zui(BUTTON)
        chip = load_zui(CHIP)
        icon_button = load_zui(ICON_BUTTON)
        activity_drawer_window = load_zui(ACTIVITY_DRAWER_WINDOW)
        welcome = load_zui(WELCOME)

        self.assertEqual(
            workbench["nodes"]["root"]["pixel_snapping"], "disabled"
        )
        self.assertEqual(
            status_bar["nodes"]["status_bar"]["pixel_snapping"],
            "snap_to_pixel",
        )
        self.assertEqual(
            divider["nodes"]["root"]["pixel_snapping"], "snap_to_pixel"
        )
        self.assertEqual(
            dropdown["nodes"]["root"]["pixel_snapping"], "disabled"
        )
        self.assertNotIn("pixel_snapping", button["nodes"]["root"])
        self.assertEqual(
            chip["nodes"]["root"]["pixel_snapping"], "disabled"
        )
        self.assertEqual(
            icon_button["nodes"]["root"]["pixel_snapping"], "disabled"
        )
        self.assertEqual(
            activity_drawer_window["nodes"]["root"]["pixel_snapping"],
            "disabled",
        )
        self.assertEqual(
            welcome["nodes"]["welcome_root"]["pixel_snapping"], "disabled"
        )
        for path, root_node in INDEPENDENT_SURFACE_ROOTS:
            with self.subTest(path=path.name):
                document = load_zui(path)
                self.assertEqual(
                    document["nodes"][root_node]["pixel_snapping"], "disabled"
                )

        for path, separator_node in (
            (
                REPO_ROOT / "zircon_editor/assets/ui/editor/workbench_menu_chrome.zui",
                "workbench_menu_separator",
            ),
            (
                REPO_ROOT / "zircon_editor/assets/ui/editor/workbench_status_bar.zui",
                "workbench_status_bar_separator",
            ),
        ):
            with self.subTest(path=path.name, separator=separator_node):
                document = load_zui(path)
                self.assertEqual(
                    document["nodes"][separator_node]["pixel_snapping"],
                    "snap_to_pixel",
                )

    def test_pointer_transformed_overlay_preserves_subpixel_motion(self):
        drag_overlay = load_zui(DRAG_OVERLAY)
        root = drag_overlay["nodes"]["root"]

        self.assertEqual(root["pixel_snapping"], "disabled")
        self.assertNotIn("pixel_snapping", root.get("props", {}))

    def test_continuously_moving_slider_geometry_preserves_subpixel_motion(self):
        for path in (SLIDER, RANGE_SLIDER):
            with self.subTest(path=path.name):
                root = load_zui(path)["nodes"]["root"]
                self.assertEqual(root["pixel_snapping"], "disabled")
                self.assertNotIn("pixel_snapping", root.get("props", {}))

    def test_animated_feedback_preserves_subpixel_motion(self):
        for path in (PROGRESS_BAR, SKELETON):
            with self.subTest(path=path.name):
                root = load_zui(path)["nodes"]["root"]
                self.assertEqual(root["pixel_snapping"], "disabled")
                self.assertNotIn("pixel_snapping", root.get("props", {}))

    def test_painters_defer_geometry_snapping_to_the_final_paint_policy(self):
        for path in RUNTIME_POLICY_BOUNDARY_FILES:
            with self.subTest(path=path.name):
                source = path.read_text(encoding="utf-8")
                self.assertNotIn("pixel_aligned_frame", source)
                self.assertNotIn(".round()", source)
        self.assertNotIn(
            "pixel_aligned_rect",
            NATIVE_SLIDER_CONTEXT.read_text(encoding="utf-8"),
        )

    def test_native_controls_preserve_fractional_frames_until_raster_coverage(self):
        for path in NATIVE_FRACTIONAL_CONTROL_GEOMETRY_FILES:
            with self.subTest(path=path.name):
                source = path.read_text(encoding="utf-8")
                self.assertNotIn("inward_pixel_aligned_rect", source)
                self.assertNotIn(".round()", source)


if __name__ == "__main__":
    unittest.main()
