from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.pack_schema_test_support import (
    assert_pack_schema_diagnostic as _assert_pack_schema_diagnostic,
    manifest_override as _manifest_override,
    missing_dependency as _missing_dependency,
    trim_report as _trim_report,
    trimmed_asset as _trimmed_asset,
    update_pack_report as _update_pack_report,
    write_library_embed_reports as _write_library_embed_reports,
)
from tools.zircon_export.tests.pack_test_support import (
    asset_entry as _asset_entry,
    chunk_entry as _chunk_entry,
    pack_manifest as _pack_manifest,
    pack_plan as _pack_plan,
)


class PipelineReportPackStageSchemaTests(unittest.TestCase):
    def test_report_stage_rejects_pack_stage_output_outside_current_stage(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            _write_library_embed_reports(out)
            pack_report_path = out / "stages" / "pack" / "report.json"
            pack_report = json.loads(pack_report_path.read_text(encoding="utf-8"))
            pack_report["stage_output"] = str(root / "external" / "pack")
            pack_report_path.write_text(
                json.dumps(pack_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report stage_output "
                f"{root / 'external' / 'pack'} does not match current "
                f"Pack stage directory {pack_report_path.parent}",
            )

    def test_report_stage_rejects_pack_manifest_unknown_field(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_library_embed_reports(out)
            _update_pack_report(out, manifest={**_pack_manifest(), "sidecar": "unexpected"})

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report manifest unknown field sidecar",
            )

    def test_report_stage_rejects_pack_manifest_nested_field_types(self) -> None:
        cases = (
            (
                "pack.version",
                _manifest_override({"pack": {**_pack_plan(), "version": "1"}}),
                "pack report manifest.pack.version must be an integer",
            ),
            (
                "pack.total_size",
                _manifest_override({"pack": {**_pack_plan(), "total_size": "8"}}),
                "pack report manifest.pack.total_size must be an integer",
            ),
            (
                "pack.chunks",
                _manifest_override({"pack": {**_pack_plan(), "chunks": ["bad"]}}),
                "pack report manifest.pack.chunks[0] must be an object",
            ),
            (
                "pack.chunks.hash",
                _manifest_override(
                    {
                        "pack": {
                            **_pack_plan(),
                            "chunks": [{**_chunk_entry(), "hash": [1] * 31}],
                        }
                    }
                ),
                "pack report manifest.pack.chunks[0].hash must be a 32-byte integer array",
            ),
            (
                "pack.assets.path",
                _manifest_override(
                    {"assets": [{**_asset_entry(), "path": 42}]}
                ),
                "pack report manifest.assets[0].path must be a string",
            ),
            (
                "pack.assets.chunk_hash",
                _manifest_override(
                    {"assets": [{**_asset_entry(), "chunk_hash": [True] * 32}]}
                ),
                "pack report manifest.assets[0].chunk_hash must be a 32-byte integer array",
            ),
            (
                "pack.assets.size",
                _manifest_override({"assets": [{**_asset_entry(), "size": "8"}]}),
                "pack report manifest.assets[0].size must be an integer",
            ),
        )
        for label, manifest, expected in cases:
            with self.subTest(label=label):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_library_embed_reports(out)
                    _update_pack_report(out, manifest=manifest)

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(self, report, expected)

    def test_report_stage_rejects_pack_manifest_version_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                manifest={
                    "pack": {**_pack_plan(hash_value=1), "version": 2},
                    "assets": [_asset_entry(hash_value=1)],
                },
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report manifest.pack.version 2 is not supported; expected 1",
            )

    def test_report_stage_rejects_pack_manifest_count_mismatch(self) -> None:
        cases = (
            (
                "asset_count",
                99,
                "pack report asset_count 99 does not match manifest.assets length 1",
            ),
            (
                "chunk_count",
                99,
                "pack report chunk_count 99 does not match manifest.pack.chunks length 1",
            ),
        )
        for field, value, expected in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_library_embed_reports(out)
                    _update_pack_report(out, manifest=_pack_manifest())
                    pack_report_path = out / "stages" / "pack" / "report.json"
                    pack_report = json.loads(
                        pack_report_path.read_text(encoding="utf-8")
                    )
                    pack_report[field] = value
                    pack_report_path.write_text(
                        json.dumps(pack_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(self, report, expected)

    def test_report_stage_rejects_pack_manifest_duplicate_chunk_hash(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                manifest={
                    "pack": {
                        "version": 1,
                        "chunks": [
                            _chunk_entry(hash_value=1),
                            {**_chunk_entry(hash_value=1), "offset": 32},
                        ],
                        "total_size": 16,
                    },
                    "assets": [_asset_entry(hash_value=1)],
                },
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report manifest.pack.chunks contains duplicate chunk hash",
            )

    def test_report_stage_rejects_pack_manifest_unsorted_chunk_hash(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                manifest={
                    "pack": {
                        "version": 1,
                        "chunks": [
                            _chunk_entry(hash_value=2),
                            {**_chunk_entry(hash_value=1), "offset": 32},
                        ],
                        "total_size": 16,
                    },
                    "assets": [_asset_entry(hash_value=2)],
                },
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report manifest.pack.chunks must be sorted by chunk hash",
            )

    def test_report_stage_rejects_pack_manifest_asset_missing_chunk_hash(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                manifest={
                    "pack": _pack_plan(hash_value=1),
                    "assets": [_asset_entry(hash_value=2)],
                },
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report manifest.assets[0].chunk_hash "
                "is not present in pack report manifest.pack.chunks",
            )

    def test_report_stage_rejects_pack_manifest_asset_chunk_size_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                manifest={
                    "pack": _pack_plan(hash_value=1),
                    "assets": [{**_asset_entry(hash_value=1), "size": 99}],
                },
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report manifest.assets[0].size 99 does not match "
                "pack report manifest.pack.chunks size 8",
            )

    def test_report_stage_rejects_pack_manifest_total_size_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                manifest={
                    "pack": {**_pack_plan(hash_value=1), "total_size": 99},
                    "assets": [_asset_entry(hash_value=1)],
                },
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report manifest.pack.total_size 99 does not match "
                "pack report manifest.pack.chunks size sum 8",
            )

    def test_report_stage_rejects_pack_manifest_chunk_offset_gap(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                manifest={
                    "pack": {
                        "version": 1,
                        "chunks": [
                            {**_chunk_entry(hash_value=1), "offset": 40},
                        ],
                        "total_size": 8,
                    },
                    "assets": [_asset_entry(hash_value=1)],
                },
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report manifest.pack.chunks[0].offset 40 does not match "
                "expected chunk offset 24",
            )

    def test_report_stage_rejects_pack_trim_report_manifest_asset_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                manifest=_pack_manifest(),
                trim_report={
                    **_trim_report(),
                    "included_assets": ["textures/not-packed.png"],
                },
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report trim_report.included_assets does not match "
                "manifest.assets paths",
            )

    def test_report_stage_rejects_pack_trim_report_unresolved_preflight(
        self,
    ) -> None:
        cases = (
            (
                "duplicate_assets",
                {
                    **_trim_report(),
                    "duplicate_assets": ["scenes/main.zscene"],
                },
                "pack report trim_report.duplicate_assets must be empty "
                "for a non-fatal Pack report",
            ),
            (
                "missing_dependencies",
                {
                    **_trim_report(),
                    "missing_dependencies": [
                        {
                            "owner": "scenes/main.zscene",
                            "dependency": "textures/missing.png",
                        }
                    ],
                },
                "pack report trim_report.missing_dependencies must be empty "
                "for a non-fatal Pack report",
            ),
        )
        for label, trim_report, expected in cases:
            with self.subTest(label=label):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_library_embed_reports(out)
                    _update_pack_report(
                        out,
                        manifest=_pack_manifest(),
                        trim_report=trim_report,
                    )

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(self, report, expected)

    def test_report_stage_rejects_pack_deduplicated_assets_manifest_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                manifest={
                    **_pack_manifest(),
                    "assets": [
                        _asset_entry(path="scenes/main.zscene"),
                        _asset_entry(path="textures/copy.png"),
                    ],
                },
                trim_report={
                    **_trim_report(),
                    "included_assets": [
                        "scenes/main.zscene",
                        "textures/copy.png",
                    ],
                },
                deduplicated_assets=[],
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report deduplicated_assets does not match "
                "manifest duplicate chunk paths",
            )

    def test_report_stage_rejects_pack_trim_report_unknown_fields(self) -> None:
        cases = (
            (
                "trim_report",
                {**_trim_report(), "sidecar": "unexpected"},
                "pack report trim_report unknown field sidecar",
            ),
            (
                "trimmed_assets",
                {
                    **_trim_report(),
                    "trimmed_assets": [
                        {**_trimmed_asset(), "sidecar": "unexpected"}
                    ],
                },
                "pack report trim_report.trimmed_assets[0] unknown field sidecar",
            ),
            (
                "missing_dependencies",
                {
                    **_trim_report(),
                    "missing_dependencies": [
                        {**_missing_dependency(), "sidecar": "unexpected"}
                    ],
                },
                "pack report trim_report.missing_dependencies[0] unknown field sidecar",
            ),
        )
        for label, trim_report, expected in cases:
            with self.subTest(label=label):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_library_embed_reports(out)
                    _update_pack_report(out, trim_report=trim_report)

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(self, report, expected)

    def test_report_stage_rejects_pack_trim_report_field_types(self) -> None:
        cases = (
            (
                "included_assets",
                {**_trim_report(), "included_assets": ["scenes/main.zscene", 42]},
                "pack report trim_report.included_assets must be a string array",
            ),
            (
                "trimmed_assets",
                {**_trim_report(), "trimmed_assets": ["bad"]},
                "pack report trim_report.trimmed_assets[0] must be an object",
            ),
            (
                "trimmed_assets.path",
                {
                    **_trim_report(),
                    "trimmed_assets": [{**_trimmed_asset(), "path": 42}],
                },
                "pack report trim_report.trimmed_assets[0].path must be a string",
            ),
            (
                "trimmed_assets.reason",
                {
                    **_trim_report(),
                    "trimmed_assets": [{**_trimmed_asset(), "reason": 42}],
                },
                "pack report trim_report.trimmed_assets[0].reason must be a string or object",
            ),
            (
                "missing_dependencies.owner",
                {
                    **_trim_report(),
                    "missing_dependencies": [
                        {**_missing_dependency(), "owner": 42}
                    ],
                },
                "pack report trim_report.missing_dependencies[0].owner must be a string",
            ),
            (
                "diagnostics",
                {**_trim_report(), "diagnostics": ["ok", 42]},
                "pack report trim_report.diagnostics must be a string array",
            ),
        )
        for label, trim_report, expected in cases:
            with self.subTest(label=label):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_library_embed_reports(out)
                    _update_pack_report(out, trim_report=trim_report)

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(self, report, expected)

if __name__ == "__main__":
    unittest.main()
