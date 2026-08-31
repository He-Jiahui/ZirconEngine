import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PAINT_NODES = (
    REPO_ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "retained_host"
    / "host_contract"
    / "paint_template_nodes"
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


class EditorBorrowedPaintIdentityClassificationPerformanceContractTests(
    unittest.TestCase
):
    @classmethod
    def setUpClass(cls) -> None:
        cls.search = (PAINT_NODES / "template_fields" / "search.rs").read_text(
            encoding="utf-8"
        )
        cls.field_tests = (
            PAINT_NODES / "template_fields_tests" / "identity.rs"
        ).read_text(encoding="utf-8")
        cls.alert = (PAINT_NODES / "template_alerts" / "identity.rs").read_text(
            encoding="utf-8"
        )
        cls.alert_tests = (
            PAINT_NODES / "template_alerts_tests" / "identity.rs"
        ).read_text(encoding="utf-8")
        cls.danger = (
            PAINT_NODES
            / "style_selector"
            / "workbench_icon_button"
            / "selection"
            / "danger.rs"
        ).read_text(encoding="utf-8")
        cls.icon_tests = (
            PAINT_NODES / "style_selector" / "workbench_icon_button" / "tests.rs"
        ).read_text(encoding="utf-8")

    def test_all_three_classifiers_use_borrowed_ascii_windows(self) -> None:
        for source in (self.search, self.alert, self.danger):
            self.assertIn("windows(", source)
            self.assertIn("eq_ignore_ascii_case", source)
            self.assertNotIn("to_ascii_lowercase", source)

    def test_search_identity_scans_borrowed_fields_and_preserves_mixed_case(self) -> None:
        body = function_body(self.search, "search_identity_text")
        self.assertIn('contains_ignore_ascii_case(value, "search")', body)
        self.assertIn("mixed_case_search_identity_remains_search_field", self.field_tests)

    def test_alert_tone_scans_borrowed_severity_and_preserves_mixed_case(self) -> None:
        body = function_body(self.alert, "tone_from_key")
        self.assertIn('contains_ignore_ascii_case(key, "warning")', body)
        self.assertIn("mixed_case_alert_severity_preserves_tone", self.alert_tests)

    def test_danger_identity_avoids_compound_string_and_preserves_mixed_case(self) -> None:
        body = function_body(self.danger, "is_danger_icon")
        self.assertNotIn("format!", body)
        self.assertIn("identity_values", body)
        self.assertIn("mixed_case_danger_identity_preserves_style", self.icon_tests)


if __name__ == "__main__":
    unittest.main()
