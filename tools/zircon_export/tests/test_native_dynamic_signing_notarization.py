from __future__ import annotations

import hashlib
import json
import sys
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.tests.export_test_support import _run_stage_quiet
from tools.zircon_export.tests.native_dynamic_test_support import (
    _export_args,
    _write_native_dynamic_fake_notarize_script,
    _write_native_dynamic_fake_sign_script,
    _write_validate_report_with_native_dynamic_exports,
    _write_windows_native_dynamic_package_fixture_at,
)


class NativeDynamicSigningNotarizationTests(unittest.TestCase):
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

    def test_native_dynamic_signing_rejects_schema_invalid_arguments_before_external_command(
        self,
    ) -> None:
        cases = [
            (
                "command",
                {"native_dynamic_sign_command": " "},
                "NativeDynamic signing command must be a non-empty trimmed string",
            ),
            (
                "arg",
                {"native_dynamic_sign_arg": ["{artifact}", " {package_id} "]},
                "NativeDynamic signing args[1] must be a non-empty trimmed string",
            ),
            (
                "profile",
                {"native_dynamic_sign_profile": " windows-store "},
                "NativeDynamic signing profile must be a non-empty trimmed string",
            ),
            (
                "platform",
                {"native_dynamic_sign_platform": " windows "},
                "NativeDynamic signing allowed platforms[0] must be a non-empty trimmed string",
            ),
        ]
        for case_name, overrides, expected_diagnostic in cases:
            with self.subTest(case_name=case_name):
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
                    args.native_dynamic_sign_profile = "windows-store"
                    args.native_dynamic_sign_platform = "windows"
                    for field, value in overrides.items():
                        setattr(args, field, value)

                    exit_code = _run_stage_quiet(args)

                    stage_dir = out / "stages" / "native_dynamic"
                    report = json.loads(
                        (stage_dir / "report.json").read_text(encoding="utf-8")
                    )
                    diagnostics = "\n".join(report["diagnostics"])
                    self.assertEqual(exit_code, 2)
                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertIn(expected_diagnostic, diagnostics)
                    self.assertTrue(report["native_signing"]["enabled"])
                    self.assertEqual(report["native_signing"]["packages"], [])
                    self.assertNotIn("could not start", diagnostics)
                    self.assertNotIn("exited with code", diagnostics)
                    self.assertFalse((stage_dir / "plugins" / "animation").exists())

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

    def test_native_dynamic_notarization_rejects_schema_invalid_arguments_before_external_command(
        self,
    ) -> None:
        cases = [
            (
                "command",
                {"native_dynamic_notarize_command": " "},
                "NativeDynamic notarization command must be a non-empty trimmed string",
            ),
            (
                "arg",
                {"native_dynamic_notarize_arg": ["{artifact}", " {package_id} "]},
                "NativeDynamic notarization args[1] must be a non-empty trimmed string",
            ),
            (
                "profile",
                {"native_dynamic_notarize_profile": " windows-attestation "},
                "NativeDynamic notarization profile must be a non-empty trimmed string",
            ),
            (
                "platform",
                {"native_dynamic_notarize_platform": " windows "},
                "NativeDynamic notarization allowed platforms[0] must be a non-empty trimmed string",
            ),
        ]
        for case_name, overrides, expected_diagnostic in cases:
            with self.subTest(case_name=case_name):
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
                    args.native_dynamic_sign_arg = [str(signer), "{artifact}"]
                    args.native_dynamic_sign_profile = "windows-store"
                    args.native_dynamic_notarize_command = sys.executable
                    args.native_dynamic_notarize_arg = [str(notarizer), "{artifact}"]
                    args.native_dynamic_notarize_profile = "windows-attestation"
                    args.native_dynamic_notarize_platform = "windows"
                    for field, value in overrides.items():
                        setattr(args, field, value)

                    exit_code = _run_stage_quiet(args)

                    stage_dir = out / "stages" / "native_dynamic"
                    report = json.loads(
                        (stage_dir / "report.json").read_text(encoding="utf-8")
                    )
                    diagnostics = "\n".join(report["diagnostics"])
                    self.assertEqual(exit_code, 2)
                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertIn(expected_diagnostic, diagnostics)
                    self.assertTrue(report["native_notarization"]["enabled"])
                    self.assertEqual(report["native_signing"]["packages"], [])
                    self.assertEqual(report["native_notarization"]["packages"], [])
                    self.assertNotIn("could not start", diagnostics)
                    self.assertNotIn("exited with code", diagnostics)
                    self.assertFalse((stage_dir / "plugins" / "animation").exists())

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
