from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
ASSET_REGISTRATION = ROOT / "zircon_runtime/src/text/font/asset_registration.rs"


def function_region(source: str, start: str, end: str) -> str:
    offset = source.index(start)
    return source[offset : source.index(end, offset)]


class Runtime80AllocationFreePrimaryFamilyMatchPerformanceContractTests(
    unittest.TestCase
):
    def test_family_match_trims_and_folds_ascii_without_allocating(self) -> None:
        source = ASSET_REGISTRATION.read_text(encoding="utf-8")
        family_match = function_region(
            source,
            "fn normalized_family_matches(",
            "fn descriptor_from_font_asset_member(",
        )

        self.assertIn("left.trim()", family_match)
        self.assertIn("eq_ignore_ascii_case(right.trim())", family_match)
        self.assertNotIn("to_ascii_lowercase", family_match)
        self.assertNotIn("normalized_family_key", family_match)

    def test_primary_member_scan_uses_the_borrowed_family_match(self) -> None:
        source = ASSET_REGISTRATION.read_text(encoding="utf-8")
        primary = function_region(
            source,
            "fn primary_family_member_index(",
            "fn normalized_family_matches(",
        )

        self.assertEqual(primary.count("normalized_family_matches("), 1)
        self.assertNotIn("normalized_family_key(member.family.as_str())", primary)
        self.assertNotIn("normalized_family_key(family)", primary)

    def test_ascii_and_non_ascii_semantics_are_covered_by_rust(self) -> None:
        source = ASSET_REGISTRATION.read_text(encoding="utf-8")

        self.assertIn(
            "fn runtime80_batch_primary_family_match_trims_and_folds_ascii_case()",
            source,
        )
        self.assertIn(
            "fn runtime80_batch_primary_family_match_keeps_non_ascii_case_strict()",
            source,
        )


if __name__ == "__main__":
    unittest.main()
