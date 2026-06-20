from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.export_test_support import (
    _compile_host_plan,
    _write_compile_host_report,
    _write_pack_report,
    _write_stage_report,
    _write_validate_report_with_strategies,
)


class CompileHostCommandSemanticsTests(unittest.TestCase):
    def test_report_stage_rejects_compile_host_command_validate_mismatch(
        self,
    ) -> None:
        cases = (
            (
                self._compile_host_command_with("-p", "zircon_editor"),
                "compile_host report command -p/--package does not match "
                "validate report plan_summary.library_embed_compile_host.package",
            ),
            (
                self._compile_host_command_with("--bin", "zircon_editor"),
                "compile_host report command --bin does not match "
                "validate report plan_summary.library_embed_compile_host.binary",
            ),
            (
                self._compile_host_command_with("--features", "target-client editor"),
                "compile_host report command --features does not match "
                "validate report plan_summary.library_embed_compile_host.app_features",
            ),
        )
        for command, expected_diagnostic in cases:
            with self.subTest(expected_diagnostic=expected_diagnostic):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_validate_report_with_strategies(out, ["library_embed"])
                    _write_compile_host_report(
                        out,
                        out / "compile" / "zircon_runtime.exe",
                    )
                    _write_stage_report(out, "cook_assets", fatal=False)
                    _write_pack_report(out, out / "pack-output" / "assets.zrpack")
                    _write_stage_report(out, "platform_bundle", fatal=False)
                    compile_report_path = (
                        out / "stages" / "compile_host" / "report.json"
                    )
                    compile_report = json.loads(
                        compile_report_path.read_text(encoding="utf-8")
                    )
                    compile_report["command"] = command
                    compile_report_path.write_text(
                        json.dumps(compile_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertTrue(
                        any(
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_compile_host_command_release_flag_mismatch(
        self,
    ) -> None:
        cases = (
            (
                {"command": [*_compile_host_plan()["command"], "--release"]},
                None,
                "compile_host report command must not include --release for debug profile",
            ),
            (
                {},
                {
                    "release": True,
                    "cargo_profile": "release",
                    "command": [*_compile_host_plan()["command"], "--release"],
                },
                "compile_host report command must include --release for release profile",
            ),
        )
        for compile_overrides, validate_overrides, expected_diagnostic in cases:
            with self.subTest(expected_diagnostic=expected_diagnostic):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_validate_report_with_strategies(out, ["library_embed"])
                    if validate_overrides:
                        self._update_validate_compile_host_plan(
                            out,
                            validate_overrides,
                        )
                    _write_compile_host_report(
                        out,
                        out / "compile" / "zircon_runtime.exe",
                    )
                    _write_stage_report(out, "cook_assets", fatal=False)
                    _write_pack_report(out, out / "pack-output" / "assets.zrpack")
                    _write_stage_report(out, "platform_bundle", fatal=False)
                    self._update_compile_host_report(out, compile_overrides)

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertTrue(
                        any(
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_compile_host_command_target_dir_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            target_dir = out / "stages" / "compile_host" / "target"
            _write_validate_report_with_strategies(out, ["library_embed"])
            _write_compile_host_report(
                out,
                target_dir / "debug" / "zircon_runtime.exe",
            )
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, out / "pack-output" / "assets.zrpack")
            _write_stage_report(out, "platform_bundle", fatal=False)
            self._update_compile_host_report(
                out,
                {
                    "command": self._compile_host_command_with(
                        "--target-dir",
                        "stages/compile_host/stale-target",
                    ),
                },
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "compile_host report command --target-dir does not match "
                    "validate report plan_summary.library_embed_compile_host.target_dir"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_accepts_compile_host_command_resolved_target_dir(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            target_dir = out / "stages" / "compile_host" / "target"
            _write_validate_report_with_strategies(out, ["library_embed"])
            _write_compile_host_report(
                out,
                target_dir / "debug" / "zircon_runtime.exe",
            )
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, out / "pack-output" / "assets.zrpack")
            _write_stage_report(out, "platform_bundle", fatal=False)
            self._update_compile_host_report(
                out,
                {
                    "command": self._compile_host_command_with(
                        "--target-dir",
                        str(target_dir),
                    ),
                },
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])

    def test_report_stage_rejects_compile_host_host_executable_target_dir_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            target_dir = out / "stages" / "compile_host" / "target"
            _write_validate_report_with_strategies(out, ["library_embed"])
            _write_compile_host_report(
                out,
                out / "compile" / "zircon_runtime.exe",
            )
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, out / "pack-output" / "assets.zrpack")
            _write_stage_report(out, "platform_bundle", fatal=False)
            self._update_compile_host_report(
                out,
                {
                    "command": self._compile_host_command_with(
                        "--target-dir",
                        str(target_dir),
                    ),
                },
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "compile_host report host_executable "
                    in diagnostic
                    and "does not match command --target-dir profile directory"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_compile_host_host_executable_binary_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            target_dir = out / "stages" / "compile_host" / "target"
            _write_validate_report_with_strategies(out, ["library_embed"])
            _write_compile_host_report(
                out,
                target_dir / "debug" / "zircon_editor.exe",
            )
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, out / "pack-output" / "assets.zrpack")
            _write_stage_report(out, "platform_bundle", fatal=False)
            self._update_compile_host_report(
                out,
                {
                    "command": self._compile_host_command_with(
                        "--target-dir",
                        str(target_dir),
                    ),
                },
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "compile_host report host_executable "
                    in diagnostic
                    and "does not match validate report "
                    "plan_summary.library_embed_compile_host.binary"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_compile_host_missing_host_executable_file(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            target_dir = out / "stages" / "compile_host" / "target"
            missing_host = target_dir / "debug" / "zircon_runtime.exe"
            _write_validate_report_with_strategies(out, ["library_embed"])
            _write_compile_host_report(out, missing_host, host_value=str(missing_host))
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, out / "pack-output" / "assets.zrpack")
            _write_stage_report(out, "platform_bundle", fatal=False)
            self._update_compile_host_report(
                out,
                {
                    "command": self._compile_host_command_with(
                        "--target-dir",
                        str(target_dir),
                    ),
                },
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "compile_host report host_executable "
                    f"{missing_host} does not exist"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_compile_host_host_executable_directory(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            target_dir = out / "stages" / "compile_host" / "target"
            host_dir = target_dir / "debug" / "zircon_runtime.exe"
            host_dir.mkdir(parents=True)
            _write_validate_report_with_strategies(out, ["library_embed"])
            _write_compile_host_report(out, host_dir, host_value=str(host_dir))
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, out / "pack-output" / "assets.zrpack")
            _write_stage_report(out, "platform_bundle", fatal=False)
            self._update_compile_host_report(
                out,
                {
                    "command": self._compile_host_command_with(
                        "--target-dir",
                        str(target_dir),
                    ),
                },
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "compile_host report host_executable "
                    f"{host_dir} is not a file"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_compile_host_host_executable_outside_output_root(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            external_host = root / "external" / "zircon_runtime.exe"
            _write_validate_report_with_strategies(out, ["library_embed"])
            _write_compile_host_report(out, external_host)
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, out / "pack-output" / "assets.zrpack")
            _write_stage_report(out, "platform_bundle", fatal=False)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "compile_host report host_executable "
                    f"{external_host} is outside current output root {out}"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def _compile_host_command_with(self, option: str, value: str) -> list[str]:
        command = list(_compile_host_plan()["command"])
        index = command.index(option)
        command[index + 1] = value
        return command

    def _update_compile_host_report(
        self,
        out: Path,
        overrides: dict[str, object],
    ) -> None:
        compile_report_path = out / "stages" / "compile_host" / "report.json"
        compile_report = json.loads(compile_report_path.read_text(encoding="utf-8"))
        compile_report.update(overrides)
        compile_report_path.write_text(
            json.dumps(compile_report, indent=2),
            encoding="utf-8",
        )

    def _update_validate_compile_host_plan(
        self,
        out: Path,
        overrides: dict[str, object],
    ) -> None:
        validate_report_path = out / "stages" / "validate" / "report.json"
        validate_report = json.loads(validate_report_path.read_text(encoding="utf-8"))
        plan = validate_report["plan_summary"]["library_embed_compile_host"]
        plan.update(overrides)
        validate_report_path.write_text(
            json.dumps(validate_report, indent=2),
            encoding="utf-8",
        )


if __name__ == "__main__":
    unittest.main()
