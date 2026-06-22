from __future__ import annotations

import tempfile
import unittest
from copy import deepcopy
from pathlib import Path
from typing import Callable

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.platform_bundle_report_test_support import (
    _read_stage_report,
    _write_bundle_manifest_from_platform_report,
    _write_platform_bundle_fixture,
    _write_stage_report,
)


class PlatformBundleTemplateResolutionStringArraySchemaTests(unittest.TestCase):
    def _assert_template_resolution_diagnostic(
        self,
        mutate: Callable[[dict[str, object]], None],
        expected_diagnostic: str,
        broad_diagnostic: str,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out, with_template_file=True)
            platform_report = _read_stage_report(out, "platform_bundle")
            resolution = _template_resolution(out)
            mutate(resolution)
            platform_report["template_resolution"] = resolution
            _write_stage_report(out, "platform_bundle", platform_report)
            _write_bundle_manifest_from_platform_report(
                fixture["bundle_manifest"],
                platform_report,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn(expected_diagnostic, report["diagnostics"])
            self.assertNotIn(broad_diagnostic, report["diagnostics"])

    def test_report_rejects_template_resolution_diagnostic_entry_non_string(
        self,
    ) -> None:
        self._assert_template_resolution_diagnostic(
            lambda resolution: resolution.__setitem__(
                "diagnostics",
                ["template skipped", 42],
            ),
            "PlatformBundle report template_resolution.diagnostics[1] "
            "must be a string",
            "PlatformBundle report template_resolution.diagnostics "
            "must be a string array",
        )

    def test_report_rejects_template_resolution_candidate_profile_entry_non_string(
        self,
    ) -> None:
        self._assert_template_resolution_diagnostic(
            lambda resolution: resolution["candidates"][0].__setitem__(
                "compatible_profiles",
                ["windows-release", 42],
            ),
            "PlatformBundle report template_resolution candidates[0]."
            "compatible_profiles[1] must be a string",
            "PlatformBundle report template_resolution candidates[0]."
            "compatible_profiles must be a string array",
        )

    def test_report_rejects_template_resolution_skipped_diagnostic_entry_non_string(
        self,
    ) -> None:
        self._assert_template_resolution_diagnostic(
            lambda resolution: resolution["skipped_candidates"][0].__setitem__(
                "diagnostics",
                ["template skipped", 42],
            ),
            "PlatformBundle report template_resolution skipped_candidates[0]."
            "diagnostics[1] must be a string",
            "PlatformBundle report template_resolution skipped_candidates[0]."
            "diagnostics must be a string array",
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
