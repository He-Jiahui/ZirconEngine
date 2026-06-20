from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.platform_bundle_report_test_support import (
    _write_platform_bundle_fixture,
)


class PlatformBundleReportFileReadTests(unittest.TestCase):
    def test_report_rejects_platform_host_output_read_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            unreadable_file = fixture["platform_host"].resolve()

            report = build_report_with_read_failure(
                out,
                unreadable_file,
                "simulated host output read failure",
            )

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "PlatformBundle report host_executable" in diagnostic
                    and "could not be read" in diagnostic
                    and "simulated host output read failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_platform_host_source_read_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            unreadable_file = fixture["host_source"].resolve()

            report = build_report_with_read_failure(
                out,
                unreadable_file,
                "simulated host source read failure",
            )

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "PlatformBundle report host_source" in diagnostic
                    and "could not be read" in diagnostic
                    and "simulated host source read failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_template_file_read_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out, with_template_file=True)
            unreadable_file = fixture["template_file"].resolve()

            report = build_report_with_read_failure(
                out,
                unreadable_file,
                "simulated template file read failure",
            )

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "PlatformBundle report template_files destination" in diagnostic
                    and "could not be read" in diagnostic
                    and "simulated template file read failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )


def build_report_with_read_failure(
    out: Path,
    unreadable_file: Path,
    message: str,
) -> dict[str, object]:
    original_read_bytes = Path.read_bytes

    def read_bytes_or_fail(path: Path) -> bytes:
        if path.resolve() == unreadable_file:
            raise OSError(message)
        return original_read_bytes(path)

    with mock.patch.object(Path, "read_bytes", read_bytes_or_fail):
        return build_pipeline_report(out, "windows-release")


if __name__ == "__main__":
    unittest.main()
