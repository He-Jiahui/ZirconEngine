from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.tests.export_test_support import _run_stage_quiet
from tools.zircon_export.tests.native_dynamic_test_support import (
    _export_args,
    _write_native_dynamic_cdylib_workspace,
    _write_native_dynamic_fake_cargo_build_script,
    _write_validate_report_with_native_dynamic_exports,
    _write_windows_native_dynamic_package_fixture_at,
)


class NativeBuildPathResolveErrorsTests(unittest.TestCase):
    def test_native_dynamic_build_plan_uses_resolved_workspace_manifest_in_command(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            crate_name = "zircon_plugin_animation_native"
            _write_windows_native_dynamic_package_fixture_at(
                repo_root,
                Path("animation"),
                package_id="animation",
                module_crate_names=[crate_name],
            )
            _write_native_dynamic_cdylib_workspace(
                repo_root,
                Path("animation") / "native",
                crate_name=crate_name,
            )
            _write_validate_report_with_native_dynamic_exports(
                out,
                profile="windows-release",
                target_platform="windows-x86_64",
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)
            workspace_manifest = repo_root / "zircon_plugins" / "Cargo.toml"
            original_resolve = Path.resolve
            workspace_manifest_resolve_count = 0

            def resolve_or_fail_second_workspace(
                path: Path,
                *args: object,
                **kwargs: object,
            ) -> Path:
                nonlocal workspace_manifest_resolve_count
                if Path(path) == workspace_manifest:
                    workspace_manifest_resolve_count += 1
                    if workspace_manifest_resolve_count > 1:
                        raise OSError("simulated redundant workspace manifest resolve failure")
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail_second_workspace):
                exit_code = _run_stage_quiet(args)

            report = json.loads(
                (out / "stages" / "native_dynamic" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            native_build_plan = report["native_build_plan"]
            package_plan = native_build_plan["packages"][0]
            command = package_plan["command"]
            self.assertEqual(exit_code, 0)
            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertFalse(native_build_plan["fatal"], native_build_plan)
            self.assertEqual(workspace_manifest_resolve_count, 1)
            self.assertEqual(
                command[command.index("--manifest-path") + 1],
                str(workspace_manifest),
            )

    def test_native_dynamic_build_rejects_expected_artifact_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            crate_name = "zircon_plugin_animation_native"
            _write_windows_native_dynamic_package_fixture_at(
                repo_root,
                Path("animation"),
                package_id="animation",
                module_crate_names=[crate_name],
                write_native_artifact=False,
            )
            _write_native_dynamic_cdylib_workspace(
                repo_root,
                Path("animation") / "native",
                crate_name=crate_name,
            )
            _write_native_dynamic_fake_cargo_build_script(repo_root, crate_name)
            _write_validate_report_with_native_dynamic_exports(
                out,
                profile="windows-release",
                target_platform="windows-x86_64",
                build_mode="Release",
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)
            args.cargo = sys.executable
            args.no_locked = True
            args.native_dynamic_build = True
            failing_artifact = (
                out
                / "stages"
                / "native_dynamic"
                / "target"
                / "release"
                / "zircon_plugin_animation_native.dll"
            )
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if Path(path) == failing_artifact:
                    raise OSError("simulated native artifact resolve failure")
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                exit_code = _run_stage_quiet(args)

            report = json.loads(
                (out / "stages" / "native_dynamic" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            native_build_execution = report["native_build_execution"]
            diagnostics = "\n".join(native_build_execution["diagnostics"])
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report)
            self.assertTrue(native_build_execution["fatal"], native_build_execution)
            self.assertEqual(native_build_execution["package_count"], 1)
            self.assertIn("expected artifact", diagnostics)
            self.assertIn("could not be resolved", diagnostics)
            self.assertIn("simulated native artifact resolve failure", diagnostics)
            self.assertTrue(report["payload_cleaned"])
            self.assertEqual(report["cleanup_reason"], "fatal_diagnostics")

    def test_native_dynamic_build_plan_rejects_target_dir_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            target_dir = root / "native-target"
            crate_name = "zircon_plugin_animation_native"
            _write_windows_native_dynamic_package_fixture_at(
                repo_root,
                Path("animation"),
                package_id="animation",
                module_crate_names=[crate_name],
            )
            _write_native_dynamic_cdylib_workspace(
                repo_root,
                Path("animation") / "native",
                crate_name=crate_name,
            )
            _write_validate_report_with_native_dynamic_exports(
                out,
                profile="windows-release",
                target_platform="windows-x86_64",
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)
            args.target_dir = str(target_dir)
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if Path(path) == target_dir:
                    raise OSError("simulated native target directory resolve failure")
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                exit_code = _run_stage_quiet(args)

            report = json.loads(
                (out / "stages" / "native_dynamic" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            native_build_plan = report["native_build_plan"]
            diagnostics = "\n".join(native_build_plan["diagnostics"])
            self.assertEqual(exit_code, 0)
            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertTrue(native_build_plan["fatal"], native_build_plan)
            self.assertEqual(native_build_plan["package_count"], 0)
            self.assertEqual(native_build_plan["target_dir"], str(out / "stages" / "native_dynamic" / "target"))
            self.assertIn("native dynamic build target directory", diagnostics)
            self.assertIn("could not be resolved", diagnostics)
            self.assertIn("simulated native target directory resolve failure", diagnostics)

    def test_native_dynamic_build_plan_rejects_workspace_manifest_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            crate_name = "zircon_plugin_animation_native"
            _write_windows_native_dynamic_package_fixture_at(
                repo_root,
                Path("animation"),
                package_id="animation",
                module_crate_names=[crate_name],
            )
            _write_native_dynamic_cdylib_workspace(
                repo_root,
                Path("animation") / "native",
                crate_name=crate_name,
            )
            _write_validate_report_with_native_dynamic_exports(
                out,
                profile="windows-release",
                target_platform="windows-x86_64",
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)
            workspace_manifest = repo_root / "zircon_plugins" / "Cargo.toml"
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if Path(path) == workspace_manifest:
                    raise OSError("simulated native workspace manifest resolve failure")
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                exit_code = _run_stage_quiet(args)

            report = json.loads(
                (out / "stages" / "native_dynamic" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            native_build_plan = report["native_build_plan"]
            diagnostics = "\n".join(native_build_plan["diagnostics"])
            self.assertEqual(exit_code, 0)
            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertTrue(native_build_plan["fatal"], native_build_plan)
            self.assertEqual(native_build_plan["package_count"], 0)
            self.assertEqual(native_build_plan["workspace_manifest"], str(workspace_manifest))
            self.assertIn("native dynamic plugin workspace manifest", diagnostics)
            self.assertIn("could not be resolved", diagnostics)
            self.assertIn("simulated native workspace manifest resolve failure", diagnostics)

    def test_native_dynamic_build_plan_rejects_member_manifest_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            crate_name = "zircon_plugin_animation_native"
            member = Path("animation") / "native"
            _write_windows_native_dynamic_package_fixture_at(
                repo_root,
                Path("animation"),
                package_id="animation",
                module_crate_names=[crate_name],
            )
            _write_native_dynamic_cdylib_workspace(
                repo_root,
                member,
                crate_name=crate_name,
            )
            _write_validate_report_with_native_dynamic_exports(
                out,
                profile="windows-release",
                target_platform="windows-x86_64",
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)
            failing_manifest = (
                repo_root / "zircon_plugins" / member / "Cargo.toml"
            )
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if Path(path) == failing_manifest:
                    raise OSError("simulated native member manifest resolve failure")
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json.loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            native_build_plan = report["native_build_plan"]
            diagnostics = "\n".join(native_build_plan["diagnostics"])
            self.assertEqual(exit_code, 0)
            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertTrue(native_build_plan["fatal"], native_build_plan)
            self.assertEqual(native_build_plan["package_count"], 0)
            self.assertIn("native dynamic workspace member animation/native manifest", diagnostics)
            self.assertIn("could not be resolved", diagnostics)
            self.assertIn("simulated native member manifest resolve failure", diagnostics)
            self.assertIsNotNone(report["loader_manifest"])
            self.assertFalse(report["payload_cleaned"])


if __name__ == "__main__":
    unittest.main()
