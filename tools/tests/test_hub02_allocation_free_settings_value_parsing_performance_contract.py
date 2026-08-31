from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
HUB_CONFIG = ROOT / "zircon_hub/src/settings/hub_config.rs"


def function_body(source: str, owner: str, signature: str) -> str:
    owner_start = source.index(owner)
    start = source.index(signature, owner_start)
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
    raise AssertionError(f"unterminated function: {owner}::{signature}")


class Hub02AllocationFreeSettingsValueParsingPerformanceContractTests(
    unittest.TestCase
):
    def test_build_profile_parser_borrows_the_trimmed_value(self) -> None:
        source = HUB_CONFIG.read_text(encoding="utf-8")
        parser = function_body(
            source,
            "impl BuildProfile {",
            "pub fn from_ui_value(value: &str)",
        )

        self.assertIn("let value = value.trim();", parser)
        self.assertIn('value.eq_ignore_ascii_case("debug")', parser)
        self.assertIn('value.eq_ignore_ascii_case("release")', parser)
        self.assertEqual(parser.count("eq_ignore_ascii_case("), 2)
        self.assertNotIn("to_ascii_lowercase", parser)
        self.assertNotIn("String::", parser)

    def test_language_parser_borrows_all_static_aliases(self) -> None:
        source = HUB_CONFIG.read_text(encoding="utf-8")
        parser = function_body(
            source,
            "impl HubLanguage {",
            "pub fn from_ui_value(value: &str)",
        )

        self.assertIn("let value = value.trim();", parser)
        for alias in ("english", "en", "chinese", "zh", "cn"):
            self.assertIn(f'value.eq_ignore_ascii_case("{alias}")', parser)
        self.assertEqual(parser.count("eq_ignore_ascii_case("), 5)
        self.assertNotIn("to_ascii_lowercase", parser)
        self.assertNotIn("String::", parser)

    def test_ascii_and_non_ascii_semantics_are_covered_by_rust(self) -> None:
        source = HUB_CONFIG.read_text(encoding="utf-8")

        self.assertIn('BuildProfile::from_ui_value(" RELEASE ")', source)
        self.assertIn('HubLanguage::from_ui_value(" ENGLISH ")', source)
        self.assertIn('HubLanguage::from_ui_value("\\u{00c9}NGLISH")', source)


if __name__ == "__main__":
    unittest.main()
