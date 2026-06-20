from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.platform_bundle_report_test_support import (
    _read_stage_report,
    _write_bundle_manifest_from_platform_report,
    _write_platform_bundle_fixture,
    _write_stage_report,
)


class PlatformBundleTemplateBundleSchemaTests(unittest.TestCase):
    def test_report_rejects_template_bundle_unsafe_relative_path(
        self,
    ) -> None:
        for field in (
            "delta_pack_path",
            "host_path",
            "manifest_path",
            "pack_path",
            "root",
        ):
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    fixture = _write_platform_bundle_fixture(
                        out,
                        with_template_file=True,
                    )
                    platform_report = _read_stage_report(out, "platform_bundle")
                    template = platform_report["template"]
                    self.assertIsInstance(template, dict)
                    template["bundle"] = {
                        "delta_pack_path": "",
                        "host_path": "",
                        "manifest_path": "bundle.json",
                        "pack_path": "",
                        "root": ".",
                    }
                    bundle = template["bundle"]
                    self.assertIsInstance(bundle, dict)
                    bundle[field] = "../escape"
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
                            f"PlatformBundle report template.bundle.{field} "
                            "must be a safe relative path" in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )


if __name__ == "__main__":
    unittest.main()
