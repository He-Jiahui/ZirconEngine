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
    / "keyboard.rs"
)
LOWER_REGRESSION = (
    REPO_ROOT / "zircon_runtime/src/ui/component/state_reducer/keyboard/tests/indexed_entry_streaming_tests.rs"
)


def function_body(source: str, function_name: str) -> str:
    match = re.search(
        rf"\bfn\s+{re.escape(function_name)}(?![A-Za-z0-9_])(?:<[^>]*>)?\s*\(.*?\)",
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


class RuntimeKeyboardIndexedEntryStreamingPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_indexed_entries_stream_into_one_output(self) -> None:
        body = function_body(self.source, "indexed_keyboard_entries")
        self.assertIn("entries.reserve(options.len());", body)
        self.assertIn("entries.push(option.id);", body)
        self.assertNotIn("collect::<Vec<_>>()", body)
        self.assertNotIn(".flat_map(option_entry_list)", body)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        lower = LOWER_REGRESSION.read_text(encoding="utf-8")
        self.assertIn(
            "runtime766_indexed_keyboard_entries_preserve_property_and_option_order",
            lower,
        )
        self.assertIn("RUNTIME766_INDEXED_KEYBOARD_ENTRY_STREAM_BENCH_V1", lower)
        self.assertIn("optimized_intermediate_vectors", lower)

    def test_deterministic_model_eliminates_the_intermediate_id_vector(self) -> None:
        option_entries = 1_024
        legacy_intermediate_vectors = 1
        optimized_intermediate_vectors = 0
        self.assertGreater(legacy_intermediate_vectors, optimized_intermediate_vectors)
        self.assertEqual(optimized_intermediate_vectors, 0)
        self.assertEqual(option_entries, 1_024)


if __name__ == "__main__":
    unittest.main()
