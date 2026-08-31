from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
BBCODE = ROOT / "zircon_runtime/src/text/rich/bbcode.rs"


def source_text() -> str:
    return BBCODE.read_text(encoding="utf-8")


def normalized_tag_region() -> str:
    source = source_text()
    offset = source.index("pub(super) fn normalized_tag(")
    return source[offset : source.index("fn is_valid_tag_name(", offset)]


def valid_tag_name_region() -> str:
    source = source_text()
    offset = source.index("fn is_valid_tag_name(")
    return source[offset : source.index("fn unquoted(", offset)]


class Runtime84ValidateBeforeNormalizingBbCodeTagPerformanceContractTests(
    unittest.TestCase
):
    def test_borrowed_tag_validation_precedes_owned_normalization(self) -> None:
        normalized = normalized_tag_region()

        validation = normalized.index("is_valid_tag_name(tag)")
        allocation = normalized.index("tag.to_ascii_lowercase()")
        self.assertLess(validation, allocation)
        self.assertIn("let tag = tag.trim();", normalized)
        self.assertIn("return None;", normalized)

        validator = valid_tag_name_region()
        self.assertIn(".bytes()", validator)
        self.assertIn("byte.is_ascii_alphanumeric()", validator)

    def test_invalid_path_does_not_allocate_a_lowercase_string(self) -> None:
        normalized = normalized_tag_region()

        self.assertNotIn("tag.trim().to_ascii_lowercase()", normalized)
        self.assertEqual(normalized.count("to_ascii_lowercase()"), 1)
        self.assertIn("Some(tag.to_ascii_lowercase())", normalized)

    def test_valid_and_invalid_tag_semantics_are_covered_by_rust(self) -> None:
        source = BBCODE.read_text(encoding="utf-8")

        self.assertIn("fn normalized_tag_trims_and_folds_valid_ascii()", source)
        self.assertIn("fn normalized_tag_rejects_invalid_or_non_ascii_names()", source)


if __name__ == "__main__":
    unittest.main()
