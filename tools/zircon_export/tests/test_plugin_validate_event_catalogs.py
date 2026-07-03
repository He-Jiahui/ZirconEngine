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


def _append_manifest(manifest_path: Path, lines: list[str]) -> None:
    manifest_path.write_text(
        manifest_path.read_text(encoding="utf-8")
        + "\n"
        + "\n".join(lines)
        + "\n",
        encoding="utf-8",
    )


class PluginValidateEventCatalogTests(unittest.TestCase):
    def test_plugin_validate_rejects_malformed_event_catalog(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _append_manifest(
                repo_root / "zircon_plugins/native_dynamic_fixture/plugin.toml",
                [
                    "[[event_catalogs]]",
                    'namespace = "Native.Dynamic"',
                    'version = "1"',
                    "events = []",
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            diagnostics = report["diagnostics"]
            self.assertIn(
                "plugin native_dynamic_fixture event_catalogs[0].namespace "
                "Native.Dynamic should contain only lowercase ASCII letters, "
                "digits, underscores, and dots",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture event_catalogs[0].version "
                "must be an integer",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture event_catalogs[0].events "
                "must not be empty when declared",
                diagnostics,
            )

    def test_plugin_validate_rejects_malformed_event_rows(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _append_manifest(
                repo_root / "zircon_plugins/native_dynamic_fixture/plugin.toml",
                [
                    "[[event_catalogs]]",
                    'namespace = "native_dynamic_fixture.events"',
                    "version = 1",
                    "events = [",
                    '  { id = "native_dynamic_fixture.other.started", '
                    'display_name = " Started ", '
                    'payload_schema = "external.started.v01" },',
                    '  { id = "native_dynamic_fixture.other.started", '
                    'display_name = "Started" },',
                    "]",
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            diagnostics = report["diagnostics"]
            self.assertIn(
                "plugin native_dynamic_fixture event_catalogs[0].events[0].id "
                "native_dynamic_fixture.other.started should stay under namespace "
                "native_dynamic_fixture.events.",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture event_catalogs[0].events[0].display_name "
                "must be a non-empty trimmed string",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture event_catalogs[0].events[0].payload_schema "
                "external.started.v01 should stay under package namespace "
                "native_dynamic_fixture.",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture event_catalogs[0].events[0].payload_schema "
                "external.started.v01 version segment should be a positive integer "
                "without leading zeroes",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture event_catalogs[0].events[1].id "
                "duplicates event row 0",
                diagnostics,
            )

    def test_plugin_validate_rejects_duplicate_event_catalog_namespace(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            event_catalog = [
                "[[event_catalogs]]",
                'namespace = "native_dynamic_fixture.events"',
                "version = 1",
                "events = [",
                '  { id = "native_dynamic_fixture.events.started", '
                'display_name = "Started", '
                'payload_schema = "native_dynamic_fixture.started.v1" },',
                "]",
            ]
            _append_manifest(
                repo_root / "zircon_plugins/native_dynamic_fixture/plugin.toml",
                event_catalog,
            )
            other_manifest = repo_root / "zircon_plugins/other/plugin.toml"
            other_manifest.parent.mkdir(parents=True, exist_ok=True)
            other_manifest.write_text(
                "\n".join(
                    [
                        'id = "other"',
                        'capabilities = ["runtime.plugin.other"]',
                        *event_catalog,
                    ]
                ),
                encoding="utf-8",
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            self.assertIn(
                "plugin native_dynamic_fixture event_catalogs[0].namespace "
                "native_dynamic_fixture.events duplicates event catalog namespace "
                "declared by other",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_unknown_event_catalog_fields(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _append_manifest(
                repo_root / "zircon_plugins/native_dynamic_fixture/plugin.toml",
                [
                    "[[event_catalogs]]",
                    'namespace = "native_dynamic_fixture.events"',
                    "version = 1",
                    'sidecar = "unexpected"',
                    "events = [",
                    '  { id = "native_dynamic_fixture.events.started", '
                    'display_name = "Started", '
                    'payload_schema = "native_dynamic_fixture.started.v1", '
                    'sidecar = "unexpected" },',
                    "]",
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            diagnostics = report["diagnostics"]
            self.assertIn(
                "plugin native_dynamic_fixture event_catalogs[0].sidecar "
                "is not a known event catalog field",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture event_catalogs[0].events[0].sidecar "
                "is not a known event field",
                diagnostics,
            )


if __name__ == "__main__":
    unittest.main()
