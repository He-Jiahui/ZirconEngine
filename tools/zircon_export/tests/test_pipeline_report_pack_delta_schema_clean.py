from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.export_test_support import (
    _pack_binary_bytes,
)
from tools.zircon_export.tests.pack_schema_test_support import (
    assert_pack_schema_diagnostic as _assert_pack_schema_diagnostic,
    update_pack_report as _update_pack_report,
    write_library_embed_reports as _write_library_embed_reports,
)
from tools.zircon_export.tests.pack_test_support import (
    delta_manifest as _delta_manifest,
    empty_delta_manifest as _empty_delta_manifest,
)


class PipelineReportPackDeltaSchemaCleanTests(unittest.TestCase):
    def test_report_stage_rejects_delta_manifest_schema_before_embedded_match(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            embedded_delta_manifest = _empty_delta_manifest()
            delta_manifest = _empty_delta_manifest()
            delta_manifest["changed_assets"] = [
                {
                    "path": "scenes/main.zscene",
                    "chunk_hash": [1] * 32,
                    "size": -1,
                }
            ]
            _write_library_embed_reports(out)
            _update_pack_report(out, delta_manifest=delta_manifest)
            delta_pack = out / "pack-output" / "assets.delta.zrpd"
            previous_pack = out / "pack-output" / "previous.zrpack"
            delta_pack.parent.mkdir(parents=True, exist_ok=True)
            delta_pack.write_bytes(
                _pack_binary_bytes(embedded_delta_manifest, b"ZRPD")
            )
            previous_pack.write_bytes(
                _pack_binary_bytes(embedded_delta_manifest["base"], b"ZRPK")
            )
            pack_report_path = out / "stages" / "pack" / "report.json"
            pack_report = json.loads(pack_report_path.read_text(encoding="utf-8"))
            pack_report["delta_pack"] = str(delta_pack)
            pack_report["previous_pack"] = str(previous_pack)
            pack_report["delta_apply_verified"] = True
            pack_report_path.write_text(
                json.dumps(pack_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report delta_manifest.changed_assets[0].size "
                "must be non-negative",
            )
            diagnostics = report["diagnostics"]
            self.assertFalse(
                any(
                    "delta_pack embedded manifest does not match delta_manifest"
                    in diagnostic
                    for diagnostic in diagnostics
                ),
                diagnostics,
            )

    def test_report_stage_rejects_delta_chunk_schema_before_changed_chunk_semantics(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            delta_manifest = _delta_manifest()
            delta_manifest["chunks"] = [
                {
                    **delta_manifest["chunks"][0],
                    "hash": [4] * 32,
                    "offset": -1,
                }
            ]
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                manifest=delta_manifest["target"],
                delta_manifest=delta_manifest,
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report delta_manifest.chunks[0].offset must be non-negative",
            )
            diagnostics = report["diagnostics"]
            self.assertFalse(
                any(
                    "delta_manifest.chunks does not match "
                    "changed asset chunk hashes" in diagnostic
                    for diagnostic in diagnostics
                ),
                diagnostics,
            )

    def test_report_stage_rejects_delta_changed_asset_schema_before_set_semantics(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            delta_manifest = _delta_manifest()
            delta_manifest["changed_assets"] = [
                {
                    **delta_manifest["changed_assets"][0],
                    "path": "scenes/not-main.zscene",
                    "size": -1,
                }
            ]
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                manifest=delta_manifest["target"],
                delta_manifest=delta_manifest,
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report delta_manifest.changed_assets[0].size "
                "must be non-negative",
            )
            diagnostics = report["diagnostics"]
            self.assertFalse(
                any(
                    "delta_manifest.changed_assets does not match "
                    "target assets missing from base chunks" in diagnostic
                    for diagnostic in diagnostics
                ),
                diagnostics,
            )

    def test_report_stage_rejects_delta_count_schema_before_length_semantics(
        self,
    ) -> None:
        cases = (
            (
                "changed_assets",
                lambda delta_manifest: delta_manifest["changed_assets"][0].update(
                    {"size": -1}
                ),
                lambda pack_report: pack_report.update({"delta_asset_count": 99}),
                "pack report delta_manifest.changed_assets[0].size must be non-negative",
                "pack report delta_asset_count 99 does not match "
                "delta_manifest.changed_assets length 1",
            ),
            (
                "chunks",
                lambda delta_manifest: delta_manifest["chunks"][0].update(
                    {"hash": [2] * 31}
                ),
                lambda pack_report: pack_report.update({"delta_chunk_count": 99}),
                "pack report delta_manifest.chunks[0].hash "
                "must be a 32-byte integer array",
                "pack report delta_chunk_count 99 does not match "
                "delta_manifest.chunks length 1",
            ),
        )
        for label, mutate_delta_manifest, mutate_pack_report, expected, blocked in cases:
            with self.subTest(label=label):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    delta_manifest = _delta_manifest()
                    mutate_delta_manifest(delta_manifest)
                    _write_library_embed_reports(out)
                    _update_pack_report(
                        out,
                        manifest=delta_manifest["target"],
                        delta_manifest=delta_manifest,
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

    def test_report_stage_rejects_delta_asset_negative_size_before_entry_semantics(
        self,
    ) -> None:
        cases = (
            (
                "target",
                lambda delta_manifest: delta_manifest["target"]["assets"][0].update(
                    {"size": -1}
                ),
                "pack report delta_manifest.target.assets[0].size must be non-negative",
            ),
            (
                "changed_assets",
                lambda delta_manifest: delta_manifest["changed_assets"][0].update(
                    {"size": -1}
                ),
                "pack report delta_manifest.changed_assets[0].size must be non-negative",
            ),
        )
        for label, mutate, expected in cases:
            with self.subTest(label=label):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    delta_manifest = _delta_manifest()
                    mutate(delta_manifest)
                    _write_library_embed_reports(out)
                    _update_pack_report(
                        out,
                        manifest=delta_manifest["target"],
                        delta_manifest=delta_manifest,
                    )

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(self, report, expected)
                    diagnostics = report["diagnostics"]
                    self.assertFalse(
                        any(
                            "delta_manifest.changed_assets does not match "
                            "target manifest asset entries" in diagnostic
                            for diagnostic in diagnostics
                        ),
                        diagnostics,
                    )


if __name__ == "__main__":
    unittest.main()
