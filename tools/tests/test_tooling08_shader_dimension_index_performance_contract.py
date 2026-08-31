from __future__ import annotations

import unittest
from types import SimpleNamespace

from tools import zircon_build_shader_prewarm_cache_artifacts as cache_artifacts


class _CountingCanonicalString(str):
    split_count = 0

    def split(self, sep=None, maxsplit=-1):
        type(self).split_count += 1
        return super().split(sep, maxsplit)


class Tooling08ShaderDimensionIndexPerformanceContractTests(unittest.TestCase):
    def test_no_expected_dimensions_skip_canonical_parsing(self) -> None:
        variants = (
            SimpleNamespace(
                canonical_string=_CountingCanonicalString(
                    "shader_variant_v1|pass=forward"
                )
            ),
        )
        _CountingCanonicalString.split_count = 0

        cache_artifacts._validate_expected_written_variant_dimensions(
            variants,
            expected_pass_types=(),
            expected_quality_tiers=(),
            expected_geometry_sources=(),
            expected_geometry_source_ids=(),
            expected_shading_model_ids=(),
        )

        self.assertEqual(0, _CountingCanonicalString.split_count)

    def test_each_written_variant_canonical_string_is_parsed_once(self) -> None:
        variants = tuple(
            SimpleNamespace(
                canonical_string=_CountingCanonicalString(
                    "shader_variant_v1"
                    f"|pass=p{pass_index}"
                    f"|quality=q{quality_index}"
                    f"|geometry={geometry_index}"
                    f"|shading={10 + shading_index}"
                )
            )
            for pass_index in range(2)
            for quality_index in range(2)
            for geometry_index in range(2)
            for shading_index in range(2)
        )
        _CountingCanonicalString.split_count = 0

        cache_artifacts._validate_expected_written_variant_dimensions(
            variants,
            expected_pass_types=("p0", "p1"),
            expected_quality_tiers=("q0", "q1"),
            expected_geometry_sources=("0", "1"),
            expected_geometry_source_ids=("custom:g0=0", "custom:g1=1"),
            expected_shading_model_ids=("custom:s0=10", "custom:s1=11"),
        )

        self.assertEqual(len(variants), _CountingCanonicalString.split_count)

    def test_duplicate_fields_keep_last_value_for_combination_matching(self) -> None:
        variants = (
            SimpleNamespace(
                canonical_string=(
                    "shader_variant_v1|pass=forward|pass=shadow"
                    "|quality=high|geometry=0|shading=10"
                )
            ),
        )

        with self.assertRaisesRegex(RuntimeError, "variant combinations"):
            cache_artifacts._validate_expected_written_variant_dimensions(
                variants,
                expected_pass_types=("forward",),
                expected_quality_tiers=("high",),
                expected_geometry_sources=("static",),
                expected_geometry_source_ids=(),
                expected_shading_model_ids=(),
            )

    def test_dimension_index_preserves_structure_guard_helpers(self) -> None:
        source = cache_artifacts.Path(cache_artifacts.__file__).read_text(
            encoding="utf-8"
        )
        self.assertIn("class _WrittenVariantDimensionIndex:", source)
        self.assertIn("def _canonical_has_dimension_value(", source)
        self.assertIn("def _custom_id_combination_matches(", source)


if __name__ == "__main__":
    unittest.main()
