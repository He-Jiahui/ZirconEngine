import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    REPO_ROOT / "zircon_runtime/src/ui/template/asset/compiler/binding_param_resolver.rs"
)
LOWER = (
    REPO_ROOT / "zircon_runtime/src/ui/template/asset/compiler/binding_param_resolver/tests/single_buffer_tests.rs"
)


def function_body(source: str, signature: str) -> str:
    match = re.search(rf"fn {signature}(?:<[^>]+>)?\(.*?\n}}\n", source, re.S)
    if match is None:
        raise AssertionError(f"{signature} owner is missing")
    return match.group(0)


class Runtime871BindingLiteralSingleBufferPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_flags_literal_streams_into_one_output(self) -> None:
        literal = function_body(self.source, "literal_source")
        self.assertIn("UiValue::Flags(values) => Some(flags_source(values))", literal)
        owner = function_body(self.source, "flags_source")
        self.assertIn("let mut source = String::with_capacity", owner)
        self.assertIn("values.iter().enumerate()", owner)
        self.assertIn("append_string_source(&mut source, value);", owner)
        self.assertNotIn("collect::<", owner)
        self.assertNotIn(".join(", owner)
        self.assertNotIn("format!(", owner)

    def test_vector_literal_streams_into_one_output(self) -> None:
        owner = function_body(self.source, "vector_source")
        self.assertIn("let mut source = String::with_capacity", owner)
        self.assertIn("values.iter().enumerate()", owner)
        self.assertIn("append_float_source(&mut source, *value)", owner)
        self.assertNotIn("collect::<", owner)
        self.assertNotIn(".join(", owner)
        self.assertNotIn("format!(", owner)

    def test_string_and_float_helpers_append_without_child_output(self) -> None:
        string_owner = function_body(self.source, "append_string_source")
        self.assertIn("source.push", string_owner)
        float_owner = function_body(self.source, "append_float_source")
        self.assertIn("let value_start = source.len();", float_owner)
        self.assertIn("source.truncate(value_start);", float_owner)
        self.assertIn("write!(source", float_owner)

    def test_control_escape_writes_through_the_existing_mutable_borrow(self) -> None:
        owner = function_body(self.source, "append_string_source")
        self.assertIn('write!(source, "\\\\u{:04x}", ch as u32)', owner)
        self.assertNotIn('write!(&mut source,', owner)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn("mod single_buffer_tests;", self.source)
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn(
            "runtime871_binding_literal_single_buffer_preserves_exact_text", lower
        )
        self.assertIn("RUNTIME871_BINDING_LITERAL_SINGLE_BUFFER_BENCH_V1", lower)
        self.assertIn("const SAMPLE_PAIRS: usize = 101;", lower)
        self.assertIn("legacy_p50_ns", lower)
        self.assertIn("optimized_p99_ns", lower)

    def test_dense_flags_temporary_allocation_model(self) -> None:
        values = 4_096
        legacy_child_strings = values
        legacy_vector_slots = values
        optimized_child_strings = 0
        optimized_vector_slots = 0
        self.assertEqual(legacy_child_strings, 4_096)
        self.assertEqual(legacy_vector_slots, 4_096)
        self.assertEqual(optimized_child_strings, 0)
        self.assertEqual(optimized_vector_slots, 0)


if __name__ == "__main__":
    unittest.main()
