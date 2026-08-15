from pathlib import Path
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
RESIZE_EVENTS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/events/resize.rs"
)
GPU_LIFECYCLE = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/presenter/gpu/lifecycle.rs"
)
WGPU_SURFACE = REPO_ROOT / "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface.rs"


def function_body(source: str, signature: str, next_signature: str) -> str:
    return source.split(signature, 1)[1].split(next_signature, 1)[0]


class EditorNativeWindowResizePerformanceContract(unittest.TestCase):
    def test_duplicate_native_size_is_rejected_before_reflow_queueing(self) -> None:
        source = RESIZE_EVENTS.read_text(encoding="utf-8")
        body = function_body(
            source,
            "fn handle_surface_resized",
            "fn handle_window_scale_factor_changed",
        )

        duplicate_gate = body.index("physical_size == self.host.window().size()")
        mutate_scale = body.index(".set_scale_factor(metrics.scale_factor as f32)")
        mutate_size = body.index("self.host.window().set_size")
        queue_reflow = body.index("self.queue_resize_reflow")
        self.assertIn("(!duplicate_size).then_some(physical_size)", body)
        self.assertLess(duplicate_gate, mutate_scale)
        self.assertLess(duplicate_gate, mutate_size)
        self.assertLess(duplicate_gate, queue_reflow)

    def test_editor_presenter_keeps_cache_on_same_size_resize(self) -> None:
        source = GPU_LIFECYCLE.read_text(encoding="utf-8")
        body = function_body(source, "fn resize", "fn diagnostics_snapshot")

        duplicate_gate = body.index("size == self.size")
        surface_resize = body.index("self.surface.resize")
        invalidate = body.index("self.surface_cache_initialized = false")
        self.assertLess(duplicate_gate, surface_resize)
        self.assertLess(duplicate_gate, invalidate)

    def test_duplicate_scale_factor_does_not_restart_resize_reflow(self) -> None:
        source = RESIZE_EVENTS.read_text(encoding="utf-8")
        body = function_body(
            source,
            "fn handle_window_scale_factor_changed",
            "fn queue_resize_reflow",
        )

        duplicate_gate = body.index(
            "scale_factor.to_bits() == self.host.window().scale_factor().to_bits()"
        )
        mutate_scale = body.index("set_scale_factor")
        queue_reflow = body.index("self.queue_resize_reflow")
        self.assertLess(duplicate_gate, mutate_scale)
        self.assertLess(duplicate_gate, queue_reflow)
        self.assertIn("self.queue_resize_reflow(None)", body)

    def test_wgpu_presenter_does_not_reconfigure_an_unchanged_extent(self) -> None:
        source = WGPU_SURFACE.read_text(encoding="utf-8")
        presenter_impl = source.split(
            "impl UiSurfacePresenter for WgpuUiSurfacePresenter", 1
        )[1]
        body = function_body(presenter_impl, "fn resize", "fn is_image_resource_resident")

        duplicate_gate = body.index("size == self.descriptor.clamped_size()")
        native_resize = body.index("renderer.resize")
        self.assertLess(duplicate_gate, native_resize)


if __name__ == "__main__":
    unittest.main()
