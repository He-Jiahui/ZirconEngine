from __future__ import annotations

import unittest

from tools import zircon_build_shader_prewarm_written_variants as written_variants


class _CountingMapping(dict[str, object]):
    def __init__(self, values: dict[str, object]) -> None:
        super().__init__(values)
        self.read_count = 0

    def get(self, key, default=None):
        self.read_count += 1
        return super().get(key, default)

    def __getitem__(self, key):
        self.read_count += 1
        return super().__getitem__(key)


def _raw_variant(index: int) -> dict[str, object]:
    return {
        "cache_hash": f"{index:064x}",
        "canonical_string": f"shader_variant_v1|index={index}",
        "source_label": f"res://shaders/source-{index}.wgsl",
        "template_revision": "template-v1",
        "naga_version": "naga-v1",
        "wgpu_version": "wgpu-v1",
    }


def _reported_variant(index: int) -> written_variants.ReportedWrittenVariant:
    raw = _raw_variant(index)
    return written_variants.ReportedWrittenVariant(
        cache_hash=str(raw["cache_hash"]),
        canonical_string=str(raw["canonical_string"]),
        source_label=str(raw["source_label"]),
        template_revision=str(raw["template_revision"]),
        naga_version=str(raw["naga_version"]),
        wgpu_version=str(raw["wgpu_version"]),
    )


class Tooling08ShaderWrittenVariantPerformanceContractTests(unittest.TestCase):
    def test_reported_variant_uses_fixed_slots(self) -> None:
        self.assertFalse(hasattr(_reported_variant(1), "__dict__"))

    def test_report_parser_reads_each_required_field_once(self) -> None:
        raw_variant = _CountingMapping(_raw_variant(1))

        parsed = written_variants.reported_written_variants(
            {"written_variants": [raw_variant]}
        )

        self.assertEqual(1, len(parsed or ()))
        self.assertEqual(6, raw_variant.read_count)

    def test_public_unique_identity_validator_keeps_duplicate_diagnostics(self) -> None:
        first = _reported_variant(1)
        duplicate = _reported_variant(1)

        with self.assertRaisesRegex(
            RuntimeError,
            "duplicate written cache variant identity: cache_hash=",
        ):
            written_variants.validate_unique_written_variant_identity(
                (first, duplicate)
            )


if __name__ == "__main__":
    unittest.main()
