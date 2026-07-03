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


class PluginValidateInterfaceTests(unittest.TestCase):
    def test_plugin_validate_rejects_malformed_provided_interface_id(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _append_manifest(
                repo_root / "zircon_plugins/native_dynamic_fixture/plugin.toml",
                [
                    "[[provides_interfaces]]",
                    'id = "Native.Dynamic"',
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            self.assertIn(
                "plugin native_dynamic_fixture provides_interfaces[0].id "
                "Native.Dynamic should contain only lowercase ASCII letters, "
                "digits, underscores, and dots",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_duplicate_provided_interface_id(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _append_manifest(
                repo_root / "zircon_plugins/native_dynamic_fixture/plugin.toml",
                [
                    "[[provides_interfaces]]",
                    'id = "native_dynamic_fixture.runtime"',
                    "",
                    "[[provides_interfaces]]",
                    'id = "native_dynamic_fixture.runtime"',
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            self.assertIn(
                "plugin native_dynamic_fixture provides_interfaces[1].id "
                "native_dynamic_fixture.runtime duplicates provided interface id "
                "provides_interfaces[0]",
                report["diagnostics"],
            )

    def test_plugin_validate_rejects_dependency_interface_namespace_and_duplicate(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _append_manifest(
                repo_root / "zircon_plugins/native_dynamic_fixture/plugin.toml",
                [
                    "[[dependencies]]",
                    'id = "renderer"',
                    "required = true",
                    'interfaces = ["Renderer.Query", "renderer.query", "renderer.query"]',
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            diagnostics = report["diagnostics"]
            self.assertIn(
                "plugin native_dynamic_fixture dependencies[0].interfaces[0] "
                "Renderer.Query should contain only lowercase ASCII letters, "
                "digits, underscores, and dots",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture dependencies[0].interfaces[2] "
                "renderer.query duplicates dependency interface interfaces[1]",
                diagnostics,
            )

    def test_plugin_validate_rejects_malformed_provided_interface_method(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _append_manifest(
                repo_root / "zircon_plugins/native_dynamic_fixture/plugin.toml",
                [
                    "[[provides_interfaces]]",
                    'id = "native_dynamic_fixture.runtime"',
                    "",
                    "[[provides_interfaces.methods]]",
                    'name = "Tick"',
                    'method_slot = "0"',
                    'parameters = [{ name = "Payload" }]',
                    'required_capabilities = ["Runtime.Bad"]',
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            diagnostics = report["diagnostics"]
            self.assertIn(
                "plugin native_dynamic_fixture provides_interfaces[0].methods[0].name "
                "Tick should contain only lowercase ASCII letters, digits, and underscores",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture provides_interfaces[0].methods[0].method_slot "
                "must be a non-negative integer",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture provides_interfaces[0].methods[0].parameters[0].name "
                "Payload should contain only lowercase ASCII letters, digits, and underscores",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture provides_interfaces[0].methods[0].required_capabilities[0] "
                "Runtime.Bad should contain only lowercase ASCII letters, digits, underscores, and dots",
                diagnostics,
            )

    def test_plugin_validate_rejects_unknown_interface_fields(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _append_manifest(
                repo_root / "zircon_plugins/native_dynamic_fixture/plugin.toml",
                [
                    "[[provides_interfaces]]",
                    'id = "native_dynamic_fixture.runtime"',
                    'sidecar = "drift"',
                    "",
                    "[[provides_interfaces.methods]]",
                    'name = "tick"',
                    "method_slot = 0",
                    'sidecar = "drift"',
                    'parameters = [{ name = "payload", value_kind = "bytes", sidecar = "drift", type_ref = { value_kind = "bytes", type_name = "Payload", sidecar = "drift" } }]',
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            diagnostics = report["diagnostics"]
            for diagnostic in (
                "plugin native_dynamic_fixture provides_interfaces[0].sidecar "
                "is not a known provided interface field",
                "plugin native_dynamic_fixture provides_interfaces[0].methods[0].sidecar "
                "is not a known provided interface method field",
                "plugin native_dynamic_fixture provides_interfaces[0].methods[0]."
                "parameters[0].sidecar is not a known interface method parameter field",
                "plugin native_dynamic_fixture provides_interfaces[0].methods[0]."
                "parameters[0].type_ref.sidecar is not a known interface method type_ref field",
            ):
                self.assertIn(diagnostic, diagnostics)

    def test_plugin_validate_rejects_duplicate_provided_interface_methods(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _append_manifest(
                repo_root / "zircon_plugins/native_dynamic_fixture/plugin.toml",
                [
                    "[[provides_interfaces]]",
                    'id = "native_dynamic_fixture.runtime"',
                    "",
                    "[[provides_interfaces.methods]]",
                    'name = "tick"',
                    "method_slot = 0",
                    'required_capabilities = ["runtime.plugin.native_dynamic_fixture", "runtime.plugin.native_dynamic_fixture"]',
                    "",
                    "[[provides_interfaces.methods]]",
                    'name = "tick"',
                    "method_slot = 0",
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            diagnostics = report["diagnostics"]
            self.assertIn(
                "plugin native_dynamic_fixture provides_interfaces[0].methods[1].name "
                "tick duplicates provided interface method methods[0]",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture provides_interfaces[0].methods[1].method_slot "
                "0 duplicates provided interface method_slot methods[0]",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture provides_interfaces[0].methods[0].required_capabilities[1] "
                "runtime.plugin.native_dynamic_fixture duplicates required capability "
                "required_capabilities[0]",
                diagnostics,
            )

    def test_plugin_validate_rejects_empty_interface_method_required_capabilities(
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
                    "[[provides_interfaces]]",
                    'id = "native_dynamic_fixture.runtime"',
                    "",
                    "[[provides_interfaces.methods]]",
                    'name = "tick"',
                    "method_slot = 0",
                    "parameters = []",
                    "required_capabilities = []",
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            diagnostics = report["diagnostics"]
            self.assertIn(
                "plugin native_dynamic_fixture provides_interfaces[0].methods[0]."
                "required_capabilities must not be empty when declared",
                diagnostics,
            )

    def test_plugin_validate_rejects_duplicate_interface_method_parameters(
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
                    "[[provides_interfaces]]",
                    'id = "native_dynamic_fixture.runtime"',
                    "",
                    "[[provides_interfaces.methods]]",
                    'name = "tick"',
                    "method_slot = 0",
                    'parameters = [{ name = "payload", value_kind = "bytes" }, '
                    '{ name = "payload", value_kind = "string" }]',
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            diagnostics = report["diagnostics"]
            self.assertIn(
                "plugin native_dynamic_fixture provides_interfaces[0].methods[0]."
                "parameters[1].name payload duplicates interface method parameter "
                "parameters[0]",
                diagnostics,
            )

    def test_plugin_validate_rejects_malformed_interface_method_documentation(
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
                    "[[provides_interfaces]]",
                    'id = "native_dynamic_fixture.runtime"',
                    "",
                    "[[provides_interfaces.methods]]",
                    'name = "tick"',
                    "method_slot = 0",
                    'documentation = " method docs "',
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            diagnostics = report["diagnostics"]
            self.assertIn(
                "plugin native_dynamic_fixture provides_interfaces[0].methods[0]."
                "documentation must be a non-empty trimmed string",
                diagnostics,
            )

    def test_plugin_validate_rejects_malformed_provided_interface_signature(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_complete_native_dynamic_fixture_manifest(repo_root, crate_name)
            _append_manifest(
                repo_root / "zircon_plugins/native_dynamic_fixture/plugin.toml",
                [
                    "[[provides_interfaces]]",
                    'id = "native_dynamic_fixture.runtime"',
                    "",
                    "[[provides_interfaces.methods]]",
                    'name = "tick"',
                    "method_slot = 0",
                    'return_value_kind = "vector"',
                    'parameters = [{ name = "payload", value_kind = "Vec3", type_ref = { value_kind = "BadKind", type_name = "" } }, { name = "context", value_kind = "string", type_ref = "bad" }, { name = "missing_kind" }]',
                ],
            )

            exit_code, report = _run_plugin_validate(repo_root)

            self.assertEqual(exit_code, 2)
            diagnostics = report["diagnostics"]
            self.assertIn(
                "plugin native_dynamic_fixture provides_interfaces[0].methods[0].return_value_kind "
                "vector is unsupported; expected one of null, bool, int, float, string, bytes, host_handle",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture provides_interfaces[0].methods[0].parameters[0].value_kind "
                "Vec3 is unsupported; expected one of null, bool, int, float, string, bytes, host_handle",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture provides_interfaces[0].methods[0].parameters[0].type_ref.value_kind "
                "BadKind is unsupported; expected one of null, bool, int, float, string, bytes, host_handle",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture provides_interfaces[0].methods[0].parameters[0].type_ref.type_name "
                "must be a non-empty trimmed string",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture provides_interfaces[0].methods[0].parameters[1].type_ref "
                "must be a table",
                diagnostics,
            )
            self.assertIn(
                "plugin native_dynamic_fixture provides_interfaces[0].methods[0].parameters[2].value_kind "
                "is required",
                diagnostics,
            )


if __name__ == "__main__":
    unittest.main()
