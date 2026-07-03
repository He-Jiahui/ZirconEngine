from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.native_dynamic_payload_file_manifest import (
    native_dynamic_plugins_bundle_file_manifest,
)
from tools.zircon_export.tests.export_test_support import (
    _export_args,
    _run_stage_quiet,
    _write_validate_report_with_native_dynamic_exports,
    json_loads,
)
from tools.zircon_export.tests.native_dynamic_export_test_support import (
    _write_native_dynamic_package_fixture,
    _write_native_dynamic_stage_plugins,
)
from tools.zircon_export.tests.platform_bundle_report_test_support import (
    _write_platform_bundle_fixture,
)


class NativeDynamicPayloadFileReadTests(unittest.TestCase):
    def test_native_dynamic_stage_rejects_package_payload_read_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root)
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(out)
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)
            unreadable_file = (
                out
                / "stages"
                / "native_dynamic"
                / "plugins"
                / "animation"
                / "native"
                / "zircon_plugin_animation.dll"
            ).resolve()

            exit_code = run_stage_with_read_failure(
                args,
                unreadable_file,
                "simulated native package read failure",
            )

            report = json_loads(
                (
                    out / "stages" / "native_dynamic" / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(report["payload_cleaned"])
            self.assertEqual(report["cleanup_reason"], "fatal_diagnostics")
            self.assertTrue(
                any(
                    "NativeDynamic payload file"
                    in diagnostic
                    and "zircon_plugin_animation.dll" in diagnostic
                    and "could not be read" in diagnostic
                    and "simulated native package read failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse(
                (out / "stages" / "native_dynamic" / "plugins" / "animation").exists()
            )
            self.assertFalse(
                (
                    out
                    / "stages"
                    / "native_dynamic"
                    / "plugins"
                    / "native_plugins.toml"
                ).exists()
            )

    def test_native_dynamic_stage_rejects_package_payload_listing_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root)
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(out)
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)
            package_dir = (
                out / "stages" / "native_dynamic" / "plugins" / "animation"
            ).resolve()
            original_rglob = Path.rglob

            def rglob_or_fail(path: Path, *args: object, **kwargs: object) -> object:
                if path.resolve() == package_dir:
                    raise OSError("simulated native package listing failure")
                return original_rglob(path, *args, **kwargs)

            with mock.patch.object(Path, "rglob", rglob_or_fail):
                exit_code = _run_stage_quiet(args)

            report = json_loads(
                (
                    out / "stages" / "native_dynamic" / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(report["payload_cleaned"])
            self.assertEqual(report["cleanup_reason"], "fatal_diagnostics")
            self.assertTrue(
                any(
                    "NativeDynamic payload directory"
                    in diagnostic
                    and "animation" in diagnostic
                    and "could not be listed" in diagnostic
                    and "simulated native package listing failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse(package_dir.exists())

    def test_native_dynamic_stage_rejects_package_destination_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root)
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(out)
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)
            package_dir = out / "stages" / "native_dynamic" / "plugins" / "animation"
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if str(path) == str(package_dir):
                    raise OSError("simulated native package resolve failure")
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                exit_code = _run_stage_quiet(args)

            report = json_loads(
                (
                    out / "stages" / "native_dynamic" / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(report["payload_cleaned"])
            self.assertEqual(report["cleanup_reason"], "fatal_diagnostics")
            self.assertTrue(
                any(
                    "native dynamic package directory" in diagnostic
                    and "could not be resolved" in diagnostic
                    and "simulated native package resolve failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse(package_dir.exists())

    def test_report_rejects_native_plugins_payload_file_read_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            unreadable_file = (
                fixture["native_plugins"]
                / "animation"
                / "native"
                / "zircon_plugin_animation.dll"
            ).resolve()

            report = build_report_with_read_failure(
                out,
                unreadable_file,
                "simulated bundled native plugin read failure",
            )

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "NativeDynamic payload file"
                    in diagnostic
                    and "zircon_plugin_animation.dll" in diagnostic
                    and "could not be read" in diagnostic
                    and "simulated bundled native plugin read failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_native_dynamic_payload_bundle_manifest_rejects_source_resolve_error(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            plugins_dir = _write_native_dynamic_stage_plugins(root / "native-source")
            original_resolve = Path.resolve
            diagnostics: list[str] = []

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if str(path) == str(plugins_dir):
                    raise OSError("simulated native plugins manifest root failure")
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                file_manifest = native_dynamic_plugins_bundle_file_manifest(
                    plugins_dir,
                    diagnostics,
                )

            self.assertEqual(file_manifest, [])
            self.assertTrue(
                any(
                    "NativeDynamic payload source" in diagnostic
                    and "could not be resolved" in diagnostic
                    and "simulated native plugins manifest root failure" in diagnostic
                    for diagnostic in diagnostics
                ),
                diagnostics,
            )


def run_stage_with_read_failure(
    args: object,
    unreadable_file: Path,
    message: str,
) -> int:
    original_read_bytes = Path.read_bytes

    def read_bytes_or_fail(path: Path) -> bytes:
        if path.resolve() == unreadable_file:
            raise OSError(message)
        return original_read_bytes(path)

    with mock.patch.object(Path, "read_bytes", read_bytes_or_fail):
        return _run_stage_quiet(args)


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
