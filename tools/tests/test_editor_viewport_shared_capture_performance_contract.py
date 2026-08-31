from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
DATA = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/data/viewport_image.rs"
)
PAINTER = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/"
    "native_panes/viewport.rs"
)


class EditorViewportSharedCapturePerformanceContract(unittest.TestCase):
    def test_viewport_data_exposes_the_existing_shared_pixel_owner(self) -> None:
        source = DATA.read_text(encoding="utf-8")
        accessor = source.split("pub(crate) fn rgba(", 1)[1]
        accessor = accessor.split("pub(crate) fn play_frame_identity", 1)[0]

        self.assertIn("Option<&Arc<[u8]>>", accessor)
        self.assertIn("self.rgba.as_ref()", accessor)
        self.assertNotIn("as_deref", accessor)

    def test_native_viewport_records_captured_pixels_by_shared_owner(self) -> None:
        source = PAINTER.read_text(encoding="utf-8")
        draw = source.split("fn draw_viewport_image(", 1)[1]

        self.assertIn("draw_shared_rgba_image_clipped_with_resource_key", source)
        self.assertIn("Some(rgba) => draw_shared_rgba_image_clipped_with_resource_key", draw)
        self.assertNotIn("draw_rgba_image_clipped_with_resource_key", source)


if __name__ == "__main__":
    unittest.main()
