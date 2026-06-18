from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.pipeline_report_native_dynamic_payload import (
    current_output_native_dynamic_report_path,
    platform_bundle_native_plugins_package_path_diagnostics,
)
from tools.zircon_export.native_dynamic_payload import (
    materialized_package_loadable_artifacts_match_manifest,
)
from tools.zircon_export.tests.test_pipeline_report_platform_bundle import (
    _read_stage_report,
    _write_platform_bundle_fixture,
    _write_stage_report,
)


class NativeDynamicPayloadPathResolveErrorsTests(unittest.TestCase):
    def test_report_rejects_native_plugins_payload_path_resolve_errors(self) -> None:
        for label, failing_path in self.payload_paths(Path("placeholder")).items():
            with self.subTest(label=label):
                self.assert_payload_path_resolve_error_becomes_diagnostic(label)

    def assert_payload_path_resolve_error_becomes_diagnostic(self, label: str) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            failing_path = self.payload_paths(out, fixture)[label]

            report = build_report_with_resolve_failure(
                out,
                failing_path,
                f"simulated {label} resolve failure",
            )

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "native_plugins_payload" in diagnostic
                    or "native_plugins" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertTrue(
                any(
                    "could not be resolved" in diagnostic
                    and f"simulated {label} resolve failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def payload_paths(
        self,
        out: Path,
        fixture: dict[str, Path] | None = None,
    ) -> dict[str, Path]:
        return {
            "native_plugins": (
                fixture["native_plugins"]
                if fixture is not None
                else out / "bundle" / "windows-release" / "plugins"
            ),
            "payload_stage_report": out / "stages" / "native_dynamic" / "report.json",
            "payload_source": out / "stages" / "native_dynamic" / "plugins",
        }

    def test_package_path_diagnostics_rejects_plugins_root_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            plugins_dir = root / "bundle" / "windows-release" / "plugins"
            package_dir = plugins_dir / "animation"
            package_report = package_dir / "native_dynamic_package.toml"
            packages = [
                {
                    "package_id": "animation",
                    "destination": str(package_dir),
                    "package_report": str(package_report),
                    "loadable_artifact_count": 0,
                    "loadable_artifacts": [],
                }
            ]
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if str(path) == str(plugins_dir):
                    raise OSError("simulated native plugins package root failure")
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                diagnostics = platform_bundle_native_plugins_package_path_diagnostics(
                    packages,
                    plugins_dir,
                )

            self.assertTrue(
                any(
                    "PlatformBundle report native_plugins" in diagnostic
                    and "could not be resolved" in diagnostic
                    and "simulated native plugins package root failure" in diagnostic
                    for diagnostic in diagnostics
                ),
                diagnostics,
            )

    def test_current_output_stage_report_path_reports_plugins_dir_resolve_error(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            native_plugins = Path(temp_dir) / "out" / "bundle" / "windows-release" / "plugins"
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if str(path) == str(native_plugins):
                    raise OSError(
                        "simulated NativeDynamic expected report path failure"
                    )
                return original_resolve(path, *args, **kwargs)

            diagnostics: list[str] = []
            with mock.patch.object(Path, "resolve", resolve_or_fail):
                report_path = current_output_native_dynamic_report_path(
                    native_plugins,
                    diagnostics,
                )

            self.assertIsNone(report_path)
            self.assertTrue(
                any(
                    "PlatformBundle report native_plugins" in diagnostic
                    and "could not be resolved" in diagnostic
                    and "simulated NativeDynamic expected report path failure"
                    in diagnostic
                    for diagnostic in diagnostics
                ),
                diagnostics,
            )

    def test_package_path_diagnostics_rejects_destination_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            plugins_dir = root / "bundle" / "windows-release" / "plugins"
            package_dir = plugins_dir / "animation"
            package_report = package_dir / "native_dynamic_package.toml"
            packages = [
                {
                    "package_id": "animation",
                    "destination": str(package_dir),
                    "package_report": str(package_report),
                    "loadable_artifact_count": 0,
                    "loadable_artifacts": [],
                }
            ]
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if str(path) == str(package_dir):
                    raise OSError("simulated native plugins destination failure")
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                diagnostics = platform_bundle_native_plugins_package_path_diagnostics(
                    packages,
                    plugins_dir,
                )

            self.assertTrue(
                any(
                    "materialized_packages[0] destination" in diagnostic
                    and "could not be resolved" in diagnostic
                    and "simulated native plugins destination failure" in diagnostic
                    for diagnostic in diagnostics
                ),
                diagnostics,
            )

    def test_package_path_diagnostics_rejects_package_report_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            plugins_dir = root / "bundle" / "windows-release" / "plugins"
            package_dir = plugins_dir / "animation"
            package_report = package_dir / "native_dynamic_package.toml"
            packages = [
                {
                    "package_id": "animation",
                    "destination": str(package_dir),
                    "package_report": str(package_report),
                    "loadable_artifact_count": 0,
                    "loadable_artifacts": [],
                }
            ]
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if str(path) == str(package_report):
                    raise OSError("simulated native plugins package report failure")
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                diagnostics = platform_bundle_native_plugins_package_path_diagnostics(
                    packages,
                    plugins_dir,
                )

            self.assertTrue(
                any(
                    "materialized_packages[0] package_report" in diagnostic
                    and "could not be resolved" in diagnostic
                    and "simulated native plugins package report failure" in diagnostic
                    for diagnostic in diagnostics
                ),
                diagnostics,
            )

    def test_loadable_artifact_manifest_match_reports_destination_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            plugins_dir = root / "bundle" / "windows-release" / "plugins"
            package_dir = plugins_dir / "animation"
            file_manifest = [
                {
                    "path": "plugins/animation/native/zircon_plugin_animation.dll",
                    "bytes": 1,
                    "sha256": "0" * 64,
                }
            ]
            materialized_packages = [
                {
                    "package_id": "animation",
                    "destination": str(package_dir),
                    "loadable_artifact_count": 1,
                    "loadable_artifacts": [
                        "plugins/animation/native/zircon_plugin_animation.dll"
                    ],
                }
            ]
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if str(path) == str(package_dir):
                    raise OSError("simulated loadable artifact destination failure")
                return original_resolve(path, *args, **kwargs)

            diagnostics: list[str] = []
            with mock.patch.object(Path, "resolve", resolve_or_fail):
                matches = materialized_package_loadable_artifacts_match_manifest(
                    materialized_packages,
                    file_manifest,
                    plugins_dir,
                    diagnostics,
                )

            self.assertFalse(matches)
            self.assertTrue(
                any(
                    "materialized_packages[0] destination" in diagnostic
                    and "could not be resolved" in diagnostic
                    and "simulated loadable artifact destination failure" in diagnostic
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
