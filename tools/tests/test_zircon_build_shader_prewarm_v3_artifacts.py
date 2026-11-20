"""Runtime v3 publishing boundary and historical shader-cache diagnostic coverage."""

import json
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace

from tools.build.zircon_build_shader_prewarm_acceptance import (
    validate_staged_shader_prewarm_acceptance_contract,
)
from tools.build.zircon_build_shader_prewarm_cache_artifacts import (
    validate_shader_prewarm_cache_artifact_contract,
)

_PASSES = ("forward", "gbuffer", "depth_prepass", "shadow", "velocity", "taa_reactive_mask")
_SOURCE_ID = "dd" * 32
_SOURCE_HASH = "a472c7a31645d310613df407eab80844346938a3b8fe4f392cae059cb181aa85"
_PAYLOAD_HASH = "8679575236253b0f98e356e5ae298999e6354c483a981516d47715ba7a13ebe9"
_PAYLOAD = bytes.fromhex("28b52ffd200c610000666e206d61696e2829207b7d")  # zstd: b"fn main() {}"


def _write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value), encoding="utf-8")


class Fixture:
    def __init__(self, root: Path, passes=_PASSES):
        self.engine = root / "ZirconEngine"
        self.cache = self.engine / "cache/shader_variants"
        self.report = self.engine / "cache/shader_variants_report.json"
        self.registry = self.engine / "cache/shader_resource_records.json"
        self.rows = []
        self.artifacts = []
        counts = dict(requested_count=len(passes), written_count=len(passes), failed_count=0)
        for i, pass_type in enumerate(passes):
            h = f"{i + 0x41:02x}" + ("ac" * 31)
            canonical = f"shader_variant_v1|fixture=1|geometry=0|shading=2|pass={pass_type}|quality=medium"
            row = dict(cache_hash=h, canonical_string=canonical, source_id=_SOURCE_ID,
                       source_label="builtin://shader/test.wgsl", template_revision="template-r1",
                       naga_version="naga-test", wgpu_version="wgpu-test")
            self.rows.append(row)
            stem = self.cache / "v3" / h[:2] / h
            meta = {k: row[k] for k in ("source_id", "canonical_string", "template_revision", "naga_version", "wgpu_version")}
            meta.update(schema_version=3, hash=h, source_hash=_SOURCE_HASH, payload_hash=_PAYLOAD_HASH,
                        compressed_bytes=len(_PAYLOAD), decoded_bytes=12, created_unix_seconds=1)
            manifest, payload = stem.with_suffix(".manifest"), stem.with_suffix(".wgsl.zst")
            self.artifacts.append((manifest, payload, meta))
            _write_json(manifest, meta)
            payload.write_bytes(_PAYLOAD)
        source = {k: self.rows[0][k] for k in ("source_id", "source_label", "template_revision", "naga_version", "wgpu_version")}
        source.update(counts, source_hash=_SOURCE_HASH, include_content_hashes=[])
        self.report_data = dict(
            counts, written_variants=self.rows,
            source_provenance=dict(source_count=1, variant_count=len(passes), sources={_SOURCE_ID: source}),
            dimension_summary=dict(pass_types={p: dict(requested_count=1, written_count=1, failed_count=0) for p in passes},
                                   quality_tiers={"medium": counts}, geometry_source_ids={"0": counts}),
        )
        _write_json(self.report, self.report_data)
        _write_json(self.registry, {"resources": []})

    def lower(self):
        validate_shader_prewarm_cache_artifact_contract(self.cache, report_path=self.report)

    def stage(self):
        validate_staged_shader_prewarm_acceptance_contract(SimpleNamespace(
            engine_root=self.engine, shader_prewarm_cache_root=self.cache,
            shader_prewarm_report_path=self.report, shader_prewarm_resource_registry_path=self.registry,
            shader_quality_tiers=("medium",), shader_geometry_sources=("static",),
            shader_geometry_source_ids=(), shader_shading_model_ids=(),
        ))

    def add_legacy(self, version=1):
        for manifest, payload, meta in self.artifacts:
            h = meta["hash"]
            legacy_stem = self.cache / f"v{version}" / h[:2] / h
            old_meta = {k: meta[k] for k in ("hash", "canonical_string", "template_revision", "naga_version",
                                            "wgpu_version", "created_unix_seconds")}
            _write_json(legacy_stem.with_suffix(".meta"), {**old_meta, "schema_version": version})
            legacy_stem.with_suffix(".wgsl.zst").write_bytes(_PAYLOAD)

    def convert_to_legacy(self, version=1):
        self.add_legacy(version=version)
        for manifest, payload, _ in self.artifacts:
            manifest.unlink()
            payload.unlink()
        for row in self.rows:
            row.pop("source_id")
        _write_json(self.report, self.report_data)


