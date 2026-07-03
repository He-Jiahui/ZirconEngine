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


class PluginValidateCapabilityStatusTests(unittest.TestCase):
    def test_plugin_validate_rejects_malformed_capability_status_row(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _append_manifest(
                repo_root / "zircon_plugins/native_dynamic_fixture/plugin.toml",
                [
                    "[[capability_statuses]]",
                    'capability = "Runtime.Plugin.Bad"',
                    'status = "done"',
                    'note = " "',
                    "",
                    "[[capability_statuses]]",
                    'status = "partial"',
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            diagnostics = report["diagnostics"]
            self.assertIn(
                "plugin native_dynamic_fixture capability_statuses[0].capability "
                "Runtime.Plugin.Bad should contain only lowercase ASCII letters, "
                "digits, underscores, and dots",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture capability_statuses[0].capability "
                "Runtime.Plugin.Bad must reference a package or optional feature "
                "capability declared by the same package",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture capability_statuses[0].status done "
                "should be one of complete, partial, stub, externalized, unsupported",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture capability_statuses[0].note "
                "must be a non-empty trimmed string",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture capability_statuses[1].capability "
                "is required",
                diagnostics,
            )

    def test_plugin_validate_rejects_duplicate_capability_status_row(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _append_manifest(
                repo_root / "zircon_plugins/native_dynamic_fixture/plugin.toml",
                [
                    "[[capability_statuses]]",
                    'capability = "runtime.plugin.native_dynamic_fixture"',
                    'status = "partial"',
                    "",
                    "[[capability_statuses]]",
                    'capability = "runtime.plugin.native_dynamic_fixture"',
                    'status = "complete"',
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            self.assertIn(
                "plugin native_dynamic_fixture capability_statuses[1].capability "
                "runtime.plugin.native_dynamic_fixture duplicates capability_status "
                "capability_statuses[0]",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_unknown_capability_status_fields(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _append_manifest(
                repo_root / "zircon_plugins/native_dynamic_fixture/plugin.toml",
                [
                    "[[capability_statuses]]",
                    'capability = "runtime.plugin.native_dynamic_fixture"',
                    'status = "partial"',
                    'sidecar = "unexpected"',
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            self.assertIn(
                "plugin native_dynamic_fixture capability_statuses[0].sidecar "
                "is not a known capability_status field",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_capability_status_target_modes_drift(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _append_manifest(
                repo_root / "zircon_plugins/native_dynamic_fixture/plugin.toml",
                [
                    "[[capability_statuses]]",
                    'capability = "runtime.plugin.native_dynamic_fixture"',
                    'status = "partial"',
                    'target_modes = ["client_runtime", "desktop", "client_runtime", "server_runtime"]',
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            diagnostics = report["diagnostics"]
            self.assertIn(
                'plugin native_dynamic_fixture capability_statuses[0].target_modes[1] '
                '"desktop" is unsupported; expected one of client_runtime, '
                "server_runtime, editor_host",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture capability_statuses[0].target_modes[2] "
                "client_runtime duplicates capability_status target_modes target_modes[0]",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture capability_statuses[0].target_modes[3] "
                "server_runtime should be covered by package supported_targets",
                diagnostics,
            )

    def test_plugin_validate_rejects_empty_capability_status_optional_arrays(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _append_manifest(
                repo_root / "zircon_plugins/native_dynamic_fixture/plugin.toml",
                [
                    "[[capability_statuses]]",
                    'capability = "runtime.plugin.native_dynamic_fixture"',
                    'status = "partial"',
                    "target_modes = []",
                    "bevy_references = []",
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            diagnostics = report["diagnostics"]
            self.assertIn(
                "plugin native_dynamic_fixture capability_statuses[0].target_modes "
                "must not be empty when declared",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture capability_statuses[0].bevy_references "
                "must not be empty when declared",
                diagnostics,
            )

    def test_plugin_validate_rejects_capability_status_bevy_reference_drift(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _append_manifest(
                repo_root / "zircon_plugins/native_dynamic_fixture/plugin.toml",
                [
                    "[[capability_statuses]]",
                    'capability = "runtime.plugin.native_dynamic_fixture"',
                    'status = "partial"',
                    'bevy_references = ["dev/bevy/crates/bevy_app/src/app.rs", "bevy/crates/bevy_ecs/src/world/mod.rs", "dev/bevy/../bad.rs", "dev\\\\bevy\\\\bad.rs", "dev/bevy/crates/bevy_app/src/app.rs"]',
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            diagnostics = report["diagnostics"]
            self.assertIn(
                "plugin native_dynamic_fixture capability_statuses[0].bevy_references[1] "
                "bevy/crates/bevy_ecs/src/world/mod.rs should start with dev/bevy/",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture capability_statuses[0].bevy_references[2] "
                "dev/bevy/../bad.rs should not contain empty, current, or parent path segments",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture capability_statuses[0].bevy_references[3] "
                "dev\\bevy\\bad.rs should use repository-relative forward-slash paths",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture capability_statuses[0].bevy_references[4] "
                "dev/bevy/crates/bevy_app/src/app.rs duplicates capability_status "
                "bevy_references bevy_references[0]",
                diagnostics,
            )


if __name__ == "__main__":
    unittest.main()
