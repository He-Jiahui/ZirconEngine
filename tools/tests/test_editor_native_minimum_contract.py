import unittest
from pathlib import Path


class EditorNativeMinimumContractTests(unittest.TestCase):
    """Static guard for the Editor logical-to-native minimum-size boundary."""

    def setUp(self) -> None:
        self.repo_root = Path(__file__).resolve().parents[2]
        base = self.repo_root / "zircon_editor/src/ui/retained_host/host_contract/window/event_loop"
        self.minimum = base / "lifecycle/native_minimum.rs"
        self.native_window = base / "lifecycle/native_window.rs"
        self.lifecycle = base / "lifecycle.rs"
        self.redraw = base / "redraw.rs"

    @staticmethod
    def _production(source: str) -> str:
        return source.split("#[cfg(test)]\nmod tests", 1)[0]

    def test_minimum_reader_prefers_published_physical_extents_and_sanitizes_fallbacks(self) -> None:
        source = self.minimum.read_text(encoding="utf-8")
        production = self._production(source)
        for anchor in (
            "const DEFAULT_SCALE_FACTOR: f32 = 1.0;",
            "pub(super) fn current_native_minimum_surface_size(host: &UiHostWindow)",
            "state.window_scale_factor.is_finite() && state.window_scale_factor > 0.0",
            "published_or_fallback_physical_extent(shell.shell_min_width_px, fallback_width, scale)",
            "published_or_fallback_physical_extent(shell.shell_min_height_px, fallback_height, scale)",
            "if published_physical.is_finite() && published_physical > 0.0",
            "physical_extent(published_physical)",
            "logical_to_physical(fallback_logical, normalized_scale(scale_factor))",
            "logical_to_physical(logical: f32, scale: f64)",
            ".ceil().min(f64::from(u32::MAX)) as u32",
        ):
            self.assertIn(anchor, production)
        self.assertLess(
            production.index("state.window_scale_factor.is_finite()"),
            production.index("published_or_fallback_physical_extent("),
        )
        self.assertNotIn("logical_to_physical(published_physical", production)

    def test_native_window_creation_receives_physical_minimum_without_redpi_scaling(self) -> None:
        source = self.native_window.read_text(encoding="utf-8")
        production = self._production(source)
        for anchor in (
            "create_native_window_or_exit(",
            "native_window_attributes(&size, minimum_size)",
            "native_window_attributes_for_size(&size, minimum_size)",
            ".with_surface_size(Size::Physical(WinitPhysicalSize::new(",
            ".with_min_surface_size(Size::Physical(minimum_size))",
            "profile_initial_client_size().unwrap_or_else(|| size.clone())",
            "parse_profile_initial_client_size(",
        ):
            self.assertIn(anchor, production)
        self.assertNotIn("scale_factor", production)
        self.assertNotIn("logical_to_physical", production)

    def test_event_loop_applies_and_refreshes_minimum_on_the_owner_thread(self) -> None:
        lifecycle = self._production(self.lifecycle.read_text(encoding="utf-8"))
        redraw = self._production(self.redraw.read_text(encoding="utf-8"))
        for anchor in (
            "use native_minimum::current_native_minimum_surface_size;",
            "use native_window::create_native_window_or_exit;",
            "let minimum_size = current_native_minimum_surface_size(&self.host);",
            "create_native_window_or_exit(event_loop, &self.host, requested_size, minimum_size)",
            "self.sync_native_window_minimum(window.as_ref());",
        ):
            self.assertIn(anchor, lifecycle)
        self.assertLess(
            lifecycle.index("let minimum_size = current_native_minimum_surface_size"),
            lifecycle.index("create_native_window_or_exit("),
        )
        self.assertLess(
            lifecycle.index("create_native_window_or_exit("),
            lifecycle.index("self.sync_native_window_minimum(window.as_ref())"),
        )
        for anchor in (
            "self.host.request_frame_update();",
            "self.host.request_interactive_frame_update();",
            "self.sync_native_window_minimum_from_presentation();",
        ):
            self.assertIn(anchor, redraw)
        self.assertLess(
            redraw.index("self.host.request_frame_update();"),
            redraw.index("self.sync_native_window_minimum_from_presentation();"),
        )
        self.assertLess(
            redraw.index("self.host.request_interactive_frame_update();"),
            redraw.index("self.sync_native_window_minimum_from_presentation();"),
        )

    def test_minimum_updates_are_deduplicated_and_regressions_cover_dpi_and_non_finite_inputs(self) -> None:
        source = self.minimum.read_text(encoding="utf-8")
        production = self._production(source)
        for anchor in (
            "let key = (minimum.width, minimum.height);",
            "if self.native_minimum_surface_size == Some(key)",
            "return;",
            "window.set_min_surface_size(Some(Size::Physical(minimum)));",
            "self.native_minimum_surface_size = Some(key);",
            "fn astra_editor_native_minimum_rounds_logical_extent_up_at_dpi()",
            "fn astra_editor_native_minimum_sanitizes_scale_and_non_finite_extent()",
            "fn astra_editor_two_x_geometry_minimum_is_published_to_native_exactly_once()",
            "fn astra_editor_native_minimum_is_synchronized_by_the_event_loop_redraw_owner()",
        ):
            self.assertIn(anchor, source)
        dedupe = production.index("if self.native_minimum_surface_size == Some(key)")
        setter = production.index("window.set_min_surface_size", dedupe)
        self.assertLess(dedupe, setter)
        self.assertIn("f32::NAN", source)
        self.assertIn("f32::INFINITY", source)
        self.assertIn("scale_factor = 2.0", source)
        self.assertIn("logical_to_physical(720.01, 1.25)", source)


if __name__ == "__main__":
    unittest.main()
