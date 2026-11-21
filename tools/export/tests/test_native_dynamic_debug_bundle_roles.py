"""Keep debug bundle contents in delivery evidence but out of runtime roles."""

from contextlib import redirect_stdout
import hashlib
import io
import json
import os
from pathlib import Path
from tempfile import TemporaryDirectory
import tomllib
import unittest
from unittest.mock import patch

from tools.export.cli import main as export_main
from tools.export.native_dynamic_payload import native_dynamic_stage_payload_summary
from tools.export.native_signing import native_dynamic_signable_artifacts
from tools.export.pipeline_report_native_dynamic_payload_bundle_evidence import (
    platform_bundle_native_plugins_current_bundle_evidence_diagnostics,
)
from tools.export.plugin_build_signature import write_plugin_build_signature
from tools.export.tests.native_dynamic_test_support import (
    _write_validate_report_with_native_dynamic_exports,
)


MAC_LIBRARY = "libanimation.dylib"
MAC_DWARF = MAC_LIBRARY + ".dSYM/Contents/Resources/DWARF/"


class NativeDynamicDebugBundleRolesTests(unittest.TestCase):
    def test_dsym_leaf_with_library_suffix_is_not_a_loadable_artifact(self) -> None:
        self._check_roles("macos-aarch64", MAC_LIBRARY, MAC_DWARF + MAC_LIBRARY)

    def test_dsym_leaf_without_library_suffix_is_still_copied(self) -> None:
        self._check_roles("macos-aarch64", MAC_LIBRARY, MAC_DWARF + "libanimation")

    def test_windows_and_linux_sidecars_are_preserved_but_not_loadable(self) -> None:
        for platform, library, symbol in (
            ("windows-x86_64", "animation.dll", "animation.pdb"),
            ("linux-x86_64", "libanimation.so", "libanimation.dbg"),
        ):
            with self.subTest(platform=platform):
                self._check_roles(platform, library, symbol)

    def test_dsym_with_same_leaf_cannot_replace_the_library(self) -> None:
        with TemporaryDirectory() as directory:
            root = Path(directory)
            code, report, stage = self._stage(
                root, "macos-aarch64", None, MAC_DWARF + MAC_LIBRARY
            )
            self.assertEqual(code, 2)
            self.assertTrue(report["fatal"])
            self.assertEqual(report["materialized_packages"], [])
            self.assertFalse((stage / "plugins/native_plugins.toml").exists())
            self.assertTrue(
                any("no loadable native library artifacts" in s for s in report["diagnostics"]),
                report["diagnostics"],
            )

    def test_v2_handoff_validates_reported_roles_against_symbol_payload(self) -> None:
        with TemporaryDirectory() as directory:
            case = Path(directory)
            platform = "macos-aarch64"
            symbol = MAC_DWARF + MAC_LIBRARY
            code, report, stage = self._stage(case, platform, MAC_LIBRARY, symbol)
            self.assertEqual(code, 0, report["diagnostics"])
            lib = "plugins/animation/native/" + MAC_LIBRARY
            debug = "plugins/animation/native/" + symbol
            self.assertIn(debug, [row["path"] for row in report["file_manifest"]])
            for roles, should_accept in (([lib], True), ([lib, debug], False)):
                with self.subTest(should_accept=should_accept):
                    # Keep the original v2 input, file manifest, hashes and staged bytes.
                    # Isolate the read-side contract from the stage writer's output.
                    package = report["materialized_packages"][0]
                    package["loadable_artifacts"] = roles
                    package["loadable_artifact_count"] = len(roles)
                    (stage / "report.json").write_text(json.dumps(report), encoding="utf-8")
                    diagnostics = []
                    handoff = native_dynamic_stage_payload_summary(
                        case / "out", platform, stage / "plugins", diagnostics,
                    )
                    if should_accept:
                        self.assertEqual(diagnostics, [])
                        self.assertIsNotNone(handoff)
                        self.assertEqual(handoff["materialized_packages"][0]["loadable_artifacts"], roles)
                    else:
                        self.assertIsNone(handoff)
                        self.assertTrue(any("loadable_artifacts" in line for line in diagnostics), diagnostics)

    def test_directory_fallback_does_not_turn_unaccompanied_symbols_into_loadables(self) -> None:
        with TemporaryDirectory() as directory:
            root = Path(directory)
            plugins = root / "plugins"
            debug = "animation/native/" + MAC_DWARF + MAC_LIBRARY
            (plugins / debug).parent.mkdir(parents=True)
            (plugins / debug).write_bytes(b"symbols only")
            diagnostics = []
            summary = native_dynamic_stage_payload_summary(root, "macos-aarch64", plugins, diagnostics)
            self.assertEqual(diagnostics, [])
            self.assertIsNotNone(summary)
            self.assertEqual([row["path"] for row in summary["file_manifest"]], ["plugins/" + debug])
            self.assertEqual(summary["materialized_packages"][0]["loadable_artifacts"], [])
            self.assertEqual(summary["materialized_packages"][0]["loadable_artifact_count"], 0)
            self.assertEqual(platform_bundle_native_plugins_current_bundle_evidence_diagnostics(
                summary, plugins, summary["file_manifest"], summary["materialized_packages"],
            ), [])

    def test_debug_directory_boundary_keeps_real_root_and_nested_libraries(self) -> None:
        with TemporaryDirectory() as directory:
            root = Path(directory) / "container.dSYM"
            plugins = root / "plugins"
            package = plugins / "animation.dSYM"
            expected = [
                "animation.dylib",  # standalone distribution's root loadable is valid
                "native/backend/extension.dylib",  # existing packaged nested library
                "native/vendor.dSYM.files/extension.dylib",  # not a debug bundle name
            ]
            sidecars = [
                "native/.dSYM/Contents/Resources/DWARF/animation.dylib",
                "native/vendor.DsYm/Contents/Resources/DWARF/animation.dylib",
                "native/windows.pdb/vendor.dll",  # materializer recognizes debug directories
                "native/linux.dbg/libvendor.so",
                "native/animation.pdb",
                "native/libanimation.dbg",
            ]
            for path in expected + sidecars:
                (package / path).parent.mkdir(parents=True, exist_ok=True)
                (package / path).write_bytes(b"payload fixture")
            diagnostics = []
            summary = native_dynamic_stage_payload_summary(root, "macos-aarch64", plugins, diagnostics)
            self.assertEqual(diagnostics, [])
            self.assertIsNotNone(summary)
            expected_bundle = ["plugins/animation.dSYM/" + path for path in expected]
            with self.subTest(role="directory summary"):
                self.assertEqual(
                    set(row["path"] for row in summary["file_manifest"]),
                    set("plugins/animation.dSYM/" + path for path in expected + sidecars),
                )
                self.assertEqual(summary["materialized_packages"][0]["loadable_artifacts"], expected_bundle)
            with self.subTest(role="final bundle evidence"):
                # Test this consumer independently even when the directory writer mislabels a role.
                roles = dict(summary["materialized_packages"][0], loadable_artifacts=expected_bundle,
                             loadable_artifact_count=len(expected_bundle))
                self.assertEqual(platform_bundle_native_plugins_current_bundle_evidence_diagnostics(
                    summary, plugins, summary["file_manifest"], [roles],
                ), [])
            with self.subTest(role="optional notarization selection"):
                signable = native_dynamic_signable_artifacts(
                    package, {".dylib"}, diagnostics, operation="notarization", package_id="animation",
                )
                self.assertEqual([p.relative_to(package).as_posix() for p in signable], expected)
            with self.subTest(role="standalone signature manifest"):
                self.assertTrue(write_plugin_build_signature(
                    package_id="animation", directory="animation", package_dir=package,
                    target_platform="macos-aarch64", signing={"enabled": False}, diagnostics=diagnostics,
                ), diagnostics)
                signature = tomllib.loads((package / "animation.sig").read_text(encoding="utf-8"))
                self.assertEqual([row["path"] for row in signature["loadable_artifacts"]], expected)

    def _check_roles(self, platform: str, library: str, symbol: str) -> None:
        with TemporaryDirectory() as directory:
            case = Path(directory)
            code, report, stage = self._stage(case, platform, library, symbol)
            self.assertEqual(code, 0, report["diagnostics"])
            self.assertFalse(report["fatal"], report["diagnostics"])
            plugin = stage / "plugins/animation"
            package_report = tomllib.loads(
                (plugin / "native_dynamic_package.toml").read_text(encoding="utf-8")
            )
            stage_rows = {row["path"]: row for row in report["file_manifest"]}
            package_rows = {row["path"]: row for row in package_report["payload"]["files"]}
            for leaf, payload in ((library, b"library"), (symbol, b"debug symbols")):
                relative = "native/" + leaf
                stage_relative = "plugins/animation/" + relative
                self.assertEqual((plugin / relative).read_bytes(), payload)
                row = {"path": relative, "bytes": len(payload), "sha256": hashlib.sha256(payload).hexdigest()}
                self.assertEqual(package_rows[relative], row)
                self.assertEqual(stage_rows[stage_relative], {**row, "path": stage_relative})
            expected = ["plugins/animation/native/" + library]
            with self.subTest(role="stage report"):
                self.assertEqual(report["materialized_packages"][0]["loadable_artifacts"], expected)
                self.assertEqual(report["materialized_packages"][0]["loadable_artifact_count"], 1)
            with self.subTest(role="PlatformBundle handoff"):
                diagnostics = []
                handoff = native_dynamic_stage_payload_summary(case / "out", platform, stage / "plugins", diagnostics)
                self.assertIsNotNone(handoff, diagnostics)
                self.assertEqual(handoff["materialized_packages"][0]["loadable_artifacts"], expected)
            with self.subTest(role="optional signer selection"):
                diagnostics = []
                signable = native_dynamic_signable_artifacts(
                    plugin, {Path(library).suffix}, diagnostics,
                    operation="signing", package_id="animation",
                )
                self.assertEqual(signable, [plugin / "native" / library], diagnostics)

    def _stage(self, root, platform, library, symbol):
        package = root / "input/animation"
        debug_path = package / "native" / symbol
        debug_path.parent.mkdir(parents=True)
        debug_path.write_bytes(b"debug symbols")
        if library is not None:
            (package / "native" / library).write_bytes(b"library")
        (package / "plugin.toml").write_bytes(b'id = "animation"\n')
        out = root / "out"
        _write_validate_report_with_native_dynamic_exports(out, profile=platform, target_platform=platform)
        receipt_path = out / "stages/validate/report.json"
        receipt = json.loads(receipt_path.read_bytes())
        receipt["schema_version"] = 2
        receipt_path.write_text(json.dumps(receipt), encoding="utf-8")
        with (
            redirect_stdout(io.StringIO()),
            patch.dict(os.environ, {"ZIRCON_NATIVE_DYNAMIC_CAS_ROOT": ""}),
            patch("subprocess.Popen", side_effect=AssertionError("unexpected subprocess")),
        ):
            status = export_main([
                "--profile", platform, "--stage", "native_dynamic", "--out", str(out),
                "--native-plugin-root", str(package.parent),
            ])
        stage = out / "stages/native_dynamic"
        return status, json.loads((stage / "report.json").read_bytes()), stage


if __name__ == "__main__":
    unittest.main()
