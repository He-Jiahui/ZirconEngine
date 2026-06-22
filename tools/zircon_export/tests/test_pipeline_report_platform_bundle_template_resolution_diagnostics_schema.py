from __future__ import annotations

import tempfile
import unittest
from copy import deepcopy
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.platform_bundle_report_test_support import (
    _read_stage_report,
    _write_bundle_manifest_from_platform_report,
    _write_platform_bundle_fixture,
    _write_stage_report,
)


class PlatformBundleTemplateResolutionDiagnosticsSchemaTests(unittest.TestCase):
    def test_report_rejects_template_resolution_padded_diagnostic_entry(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out, with_template_file=True)
            platform_report = _read_stage_report(out, "platform_bundle")
            resolution = _template_resolution(out)
            resolution["fatal"] = True
            resolution["template_dir"] = None
            resolution["diagnostics"] = [" forced fatal "]
            platform_report["template_resolution"] = resolution
            _write_stage_report(out, "platform_bundle", platform_report)
            _write_bundle_manifest_from_platform_report(
                fixture["bundle_manifest"],
                platform_report,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "PlatformBundle report template_resolution.diagnostics[0] "
                    "must be a non-empty trimmed string" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_template_resolution_skipped_candidate_padded_diagnostic_entry(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out, with_template_file=True)
            platform_report = _read_stage_report(out, "platform_bundle")
            resolution = _template_resolution(out)
            skipped_candidates = resolution["skipped_candidates"]
            self.assertIsInstance(skipped_candidates, list)
            skipped_candidate = skipped_candidates[0]
            self.assertIsInstance(skipped_candidate, dict)
            skipped_candidate["diagnostics"] = [
                " template format_version 999 is not supported "
            ]
            platform_report["template_resolution"] = resolution
            _write_stage_report(out, "platform_bundle", platform_report)
            _write_bundle_manifest_from_platform_report(
                fixture["bundle_manifest"],
                platform_report,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "PlatformBundle report template_resolution "
                    "skipped_candidates[0].diagnostics[0] "
                    "must be a non-empty trimmed string" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )


def _template_resolution(out: Path) -> dict[str, object]:
    template_root = out
    template_dir = out / "template"
    candidate = {
        "template_dir": str(template_dir),
        "template_id": "windows-template",
        "engine_version": "0.1.0",
        "target_platform": "windows-x86_64",
        "compatible_profiles": ["windows-release"],
        "bundle_format": "directory",
    }
    return {
        "template_root": str(template_root),
        "profile": "windows-release",
        "expected_engine_version": "0.1.0",
        "expected_target_platform": "windows-x86_64",
        "fatal": False,
        "diagnostics": [],
        "candidates": [deepcopy(candidate)],
        "skipped_candidates": [
            {
                "template_dir": str(out / "broken-template"),
                "diagnostics": ["template format_version 999 is not supported"],
            }
        ],
        "template_dir": str(template_dir),
    }


if __name__ == "__main__":
    unittest.main()
