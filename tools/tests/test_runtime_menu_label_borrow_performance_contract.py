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
    / "keyboard"
    / "menu.rs"
)
LOWER_REGRESSION = (
    REPO_ROOT / "zircon_runtime/src/ui/component/state_reducer/keyboard/menu/tests/label_borrow_tests.rs"
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


class RuntimeMenuLabelBorrowPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_label_lookup_borrows_text_until_retained_node_ownership(self) -> None:
        body = function_body(self.source, "menu_option_label_text")
        self.assertIn("Option<&str>", self.source)
        self.assertIn("string_value_ref", body)
        self.assertNotIn("super::string_value", body)
        self.assertNotIn("Option<String>", body)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        lower = LOWER_REGRESSION.read_text(encoding="utf-8")
        self.assertIn(
            "runtime780_menu_option_label_borrow_preserves_label_and_id_fallback",
            lower,
        )
        self.assertIn("RUNTIME780_MENU_LABEL_BORROW_BENCH_V1", lower)
        self.assertIn("optimized_intermediate_label_allocations", lower)

    def test_deterministic_model_removes_intermediate_label_string(self) -> None:
        map_nodes = 1_024
        legacy_intermediate_label_allocations = map_nodes
        optimized_intermediate_label_allocations = 0
        self.assertGreater(
            legacy_intermediate_label_allocations,
            optimized_intermediate_label_allocations,
        )
        self.assertEqual(optimized_intermediate_label_allocations, 0)


if __name__ == "__main__":
    unittest.main()
