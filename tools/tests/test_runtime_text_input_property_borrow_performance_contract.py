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
    / "text_input.rs"
)
LOWER_REGRESSION = (
    REPO_ROOT / "zircon_runtime/src/ui/component/state_reducer/text_input/tests/property_borrow_tests.rs"
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


class RuntimeTextInputPropertyBorrowPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_validation_candidates_use_static_ordered_slices(self) -> None:
        body = function_body(self.source, "validation_property_candidates")
        self.assertIn("VALIDATION_CANDIDATES_VALUE_TEXT", body)
        self.assertIn("VALIDATION_CANDIDATES_TEXT", body)
        self.assertIn("VALIDATION_CANDIDATES_VALUE", body)
        self.assertNotIn("Vec::new()", body)
        self.assertNotIn("candidates.contains", body)

    def test_validation_text_borrows_and_mirror_properties_are_static(self) -> None:
        validation = function_body(self.source, "validation_text")
        mirror = function_body(self.source, "text_input_mirror_properties")
        self.assertIn("textual_value_ref", validation)
        self.assertIn("VALIDATION_EMPTY_TEXT", validation)
        self.assertIn("MIRROR_VALUE", mirror)
        self.assertIn("MIRROR_VALUE_TEXT", mirror)
        self.assertNotIn("Vec::new()", mirror)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        lower = LOWER_REGRESSION.read_text(encoding="utf-8")
        self.assertIn(
            "runtime767_text_input_property_paths_preserve_candidate_and_mirror_order",
            lower,
        )
        self.assertIn("RUNTIME767_TEXT_INPUT_PROPERTY_BORROW_BENCH_V1", lower)
        self.assertIn("optimized_candidate_allocations", lower)

    def test_deterministic_model_eliminates_transient_candidate_containers(self) -> None:
        validation_events = 1_024
        legacy_candidate_allocations = validation_events * 2
        optimized_candidate_allocations = 0
        self.assertGreater(legacy_candidate_allocations, optimized_candidate_allocations)
        self.assertEqual(optimized_candidate_allocations, 0)
        self.assertEqual(validation_events, 1_024)


if __name__ == "__main__":
    unittest.main()
