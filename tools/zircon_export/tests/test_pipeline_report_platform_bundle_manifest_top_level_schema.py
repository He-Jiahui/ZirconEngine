from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.platform_bundle_report_test_support import (
    _write_platform_bundle_fixture,
)


class PlatformBundleManifestTopLevelSchemaTests(unittest.TestCase):
    def test_report_rejects_bundle_manifest_required_path_blank_string(
        self,
    ) -> None:
        cases = (
            "profile",
            "host_executable",
            "host_source",
            "host_source_origin",
            "pack",
            "pack_source",
            "pack_source_origin",
        )
        for field in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    fixture = _write_platform_bundle_fixture(out)
                    manifest = json.loads(
                        fixture["bundle_manifest"].read_text(encoding="utf-8")
                    )
                    manifest[field] = " "
                    fixture["bundle_manifest"].write_text(
                        json.dumps(manifest, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertTrue(
                        any(
                            f"PlatformBundle bundle_manifest {field} must be a non-empty string"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )


if __name__ == "__main__":
    unittest.main()
