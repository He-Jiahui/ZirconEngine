from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
POINTER_TABLE = (
    ROOT / "zircon_runtime/src/ui/dispatch/input_manager/pointer_table.rs"
)


def rust_block(source: str, signature: str) -> str:
    start = source.index(signature)
    opening = source.index("{", start)
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index]
    raise AssertionError(f"unterminated Rust block: {signature}")


class RuntimePointerHoverPathPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = POINTER_TABLE.read_text(encoding="utf-8")

    def test_slice_adapter_forwards_a_borrowed_iterator(self) -> None:
        body = rust_block(self.source, "pub fn set_hovered_path(")

        self.assertIn("impl AsRef<[UiNodeId]>", self.source)
        self.assertIn("set_hovered_path_iter(pointer_id, hovered.iter().copied())", body)
        self.assertNotIn("to_vec()", body)
        self.assertNotIn(".clone()", body)

    def test_iterator_setter_compares_before_reusing_the_retained_buffer(self) -> None:
        body = rust_block(self.source, "pub fn set_hovered_path_iter(")

        self.assertIn("Iterator<Item = UiNodeId> + Clone", self.source)
        self.assertIn("entry.hovered.iter().copied().eq(hovered.clone())", body)
        self.assertIn("entry.hovered.clear()", body)
        self.assertIn("entry.hovered.extend(hovered)", body)
        self.assertNotIn("entry.hovered = hovered", body)
        self.assertNotIn(".collect()", body)
        self.assertNotIn("to_vec()", body)

    def test_retained_buffer_and_release_benchmark_are_compiled_as_rust_tests(self) -> None:
        self.assertIn('path = "pointer_table/hovered_path_tests.rs"', self.source)
        tests = (
            POINTER_TABLE.parent / "pointer_table/hovered_path_tests.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("runtime200_pointer_hover_path_reuses_retained_buffer", tests)
        self.assertIn("runtime200_pointer_hover_path_reuse_p95", tests)
        self.assertIn("RUNTIME200_POINTER_HOVER_RETAINED_PATH_BENCH_V1", tests)
        self.assertIn('ignore = "release performance evidence"', tests)


if __name__ == "__main__":
    unittest.main()
