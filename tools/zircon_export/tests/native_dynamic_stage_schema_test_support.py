from __future__ import annotations

import json
import tempfile
from collections.abc import Callable
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.export_test_support import (
    _write_compile_host_report,
    _write_pack_report,
    _write_stage_report,
    _write_validate_report_with_native_dynamic_exports,
)
from tools.zircon_export.tests.native_dynamic_export_test_support import (
    _write_native_dynamic_report,
    _write_native_dynamic_stage_plugins,
)


class NativeDynamicStageSchemaReportAssertions:
    def _write_native_dynamic_reports(self, out: Path) -> Path:
        _write_validate_report_with_native_dynamic_exports(out)
        native_plugins = _write_native_dynamic_stage_plugins(
            out / "stages" / "native_dynamic"
        )
        _write_native_dynamic_report(out, native_plugins)
        _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
        _write_stage_report(out, "cook_assets", fatal=False)
        _write_pack_report(out, out / "pack-output" / "assets.zrpack")
        _write_stage_report(out, "platform_bundle", fatal=False)
        return out / "stages" / "native_dynamic" / "report.json"

    def _assert_native_dynamic_report_field_diagnostic(
        self,
        field: str,
        value: object,
        expected_diagnostic: str,
        unexpected_diagnostic: str | None = None,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = self._write_native_dynamic_reports(out)
            native_report = json.loads(
                native_report_path.read_text(encoding="utf-8")
            )
            native_report[field] = value
            native_report_path.write_text(
                json.dumps(native_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("NativeDynamic", report["fatal_stages"])
            self.assertTrue(
                any(
                    expected_diagnostic in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            if unexpected_diagnostic is not None:
                self.assertFalse(
                    any(
                        unexpected_diagnostic in diagnostic
                        for diagnostic in report["diagnostics"]
                    ),
                    report["diagnostics"],
                )

    def _assert_native_dynamic_report_mutation_diagnostic(
        self,
        mutate_report: Callable[[dict[str, object]], None],
        expected_diagnostic: str | tuple[str, ...],
    ) -> None:
        expected_diagnostics = (
            (expected_diagnostic,)
            if isinstance(expected_diagnostic, str)
            else expected_diagnostic
        )
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = self._write_native_dynamic_reports(out)
            native_report = json.loads(
                native_report_path.read_text(encoding="utf-8")
            )
            mutate_report(native_report)
            native_report_path.write_text(
                json.dumps(native_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("NativeDynamic", report["fatal_stages"])
            for expected in expected_diagnostics:
                self.assertTrue(
                    any(
                        expected in diagnostic
                        for diagnostic in report["diagnostics"]
                    ),
                    report["diagnostics"],
                )


def _native_build_plan(**overrides: object) -> dict[str, object]:
    plan = {
        "fatal": False,
        "diagnostics": [],
        "workspace_manifest": "zircon_plugins/Cargo.toml",
        "target_dir": "target/native_dynamic",
        "cargo_profile": "release",
        "release": True,
        "build_features": ["v3_fixture_diagnostics"],
        "package_count": 1,
        "packages": [_native_build_plan_package()],
    }
    plan.update(overrides)
    return plan


def _native_build_plan_without(field: str) -> dict[str, object]:
    plan = _native_build_plan()
    plan.pop(field, None)
    return plan


def _native_build_plan_package(**overrides: object) -> dict[str, object]:
    package = {
        "package_id": "animation",
        "crate_name": "zircon_plugin_animation_native",
        "manifest_path": "zircon_plugins/animation/native/Cargo.toml",
        "workspace_manifest": "zircon_plugins/Cargo.toml",
        "target_dir": "target/native_dynamic",
        "cargo_profile": "release",
        "release": True,
        "features": ["v3_fixture_diagnostics"],
        "command": [
            "cargo",
            "build",
            "--manifest-path",
            "zircon_plugins/Cargo.toml",
            "-p",
            "zircon_plugin_animation_native",
            "--target-dir",
            "target/native_dynamic",
            "--features",
            "v3_fixture_diagnostics",
            "--release",
        ],
        "expected_loadable_artifact": (
            "target/native_dynamic/release/zircon_plugin_animation_native.dll"
        ),
    }
    package.update(overrides)
    return package


def _native_build_plan_package_without(field: str) -> dict[str, object]:
    package = _native_build_plan_package()
    package.pop(field, None)
    return package


def _native_build_execution(**overrides: object) -> dict[str, object]:
    execution = {
        "enabled": True,
        "fatal": False,
        "skipped": False,
        "diagnostics": [],
        "package_count": 1,
        "packages": [_native_build_execution_package()],
    }
    execution.update(overrides)
    return execution


def _native_build_execution_without(field: str) -> dict[str, object]:
    execution = _native_build_execution()
    execution.pop(field, None)
    return execution


def _native_build_execution_package(**overrides: object) -> dict[str, object]:
    package = {
        "package_id": "animation",
        "crate_name": "zircon_plugin_animation_native",
        "command": [
            "cargo",
            "build",
            "--manifest-path",
            "zircon_plugins/Cargo.toml",
            "-p",
            "zircon_plugin_animation_native",
            "--target-dir",
            "target/native_dynamic",
            "--features",
            "v3_fixture_diagnostics",
            "--release",
        ],
        "exit_code": 0,
        "stdout": "",
        "stderr": "",
        "expected_loadable_artifact": (
            "target/native_dynamic/release/zircon_plugin_animation_native.dll"
        ),
        "copied_loadable_artifact": "plugins/animation/native/plugin.dll",
        "copied_sidecars": ["plugins/animation/native/plugin.pdb"],
    }
    package.update(overrides)
    return package


def _native_build_execution_package_for_default_report(
    **overrides: object,
) -> dict[str, object]:
    return _native_build_execution_package(
        command=[
            "cargo",
            "build",
            "--manifest-path",
            "zircon_plugins/Cargo.toml",
            "-p",
            "zircon_plugin_animation_native",
            "--target-dir",
            "target/native_dynamic",
        ],
        expected_loadable_artifact=(
            "target/native_dynamic/debug/zircon_plugin_animation_native.dll"
        ),
        copied_loadable_artifact=(
            "plugins/animation/native/zircon_plugin_animation.dll"
        ),
        copied_sidecars=[],
        **overrides,
    )


def _native_build_execution_package_without(field: str) -> dict[str, object]:
    package = _native_build_execution_package()
    package.pop(field, None)
    return package


def _native_build_plan_package_with_features(
    features: list[str],
) -> dict[str, object]:
    package = _native_build_plan_package(features=features)
    command = list(package["command"])
    command[command.index("--features") + 1] = ",".join(features)
    package["command"] = command
    return package
