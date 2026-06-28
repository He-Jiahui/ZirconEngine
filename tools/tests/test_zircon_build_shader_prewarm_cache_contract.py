import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_build_shader_prewarm_cache_artifacts import (
    validate_shader_prewarm_cache_artifact_contract,
)


class ZirconBuildShaderPrewarmCacheContractTests(unittest.TestCase):
    def test_validate_cache_artifact_contract_requires_written_cache_pairs(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            report_path = _write_report(
                root,
                written_count=2,
                include_written_variants=False,
            )
            cache_root = root / "shader_variants"

            with self.assertRaisesRegex(
                RuntimeError,
                "cache artifacts do not cover written variants",
            ):
                validate_shader_prewarm_cache_artifact_contract(
                    cache_root,
                    report_path=report_path,
                )

    def test_validate_cache_artifact_contract_rejects_orphan_wgsl_artifacts(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            report_path = _write_report(root, written_count=1)
            shard = root / "shader_variants" / "v1" / _HASH_A[:2]
            shard.mkdir(parents=True)
            (shard / f"{_HASH_A}.wgsl.zst").write_bytes(b"compressed-wgsl")

            with self.assertRaisesRegex(RuntimeError, "missing metadata"):
                validate_shader_prewarm_cache_artifact_contract(
                    root / "shader_variants",
                    report_path=report_path,
                )

    def test_validate_cache_artifact_contract_rejects_invalid_metadata(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            report_path = _write_report(root, written_count=1)
            shard = root / "shader_variants" / "v1" / _HASH_A[:2]
            shard.mkdir(parents=True)
            (shard / f"{_HASH_A}.wgsl.zst").write_bytes(b"compressed-wgsl")
            (shard / f"{_HASH_A}.meta").write_text("{ invalid json", encoding="utf-8")

            with self.assertRaisesRegex(RuntimeError, "invalid cache metadata"):
                validate_shader_prewarm_cache_artifact_contract(
                    root / "shader_variants",
                    report_path=report_path,
                )

    def test_validate_cache_artifact_contract_rejects_invalid_metadata_field_types(
        self,
    ):
        invalid_fields = (
            ("schema_version", True),
            ("canonical_string", ["shader_variant_v1"]),
            ("template_revision", 7),
            ("naga_version", ["naga-test"]),
            ("wgpu_version", {"version": "wgpu-test"}),
            ("created_unix_seconds", "1"),
            ("created_unix_seconds", True),
        )
        for field, value in invalid_fields:
            with self.subTest(field=field, value=value):
                with tempfile.TemporaryDirectory() as temp_dir:
                    root = Path(temp_dir)
                    report_path = _write_report(root, written_hashes=(_HASH_A,))
                    shard = root / "shader_variants" / "v1" / _HASH_A[:2]
                    shard.mkdir(parents=True)
                    (shard / f"{_HASH_A}.wgsl.zst").write_bytes(b"compressed-wgsl")
                    _write_meta(
                        shard / f"{_HASH_A}.meta",
                        hash_value=_HASH_A,
                        metadata_overrides={field: value},
                    )

                    with self.assertRaisesRegex(
                        RuntimeError,
                        f"invalid cache metadata.*{field}",
                    ):
                        validate_shader_prewarm_cache_artifact_contract(
                            root / "shader_variants",
                            report_path=report_path,
                        )

    def test_validate_cache_artifact_contract_rejects_metadata_hash_mismatch(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            report_path = _write_report(root, written_hashes=(_HASH_A,))
            shard = root / "shader_variants" / "v1" / _HASH_A[:2]
            shard.mkdir(parents=True)
            (shard / f"{_HASH_A}.wgsl.zst").write_bytes(b"compressed-wgsl")
            _write_meta(shard / f"{_HASH_A}.meta", hash_value=_HASH_B)

            with self.assertRaisesRegex(RuntimeError, "cache metadata hash mismatch"):
                validate_shader_prewarm_cache_artifact_contract(
                    root / "shader_variants",
                    report_path=report_path,
                )

    def test_validate_cache_artifact_contract_requires_report_written_variants(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            report_path = _write_report(root, written_hashes=(_HASH_A, _HASH_B))
            shard = root / "shader_variants" / "v1" / _HASH_A[:2]
            shard.mkdir(parents=True)
            (shard / f"{_HASH_A}.wgsl.zst").write_bytes(b"compressed-wgsl")
            _write_meta(shard / f"{_HASH_A}.meta", hash_value=_HASH_A)

            with self.assertRaisesRegex(RuntimeError, "missing reported cache variants"):
                validate_shader_prewarm_cache_artifact_contract(
                    root / "shader_variants",
                    report_path=report_path,
                )

    def test_validate_cache_artifact_contract_rejects_partial_written_variant_report(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            report_path = _write_report(root, written_count=1, written_hashes=())
            shard = root / "shader_variants" / "v1" / _HASH_A[:2]
            shard.mkdir(parents=True)
            (shard / f"{_HASH_A}.wgsl.zst").write_bytes(b"compressed-wgsl")
            _write_meta(shard / f"{_HASH_A}.meta", hash_value=_HASH_A)

            with self.assertRaisesRegex(
                RuntimeError,
                "written cache variant count does not match",
            ):
                validate_shader_prewarm_cache_artifact_contract(
                    root / "shader_variants",
                    report_path=report_path,
                )

    def test_validate_cache_artifact_contract_rejects_wrong_canonical_variant(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            report_path = _write_report(
                root,
                written_hashes=(_HASH_A,),
                canonical_prefix="shader_variant_v1|quality=high",
            )
            shard = root / "shader_variants" / "v1" / _HASH_A[:2]
            shard.mkdir(parents=True)
            (shard / f"{_HASH_A}.wgsl.zst").write_bytes(b"compressed-wgsl")
            _write_meta(
                shard / f"{_HASH_A}.meta",
                hash_value=_HASH_A,
                canonical_string="shader_variant_v1|quality=medium",
            )

            with self.assertRaisesRegex(RuntimeError, "reported cache variant mismatch"):
                validate_shader_prewarm_cache_artifact_contract(
                    root / "shader_variants",
                    report_path=report_path,
                )

    def test_validate_cache_artifact_contract_rejects_duplicate_written_variant_identity(
        self,
    ):
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            canonical_string = "shader_variant_v1|pass=forward|geometry=0|shading=0"
            report_path = _write_report_for_canonical_strings(
                root,
                (
                    (_HASH_A, canonical_string),
                    (_HASH_B, canonical_string),
                ),
            )
            shard = root / "shader_variants" / "v1" / _HASH_A[:2]
            shard.mkdir(parents=True)
            for hash_value in (_HASH_A, _HASH_B):
                (shard / f"{hash_value}.wgsl.zst").write_bytes(b"compressed-wgsl")
                _write_meta(
                    shard / f"{hash_value}.meta",
                    hash_value=hash_value,
                    canonical_string=canonical_string,
                )

            with self.assertRaisesRegex(
                RuntimeError,
                "duplicate written cache variant identity",
            ):
                validate_shader_prewarm_cache_artifact_contract(
                    root / "shader_variants",
                    report_path=report_path,
                )

    def test_validate_cache_artifact_contract_rejects_non_runtime_cache_layout(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            report_path = _write_report(root, written_hashes=(_HASH_A,))
            shard = root / "shader_variants" / "v1" / "zz"
            shard.mkdir(parents=True)
            (shard / f"{_HASH_A}.wgsl.zst").write_bytes(b"compressed-wgsl")
            _write_meta(shard / f"{_HASH_A}.meta", hash_value=_HASH_A)

            with self.assertRaisesRegex(RuntimeError, "runtime cache layout"):
                validate_shader_prewarm_cache_artifact_contract(
                    root / "shader_variants",
                    report_path=report_path,
                )

    def test_validate_cache_artifact_contract_rejects_schema_version_mismatch(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            report_path = _write_report(root, written_hashes=(_HASH_A,))
            shard = root / "shader_variants" / "v1" / _HASH_A[:2]
            shard.mkdir(parents=True)
            (shard / f"{_HASH_A}.wgsl.zst").write_bytes(b"compressed-wgsl")
            _write_meta(shard / f"{_HASH_A}.meta", hash_value=_HASH_A, schema_version=2)

            with self.assertRaisesRegex(RuntimeError, "cache metadata schema mismatch"):
                validate_shader_prewarm_cache_artifact_contract(
                    root / "shader_variants",
                    report_path=report_path,
                )

    def test_validate_cache_artifact_contract_accepts_written_cache_pairs(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            report_path = _write_report(root, written_hashes=(_HASH_A, _HASH_B))
            shard = root / "shader_variants" / "v1" / _HASH_A[:2]
            shard.mkdir(parents=True)
            for hash_value in (_HASH_A, _HASH_B):
                (shard / f"{hash_value}.wgsl.zst").write_bytes(b"compressed-wgsl")
                _write_meta(shard / f"{hash_value}.meta", hash_value=hash_value)

            validate_shader_prewarm_cache_artifact_contract(
                root / "shader_variants",
                report_path=report_path,
            )

    def test_validate_cache_artifact_contract_requires_written_variant_source_labels_in_provenance(
        self,
    ):
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            report_path = _write_report_with_source_provenance(
                root,
                written_source_label="res://materials/missing-source.wgsl",
                provenance_source_label="res://materials/source-a.wgsl",
            )
            shard = root / "shader_variants" / "v1" / _HASH_A[:2]
            shard.mkdir(parents=True)
            (shard / f"{_HASH_A}.wgsl.zst").write_bytes(b"compressed-wgsl")
            _write_meta(shard / f"{_HASH_A}.meta", hash_value=_HASH_A)

            with self.assertRaisesRegex(
                RuntimeError,
                "written variant source labels missing source provenance",
            ):
                validate_shader_prewarm_cache_artifact_contract(
                    root / "shader_variants",
                    report_path=report_path,
                )

    def test_validate_cache_artifact_contract_rejects_non_blake3_hex_cache_hash(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            report_path = _write_report(root, written_hashes=("abcdef",))
            shard = root / "shader_variants" / "v1" / "ab"
            shard.mkdir(parents=True)
            (shard / "abcdef.wgsl.zst").write_bytes(b"compressed-wgsl")
            _write_meta(shard / "abcdef.meta", hash_value="abcdef")

            with self.assertRaisesRegex(RuntimeError, "cache hash shape"):
                validate_shader_prewarm_cache_artifact_contract(
                    root / "shader_variants",
                    report_path=report_path,
                )

    def test_validate_cache_artifact_contract_requires_requested_pass_types(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            report_path = _write_report(
                root,
                written_hashes=(_HASH_A,),
                canonical_prefix="shader_variant_v1|pass=shadow",
            )
            shard = root / "shader_variants" / "v1" / _HASH_A[:2]
            shard.mkdir(parents=True)
            (shard / f"{_HASH_A}.wgsl.zst").write_bytes(b"compressed-wgsl")
            _write_meta(
                shard / f"{_HASH_A}.meta",
                hash_value=_HASH_A,
                canonical_string=f"shader_variant_v1|pass=shadow|hash={_HASH_A}",
            )

            with self.assertRaisesRegex(
                RuntimeError,
                "cache artifacts are missing requested shader pass types",
            ):
                validate_shader_prewarm_cache_artifact_contract(
                    root / "shader_variants",
                    report_path=report_path,
                    expected_pass_types=("forward",),
                )

    def test_validate_cache_artifact_contract_accepts_requested_pass_types(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            report_path = _write_report(
                root,
                written_hashes=(_HASH_A,),
                canonical_prefix="shader_variant_v1|pass=forward",
            )
            shard = root / "shader_variants" / "v1" / _HASH_A[:2]
            shard.mkdir(parents=True)
            (shard / f"{_HASH_A}.wgsl.zst").write_bytes(b"compressed-wgsl")
            _write_meta(
                shard / f"{_HASH_A}.meta",
                hash_value=_HASH_A,
                canonical_string=f"shader_variant_v1|pass=forward|hash={_HASH_A}",
            )

            validate_shader_prewarm_cache_artifact_contract(
                root / "shader_variants",
                report_path=report_path,
                expected_pass_types=("forward",),
            )

    def test_validate_cache_artifact_contract_requires_requested_quality_tiers(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            report_path = _write_report(
                root,
                written_hashes=(_HASH_A,),
                canonical_prefix="shader_variant_v1|pass=forward|quality=medium",
            )
            shard = root / "shader_variants" / "v1" / _HASH_A[:2]
            shard.mkdir(parents=True)
            (shard / f"{_HASH_A}.wgsl.zst").write_bytes(b"compressed-wgsl")
            _write_meta(
                shard / f"{_HASH_A}.meta",
                hash_value=_HASH_A,
                canonical_string=(
                    f"shader_variant_v1|pass=forward|quality=medium|hash={_HASH_A}"
                ),
            )

            with self.assertRaisesRegex(
                RuntimeError,
                "cache artifacts are missing requested shader quality tiers",
            ):
                validate_shader_prewarm_cache_artifact_contract(
                    root / "shader_variants",
                    report_path=report_path,
                    expected_quality_tiers=("high",),
                )

    def test_validate_cache_artifact_contract_requires_requested_geometry_sources(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            report_path = _write_report(
                root,
                written_hashes=(_HASH_A,),
                canonical_prefix="shader_variant_v1|pass=forward|geometry=0",
            )
            shard = root / "shader_variants" / "v1" / _HASH_A[:2]
            shard.mkdir(parents=True)
            (shard / f"{_HASH_A}.wgsl.zst").write_bytes(b"compressed-wgsl")
            _write_meta(
                shard / f"{_HASH_A}.meta",
                hash_value=_HASH_A,
                canonical_string=(
                    f"shader_variant_v1|pass=forward|geometry=0|hash={_HASH_A}"
                ),
            )

            with self.assertRaisesRegex(
                RuntimeError,
                "cache artifacts are missing requested shader geometry sources",
            ):
                validate_shader_prewarm_cache_artifact_contract(
                    root / "shader_variants",
                    report_path=report_path,
                    expected_geometry_sources=("skinned",),
                )

    def test_validate_cache_artifact_contract_accepts_requested_quality_and_geometry(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            report_path = _write_report(
                root,
                written_hashes=(_HASH_A,),
                canonical_prefix="shader_variant_v1|pass=forward|geometry=1|quality=high",
            )
            shard = root / "shader_variants" / "v1" / _HASH_A[:2]
            shard.mkdir(parents=True)
            (shard / f"{_HASH_A}.wgsl.zst").write_bytes(b"compressed-wgsl")
            _write_meta(
                shard / f"{_HASH_A}.meta",
                hash_value=_HASH_A,
                canonical_string=(
                    "shader_variant_v1|pass=forward|geometry=1|quality=high"
                    f"|hash={_HASH_A}"
                ),
            )

            validate_shader_prewarm_cache_artifact_contract(
                root / "shader_variants",
                report_path=report_path,
                expected_quality_tiers=("high",),
                expected_geometry_sources=("skinned",),
            )

    def test_validate_cache_artifact_contract_requires_requested_dimension_combinations(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            canonical_strings = (
                f"shader_variant_v1|pass=forward|geometry=0|quality=high|hash={_HASH_A}",
                f"shader_variant_v1|pass=shadow|geometry=1|quality=medium|hash={_HASH_B}",
            )
            report_path = _write_report_for_canonical_strings(
                root,
                ((_HASH_A, canonical_strings[0]), (_HASH_B, canonical_strings[1])),
            )
            shard = root / "shader_variants" / "v1" / _HASH_A[:2]
            shard.mkdir(parents=True)
            for hash_value, canonical_string in (
                (_HASH_A, canonical_strings[0]),
                (_HASH_B, canonical_strings[1]),
            ):
                (shard / f"{hash_value}.wgsl.zst").write_bytes(b"compressed-wgsl")
                _write_meta(
                    shard / f"{hash_value}.meta",
                    hash_value=hash_value,
                    canonical_string=canonical_string,
                )

            with self.assertRaisesRegex(
                RuntimeError,
                "cache artifacts are missing requested shader variant combinations",
            ):
                validate_shader_prewarm_cache_artifact_contract(
                    root / "shader_variants",
                    report_path=report_path,
                    expected_pass_types=("forward",),
                    expected_quality_tiers=("high",),
                    expected_geometry_sources=("skinned",),
                )

    def test_validate_cache_artifact_contract_requires_requested_custom_id_combinations(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            canonical_strings = (
                f"shader_variant_v1|pass=forward|geometry=0|quality=high|shading=0|hash={_HASH_A}",
                f"shader_variant_v1|pass=shadow|geometry=4|quality=medium|shading=16|hash={_HASH_B}",
            )
            report_path = _write_report_for_canonical_strings(
                root,
                ((_HASH_A, canonical_strings[0]), (_HASH_B, canonical_strings[1])),
            )
            shard = root / "shader_variants" / "v1" / _HASH_A[:2]
            shard.mkdir(parents=True)
            for hash_value, canonical_string in (
                (_HASH_A, canonical_strings[0]),
                (_HASH_B, canonical_strings[1]),
            ):
                (shard / f"{hash_value}.wgsl.zst").write_bytes(b"compressed-wgsl")
                _write_meta(
                    shard / f"{hash_value}.meta",
                    hash_value=hash_value,
                    canonical_string=canonical_string,
                )

            with self.assertRaisesRegex(
                RuntimeError,
                "cache artifacts are missing requested shader custom id combinations",
            ):
                validate_shader_prewarm_cache_artifact_contract(
                    root / "shader_variants",
                    report_path=report_path,
                    expected_pass_types=("forward",),
                    expected_quality_tiers=("high",),
                    expected_geometry_source_ids=("custom:virtual_geometry=4",),
                    expected_shading_model_ids=("custom:toon=16",),
                )

    def test_validate_cache_artifact_contract_requires_requested_custom_ids(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            report_path = _write_report(
                root,
                written_hashes=(_HASH_A,),
                canonical_prefix="shader_variant_v1|geometry=0|shading=0",
            )
            shard = root / "shader_variants" / "v1" / _HASH_A[:2]
            shard.mkdir(parents=True)
            (shard / f"{_HASH_A}.wgsl.zst").write_bytes(b"compressed-wgsl")
            _write_meta(
                shard / f"{_HASH_A}.meta",
                hash_value=_HASH_A,
                canonical_string=(
                    f"shader_variant_v1|geometry=0|shading=0|hash={_HASH_A}"
                ),
            )

            with self.assertRaisesRegex(
                RuntimeError,
                "cache artifacts are missing requested shader geometry source ids",
            ):
                validate_shader_prewarm_cache_artifact_contract(
                    root / "shader_variants",
                    report_path=report_path,
                    expected_geometry_source_ids=("custom:virtual_geometry=4",),
                    expected_shading_model_ids=("custom:toon=16",),
                )

    def test_validate_cache_artifact_contract_requires_requested_shading_ids(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            report_path = _write_report(
                root,
                written_hashes=(_HASH_A,),
                canonical_prefix="shader_variant_v1|geometry=4|shading=0",
            )
            shard = root / "shader_variants" / "v1" / _HASH_A[:2]
            shard.mkdir(parents=True)
            (shard / f"{_HASH_A}.wgsl.zst").write_bytes(b"compressed-wgsl")
            _write_meta(
                shard / f"{_HASH_A}.meta",
                hash_value=_HASH_A,
                canonical_string=(
                    f"shader_variant_v1|geometry=4|shading=0|hash={_HASH_A}"
                ),
            )

            with self.assertRaisesRegex(
                RuntimeError,
                "cache artifacts are missing requested shader shading model ids",
            ):
                validate_shader_prewarm_cache_artifact_contract(
                    root / "shader_variants",
                    report_path=report_path,
                    expected_geometry_source_ids=("custom:virtual_geometry=4",),
                    expected_shading_model_ids=("custom:toon=16",),
                )

    def test_validate_cache_artifact_contract_accepts_requested_custom_ids(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            report_path = _write_report(
                root,
                written_hashes=(_HASH_A,),
                canonical_prefix="shader_variant_v1|geometry=4|shading=16",
            )
            shard = root / "shader_variants" / "v1" / _HASH_A[:2]
            shard.mkdir(parents=True)
            (shard / f"{_HASH_A}.wgsl.zst").write_bytes(b"compressed-wgsl")
            _write_meta(
                shard / f"{_HASH_A}.meta",
                hash_value=_HASH_A,
                canonical_string=(
                    f"shader_variant_v1|geometry=4|shading=16|hash={_HASH_A}"
                ),
            )

            validate_shader_prewarm_cache_artifact_contract(
                root / "shader_variants",
                report_path=report_path,
                expected_geometry_source_ids=("custom:virtual_geometry=4",),
                expected_shading_model_ids=("custom:toon=16",),
            )


def _write_report(
    root: Path,
    *,
    written_count: int | None = None,
    written_hashes: tuple[str, ...] = (),
    canonical_prefix: str = "shader_variant_v1",
    include_written_variants: bool = True,
) -> Path:
    if written_count is None:
        written_count = len(written_hashes)
    report_path = root / "shader_variants_report.json"
    written_variants = [
        {
            "cache_hash": hash_value,
            "canonical_string": f"{canonical_prefix}|hash={hash_value}",
            "source_label": f"res://materials/{hash_value}.wgsl",
            "template_revision": "template-r1",
            "naga_version": "naga-test",
            "wgpu_version": "wgpu-test",
        }
        for hash_value in written_hashes
    ]
    report = {
        "requested_count": written_count,
        "written_count": written_count,
        "failed_count": 0,
    }
    if include_written_variants:
        report["written_variants"] = written_variants
    report_path.write_text(json.dumps(report), encoding="utf-8")
    return report_path


def _write_report_for_canonical_strings(
    root: Path,
    written_variants: tuple[tuple[str, str], ...],
) -> Path:
    report_path = root / "shader_variants_report.json"
    report = {
        "requested_count": len(written_variants),
        "written_count": len(written_variants),
        "failed_count": 0,
        "written_variants": [
            {
                "cache_hash": hash_value,
                "canonical_string": canonical_string,
                "source_label": f"res://materials/{hash_value}.wgsl",
                "template_revision": "template-r1",
                "naga_version": "naga-test",
                "wgpu_version": "wgpu-test",
            }
            for hash_value, canonical_string in written_variants
        ],
    }
    report_path.write_text(json.dumps(report), encoding="utf-8")
    return report_path


def _write_report_with_source_provenance(
    root: Path,
    *,
    written_source_label: str,
    provenance_source_label: str,
) -> Path:
    report_path = root / "shader_variants_report.json"
    report = {
        "requested_count": 1,
        "written_count": 1,
        "failed_count": 0,
        "written_variants": [
            {
                "cache_hash": _HASH_A,
                "canonical_string": f"shader_variant_v1|hash={_HASH_A}",
                "source_label": written_source_label,
                "template_revision": "template-r1",
                "naga_version": "naga-test",
                "wgpu_version": "wgpu-test",
            }
        ],
        "source_provenance": {
            "source_count": 1,
            "variant_count": 1,
            "sources": {
                "source-a#source-hash-a#template-r1": {
                    "source_label": provenance_source_label,
                    "source_hash": "source-hash-a",
                    "include_content_hashes": [],
                    "template_revision": "template-r1",
                    "naga_version": "naga-test",
                    "wgpu_version": "wgpu-test",
                    "requested_count": 1,
                    "written_count": 1,
                    "failed_count": 0,
                }
            },
        },
    }
    report_path.write_text(json.dumps(report), encoding="utf-8")
    return report_path


_HASH_A = "ab" + ("0" * 62)
_HASH_B = "ab" + ("1" * 62)


def _write_meta(
    path: Path,
    *,
    hash_value: str,
    canonical_string: str | None = None,
    schema_version: int = 1,
    metadata_overrides: dict[str, object] | None = None,
) -> None:
    metadata = {
        "schema_version": schema_version,
        "hash": hash_value,
        "canonical_string": canonical_string
        or f"shader_variant_v1|hash={hash_value}",
        "template_revision": "template-r1",
        "naga_version": "naga-test",
        "wgpu_version": "wgpu-test",
        "created_unix_seconds": 1,
    }
    if metadata_overrides:
        metadata.update(metadata_overrides)
    path.write_text(
        json.dumps(metadata),
        encoding="utf-8",
    )


if __name__ == "__main__":
    unittest.main()
