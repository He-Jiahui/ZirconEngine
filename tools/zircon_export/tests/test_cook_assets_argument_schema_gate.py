from __future__ import annotations

import contextlib
import io
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.cli import apply_pipeline_stage_defaults, run_cook_assets
from tools.zircon_export.tests.export_test_support import (
    _cook_assets_args,
    _run_cook_assets_quiet,
    _write_validate_report_with_asset_filter,
    json_dumps,
    json_loads,
)


class CookAssetsArgumentSchemaGateTests(unittest.TestCase):
    def test_cook_assets_rejects_whitespace_explicit_asset_filter(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            source_manifest = write_minimal_asset_manifest(root)
            args = _cook_assets_args(out=out, asset_manifest=source_manifest)
            args.asset_filter = "   "

            exit_code = _run_cook_assets_quiet(args)

            report = json_loads(
                (out / "stages" / "cook_assets" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertFalse((out / "stages" / "cook_assets" / "assets.json").exists())
            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "asset_filter argument must be a non-empty string" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_pipeline_cook_assets_preserves_whitespace_explicit_asset_filter_gate(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            source_manifest = write_minimal_asset_manifest(root)
            _write_validate_report_with_asset_filter(out, "shipping")
            args = _cook_assets_args(out=out, asset_manifest=source_manifest)
            args.asset_filter = "   "

            apply_pipeline_stage_defaults(args, "cook_assets")
            exit_code = _run_cook_assets_quiet(args)

            report = json_loads(
                (out / "stages" / "cook_assets" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(args.asset_filter, "   ")
            self.assertEqual(exit_code, 2)
            self.assertFalse((out / "stages" / "cook_assets" / "assets.json").exists())
            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "asset_filter argument must be a non-empty string" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_cook_assets_dry_run_rejects_whitespace_explicit_asset_filter(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            args = _cook_assets_args(out=root / "out")
            args.asset_filter = "   "
            args.dry_run = True

            stdout = io.StringIO()
            with contextlib.redirect_stdout(stdout):
                exit_code = run_cook_assets(args)

            self.assertEqual(exit_code, 2)
            self.assertIn(
                "diagnostic=asset_filter argument must be a non-empty string",
                stdout.getvalue(),
            )

    def test_cook_assets_rejects_whitespace_explicit_path_arguments(self) -> None:
        cases = (
            (
                "asset_manifest",
                "source_asset_manifest",
                "CookAssets asset_manifest argument must be a non-empty path",
            ),
            (
                "project",
                "project_manifest",
                "CookAssets project_manifest argument must be a non-empty path",
            ),
        )
        for argument, report_field, expected_diagnostic in cases:
            with self.subTest(argument=argument):
                with tempfile.TemporaryDirectory() as temp_dir:
                    root = Path(temp_dir)
                    out = root / "out"
                    args = _cook_assets_args(out=out)
                    setattr(args, argument, "   ")

                    exit_code = _run_cook_assets_quiet(args)

                    report = json_loads(
                        (out / "stages" / "cook_assets" / "report.json").read_text(
                            encoding="utf-8"
                        )
                    )
                    self.assertEqual(exit_code, 2)
                    self.assertIsNone(report[report_field])
                    self.assertFalse(
                        (out / "stages" / "cook_assets" / "assets.json").exists()
                    )
                    self.assertTrue(report["fatal"])
                    self.assertIn(expected_diagnostic, report["diagnostics"])

    def test_cook_assets_dry_run_rejects_whitespace_explicit_path_arguments(
        self,
    ) -> None:
        cases = (
            (
                "asset_manifest",
                "CookAssets asset_manifest argument must be a non-empty path",
            ),
            (
                "project",
                "CookAssets project_manifest argument must be a non-empty path",
            ),
        )
        for argument, expected_diagnostic in cases:
            with self.subTest(argument=argument):
                with tempfile.TemporaryDirectory() as temp_dir:
                    root = Path(temp_dir)
                    args = _cook_assets_args(out=root / "out")
                    setattr(args, argument, "   ")
                    args.dry_run = True

                    stdout = io.StringIO()
                    with contextlib.redirect_stdout(stdout):
                        exit_code = run_cook_assets(args)

                    self.assertEqual(exit_code, 2)
                    self.assertIn(f"diagnostic={expected_diagnostic}", stdout.getvalue())


def write_minimal_asset_manifest(root: Path) -> Path:
    source_manifest = root / "source" / "assets.json"
    source_manifest.parent.mkdir(parents=True)
    (source_manifest.parent / "main.scene").write_text("scene", encoding="utf-8")
    source_manifest.write_text(
        json_dumps(
            {
                "roots": ["scenes/main.zscene"],
                "assets": [
                    {
                        "path": "scenes/main.zscene",
                        "source": "main.scene",
                        "labels": ["shipping"],
                    }
                ],
            }
        ),
        encoding="utf-8",
    )
    return source_manifest
