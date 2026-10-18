import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
COMMAND_SOURCE = ROOT / "zircon_runtime_interface/src/ui/surface/render/command.rs"
LIST_SOURCE = ROOT / "zircon_runtime_interface/src/ui/surface/render/list.rs"
FRAME_SOURCE = ROOT / "zircon_runtime_interface/src/ui/surface/render/frame_extract.rs"
TEST_SOURCE = (
    ROOT / "zircon_runtime_interface/src/ui/surface/render/frame_extract/tests/direct_append_tests.rs"
)


def function_body(source: str, signature: str) -> str:
    start = source.index(signature)
    brace = source.index("{", start)
    depth = 0
    for index in range(brace, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[brace : index + 1]
    raise AssertionError(f"unterminated function body for {signature}")


class RuntimeInterface03RenderListDirectAppendPerformanceContract(unittest.TestCase):
    def test_command_exposes_crate_internal_append_without_clearing_output(self) -> None:
        source = COMMAND_SOURCE.read_text(encoding="utf-8")
        body = function_body(source, "pub(super) fn append_paint_elements(")

        self.assertIn("append_paint_elements_with_metrics", body)
        self.assertNotIn("elements.clear()", body)
        self.assertIn("let first_element_index = elements.len();", source)
        self.assertIn("elements.len() - first_element_index", source)

    def test_flat_render_list_writes_commands_into_one_output_buffer(self) -> None:
        source = LIST_SOURCE.read_text(encoding="utf-8")
        body = function_body(source, "pub fn to_paint_elements_with_metrics(")

        self.assertIn("command.append_paint_elements(", body)
        self.assertNotIn("let mut command_elements", body)
        self.assertNotIn("elements.append", body)

    def test_persistent_frame_list_writes_commands_into_one_output_buffer(self) -> None:
        source = FRAME_SOURCE.read_text(encoding="utf-8")
        body = function_body(source, "pub fn to_paint_elements_with_metrics(")

        self.assertIn("command.append_paint_elements(", body)
        self.assertNotIn("let mut command_elements", body)
        self.assertNotIn("elements.append", body)
        self.assertIn("mod direct_append_tests;", source)

    def test_both_release_benchmarks_are_present(self) -> None:
        source = TEST_SOURCE.read_text(encoding="utf-8")
        self.assertIn("RUNTIME_INTERFACE03_RENDER_LIST_DIRECT_APPEND_BENCH_V1", source)
        self.assertIn(
            "RUNTIME_INTERFACE03_RENDER_FRAME_LIST_DIRECT_APPEND_BENCH_V1",
            source,
        )


if __name__ == "__main__":
    unittest.main()
