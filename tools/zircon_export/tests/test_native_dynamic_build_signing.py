from __future__ import annotations

import hashlib
import json
import shutil
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
    _write_native_dynamic_fake_notarize_script,
    _write_native_dynamic_fake_sign_script,
    _write_validate_report_with_native_dynamic_exports,
    _write_windows_native_dynamic_package_fixture_at,
)


class NativeDynamicBuildAndSigningTests(unittest.TestCase):
    def test_native_dynamic_stage_reports_native_cdylib_build_plan(self) -> None:
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
                build_mode="Release",
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)
            args.offline = True

            exit_code = _run_stage_quiet(args)

            report = json.loads(
                (out / "stages" / "native_dynamic" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            native_build_plan = report["native_build_plan"]
            native_build_execution = report["native_build_execution"]
            package_plan = native_build_plan["packages"][0]
            command = package_plan["command"]
            self.assertEqual(exit_code, 0)
            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertFalse(native_build_execution["enabled"])
            self.assertFalse(native_build_execution["fatal"])
            self.assertFalse(native_build_execution["skipped"])
            self.assertEqual(native_build_plan["package_count"], 1)
            self.assertEqual(native_build_plan["cargo_profile"], "release")
            self.assertEqual(package_plan["package_id"], "animation")
            self.assertEqual(package_plan["crate_name"], crate_name)
            self.assertEqual(
                package_plan["expected_loadable_artifact"],
                str(
                    out
                    / "stages"
                    / "native_dynamic"
                    / "target"
                    / "release"
                    / "zircon_plugin_animation_native.dll"
                ),
            )
            self.assertEqual(command[0:2], ["cargo", "build"])
            self.assertIn("--manifest-path", command)
            self.assertEqual(
                command[command.index("--manifest-path") + 1],
                str((repo_root / "zircon_plugins" / "Cargo.toml").resolve()),
            )
            self.assertIn("-p", command)
            self.assertEqual(command[command.index("-p") + 1], crate_name)
            self.assertIn("--target-dir", command)
            self.assertEqual(
                command[command.index("--target-dir") + 1],
                str((out / "stages" / "native_dynamic" / "target").resolve()),
            )
            self.assertIn("--locked", command)
            self.assertIn("--release", command)
            self.assertIn("--offline", command)

    def test_native_dynamic_build_plan_records_cargo_features(self) -> None:
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
            args.native_dynamic_build_feature = [
                "v3_fixture_diagnostics",
                " v3_fixture_diagnostics ",
                "",
            ]

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
            self.assertEqual(native_build_plan["build_features"], ["v3_fixture_diagnostics"])
            self.assertEqual(package_plan["features"], ["v3_fixture_diagnostics"])
            self.assertIn("--features", command)
            self.assertEqual(command[command.index("--features") + 1], "v3_fixture_diagnostics")

    def test_native_dynamic_build_plan_respects_target_dir_override(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            target_dir = (root / "custom-native-target").resolve()
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
                build_mode="Release",
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)
            args.target_dir = str(target_dir)

            exit_code = _run_stage_quiet(args)

            report = json.loads(
                (out / "stages" / "native_dynamic" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            native_build_plan = report["native_build_plan"]
            package_plan = native_build_plan["packages"][0]
            command = package_plan["command"]
            expected_loadable = (
                target_dir / "release" / "zircon_plugin_animation_native.dll"
            )
            self.assertEqual(exit_code, 0)
            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertEqual(native_build_plan["target_dir"], str(target_dir))
            self.assertEqual(package_plan["target_dir"], str(target_dir))
            self.assertEqual(
                package_plan["expected_loadable_artifact"],
                str(expected_loadable),
            )
            self.assertIn("--target-dir", command)
            self.assertEqual(
                command[command.index("--target-dir") + 1],
                str(target_dir),
            )

    def test_native_dynamic_build_plan_rejects_workspace_manifest_directory(self) -> None:
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
            workspace_manifest = repo_root / "zircon_plugins" / "Cargo.toml"
            workspace_manifest.mkdir(parents=True)
            _write_validate_report_with_native_dynamic_exports(
                out,
                profile="windows-release",
                target_platform="windows-x86_64",
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json.loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            native_build_plan = report["native_build_plan"]
            diagnostics = "\n".join(report["diagnostics"])
            self.assertEqual(exit_code, 0)
            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertTrue(native_build_plan["fatal"])
            self.assertEqual(native_build_plan["package_count"], 0)
            self.assertIn("TOML file", "\n".join(native_build_plan["diagnostics"]))
            self.assertIn("zircon_plugins", "\n".join(native_build_plan["diagnostics"]))
            self.assertIn("Cargo.toml", "\n".join(native_build_plan["diagnostics"]))
            self.assertIn("is not a file", "\n".join(native_build_plan["diagnostics"]))
            self.assertNotIn("is not a file", diagnostics)
            self.assertIsNotNone(report["loader_manifest"])
            self.assertFalse(report["payload_cleaned"])
            self.assertTrue((stage_dir / "plugins" / "native_plugins.toml").exists())

    def test_native_dynamic_build_plan_rejects_crate_manifest_directory(self) -> None:
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
            crate_manifest = (
                repo_root / "zircon_plugins" / "animation" / "native" / "Cargo.toml"
            )
            crate_manifest.unlink()
            crate_manifest.mkdir()
            _write_validate_report_with_native_dynamic_exports(
                out,
                profile="windows-release",
                target_platform="windows-x86_64",
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json.loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            native_build_plan = report["native_build_plan"]
            diagnostics = "\n".join(report["diagnostics"])
            self.assertEqual(exit_code, 0)
            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertTrue(native_build_plan["fatal"])
            self.assertEqual(native_build_plan["package_count"], 0)
            self.assertIn("TOML file", "\n".join(native_build_plan["diagnostics"]))
            self.assertIn("animation", "\n".join(native_build_plan["diagnostics"]))
            self.assertIn("native", "\n".join(native_build_plan["diagnostics"]))
            self.assertIn("Cargo.toml", "\n".join(native_build_plan["diagnostics"]))
            self.assertIn("is not a file", "\n".join(native_build_plan["diagnostics"]))
            self.assertNotIn("is not a file", diagnostics)
            self.assertIsNotNone(report["loader_manifest"])
            self.assertFalse(report["payload_cleaned"])
            self.assertTrue((stage_dir / "plugins" / "native_plugins.toml").exists())

    def test_native_dynamic_build_executes_plan_and_stages_cdylib(self) -> None:
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

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            built_artifact = (
                stage_dir
                / "plugins"
                / "animation"
                / "native"
                / "zircon_plugin_animation_native.dll"
            )
            report = json.loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            package_report = (
                stage_dir
                / "plugins"
                / "animation"
                / "native_dynamic_package.toml"
            ).read_text(encoding="utf-8")
            execution = report["native_build_execution"]
            package_execution = execution["packages"][0]
            materialized_package = report["materialized_packages"][0]
            manifest_paths = [entry["path"] for entry in report["file_manifest"]]
            self.assertEqual(exit_code, 0)
            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertTrue(execution["enabled"])
            self.assertFalse(execution["fatal"], execution["diagnostics"])
            self.assertFalse(execution["skipped"])
            self.assertEqual(execution["package_count"], 1)
            self.assertEqual(package_execution["package_id"], "animation")
            self.assertEqual(package_execution["exit_code"], 0)
            self.assertEqual(package_execution["copied_loadable_artifact"], str(built_artifact))
            self.assertTrue(built_artifact.exists())
            self.assertEqual(
                materialized_package["loadable_artifacts"],
                ["plugins/animation/native/zircon_plugin_animation_native.dll"],
            )
            self.assertIn(
                "plugins/animation/native/zircon_plugin_animation_native.dll",
                manifest_paths,
            )
            self.assertIn(
                'path = "native/zircon_plugin_animation_native.dll"',
                package_report,
            )

    def test_native_dynamic_build_rejects_staged_cdylib_copy_error(self) -> None:
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
            source_artifact = (
                out
                / "stages"
                / "native_dynamic"
                / "target"
                / "release"
                / "zircon_plugin_animation_native.dll"
            ).resolve()
            original_copy2 = shutil.copy2

            def copy_or_fail(source: Path, destination: Path) -> None:
                if Path(source).resolve() == source_artifact:
                    raise OSError("simulated native build artifact copy failure")
                original_copy2(source, destination)

            with mock.patch(
                "tools.zircon_export.native_build.shutil.copy2",
                side_effect=copy_or_fail,
            ):
                exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json.loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            execution = report["native_build_execution"]
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(execution["fatal"], execution["diagnostics"])
            self.assertTrue(report["payload_cleaned"])
            self.assertEqual(report["cleanup_reason"], "fatal_diagnostics")
            self.assertFalse((stage_dir / "plugins" / "animation").exists())
            self.assertTrue(
                any(
                    "NativeDynamic native build for package animation artifact"
                    in diagnostic
                    and "could not be copied" in diagnostic
                    and "simulated native build artifact copy failure" in diagnostic
                    for diagnostic in execution["diagnostics"]
                ),
                execution["diagnostics"],
            )

    def test_native_dynamic_signs_loadable_artifact_before_manifest_hash(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            _write_windows_native_dynamic_package_fixture_at(
                repo_root,
                Path("animation"),
                package_id="animation",
            )
            signer = _write_native_dynamic_fake_sign_script(repo_root)
            _write_validate_report_with_native_dynamic_exports(
                out,
                profile="windows-release",
                target_platform="windows-x86_64",
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)
            args.native_dynamic_sign_command = sys.executable
            args.native_dynamic_sign_arg = [
                str(signer),
                "{artifact}",
                "{package_id}",
                "{target_platform}",
            ]

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            artifact = (
                stage_dir
                / "plugins"
                / "animation"
                / "native"
                / "zircon_plugin_animation.dll"
            )
            report = json.loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            package_report = (
                stage_dir
                / "plugins"
                / "animation"
                / "native_dynamic_package.toml"
            ).read_text(encoding="utf-8")
            signing = report["native_signing"]
            package_signing = signing["packages"][0]
            artifact_signing = package_signing["artifacts"][0]
            artifact_hash = hashlib.sha256(artifact.read_bytes()).hexdigest()
            file_manifest_entry = next(
                entry
                for entry in report["file_manifest"]
                if entry["path"] == "plugins/animation/native/zircon_plugin_animation.dll"
            )
            self.assertEqual(exit_code, 0)
            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertTrue(signing["enabled"])
            self.assertFalse(signing["fatal"], signing["diagnostics"])
            self.assertEqual(signing["package_count"], 1)
            self.assertEqual(package_signing["package_id"], "animation")
            self.assertEqual(package_signing["artifact_count"], 1)
            self.assertEqual(artifact_signing["exit_code"], 0)
            self.assertEqual(artifact_signing["after_sha256"], artifact_hash)
            self.assertNotEqual(
                artifact_signing["before_sha256"],
                artifact_signing["after_sha256"],
            )
            self.assertEqual(file_manifest_entry["sha256"], artifact_hash)
            self.assertIn(artifact_hash, package_report)
            self.assertIn(
                "signed:animation:windows-x86_64",
                artifact.read_text(encoding="utf-8"),
            )

    def test_native_dynamic_signing_profile_records_platform_gate(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            _write_windows_native_dynamic_package_fixture_at(
                repo_root,
                Path("animation"),
                package_id="animation",
            )
            signer = _write_native_dynamic_fake_sign_script(repo_root)
            _write_validate_report_with_native_dynamic_exports(
                out,
                profile="windows-release",
                target_platform="windows-x86_64",
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)
            args.native_dynamic_sign_command = sys.executable
            args.native_dynamic_sign_arg = [
                str(signer),
                "{artifact}",
                "{package_id}",
                "{target_platform}",
                "{signing_profile}",
            ]
            args.native_dynamic_sign_profile = "windows-store"
            args.native_dynamic_sign_platform = "windows"

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            artifact = (
                stage_dir
                / "plugins"
                / "animation"
                / "native"
                / "zircon_plugin_animation.dll"
            )
            report = json.loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            signing = report["native_signing"]
            artifact_signing = signing["packages"][0]["artifacts"][0]
            self.assertEqual(exit_code, 0)
            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertEqual(signing["profile"], "windows-store")
            self.assertEqual(signing["target_platform"], "windows-x86_64")
            self.assertEqual(signing["allowed_platforms"], ["windows"])
            self.assertTrue(signing["platform_allowed"])
            self.assertIn("windows-store", artifact_signing["command"])
            self.assertIn(
                "signed:animation:windows-x86_64:windows-store",
                artifact.read_text(encoding="utf-8"),
            )

    def test_native_dynamic_signing_profile_rejects_platform_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            _write_windows_native_dynamic_package_fixture_at(
                repo_root,
                Path("animation"),
                package_id="animation",
            )
            signer = _write_native_dynamic_fake_sign_script(repo_root)
            _write_validate_report_with_native_dynamic_exports(
                out,
                profile="windows-release",
                target_platform="windows-x86_64",
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)
            args.native_dynamic_sign_command = sys.executable
            args.native_dynamic_sign_arg = [str(signer), "{artifact}"]
            args.native_dynamic_sign_profile = "macos-dev-id"
            args.native_dynamic_sign_platform = "macos"

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json.loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            signing = report["native_signing"]
            diagnostics = "\n".join(report["diagnostics"])
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertTrue(signing["enabled"])
            self.assertTrue(signing["fatal"])
            self.assertEqual(signing["profile"], "macos-dev-id")
            self.assertEqual(signing["allowed_platforms"], ["macos"])
            self.assertFalse(signing["platform_allowed"])
            self.assertIn("does not allow target platform windows-x86_64", diagnostics)
            self.assertEqual(signing["packages"], [])
            self.assertTrue(report["payload_cleaned"])
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())

    def test_native_dynamic_notarization_runs_after_signing_before_manifest_hash(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            _write_windows_native_dynamic_package_fixture_at(
                repo_root,
                Path("animation"),
                package_id="animation",
            )
            signer = _write_native_dynamic_fake_sign_script(repo_root)
            notarizer = _write_native_dynamic_fake_notarize_script(repo_root)
            _write_validate_report_with_native_dynamic_exports(
                out,
                profile="windows-release",
                target_platform="windows-x86_64",
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)
            args.native_dynamic_sign_command = sys.executable
            args.native_dynamic_sign_arg = [
                str(signer),
                "{artifact}",
                "{package_id}",
                "{target_platform}",
                "{signing_profile}",
            ]
            args.native_dynamic_sign_profile = "windows-store"
            args.native_dynamic_notarize_command = sys.executable
            args.native_dynamic_notarize_arg = [
                str(notarizer),
                "{artifact}",
                "{package_id}",
                "{target_platform}",
                "{signing_profile}",
                "{notarization_profile}",
            ]
            args.native_dynamic_notarize_profile = "windows-attestation"
            args.native_dynamic_notarize_platform = "windows"

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            artifact = (
                stage_dir
                / "plugins"
                / "animation"
                / "native"
                / "zircon_plugin_animation.dll"
            )
            report = json.loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            package_report = (
                stage_dir
                / "plugins"
                / "animation"
                / "native_dynamic_package.toml"
            ).read_text(encoding="utf-8")
            signing = report["native_signing"]
            notarization = report["native_notarization"]
            artifact_signing = signing["packages"][0]["artifacts"][0]
            artifact_notarization = notarization["packages"][0]["artifacts"][0]
            artifact_hash = hashlib.sha256(artifact.read_bytes()).hexdigest()
            file_manifest_entry = next(
                entry
                for entry in report["file_manifest"]
                if entry["path"] == "plugins/animation/native/zircon_plugin_animation.dll"
            )
            artifact_text = artifact.read_text(encoding="utf-8")
            self.assertEqual(exit_code, 0)
            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertTrue(notarization["enabled"])
            self.assertFalse(notarization["fatal"], notarization["diagnostics"])
            self.assertEqual(notarization["profile"], "windows-attestation")
            self.assertEqual(notarization["allowed_platforms"], ["windows"])
            self.assertEqual(notarization["package_count"], 1)
            self.assertEqual(artifact_notarization["before_sha256"], artifact_signing["after_sha256"])
            self.assertEqual(artifact_notarization["after_sha256"], artifact_hash)
            self.assertNotEqual(artifact_signing["after_sha256"], artifact_hash)
            self.assertEqual(file_manifest_entry["sha256"], artifact_hash)
            self.assertIn(artifact_hash, package_report)
            self.assertIn(
                "signed:animation:windows-x86_64:windows-store",
                artifact_text,
            )
            self.assertIn(
                "notarized:animation:windows-x86_64:windows-store:windows-attestation",
                artifact_text,
            )

    def test_native_dynamic_notarization_profile_rejects_platform_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            _write_windows_native_dynamic_package_fixture_at(
                repo_root,
                Path("animation"),
                package_id="animation",
            )
            notarizer = _write_native_dynamic_fake_notarize_script(repo_root)
            _write_validate_report_with_native_dynamic_exports(
                out,
                profile="windows-release",
                target_platform="windows-x86_64",
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)
            args.native_dynamic_notarize_command = sys.executable
            args.native_dynamic_notarize_arg = [str(notarizer), "{artifact}"]
            args.native_dynamic_notarize_profile = "macos-notary"
            args.native_dynamic_notarize_platform = "macos"

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json.loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            notarization = report["native_notarization"]
            diagnostics = "\n".join(report["diagnostics"])
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertTrue(notarization["enabled"])
            self.assertTrue(notarization["fatal"])
            self.assertEqual(notarization["profile"], "macos-notary")
            self.assertEqual(notarization["allowed_platforms"], ["macos"])
            self.assertFalse(notarization["platform_allowed"])
            self.assertIn("does not allow target platform windows-x86_64", diagnostics)
            self.assertEqual(notarization["packages"], [])
            self.assertTrue(report["payload_cleaned"])
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())

    def test_native_dynamic_signing_failure_cleans_staged_payload(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            _write_windows_native_dynamic_package_fixture_at(
                repo_root,
                Path("animation"),
                package_id="animation",
            )
            signer = _write_native_dynamic_fake_sign_script(repo_root, exit_code=7)
            _write_validate_report_with_native_dynamic_exports(
                out,
                profile="windows-release",
                target_platform="windows-x86_64",
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)
            args.native_dynamic_sign_command = sys.executable
            args.native_dynamic_sign_arg = [str(signer), "{artifact}"]

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json.loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            signing = report["native_signing"]
            artifact_signing = signing["packages"][0]["artifacts"][0]
            diagnostics = "\n".join(report["diagnostics"])
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertTrue(signing["enabled"])
            self.assertTrue(signing["fatal"])
            self.assertEqual(artifact_signing["exit_code"], 7)
            self.assertIn("exited with code 7", diagnostics)
            self.assertIsNone(report["loader_manifest"])
            self.assertTrue(report["payload_cleaned"])
            self.assertEqual(report["cleanup_reason"], "fatal_diagnostics")
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())
            self.assertEqual(report["materialized_packages"], [])
