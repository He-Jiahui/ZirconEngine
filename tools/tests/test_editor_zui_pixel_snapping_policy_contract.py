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
    def test_static_editor_chrome_declares_device_pixel_alignment(self):
        workbench = load_zui(WORKBENCH_WINDOW)
        status_bar = load_zui(STATUS_BAR)
        divider = load_zui(DIVIDER)
        dropdown = load_zui(DROPDOWN)

        self.assertEqual(
            workbench["nodes"]["root"]["pixel_snapping"], "snap_to_pixel"
        )
        self.assertEqual(
            status_bar["nodes"]["status_bar"]["pixel_snapping"],
            "snap_to_pixel",
        )
        self.assertEqual(
            divider["nodes"]["root"]["pixel_snapping"], "snap_to_pixel"
        )
        self.assertEqual(
            dropdown["nodes"]["root"]["pixel_snapping"], "snap_to_pixel"
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
