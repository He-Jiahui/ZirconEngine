from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
HALF_RES = ROOT / (
    "zircon_runtime/src/graphics/pipeline/render_pipeline_asset/"
    "half_resolution_transparency.rs"
)


def function_region(source: str, start: str, end: str) -> str:
    offset = source.index(start)
    return source[offset : source.index(end, offset)]


class Runtime89StreamedHalfResOwnerPerformanceContractTests(unittest.TestCase):
    def test_owner_selection_streams_only_the_first_two_matches(self) -> None:
        source = HALF_RES.read_text(encoding="utf-8")
        replace = function_region(
            source,
            "fn replace_with_half_resolution_transparent_mesh_pass(",
            "#[cfg(test)]",
        )

        self.assertIn("let mut owners =", replace)
        self.assertIn("descriptors\n                .iter()", replace)
        self.assertIn("owners.next()", replace)
        self.assertIn("if owners.next().is_some()", replace)
        self.assertNotIn("collect::<Vec<_>>()", replace)
        self.assertNotIn("match owners.as_slice()", replace)

    def test_unique_owner_is_cloned_only_after_duplicate_check(self) -> None:
        source = HALF_RES.read_text(encoding="utf-8")
        replace = function_region(
            source,
            "fn replace_with_half_resolution_transparent_mesh_pass(",
            "#[cfg(test)]",
        )
        duplicate_check = replace.index("if owners.next().is_some()")
        template_clone = replace.index("transparent_template.clone()")

        self.assertLess(duplicate_check, template_clone)
        self.assertEqual(replace.count("transparent_template.clone()"), 1)

    def test_zero_unique_and_duplicate_owner_semantics_are_covered_by_rust(self) -> None:
        source = HALF_RES.read_text(encoding="utf-8")

        self.assertIn(
            "fn streamed_half_resolution_owner_selection_preserves_cardinality_contract()",
            source,
        )
        self.assertIn("assert!(!replace_with_half_resolution_transparent_mesh_pass(&mut empty)", source)
        self.assertIn("assert!(replace_with_half_resolution_transparent_mesh_pass(&mut unique)", source)
        self.assertIn("assert!(duplicate_error.contains(\"exactly one\"));", source)


if __name__ == "__main__":
    unittest.main()
