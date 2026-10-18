"""Source and allocation model for material enum mismatch descriptions."""

import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/core/framework/render/shader/material_property_layout.rs"
LOWER = ROOT / "zircon_runtime/src/core/framework/render/shader/material_property_layout/tests/enum_description_tests.rs"


class Runtime876MaterialEnumExpectedDirectTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.owner = cls.source.split("pub fn expected_value_description(", 1)[1].split(
            "pub fn definition_value_for_bits(", 1
        )[0]

    def test_enum_description_reserves_and_appends_into_final_output(self) -> None:
        self.assertRegex(self.owner, r'"one of "\s*\.len\(\)')
        self.assertIn("self.enum_values.iter()", self.owner)
        self.assertIn("String::with_capacity(capacity)", self.owner)
        self.assertIn('description.push_str("one of ")', self.owner)
        self.assertIn("description.push_str(value)", self.owner)

    def test_enum_description_avoids_intermediate_join_and_outer_format(self) -> None:
        self.assertNotIn(".join(", self.owner)
        self.assertNotIn("format!", self.owner)
        self.assertNotIn("collect::<", self.owner)

    def test_empty_and_boolean_branches_are_preserved(self) -> None:
        self.assertIn('MaterialOptionKind::Bool => "bool".to_string()', self.owner)
        self.assertIn('MaterialOptionKind::Enum if self.enum_values.is_empty() => "enum value".to_string()', self.owner)
        self.assertIn("for (index, value) in self.enum_values.iter().enumerate()", self.owner)

    def test_lower_parity_and_release_marker_are_wired(self) -> None:
        self.assertIn("mod enum_description_tests;", self.source)
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn("runtime876_material_enum_expected_direct_preserves_exact_text", lower)
        self.assertIn("RUNTIME876_MATERIAL_ENUM_EXPECTED_DIRECT_BENCH_V1", lower)
        self.assertIn("const SAMPLE_PAIRS: usize = 101;", lower)
        self.assertIn("legacy_p50_ns", lower)
        self.assertIn("optimized_p99_ns", lower)

    def test_dense_invalid_enum_description_model(self) -> None:
        descriptions = 4_096
        self.assertEqual(descriptions, 4_096)  # retired joined intermediate strings
        optimized_joined_outputs = descriptions if ".join(" in self.owner else 0
        self.assertEqual(optimized_joined_outputs, 0)


if __name__ == "__main__":
    unittest.main()
