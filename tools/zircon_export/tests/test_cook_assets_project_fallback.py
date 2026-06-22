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


class CookAssetsProjectFallbackTests(unittest.TestCase):
    def test_cook_assets_project_manifest_asset_manifest_records_dependency_closure(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            project_root = root / "project"
            project = project_root / "zircon-project.toml"
            assets_root = project_root / "assets"
            source_manifest = project_root / "export" / "assets.json"
            scene = assets_root / "scenes" / "main.scene.toml"
            material = assets_root / "materials" / "player.zmaterial"
            texture = assets_root / "textures" / "player_albedo.png"
            scene.parent.mkdir(parents=True)
            material.parent.mkdir(parents=True)
            texture.parent.mkdir(parents=True)
            source_manifest.parent.mkdir(parents=True)
            scene.write_bytes(b"\x00scene-without-text-res-references")
            material.write_bytes(b"material")
            texture.write_bytes(b"texture")
            source_manifest.write_text(
                json_dumps(
                    {
                        "roots": ["scenes/main.scene.toml"],
                        "assets": [
                            {
                                "path": "textures/player_albedo.png",
                                "source": "../assets/textures/player_albedo.png",
                                "dependencies": [],
                                "labels": [],
                            },
                            {
                                "path": "materials/player.zmaterial",
                                "source": "../assets/materials/player.zmaterial",
                                "dependencies": ["textures/player_albedo.png"],
                                "labels": [],
                            },
                            {
                                "path": "scenes/main.scene.toml",
                                "source": "../assets/scenes/main.scene.toml",
                                "dependencies": ["materials/player.zmaterial"],
                                "labels": [],
                            },
                        ],
                    }
                ),
                encoding="utf-8",
            )
            project.write_text(
                "\n".join(
                    [
                        'name = "Export Fixture"',
                        'default_scene = "res://scenes/main.scene.toml"',
                        'asset_manifest = "export/assets.json"',
                    ]
                ),
                encoding="utf-8",
            )
            out = root / "out"
            args = _cook_assets_args(out=out, project=project)
            args.asset_filter = "shipping"

            exit_code = _run_cook_assets_quiet(args)

            self.assertEqual(exit_code, 0)
            manifest = json_loads(
                (out / "stages" / "cook_assets" / "assets.json").read_text(
                    encoding="utf-8"
                )
            )
            assets_by_path = {
                asset["path"]: asset for asset in manifest["assets"]
            }
            self.assertEqual(
                sorted(assets_by_path),
                [
                    "materials/player.zmaterial",
                    "scenes/main.scene.toml",
                    "textures/player_albedo.png",
                ],
            )
            self.assertEqual(
                assets_by_path["scenes/main.scene.toml"]["dependencies"],
                ["materials/player.zmaterial"],
            )
            self.assertEqual(
                assets_by_path["materials/player.zmaterial"]["dependencies"],
                ["textures/player_albedo.png"],
            )
            self.assertEqual(manifest["asset_filter"], "shipping")
            report = json_loads(
                (out / "stages" / "cook_assets" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertTrue(report["generated_from_project"])
            self.assertEqual(
                report["source_asset_manifest"],
                str(source_manifest.resolve()),
            )
            self.assertEqual(report["project_default_scene"], "res://scenes/main.scene.toml")
            self.assertEqual(report["asset_count"], 3)

    def test_cook_assets_project_manifest_rejects_unsafe_asset_manifest_path(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            project_root = root / "project"
            project = project_root / "zircon-project.toml"
            scene = project_root / "assets" / "scenes" / "main.scene.toml"
            scene.parent.mkdir(parents=True)
            scene.write_text("scene", encoding="utf-8")
            project.write_text(
                "\n".join(
                    [
                        'name = "Export Fixture"',
                        'default_scene = "res://scenes/main.scene.toml"',
                        'asset_manifest = "../outside/assets.json"',
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
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertFalse((out / "stages" / "cook_assets" / "assets.json").exists())
            self.assertIn(
                (
                    f"project manifest {project.resolve()} asset_manifest "
                    "../outside/assets.json must be a safe relative path"
                ),
                report["diagnostics"],
            )

    def test_cook_assets_project_fallback_rejects_default_scene_empty_path_segment(
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
                        'default_scene = "res://scenes//main.scene.toml"',
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
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertFalse((out / "stages" / "cook_assets" / "assets.json").exists())
            self.assertIn(
                (
                    "project default_scene res://scenes//main.scene.toml "
                    "does not resolve to a safe asset path"
                ),
                report["diagnostics"],
            )

    def test_cook_assets_project_fallback_rejects_direct_reference_empty_path_segment(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            project = root / "project" / "zircon-project.toml"
            assets_root = root / "project" / "assets"
            scene = assets_root / "scenes" / "main.scene.toml"
            texture = assets_root / "textures" / "hero.png"
            scene.parent.mkdir(parents=True)
            texture.parent.mkdir(parents=True)
            scene.write_text(
                'texture = "res://textures//hero.png"',
                encoding="utf-8",
            )
            texture.write_bytes(b"texture")
            project.write_text(
                "\n".join(
                    [
                        'name = "Export Fixture"',
                        'default_scene = "res://scenes/main.scene.toml"',
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
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertFalse((out / "stages" / "cook_assets" / "assets.json").exists())
            self.assertIn(
                (
                    "project asset reference res://textures//hero.png "
                    "does not resolve to a safe asset path"
                ),
                report["diagnostics"],
            )

    def test_cook_assets_project_fallback_records_recursive_direct_references(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            project = root / "project" / "zircon-project.toml"
            assets_root = root / "project" / "assets"
            scene = assets_root / "scenes" / "main.scene.toml"
            material = assets_root / "materials" / "player.zmaterial"
            texture = assets_root / "textures" / "player_albedo.png"
            scene.parent.mkdir(parents=True)
            material.parent.mkdir(parents=True)
            texture.parent.mkdir(parents=True)
            scene.write_text(
                'material = "res://materials/player.zmaterial"',
                encoding="utf-8",
            )
            material.write_text(
                'albedo = "res://textures/player_albedo.png#main"',
                encoding="utf-8",
            )
            texture.write_bytes(b"texture")
            project.write_text(
                "\n".join(
                    [
                        'name = "Export Fixture"',
                        'default_scene = "res://scenes/main.scene.toml"',
                    ]
                ),
                encoding="utf-8",
            )
            out = root / "out"
            args = _cook_assets_args(out=out, project=project)
            args.asset_filter = "shipping"

            exit_code = _run_cook_assets_quiet(args)

            self.assertEqual(exit_code, 0)
            manifest = json_loads(
                (out / "stages" / "cook_assets" / "assets.json").read_text(
                    encoding="utf-8"
                )
            )
            assets_by_path = {
                asset["path"]: asset for asset in manifest["assets"]
            }
            self.assertEqual(
                sorted(assets_by_path),
                [
                    "materials/player.zmaterial",
                    "scenes/main.scene.toml",
                    "textures/player_albedo.png",
                ],
            )
            self.assertEqual(
                assets_by_path["scenes/main.scene.toml"]["dependencies"],
                ["materials/player.zmaterial"],
            )
            self.assertEqual(
                assets_by_path["materials/player.zmaterial"]["dependencies"],
                ["textures/player_albedo.png"],
            )
            self.assertEqual(
                assets_by_path["textures/player_albedo.png"]["dependencies"],
                [],
            )
            self.assertEqual(
                assets_by_path["textures/player_albedo.png"]["source"],
                str(texture.resolve()),
            )
            self.assertEqual(
                assets_by_path["materials/player.zmaterial"]["labels"],
                ["shipping"],
            )
            report = json_loads(
                (out / "stages" / "cook_assets" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertEqual(report["asset_count"], 3)

    def test_cook_assets_project_fallback_treats_binary_reference_as_leaf(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            project = root / "project" / "zircon-project.toml"
            assets_root = root / "project" / "assets"
            scene = assets_root / "scenes" / "main.scene.toml"
            texture = assets_root / "textures" / "player_albedo.png"
            scene.parent.mkdir(parents=True)
            texture.parent.mkdir(parents=True)
            scene.write_text(
                'texture = "res://textures/player_albedo.png"',
                encoding="utf-8",
            )
            texture.write_bytes(b"\x89PNG\r\n\x1a\n\xff\x00")
            project.write_text(
                "\n".join(
                    [
                        'name = "Export Fixture"',
                        'default_scene = "res://scenes/main.scene.toml"',
                    ]
                ),
                encoding="utf-8",
            )
            out = root / "out"

            exit_code = _run_cook_assets_quiet(
                _cook_assets_args(out=out, project=project)
            )

            self.assertEqual(exit_code, 0)
            manifest = json_loads(
                (out / "stages" / "cook_assets" / "assets.json").read_text(
                    encoding="utf-8"
                )
            )
            assets_by_path = {
                asset["path"]: asset for asset in manifest["assets"]
            }
            self.assertEqual(
                sorted(assets_by_path),
                ["scenes/main.scene.toml", "textures/player_albedo.png"],
            )
            self.assertEqual(
                assets_by_path["textures/player_albedo.png"]["dependencies"],
                [],
            )
            report = json_loads(
                (out / "stages" / "cook_assets" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertEqual(report["asset_count"], 2)

    def test_cook_assets_project_fallback_orders_assets_and_dependencies_deterministically(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            project = root / "project" / "zircon-project.toml"
            assets_root = root / "project" / "assets"
            scene = assets_root / "scenes" / "main.scene.toml"
            material = assets_root / "materials" / "player.zmaterial"
            texture_a = assets_root / "textures" / "a.png"
            texture_b = assets_root / "textures" / "b.png"
            texture_z = assets_root / "textures" / "z.png"
            scene.parent.mkdir(parents=True)
            material.parent.mkdir(parents=True)
            texture_a.parent.mkdir(parents=True)
            scene.write_text(
                "\n".join(
                    [
                        'z_texture = "res://textures/z.png"',
                        'material = "res://materials/player.zmaterial"',
                        'z_texture_again = "res://textures/z.png#main"',
                    ]
                ),
                encoding="utf-8",
            )
            material.write_text(
                "\n".join(
                    [
                        'b_texture = "res://textures/b.png"',
                        'a_texture = "res://textures/a.png"',
                    ]
                ),
                encoding="utf-8",
            )
            texture_a.write_bytes(b"a")
            texture_b.write_bytes(b"b")
            texture_z.write_bytes(b"z")
            project.write_text(
                "\n".join(
                    [
                        'name = "Export Fixture"',
                        'default_scene = "res://scenes/main.scene.toml"',
                    ]
                ),
                encoding="utf-8",
            )
            out = root / "out"

            exit_code = _run_cook_assets_quiet(
                _cook_assets_args(out=out, project=project)
            )

            self.assertEqual(exit_code, 0)
            manifest = json_loads(
                (out / "stages" / "cook_assets" / "assets.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(
                [asset["path"] for asset in manifest["assets"]],
                [
                    "materials/player.zmaterial",
                    "scenes/main.scene.toml",
                    "textures/a.png",
                    "textures/b.png",
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
                ["textures/a.png", "textures/b.png"],
            )


if __name__ == "__main__":
    unittest.main()
