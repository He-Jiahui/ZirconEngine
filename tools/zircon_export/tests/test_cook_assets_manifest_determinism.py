from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.tests.export_test_support import (
    _cook_assets_args,
    _run_cook_assets_quiet,
    json_dumps,
    json_loads,
)


class CookAssetsManifestDeterminismTests(unittest.TestCase):
    def test_cook_assets_stage_rejects_explicit_manifest_source_directory(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            source_dir = root / "source"
            asset_source_dir = source_dir / "main.scene"
            asset_source_dir.mkdir(parents=True)
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

            exit_code = _run_cook_assets_quiet(
                _cook_assets_args(out=root / "out", asset_manifest=source_manifest)
            )

            report = json_loads(
                (root / "out" / "stages" / "cook_assets" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertFalse(
                (root / "out" / "stages" / "cook_assets" / "assets.json").exists()
            )
            self.assertTrue(
                any(
                    "asset source for scenes/main.zscene is not a file"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_cook_assets_stage_orders_explicit_manifest_roots(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            source_dir = root / "source"
            source_dir.mkdir()
            for filename in ("main.scene", "intro.scene"):
                (source_dir / filename).write_text(filename, encoding="utf-8")
            source_manifest = source_dir / "assets.json"
            source_manifest.write_text(
                json_dumps(
                    {
                        "roots": [
                            "scenes/main.zscene",
                            "scenes/intro.zscene",
                            "scenes/main.zscene",
                        ],
                        "assets": [
                            {
                                "path": "scenes/main.zscene",
                                "source": "main.scene",
                            },
                            {
                                "path": "scenes/intro.zscene",
                                "source": "intro.scene",
                            },
                        ],
                    }
                ),
                encoding="utf-8",
            )

            exit_code = _run_cook_assets_quiet(
                _cook_assets_args(out=root / "out", asset_manifest=source_manifest)
            )

            manifest = json_loads(
                (root / "out" / "stages" / "cook_assets" / "assets.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 0)
            self.assertEqual(
                manifest["roots"],
                ["scenes/intro.zscene", "scenes/main.zscene"],
            )

    def test_cook_assets_stage_orders_explicit_manifest_labels(self) -> None:
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
                                "labels": ["shipping", "editor", "shipping"],
                            }
                        ],
                    }
                ),
                encoding="utf-8",
            )

            exit_code = _run_cook_assets_quiet(
                _cook_assets_args(out=root / "out", asset_manifest=source_manifest)
            )

            manifest = json_loads(
                (root / "out" / "stages" / "cook_assets" / "assets.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 0)
            self.assertEqual(manifest["assets"][0]["labels"], ["editor", "shipping"])


if __name__ == "__main__":
    unittest.main()
