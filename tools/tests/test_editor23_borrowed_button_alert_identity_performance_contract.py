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


class EditorBorrowedButtonAlertIdentityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.button_identity = (
            PAINT_NODES / "template_buttons" / "identity.rs"
        ).read_text(encoding="utf-8")
        cls.button_glyph = (
            PAINT_NODES / "template_buttons" / "content" / "glyph.rs"
        ).read_text(encoding="utf-8")
        cls.button_tests = (
            PAINT_NODES / "template_buttons_tests" / "identity.rs"
        ).read_text(encoding="utf-8")
        cls.alert_variants = (
            PAINT_NODES
            / "material_primitives"
            / "alert"
            / "style"
            / "variants.rs"
        ).read_text(encoding="utf-8")

    def test_button_identity_is_borrowed_and_allocation_free(self) -> None:
        self.assertIn("fn button_identity_values", self.button_identity)
        self.assertIn("fn button_identity_contains", self.button_identity)
        self.assertIn("windows(", self.button_identity)
        self.assertIn("eq_ignore_ascii_case", self.button_identity)
        self.assertNotIn("fn button_key", self.button_identity)
        self.assertNotIn("format!", self.button_identity)
        self.assertNotIn("to_ascii_lowercase", self.button_identity)

    def test_button_kind_and_glyph_reuse_borrowed_identity(self) -> None:
        kind = function_body(self.button_identity, "button_kind")
        glyph = function_body(self.button_glyph, "button_glyph")
        self.assertIn("button_identity_values(node)", kind)
        self.assertIn("button_identity_values(node)", glyph)
        self.assertNotIn("button_key", glyph)
        self.assertIn("mixed_case_button_identity_preserves_kind_and_glyph", self.button_tests)

    def test_button_glyph_no_longer_uses_the_owned_key_api(self) -> None:
        self.assertNotIn("button_glyph_for_key", self.button_glyph)
        self.assertNotIn("button_key", self.button_glyph)

    def test_alert_color_uses_static_variant_pairs_without_formatting(self) -> None:
        body = function_body(self.alert_variants, "alert_color_token")
        self.assertIn('(\"success\", \"colorSuccess\")', body)
        self.assertIn('(\"danger\", \"colorDanger\")', body)
        self.assertNotIn("format!", body)
        self.assertNotIn("pascal_case", self.alert_variants)
        self.assertIn(
            "mixed_case_material_alert_variant_preserves_color_precedence",
            self.alert_variants,
        )


if __name__ == "__main__":
    unittest.main()
