from __future__ import annotations

import contextlib
import io
import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.cli import main
from tools.zircon_export.tests.plugin_validate_support import (
    _write_complete_native_dynamic_fixture_manifest,
)
from tools.zircon_export.tests.test_plugin_build import _write_dist_plugin_workspace


def _run_plugin_validate(repo_root: Path) -> tuple[int, dict[str, object]]:
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
    return exit_code, json.loads(output.getvalue())


def _write_package_kind_fixture(repo_root: Path) -> Path:
    crate_name = "zircon_plugin_native_dynamic_fixture_native"
    _write_dist_plugin_workspace(repo_root, crate_name)
    _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
    return repo_root / "zircon_plugins" / "native_dynamic_fixture" / "plugin.toml"


def _append_manifest(manifest_path: Path, lines: list[str]) -> None:
    manifest_path.write_text(
        manifest_path.read_text(encoding="utf-8")
        + "\n"
        + "\n".join(lines)
        + "\n",
        encoding="utf-8",
    )


def _insert_before_distribution(manifest_path: Path, lines: list[str]) -> None:
    text = manifest_path.read_text(encoding="utf-8")
    marker = "\n[distribution]"
    if marker not in text:
        raise AssertionError("manifest fixture is missing [distribution]")
    manifest_path.write_text(
        text.replace(marker, "\n" + "\n".join(lines) + "\n" + marker, 1),
        encoding="utf-8",
    )


class PluginValidatePackageKindTests(unittest.TestCase):
    def test_plugin_validate_rejects_unknown_package_kind(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            plugin_manifest_path = _write_package_kind_fixture(repo_root)
            _insert_before_distribution(
                plugin_manifest_path, ['package_kind = "preview"']
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertIn(
                "plugin native_dynamic_fixture package_kind preview "
                "should be standard or feature_extension",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_feature_extension_package_kind_without_rows(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            plugin_manifest_path = _write_package_kind_fixture(repo_root)
            _insert_before_distribution(
                plugin_manifest_path, ['package_kind = "feature_extension"']
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertIn(
                "plugin native_dynamic_fixture package_kind feature_extension "
                "should declare at least one feature_extensions row",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_standard_package_kind_with_feature_rows(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            plugin_manifest_path = _write_package_kind_fixture(repo_root)
            _insert_before_distribution(
                plugin_manifest_path, ['package_kind = "standard"']
            )
            _append_manifest(
                plugin_manifest_path,
                [
                    "[[feature_extensions]]",
                    'id = "native_dynamic_fixture.preview"',
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertIn(
                "plugin native_dynamic_fixture standard package_kind "
                "should not declare feature_extensions rows",
                report["diagnostics"],
            )


if __name__ == "__main__":
    unittest.main()
