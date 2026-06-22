from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.pack_schema_test_support import (
    assert_pack_schema_diagnostic as _assert_pack_schema_diagnostic,
    manifest_override as _manifest_override,
    update_pack_report as _update_pack_report,
    write_library_embed_reports as _write_library_embed_reports,
)
from tools.zircon_export.tests.pack_test_support import (
    asset_entry as _asset_entry,
    chunk_entry as _chunk_entry,
    pack_plan as _pack_plan,
)


class PipelineReportPackChunkSchemaTests(unittest.TestCase):
    def test_report_stage_rejects_pack_asset_chunk_reference_after_schema_clean(
        self,
    ) -> None:
        cases = (
            (
                "asset_size",
                {
                    "pack": _pack_plan(hash_value=1),
                    "assets": [
                        {
                            **_asset_entry(hash_value=2),
                            "size": -1,
                        }
                    ],
                },
                "pack report manifest.assets[0].size must be non-negative",
            ),
            (
                "chunk_size",
                {
                    "pack": {
                        "version": 1,
                        "chunks": [
                            {
                                **_chunk_entry(hash_value=1),
                                "size": -1,
                            }
                        ],
                        "total_size": 8,
                    },
                    "assets": [_asset_entry(hash_value=2)],
                },
                "pack report manifest.pack.chunks[0].size must be non-negative",
            ),
        )
        for label, manifest, expected in cases:
            with self.subTest(label=label):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_library_embed_reports(out)
                    _update_pack_report(
                        out,
                        manifest=_manifest_override(manifest),
                    )

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(self, report, expected)
                    diagnostics = report["diagnostics"]
                    self.assertFalse(
                        any(
                            "chunk_hash is not present in "
                            "pack report manifest.pack.chunks" in diagnostic
                            for diagnostic in diagnostics
                        ),
                        diagnostics,
                    )

    def test_report_stage_rejects_pack_manifest_count_schema_before_length_semantics(
        self,
    ) -> None:
        cases = (
            (
                "asset_count",
                {
                    "pack": _pack_plan(hash_value=1),
                    "assets": [
                        {
                            **_asset_entry(hash_value=1),
                            "size": -1,
                        }
                    ],
                },
                lambda pack_report: pack_report.update({"asset_count": 99}),
                "pack report manifest.assets[0].size must be non-negative",
                "pack report asset_count 99 does not match manifest.assets length 1",
            ),
            (
                "chunk_count",
                {
                    "pack": {
                        "version": 1,
                        "chunks": [
                            {
                                **_chunk_entry(hash_value=1),
                                "hash": [1] * 31,
                            }
                        ],
                        "total_size": 8,
                    },
                    "assets": [_asset_entry(hash_value=1)],
                },
                lambda pack_report: pack_report.update({"chunk_count": 99}),
                "pack report manifest.pack.chunks[0].hash "
                "must be a 32-byte integer array",
                "pack report chunk_count 99 does not match "
                "manifest.pack.chunks length 1",
            ),
        )
        for label, manifest, mutate_pack_report, expected, blocked in cases:
            with self.subTest(label=label):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_library_embed_reports(out)
                    _update_pack_report(
                        out,
                        manifest=_manifest_override(manifest),
                    )
                    pack_report_path = out / "stages" / "pack" / "report.json"
                    pack_report = json.loads(
                        pack_report_path.read_text(encoding="utf-8")
                    )
                    mutate_pack_report(pack_report)
                    pack_report_path.write_text(
                        json.dumps(pack_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(self, report, expected)
                    diagnostics = report["diagnostics"]
                    self.assertFalse(
                        any(blocked in diagnostic for diagnostic in diagnostics),
                        diagnostics,
                    )

    def test_report_stage_rejects_pack_chunk_hash_malformed_before_chunk_semantics(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                manifest=_manifest_override(
                    {
                        "pack": {
                            "version": 1,
                            "chunks": [
                                {
                                    "hash": [1] * 31,
                                    "offset": 25,
                                    "size": 8,
                                }
                            ],
                            "total_size": 99,
                        },
                        "assets": [_asset_entry(hash_value=1)],
                    }
                ),
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report manifest.pack.chunks[0].hash "
                "must be a 32-byte integer array",
            )
            diagnostics = report["diagnostics"]
            self.assertFalse(
                any(
                    "manifest.pack.total_size" in diagnostic
                    or "manifest.pack.chunks[0].offset" in diagnostic
                    or "pack embedded manifest does not match manifest" in diagnostic
                    or "trim_report.included_assets does not match" in diagnostic
                    for diagnostic in diagnostics
                ),
                diagnostics,
            )

    def test_report_stage_rejects_pack_chunk_or_asset_size_schema_before_size_semantics(
        self,
    ) -> None:
        cases = (
            (
                "chunk_offset",
                {
                    "pack": {
                        "version": 1,
                        "chunks": [
                            {
                                **_chunk_entry(hash_value=1),
                                "offset": -1,
                                "size": 99,
                            }
                        ],
                        "total_size": 99,
                    },
                    "assets": [_asset_entry(hash_value=1)],
                },
                "pack report manifest.pack.chunks[0].offset must be non-negative",
            ),
            (
                "chunk_size",
                {
                    "pack": {
                        "version": 1,
                        "chunks": [
                            {
                                **_chunk_entry(hash_value=1),
                                "size": -1,
                            }
                        ],
                        "total_size": 99,
                    },
                    "assets": [_asset_entry(hash_value=1)],
                },
                "pack report manifest.pack.chunks[0].size must be non-negative",
            ),
            (
                "asset_size",
                {
                    "pack": _pack_plan(hash_value=1),
                    "assets": [
                        {
                            **_asset_entry(hash_value=1),
                            "size": -1,
                        }
                    ],
                },
                "pack report manifest.assets[0].size must be non-negative",
            ),
        )
        for label, manifest, expected in cases:
            with self.subTest(label=label):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_library_embed_reports(out)
                    _update_pack_report(
                        out,
                        manifest=_manifest_override(manifest),
                    )

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(self, report, expected)
                    diagnostics = report["diagnostics"]
                    self.assertFalse(
                        any(
                            "does not match pack report manifest.pack.chunks size"
                            in diagnostic
                            for diagnostic in diagnostics
                        ),
                        diagnostics,
                    )


if __name__ == "__main__":
    unittest.main()
