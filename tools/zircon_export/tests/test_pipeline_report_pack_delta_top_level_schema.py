from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.pack_schema_test_support import (
    assert_pack_schema_diagnostic as _assert_pack_schema_diagnostic,
    manifest_override as _manifest_override,
    sync_delta_report_counts as _sync_delta_report_counts,
    update_pack_report as _update_pack_report,
    write_library_embed_reports as _write_library_embed_reports,
)
from tools.zircon_export.tests.pack_test_support import (
    delta_manifest as _delta_manifest,
    pack_plan as _pack_plan,
)


class PipelineReportPackDeltaTopLevelSchemaTests(unittest.TestCase):
    def test_report_stage_rejects_pack_delta_manifest_unknown_field(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                delta_manifest={**_delta_manifest(), "sidecar": "unexpected"},
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report delta_manifest unknown field sidecar",
            )

    def test_report_stage_rejects_pack_delta_manifest_field_types(self) -> None:
        cases = (
            (
                "format_version",
                {**_delta_manifest(), "format_version": "1"},
                "pack report delta_manifest.format_version must be an integer",
            ),
            (
                "base",
                {**_delta_manifest(), "base": "bad"},
                "pack report delta_manifest.base must be an object",
            ),
            (
                "target.pack.version",
                {
                    **_delta_manifest(),
                    "target": _manifest_override(
                        {"pack": {**_pack_plan(hash_value=2), "version": "1"}},
                        hash_value=2,
                    ),
                },
                "pack report delta_manifest.target.pack.version must be an integer",
            ),
            (
                "chunks",
                {**_delta_manifest(), "chunks": ["bad"]},
                "pack report delta_manifest.chunks[0] must be an object",
            ),
            (
                "changed_assets",
                {**_delta_manifest(), "changed_assets": ["bad"]},
                "pack report delta_manifest.changed_assets[0] must be an object",
            ),
            (
                "removed_assets",
                {**_delta_manifest(), "removed_assets": ["old.scene", 42]},
                "pack report delta_manifest.removed_assets must be a string array",
            ),
        )
        for label, delta_manifest, expected in cases:
            with self.subTest(label=label):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_library_embed_reports(out)
                    _update_pack_report(out, delta_manifest=delta_manifest)

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(self, report, expected)

    def test_report_stage_rejects_pack_delta_manifest_missing_required_field(
        self,
    ) -> None:
        cases = (
            (
                "format_version",
                {
                    key: value
                    for key, value in _delta_manifest().items()
                    if key != "format_version"
                },
                "pack report delta_manifest.format_version must be an integer",
            ),
            (
                "base",
                {
                    key: value
                    for key, value in _delta_manifest().items()
                    if key != "base"
                },
                "pack report delta_manifest.base must be an object",
            ),
            (
                "target",
                {
                    key: value
                    for key, value in _delta_manifest().items()
                    if key != "target"
                },
                "pack report delta_manifest.target must be an object",
            ),
            (
                "chunks",
                {
                    key: value
                    for key, value in _delta_manifest().items()
                    if key != "chunks"
                },
                "pack report delta_manifest.chunks must be an object array",
            ),
            (
                "changed_assets",
                {
                    key: value
                    for key, value in _delta_manifest().items()
                    if key != "changed_assets"
                },
                "pack report delta_manifest.changed_assets must be an object array",
            ),
            (
                "removed_assets",
                {
                    key: value
                    for key, value in _delta_manifest().items()
                    if key != "removed_assets"
                },
                "pack report delta_manifest.removed_assets must be a string array",
            ),
        )
        for label, delta_manifest, expected in cases:
            with self.subTest(label=label):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_library_embed_reports(out)
                    _update_pack_report(out, delta_manifest=delta_manifest)

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(self, report, expected)

    def test_report_stage_rejects_pack_delta_path_array_blank_entry(self) -> None:
        cases = (
            (
                "delta_removed_assets",
                {"delta_removed_assets": [" "]},
                "pack report delta_removed_assets must not contain blank entries",
            ),
            (
                "delta_reused_assets",
                {"delta_reused_assets": [" "]},
                "pack report delta_reused_assets must not contain blank entries",
            ),
            (
                "delta_manifest.removed_assets",
                {"delta_manifest": {**_delta_manifest(), "removed_assets": [" "]}},
                "pack report delta_manifest.removed_assets must not contain blank entries",
            ),
        )
        for label, overrides, expected in cases:
            with self.subTest(label=label):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_library_embed_reports(out)
                    _update_pack_report(out, delta_manifest=_delta_manifest())
                    pack_report_path = out / "stages" / "pack" / "report.json"
                    pack_report = json.loads(
                        pack_report_path.read_text(encoding="utf-8")
                    )
                    pack_report.update(overrides)
                    if "delta_manifest" in overrides:
                        _sync_delta_report_counts(pack_report)
                    pack_report_path.write_text(
                        json.dumps(pack_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(self, report, expected)

    def test_report_stage_rejects_pack_delta_missing_report_audit_field(
        self,
    ) -> None:
        cases = (
            (
                "delta_asset_count",
                "pack report delta_asset_count must be an integer",
            ),
            (
                "delta_chunk_count",
                "pack report delta_chunk_count must be an integer",
            ),
            (
                "delta_removed_assets",
                "pack report delta_removed_assets must be a string array",
            ),
            (
                "delta_reused_assets",
                "pack report delta_reused_assets must be a string array",
            ),
        )
        for field, expected in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    delta_pack = out / "pack-output" / "assets.delta.zrpd"
                    delta_manifest = _delta_manifest()
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
                    pack_report["delta_pack"] = str(delta_pack)
                    pack_report["delta_apply_verified"] = True
                    pack_report["trim_report"]["included_assets"] = [
                        asset["path"] for asset in delta_manifest["target"]["assets"]
                    ]
                    pack_report.pop(field)
                    pack_report_path.write_text(
                        json.dumps(pack_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(self, report, expected)

    def test_report_stage_rejects_pack_delta_missing_previous_pack(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            delta_pack = out / "pack-output" / "assets.delta.zrpd"
            delta_manifest = _delta_manifest()
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                manifest=delta_manifest["target"],
                delta_manifest=delta_manifest,
            )
            pack_report_path = out / "stages" / "pack" / "report.json"
            pack_report = json.loads(pack_report_path.read_text(encoding="utf-8"))
            pack_report["delta_pack"] = str(delta_pack)
            pack_report["delta_apply_verified"] = True
            pack_report["trim_report"]["included_assets"] = [
                asset["path"] for asset in delta_manifest["target"]["assets"]
            ]
            pack_report.pop("previous_pack", None)
            pack_report_path.write_text(
                json.dumps(pack_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report previous_pack must be a string",
            )

    def test_report_stage_rejects_pack_delta_blank_previous_pack(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            delta_pack = out / "pack-output" / "assets.delta.zrpd"
            delta_manifest = _delta_manifest()
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                manifest=delta_manifest["target"],
                delta_manifest=delta_manifest,
            )
            pack_report_path = out / "stages" / "pack" / "report.json"
            pack_report = json.loads(pack_report_path.read_text(encoding="utf-8"))
            pack_report["previous_pack"] = " "
            pack_report["delta_pack"] = str(delta_pack)
            pack_report["delta_apply_verified"] = True
            pack_report["trim_report"]["included_assets"] = [
                asset["path"] for asset in delta_manifest["target"]["assets"]
            ]
            pack_report_path.write_text(
                json.dumps(pack_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report previous_pack must be a non-empty string",
            )

    def test_report_stage_rejects_pack_delta_blank_delta_pack(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            previous_pack = out / "pack-output" / "previous.zrpack"
            delta_manifest = _delta_manifest()
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                manifest=delta_manifest["target"],
                delta_manifest=delta_manifest,
            )
            pack_report_path = out / "stages" / "pack" / "report.json"
            pack_report = json.loads(pack_report_path.read_text(encoding="utf-8"))
            pack_report["previous_pack"] = str(previous_pack)
            pack_report["delta_pack"] = " "
            pack_report["delta_apply_verified"] = True
            pack_report["trim_report"]["included_assets"] = [
                asset["path"] for asset in delta_manifest["target"]["assets"]
            ]
            pack_report_path.write_text(
                json.dumps(pack_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report delta_pack must be a non-empty string",
            )


if __name__ == "__main__":
    unittest.main()
