from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.tests.export_test_support import (
    _cook_assets_args,
    _run_cook_assets_quiet,
    json_loads,
)


class CookAssetsPathResolveErrorTests(unittest.TestCase):
    def test_cook_assets_reports_asset_manifest_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            asset_manifest = root / "source" / "assets.json"
            args = _cook_assets_args(
                out=root / "out",
                asset_manifest=asset_manifest,
            )
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if str(path) == str(asset_manifest):
                    raise OSError("simulated CookAssets asset manifest failure")
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                exit_code = _run_cook_assets_quiet(args)

            report = json_loads(
                (
                    root / "out" / "stages" / "cook_assets" / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertIsNone(report["source_asset_manifest"])
            self.assertEqual(report["asset_count"], 0)
            self.assertFalse(
                (root / "out" / "stages" / "cook_assets" / "assets.json").exists()
            )
            self.assertTrue(
                any(
                    "CookAssets asset_manifest" in diagnostic
                    and "could not be resolved" in diagnostic
                    and "simulated CookAssets asset manifest failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_cook_assets_reports_project_manifest_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            project = root / "project" / "zircon-project.toml"
            args = _cook_assets_args(out=root / "out", project=project)
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if str(path) == str(project):
                    raise OSError("simulated CookAssets project manifest failure")
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                exit_code = _run_cook_assets_quiet(args)

            report = json_loads(
                (
                    root / "out" / "stages" / "cook_assets" / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertIsNone(report["project_manifest"])
            self.assertFalse(report["generated_from_project"])
            self.assertFalse(
                (root / "out" / "stages" / "cook_assets" / "assets.json").exists()
            )
            self.assertTrue(
                any(
                    "CookAssets project_manifest" in diagnostic
                    and "could not be resolved" in diagnostic
                    and "simulated CookAssets project manifest failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )


if __name__ == "__main__":
    unittest.main()
