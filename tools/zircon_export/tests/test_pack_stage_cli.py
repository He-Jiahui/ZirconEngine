from __future__ import annotations

import contextlib
import io
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.pack_stage import run_pack
from tools.zircon_export.tests.export_test_support import (
    _default_cooked_manifest,
    _pack_args,
    _run_pack_quiet,
    json_dumps,
    json_loads,
)


class PackStageCliTests(unittest.TestCase):
    def test_pack_defaults_to_cook_assets_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            args = _pack_args(out=root / "out")

            stdout = io.StringIO()
            with contextlib.redirect_stdout(stdout):
                exit_code = run_pack(args)

            self.assertEqual(exit_code, 0)
            self.assertIn(
                f"asset_manifest={_default_cooked_manifest(root / 'out')}",
                stdout.getvalue(),
            )

    def test_pack_command_forwards_profile_to_packer(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            args = _pack_args(out=root / "out")

            stdout = io.StringIO()
            with contextlib.redirect_stdout(stdout):
                exit_code = run_pack(args)

            self.assertEqual(exit_code, 0)
            output = stdout.getvalue()
            self.assertIn("--profile", output)
            self.assertIn("windows-release", output)

    def test_pack_requires_bundle_strategy(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            asset_manifest = root / "source" / "assets.json"
            asset_manifest.parent.mkdir(parents=True)
            asset_manifest.write_text(
                json_dumps({"roots": [], "assets": []}),
                encoding="utf-8",
            )
            validate_report_path = out / "stages" / "validate" / "report.json"
            validate_report_path.parent.mkdir(parents=True)
            validate_report_path.write_text(
                json_dumps(
                    {
                        "stage": "Validate",
                        "profile": "windows-release",
                        "fatal": False,
                        "diagnostics": [],
                        "profile_summary": {
                            "strategies": ["source_template"],
                        },
                    }
                ),
                encoding="utf-8",
            )
            args = _pack_args(out=out)
            args.asset_manifest = str(asset_manifest)

            stdout = io.StringIO()
            with contextlib.redirect_stdout(stdout):
                exit_code = run_pack(args)

            self.assertEqual(exit_code, 2)
            output = stdout.getvalue()
            self.assertIn(
                "diagnostic=Pack stage requires library_embed or native_dynamic strategy",
                output,
            )
            self.assertIn("command=<skipped>", output)
            self.assertNotIn("--manifest", output)

    def test_pack_reports_missing_asset_manifest_before_packer(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            args = _pack_args(out=out, dry_run=False)

            with mock.patch("tools.zircon_export.pack_stage.subprocess.call", return_value=0) as packer:
                exit_code = _run_pack_quiet(args)

            report_path = out / "stages" / "pack" / "report.json"
            self.assertEqual(exit_code, 2)
            packer.assert_not_called()
            self.assertTrue(report_path.exists())
            report = json_loads(report_path.read_text(encoding="utf-8"))
            self.assertTrue(report["fatal"])
            self.assertEqual(report["stage"], "Pack")
            self.assertEqual(report["profile"], "windows-release")
            self.assertEqual(Path(report["asset_manifest"]), _default_cooked_manifest(out))
            self.assertEqual(Path(report["pack"]), out / "stages" / "pack" / "assets.zrpack")
            self.assertTrue(
                any(
                    "asset manifest" in diagnostic and "does not exist" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_pack_delta_args_are_forwarded_to_packer(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            previous_pack = root / "previous.zrpack"
            delta_pack = root / "out" / "stages" / "pack" / "assets.delta.zrpd"
            args = _pack_args(out=root / "out")
            args.previous_pack = str(previous_pack)
            args.delta_pack = str(delta_pack)

            stdout = io.StringIO()
            with contextlib.redirect_stdout(stdout):
                exit_code = run_pack(args)

            self.assertEqual(exit_code, 0)
            output = stdout.getvalue()
            self.assertIn(f"previous_pack={previous_pack}", output)
            self.assertIn(f"delta_pack={delta_pack}", output)
            self.assertIn("--previous-pack", output)
            self.assertIn(str(previous_pack), output)
            self.assertIn("--delta-pack", output)
            self.assertIn(str(delta_pack), output)

    def test_pack_rejects_unpaired_previous_pack(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            previous_pack = root / "previous.zrpack"
            args = _pack_args(out=root / "out")
            args.previous_pack = str(previous_pack)

            stdout = io.StringIO()
            with contextlib.redirect_stdout(stdout):
                exit_code = run_pack(args)

            self.assertEqual(exit_code, 2)
            output = stdout.getvalue()
            self.assertIn(
                "diagnostic=previous_pack and delta_pack must be supplied together",
                output,
            )
            self.assertIn("command=<skipped>", output)
            self.assertNotIn("--previous-pack", output)

    def test_pack_rejects_empty_delta_pack_argument(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            previous_pack = root / "previous.zrpack"
            args = _pack_args(out=root / "out")
            args.previous_pack = str(previous_pack)
            args.delta_pack = ""

            stdout = io.StringIO()
            with contextlib.redirect_stdout(stdout):
                exit_code = run_pack(args)

            self.assertEqual(exit_code, 2)
            output = stdout.getvalue()
            self.assertIn(
                "diagnostic=delta_pack argument must be a non-empty string",
                output,
            )
            self.assertIn("command=<skipped>", output)
            self.assertNotIn("--previous-pack", output)

    def test_pack_rejects_previous_pack_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            previous_pack = root / "previous.zrpack"
            delta_pack = root / "out" / "stages" / "pack" / "assets.delta.zrpd"
            args = _pack_args(out=root / "out")
            args.previous_pack = str(previous_pack)
            args.delta_pack = str(delta_pack)
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if str(path) == str(previous_pack):
                    raise OSError("simulated previous pack resolve failure")
                return original_resolve(path, *args, **kwargs)

            stdout = io.StringIO()
            with mock.patch.object(Path, "resolve", resolve_or_fail):
                with contextlib.redirect_stdout(stdout):
                    exit_code = run_pack(args)

            output = stdout.getvalue()
            self.assertEqual(exit_code, 2)
            self.assertIn("diagnostic=previous_pack", output)
            self.assertIn("could not be resolved", output)
            self.assertIn("simulated previous pack resolve failure", output)
            self.assertIn("command=<skipped>", output)
            self.assertNotIn("--previous-pack", output)

    def test_pack_rejects_delta_pack_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            previous_pack = root / "previous.zrpack"
            delta_pack = root / "out" / "stages" / "pack" / "assets.delta.zrpd"
            args = _pack_args(out=root / "out")
            args.previous_pack = str(previous_pack)
            args.delta_pack = str(delta_pack)
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if str(path) == str(delta_pack):
                    raise OSError("simulated delta pack resolve failure")
                return original_resolve(path, *args, **kwargs)

            stdout = io.StringIO()
            with mock.patch.object(Path, "resolve", resolve_or_fail):
                with contextlib.redirect_stdout(stdout):
                    exit_code = run_pack(args)

            output = stdout.getvalue()
            self.assertEqual(exit_code, 2)
            self.assertIn("diagnostic=delta_pack", output)
            self.assertIn("could not be resolved", output)
            self.assertIn("simulated delta pack resolve failure", output)
            self.assertIn("command=<skipped>", output)
            self.assertNotIn("--delta-pack", output)



if __name__ == "__main__":
    unittest.main()
