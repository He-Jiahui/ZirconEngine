"""Observe CookAssets to Pack at the real CLI subprocess boundary."""

from __future__ import annotations

import contextlib
import io
import shutil
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.export.cli import main
from tools.export.tests.export_test_support import (
    REPO_ROOT,
    _write_validate_report_with_asset_filter,
    json_dumps,
    json_loads,
)


class PipelineManifestHandoffTests(unittest.TestCase):
    def test_same_run_cook_sources_do_not_replace_the_cooked_pack_input(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            project_root = root / "project"
            shutil.copytree(
                REPO_ROOT / "templates" / "projects" / "renderable-empty",
                project_root,
            )
            project = project_root / "zircon-project.toml"
            source = project_root / "source_assets.json"
            project.write_text(
                'project_guid = "3509fca7-84da-4911-8e9d-839e10f6f358"\n'
                'asset_manifest = "source_assets.json"\n'
                + project.read_text(encoding="utf-8")
                .replace("[export_profiles.desktop_windows]", "[export_profiles.windows-release]")
                .replace(
                    'path = ["source_template", "library_embed", "native_dynamic"]',
                    'path = ["library_embed"]\nasset_filter = "shipping"',
                ),
                encoding="utf-8",
            )
            scene = "scenes/main.scene.toml"
            material = "materials/default.zmaterial"
            model = "models/cube.obj"
            graph = {scene: [model, material], material: [], model: []}

            # Ordinary Windows-style package paths are accepted by CookAssets
            # and normalized before serialization. All bytes come from the template.
            raw_manifest = {
                "roots": [r"scenes\main.scene.toml"],
                "assets": [
                    {
                        "path": path.replace("/", "\\"),
                        "source": f"assets/{path}",
                        "dependencies": [edge.replace("/", "\\") for edge in edges],
                        "labels": ["shipping"],
                    }
                    for path, edges in graph.items()
                ],
            }
            source.write_text(json_dumps(raw_manifest), encoding="utf-8")

            def capture_packer_input(out: Path, *args: str) -> Path:
                argv = [
                    "--profile", "windows-release", "--project", str(project),
                    "--out", str(out), *args,
                ]
                # Stop at the external executable, without minting a success report.
                # CookAssets, Pack preflight, CLI routing, and packer argv are real.
                with contextlib.redirect_stdout(io.StringIO()), mock.patch(
                    "tools.export.pack_stage.subprocess.call", return_value=79
                ) as packer:
                    exit_code = main(argv)
                packer.assert_called_once()
                self.assertEqual(exit_code, 79)
                command = packer.call_args.args[0]
                return Path(command[command.index("--manifest") + 1])

            declared_out = root / "project-declared"
            explicit_out = root / "explicit"
            for out, options in (
                (declared_out, ()),
                (explicit_out, ("--asset-manifest", str(source))),
            ):
                _write_validate_report_with_asset_filter(out, "shipping")
                pack_input = capture_packer_input(out, "--resume-from", "cook_assets", *options)
                cooked_path = out / "stages" / "cook_assets" / "assets.json"
                report = json_loads(
                    cooked_path.with_name("report.json").read_text(encoding="utf-8")
                )
                cooked = json_loads(cooked_path.read_text(encoding="utf-8"))
                self.assertFalse(report["fatal"], report["diagnostics"])
                self.assertEqual(Path(report["source_asset_manifest"]), source)
                self.assertEqual(Path(report["cooked_asset_manifest"]), cooked_path)
                self.assertEqual(report["root_count"], 1)
                self.assertEqual(report["asset_count"], len(graph))
                self.assertEqual(cooked["roots"], [scene])
                self.assertEqual(cooked["asset_filter"], "shipping")
                self.assertEqual(
                    {row["path"]: row["dependencies"] for row in cooked["assets"]},
                    {path: sorted(edges) for path, edges in graph.items()},
                )
                self.assertEqual(
                    {Path(row["source"]) for row in cooked["assets"]},
                    {project_root / "assets" / path for path in graph},
                )

                if out == declared_out:
                    self.assertEqual(pack_input, cooked_path)
                    # Pack-only callers keep a direct, pre-normalized input override.
                    override = root / "pack_input.json"
                    override.write_text(json_dumps(cooked), encoding="utf-8")
                    for entry in (("--stage", "pack"), ("--resume-from", "pack")):
                        self.assertEqual(
                            capture_packer_input(out, *entry, "--asset-manifest", str(override)),
                            override,
                        )
                else:
                    self.assertEqual(pack_input, cooked_path)
