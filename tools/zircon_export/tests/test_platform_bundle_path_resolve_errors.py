from __future__ import annotations

import shutil
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.tests.export_test_support import (
    VALID_TEMPLATE,
    _append_template_file_entry,
    _file_sha256,
    _platform_bundle_args,
    _run_platform_bundle_quiet,
    json_loads,
)
from tools.zircon_export.tests.native_dynamic_export_test_support import (
    _write_native_dynamic_stage_plugins,
)


class PlatformBundlePathResolveErrorsTests(unittest.TestCase):
    def test_platform_bundle_rejects_repo_root_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            host = root / "zircon_runtime.exe"
            pack = root / "assets.zrpack"
            repo_root = root / "repo"
            host.write_text("host placeholder", encoding="utf-8")
            pack.write_text("pack placeholder", encoding="utf-8")
            args = _platform_bundle_args(
                out=out,
                profile="windows-release",
                template_dir=None,
                pack_file=pack,
                target_platform="windows-x86_64",
            )
            args.host_executable = str(host)
            args.repo_root = str(repo_root)
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if str(path) == str(repo_root):
                    raise OSError("simulated repo_root resolve failure")
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                exit_code = _run_platform_bundle_quiet(args)

            report = json_loads(
                (
                    out / "stages" / "platform_bundle" / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertIsNone(report["bundle_manifest"])
            self.assertIsNone(report["host_executable"])
            self.assertIsNone(report["pack"])
            self.assertFalse((out / "bundle" / "windows-release").exists())
            self.assertTrue(
                any(
                    "repo_root" in diagnostic
                    and "could not be resolved" in diagnostic
                    and "simulated repo_root resolve failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_platform_bundle_rejects_release_input_path_resolve_errors(self) -> None:
        for field in (
            "host_executable",
            "pack_file",
            "delta_pack",
            "template_dir",
            "template_root",
        ):
            with self.subTest(field=field):
                self.assert_path_resolve_error_becomes_report_diagnostic(field)

    def assert_path_resolve_error_becomes_report_diagnostic(self, field: str) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            host = root / "zircon_runtime.exe"
            pack = root / "assets.zrpack"
            delta_pack = root / "assets.delta.zrpd"
            host.write_text("host placeholder", encoding="utf-8")
            pack.write_text("pack placeholder", encoding="utf-8")
            delta_pack.write_text("delta placeholder", encoding="utf-8")
            args = _platform_bundle_args(
                out=out,
                profile="windows-release",
                template_dir=None,
                pack_file=pack,
                target_platform="windows-x86_64",
            )
            args.host_executable = str(host)
            args.delta_pack = str(delta_pack)
            args.delta_pack_explicit = True
            args.template_dir = str(root / "template")
            args.template_root = str(root / "templates")
            failing_path = {
                "host_executable": host,
                "pack_file": pack,
                "delta_pack": delta_pack,
                "template_dir": root / "template",
                "template_root": root / "templates",
            }[field]
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if str(path) == str(failing_path):
                    raise OSError(f"simulated {field} resolve failure")
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                exit_code = _run_platform_bundle_quiet(args)

            report = json_loads(
                (
                    out / "stages" / "platform_bundle" / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertIsNone(report["bundle_manifest"])
            self.assertIsNone(report["host_executable"])
            self.assertIsNone(report["pack"])
            self.assertIsNone(report["delta_pack"])
            self.assertFalse((out / "bundle" / "windows-release").exists())
            self.assertTrue(
                any(
                    field in diagnostic
                    and "could not be resolved" in diagnostic
                    and f"simulated {field} resolve failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_platform_bundle_rejects_template_plugins_filter_resolve_error(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            template_dir = root / "template"
            shutil.copytree(VALID_TEMPLATE, template_dir)
            template_stale_plugin = template_dir / "plugins" / "template-stale.dll"
            template_stale_plugin.parent.mkdir(parents=True)
            template_stale_plugin.write_text("template stale plugin", encoding="utf-8")
            _append_template_file_entry(
                template_dir,
                path="plugins/template-stale.dll",
                sha256=_file_sha256(template_stale_plugin),
            )
            host = root / "zircon_runtime.exe"
            pack = root / "assets.zrpack"
            host.write_text("host placeholder", encoding="utf-8")
            pack.write_text("pack placeholder", encoding="utf-8")
            native_plugins = _write_native_dynamic_stage_plugins(root / "native-source")
            args = _platform_bundle_args(
                out=out,
                profile="windows-release",
                template_dir=template_dir,
                pack_file=pack,
                target_platform="windows-x86_64",
            )
            args.host_executable = str(host)
            args.native_plugins_dir = str(native_plugins)
            stale_destination = (
                out / "bundle" / "windows-release" / "plugins" / "template-stale.dll"
            )
            original_resolve = Path.resolve
            stale_destination_resolve_count = 0

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                nonlocal stale_destination_resolve_count
                if str(path) == str(stale_destination):
                    stale_destination_resolve_count += 1
                    if stale_destination_resolve_count > 1:
                        raise OSError(
                            "simulated template plugins filter resolve failure"
                        )
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                exit_code = _run_platform_bundle_quiet(args)

            report = json_loads(
                (
                    out / "stages" / "platform_bundle" / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertIsNone(report["bundle_manifest"])
            self.assertIsNone(report["native_plugins"])
            self.assertIsNone(report["native_plugins_payload"])
            self.assertEqual(report["template_files"], [])
            self.assertFalse((out / "bundle" / "windows-release").exists())
            self.assertTrue(
                any(
                    "PlatformBundle template_files destination" in diagnostic
                    and "could not be resolved" in diagnostic
                    and "simulated template plugins filter resolve failure"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )


if __name__ == "__main__":
    unittest.main()
