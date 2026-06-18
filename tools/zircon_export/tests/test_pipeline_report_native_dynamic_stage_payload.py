from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.export_test_support import (
    _native_dynamic_content_hash,
    _native_dynamic_package_export,
    _native_dynamic_package_payload_file_manifest,
    _native_dynamic_plugins_file_manifest,
    _write_compile_host_report,
    _write_native_dynamic_report,
    _write_native_dynamic_package_report,
    _write_native_dynamic_stage_plugins,
    _write_pack_report,
    _write_stage_report,
    _write_validate_report_with_native_dynamic_exports,
)


class PipelineReportNativeDynamicStagePayloadTests(unittest.TestCase):
    def _write_native_dynamic_reports(self, out: Path) -> Path:
        _write_validate_report_with_native_dynamic_exports(out)
        native_plugins = _write_native_dynamic_stage_plugins(
            out / "stages" / "native_dynamic"
        )
        _write_native_dynamic_report(out, native_plugins)
        _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
        _write_stage_report(out, "cook_assets", fatal=False)
        _write_pack_report(out, out / "stages" / "pack" / "assets.zrpack")
        _write_stage_report(out, "platform_bundle", fatal=False)
        return out / "stages" / "native_dynamic" / "report.json"

    def _rewrite_loader_manifest(
        self,
        native_report_path: Path,
        loader_manifest: str,
    ) -> None:
        current_plugins = native_report_path.parent / "plugins"
        current_plugins.joinpath("native_plugins.toml").write_text(
            loader_manifest,
            encoding="utf-8",
        )
        native_report = json.loads(native_report_path.read_text(encoding="utf-8"))
        file_manifest = _native_dynamic_plugins_file_manifest(current_plugins)
        native_report["file_manifest"] = file_manifest
        native_report["content_hash"] = _native_dynamic_content_hash(file_manifest)
        native_report["package_exports"] = [_native_dynamic_package_export()]
        native_report_path.write_text(
            json.dumps(native_report, indent=2),
            encoding="utf-8",
        )

    def _refresh_native_dynamic_report_payload(
        self,
        native_report_path: Path,
    ) -> None:
        current_plugins = native_report_path.parent / "plugins"
        native_report = json.loads(native_report_path.read_text(encoding="utf-8"))
        file_manifest = _native_dynamic_plugins_file_manifest(current_plugins)
        native_report["file_manifest"] = file_manifest
        native_report["content_hash"] = _native_dynamic_content_hash(file_manifest)
        native_report_path.write_text(
            json.dumps(native_report, indent=2),
            encoding="utf-8",
        )

    def test_report_stage_rejects_native_dynamic_loader_manifest_path_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = self._write_native_dynamic_reports(out)
            self._rewrite_loader_manifest(
                native_report_path,
                _loader_manifest_with_overrides(path="plugins/forged"),
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("NativeDynamic", report["fatal_stages"])
            self.assertTrue(
                any(
                    "native_dynamic loader_manifest plugin animation path "
                    "plugins/forged does not match native_dynamic report "
                    "package_exports path plugins/animation"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_loader_manifest_abi_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = self._write_native_dynamic_reports(out)
            self._rewrite_loader_manifest(
                native_report_path,
                _loader_manifest_with_overrides(
                    abi_overrides={
                        "descriptor_symbol": "zircon_native_plugin_descriptor_v2"
                    },
                ),
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("NativeDynamic", report["fatal_stages"])
            self.assertTrue(
                any(
                    "native_dynamic loader_manifest plugin animation "
                    "abi.descriptor_symbol zircon_native_plugin_descriptor_v2 "
                    "does not match native_dynamic report package_exports "
                    "abi.descriptor_symbol zircon_native_plugin_descriptor_v3"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_loader_manifest_missing_plugins_table(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = self._write_native_dynamic_reports(out)
            self._rewrite_loader_manifest(
                native_report_path,
                '[metadata]\nname = "animation"\n',
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("NativeDynamic", report["fatal_stages"])
            self.assertTrue(
                any(
                    "native_dynamic loader_manifest plugins must be an array"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_loader_manifest_bad_plugin_id(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = self._write_native_dynamic_reports(out)
            self._rewrite_loader_manifest(
                native_report_path,
                "[[plugins]]\nid = 42\n",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("NativeDynamic", report["fatal_stages"])
            self.assertTrue(
                any(
                    "native_dynamic loader_manifest plugins[0].id "
                    "must be a non-empty string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_package_report_id_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = self._write_native_dynamic_reports(out)
            package_report = (
                native_report_path.parent
                / "plugins"
                / "animation"
                / "native_dynamic_package.toml"
            )
            package_report.write_text(
                "\n".join(
                    [
                        "format_version = 1",
                        'package_id = "physics"',
                        'directory = "animation"',
                        'path = "plugins/animation"',
                        'manifest = "plugins/animation/plugin.toml"',
                    ]
                )
                + "\n",
                encoding="utf-8",
            )
            self._refresh_native_dynamic_report_payload(native_report_path)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("NativeDynamic", report["fatal_stages"])
            self.assertTrue(
                any(
                    "native_dynamic report materialized_packages[0] "
                    "package_report package_id physics does not match "
                    "materialized package animation"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_package_report_payload_hash_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = self._write_native_dynamic_reports(out)
            package_report = (
                native_report_path.parent
                / "plugins"
                / "animation"
                / "native_dynamic_package.toml"
            )
            package_report.write_text(
                "\n".join(
                    [
                        "format_version = 1",
                        'package_id = "animation"',
                        'directory = "animation"',
                        'path = "plugins/animation"',
                        'manifest = "plugins/animation/plugin.toml"',
                        "",
                        "[payload]",
                        "file_count = 1",
                        f'content_hash = "{"0" * 64}"',
                    ]
                )
                + "\n",
                encoding="utf-8",
            )
            self._refresh_native_dynamic_report_payload(native_report_path)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("NativeDynamic", report["fatal_stages"])
            self.assertTrue(
                any(
                    "native_dynamic report materialized_packages[0] "
                    "package_report payload content_hash "
                    in diagnostic
                    and "does not match current package payload" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_unreported_loadable_artifact(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = self._write_native_dynamic_reports(out)
            package_dir = native_report_path.parent / "plugins" / "animation"
            package_dir.joinpath(
                "native",
                "zircon_plugin_animation_extra.dll",
            ).write_text(
                "extra loadable",
                encoding="utf-8",
            )
            _write_native_dynamic_package_report(package_dir)
            self._refresh_native_dynamic_report_payload(native_report_path)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("NativeDynamic", report["fatal_stages"])
            self.assertTrue(
                any(
                    "NativeDynamic payload materialized_packages[0] "
                    "loadable_artifacts do not include current loadable "
                    "artifact "
                    "plugins/animation/native/"
                    "zircon_plugin_animation_extra.dll"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_package_export_path_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = self._write_native_dynamic_reports(out)
            plugins_dir = native_report_path.parent / "plugins"
            package_dir = plugins_dir / "animation"
            forged_package_dir = plugins_dir / "forged-animation"
            package_dir.rename(forged_package_dir)
            _write_package_report_for_directory(
                forged_package_dir,
                directory="forged-animation",
            )
            native_report = json.loads(
                native_report_path.read_text(encoding="utf-8")
            )
            materialized_package = native_report["materialized_packages"][0]
            materialized_package["destination"] = str(forged_package_dir)
            materialized_package["package_report"] = str(
                forged_package_dir / "native_dynamic_package.toml"
            )
            materialized_package["loadable_artifacts"] = [
                "plugins/forged-animation/native/zircon_plugin_animation.dll"
            ]
            materialized_package["loadable_artifact_count"] = 1
            native_report["package_exports"] = [_native_dynamic_package_export()]
            file_manifest = _native_dynamic_plugins_file_manifest(plugins_dir)
            native_report["file_manifest"] = file_manifest
            native_report["content_hash"] = _native_dynamic_content_hash(
                file_manifest
            )
            native_report_path.write_text(
                json.dumps(native_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("NativeDynamic", report["fatal_stages"])
            self.assertTrue(
                any(
                    "native_dynamic report package_exports package animation "
                    "path plugins/animation does not match materialized "
                    "package path plugins/forged-animation"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )


def _loader_manifest_with_overrides(
    *,
    path: str = "plugins/animation",
    manifest: str = "plugins/animation/plugin.toml",
    package_report: str = "plugins/animation/native_dynamic_package.toml",
    abi_overrides: dict[str, object] | None = None,
) -> str:
    package_export = _native_dynamic_package_export()
    abi = dict(package_export["abi"])
    if abi_overrides:
        abi.update(abi_overrides)
    return (
        '[[plugins]]\n'
        'id = "animation"\n'
        f'path = "{path}"\n'
        f'manifest = "{manifest}"\n'
        f'package_report = "{package_report}"\n'
        '\n'
        '[plugins.abi]\n'
        f'abi_version = {abi["abi_version"]}\n'
        f'descriptor_symbol = "{abi["descriptor_symbol"]}"\n'
        f'descriptor_contract = "{abi["descriptor_contract"]}"\n'
        f'runtime_entry_source = "{abi["runtime_entry_source"]}"\n'
        f'editor_entry_source = "{abi["editor_entry_source"]}"\n'
        f'host_function_table = "{abi["host_function_table"]}"\n'
        f'entry_report_contract = "{abi["entry_report_contract"]}"\n'
        f'behavior_contract = "{abi["behavior_contract"]}"\n'
        f'state_snapshot_contract = "{abi["state_snapshot_contract"]}"\n'
        f'bridge_method_table = "{abi["bridge_method_table"]}"\n'
    )


def _write_package_report_for_directory(package_dir: Path, *, directory: str) -> None:
    payload_files = _native_dynamic_package_payload_file_manifest(package_dir)
    package_export = _native_dynamic_package_export(
        {
            "directory": directory,
            "path": f"plugins/{directory}",
            "manifest": f"plugins/{directory}/plugin.toml",
            "package_report": f"plugins/{directory}/native_dynamic_package.toml",
        }
    )
    abi = package_export["abi"]
    lines = [
        "# Generated by Zircon export. Native dynamic package report.",
        "format_version = 1",
        'package_id = "animation"',
        f'directory = "{directory}"',
        f'path = "plugins/{directory}"',
        f'manifest = "plugins/{directory}/plugin.toml"',
        "",
        "[abi]",
        f'abi_version = {abi["abi_version"]}',
        f'descriptor_symbol = "{abi["descriptor_symbol"]}"',
        f'descriptor_contract = "{abi["descriptor_contract"]}"',
        f'runtime_entry_source = "{abi["runtime_entry_source"]}"',
        f'editor_entry_source = "{abi["editor_entry_source"]}"',
        f'host_function_table = "{abi["host_function_table"]}"',
        f'entry_report_contract = "{abi["entry_report_contract"]}"',
        f'behavior_contract = "{abi["behavior_contract"]}"',
        f'state_snapshot_contract = "{abi["state_snapshot_contract"]}"',
        f'bridge_method_table = "{abi["bridge_method_table"]}"',
        "",
        "[payload]",
        f"file_count = {len(payload_files)}",
        f'content_hash = "{_native_dynamic_content_hash(payload_files)}"',
    ]
    for entry in payload_files:
        lines.extend(
            [
                "",
                "[[payload.files]]",
                f'path = "{entry["path"]}"',
                f'bytes = {entry["bytes"]}',
                f'sha256 = "{entry["sha256"]}"',
            ]
        )
    package_dir.joinpath("native_dynamic_package.toml").write_text(
        "\n".join(lines) + "\n",
        encoding="utf-8",
    )
