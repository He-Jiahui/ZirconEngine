from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.export_template import compute_template_content_hash
from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.platform_bundle_report_test_support import (
    _read_stage_report,
    _write_bundle_manifest_from_platform_report,
    _write_platform_bundle_fixture,
    _write_stage_report,
)


class PlatformBundleTemplateManifestFileSchemaTests(unittest.TestCase):
    def test_report_rejects_template_report_manifest_file_bundle_path_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(
                out,
                with_template_file=True,
            )
            platform_report = _read_stage_report(out, "platform_bundle")
            template = platform_report["template"]
            self.assertIsInstance(template, dict)
            template_dir = Path(str(template["template_dir"]))
            files = template["files"]
            self.assertIsInstance(files, list)
            file_entry = files[0]
            self.assertIsInstance(file_entry, dict)
            manifest = template_dir / "template.toml"
            manifest.write_text(
                (
                    self._template_manifest_header(file_entry)
                    +
                    "[[files]]\n"
                    f'path = "{file_entry["path"]}"\n'
                    f'sha256 = "{file_entry["sha256"]}"\n'
                    'bundle_path = "Contents/OtherInfo.plist"\n'
                    f'purpose = "{file_entry["purpose"]}"\n'
                ),
                encoding="utf-8",
            )
            template["manifest"] = str(manifest)
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
                    "PlatformBundle report template.manifest files[0].bundle_path "
                    "Contents/OtherInfo.plist does not match "
                    "template.files[0].bundle_path Contents/Info.plist" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    @staticmethod
    def _template_manifest_header(file_entry: dict[str, object]) -> str:
        content_hash = compute_template_content_hash(
            [
                {
                    "path": str(file_entry["path"]),
                    "bundle_path": str(file_entry["bundle_path"]),
                    "sha256": str(file_entry["sha256"]),
                }
            ]
        )
        return (
            "format_version = 1\n"
            'template_id = "fixture-template"\n'
            'engine_version = "0.1.0"\n'
            'target_platform = "windows-x86_64"\n'
            'host_kind = "desktop"\n'
            'host_artifact = "precompiled"\n'
            'resource_strategy = "filesystem_bundle"\n'
            'plugin_strategy = "native_dynamic_allowed"\n'
            'bundle_format = "directory"\n'
            f'content_hash = "{content_hash}"\n'
            "\n[paths]\n"
            f'host_executable = "{file_entry["path"]}"\n'
        )


if __name__ == "__main__":
    unittest.main()
