from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
CREATE_PROJECT_REQUEST = ROOT / "zircon_hub/src/projects/create_project_request.rs"


def function_body(source: str, signature: str) -> str:
    start = source.index(signature)
    opening = source.index("{", start)
    depth = 0
    for offset in range(opening, len(source)):
        character = source[offset]
        if character == "{":
            depth += 1
        elif character == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : offset]
    raise AssertionError(f"unterminated function: {signature}")


class Hub01AllocationFreeTemplateIdMatchPerformanceContractTests(unittest.TestCase):
    def test_enabled_template_match_borrows_the_trimmed_id(self) -> None:
        source = CREATE_PROJECT_REQUEST.read_text(encoding="utf-8")
        matcher = function_body(source, "pub fn from_enabled_id(id: &str)")

        self.assertIn("id.trim()", matcher)
        self.assertIn('eq_ignore_ascii_case("renderable-empty")', matcher)
        self.assertNotIn("to_ascii_lowercase", matcher)
        self.assertNotIn("to_string", matcher)

    def test_match_maps_only_the_enabled_template_without_a_temporary_string(self) -> None:
        source = CREATE_PROJECT_REQUEST.read_text(encoding="utf-8")
        matcher = function_body(source, "pub fn from_enabled_id(id: &str)")

        self.assertEqual(matcher.count("eq_ignore_ascii_case("), 1)
        self.assertIn("then_some(Self::RenderableEmpty)", matcher)
        self.assertNotIn("String::", matcher)

    def test_ascii_and_non_ascii_semantics_are_covered_by_rust(self) -> None:
        source = CREATE_PROJECT_REQUEST.read_text(encoding="utf-8")

        self.assertIn(
            "fn enabled_template_id_match_trims_and_folds_ascii_case()", source
        )
        self.assertIn(
            "fn enabled_template_id_match_keeps_non_ascii_case_strict()", source
        )


if __name__ == "__main__":
    unittest.main()
