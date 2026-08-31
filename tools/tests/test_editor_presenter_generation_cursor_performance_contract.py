from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
PRESENTER_TRAIT = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/presenter/"
    "host_chrome_presenter.rs"
)
REDRAW_PRESENT = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/"
    "redraw/present.rs"
)
GPU_IMPL = ROOT / "zircon_editor/src/ui/retained_host/host_contract/presenter/gpu.rs"
GPU_PRESENT = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/presenter/gpu/present.rs"
)
SOFTBUFFER_IMPL = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer.rs"
)
SOFTBUFFER_PRESENT = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/present.rs"
)


class EditorPresenterGenerationCursorPerformanceContractTests(unittest.TestCase):
    def test_redraw_binds_structure_and_cursor_from_one_generation(self) -> None:
        source = REDRAW_PRESENT.read_text(encoding="utf-8")
        body = source.split("fn present_redraw", 1)[1]

        generation = body.index("get_host_presentation_generation()")
        cursor = body.index("let presentation_cursor = generation.cursor()")
        structure = body.index("let presentation = generation.structure()")
        present = body.index("presenter.present(")
        self.assertLess(generation, cursor)
        self.assertLess(cursor, present)
        self.assertLess(structure, present)
        self.assertIn("presentation_cursor", body[present : present + 220])

    def test_presenter_trait_requires_generation_cursor_for_all_present_modes(self) -> None:
        source = PRESENTER_TRAIT.read_text(encoding="utf-8")
        trait_body = source.split("trait HostChromePresenter", 1)[1]

        self.assertGreaterEqual(
            trait_body.count("HostPresentationGenerationCursor"),
            2,
        )
        fallback = trait_body.split("fn present_during_native_resize", 1)[1]
        self.assertIn("self.present(presentation, presentation_cursor, None", fallback)

    def test_gpu_backend_receives_the_cursor_beyond_the_trait_adapter(self) -> None:
        adapter = GPU_IMPL.read_text(encoding="utf-8")
        backend = GPU_PRESENT.read_text(encoding="utf-8")
        compact_adapter = "".join(adapter.split())

        self.assertIn("presentation_cursor: HostPresentationGenerationCursor", adapter)
        self.assertIn(
            "GpuChromePresenter::present(self,presentation,presentation_cursor",
            compact_adapter,
        )
        self.assertIn("presentation_cursor: HostPresentationGenerationCursor", backend)

    def test_softbuffer_backend_receives_the_cursor_beyond_the_trait_adapter(self) -> None:
        adapter = SOFTBUFFER_IMPL.read_text(encoding="utf-8")
        backend = SOFTBUFFER_PRESENT.read_text(encoding="utf-8")
        compact_adapter = "".join(adapter.split())

        self.assertIn("presentation_cursor: HostPresentationGenerationCursor", adapter)
        self.assertIn(
            "SoftbufferHostPresenter::present(self,presentation,presentation_cursor",
            compact_adapter,
        )
        self.assertIn("presentation_cursor: HostPresentationGenerationCursor", backend)


if __name__ == "__main__":
    unittest.main()
