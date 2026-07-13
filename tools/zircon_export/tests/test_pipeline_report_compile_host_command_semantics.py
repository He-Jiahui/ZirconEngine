from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.export_test_support import (
    _write_compile_host_report,
    _write_pack_report,
    _write_stage_report,
    _write_validate_report_with_strategies,
)


class CompileHostCommandSemanticsTests(unittest.TestCase):
    def test_report_stage_accepts_staged_build_command(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = self._write_pipeline(Path(temp_dir))

            report = build_pipeline_report(out, "windows-release")

            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])

    def test_report_stage_rejects_removed_cargo_report_options(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = self._write_pipeline(Path(temp_dir))
            self._update_compile_command(out, ["--release", "--target-dir", "legacy"])

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any("uses removed Cargo options" in value for value in report["diagnostics"]),
                report["diagnostics"],
            )

    def test_report_stage_rejects_target_mode_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = self._write_pipeline(Path(temp_dir))
            self._replace_compile_option(out, "--targets", "runtime")

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any("--targets does not match" in value for value in report["diagnostics"]),
                report["diagnostics"],
            )

    def test_report_stage_rejects_staged_root_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = self._write_pipeline(root)
            self._replace_compile_option(out, "--out", str(root / "other"))

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any("--out does not match" in value for value in report["diagnostics"]),
                report["diagnostics"],
            )

    def test_report_stage_rejects_host_outside_staged_engine_root(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = self._write_pipeline(root)
            external = root / "external" / "zircon_hub.exe"
            external.parent.mkdir(parents=True)
            external.write_text("host", encoding="utf-8")
            self._update_compile_report(out, {"host_executable": str(external)})

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any("must remain inside current output root" in value for value in report["diagnostics"]),
                report["diagnostics"],
            )

    @staticmethod
    def _write_pipeline(root: Path) -> Path:
        out = root / "out"
        _write_validate_report_with_strategies(out, ["library_embed"])
        _write_compile_host_report(out, root / "removed-cargo-host.exe")
        _write_stage_report(out, "cook_assets", fatal=False)
        _write_pack_report(out, out / "pack-output" / "assets.zrpack")
        _write_stage_report(out, "platform_bundle", fatal=False)
        return out

    @staticmethod
    def _compile_report_path(out: Path) -> Path:
        return out / "stages" / "compile_host" / "report.json"

    @classmethod
    def _update_compile_report(cls, out: Path, updates: dict[str, object]) -> None:
        path = cls._compile_report_path(out)
        report = json.loads(path.read_text(encoding="utf-8"))
        report.update(updates)
        path.write_text(json.dumps(report, indent=2), encoding="utf-8")

    @classmethod
    def _update_compile_command(cls, out: Path, suffix: list[str]) -> None:
        path = cls._compile_report_path(out)
        report = json.loads(path.read_text(encoding="utf-8"))
        report["command"].extend(suffix)
        path.write_text(json.dumps(report, indent=2), encoding="utf-8")

    @classmethod
    def _replace_compile_option(cls, out: Path, option: str, value: str) -> None:
        path = cls._compile_report_path(out)
        report = json.loads(path.read_text(encoding="utf-8"))
        command = report["command"]
        command[command.index(option) + 1] = value
        path.write_text(json.dumps(report, indent=2), encoding="utf-8")


if __name__ == "__main__":
    unittest.main()
