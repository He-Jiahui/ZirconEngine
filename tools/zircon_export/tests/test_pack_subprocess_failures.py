from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.tests.export_test_support import (
    _pack_args,
    _run_pack_quiet,
    json_dumps,
    json_loads,
)


class PackSubprocessFailureTests(unittest.TestCase):
    def test_pack_preflight_failure_report_matches_pack_schema(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            args = _pack_args(out=out, dry_run=False)

            exit_code = _run_pack_quiet(args)

            report_path = out / "stages" / "pack" / "report.json"
            self.assertEqual(exit_code, 2)
            self.assertTrue(report_path.exists())
            report = json_loads(report_path.read_text(encoding="utf-8"))
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["trim_report"]["duplicate_assets"], [])
            self.assertFalse(report["delta_apply_verified"])

    def test_pack_reports_failed_packer_without_stage_report(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            source = root / "main.scene"
            asset_manifest = root / "assets.json"
            out = root / "out"
            source.write_text("scene", encoding="utf-8")
            asset_manifest.write_text(
                json_dumps(
                    {
                        "roots": ["scenes/main.zscene"],
                        "assets": [
                            {
                                "path": "scenes/main.zscene",
                                "source": str(source),
                            }
                        ],
                    }
                ),
                encoding="utf-8",
            )
            args = _pack_args(out=out, dry_run=False)
            args.asset_manifest = str(asset_manifest)

            with mock.patch(
                "tools.zircon_export.cli.subprocess.call",
                return_value=2,
            ):
                exit_code = _run_pack_quiet(args)

            report_path = out / "stages" / "pack" / "report.json"
            self.assertEqual(exit_code, 2)
            self.assertTrue(report_path.exists())
            report = json_loads(report_path.read_text(encoding="utf-8"))
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["stage"], "Pack")
            self.assertEqual(report["profile"], "windows-release")
            self.assertEqual(report["asset_manifest"], str(asset_manifest.resolve()))
            self.assertEqual(
                report["pack"],
                str(out.resolve() / "stages" / "pack" / "assets.zrpack"),
            )
            self.assertIsNone(report["manifest"])
            self.assertEqual(report["asset_count"], 0)
            self.assertEqual(report["chunk_count"], 0)
            self.assertTrue(
                any(
                    "Pack command exited with code 2 but did not write report"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_pack_reports_successful_packer_without_stage_report(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            source = root / "main.scene"
            asset_manifest = root / "assets.json"
            out = root / "out"
            source.write_text("scene", encoding="utf-8")
            asset_manifest.write_text(
                json_dumps(
                    {
                        "roots": ["scenes/main.zscene"],
                        "assets": [
                            {
                                "path": "scenes/main.zscene",
                                "source": str(source),
                            }
                        ],
                    }
                ),
                encoding="utf-8",
            )
            args = _pack_args(out=out, dry_run=False)
            args.asset_manifest = str(asset_manifest)

            with mock.patch(
                "tools.zircon_export.cli.subprocess.call",
                return_value=0,
            ):
                exit_code = _run_pack_quiet(args)

            report_path = out / "stages" / "pack" / "report.json"
            self.assertEqual(exit_code, 2)
            self.assertTrue(report_path.exists())
            report = json_loads(report_path.read_text(encoding="utf-8"))
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertIsNone(report["manifest"])
            self.assertEqual(report["asset_count"], 0)
            self.assertEqual(report["chunk_count"], 0)
            self.assertTrue(
                any(
                    "Pack command exited with code 0 but did not write report"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )


if __name__ == "__main__":
    unittest.main()
