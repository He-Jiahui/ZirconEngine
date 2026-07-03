from __future__ import annotations

import contextlib
import hashlib
import io
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.cli import (
    apply_pipeline_stage_defaults,
    run_cook_assets,
)
from tools.zircon_export.tests.export_test_support import (
    _cook_assets_args,
    _run_cook_assets_quiet,
    _run_stage_quiet,
    _write_validate_report_with_asset_filter,
    json_dumps,
    json_loads,
)


class CookAssetsPackStageTests(unittest.TestCase):
    def test_cook_assets_stage_writes_default_manifest_and_report(self) -> None:
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
                        "asset_filter": "shipping",
                        "assets": [
                            {
                                "path": "scenes/main.zscene",
                                "source": "main.scene",
                                "dependencies": [],
                                "labels": ["shipping"],
                            }
                        ],
                    }
                ),
                encoding="utf-8",
            )

            exit_code = _run_cook_assets_quiet(
                _cook_assets_args(out=root / "out", asset_manifest=source_manifest)
            )

            staged_manifest = root / "out" / "stages" / "cook_assets" / "assets.json"
            report = root / "out" / "stages" / "cook_assets" / "report.json"
            self.assertEqual(exit_code, 0)
            self.assertTrue(staged_manifest.exists())
            self.assertTrue(report.exists())
            manifest = json_loads(staged_manifest.read_text(encoding="utf-8"))
            self.assertEqual(
                manifest["assets"][0]["source"],
                str((source_dir / "main.scene").resolve()),
            )
            stage_report = json_loads(report.read_text(encoding="utf-8"))
            self.assertFalse(stage_report["fatal"], stage_report["diagnostics"])
            self.assertEqual(stage_report["asset_count"], 1)
            self.assertEqual(
                stage_report["cooked_asset_manifest_sha256"],
                hashlib.sha256(staged_manifest.read_bytes()).hexdigest(),
            )

    def test_cook_assets_stage_orders_explicit_manifest_assets_and_dependencies(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            source_dir = root / "source"
            source_dir.mkdir()
            for filename in ("scene.toml", "material.toml", "a.png", "z.png"):
                (source_dir / filename).write_text(filename, encoding="utf-8")
            source_manifest = source_dir / "assets.json"
            source_manifest.write_text(
                json_dumps(
                    {
                        "roots": ["scenes/main.scene.toml"],
                        "assets": [
                            {
                                "path": "textures/z.png",
                                "source": "z.png",
                                "dependencies": [],
                            },
                            {
                                "path": "scenes/main.scene.toml",
                                "source": "scene.toml",
                                "dependencies": [
                                    "textures/z.png",
                                    "materials/player.zmaterial",
                                    "textures/z.png",
                                ],
                            },
                            {
                                "path": "materials/player.zmaterial",
                                "source": "material.toml",
                                "dependencies": ["textures/z.png", "textures/a.png"],
                            },
                            {
                                "path": "textures/a.png",
                                "source": "a.png",
                                "dependencies": [],
                            },
                        ],
                    }
                ),
                encoding="utf-8",
            )

            exit_code = _run_cook_assets_quiet(
                _cook_assets_args(out=root / "out", asset_manifest=source_manifest)
            )

            self.assertEqual(exit_code, 0)
            manifest = json_loads(
                (
                    root
                    / "out"
                    / "stages"
                    / "cook_assets"
                    / "assets.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(
                [asset["path"] for asset in manifest["assets"]],
                [
                    "materials/player.zmaterial",
                    "scenes/main.scene.toml",
                    "textures/a.png",
                    "textures/z.png",
                ],
            )
            assets_by_path = {
                asset["path"]: asset for asset in manifest["assets"]
            }
            self.assertEqual(
                assets_by_path["scenes/main.scene.toml"]["dependencies"],
                ["materials/player.zmaterial", "textures/z.png"],
            )
            self.assertEqual(
                assets_by_path["materials/player.zmaterial"]["dependencies"],
                ["textures/a.png", "textures/z.png"],
            )

    def test_cook_assets_stage_rejects_cooked_manifest_write_error(self) -> None:
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
            cooked_manifest = (
                root / "out" / "stages" / "cook_assets" / "assets.json"
            ).resolve()
            original_write_text = Path.write_text

            def write_text_or_fail(path: Path, *args: object, **kwargs: object) -> int:
                if path.resolve() == cooked_manifest:
                    raise OSError("simulated cooked manifest write failure")
                return original_write_text(path, *args, **kwargs)

            with mock.patch.object(Path, "write_text", write_text_or_fail):
                exit_code = _run_cook_assets_quiet(
                    _cook_assets_args(out=root / "out", asset_manifest=source_manifest)
                )

            report = json_loads(
                (
                    root
                    / "out"
                    / "stages"
                    / "cook_assets"
                    / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertFalse(cooked_manifest.exists())
            self.assertTrue(
                any(
                    "cooked asset manifest" in diagnostic
                    and "could not be written" in diagnostic
                    and "simulated cooked manifest write failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_cook_assets_rejects_asset_source_path_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            source_dir = root / "source"
            source_dir.mkdir()
            source = source_dir / "main.scene"
            source.write_text("scene", encoding="utf-8")
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
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if Path(path) == source:
                    raise OSError("simulated asset source resolve failure")
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                exit_code = _run_cook_assets_quiet(
                    _cook_assets_args(out=root / "out", asset_manifest=source_manifest)
                )

            report = json_loads(
                (
                    root
                    / "out"
                    / "stages"
                    / "cook_assets"
                    / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertFalse((root / "out" / "stages" / "cook_assets" / "assets.json").exists())
            self.assertTrue(
                any(
                    "asset source for scenes/main.zscene could not be resolved"
                    in diagnostic
                    and "simulated asset source resolve failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_pipeline_cook_assets_uses_validate_report_asset_filter(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
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
            _write_validate_report_with_asset_filter(out, "shipping")
            args = _cook_assets_args(out=out, asset_manifest=source_manifest)

            apply_pipeline_stage_defaults(args, "cook_assets")
            exit_code = _run_cook_assets_quiet(args)

            staged_manifest = json_loads(
                (out / "stages" / "cook_assets" / "assets.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 0)
            self.assertEqual(staged_manifest["asset_filter"], "shipping")

    def test_stage_cook_assets_uses_validate_report_asset_filter(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
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
            _write_validate_report_with_asset_filter(out, "shipping")
            args = _cook_assets_args(out=out, asset_manifest=source_manifest)

            exit_code = _run_stage_quiet(args)

            staged_manifest = json_loads(
                (out / "stages" / "cook_assets" / "assets.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 0)
            self.assertEqual(staged_manifest["asset_filter"], "shipping")

    def test_pipeline_cook_assets_rejects_invalid_validate_report_asset_filter(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
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
            _write_validate_report_with_asset_filter(out, [])
            args = _cook_assets_args(out=out, asset_manifest=source_manifest)

            apply_pipeline_stage_defaults(args, "cook_assets")
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
                    "Validate report field profile_summary.asset_filter must be a non-empty string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_stage_cook_assets_rejects_invalid_validate_report_asset_filter(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
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
            _write_validate_report_with_asset_filter(out, [])
            args = _cook_assets_args(out=out, asset_manifest=source_manifest)

            exit_code = _run_stage_quiet(args)

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
                    "Validate report field profile_summary.asset_filter must be a non-empty string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_stage_cook_assets_rejects_invalid_validate_metadata(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
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
            _write_validate_report_with_asset_filter(out, "shipping")
            validate_report_path = out / "stages" / "validate" / "report.json"
            validate_report = json_loads(validate_report_path.read_text(encoding="utf-8"))
            validate_report["fatal"] = []
            validate_report_path.write_text(json_dumps(validate_report), encoding="utf-8")
            args = _cook_assets_args(out=out, asset_manifest=source_manifest)

            exit_code = _run_stage_quiet(args)

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
                    "Validate report fatal must be a boolean" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_stage_cook_assets_requires_bundle_strategy(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
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
            args = _cook_assets_args(out=out, asset_manifest=source_manifest)

            exit_code = _run_stage_quiet(args)

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
                    "CookAssets stage requires library_embed or native_dynamic strategy"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_stage_cook_assets_rejects_invalid_strategy_metadata(self) -> None:
        cases = (
            ("library_embed", "profile_summary.strategies must be a list"),
            (
                [],
                "profile_summary.strategies must include at least one supported export strategy",
            ),
            (["library_embed", "ghost_path"], "unsupported export strategy ghost_path"),
        )
        for strategies, expected_diagnostic in cases:
            with self.subTest(strategies=strategies):
                with tempfile.TemporaryDirectory() as temp_dir:
                    root = Path(temp_dir)
                    out = root / "out"
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
                                    "strategies": strategies,
                                },
                            }
                        ),
                        encoding="utf-8",
                    )
                    args = _cook_assets_args(out=out, asset_manifest=source_manifest)

                    exit_code = _run_stage_quiet(args)

                    report = json_loads(
                        (out / "stages" / "cook_assets" / "report.json").read_text(
                            encoding="utf-8"
                        )
                    )
                    self.assertEqual(exit_code, 2)
                    self.assertFalse((out / "stages" / "cook_assets" / "assets.json").exists())
                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertTrue(
                        any(
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_stage_cook_assets_reports_all_unsupported_strategies(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
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
                            "strategies": [
                                "library_embed",
                                "future_export_path",
                                "console_bundle",
                            ],
                        },
                    }
                ),
                encoding="utf-8",
            )
            args = _cook_assets_args(out=out, asset_manifest=source_manifest)

            exit_code = _run_stage_quiet(args)

            report = json_loads(
                (out / "stages" / "cook_assets" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertFalse((out / "stages" / "cook_assets" / "assets.json").exists())
            self.assertIn(
                "unsupported export strategy future_export_path",
                report["diagnostics"],
            )
            self.assertIn(
                "unsupported export strategy console_bundle",
                report["diagnostics"],
            )

    def test_cook_assets_preserves_manifest_asset_filter_over_pipeline_default(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            source_manifest = root / "source" / "assets.json"
            source_manifest.parent.mkdir(parents=True)
            (source_manifest.parent / "main.scene").write_text("scene", encoding="utf-8")
            source_manifest.write_text(
                json_dumps(
                    {
                        "roots": ["scenes/main.zscene"],
                        "asset_filter": "editor",
                        "assets": [
                            {
                                "path": "scenes/main.zscene",
                                "source": "main.scene",
                                "labels": ["editor"],
                            }
                        ],
                    }
                ),
                encoding="utf-8",
            )
            _write_validate_report_with_asset_filter(out, "shipping")
            args = _cook_assets_args(out=out, asset_manifest=source_manifest)

            apply_pipeline_stage_defaults(args, "cook_assets")
            exit_code = _run_cook_assets_quiet(args)

            staged_manifest = json_loads(
                (out / "stages" / "cook_assets" / "assets.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 0)
            self.assertEqual(staged_manifest["asset_filter"], "editor")

    def test_cook_assets_rejects_empty_explicit_asset_filter(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
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
            args = _cook_assets_args(out=out, asset_manifest=source_manifest)
            args.asset_filter = ""

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

    def test_pipeline_cook_assets_preserves_empty_explicit_asset_filter(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
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
            _write_validate_report_with_asset_filter(out, "shipping")
            args = _cook_assets_args(out=out, asset_manifest=source_manifest)
            args.asset_filter = ""

            apply_pipeline_stage_defaults(args, "cook_assets")
            exit_code = _run_cook_assets_quiet(args)

            report = json_loads(
                (out / "stages" / "cook_assets" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(args.asset_filter, "")
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

    def test_cook_assets_dry_run_rejects_empty_explicit_asset_filter(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            args = _cook_assets_args(out=root / "out")
            args.asset_filter = ""
            args.dry_run = True

            stdout = io.StringIO()
            with contextlib.redirect_stdout(stdout):
                exit_code = run_cook_assets(args)

            self.assertEqual(exit_code, 2)
            self.assertIn(
                "diagnostic=asset_filter argument must be a non-empty string",
                stdout.getvalue(),
            )


if __name__ == "__main__":
    unittest.main()
