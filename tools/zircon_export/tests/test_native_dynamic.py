from __future__ import annotations

import argparse
import contextlib
import hashlib
import io
import json
import sys
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.cli import run_stage
from tools.zircon_export.native_dynamic import native_dynamic_stage_payload_summary


REPO_ROOT = Path(__file__).resolve().parents[3]


class NativeDynamicArtifactTests(unittest.TestCase):
    def test_native_dynamic_stage_rejects_wrong_v3_descriptor_symbol(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            _write_windows_native_dynamic_package_fixture_at(
                repo_root,
                Path("animation"),
                package_id="animation",
            )
            _write_validate_report_with_native_dynamic_exports(
                out,
                profile="windows-release",
                target_platform="windows-x86_64",
                abi_overrides={
                    "descriptor_symbol": "zircon_native_plugin_descriptor_v2"
                },
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json.loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            diagnostics = "\n".join(report["diagnostics"])
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertIn(
                "abi.descriptor_symbol must be zircon_native_plugin_descriptor_v3",
                diagnostics,
            )
            self.assertIsNone(report["loader_manifest"])
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())

    def test_native_dynamic_stage_rejects_non_v3_abi_version(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            _write_windows_native_dynamic_package_fixture_at(
                repo_root,
                Path("animation"),
                package_id="animation",
            )
            _write_validate_report_with_native_dynamic_exports(
                out,
                profile="windows-release",
                target_platform="windows-x86_64",
                abi_overrides={"abi_version": 2},
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json.loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            diagnostics = "\n".join(report["diagnostics"])
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertIn("abi.abi_version must be 3", diagnostics)
            self.assertIsNone(report["loader_manifest"])
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())

    def test_native_dynamic_stage_rejects_duplicate_recursive_package_sources(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            _write_windows_native_dynamic_package_fixture_at(
                repo_root,
                Path("group_a") / "animation_runtime",
                package_id="animation",
            )
            _write_windows_native_dynamic_package_fixture_at(
                repo_root,
                Path("group_b") / "animation_runtime",
                package_id="animation",
            )
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
            diagnostics = "\n".join(report["diagnostics"])
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertIn("multiple source package manifests", diagnostics)
            self.assertIsNone(report["loader_manifest"])
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())

    def test_native_dynamic_stage_copies_macos_dsym_bundle(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            _write_macos_native_dynamic_package_fixture(repo_root)
            _write_validate_report_with_native_dynamic_exports(
                out,
                profile="macos-release",
                target_platform="macos-aarch64",
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.profile = "macos-release"
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json.loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            manifest_paths = [entry["path"] for entry in report["file_manifest"]]
            self.assertEqual(exit_code, 0)
            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertIn(
                "plugins/animation/native/libzircon_plugin_animation.dylib",
                manifest_paths,
            )
            self.assertIn(
                "plugins/animation/native/zircon_plugin_animation.dSYM/Contents/Resources/DWARF/zircon_plugin_animation",
                manifest_paths,
            )

    def test_native_dynamic_stage_reports_package_loadable_artifacts(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            _write_windows_native_dynamic_package_fixture_at(
                repo_root,
                Path("animation"),
                package_id="animation",
            )
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
            materialized_package = report["materialized_packages"][0]
            self.assertEqual(exit_code, 0)
            self.assertEqual(materialized_package["package_id"], "animation")
            self.assertEqual(materialized_package["loadable_artifact_count"], 1)
            self.assertEqual(
                materialized_package["loadable_artifacts"],
                ["plugins/animation/native/zircon_plugin_animation.dll"],
            )

    def test_native_dynamic_payload_summary_keeps_loadable_artifact_audit(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            _write_windows_native_dynamic_package_fixture_at(
                repo_root,
                Path("animation"),
                package_id="animation",
            )
            _write_validate_report_with_native_dynamic_exports(
                out,
                profile="windows-release",
                target_platform="windows-x86_64",
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)
            diagnostics: list[str] = []
            summary = native_dynamic_stage_payload_summary(
                out,
                "windows-release",
                out / "stages" / "native_dynamic" / "plugins",
                diagnostics,
            )

            self.assertEqual(exit_code, 0)
            self.assertEqual(diagnostics, [])
            self.assertIsNotNone(summary)
            package_summary = summary["materialized_packages"][0]
            self.assertEqual(package_summary["package_id"], "animation")
            self.assertEqual(package_summary["loadable_artifact_count"], 1)
            self.assertEqual(
                package_summary["loadable_artifacts"],
                ["plugins/animation/native/zircon_plugin_animation.dll"],
            )

    def test_native_dynamic_payload_summary_rejects_malformed_package_audit(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            _write_windows_native_dynamic_package_fixture_at(
                repo_root,
                Path("animation"),
                package_id="animation",
            )
            _write_validate_report_with_native_dynamic_exports(
                out,
                profile="windows-release",
                target_platform="windows-x86_64",
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)
            exit_code = _run_stage_quiet(args)
            report_path = out / "stages" / "native_dynamic" / "report.json"
            report = json.loads(report_path.read_text(encoding="utf-8"))
            del report["materialized_packages"][0]["loadable_artifact_count"]
            report_path.write_text(json.dumps(report, indent=2), encoding="utf-8")

            diagnostics: list[str] = []
            summary = native_dynamic_stage_payload_summary(
                out,
                "windows-release",
                out / "stages" / "native_dynamic" / "plugins",
                diagnostics,
            )

            self.assertEqual(exit_code, 0)
            self.assertIsNone(summary)
            self.assertIn(
                "NativeDynamic report materialized_packages are malformed",
                diagnostics,
            )

    def test_native_dynamic_payload_summary_rejects_loadable_artifact_not_in_manifest(
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
            _write_validate_report_with_native_dynamic_exports(
                out,
                profile="windows-release",
                target_platform="windows-x86_64",
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)
            exit_code = _run_stage_quiet(args)
            report_path = out / "stages" / "native_dynamic" / "report.json"
            report = json.loads(report_path.read_text(encoding="utf-8"))
            report["materialized_packages"][0]["loadable_artifacts"] = [
                "plugins/animation/native/missing_plugin.dll"
            ]
            report_path.write_text(json.dumps(report, indent=2), encoding="utf-8")

            diagnostics: list[str] = []
            summary = native_dynamic_stage_payload_summary(
                out,
                "windows-release",
                out / "stages" / "native_dynamic" / "plugins",
                diagnostics,
            )

            self.assertEqual(exit_code, 0)
            self.assertIsNone(summary)
            self.assertIn(
                "NativeDynamic report loadable_artifacts are not present in file_manifest",
                diagnostics,
            )

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
            package_plan = native_build_plan["packages"][0]
            command = package_plan["command"]
            self.assertEqual(exit_code, 0)
            self.assertFalse(report["fatal"], report["diagnostics"])
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
                "abi_v2_only",
                " abi_v2_only ",
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
            self.assertEqual(native_build_plan["build_features"], ["abi_v2_only"])
            self.assertEqual(package_plan["features"], ["abi_v2_only"])
            self.assertIn("--features", command)
            self.assertEqual(command[command.index("--features") + 1], "abi_v2_only")

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

    def test_native_dynamic_payload_summary_accepts_sanitized_package_directory(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            _write_windows_native_dynamic_package_fixture_at(
                repo_root,
                Path("animation.fx"),
                package_id="animation.fx",
            )
            _write_validate_report_with_native_dynamic_exports(
                out,
                profile="windows-release",
                target_platform="windows-x86_64",
                native_dynamic_packages=["animation.fx"],
                package_export_overrides={
                    "package_id": "animation.fx",
                    "directory": "animation_fx",
                    "path": "plugins/animation_fx",
                    "manifest": "plugins/animation_fx/plugin.toml",
                    "package_report": "plugins/animation_fx/native_dynamic_package.toml",
                },
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)
            diagnostics: list[str] = []
            summary = native_dynamic_stage_payload_summary(
                out,
                "windows-release",
                out / "stages" / "native_dynamic" / "plugins",
                diagnostics,
            )

            self.assertEqual(exit_code, 0)
            self.assertEqual(diagnostics, [])
            self.assertIsNotNone(summary)
            self.assertEqual(
                summary["materialized_packages"][0]["loadable_artifacts"],
                ["plugins/animation_fx/native/zircon_plugin_animation.dll"],
            )


def _run_stage_quiet(args: argparse.Namespace) -> int:
    with contextlib.redirect_stdout(io.StringIO()):
        return run_stage(args)


def _export_args(
    *,
    out: Path,
    stage: str,
    dry_run: bool,
) -> argparse.Namespace:
    return argparse.Namespace(
        profile="windows-release",
        project="zircon-project.toml",
        out=str(out),
        stage=stage,
        resume_from="validate",
        repo_root=str(REPO_ROOT),
        cargo="cargo",
        validator=None,
        packer=None,
        validate_report=None,
        native_plugin_root=None,
        asset_manifest=None,
        asset_filter=None,
        pack_file=None,
        previous_pack=None,
        delta_pack=None,
        host_executable=None,
        native_plugins_dir=None,
        template_dir=None,
        template_root=None,
        engine_version="0.1.0",
        target_platform=None,
        determinism_check=False,
        source_template_build=False,
        target_dir=None,
        offline=False,
        no_locked=False,
        native_dynamic_build=False,
        native_dynamic_build_feature=[],
        native_dynamic_sign_command=None,
        native_dynamic_sign_arg=[],
        native_dynamic_sign_profile=None,
        native_dynamic_sign_platform=[],
        native_dynamic_notarize_command=None,
        native_dynamic_notarize_arg=[],
        native_dynamic_notarize_profile=None,
        native_dynamic_notarize_platform=[],
        pretty=False,
        dry_run=dry_run,
    )


def _write_validate_report_with_native_dynamic_exports(
    out: Path,
    *,
    profile: str,
    target_platform: str,
    build_mode: str | None = None,
    abi_overrides: dict[str, object] | None = None,
    native_dynamic_packages: list[str] | None = None,
    package_export_overrides: dict[str, object] | None = None,
) -> None:
    report_dir = out / "stages" / "validate"
    report_dir.mkdir(parents=True, exist_ok=True)
    report_dir.joinpath("report.json").write_text(
        json.dumps(
            {
                "stage": "Validate",
                "profile": profile,
                "fatal": False,
                "diagnostics": [],
                "profile_summary": {
                    "strategies": ["native_dynamic"],
                    "target_platform": target_platform,
                    **({"build_mode": build_mode} if build_mode else {}),
                },
                "plan_summary": {
                    "native_dynamic_packages": native_dynamic_packages or ["animation"],
                    "native_dynamic_package_exports": [
                        _native_dynamic_package_export(
                            abi_overrides,
                            package_export_overrides,
                        )
                    ],
                },
            },
            indent=2,
        ),
        encoding="utf-8",
    )


def _native_dynamic_package_export(
    abi_overrides: dict[str, object] | None = None,
    package_export_overrides: dict[str, object] | None = None,
) -> dict[str, object]:
    abi: dict[str, object] = {
        "abi_version": 3,
        "descriptor_symbol": "zircon_native_plugin_descriptor_v3",
        "descriptor_contract": "NativePluginAbiV3",
        "runtime_entry_source": "NativePluginAbiV3.runtime_entry_name",
        "editor_entry_source": "NativePluginAbiV3.editor_entry_name",
        "host_function_table": "NativePluginHostFunctionTableV3",
        "entry_report_contract": "NativePluginEntryReportV3",
        "behavior_contract": "NativePluginBehaviorV3",
        "state_snapshot_contract": "NativePluginBehaviorV3.save_state/restore_state",
        "bridge_method_table": "NativePluginBridgeMethodTableV3",
    }
    if abi_overrides:
        abi.update(abi_overrides)
    package_export: dict[str, object] = {
        "package_id": "animation",
        "directory": "animation",
        "path": "plugins/animation",
        "manifest": "plugins/animation/plugin.toml",
        "package_report": "plugins/animation/native_dynamic_package.toml",
        "abi": abi,
    }
    if package_export_overrides:
        package_export.update(package_export_overrides)
    return package_export


def _write_macos_native_dynamic_package_fixture(repo_root: Path) -> None:
    package = repo_root / "zircon_plugins" / "animation"
    native_dir = package / "native"
    dsym_dwarf_dir = (
        native_dir
        / "zircon_plugin_animation.dSYM"
        / "Contents"
        / "Resources"
        / "DWARF"
    )
    native_dir.mkdir(parents=True)
    dsym_dwarf_dir.mkdir(parents=True)
    (package / "plugin.toml").write_text(
        "\n".join(
            [
                'id = "animation"',
                'name = "Animation"',
                'default_packaging = ["native_dynamic"]',
            ]
        ),
        encoding="utf-8",
    )
    (native_dir / "libzircon_plugin_animation.dylib").write_text(
        "native dynamic placeholder",
        encoding="utf-8",
    )
    (dsym_dwarf_dir / "zircon_plugin_animation").write_text(
        "debug symbols placeholder",
        encoding="utf-8",
    )


def _write_windows_native_dynamic_package_fixture_at(
    repo_root: Path,
    relative_package_path: Path,
    *,
    package_id: str,
    module_crate_names: list[str] | None = None,
    write_native_artifact: bool = True,
) -> None:
    package = repo_root / "zircon_plugins" / relative_package_path
    native_dir = package / "native"
    package.mkdir(parents=True, exist_ok=True)
    if write_native_artifact:
        native_dir.mkdir(parents=True, exist_ok=True)
    plugin_manifest_lines = [
        f'id = "{package_id}"',
        'name = "Animation"',
        'default_packaging = ["native_dynamic"]',
    ]
    for crate_name in module_crate_names or []:
        plugin_manifest_lines.extend(
            [
                "",
                "[[modules]]",
                f'name = "{package_id}.runtime"',
                'kind = "runtime"',
                f'crate_name = "{crate_name}"',
            ]
        )
    (package / "plugin.toml").write_text(
        "\n".join(plugin_manifest_lines),
        encoding="utf-8",
    )
    if write_native_artifact:
        (native_dir / "zircon_plugin_animation.dll").write_text(
            "native dynamic placeholder",
            encoding="utf-8",
        )


def _write_native_dynamic_cdylib_workspace(
    repo_root: Path,
    member: Path,
    *,
    crate_name: str,
) -> None:
    plugins_root = repo_root / "zircon_plugins"
    plugins_root.mkdir(parents=True, exist_ok=True)
    (plugins_root / "Cargo.toml").write_text(
        "\n".join(
            [
                "[workspace]",
                f'members = ["{member.as_posix()}"]',
                'resolver = "2"',
            ]
        ),
        encoding="utf-8",
    )
    crate_dir = plugins_root / member
    crate_dir.mkdir(parents=True, exist_ok=True)
    (crate_dir / "Cargo.toml").write_text(
        "\n".join(
            [
                "[package]",
                f'name = "{crate_name}"',
                'version = "0.1.0"',
                'edition = "2021"',
                "",
                "[lib]",
                'crate-type = ["cdylib"]',
            ]
        ),
        encoding="utf-8",
    )


def _write_native_dynamic_fake_cargo_build_script(repo_root: Path, crate_name: str) -> None:
    (repo_root / "build").write_text(
        "\n".join(
            [
                "from pathlib import Path",
                "import sys",
                "",
                "target_dir = Path(sys.argv[sys.argv.index('--target-dir') + 1])",
                "profile = 'release' if '--release' in sys.argv else 'debug'",
                f"artifact = target_dir / profile / '{crate_name}.dll'",
                "artifact.parent.mkdir(parents=True, exist_ok=True)",
                "artifact.write_text('built native dynamic artifact', encoding='utf-8')",
            ]
        ),
        encoding="utf-8",
    )


def _write_native_dynamic_fake_sign_script(repo_root: Path, exit_code: int = 0) -> Path:
    script = repo_root / "sign_native.py"
    script.write_text(
        "\n".join(
            [
                "from pathlib import Path",
                "import sys",
                "",
                f"exit_code = {exit_code}",
                "if exit_code:",
                "    print('signing failed', file=sys.stderr)",
                "    sys.exit(exit_code)",
                "artifact = Path(sys.argv[1])",
                "package_id = sys.argv[2] if len(sys.argv) > 2 else ''",
                "target_platform = sys.argv[3] if len(sys.argv) > 3 else ''",
                "signing_profile = sys.argv[4] if len(sys.argv) > 4 else ''",
                "with artifact.open('a', encoding='utf-8') as output:",
                "    output.write(f'\\nsigned:{package_id}:{target_platform}:{signing_profile}')",
                "print(f'signed {artifact.name}')",
            ]
        ),
        encoding="utf-8",
    )
    return script


def _write_native_dynamic_fake_notarize_script(
    repo_root: Path,
    exit_code: int = 0,
) -> Path:
    script = repo_root / "notarize_native.py"
    script.write_text(
        "\n".join(
            [
                "from pathlib import Path",
                "import sys",
                "",
                f"exit_code = {exit_code}",
                "if exit_code:",
                "    print('notarization failed', file=sys.stderr)",
                "    sys.exit(exit_code)",
                "artifact = Path(sys.argv[1])",
                "package_id = sys.argv[2] if len(sys.argv) > 2 else ''",
                "target_platform = sys.argv[3] if len(sys.argv) > 3 else ''",
                "signing_profile = sys.argv[4] if len(sys.argv) > 4 else ''",
                "notarization_profile = sys.argv[5] if len(sys.argv) > 5 else ''",
                "with artifact.open('a', encoding='utf-8') as output:",
                "    output.write(f'\\nnotarized:{package_id}:{target_platform}:{signing_profile}:{notarization_profile}')",
                "print(f'notarized {artifact.name}')",
            ]
        ),
        encoding="utf-8",
    )
    return script
