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
    run_pack,
)
from tools.zircon_export.tests.export_test_support import (
    _cook_assets_args,
    _default_cooked_manifest,
    _pack_args,
    _run_cook_assets_quiet,
    _run_pack_quiet,
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

    def test_cook_assets_derives_project_default_scene_without_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            project = root / "project" / "zircon-project.toml"
            scene = root / "project" / "assets" / "scenes" / "main.scene.toml"
            scene.parent.mkdir(parents=True)
            scene.write_text("scene", encoding="utf-8")
            project.write_text(
                "\n".join(
                    [
                        'name = "Export Fixture"',
                        "format_version = 1",
                        'default_scene = "res://scenes/main.scene.toml"',
                        "library_version = 3",
                    ]
                ),
                encoding="utf-8",
            )
            out = root / "out"
            args = _cook_assets_args(out=out, project=project)
            args.asset_filter = "shipping"

            exit_code = _run_cook_assets_quiet(args)

            staged_manifest = json_loads(
                (out / "stages" / "cook_assets" / "assets.json").read_text(
                    encoding="utf-8"
                )
            )
            report = json_loads(
                (out / "stages" / "cook_assets" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 0)
            self.assertEqual(staged_manifest["roots"], ["scenes/main.scene.toml"])
            self.assertEqual(staged_manifest["asset_filter"], "shipping")
            self.assertEqual(staged_manifest["assets"][0]["path"], "scenes/main.scene.toml")
            self.assertEqual(staged_manifest["assets"][0]["labels"], ["shipping"])
            self.assertEqual(staged_manifest["assets"][0]["source"], str(scene.resolve()))
            self.assertTrue(report["generated_from_project"])
            self.assertEqual(report["project_manifest"], str(project.resolve()))
            self.assertEqual(report["project_default_scene"], "res://scenes/main.scene.toml")

    def test_cook_assets_project_fallback_records_direct_res_asset_references(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            project = root / "project" / "zircon-project.toml"
            assets_root = root / "project" / "assets"
            scene = assets_root / "scenes" / "main.scene.toml"
            material = assets_root / "materials" / "player.zmaterial"
            model = assets_root / "models" / "player.glb"
            scene.parent.mkdir(parents=True)
            material.parent.mkdir(parents=True)
            model.parent.mkdir(parents=True)
            scene.write_text(
                "\n".join(
                    [
                        'material = "res://materials/player.zmaterial"',
                        'mesh = "res://models/player.glb#Mesh0/Primitive0"',
                    ]
                ),
                encoding="utf-8",
            )
            material.write_text("material", encoding="utf-8")
            model.write_text("model", encoding="utf-8")
            project.write_text(
                "\n".join(
                    [
                        'name = "Export Fixture"',
                        "format_version = 1",
                        'default_scene = "res://scenes/main.scene.toml"',
                        "library_version = 3",
                    ]
                ),
                encoding="utf-8",
            )
            out = root / "out"
            args = _cook_assets_args(out=out, project=project)

            exit_code = _run_cook_assets_quiet(args)

            staged_manifest = json_loads(
                (out / "stages" / "cook_assets" / "assets.json").read_text(
                    encoding="utf-8"
                )
            )
            report = json_loads(
                (out / "stages" / "cook_assets" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            assets_by_path = {
                asset["path"]: asset
                for asset in staged_manifest["assets"]
            }
            self.assertEqual(exit_code, 0)
            self.assertEqual(
                staged_manifest["roots"],
                ["scenes/main.scene.toml"],
            )
            self.assertEqual(
                assets_by_path["scenes/main.scene.toml"]["dependencies"],
                ["materials/player.zmaterial", "models/player.glb"],
            )
            self.assertEqual(
                assets_by_path["materials/player.zmaterial"]["source"],
                str(material.resolve()),
            )
            self.assertEqual(
                assets_by_path["models/player.glb"]["source"],
                str(model.resolve()),
            )
            self.assertEqual(report["asset_count"], 3)

    def test_cook_assets_project_fallback_rejects_missing_direct_reference(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            project = root / "project" / "zircon-project.toml"
            assets_root = root / "project" / "assets"
            scene = assets_root / "scenes" / "main.scene.toml"
            scene.parent.mkdir(parents=True)
            scene.write_text(
                'material = "res://materials/missing.zmaterial"',
                encoding="utf-8",
            )
            project.write_text(
                "\n".join(
                    [
                        'name = "Export Fixture"',
                        "format_version = 1",
                        'default_scene = "res://scenes/main.scene.toml"',
                        "library_version = 3",
                    ]
                ),
                encoding="utf-8",
            )
            out = root / "out"

            exit_code = _run_cook_assets_quiet(_cook_assets_args(out=out, project=project))

            report = json_loads(
                (out / "stages" / "cook_assets" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertFalse((out / "stages" / "cook_assets" / "assets.json").exists())
            self.assertTrue(
                any(
                    "asset source for materials/missing.zmaterial does not exist"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_cook_assets_project_fallback_rejects_unsafe_direct_reference(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            project = root / "project" / "zircon-project.toml"
            assets_root = root / "project" / "assets"
            scene = assets_root / "scenes" / "main.scene.toml"
            scene.parent.mkdir(parents=True)
            scene.write_text(
                'material = "res://../outside.zmaterial"',
                encoding="utf-8",
            )
            project.write_text(
                "\n".join(
                    [
                        'name = "Export Fixture"',
                        "format_version = 1",
                        'default_scene = "res://scenes/main.scene.toml"',
                        "library_version = 3",
                    ]
                ),
                encoding="utf-8",
            )
            out = root / "out"

            exit_code = _run_cook_assets_quiet(_cook_assets_args(out=out, project=project))

            report = json_loads(
                (out / "stages" / "cook_assets" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertFalse((out / "stages" / "cook_assets" / "assets.json").exists())
            self.assertTrue(
                any(
                    "project asset reference res://../outside.zmaterial"
                    in diagnostic
                    and "does not resolve to a safe asset path" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_cook_assets_rejects_project_default_scene_source_resolve_error(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            project = root / "project" / "zircon-project.toml"
            scene = root / "project" / "assets" / "scenes" / "main.scene.toml"
            scene.parent.mkdir(parents=True)
            scene.write_text("scene", encoding="utf-8")
            project.write_text(
                "\n".join(
                    [
                        'name = "Export Fixture"',
                        "format_version = 1",
                        'default_scene = "res://scenes/main.scene.toml"',
                        "library_version = 3",
                    ]
                ),
                encoding="utf-8",
            )
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if Path(path) == scene:
                    raise OSError("simulated project asset source resolve failure")
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                exit_code = _run_cook_assets_quiet(
                    _cook_assets_args(out=root / "out", project=project)
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
            self.assertTrue(report["generated_from_project"])
            self.assertEqual(report["project_default_scene"], "res://scenes/main.scene.toml")
            self.assertFalse((root / "out" / "stages" / "cook_assets" / "assets.json").exists())
            self.assertTrue(
                any(
                    "asset source for scenes/main.scene.toml could not be resolved"
                    in diagnostic
                    and "simulated project asset source resolve failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_cook_assets_rejects_asset_manifest_directory(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            source_manifest = root / "source" / "assets.json"
            source_manifest.mkdir(parents=True)
            out = root / "out"

            exit_code = _run_cook_assets_quiet(
                _cook_assets_args(out=out, asset_manifest=source_manifest)
            )

            report = json_loads(
                (out / "stages" / "cook_assets" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertFalse((out / "stages" / "cook_assets" / "assets.json").exists())
            self.assertTrue(
                any(
                    f"asset manifest {source_manifest.resolve()} is not a file"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_cook_assets_rejects_project_manifest_directory(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            project = root / "project" / "zircon-project.toml"
            project.mkdir(parents=True)
            out = root / "out"

            exit_code = _run_cook_assets_quiet(
                _cook_assets_args(out=out, project=project)
            )

            report = json_loads(
                (out / "stages" / "cook_assets" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertFalse((out / "stages" / "cook_assets" / "assets.json").exists())
            self.assertTrue(
                any(
                    f"project manifest {project.resolve()} is not a file"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_cook_assets_reports_missing_project_default_scene_source(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            project = root / "project" / "zircon-project.toml"
            project.parent.mkdir(parents=True)
            project.write_text(
                "\n".join(
                    [
                        'name = "Missing Scene Fixture"',
                        "format_version = 1",
                        'default_scene = "res://scenes/missing.scene.toml"',
                        "library_version = 3",
                    ]
                ),
                encoding="utf-8",
            )
            out = root / "out"

            exit_code = _run_cook_assets_quiet(
                _cook_assets_args(out=out, project=project)
            )

            report = json_loads(
                (out / "stages" / "cook_assets" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertFalse((out / "stages" / "cook_assets" / "assets.json").exists())
            self.assertTrue(
                any("does not exist" in diagnostic for diagnostic in report["diagnostics"]),
                report["diagnostics"],
            )

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

            with mock.patch("tools.zircon_export.cli.subprocess.call", return_value=0) as packer:
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
