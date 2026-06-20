from __future__ import annotations

import hashlib
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


class PlatformBundleTemplateFileSchemaTests(unittest.TestCase):
    def test_report_rejects_template_file_duplicate_path(
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
            files = template["files"]
            self.assertIsInstance(files, list)
            first_entry = files[0]
            self.assertIsInstance(first_entry, dict)
            files.append(dict(first_entry))
            content_hash = compute_template_content_hash(files)
            template["content_hash"] = content_hash
            template["computed_content_hash"] = content_hash
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
                    "PlatformBundle report template.files[1].path duplicates "
                    "PlatformBundle report template.files[0].path" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_template_file_duplicate_bundle_path(
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
            template_dir = template["template_dir"]
            self.assertIsInstance(template_dir, str)
            files = template["files"]
            self.assertIsInstance(files, list)
            first_entry = files[0]
            self.assertIsInstance(first_entry, dict)
            second_payload = b"<plist>other</plist>"
            second_template_file = Path(template_dir) / "Other.plist"
            second_template_file.write_bytes(second_payload)
            files.append(
                {
                    "path": second_template_file.name,
                    "bundle_path": first_entry["bundle_path"],
                    "sha256": hashlib.sha256(second_payload).hexdigest(),
                    "purpose": "platform_metadata",
                }
            )
            content_hash = compute_template_content_hash(files)
            template["content_hash"] = content_hash
            template["computed_content_hash"] = content_hash
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
                    "PlatformBundle report template.files[1].bundle_path duplicates "
                    "PlatformBundle report template.files[0].bundle_path" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_template_file_unsafe_relative_path(
        self,
    ) -> None:
        for field in ("bundle_path", "path"):
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
                    files = template["files"]
                    self.assertIsInstance(files, list)
                    entry = files[0]
                    self.assertIsInstance(entry, dict)
                    entry[field] = "../Info.plist"
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
                            f"PlatformBundle report template.files[0].{field} "
                            "must be a safe relative path" in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_rejects_template_file_malformed_sha256(
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
            files = template["files"]
            self.assertIsInstance(files, list)
            entry = files[0]
            self.assertIsInstance(entry, dict)
            entry["sha256"] = "not-a-hash"
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
                    "PlatformBundle report template.files[0].sha256 "
                    "must be a SHA-256 hex digest" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_template_file_source_hash_mismatch(
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
            template_dir = template["template_dir"]
            self.assertIsInstance(template_dir, str)
            files = template["files"]
            self.assertIsInstance(files, list)
            entry = files[0]
            self.assertIsInstance(entry, dict)
            template_source = Path(template_dir) / str(entry["path"])
            template_source.write_text("<plist>tampered</plist>", encoding="utf-8")
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
                    "PlatformBundle report template.files[0].sha256 "
                    in diagnostic
                    and "does not match actual" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_copied_template_file_destination_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(
                out,
                with_template_file=True,
            )
            wrong_destination = fixture["template_file"].parent / "WrongInfo.plist"
            wrong_destination.write_bytes(fixture["template_file"].read_bytes())
            platform_report = _read_stage_report(out, "platform_bundle")
            template_files = platform_report["template_files"]
            self.assertIsInstance(template_files, list)
            entry = template_files[0]
            self.assertIsInstance(entry, dict)
            entry["destination"] = str(wrong_destination)
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
                    "PlatformBundle report template_files[0].destination "
                    "does not match template.files[0].bundle_path" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_duplicate_copied_template_file_entry(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(
                out,
                with_template_file=True,
            )
            platform_report = _read_stage_report(out, "platform_bundle")
            template_files = platform_report["template_files"]
            self.assertIsInstance(template_files, list)
            first_entry = template_files[0]
            self.assertIsInstance(first_entry, dict)
            template_files.append(dict(first_entry))
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
                    "PlatformBundle report template_files[1] duplicates "
                    "PlatformBundle report template_files[0]" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_template_file_required_string_blank(
        self,
    ) -> None:
        for field in ("bundle_path", "path", "sha256"):
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
                    files = template["files"]
                    self.assertIsInstance(files, list)
                    entry = files[0]
                    self.assertIsInstance(entry, dict)
                    entry[field] = " "
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
                            f"PlatformBundle report template.files[0].{field} must be a non-empty string"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_rejects_template_file_purpose_blank_when_present(
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
            files = template["files"]
            self.assertIsInstance(files, list)
            entry = files[0]
            self.assertIsInstance(entry, dict)
            entry["purpose"] = "   "
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
                    "PlatformBundle report template.files[0].purpose "
                    "must be non-empty when present" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_template_copied_file_required_string_blank(
        self,
    ) -> None:
        for field in ("destination", "source"):
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    fixture = _write_platform_bundle_fixture(
                        out,
                        with_template_file=True,
                    )
                    platform_report = _read_stage_report(out, "platform_bundle")
                    template_files = platform_report["template_files"]
                    self.assertIsInstance(template_files, list)
                    entry = template_files[0]
                    self.assertIsInstance(entry, dict)
                    entry[field] = " "
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
                            f"PlatformBundle report template_files[0].{field} must be a non-empty string"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )


if __name__ == "__main__":
    unittest.main()
