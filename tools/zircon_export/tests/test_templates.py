from __future__ import annotations

import shutil
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.export_template import validate_export_template
from tools.zircon_export.tests.export_test_support import (
    LINUX_TEMPLATE,
    MACOS_TEMPLATE,
    REPO_ROOT,
    VALID_TEMPLATE,
    _file_sha256,
    _platform_bundle_args,
    _run_platform_bundle_quiet,
    _template_content_hash,
    json_dumps,
    json_loads,
)


class ExportTemplateValidationTests(unittest.TestCase):
    def test_template_version_mismatch_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            template_dir = Path(temp_dir) / "template"
            shutil.copytree(VALID_TEMPLATE, template_dir)
            manifest = template_dir / "template.toml"
            manifest.write_text(
                manifest.read_text(encoding="utf-8").replace(
                    "format_version = 1",
                    "format_version = 999",
                ),
                encoding="utf-8",
            )

            report = validate_export_template(
                template_dir=template_dir,
                expected_engine_version="0.1.0",
                profile="windows-release",
                expected_target_platform="windows-x86_64",
            )

        self.assertTrue(report["fatal"])
        self.assertTrue(
            any("format_version 999" in diagnostic for diagnostic in report["diagnostics"]),
            report["diagnostics"],
        )

    def test_valid_template_resolves_declared_host(self) -> None:
        report = validate_export_template(
            template_dir=VALID_TEMPLATE,
            expected_engine_version="0.1.0",
            profile="windows-release",
            expected_target_platform="windows-x86_64",
        )

        self.assertFalse(report["fatal"], report["diagnostics"])
        self.assertEqual(report["format_version"], 1)
        self.assertEqual(report["target_platform"], "windows-x86_64")
        self.assertEqual(
            Path(report["host_executable"]),
            VALID_TEMPLATE / "bin" / "zircon_runtime.host-placeholder",
        )
        self.assertEqual(
            report["computed_content_hash"],
            _template_content_hash(
                "bin/zircon_runtime.host-placeholder",
                report["files"][0]["sha256"],
            ),
        )

    def test_template_rejects_declared_directory_file(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            template_dir = Path(temp_dir) / "template"
            shutil.copytree(VALID_TEMPLATE, template_dir)
            host = template_dir / "bin" / "zircon_runtime.host-placeholder"
            host.unlink()
            host.mkdir()

            report = validate_export_template(
                template_dir=template_dir,
                expected_engine_version="0.1.0",
                profile="windows-release",
                expected_target_platform="windows-x86_64",
            )

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(
                any(
                    "template file bin/zircon_runtime.host-placeholder is not a file"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_template_rejects_declared_file_read_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            template_dir = Path(temp_dir) / "template"
            shutil.copytree(VALID_TEMPLATE, template_dir)
            unreadable_file = (
                template_dir / "bin" / "zircon_runtime.host-placeholder"
            ).resolve()
            original_read_bytes = Path.read_bytes

            def read_bytes_or_fail(path: Path) -> bytes:
                if path.resolve() == unreadable_file:
                    raise OSError("simulated template read failure")
                return original_read_bytes(path)

            with mock.patch.object(Path, "read_bytes", read_bytes_or_fail):
                report = validate_export_template(
                    template_dir=template_dir,
                    expected_engine_version="0.1.0",
                    profile="windows-release",
                    expected_target_platform="windows-x86_64",
                )

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(
                any(
                    "template file bin/zircon_runtime.host-placeholder could not be read"
                    in diagnostic
                    and "simulated template read failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_template_rejects_declared_file_path_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            template_dir = Path(temp_dir) / "template"
            shutil.copytree(VALID_TEMPLATE, template_dir)
            failing_file = template_dir / "bin" / "zircon_runtime.host-placeholder"
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if Path(path) == failing_file:
                    raise OSError("simulated template path resolve failure")
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                report = validate_export_template(
                    template_dir=template_dir,
                    expected_engine_version="0.1.0",
                    profile="windows-release",
                    expected_target_platform="windows-x86_64",
                )

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(
                any(
                    "template path bin/zircon_runtime.host-placeholder could not be resolved"
                    in diagnostic
                    and "simulated template path resolve failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_template_rejects_manifest_directory(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            template_dir = Path(temp_dir) / "template"
            shutil.copytree(VALID_TEMPLATE, template_dir)
            manifest = template_dir / "template.toml"
            manifest.unlink()
            manifest.mkdir()

            report = validate_export_template(
                template_dir=template_dir,
                expected_engine_version="0.1.0",
                profile="windows-release",
                expected_target_platform="windows-x86_64",
            )

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(
                any(
                    f"export template manifest {manifest} is not a file" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_template_rejects_aliasing_file_and_host_paths(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            template_dir = Path(temp_dir) / "template"
            shutil.copytree(VALID_TEMPLATE, template_dir)
            manifest = template_dir / "template.toml"
            aliased_path = "bin/./zircon_runtime.host-placeholder"
            bundle_path = "bin/zircon_runtime.host-placeholder"
            aliased_hash = _template_content_hash(
                aliased_path,
                _file_sha256(template_dir / "bin" / "zircon_runtime.host-placeholder"),
                bundle_path=bundle_path,
            )
            manifest.write_text(
                manifest.read_text(encoding="utf-8")
                .replace(
                    'host_executable = "bin/zircon_runtime.host-placeholder"',
                    f'host_executable = "{aliased_path}"',
                )
                .replace(
                    'path = "bin/zircon_runtime.host-placeholder"',
                    f'path = "{aliased_path}"\nbundle_path = "{bundle_path}"',
                )
                .replace(
                    'content_hash = "e5acc99c1ccc705e08793501ff1226adcc8e181c6d1d9ffbff7cef2270a99304"',
                    f'content_hash = "{aliased_hash}"',
                ),
                encoding="utf-8",
            )

            report = validate_export_template(
                template_dir=template_dir,
                expected_engine_version="0.1.0",
                profile="windows-release",
                expected_target_platform="windows-x86_64",
            )

        self.assertTrue(report["fatal"])
        self.assertTrue(
            any("paths.host_executable must be a safe relative path" in diagnostic for diagnostic in report["diagnostics"]),
            report["diagnostics"],
        )
        self.assertTrue(
            any("[[files]] entry 0 path must be a safe relative path" in diagnostic for diagnostic in report["diagnostics"]),
            report["diagnostics"],
        )

    def test_linux_template_materializes_directory_layout(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            pack = root / "assets.zrpack"
            pack.write_text("pack placeholder", encoding="utf-8")

            exit_code = _run_platform_bundle_quiet(
                _platform_bundle_args(
                    out=root / "out",
                    profile="linux-release",
                    template_dir=LINUX_TEMPLATE,
                    pack_file=pack,
                    target_platform="linux-x86_64",
                )
            )

            self.assertEqual(exit_code, 0)
            self.assertTrue((root / "out" / "bundle" / "linux-release" / "ZirconRuntime").exists())
            self.assertTrue(
                (root / "out" / "bundle" / "linux-release" / "data" / "assets.zrpack").exists()
            )
            self.assertTrue(
                (root / "out" / "bundle" / "linux-release" / "zircon-export.json").exists()
            )

    def test_platform_bundle_rejects_host_copy_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            pack = root / "assets.zrpack"
            pack.write_text("pack placeholder", encoding="utf-8")
            host = VALID_TEMPLATE / "bin" / "zircon_runtime.host-placeholder"
            original_copy2 = shutil.copy2

            def copy_or_fail(source: Path, destination: Path) -> None:
                if Path(source).resolve() == host.resolve():
                    raise OSError("simulated host copy failure")
                original_copy2(source, destination)

            with mock.patch(
                "tools.zircon_export.platform_bundle.shutil.copy2",
                side_effect=copy_or_fail,
            ):
                exit_code = _run_platform_bundle_quiet(
                    _platform_bundle_args(
                        out=root / "out",
                        profile="windows-release",
                        template_dir=VALID_TEMPLATE,
                        pack_file=pack,
                        target_platform="windows-x86_64",
                    )
                )

            report = json_loads(
                (
                    root
                    / "out"
                    / "stages"
                    / "platform_bundle"
                    / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertIsNone(report["bundle_manifest"])
            self.assertFalse((root / "out" / "bundle" / "windows-release").exists())
            self.assertTrue(
                any(
                    "host executable" in diagnostic
                    and "could not be copied" in diagnostic
                    and "simulated host copy failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_platform_bundle_rejects_template_copy_source_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            template_dir = root / "template"
            shutil.copytree(VALID_TEMPLATE, template_dir)
            pack = root / "assets.zrpack"
            pack.write_text("pack placeholder", encoding="utf-8")
            failing_source = template_dir / "bin" / "zircon_runtime.host-placeholder"
            template_report = validate_export_template(
                template_dir=template_dir,
                expected_engine_version="0.1.0",
                profile="windows-release",
                expected_target_platform="windows-x86_64",
            )
            self.assertFalse(template_report["fatal"], template_report["diagnostics"])
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if Path(path) == failing_source:
                    raise OSError("simulated template source resolve failure")
                return original_resolve(path, *args, **kwargs)

            with (
                mock.patch(
                    "tools.zircon_export.platform_bundle.validate_export_template",
                    return_value=template_report,
                ),
                mock.patch.object(Path, "resolve", resolve_or_fail),
            ):
                exit_code = _run_platform_bundle_quiet(
                    _platform_bundle_args(
                        out=root / "out",
                        profile="windows-release",
                        template_dir=template_dir,
                        pack_file=pack,
                        target_platform="windows-x86_64",
                    )
                )

            report = json_loads(
                (
                    root
                    / "out"
                    / "stages"
                    / "platform_bundle"
                    / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertIsNone(report["bundle_manifest"])
            self.assertFalse((root / "out" / "bundle" / "windows-release").exists())
            self.assertTrue(
                any(
                    "template file" in diagnostic
                    and str(failing_source) in diagnostic
                    and "could not be resolved during bundle copy" in diagnostic
                    and "simulated template source resolve failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_platform_bundle_rejects_bundle_output_path_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            template_dir = root / "template"
            shutil.copytree(LINUX_TEMPLATE, template_dir)
            pack = root / "assets.zrpack"
            pack.write_text("pack placeholder", encoding="utf-8")
            failing_destination = (
                root / "out" / "bundle" / "linux-release" / "ZirconRuntime"
            )
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if Path(path) == failing_destination:
                    raise OSError("simulated bundle output resolve failure")
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                exit_code = _run_platform_bundle_quiet(
                    _platform_bundle_args(
                        out=root / "out",
                        profile="linux-release",
                        template_dir=template_dir,
                        pack_file=pack,
                        target_platform="linux-x86_64",
                    )
                )

            report = json_loads(
                (
                    root
                    / "out"
                    / "stages"
                    / "platform_bundle"
                    / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertIsNone(report["bundle_manifest"])
            self.assertFalse((root / "out" / "bundle" / "linux-release").exists())
            self.assertTrue(
                any(
                    "bundle path ZirconRuntime could not be resolved" in diagnostic
                    and "simulated bundle output resolve failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_platform_bundle_rejects_bundle_manifest_write_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            pack = root / "assets.zrpack"
            pack.write_text("pack placeholder", encoding="utf-8")
            bundle_manifest = (
                root
                / "out"
                / "bundle"
                / "windows-release"
                / "bundle.json"
            ).resolve()
            original_write_text = Path.write_text

            def write_text_or_fail(path: Path, *args: object, **kwargs: object) -> int:
                if path.resolve() == bundle_manifest:
                    raise OSError("simulated bundle manifest write failure")
                return original_write_text(path, *args, **kwargs)

            with mock.patch.object(Path, "write_text", write_text_or_fail):
                exit_code = _run_platform_bundle_quiet(
                    _platform_bundle_args(
                        out=root / "out",
                        profile="windows-release",
                        template_dir=VALID_TEMPLATE,
                        pack_file=pack,
                        target_platform="windows-x86_64",
                    )
                )

            report = json_loads(
                (
                    root
                    / "out"
                    / "stages"
                    / "platform_bundle"
                    / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertIsNone(report["bundle_manifest"])
            self.assertFalse((root / "out" / "bundle" / "windows-release").exists())
            self.assertTrue(
                any(
                    "bundle manifest" in diagnostic
                    and "could not be written" in diagnostic
                    and "simulated bundle manifest write failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_macos_template_materializes_app_bundle_layout(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            pack = root / "assets.zrpack"
            pack.write_text("pack placeholder", encoding="utf-8")

            exit_code = _run_platform_bundle_quiet(
                _platform_bundle_args(
                    out=root / "out",
                    profile="macos-release",
                    template_dir=MACOS_TEMPLATE,
                    pack_file=pack,
                    target_platform="macos-aarch64",
                )
            )

            app_root = root / "out" / "bundle" / "macos-release" / "ZirconRuntime.app"
            self.assertEqual(exit_code, 0)
            self.assertTrue((app_root / "Contents" / "MacOS" / "ZirconRuntime").exists())
            self.assertTrue((app_root / "Contents" / "Resources" / "assets.zrpack").exists())
            self.assertTrue((app_root / "Contents" / "Info.plist").exists())
            self.assertTrue((app_root / "Contents" / "Resources" / "zircon-export.json").exists())

    def test_template_root_resolves_compatible_platform_bundle_template(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            pack = root / "assets.zrpack"
            pack.write_text("pack placeholder", encoding="utf-8")

            exit_code = _run_platform_bundle_quiet(
                _platform_bundle_args(
                    out=root / "out",
                    profile="linux-release",
                    template_dir=None,
                    template_root=REPO_ROOT / "export-templates",
                    pack_file=pack,
                    target_platform="linux-x86_64",
                )
            )

            report = json_loads(
                (root / "out" / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 0)
            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertEqual(
                Path(report["template_resolution"]["template_dir"]),
                LINUX_TEMPLATE,
            )
            self.assertEqual(report["template"]["template_id"], "linux-x86_64-library_embed-debug")
            self.assertEqual(
                Path(report["host_source"]),
                Path(report["template"]["host_executable"]),
            )
            self.assertEqual(report["host_source_origin"], "template")
            self.assertTrue(
                (root / "out" / "bundle" / "linux-release" / "data" / "assets.zrpack").exists()
            )

    def test_template_root_rejects_workspace_manifest_directory(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            repo_root.mkdir()
            (repo_root / "Cargo.toml").mkdir()
            pack = root / "assets.zrpack"
            pack.write_text("pack placeholder", encoding="utf-8")
            args = _platform_bundle_args(
                out=root / "out",
                profile="linux-release",
                template_dir=None,
                template_root=REPO_ROOT / "export-templates",
                pack_file=pack,
                target_platform="linux-x86_64",
            )
            args.repo_root = str(repo_root)
            args.engine_version = None

            exit_code = _run_platform_bundle_quiet(args)

            report = json_loads(
                (root / "out" / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertIsNone(report["template_resolution"])
            self.assertIsNone(report["bundle_manifest"])
            self.assertFalse((root / "out" / "bundle" / "linux-release").exists())
            self.assertTrue(
                any(
                    "workspace manifest" in diagnostic
                    and "Cargo.toml" in diagnostic
                    and "is not a file" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_template_root_skips_manifest_directory_candidate(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            template_root = root / "templates"
            valid_template = template_root / "linux-valid"
            invalid_template = template_root / "linux-invalid"
            shutil.copytree(LINUX_TEMPLATE, valid_template)
            shutil.copytree(LINUX_TEMPLATE, invalid_template)
            manifest = invalid_template / "template.toml"
            manifest.unlink()
            manifest.mkdir()
            pack = root / "assets.zrpack"
            pack.write_text("pack placeholder", encoding="utf-8")

            exit_code = _run_platform_bundle_quiet(
                _platform_bundle_args(
                    out=root / "out",
                    profile="linux-release",
                    template_dir=None,
                    template_root=template_root,
                    pack_file=pack,
                    target_platform="linux-x86_64",
                )
            )

            report = json_loads(
                (root / "out" / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 0, report["diagnostics"])
            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertEqual(Path(report["template_resolution"]["template_dir"]), valid_template)
            skipped_candidates = report["template_resolution"]["skipped_candidates"]
            self.assertEqual(len(skipped_candidates), 1)
            self.assertEqual(Path(skipped_candidates[0]["template_dir"]), invalid_template)
            self.assertTrue(
                any(
                    "is not a file" in diagnostic
                    for diagnostic in skipped_candidates[0]["diagnostics"]
                ),
                skipped_candidates[0]["diagnostics"],
            )

    def test_template_root_ignores_target_platform_from_wrong_profile_validate_report(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            validate_report = out / "stages" / "validate" / "report.json"
            validate_report.parent.mkdir(parents=True)
            validate_report.write_text(
                json_dumps(
                    {
                        "stage": "Validate",
                        "profile": "other-profile",
                        "fatal": False,
                        "diagnostics": [],
                        "profile_summary": {
                            "strategies": ["library_embed"],
                            "target_platform": "linux-x86_64",
                        },
                    }
                ),
                encoding="utf-8",
            )
            pack = root / "assets.zrpack"
            pack.write_text("pack placeholder", encoding="utf-8")

            exit_code = _run_platform_bundle_quiet(
                _platform_bundle_args(
                    out=out,
                    profile="windows-release",
                    template_dir=None,
                    template_root=REPO_ROOT / "export-templates",
                    pack_file=pack,
                    target_platform=None,
                )
            )

            report = json_loads(
                (out / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertIsNone(report["template_resolution"])
            self.assertTrue(
                any(
                    "Validate report profile other-profile does not match requested profile windows-release"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_template_root_skips_invalid_matching_template_candidate(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            template_root = root / "templates"
            valid_template = template_root / "linux-valid"
            invalid_template = template_root / "linux-invalid"
            shutil.copytree(LINUX_TEMPLATE, valid_template)
            shutil.copytree(LINUX_TEMPLATE, invalid_template)
            manifest = invalid_template / "template.toml"
            manifest.write_text(
                manifest.read_text(encoding="utf-8").replace(
                    'content_hash = "ba15973051598ad7709f6314f11ab35863f322306cf565ff875747e999896398"',
                    'content_hash = "0000000000000000000000000000000000000000000000000000000000000000"',
                ),
                encoding="utf-8",
            )
            pack = root / "assets.zrpack"
            pack.write_text("pack placeholder", encoding="utf-8")

            exit_code = _run_platform_bundle_quiet(
                _platform_bundle_args(
                    out=root / "out",
                    profile="linux-release",
                    template_dir=None,
                    template_root=template_root,
                    pack_file=pack,
                    target_platform="linux-x86_64",
                )
            )

            report = json_loads(
                (root / "out" / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 0, report["diagnostics"])
            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertEqual(Path(report["template_resolution"]["template_dir"]), valid_template)
            self.assertEqual(report["template"]["template_id"], "linux-x86_64-library_embed-debug")
            skipped_candidates = report["template_resolution"]["skipped_candidates"]
            self.assertEqual(len(skipped_candidates), 1)
            self.assertEqual(Path(skipped_candidates[0]["template_dir"]), invalid_template)
            self.assertTrue(
                any(
                    "content_hash" in diagnostic
                    for diagnostic in skipped_candidates[0]["diagnostics"]
                ),
                skipped_candidates[0]["diagnostics"],
            )

    def test_template_root_skips_malformed_template_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            template_root = root / "templates"
            valid_template = template_root / "linux-valid"
            malformed_template = template_root / "malformed"
            shutil.copytree(LINUX_TEMPLATE, valid_template)
            malformed_template.mkdir(parents=True)
            (malformed_template / "template.toml").write_text(
                'format_version = "not closed',
                encoding="utf-8",
            )
            pack = root / "assets.zrpack"
            pack.write_text("pack placeholder", encoding="utf-8")

            exit_code = _run_platform_bundle_quiet(
                _platform_bundle_args(
                    out=root / "out",
                    profile="linux-release",
                    template_dir=None,
                    template_root=template_root,
                    pack_file=pack,
                    target_platform="linux-x86_64",
                )
            )

            report = json_loads(
                (root / "out" / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 0, report["diagnostics"])
            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertEqual(Path(report["template_resolution"]["template_dir"]), valid_template)
            skipped_candidates = report["template_resolution"]["skipped_candidates"]
            self.assertEqual(len(skipped_candidates), 1)
            self.assertEqual(Path(skipped_candidates[0]["template_dir"]), malformed_template)
            self.assertTrue(
                any(
                    "not valid TOML" in diagnostic
                    for diagnostic in skipped_candidates[0]["diagnostics"]
                ),
                skipped_candidates[0]["diagnostics"],
            )

    def test_template_root_reports_missing_profile_match(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            pack = root / "assets.zrpack"
            pack.write_text("pack placeholder", encoding="utf-8")

            exit_code = _run_platform_bundle_quiet(
                _platform_bundle_args(
                    out=root / "out",
                    profile="missing-profile",
                    template_dir=None,
                    template_root=REPO_ROOT / "export-templates",
                    pack_file=pack,
                    target_platform="windows-x86_64",
                )
            )

            report = json_loads(
                (root / "out" / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertTrue(report["template_resolution"]["fatal"])
            self.assertTrue(
                any(
                    "no export template" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse(
                (root / "out" / "bundle" / "missing-profile" / "assets.zrpack").exists()
            )
            self.assertFalse((root / "out" / "bundle" / "missing-profile").exists())
            self.assertIsNone(report["bundle_manifest"])


if __name__ == "__main__":
    unittest.main()
