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


class CookAssetsManifestSchemaGateTests(unittest.TestCase):
    def test_cook_assets_rejects_unknown_manifest_fields(self) -> None:
        cases = (
            (
                {
                    "roots": [],
                    "assets": [],
                    "unexpected": True,
                },
                "asset manifest unknown field unexpected",
            ),
            (
                {
                    "roots": [],
                    "assets": [
                        {
                            "path": "scenes/main.zscene",
                            "unexpected": True,
                        }
                    ],
                },
                "asset manifest entry 0 unknown field unexpected",
            ),
        )
        for manifest, expected_diagnostic in cases:
            with self.subTest(expected_diagnostic=expected_diagnostic):
                with tempfile.TemporaryDirectory() as temp_dir:
                    root = Path(temp_dir)
                    out = root / "out"
                    source_manifest = write_manifest(root, manifest)

                    exit_code = _run_cook_assets_quiet(
                        _cook_assets_args(out=out, asset_manifest=source_manifest)
                    )

                    report = json_loads(
                        (out / "stages" / "cook_assets" / "report.json").read_text(
                            encoding="utf-8"
                        )
                    )
                    self.assertEqual(exit_code, 2)
                    self.assertFalse(
                        (out / "stages" / "cook_assets" / "assets.json").exists()
                    )
                    self.assertTrue(report["fatal"])
                    self.assertIn(expected_diagnostic, report["diagnostics"])

    def test_cook_assets_rejects_manifest_missing_references(self) -> None:
        cases = (
            (
                {
                    "roots": ["scenes/missing.zscene"],
                    "assets": [],
                },
                "asset manifest root scenes/missing.zscene is not declared in assets",
            ),
            (
                {
                    "roots": ["scenes/main.zscene"],
                    "assets": [
                        {
                            "path": "scenes/main.zscene",
                            "dependencies": ["textures/missing.png"],
                        }
                    ],
                },
                (
                    "asset manifest entry 0 dependency textures/missing.png "
                    "is not declared in assets"
                ),
            ),
        )
        for manifest, expected_diagnostic in cases:
            with self.subTest(expected_diagnostic=expected_diagnostic):
                with tempfile.TemporaryDirectory() as temp_dir:
                    root = Path(temp_dir)
                    out = root / "out"
                    source_manifest = write_manifest(root, manifest)

                    exit_code = _run_cook_assets_quiet(
                        _cook_assets_args(out=out, asset_manifest=source_manifest)
                    )

                    report = json_loads(
                        (out / "stages" / "cook_assets" / "report.json").read_text(
                            encoding="utf-8"
                        )
                    )
                    self.assertEqual(exit_code, 2)
                    self.assertFalse(
                        (out / "stages" / "cook_assets" / "assets.json").exists()
                    )
                    self.assertTrue(report["fatal"])
                    self.assertIn(expected_diagnostic, report["diagnostics"])

    def test_cook_assets_rejects_blank_manifest_asset_filter(self) -> None:
        cases = ("", "   ")
        for asset_filter in cases:
            with self.subTest(asset_filter=repr(asset_filter)):
                with tempfile.TemporaryDirectory() as temp_dir:
                    root = Path(temp_dir)
                    out = root / "out"
                    source_manifest = write_manifest_with_asset_filter(
                        root,
                        asset_filter,
                    )

                    exit_code = _run_cook_assets_quiet(
                        _cook_assets_args(out=out, asset_manifest=source_manifest)
                    )

                    report = json_loads(
                        (out / "stages" / "cook_assets" / "report.json").read_text(
                            encoding="utf-8"
                        )
                    )
                    self.assertEqual(exit_code, 2)
                    self.assertFalse(
                        (out / "stages" / "cook_assets" / "assets.json").exists()
                    )
                    self.assertTrue(report["fatal"])
                    self.assertIn(
                        "asset manifest field asset_filter must be a non-empty string when present",
                        report["diagnostics"],
                    )

    def test_cook_assets_rejects_blank_manifest_path_array_entries(self) -> None:
        cases = (
            (
                {"roots": ["   "], "assets": []},
                "asset manifest field roots entry 0 must be a non-empty string",
            ),
            (
                {"roots": [], "assets": [{"path": "   "}]},
                "asset manifest entry 0 needs a non-empty path",
            ),
            (
                {
                    "roots": [],
                    "assets": [
                        {
                            "path": "scenes/main.zscene",
                            "dependencies": ["   "],
                        }
                    ],
                },
                "asset manifest entry 0 field dependencies entry 0 must be a non-empty string",
            ),
            (
                {
                    "roots": [],
                    "assets": [
                        {
                            "path": "scenes/main.zscene",
                            "labels": ["   "],
                        }
                    ],
                },
                "asset manifest entry 0 field labels entry 0 must be a non-empty string",
            ),
        )
        for manifest, expected_diagnostic in cases:
            with self.subTest(expected_diagnostic=expected_diagnostic):
                with tempfile.TemporaryDirectory() as temp_dir:
                    root = Path(temp_dir)
                    out = root / "out"
                    source_manifest = write_manifest(root, manifest)

                    exit_code = _run_cook_assets_quiet(
                        _cook_assets_args(out=out, asset_manifest=source_manifest)
                    )

                    report = json_loads(
                        (out / "stages" / "cook_assets" / "report.json").read_text(
                            encoding="utf-8"
                        )
                    )
                    self.assertEqual(exit_code, 2)
                    self.assertFalse(
                        (out / "stages" / "cook_assets" / "assets.json").exists()
                    )
                    self.assertTrue(report["fatal"])
                    self.assertIn(expected_diagnostic, report["diagnostics"])

    def test_cook_assets_rejects_non_string_manifest_string_array_entry_before_array_shape(
        self,
    ) -> None:
        cases = (
            (
                {"roots": ["scenes/main.zscene", 42], "assets": []},
                "asset manifest field roots entry 1 must be a string",
                "asset manifest field roots must be a string array",
            ),
            (
                {
                    "roots": [],
                    "assets": [
                        {
                            "path": "scenes/main.zscene",
                            "dependencies": ["textures/hero.png", 42],
                        },
                        {"path": "textures/hero.png"},
                    ],
                },
                "asset manifest entry 0 field dependencies entry 1 must be a string",
                "asset manifest entry 0 field dependencies must be a string array",
            ),
            (
                {
                    "roots": [],
                    "assets": [
                        {
                            "path": "textures/hero.png",
                            "labels": ["shipping", False],
                        }
                    ],
                },
                "asset manifest entry 0 field labels entry 1 must be a string",
                "asset manifest entry 0 field labels must be a string array",
            ),
        )
        for manifest, expected_diagnostic, unexpected_diagnostic in cases:
            with self.subTest(expected_diagnostic=expected_diagnostic):
                with tempfile.TemporaryDirectory() as temp_dir:
                    root = Path(temp_dir)
                    out = root / "out"
                    source_manifest = write_manifest(root, manifest)

                    exit_code = _run_cook_assets_quiet(
                        _cook_assets_args(out=out, asset_manifest=source_manifest)
                    )

                    report = json_loads(
                        (out / "stages" / "cook_assets" / "report.json").read_text(
                            encoding="utf-8"
                        )
                    )
                    self.assertEqual(exit_code, 2)
                    self.assertFalse(
                        (out / "stages" / "cook_assets" / "assets.json").exists()
                    )
                    self.assertTrue(report["fatal"])
                    self.assertIn(expected_diagnostic, report["diagnostics"])
                    self.assertNotIn(unexpected_diagnostic, report["diagnostics"])

    def test_cook_assets_rejects_blank_manifest_source_when_present(self) -> None:
        cases = ("", "   ")
        for source in cases:
            with self.subTest(source=repr(source)):
                with tempfile.TemporaryDirectory() as temp_dir:
                    root = Path(temp_dir)
                    out = root / "out"
                    source_manifest = write_manifest(
                        root,
                        {
                            "roots": ["scenes/main.zscene"],
                            "assets": [
                                {
                                    "path": "scenes/main.zscene",
                                    "source": source,
                                }
                            ],
                        },
                    )

                    exit_code = _run_cook_assets_quiet(
                        _cook_assets_args(out=out, asset_manifest=source_manifest)
                    )

                    report = json_loads(
                        (out / "stages" / "cook_assets" / "report.json").read_text(
                            encoding="utf-8"
                        )
                    )
                    self.assertEqual(exit_code, 2)
                    self.assertFalse(
                        (out / "stages" / "cook_assets" / "assets.json").exists()
                    )
                    self.assertTrue(report["fatal"])
                    self.assertIn(
                        "asset manifest entry 0 field source must be a non-empty string when present",
                        report["diagnostics"],
                    )

    def test_cook_assets_rejects_unsafe_manifest_asset_paths(self) -> None:
        cases = (
            (
                {"roots": ["../escape.zscene"], "assets": []},
                "asset manifest field roots entry 0 must be a safe relative asset path",
            ),
            (
                {"roots": [], "assets": [{"path": "../escape.zscene"}]},
                "asset manifest entry 0 path must be a safe relative asset path",
            ),
            (
                {
                    "roots": [],
                    "assets": [
                        {
                            "path": "scenes/main.zscene",
                            "dependencies": ["../escape.texture"],
                        }
                    ],
                },
                "asset manifest entry 0 field dependencies entry 0 must be a safe relative asset path",
            ),
        )
        for manifest, expected_diagnostic in cases:
            with self.subTest(expected_diagnostic=expected_diagnostic):
                with tempfile.TemporaryDirectory() as temp_dir:
                    root = Path(temp_dir)
                    out = root / "out"
                    source_manifest = write_manifest(root, manifest)

                    exit_code = _run_cook_assets_quiet(
                        _cook_assets_args(out=out, asset_manifest=source_manifest)
                    )

                    report = json_loads(
                        (out / "stages" / "cook_assets" / "report.json").read_text(
                            encoding="utf-8"
                        )
                    )
                    self.assertEqual(exit_code, 2)
                    self.assertFalse(
                        (out / "stages" / "cook_assets" / "assets.json").exists()
                    )
                    self.assertTrue(report["fatal"])
                    self.assertIn(expected_diagnostic, report["diagnostics"])

    def test_cook_assets_normalizes_explicit_manifest_package_paths(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            source_manifest = write_manifest(
                root,
                {
                    "roots": [" scenes\\main.zscene "],
                    "assets": [
                        {
                            "path": "scenes\\main.zscene",
                            "source": "main.scene",
                            "dependencies": [" textures\\hero.png "],
                        },
                        {
                            "path": "textures\\hero.png",
                            "labels": ["shipping"],
                        },
                    ],
                },
            )

            exit_code = _run_cook_assets_quiet(
                _cook_assets_args(out=out, asset_manifest=source_manifest)
            )

            manifest = json_loads(
                (out / "stages" / "cook_assets" / "assets.json").read_text(
                    encoding="utf-8"
                )
            )
            report = json_loads(
                (out / "stages" / "cook_assets" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 0, report["diagnostics"])
            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertEqual(manifest["roots"], ["scenes/main.zscene"])
            self.assertEqual(
                [asset["path"] for asset in manifest["assets"]],
                ["scenes/main.zscene", "textures/hero.png"],
            )
            self.assertEqual(
                manifest["assets"][0]["dependencies"],
                ["textures/hero.png"],
            )

    def test_cook_assets_rejects_duplicate_manifest_paths_after_normalization(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            source_manifest = write_manifest(
                root,
                {
                    "roots": [],
                    "assets": [
                        {"path": "textures\\hero.png"},
                        {"path": "textures/hero.png"},
                    ],
                },
            )

            exit_code = _run_cook_assets_quiet(
                _cook_assets_args(out=out, asset_manifest=source_manifest)
            )

            report = json_loads(
                (out / "stages" / "cook_assets" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertFalse(
                (out / "stages" / "cook_assets" / "assets.json").exists()
            )
            self.assertTrue(report["fatal"])
            self.assertIn(
                "asset manifest path textures/hero.png is declared more than once",
                report["diagnostics"],
            )

    def test_cook_assets_normalizes_manifest_filter_and_labels(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            source_manifest = write_manifest(
                root,
                {
                    "roots": ["scenes/main.zscene"],
                    "asset_filter": " shipping ",
                    "assets": [
                        {
                            "path": "scenes/main.zscene",
                            "source": "main.scene",
                            "labels": [" shipping ", "editor", "shipping"],
                        }
                    ],
                },
            )

            exit_code = _run_cook_assets_quiet(
                _cook_assets_args(out=out, asset_manifest=source_manifest)
            )

            manifest = json_loads(
                (out / "stages" / "cook_assets" / "assets.json").read_text(
                    encoding="utf-8"
                )
            )
            report = json_loads(
                (out / "stages" / "cook_assets" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 0, report["diagnostics"])
            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertEqual(manifest["asset_filter"], "shipping")
            self.assertEqual(report["asset_filter"], "shipping")
            self.assertEqual(manifest["assets"][0]["labels"], ["editor", "shipping"])

    def test_cook_assets_normalizes_manifest_source_when_present(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            source_manifest = write_manifest(
                root,
                {
                    "roots": ["scenes/main.zscene"],
                    "assets": [
                        {
                            "path": "scenes/main.zscene",
                            "source": " main.scene ",
                        }
                    ],
                },
            )

            exit_code = _run_cook_assets_quiet(
                _cook_assets_args(out=out, asset_manifest=source_manifest)
            )

            manifest = json_loads(
                (out / "stages" / "cook_assets" / "assets.json").read_text(
                    encoding="utf-8"
                )
            )
            report = json_loads(
                (out / "stages" / "cook_assets" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 0, report["diagnostics"])
            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertEqual(
                manifest["assets"][0]["source"],
                str((root / "source" / "main.scene").resolve()),
            )


def write_manifest_with_asset_filter(root: Path, asset_filter: str) -> Path:
    return write_manifest(
        root,
        {
            "roots": ["scenes/main.zscene"],
            "asset_filter": asset_filter,
            "assets": [
                {
                    "path": "scenes/main.zscene",
                    "source": "main.scene",
                    "labels": ["shipping"],
                }
            ],
        },
    )


def write_manifest(root: Path, manifest: dict[str, object]) -> Path:
    source_manifest = root / "source" / "assets.json"
    source_manifest.parent.mkdir(parents=True)
    (source_manifest.parent / "main.scene").write_text("scene", encoding="utf-8")
    source_manifest.write_text(
        json_dumps(manifest),
        encoding="utf-8",
    )
    return source_manifest
