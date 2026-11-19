"""Verify the staged engine archive records the selected asset root."""

import hashlib
import json
import os
import tempfile
import types
import unittest
import zipfile
from pathlib import Path
from unittest.mock import patch

from tools.build.zircon_build import parse_args, resolve_config
from tools.build.zircon_build_asset_staging import stage_engine_assets
from tools.build.zircon_build_size_report import write_deterministic_archive
from tools.build.zircon_build_staging_manifest import write_staging_manifest


ASSET = Path("icons/status-dot.svg")
TARGET = Path("assets") / ASSET
RUNTIME_SOURCE = Path("zircon_runtime") / TARGET
EDITOR_SOURCE = Path("zircon_editor") / TARGET
EDITOR_ONLY = TARGET.with_name("editor-only.svg")
RUNTIME_ONLY = TARGET.with_name("runtime-only.svg")
RUNTIME_SVG = (
    b'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16">'
    b'<circle cx="8" cy="8" r="5" fill="#228B22"/></svg>\n'
)
EDITOR_SVG = RUNTIME_SVG.replace(b"#228B22", b"#4169E1")


class ZirconBuildAssetManifestOriginTests(unittest.TestCase):
    def setUp(self):
        self.root = Path(self.enterContext(tempfile.TemporaryDirectory())).resolve()
        self.repo = self.root / "repo"
        self.enterContext(patch.dict(os.environ, {
            "ZIRCON_UI_COMPILED_ARTIFACT_CACHE": str(self.root / "ui-cache"),
        }))

    def test_runtime_windows_uses_runtime_origin_for_identical_editor_asset(self):
        self._assert_runtime_archive_origin(RUNTIME_SVG)

    def test_runtime_windows_uses_runtime_origin_for_distinct_editor_asset(self):
        self._assert_runtime_archive_origin(EDITOR_SVG)

    def test_merged_and_missing_scope_keep_editor_first(self):
        self._seed_assets(RUNTIME_SVG)
        for scope, options in (("all", {"asset_scope": "all"}), ("implicit", {})):
            with self.subTest(scope=scope):
                config = self._dev_config(scope, **options)
                stage_engine_assets(config)
                document = json.loads(write_staging_manifest(config).read_bytes())
                row = self._target_row(document)
                self.assertEqual(
                    {"kind": "source_asset", "path": EDITOR_SOURCE.as_posix()},
                    row["source"],
                )
                self.assertTrue((config.engine_root / EDITOR_ONLY).is_file())
                self.assertTrue((config.engine_root / RUNTIME_ONLY).is_file())

    def test_none_scope_clears_assets_before_writing_manifest(self):
        self._seed_assets(RUNTIME_SVG)
        config = self._dev_config("none", asset_scope="none")
        self._write(config.engine_root / TARGET, RUNTIME_SVG)
        stage_engine_assets(config)
        self.assertFalse((config.engine_root / "assets").exists())
        self.assertEqual([], json.loads(write_staging_manifest(config).read_bytes())["artifacts"])

    def test_editor_windows_keeps_assets_outside_engine_manifest(self):
        self._seed_assets(EDITOR_SVG)
        config = self._shipping_config("editor-windows")
        config.engine_root.mkdir(parents=True)
        stage_engine_assets(config, config.asset_scope)
        self.assertIsNotNone(config.editor_asset_root)
        self.assertEqual(EDITOR_SVG, (config.editor_asset_root / ASSET).read_bytes())
        self.assertFalse((config.editor_asset_root / "icons/runtime-only.svg").exists())
        self.assertFalse((config.engine_root / "assets").exists())
        self.assertEqual([], json.loads(write_staging_manifest(config).read_bytes())["artifacts"])

    def _assert_runtime_archive_origin(self, editor_bytes):
        self._seed_assets(editor_bytes)
        config = self._shipping_config("runtime-windows")
        self.assertEqual("runtime", config.asset_scope)
        stage_engine_assets(config, config.asset_scope)
        manifest_path = write_staging_manifest(config)
        profile = config.product_profile
        self.assertIsNotNone(profile)
        write_deterministic_archive(
            config.engine_root,
            config.archive_path,
            product_kind=profile.product_kind,
            forbidden_path_tokens=profile.forbidden_path_tokens,
            include_project_pack=profile.include_project_pack,
        )
        with zipfile.ZipFile(config.archive_path) as archive:
            self.assertEqual(RUNTIME_SVG, archive.read(TARGET.as_posix()))
            self.assertIn(RUNTIME_ONLY.as_posix(), archive.namelist())
            self.assertNotIn(EDITOR_ONLY.as_posix(), archive.namelist())
            manifest = archive.read(manifest_path.name)
        self.assertEqual(manifest_path.read_bytes(), manifest)
        row = self._target_row(json.loads(manifest))
        self.assertEqual(hashlib.sha256(RUNTIME_SVG).hexdigest(), row["sha256"])
        self.assertEqual(
            {"kind": "source_asset", "path": RUNTIME_SOURCE.as_posix()}, row["source"]
        )

    def _shipping_config(self, product):
        args = parse_args([
            "--product-profile", product, "--mode", "shipping",
            "--out", str(self.root / "out"),
        ])
        return resolve_config(args, self.repo, ())

    def _dev_config(self, name, **options):
        out = self.root / name
        return types.SimpleNamespace(
            repo_root=self.repo,
            out_root=out,
            engine_root=out / "ZirconEngine",
            targets_root=out / "targets",
            mode="debug",
            targets=("editor", "runtime"),
            runtime_features=("target-client",),
            dry_run=False,
            **options,
        )

    def _seed_assets(self, editor_bytes):
        self._write(self.repo / RUNTIME_SOURCE, RUNTIME_SVG)
        self._write(self.repo / EDITOR_SOURCE, editor_bytes)
        self._write(self.repo / "zircon_runtime" / RUNTIME_ONLY, RUNTIME_SVG)
        self._write(self.repo / "zircon_editor" / EDITOR_ONLY, RUNTIME_SVG)

    @staticmethod
    def _write(path, contents):
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(contents)

    @staticmethod
    def _target_row(document):
        (row,) = [row for row in document["artifacts"] if row["target_path"] == TARGET.as_posix()]
        return row
