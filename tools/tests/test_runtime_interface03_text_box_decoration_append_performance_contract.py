import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/surface/render/command.rs"
BENCH_SOURCE = ROOT / "zircon_runtime_interface/src/ui/surface/render/command/tests/text_box_decoration_performance_tests.rs"


class RuntimeInterface03TextBoxDecorationAppendPerformanceContract(unittest.TestCase):
    def test_text_paint_appends_box_decorations_into_final_buffer(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("    fn text_paint(&self) -> UiTextPaint {")
        end = source.index("\n    }\n}", start)
        body = source[start:end]

        self.assertIn(
            "append_text_box_background_decorations(&layout.boxes, &mut decorations)",
            body,
        )
        self.assertIn(
            "append_text_box_border_decorations(&layout.boxes, &mut decorations)",
            body,
        )
        self.assertIn("layout.boxes.len().saturating_mul(2)", body)
        self.assertNotIn("decorations.extend(text_box_background_decorations", body)
        self.assertNotIn("decorations.extend(text_box_border_decorations", body)

    def test_box_decoration_helpers_fill_caller_owned_output(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")

        for helper in [
            "append_text_box_background_decorations",
            "append_text_box_border_decorations",
        ]:
            start = source.index(f"fn {helper}(")
            end = source.index("\n}\n", start) + 2
            body = source[start:end]
            self.assertIn("decorations: &mut Vec<UiTextPaintDecoration>", body)
            self.assertIn("decorations.extend(", body)

    def test_direct_box_decoration_append_has_release_benchmark(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        bench_source = BENCH_SOURCE.read_text(encoding="utf-8")

        self.assertIn("mod text_box_decoration_performance_tests;", source)
        self.assertIn(
            "RUNTIME_INTERFACE03_DIRECT_TEXT_BOX_DECORATION_APPEND_BENCH_V1",
            bench_source,
        )
        self.assertIn("saturating_mul(10)", bench_source)
        self.assertIn("saturating_mul(9)", bench_source)


if __name__ == "__main__":
    unittest.main()
