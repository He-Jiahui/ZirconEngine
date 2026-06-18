from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.pipeline_report_platform_bundle import (
    path_is_relative_to,
    platform_bundle_report_bundle_path,
    platform_bundle_template_file_expected_hash,
    platform_bundle_template_file_hashes,
)
from tools.zircon_export.tests.test_pipeline_report_platform_bundle import (
    _write_platform_bundle_fixture,
)


class PlatformBundleReportPathResolveErrorsTests(unittest.TestCase):
    def test_report_rejects_host_pack_delta_evidence_resolve_errors(self) -> None:
        for field, fixture_key in (
            ("host_executable", "host_source"),
            ("pack", "pack_source"),
            ("delta_pack", "delta_source"),
        ):
            with self.subTest(field=field):
                self.assert_report_rejects_resolve_error(field, fixture_key)

    def assert_report_rejects_resolve_error(
        self,
        field: str,
        fixture_key: str,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out, with_delta=True)
            failing_path = fixture[fixture_key]

            report = build_report_with_resolve_failure(
                out,
                failing_path,
                f"simulated {field} resolve failure",
            )

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "could not be resolved" in diagnostic
                    and f"simulated {field} resolve failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_template_file_destination_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out, with_template_file=True)

            report = build_report_with_resolve_failure(
                out,
                fixture["template_file"],
                "simulated template destination resolve failure",
            )

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "PlatformBundle report template_files destination" in diagnostic
                    and "could not be resolved" in diagnostic
                    and "simulated template destination resolve failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_path_is_relative_to_treats_resolve_error_as_outside(self) -> None:
        root = Path("bundle")
        child = root / "Contents" / "Info.plist"
        original_resolve = Path.resolve

        def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
            if path == child:
                raise OSError("simulated relative path resolve failure")
            return original_resolve(path, *args, **kwargs)

        with mock.patch.object(Path, "resolve", resolve_or_fail):
            self.assertFalse(path_is_relative_to(child, root))

    def test_report_rejects_payload_containment_path_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out, with_delta=True)
            failing_path = fixture["platform_host"]
            original_resolve = Path.resolve
            resolve_count = 0

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                nonlocal resolve_count
                if str(path) == str(failing_path):
                    resolve_count += 1
                    if resolve_count > 1:
                        raise OSError(
                            "simulated PlatformBundle payload containment failure"
                        )
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "PlatformBundle report host_executable" in diagnostic
                    and "could not be resolved" in diagnostic
                    and "simulated PlatformBundle payload containment failure"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_payload_containment_parent_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out, with_delta=True)
            failing_path = out / "bundle" / "windows-release"
            original_resolve = Path.resolve
            resolve_count = 0

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                nonlocal resolve_count
                if str(path) == str(failing_path):
                    resolve_count += 1
                    if resolve_count > 2:
                        raise OSError(
                            "simulated PlatformBundle bundle containment failure"
                        )
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "PlatformBundle bundle_manifest" in diagnostic
                    and "PlatformBundle bundle" in diagnostic
                    and "could not be resolved" in diagnostic
                    and "simulated PlatformBundle bundle containment failure"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_bundle_path_rejects_stage_path_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            bundle = root / "out" / "bundle" / "windows-release"
            bundle.mkdir(parents=True)
            stage_report_path = root / "out" / "stages" / "platform_bundle" / "report.json"
            report = {
                "profile": "windows-release",
                "bundle": str(bundle),
            }
            stage_report = {
                "path": str(stage_report_path),
            }
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if str(path) == str(stage_report_path):
                    raise OSError("simulated PlatformBundle stage path failure")
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                bundle_path, diagnostics = platform_bundle_report_bundle_path(
                    report,
                    stage_report,
                )

            self.assertIsNone(bundle_path)
            self.assertTrue(
                any(
                    "PlatformBundle stage report path" in diagnostic
                    and "could not be resolved" in diagnostic
                    and "simulated PlatformBundle stage path failure" in diagnostic
                    for diagnostic in diagnostics
                ),
                diagnostics,
            )

    def test_template_file_hashes_rejects_declared_path_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            template_dir = Path(temp_dir) / "template"
            template_dir.mkdir()
            declared_file = template_dir / "Info.plist"
            template = {
                "template_dir": str(template_dir),
                "files": [
                    {
                        "path": declared_file.name,
                        "sha256": "abc123",
                    }
                ],
            }
            diagnostics: list[str] = []
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if str(path) == str(declared_file):
                    raise OSError("simulated template file path failure")
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                hashes = platform_bundle_template_file_hashes(template, diagnostics)

            self.assertEqual(hashes, {})
            self.assertTrue(
                any(
                    "PlatformBundle report template.files entry 0 path" in diagnostic
                    and "could not be resolved" in diagnostic
                    and "simulated template file path failure" in diagnostic
                    for diagnostic in diagnostics
                ),
                diagnostics,
            )

    def test_template_file_expected_hash_rejects_source_resolve_error(self) -> None:
        source = Path("template") / "Info.plist"
        entry = {
            "source": str(source),
        }
        diagnostics: list[str] = []
        original_resolve = Path.resolve

        def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
            if path == source:
                raise OSError("simulated template source path failure")
            return original_resolve(path, *args, **kwargs)

        with mock.patch.object(Path, "resolve", resolve_or_fail):
            expected_hash = platform_bundle_template_file_expected_hash(
                entry,
                {},
                diagnostics,
            )

        self.assertIsNone(expected_hash)
        self.assertTrue(
            any(
                "PlatformBundle report template_files source" in diagnostic
                and "could not be resolved" in diagnostic
                and "simulated template source path failure" in diagnostic
                for diagnostic in diagnostics
            ),
            diagnostics,
        )


def build_report_with_resolve_failure(
    out: Path,
    failing_path: Path,
    message: str,
) -> dict[str, object]:
    original_resolve = Path.resolve

    def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
        if str(path) == str(failing_path):
            raise OSError(message)
        return original_resolve(path, *args, **kwargs)

    with mock.patch.object(Path, "resolve", resolve_or_fail):
        return build_pipeline_report(out, "windows-release")


if __name__ == "__main__":
    unittest.main()
