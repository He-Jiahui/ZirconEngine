from __future__ import annotations

import contextlib
import io
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.cli import run_cook_assets, run_report
from tools.zircon_export.tests.export_test_support import (
    _cook_assets_args,
    _report_args,
    _write_stage_report,
    json_dumps,
    json_loads,
)


def _printed_json_object(output: str) -> dict[str, object]:
    json_start = output.find("{\n")
    if json_start == -1:
        raise AssertionError(f"no JSON object found in output:\n{output}")
    parsed = json_loads(output[json_start:])
    if not isinstance(parsed, dict):
        raise AssertionError(f"expected JSON object, got {type(parsed).__name__}")
    return parsed


class ReportWriteErrorTests(unittest.TestCase):
    def test_cook_assets_stage_reports_report_write_error_to_stdout(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            source_dir = root / "source"
            source_dir.mkdir()
            (source_dir / "main.scene").write_text("scene", encoding="utf-8")
            source_manifest = source_dir / "assets.json"
            source_manifest.write_text(
                json_dumps(
                    {
                        "roots": ["scenes/main.zscene"],
                        "assets": [
                            {
                                "path": "scenes/main.zscene",
                                "source": "main.scene",
                            }
                        ],
                    }
                ),
                encoding="utf-8",
            )
            out = root / "out"
            report_path = (out / "stages" / "cook_assets" / "report.json").resolve()
            original_write_text = Path.write_text

            def write_text_or_fail(path: Path, *args: object, **kwargs: object) -> int:
                if path.resolve() == report_path:
                    raise OSError("simulated stage report write failure")
                return original_write_text(path, *args, **kwargs)

            stdout = io.StringIO()
            with mock.patch.object(Path, "write_text", write_text_or_fail):
                with contextlib.redirect_stdout(stdout):
                    exit_code = run_cook_assets(
                        _cook_assets_args(out=out, asset_manifest=source_manifest)
                    )

            printed_report = _printed_json_object(stdout.getvalue())
            self.assertEqual(exit_code, 2)
            self.assertFalse(report_path.exists())
            self.assertTrue(printed_report["fatal"], printed_report["diagnostics"])
            self.assertTrue(
                any(
                    "CookAssets report" in diagnostic
                    and "could not be written" in diagnostic
                    and "simulated stage report write failure" in diagnostic
                    for diagnostic in printed_report["diagnostics"]
                ),
                printed_report["diagnostics"],
            )

    def test_report_stage_records_pipeline_report_write_error_in_stage_report(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            for stage in (
                "validate",
                "compile_host",
                "cook_assets",
                "pack",
                "platform_bundle",
            ):
                _write_stage_report(out, stage, fatal=False)
            stage_report_path = out / "stages" / "report" / "report.json"
            pipeline_report_path = (out / "report.json").resolve()
            original_write_text = Path.write_text

            def write_text_or_fail(path: Path, *args: object, **kwargs: object) -> int:
                if path.resolve() == pipeline_report_path:
                    raise OSError("simulated pipeline report write failure")
                return original_write_text(path, *args, **kwargs)

            stdout = io.StringIO()
            with mock.patch.object(Path, "write_text", write_text_or_fail):
                with contextlib.redirect_stdout(stdout):
                    exit_code = run_report(_report_args(out=out))

            stage_report = json_loads(stage_report_path.read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 2)
            self.assertFalse(pipeline_report_path.exists())
            self.assertTrue(stage_report["fatal"], stage_report["diagnostics"])
            self.assertTrue(
                any(
                    "pipeline report" in diagnostic
                    and "could not be written" in diagnostic
                    and "simulated pipeline report write failure" in diagnostic
                    for diagnostic in stage_report["diagnostics"]
                ),
                stage_report["diagnostics"],
            )
            self.assertEqual(stage_report, _printed_json_object(stdout.getvalue()))

    def test_report_stage_removes_stale_stage_report_when_rewrite_fails(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            for stage in (
                "validate",
                "compile_host",
                "cook_assets",
                "pack",
                "platform_bundle",
            ):
                _write_stage_report(out, stage, fatal=False)
            stage_report_path = (out / "stages" / "report" / "report.json").resolve()
            pipeline_report_path = (out / "report.json").resolve()
            original_write_text = Path.write_text
            stage_report_write_count = 0

            def write_text_or_fail(path: Path, *args: object, **kwargs: object) -> int:
                nonlocal stage_report_write_count
                if path.resolve() == pipeline_report_path:
                    raise OSError("simulated pipeline report write failure")
                if path.resolve() == stage_report_path:
                    stage_report_write_count += 1
                    if stage_report_write_count == 2:
                        raise OSError("simulated stage report rewrite failure")
                return original_write_text(path, *args, **kwargs)

            stdout = io.StringIO()
            with mock.patch.object(Path, "write_text", write_text_or_fail):
                with contextlib.redirect_stdout(stdout):
                    exit_code = run_report(_report_args(out=out))

            printed_report = _printed_json_object(stdout.getvalue())
            self.assertEqual(exit_code, 2)
            self.assertFalse(pipeline_report_path.exists())
            self.assertFalse(stage_report_path.exists())
            self.assertTrue(printed_report["fatal"], printed_report["diagnostics"])
            self.assertTrue(
                any(
                    "pipeline report" in diagnostic
                    and "could not be written" in diagnostic
                    and "simulated pipeline report write failure" in diagnostic
                    for diagnostic in printed_report["diagnostics"]
                ),
                printed_report["diagnostics"],
            )
            self.assertTrue(
                any(
                    "Report stage report update" in diagnostic
                    and "could not be written" in diagnostic
                    and "simulated stage report rewrite failure" in diagnostic
                    for diagnostic in printed_report["diagnostics"]
                ),
                printed_report["diagnostics"],
            )

    def test_report_stage_writes_pipeline_report_when_stage_dir_create_fails(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            for stage in (
                "validate",
                "compile_host",
                "cook_assets",
                "pack",
                "platform_bundle",
            ):
                _write_stage_report(out, stage, fatal=False)
            report_stage_dir = (out / "stages" / "report").resolve()
            original_mkdir = Path.mkdir

            def mkdir_or_fail(path: Path, *args: object, **kwargs: object) -> None:
                if path.resolve() == report_stage_dir:
                    raise OSError("simulated report stage dir create failure")
                original_mkdir(path, *args, **kwargs)

            stdout = io.StringIO()
            with mock.patch.object(Path, "mkdir", mkdir_or_fail):
                with contextlib.redirect_stdout(stdout):
                    exit_code = run_report(_report_args(out=out))

            pipeline_report = json_loads((out / "report.json").read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 2)
            self.assertFalse((report_stage_dir / "report.json").exists())
            self.assertTrue(pipeline_report["fatal"], pipeline_report["diagnostics"])
            self.assertTrue(
                any(
                    "Report stage directory" in diagnostic
                    and "could not be created" in diagnostic
                    and "simulated report stage dir create failure" in diagnostic
                    for diagnostic in pipeline_report["diagnostics"]
                ),
                pipeline_report["diagnostics"],
            )
            self.assertEqual(pipeline_report, _printed_json_object(stdout.getvalue()))


if __name__ == "__main__":
    unittest.main()
