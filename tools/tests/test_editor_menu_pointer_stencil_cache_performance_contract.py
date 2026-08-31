from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
MENU_LAYOUT = (
    ROOT
    / "zircon_editor/src/ui/retained_host/menu_pointer/build_host_menu_pointer_layout.rs"
)


class EditorMenuPointerStencilCachePerformanceContractTests(unittest.TestCase):
    def test_pointer_stencil_is_cached_by_resource_generation(self) -> None:
        source = MENU_LAYOUT.read_text(encoding="utf-8")

        self.assertIn("thread_local!", source)
        self.assertIn("struct MenuPointerChromeStencilCache", source)
        self.assertIn("ViewTemplateResourceGeneration", source)
        self.assertIn("view_template_resource_generation(MENU_CHROME_ASSET, &[])", source)
        self.assertIn("cached.generation == generation", source)
        self.assertIn("MENU_POINTER_CHROME_STENCIL_CACHE", source)

    def test_resize_path_does_not_reproject_the_stencil_per_frame(self) -> None:
        source = MENU_LAYOUT.read_text(encoding="utf-8")
        start = source.index("fn menu_button_frames_from_chrome_asset(")
        end = source.index("\nfn ", start + 1)
        body = source[start:end]

        self.assertIn("menu_pointer_chrome_stencil()", body)
        self.assertNotIn("build_view_template_node_projection(", body)
        self.assertIn("frame.x + slot.x", body)
        self.assertIn("frame.y + slot.y", body)

    def test_stencil_cache_is_single_entry_and_not_keyed_by_window_size(self) -> None:
        source = MENU_LAYOUT.read_text(encoding="utf-8")
        start = source.index("fn menu_pointer_chrome_stencil(")
        end = source.index("\nfn ", start + 1)
        body = source[start:end]

        self.assertIn("MENU_STENCIL_REFERENCE_WIDTH", body)
        self.assertIn("MENU_STENCIL_REFERENCE_HEIGHT", body)
        self.assertNotIn("HashMap", source)
        self.assertNotIn("UiSize::new(frame.width", body)
        self.assertNotIn("UiSize::new(frame.height", body)


if __name__ == "__main__":
    unittest.main()
