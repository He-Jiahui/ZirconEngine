from __future__ import annotations

import contextlib
import io
import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.cli import main
from tools.zircon_export.plugin_validate_retired_ui_assets import (
    PLUGIN_VALIDATE_RETIRED_UI_ASSET_SCAN_ROOTS,
    validate_plugin_retired_ui_asset_files,
    validate_plugin_target_retired_ui_asset_files,
)
from tools.zircon_export.tests.plugin_validate_support import (
    _write_complete_native_dynamic_fixture_manifest,
)
from tools.zircon_export.tests.test_plugin_build import _write_dist_plugin_workspace


class PluginValidateRetiredUiAssetTests(unittest.TestCase):
    def test_plugin_validate_retired_ui_asset_files_reports_legacy_ui_toml_files(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            editor_ui_root = repo_root / "zircon_editor" / "ui"
            plugin_ui_root = (
                repo_root
                / "zircon_plugins"
                / "native_dynamic_fixture"
                / "editor"
                / "ui"
            )
            runtime_ui_root = repo_root / "zircon_runtime" / "ui"
            editor_ui_root.mkdir(parents=True)
            plugin_ui_root.mkdir(parents=True)
            runtime_ui_root.mkdir(parents=True)
            (editor_ui_root / "legacy_editor_panel.v2.ui.toml").write_text(
                '[ui]\nkind = "panel"\n',
                encoding="utf-8",
            )
            (plugin_ui_root / "legacy_plugin_panel.ui.toml").write_text(
                '[ui]\nkind = "panel"\n',
                encoding="utf-8",
            )
            (runtime_ui_root / "current_panel.zui").write_text(
                "(panel)",
                encoding="utf-8",
            )

            diagnostics: list[str] = []
            validate_plugin_retired_ui_asset_files(repo_root, diagnostics)

            self.assertEqual(
                diagnostics,
                [
                    "plugin validate --all retired UI asset file "
                    "zircon_editor/ui/legacy_editor_panel.v2.ui.toml uses "
                    "retired UI asset suffix .v2.ui.toml; use .zui",
                    "plugin validate --all retired UI asset file "
                    "zircon_plugins/native_dynamic_fixture/editor/ui/"
                    "legacy_plugin_panel.ui.toml uses retired UI asset suffix "
                    ".ui.toml; use .zui",
                ],
            )

    def test_plugin_validate_retired_ui_asset_files_accepts_zui_only_assets(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            for scan_root in PLUGIN_VALIDATE_RETIRED_UI_ASSET_SCAN_ROOTS:
                ui_root = repo_root / scan_root / "ui"
                ui_root.mkdir(parents=True)
                (ui_root / "current_panel.zui").write_text(
                    "(panel)",
                    encoding="utf-8",
                )
            outside_root = repo_root / "docs"
            outside_root.mkdir(parents=True)
            (outside_root / "historical.ui.toml").write_text(
                '[ui]\nkind = "archived"\n',
                encoding="utf-8",
            )

            diagnostics: list[str] = []
            validate_plugin_retired_ui_asset_files(repo_root, diagnostics)

            self.assertEqual(diagnostics, [])

    def test_plugin_validate_all_reports_retired_ui_asset_files(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            plugin_ui_root = (
                repo_root
                / "zircon_plugins"
                / "native_dynamic_fixture"
                / "editor"
                / "ui"
            )
            plugin_ui_root.mkdir(parents=True)
            (plugin_ui_root / "legacy_plugin_panel.ui.toml").write_text(
                '[ui]\nkind = "panel"\n',
                encoding="utf-8",
            )

            output = io.StringIO()
            with contextlib.redirect_stdout(output):
                exit_code = main(
                    [
                        "plugin",
                        "validate",
                        "--all",
                        "--repo-root",
                        str(repo_root),
                        "--json",
                    ]
                )

            report = json.loads(output.getvalue())
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertEqual(report["target_count"], 1)
            self.assertEqual(report["failed_count"], 0)
            self.assertIn(
                "plugin validate --all retired UI asset file "
                "zircon_plugins/native_dynamic_fixture/editor/ui/"
                "legacy_plugin_panel.ui.toml uses retired UI asset suffix "
                ".ui.toml; use .zui",
                report["diagnostics"],
            )
            self.assertEqual(
                report["items"][0]["diagnostics"],
                [],
                "all-target validation should report repo-level retired UI assets once",
            )

    def test_plugin_validate_single_reports_retired_ui_asset_files(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            plugin_root = repo_root / "zircon_plugins" / "native_dynamic_fixture"
            ui_root = plugin_root / "editor" / "ui"
            ui_root.mkdir(parents=True)
            (ui_root / "legacy_plugin_panel.ui.toml").write_text(
                '[ui]\nkind = "panel"\n',
                encoding="utf-8",
            )

            output = io.StringIO()
            with contextlib.redirect_stdout(output):
                exit_code = main(
                    [
                        "plugin",
                        "validate",
                        "native_dynamic_fixture",
                        "--repo-root",
                        str(repo_root),
                        "--json",
                    ]
                )

            report = json.loads(output.getvalue())
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertIn(
                "plugin native_dynamic_fixture retired UI asset file "
                "editor/ui/legacy_plugin_panel.ui.toml uses retired UI asset "
                "suffix .ui.toml; use .zui",
                report["diagnostics"],
            )

    def test_plugin_validate_target_retired_ui_asset_files_reports_plugin_relative_paths(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            plugin_root = Path(temp_dir) / "plugin"
            ui_root = plugin_root / "editor" / "ui"
            ui_root.mkdir(parents=True)
            (ui_root / "legacy_panel.v2.ui.toml").write_text(
                '[ui]\nkind = "panel"\n',
                encoding="utf-8",
            )

            diagnostics: list[str] = []
            validate_plugin_target_retired_ui_asset_files(
                plugin_manifest_path=plugin_root / "plugin.toml",
                package_id="native_dynamic_fixture",
                diagnostics=diagnostics,
            )

            self.assertEqual(
                diagnostics,
                [
                    "plugin native_dynamic_fixture retired UI asset file "
                    "editor/ui/legacy_panel.v2.ui.toml uses retired UI asset "
                    "suffix .v2.ui.toml; use .zui",
                ],
            )


if __name__ == "__main__":
    unittest.main()
