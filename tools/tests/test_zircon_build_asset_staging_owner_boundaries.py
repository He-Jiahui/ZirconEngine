import tempfile
import types
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
ZIRCON_BUILD = REPO_ROOT / "tools/zircon_build.py"
ZIRCON_BUILD_ASSET_STAGING = REPO_ROOT / "tools/zircon_build_asset_staging.py"


class ZirconBuildAssetStagingOwnerBoundaryTests(unittest.TestCase):
    def test_asset_staging_lives_in_asset_staging_owner(self):
        self.assertTrue(
            ZIRCON_BUILD_ASSET_STAGING.exists(),
            "engine asset staging and plugin resource directory copy belong in zircon_build_asset_staging.py",
        )
        build_text = ZIRCON_BUILD.read_text(encoding="utf-8")
        staging_text = ZIRCON_BUILD_ASSET_STAGING.read_text(encoding="utf-8")

        self.assertIn(
            "from .zircon_build_asset_staging import (",
            build_text,
        )
        self.assertIn(
            "from zircon_build_asset_staging import (",
            build_text,
        )
        for function_name in (
            "stage_engine_assets",
            "copy_tree_contents",
            "stage_ui_compiled_artifacts",
            "ui_compiled_artifact_cache_root",
            "copy_asset_file",
            "copy_resource_dirs",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                build_text,
                f"{function_name} belongs in zircon_build_asset_staging.py",
            )
            self.assertIn(f"def {function_name}(", staging_text)

        for constant_name in (
            "ENGINE_ASSET_ROOTS",
            "UI_COMPILED_ARTIFACT_CACHE_ENV",
            "UI_COMPILED_ARTIFACT_STAGE_ROOT",
            "UI_COMPILED_ARTIFACT_SUFFIXES",
        ):
            self.assertNotIn(
                f"{constant_name} =",
                build_text,
                f"{constant_name} belongs in zircon_build_asset_staging.py",
            )
            self.assertIn(f"{constant_name} =", staging_text)

        self.assertIn("validate_staged_engine_asset_suffix", staging_text)
        self.assertIn(".zui", staging_text)
        self.assertLessEqual(
            len(staging_text.splitlines()),
            180,
            "zircon_build_asset_staging.py should stay focused on asset staging",
        )

    def test_asset_staging_owner_preserves_zui_and_resource_copy_semantics(self):
        from tools.zircon_build_asset_staging import (
            copy_resource_dirs,
            stage_engine_assets,
        )

        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            repo_root = root / "repo"
            engine_root = root / "out" / "ZirconEngine"
            editor_ui_root = repo_root / "zircon_editor" / "assets" / "ui"
            runtime_font_root = repo_root / "zircon_runtime" / "assets" / "fonts"
            editor_ui_root.mkdir(parents=True)
            runtime_font_root.mkdir(parents=True)
            (editor_ui_root / "panel.zui").write_text(
                """
[asset]
kind = "view"
schema_version = 1
""",
                encoding="utf-8",
            )
            (runtime_font_root / "default.font.toml").write_text(
                "[font]\nfamily = \"Default\"\n",
                encoding="utf-8",
            )
            config = types.SimpleNamespace(
                repo_root=repo_root,
                engine_root=engine_root,
                dry_run=False,
            )

            stage_engine_assets(config)

            self.assertTrue((engine_root / "assets" / "ui" / "panel.zui").exists())
            self.assertTrue(
                (engine_root / "assets" / "fonts" / "default.font.toml").exists()
            )

            plugin_root = root / "plugin"
            (plugin_root / "assets").mkdir(parents=True)
            (plugin_root / "assets" / "payload.txt").write_text(
                "payload",
                encoding="utf-8",
            )
            package_out = root / "package"

            copy_resource_dirs(plugin_root, package_out, config)

            self.assertEqual(
                "payload",
                (package_out / "assets" / "payload.txt").read_text(
                    encoding="utf-8"
                ),
            )


if __name__ == "__main__":
    unittest.main()
