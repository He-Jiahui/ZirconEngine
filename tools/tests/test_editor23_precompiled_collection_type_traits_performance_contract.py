import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
COLLECTION_FIELDS = (
    REPO_ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "retained_host"
    / "ui"
    / "pane_data_conversion"
    / "pane_component_projection"
    / "collection_fields"
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


class EditorCollectionTypeTraitsPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.type_tokens = (COLLECTION_FIELDS / "type_tokens.rs").read_text(
            encoding="utf-8"
        )
        cls.roles = (COLLECTION_FIELDS / "roles.rs").read_text(encoding="utf-8")
        cls.validation = (COLLECTION_FIELDS / "validation.rs").read_text(
            encoding="utf-8"
        )
        cls.array = (COLLECTION_FIELDS / "array.rs").read_text(encoding="utf-8")
        cls.map = (COLLECTION_FIELDS / "map.rs").read_text(encoding="utf-8")
        cls.tests = (COLLECTION_FIELDS / "tests.rs").read_text(encoding="utf-8")

    def test_declared_type_traits_use_borrowed_ascii_matching(self) -> None:
        self.assertIn("struct CollectionTypeTraits", self.type_tokens)
        self.assertIn("fn from_declared_type", self.type_tokens)
        self.assertIn("eq_ignore_ascii_case", self.type_tokens)
        self.assertIn("windows(", self.type_tokens)
        self.assertNotIn("to_ascii_lowercase", self.type_tokens)
        self.assertNotIn("to_ascii_lowercase", self.roles)
        self.assertNotIn("to_ascii_lowercase", self.validation)

    def test_array_projection_compiles_traits_outside_the_row_loop(self) -> None:
        body = function_body(self.array, "array_collection_fields")
        self.assertEqual(body.count("CollectionTypeTraits::from_declared_type"), 1)
        row_loop = body[body.index(".map(|(index, value)|") :]
        compact = re.sub(r"\s+", "", row_loop)
        self.assertNotIn("from_declared_type", row_loop)
        self.assertIn("collection_field_role(element_traits", compact)
        self.assertIn(
            "collection_value_validation(&element_type,element_traits,", compact
        )

    def test_map_projection_reuses_key_and_value_traits_for_every_row(self) -> None:
        body = function_body(self.map, "map_collection_fields")
        self.assertEqual(body.count("CollectionTypeTraits::from_declared_type"), 2)
        row_loop = body[body.index(".map(|(key, value)|") :]
        compact = re.sub(r"\s+", "", row_loop)
        self.assertNotIn("from_declared_type", row_loop)
        self.assertIn("collection_field_role(key_traits", compact)
        self.assertIn("collection_field_role(value_traits", compact)
        self.assertIn(
            "collection_map_entry_validation(&key_type,key_traits,", compact
        )

    def test_rust_regression_covers_mixed_case_traits_and_row_semantics(self) -> None:
        self.assertIn(
            "collection_type_traits_are_case_insensitive_and_reused_per_projection",
            self.tests,
        )


if __name__ == "__main__":
    unittest.main()
