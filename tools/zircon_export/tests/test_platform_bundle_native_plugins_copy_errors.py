from __future__ import annotations

import shutil
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.tests.export_test_support import (
    _platform_bundle_args,
    _run_platform_bundle_quiet,
    json_loads,
)
from tools.zircon_export.tests.native_dynamic_export_test_support import (
    _write_native_dynamic_stage_plugins,
)


class PlatformBundleNativePluginsCopyErrorsTests(unittest.TestCase):
    def test_platform_bundle_rejects_native_plugins_path_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            host = root / "zircon_runtime.exe"
            pack = root / "assets.zrpack"
            host.write_text("host placeholder", encoding="utf-8")
            pack.write_text("pack placeholder", encoding="utf-8")
            native_plugins = _write_native_dynamic_stage_plugins(root / "native-source")
            args = _platform_bundle_args(
                out=out,
                profile="windows-release",
                template_dir=None,
                pack_file=pack,
                target_platform="windows-x86_64",
            )
            args.host_executable = str(host)
            args.native_plugins_dir = str(native_plugins)
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if str(path) == str(native_plugins):
                    raise OSError("simulated native plugins resolve failure")
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                exit_code = _run_platform_bundle_quiet(args)

            stage_report = out / "stages" / "platform_bundle" / "report.json"
            report = json_loads(stage_report.read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertIsNone(report["bundle_manifest"])
            self.assertIsNone(report["native_plugins"])
            self.assertIsNone(report["native_plugins_payload"])
            self.assertFalse((out / "bundle" / "windows-release").exists())
            self.assertTrue(
                any(
                    "native_plugins_dir" in diagnostic
                    and "could not be resolved" in diagnostic
                    and "simulated native plugins resolve failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_platform_bundle_rejects_native_plugins_payload_source_resolve_error(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            host = root / "zircon_runtime.exe"
            pack = root / "assets.zrpack"
            host.write_text("host placeholder", encoding="utf-8")
            pack.write_text("pack placeholder", encoding="utf-8")
            native_plugins = _write_native_dynamic_stage_plugins(root / "native-source")
            args = _platform_bundle_args(
                out=out,
                profile="windows-release",
                template_dir=None,
                pack_file=pack,
                target_platform="windows-x86_64",
            )
            args.host_executable = str(host)
            args.native_plugins_dir = str(native_plugins)
            original_resolve = Path.resolve
            resolve_count = 0

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                nonlocal resolve_count
                if str(path) == str(native_plugins):
                    resolve_count += 1
                    if resolve_count > 1:
                        raise OSError(
                            "simulated native plugins payload source resolve failure"
                        )
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                exit_code = _run_platform_bundle_quiet(args)

            stage_report = out / "stages" / "platform_bundle" / "report.json"
            report = json_loads(stage_report.read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertIsNone(report["bundle_manifest"])
            self.assertIsNone(report["native_plugins"])
            self.assertIsNone(report["native_plugins_payload"])
            self.assertFalse((out / "bundle" / "windows-release").exists())
            self.assertTrue(
                any(
                    "NativeDynamic payload source" in diagnostic
                    and "could not be resolved" in diagnostic
                    and "simulated native plugins payload source resolve failure"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_platform_bundle_rejects_native_plugins_copy_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            host = root / "zircon_runtime.exe"
            pack = root / "assets.zrpack"
            host.write_text("host placeholder", encoding="utf-8")
            pack.write_text("pack placeholder", encoding="utf-8")
            native_plugins = _write_native_dynamic_stage_plugins(root / "native-source")
            native_artifact = (
                native_plugins
                / "animation"
                / "native"
                / "zircon_plugin_animation.dll"
            ).resolve()
            args = _platform_bundle_args(
                out=out,
                profile="windows-release",
                template_dir=None,
                pack_file=pack,
                target_platform="windows-x86_64",
            )
            args.host_executable = str(host)
            args.native_plugins_dir = str(native_plugins)
            original_copy2 = shutil.copy2

            def copy_or_fail(source: Path, destination: Path) -> None:
                if Path(source).resolve() == native_artifact:
                    raise OSError("simulated native plugins bundle copy failure")
                original_copy2(source, destination)

            with mock.patch(
                "tools.zircon_export.platform_bundle_native_plugins_materialize.shutil.copy2",
                side_effect=copy_or_fail,
            ):
                exit_code = _run_platform_bundle_quiet(args)

            stage_report = out / "stages" / "platform_bundle" / "report.json"
            report = json_loads(stage_report.read_text(encoding="utf-8"))
            bundle_dir = out / "bundle" / "windows-release"
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertIsNone(report["bundle_manifest"])
            self.assertIsNone(report["native_plugins"])
            self.assertIsNone(report["native_plugins_payload"])
            self.assertFalse(bundle_dir.exists())
            self.assertTrue(
                any(
                    "native plugins file" in diagnostic
                    and "could not be copied" in diagnostic
                    and "simulated native plugins bundle copy failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )


if __name__ == "__main__":
    unittest.main()
