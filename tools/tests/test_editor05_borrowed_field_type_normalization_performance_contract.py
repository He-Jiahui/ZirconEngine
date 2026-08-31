import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
FIELD_EDITOR = (
    REPO_ROOT
    / "zircon_editor"
    / "src"
    / "core"
    / "extension"
    / "inspector"
    / "field_editor.rs"
)


def function_body(source: str, function_name: str) -> str:
    match = re.search(rf"\bfn\s+{re.escape(function_name)}\s*\(", source)
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


class EditorFieldTypeNormalizationPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = FIELD_EDITOR.read_text(encoding="utf-8")

    def test_field_type_normalization_uses_borrowed_ascii_matching(self) -> None:
        body = function_body(self.source, "normalize_field_type_name")
        self.assertNotIn("to_ascii_lowercase", body)
        self.assertIn("eq_ignore_ascii_case", body)
        self.assertIn("ends_with_ignore_ascii_case", body)
        self.assertIn("contains_ignore_ascii_case", body)

    def test_builtin_alias_admission_does_not_allocate_lowercase_strings(self) -> None:
        body = function_body(self.source, "is_builtin_field_editor_alias")
        self.assertNotIn("to_ascii_lowercase", body)
        self.assertIn("eq_ignore_ascii_case", body)
        self.assertIn("NUMBER_FIELD_TYPE_ALIASES", body)

    def test_rust_regression_covers_aliases_suffixes_and_qualified_types(self) -> None:
        self.assertIn(
            "field_type_normalization_borrows_ascii_aliases_and_preserves_qualified_types",
            self.source,
        )


if __name__ == "__main__":
    unittest.main()