class ShaderPrewarmV3ArtifactTests(unittest.TestCase):
    def fixture(self, passes=("forward",)):
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        return Fixture(Path(directory.name), passes)

    def test_accepts_committed_v3_through_actual_six_pass_stage_gate(self):
        self.fixture(_PASSES).stage()

    def test_ignores_unreported_inflight_payload_and_stale_legacy_tree(self):
        fixture = self.fixture(_PASSES)
        h = "bc" * 32
        path = fixture.cache / "v3/bc" / f"{h}.wgsl.zst"
        path.parent.mkdir(parents=True)
        path.write_bytes(_PAYLOAD)
        path.with_name(f".{h}.manifest.tmp-1-0").write_bytes(b"not yet committed")
        fixture.add_legacy()
        fixture.stage()

    def test_reported_payload_without_manifest_does_not_count_as_published(self):
        fixture = self.fixture()
        manifest, _, _ = fixture.artifacts[0]
        manifest.unlink()
        fixture.add_legacy()
        with self.assertRaisesRegex(RuntimeError, "missing committed v3 manifest"):
            fixture.lower()

    def test_manifest_only_is_a_broken_published_pair(self):
        fixture = self.fixture()
        _, payload, _ = fixture.artifacts[0]
        payload.unlink()
        with self.assertRaisesRegex(RuntimeError, "no immutable WGSL payload"):
            fixture.lower()

    def test_rejects_wrong_v3_shard_even_when_pair_is_complete(self):
        fixture = self.fixture()
        manifest, payload, _ = fixture.artifacts[0]
        dest = fixture.cache / "v3/ff"
        dest.mkdir()
        manifest.rename(dest / manifest.name)
        payload.rename(dest / payload.name)
        with self.assertRaisesRegex(RuntimeError, "missing committed v3 manifest"):
            fixture.lower()

    def test_checks_schema_identity_provenance_and_uint_budgets(self):
        fixture = self.fixture()
        manifest, _, meta = fixture.artifacts[0]
        for field, bad, expected in (
            ("schema_version", 2, "manifest schema mismatch"),
            ("schema_version", True, "manifest schema mismatch"),
            ("hash", "ee" * 32, "reported cache variant mismatch"),
            ("canonical_string", "wrong-variant", "reported cache variant mismatch"),
            ("wgpu_version", "other-toolchain", "reported cache variant mismatch"),
            ("source_id", "ee" * 32, "source_id mismatch"),
            ("source_hash", "ee" * 32, "source provenance mismatch"),
            ("payload_hash", "bad", "field=payload_hash"),
            ("compressed_bytes", True, "invalid compressed_bytes"),
            ("decoded_bytes", 32 * 1024 * 1024 + 1, "invalid decoded_bytes"),
            ("created_unix_seconds", -1, "invalid created_unix_seconds"),
        ):
            with self.subTest(field=field, bad=bad):
                _write_json(manifest, {**meta, field: bad})
                with self.assertRaisesRegex(RuntimeError, expected):
                    fixture.lower()

    def test_rejects_v3_when_report_identity_or_provenance_is_missing(self):
        fixture = self.fixture(_PASSES)
        fixture.rows[0].pop("source_id")
        _write_json(fixture.report, fixture.report_data)
        with self.assertRaisesRegex(RuntimeError, "source_id mismatch"):
            fixture.stage()

    def test_caps_committed_manifest_read_and_reports_malformed_utf8(self):
        fixture = self.fixture()
        manifest, _, _ = fixture.artifacts[0]
        for blob, expected in ((b"*" * 65537, "manifest exceeds byte budget"), (b"\xff", "invalid v3 manifest")):
            with self.subTest(expected=expected):
                manifest.write_bytes(blob)
                with self.assertRaisesRegex(RuntimeError, expected):
                    fixture.lower()

    def test_checks_physical_payload_size_and_zstd_magic(self):
        fixture = self.fixture()
        _, payload, _ = fixture.artifacts[0]
        payload.write_bytes(_PAYLOAD[:-1])
        with self.assertRaisesRegex(RuntimeError, "compressed_bytes mismatch"):
            fixture.lower()
        payload.write_bytes(b"N" + _PAYLOAD[1:])
        with self.assertRaisesRegex(RuntimeError, "not a Zstd frame"):
            fixture.lower()

    def test_historical_v1_diagnostic_stays_valid_but_stage_rejects_it(self):
        fixture = self.fixture(_PASSES)
        fixture.convert_to_legacy()
        fixture.lower()
        with self.assertRaisesRegex(RuntimeError, "missing committed v3 manifest"):
            fixture.stage()

    def test_does_not_enable_historical_v2_split_format(self):
        fixture = self.fixture()
        fixture.convert_to_legacy(version=2)
        with self.assertRaisesRegex(RuntimeError, "runtime cache layout"):
            fixture.lower()
