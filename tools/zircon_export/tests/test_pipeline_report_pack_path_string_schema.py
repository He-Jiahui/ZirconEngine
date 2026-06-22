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
    delta_manifest as _delta_manifest,
    pack_manifest as _pack_manifest,
    pack_plan as _pack_plan,
)


class PipelineReportPackPathStringSchemaTests(unittest.TestCase):
    def test_report_stage_rejects_pack_path_field_padded_string(self) -> None:
        cases = ("asset_manifest", "pack", "stage_output")
        for field in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_library_embed_reports(out)
                    pack_report_path = out / "stages" / "pack" / "report.json"
                    pack_report = json.loads(
                        pack_report_path.read_text(encoding="utf-8")
                    )
                    pack_report[field] = f" {pack_report[field]} "
                    pack_report_path.write_text(
                        json.dumps(pack_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(
                        self,
                        report,
                        f"pack report {field} must be a non-empty trimmed string",
                    )

    def test_report_stage_rejects_pack_manifest_asset_path_padded_string(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                manifest=_manifest_override(
                    {
                        "assets": [
                            {
                                **_asset_entry(hash_value=1),
                                "path": " scenes/main.zscene ",
                            },
                            _asset_entry(
                                hash_value=1,
                                path="textures/hero.png",
                            ),
                        ],
                        "pack": _pack_plan(hash_value=1),
                    }
                ),
                deduplicated_assets=[],
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report manifest.assets[0].path "
                "must be a non-empty trimmed string",
            )
            diagnostics = report["diagnostics"]
            self.assertFalse(
                any(
                    "deduplicated_assets does not match" in diagnostic
                    for diagnostic in diagnostics
                ),
                diagnostics,
            )
            self.assertFalse(
                any(
                    "pack embedded manifest does not match manifest" in diagnostic
                    for diagnostic in diagnostics
                ),
                diagnostics,
            )

    def test_report_stage_rejects_pack_trim_report_path_padded_string(
        self,
    ) -> None:
        cases = (
            (
                "included_assets",
                {
                    **_trim_report(),
                    "included_assets": [" scenes/main.zscene "],
                },
                "pack report trim_report.included_assets[0] "
                "must be a non-empty trimmed string",
            ),
            (
                "duplicate_assets",
                {
                    **_trim_report(),
                    "duplicate_assets": [" scenes/main.zscene "],
                },
                "pack report trim_report.duplicate_assets[0] "
                "must be a non-empty trimmed string",
            ),
            (
                "trimmed_assets.path",
                {
                    **_trim_report(),
                    "trimmed_assets": [
                        {
                            **_trimmed_asset(),
                            "path": " textures/unused.png ",
                        }
                    ],
                },
                "pack report trim_report.trimmed_assets[0].path "
                "must be a non-empty trimmed string",
            ),
            (
                "missing_dependencies.owner",
                {
                    **_trim_report(),
                    "missing_dependencies": [
                        {
                            **_missing_dependency(),
                            "owner": " scenes/main.zscene ",
                        }
                    ],
                },
                "pack report trim_report.missing_dependencies[0].owner "
                "must be a non-empty trimmed string",
            ),
            (
                "missing_dependencies.dependency",
                {
                    **_trim_report(),
                    "missing_dependencies": [
                        {
                            **_missing_dependency(),
                            "dependency": " textures/missing.png ",
                        }
                    ],
                },
                "pack report trim_report.missing_dependencies[0].dependency "
                "must be a non-empty trimmed string",
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
                    self.assertFalse(
                        any(
                            "must use a normalized relative asset path" in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_pack_trim_report_schema_before_preflight(
        self,
    ) -> None:
        cases = (
            (
                "duplicate_assets",
                {**_trim_report(), "duplicate_assets": [" scenes/main.zscene "]},
                "pack report trim_report.duplicate_assets[0] "
                "must be a non-empty trimmed string",
                "pack report trim_report.duplicate_assets must be empty "
                "for a non-fatal Pack report",
            ),
            (
                "missing_dependencies",
                {
                    **_trim_report(),
                    "missing_dependencies": [
                        {
                            **_missing_dependency(),
                            "dependency": " textures/missing.png ",
                        }
                    ],
                },
                "pack report trim_report.missing_dependencies[0].dependency "
                "must be a non-empty trimmed string",
                "pack report trim_report.missing_dependencies must be empty "
                "for a non-fatal Pack report",
            ),
        )
        for label, trim_report, expected, blocked in cases:
            with self.subTest(label=label):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_library_embed_reports(out)
                    _update_pack_report(out, trim_report=trim_report)

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(self, report, expected)
                    self.assertFalse(
                        any(blocked in diagnostic for diagnostic in report["diagnostics"]),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_pack_deduplicated_asset_path_padded_string(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                deduplicated_assets=[" scenes/main.zscene "],
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report deduplicated_assets[0] "
                "must be a non-empty trimmed string",
            )
            diagnostics = report["diagnostics"]
            self.assertFalse(
                any(
                    "deduplicated_assets does not match" in diagnostic
                    for diagnostic in diagnostics
                ),
                diagnostics,
            )

    def test_report_stage_rejects_pack_delta_path_field_padded_string(self) -> None:
        cases = ("delta_pack", "previous_pack")
        for field in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    delta_manifest = _delta_manifest()
                    delta_pack = out / "pack-output" / "assets.delta.zrpd"
                    previous_pack = out / "pack-output" / "previous.zrpack"
                    _write_library_embed_reports(out)
                    _update_pack_report(
                        out,
                        manifest=delta_manifest["target"],
                        delta_manifest=delta_manifest,
                    )
                    delta_pack.parent.mkdir(parents=True, exist_ok=True)
                    delta_pack.write_text("delta pack placeholder", encoding="utf-8")
                    previous_pack.write_text("previous pack placeholder", encoding="utf-8")
                    pack_report_path = out / "stages" / "pack" / "report.json"
                    pack_report = json.loads(
                        pack_report_path.read_text(encoding="utf-8")
                    )
                    pack_report["delta_pack"] = str(delta_pack)
                    pack_report["previous_pack"] = str(previous_pack)
                    pack_report["delta_apply_verified"] = True
                    pack_report["trim_report"]["included_assets"] = [
                        asset["path"] for asset in delta_manifest["target"]["assets"]
                    ]
                    pack_report[field] = f" {pack_report[field]} "
                    pack_report_path.write_text(
                        json.dumps(pack_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(
                        self,
                        report,
                        f"pack report {field} must be a non-empty trimmed string",
                    )

    def test_report_stage_rejects_pack_delta_publication_padded_path_before_pairing(
        self,
    ) -> None:
        cases = (
            (
                "delta_pack",
                {"delta_pack": " pack-output/assets.delta.zrpd "},
                "pack report delta_pack must be a non-empty trimmed string",
                (
                    "delta_pack is present but delta_manifest is missing",
                    "delta_apply_verified is not true",
                ),
            ),
            (
                "previous_pack",
                {"previous_pack": " pack-output/previous.zrpack "},
                "pack report previous_pack must be a non-empty trimmed string",
                ("previous_pack is present but delta_pack is missing",),
            ),
        )
        for label, overrides, expected, blocked_fragments in cases:
            with self.subTest(label=label):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_library_embed_reports(out)
                    pack_report_path = out / "stages" / "pack" / "report.json"
                    pack_report = json.loads(
                        pack_report_path.read_text(encoding="utf-8")
                    )
                    pack_report.pop("delta_manifest", None)
                    pack_report.pop("delta_pack", None)
                    pack_report.pop("previous_pack", None)
                    pack_report.pop("delta_apply_verified", None)
                    pack_report.update(overrides)
                    pack_report_path.write_text(
                        json.dumps(pack_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(self, report, expected)
                    diagnostics = report["diagnostics"]
                    self.assertFalse(
                        any(
                            any(fragment in diagnostic for fragment in blocked_fragments)
                            for diagnostic in diagnostics
                        ),
                        diagnostics,
                    )

    def test_report_stage_rejects_invalid_pack_delta_path_before_cross_stage_delta(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_library_embed_reports(out)
            pack_report_path = out / "stages" / "pack" / "report.json"
            pack_report = json.loads(pack_report_path.read_text(encoding="utf-8"))
            pack_report["delta_pack"] = ["pack-output/assets.delta.zrpd"]
            pack_report_path.write_text(
                json.dumps(pack_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report delta_pack must be a string",
            )
            diagnostics = report["diagnostics"]
            self.assertFalse(
                any(
                    "pack report delta_pack must be a non-empty string" in diagnostic
                    or "delta_apply_verified is not true" in diagnostic
                    for diagnostic in diagnostics
                ),
                diagnostics,
            )

    def test_report_stage_rejects_pack_delta_manifest_path_padded_string(
        self,
    ) -> None:
        cases = (
            (
                "base.assets",
                {
                    **_delta_manifest(),
                    "base": _manifest_override(
                        {
                            "assets": [
                                {
                                    **_asset_entry(hash_value=1),
                                    "path": " textures/old.png ",
                                }
                            ]
                        }
                    ),
                },
                "pack report delta_manifest.base.assets[0].path "
                "must be a non-empty trimmed string",
            ),
            (
                "target.assets",
                {
                    **_delta_manifest(),
                    "target": _manifest_override(
                        {
                            "assets": [
                                {
                                    **_asset_entry(hash_value=2),
                                    "path": " scenes/main.zscene ",
                                }
                            ]
                        },
                        hash_value=2,
                    ),
                },
                "pack report delta_manifest.target.assets[0].path "
                "must be a non-empty trimmed string",
            ),
            (
                "changed_assets",
                {
                    **_delta_manifest(),
                    "changed_assets": [
                        {
                            **_asset_entry(hash_value=2),
                            "path": " scenes/main.zscene ",
                        }
                    ],
                },
                "pack report delta_manifest.changed_assets[0].path "
                "must be a non-empty trimmed string",
            ),
        )
        for label, delta_manifest, expected in cases:
            with self.subTest(label=label):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_library_embed_reports(out)
                    _update_pack_report(
                        out,
                        manifest=delta_manifest.get("target", _pack_manifest()),
                        delta_manifest=delta_manifest,
                    )

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(self, report, expected)
                    diagnostics = report["diagnostics"]
                    self.assertFalse(
                        any(
                            "must use a normalized relative asset path" in diagnostic
                            for diagnostic in diagnostics
                        ),
                        diagnostics,
                    )
                    self.assertFalse(
                        any(
                            "delta_manifest.changed_assets does not match" in diagnostic
                            or "delta_manifest.removed_assets does not match" in diagnostic
                            or "delta_removed_assets does not match" in diagnostic
                            for diagnostic in diagnostics
                        ),
                        diagnostics,
                    )

    def test_report_stage_rejects_pack_delta_audit_path_padded_string(
        self,
    ) -> None:
        cases = (
            (
                "delta_removed_assets",
                {"delta_removed_assets": [" textures/old.png "]},
                "pack report delta_removed_assets[0] "
                "must be a non-empty trimmed string",
            ),
            (
                "delta_reused_assets",
                {"delta_reused_assets": [" textures/reused.png "]},
                "pack report delta_reused_assets[0] "
                "must be a non-empty trimmed string",
            ),
            (
                "delta_manifest.removed_assets",
                {
                    "delta_manifest": {
                        **_delta_manifest(),
                        "removed_assets": [" textures/old.png "],
                    }
                },
                "pack report delta_manifest.removed_assets[0] "
                "must be a non-empty trimmed string",
            ),
        )
        for label, overrides, expected in cases:
            with self.subTest(label=label):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_library_embed_reports(out)
                    _update_pack_report(
                        out,
                        manifest=_delta_manifest()["target"],
                        delta_manifest=_delta_manifest(),
                    )
                    pack_report_path = out / "stages" / "pack" / "report.json"
                    pack_report = json.loads(
                        pack_report_path.read_text(encoding="utf-8")
                    )
                    pack_report.update(overrides)
                    pack_report_path.write_text(
                        json.dumps(pack_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(self, report, expected)
                    diagnostics = report["diagnostics"]
                    self.assertFalse(
                        any(
                            "must use a normalized relative asset path" in diagnostic
                            for diagnostic in diagnostics
                        ),
                        diagnostics,
                    )
                    self.assertFalse(
                        any(
                            "delta_manifest.removed_assets does not match" in diagnostic
                            or "delta_removed_assets does not match" in diagnostic
                            or "delta_reused_assets does not match" in diagnostic
                            for diagnostic in diagnostics
                        ),
                        diagnostics,
                    )


if __name__ == "__main__":
    unittest.main()
