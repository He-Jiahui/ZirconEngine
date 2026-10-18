import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/surface/render/command.rs"
BENCH_SOURCE = ROOT / "zircon_runtime_interface/src/ui/surface/render/command/tests/debug_label_performance_tests.rs"


class RuntimeInterface03RenderCommandDebugLabelPerformanceContract(unittest.TestCase):
    def test_cached_paint_element_uses_static_kind_label(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("    fn base_paint_element(")
        end = source.index("\n    fn resolved_paint_metrics", start)
        body = source[start:end]

        self.assertIn("render_command_debug_label(self.kind)", body)
        self.assertNotIn('format!("{:?}", self.kind)', body)

    def test_static_kind_label_covers_all_variants(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("fn render_command_kind_name(")
        end = source.index("\n}\n", start) + 2
        body = source[start:end]

        for variant in ["Group", "Quad", "Text", "Image"]:
            self.assertIn(
                f'UiRenderCommandKind::{variant} => "{variant}"',
                body,
            )

    def test_static_debug_label_has_release_benchmark(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        bench_source = BENCH_SOURCE.read_text(encoding="utf-8")

        self.assertIn("mod debug_label_performance_tests;", source)
        self.assertIn(
            "RUNTIME_INTERFACE03_STATIC_RENDER_COMMAND_DEBUG_LABEL_BENCH_V1",
            bench_source,
        )
        self.assertIn("saturating_mul(5)", bench_source)
        self.assertIn("saturating_mul(4)", bench_source)


if __name__ == "__main__":
    unittest.main()
