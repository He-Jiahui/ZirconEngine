from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.pack_schema_test_support import (
    assert_pack_schema_diagnostic as _assert_pack_schema_diagnostic,
    missing_dependency as _missing_dependency,
    trim_report as _trim_report,
    trimmed_asset as _trimmed_asset,
    update_pack_report as _update_pack_report,
    write_library_embed_reports as _write_library_embed_reports,
)
from tools.zircon_export.tests.pack_test_support import (
    pack_manifest as _pack_manifest,
)


class PipelineReportPackTrimReportSchemaTests(unittest.TestCase):
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

    def test_report_stage_rejects_pack_trim_report_path_shape(self) -> None:
        cases = (
            (
                "included_assets.unsafe",
                {**_trim_report(), "included_assets": ["../escape.png"]},
                "pack report trim_report.included_assets[0] "
                "must be a safe relative asset path",
            ),
            (
                "duplicate_assets.unnormalized",
                {**_trim_report(), "duplicate_assets": ["textures\\hero.png"]},
                "pack report trim_report.duplicate_assets[0] "
                "must use a normalized relative asset path",
            ),
            (
                "included_assets.duplicate",
                {
                    **_trim_report(),
                    "included_assets": [
                        "scenes/main.zscene",
                        "scenes/main.zscene",
                    ],
                },
                "pack report trim_report.included_assets path scenes/main.zscene "
                "is declared more than once",
            ),
            (
                "trimmed_assets.path",
                {
                    **_trim_report(),
                    "trimmed_assets": [
                        {**_trimmed_asset(), "path": "../escape.png"}
                    ],
                },
                "pack report trim_report.trimmed_assets[0].path "
                "must be a safe relative asset path",
            ),
            (
                "missing_dependencies.owner",
                {
                    **_trim_report(),
                    "missing_dependencies": [
                        {
                            **_missing_dependency(),
                            "owner": "scenes\\main.zscene",
                        }
                    ],
                },
                "pack report trim_report.missing_dependencies[0].owner "
                "must use a normalized relative asset path",
            ),
            (
                "missing_dependencies.dependency",
                {
                    **_trim_report(),
                    "missing_dependencies": [
                        {**_missing_dependency(), "dependency": "../missing.png"}
                    ],
                },
                "pack report trim_report.missing_dependencies[0].dependency "
                "must be a safe relative asset path",
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
                "pack report trim_report.included_assets[1] must be a string",
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
                "pack report trim_report.diagnostics[1] must be a string",
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

    def test_report_stage_rejects_pack_trim_report_string_array_non_string_entry_before_array_shape(
        self,
    ) -> None:
        cases = (
            (
                "included_assets",
                {**_trim_report(), "included_assets": ["scenes/main.zscene", 42]},
                "pack report trim_report.included_assets[1] must be a string",
                "pack report trim_report.included_assets must be a string array",
            ),
            (
                "diagnostics",
                {**_trim_report(), "diagnostics": ["ok", None]},
                "pack report trim_report.diagnostics[1] must be a string",
                "pack report trim_report.diagnostics must be a string array",
            ),
        )
        for label, trim_report, expected, unexpected in cases:
            with self.subTest(label=label):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_library_embed_reports(out)
                    _update_pack_report(out, trim_report=trim_report)

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(self, report, expected)
                    self.assertFalse(
                        any(
                            unexpected in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_pack_trim_report_blank_diagnostic_entry(self) -> None:
        for diagnostics in ([""], ["   "], ["trimmed asset", ""]):
            with self.subTest(diagnostics=diagnostics):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_library_embed_reports(out)
                    _update_pack_report(
                        out,
                        trim_report={
                            **_trim_report(),
                            "diagnostics": diagnostics,
                        },
                    )

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(
                        self,
                        report,
                        "pack report trim_report.diagnostics "
                        "must not contain blank entries",
                    )


if __name__ == "__main__":
    unittest.main()
