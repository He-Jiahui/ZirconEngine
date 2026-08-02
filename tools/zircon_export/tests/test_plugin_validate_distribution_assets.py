from __future__ import annotations

import contextlib
import io
import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.cli import main
from tools.zircon_export.plugin_validate_distribution_assets import (
    _is_plugin_relative_asset_glob,
    plugin_validate_distribution_assets,
)
from tools.zircon_export.tests.plugin_validate_support import (
    _replace_manifest_line,
    _write_complete_native_dynamic_fixture_manifest,
)
from tools.zircon_export.tests.test_plugin_build import _write_dist_plugin_workspace


class PluginValidateDistributionAssetTests(unittest.TestCase):
    def test_distribution_asset_glob_rejects_windows_root_relative_form(self) -> None:
        self.assertFalse(_is_plugin_relative_asset_glob(r"\outside\**"))

    def test_distribution_assets_rejects_windows_drive_relative_glob(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            plugin_root = Path(temp_dir) / "plugin"
            plugin_root.mkdir()
            diagnostics: list[str] = []

            plugin_validate_distribution_assets(
                {"assets": ["C:outside/**"]},
                "native_dynamic_fixture",
                diagnostics,
                plugin_manifest_path=plugin_root / "plugin.toml",
            )

            self.assertEqual(
                diagnostics,
                [
                    "plugin native_dynamic_fixture distribution.assets[0] "
                    "must be a plugin-relative glob"
                ],
            )

    def test_distribution_assets_terminal_recursive_glob_matches_nested_files(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            plugin_root = Path(temp_dir) / "plugin"
            shader_root = plugin_root / "assets" / "shaders"
            shader_root.mkdir(parents=True)
            (shader_root / "surface.wgsl").write_text("// fixture\n", encoding="utf-8")

            diagnostics: list[str] = []
            plugin_validate_distribution_assets(
                {"assets": ["assets/**"]},
                "native_dynamic_fixture",
                diagnostics,
                plugin_manifest_path=plugin_root / "plugin.toml",
            )

            self.assertEqual(diagnostics, [])

    def test_distribution_assets_rejects_malformed_zui_documents(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            plugin_root = Path(temp_dir) / "plugin"
            ui_root = plugin_root / "editor" / "ui"
            ui_root.mkdir(parents=True)
            (ui_root / "broken.zui").write_text("[asset\nkind = \"view\"\n")

            diagnostics: list[str] = []
            plugin_validate_distribution_assets(
                {"assets": ["editor/ui/*.zui"]},
                "native_dynamic_fixture",
                diagnostics,
                plugin_manifest_path=plugin_root / "plugin.toml",
            )

            self.assertTrue(
                any(
                    diagnostic.startswith(
                        "plugin native_dynamic_fixture distribution.assets[0] "
                        "matched .zui asset editor/ui/broken.zui could not be "
                        "parsed as TOML:"
                    )
                    for diagnostic in diagnostics
                ),
                diagnostics,
            )

    def test_distribution_assets_rejects_zui_documents_without_known_kind(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            plugin_root = Path(temp_dir) / "plugin"
            ui_root = plugin_root / "editor" / "ui"
            ui_root.mkdir(parents=True)
            (ui_root / "missing_kind.zui").write_text("[asset]\n")
            (ui_root / "unsupported_kind.zui").write_text(
                '[asset]\nkind = "blueprint"\n'
            )

            diagnostics: list[str] = []
            plugin_validate_distribution_assets(
                {"assets": ["editor/ui/*.zui"]},
                "native_dynamic_fixture",
                diagnostics,
                plugin_manifest_path=plugin_root / "plugin.toml",
            )

            self.assertEqual(
                diagnostics,
                [
                    "plugin native_dynamic_fixture distribution.assets[0] "
                    "matched .zui asset editor/ui/missing_kind.zui must "
                    "declare asset.kind",
                    "plugin native_dynamic_fixture distribution.assets[0] "
                    "matched .zui asset editor/ui/unsupported_kind.zui has "
                    "unsupported asset.kind blueprint; expected one of "
                    "component, style, theme_tokens, view",
                ],
            )

    def test_distribution_assets_accepts_current_zui_document_kinds(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            plugin_root = Path(temp_dir) / "plugin"
            ui_root = plugin_root / "editor" / "ui"
            ui_root.mkdir(parents=True)
            for kind in ("component", "style", "theme_tokens", "view"):
                (ui_root / f"{kind}.zui").write_text(
                    f'[asset]\nkind = "{kind}"\n',
                    encoding="utf-8",
                )

            diagnostics: list[str] = []
            plugin_validate_distribution_assets(
                {"assets": ["editor/ui/*.zui"]},
                "native_dynamic_fixture",
                diagnostics,
                plugin_manifest_path=plugin_root / "plugin.toml",
            )

            self.assertEqual(diagnostics, [])

    def test_plugin_validate_reports_distribution_assets_zui_kind_drift(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            plugin_root = repo_root / "zircon_plugins" / "native_dynamic_fixture"
            ui_root = plugin_root / "editor" / "ui"
            ui_root.mkdir(parents=True)
            (ui_root / "bad_kind.zui").write_text(
                '[asset]\nkind = "blueprint"\n',
                encoding="utf-8",
            )
            _replace_manifest_line(
                plugin_root / "plugin.toml",
                'runtime_entry = "zircon_native_dynamic_fixture_runtime_entry_v3"',
                'runtime_entry = "zircon_native_dynamic_fixture_runtime_entry_v3"\n'
                'assets = ["editor/ui/*.zui"]',
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
                "plugin native_dynamic_fixture distribution.assets[0] matched "
                ".zui asset editor/ui/bad_kind.zui has unsupported asset.kind "
                "blueprint; expected one of component, style, theme_tokens, view",
                report["diagnostics"],
            )


if __name__ == "__main__":
    unittest.main()
