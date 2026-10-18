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


class RuntimeMenuSearchQueryBorrowPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_query_normalizes_from_a_borrowed_nonempty_setting(self) -> None:
        body = function_body(self.source, "search_query")
        self.assertIn("nonempty_string_setting_ref", body)
        self.assertIn("let query = query.trim();", body)
        self.assertIn("query.to_lowercase()", body)
        self.assertNotIn("super::string_setting", body)

    def test_nonempty_borrowed_lookup_preserves_setting_fallback_shape(self) -> None:
        body = function_body(self.source, "nonempty_string_setting_ref")
        self.assertIn("state", body)
        self.assertIn("descriptor", body)
        self.assertIn(".or_else", body)
        self.assertIn("nonempty_string_value_ref", body)


if __name__ == "__main__":
    unittest.main()
