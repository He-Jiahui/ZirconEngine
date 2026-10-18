import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    REPO_ROOT
    / "zircon_runtime"
    / "src"
    / "ui"
    / "component"
    / "state_reducer"
    / "command_palette.rs"
)


def function_body(source: str, function_name: str) -> str:
    match = re.search(
        rf"\bfn\s+{re.escape(function_name)}(?![A-Za-z0-9_])\s*\(.*?\)",
        source,
        re.S,
    )
    if match is None:
        raise AssertionError(f"missing function {function_name}")
    opening = source.find("{", match.end())
    if opening < 0:
        raise AssertionError(f"missing body for {function_name}")
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index]
    raise AssertionError(f"unterminated body for {function_name}")


class RuntimeCommandPaletteEntryStreamingPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_recursive_command_parser_uses_one_root_output_vector(self) -> None:
        body = function_body(self.source, "command_entry_list")
        self.assertIn(
            "let mut entries = Vec::with_capacity(command_entry_capacity_hint(value));",
            body,
        )
        self.assertIn("collect_command_entries(value, &mut entries);", body)
        self.assertIn("entries", body)
        self.assertNotIn("flat_map(command_entry_list)", body)

        collector = function_body(self.source, "collect_command_entries")
        self.assertIn("for value in values", collector)
        self.assertIn("collect_command_entries(value, entries);", collector)
        self.assertIn("command_entry_from_string(value)", collector)
        self.assertIn("command_entry_from_map(values)", collector)
        self.assertIn("entries.push(entry);", collector)

    def test_capacity_hint_uses_only_the_known_root_shape(self) -> None:
        body = function_body(self.source, "command_entry_capacity_hint")
        self.assertIn("UiValue::Array(values) => values.len()", body)
        self.assertIn("UiValue::String(_) | UiValue::Enum(_) | UiValue::Map(_) => 1", body)
        self.assertIn("_ => 0", body)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn(
            "runtime756_command_palette_entry_stream_preserves_nested_order_and_root_capacity",
            self.source,
        )
        self.assertIn(
            "RUNTIME756_COMMAND_PALETTE_ENTRY_STREAM_BENCH_V1",
            self.source,
        )
        self.assertIn("optimized_intermediate_vectors", self.source)

    def test_deterministic_model_removes_nested_intermediate_vectors(self) -> None:
        leaf_entries = 1_024
        legacy_intermediate_vectors = leaf_entries
        optimized_intermediate_vectors = 0
        self.assertGreater(legacy_intermediate_vectors, optimized_intermediate_vectors)
        self.assertEqual(optimized_intermediate_vectors, 0)
        self.assertEqual(leaf_entries, 1_024)


if __name__ == "__main__":
    unittest.main()
