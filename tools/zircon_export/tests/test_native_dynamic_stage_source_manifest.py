from __future__ import annotations

import tempfile
import tomllib
import unittest
from pathlib import Path

from tools.zircon_export.tests.export_test_support import (
    _export_args,
    _file_sha256,
    _run_stage_quiet,
    _write_validate_report_with_native_dynamic_exports,
    json_dumps,
    json_loads,
)
from tools.zircon_export.tests.native_dynamic_export_test_support import (
    _native_dynamic_content_hash,
    _native_dynamic_package_export,
    _native_dynamic_package_payload_file_manifest,
    _write_native_dynamic_package_fixture,
)


class NativeDynamicStageSourceManifestTests(unittest.TestCase):
    def test_native_dynamic_stage_rejects_source_manifest_id_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root, package_id="animation")
            animation_manifest = repo_root / "zircon_plugins" / "animation" / "plugin.toml"
            animation_manifest.write_text(
                "\n".join(
                    [
                        'id = "wrong-animation"',
                        'version = "0.1.0"',
                        'default_packaging = ["native_dynamic"]',
                    ]
                ),
                encoding="utf-8",
            )
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(out)
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertTrue(report["payload_cleaned"])
            self.assertEqual(report["loader_manifest"], None)
            self.assertEqual(report["materialized_packages"], [])
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())
            self.assertTrue(
                any(
                    "manifest id wrong-animation does not match selected package animation" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse(
                any(
                    "no plugin.toml was found" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_native_dynamic_stage_rejects_padded_source_manifest_id_before_package_match(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root, package_id="animation")
            animation_manifest = repo_root / "zircon_plugins" / "animation" / "plugin.toml"
            animation_manifest.write_text(
                "\n".join(
                    [
                        'id = " animation "',
                        'version = "0.1.0"',
                        'default_packaging = ["native_dynamic"]',
                    ]
                ),
                encoding="utf-8",
            )
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(out)
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            diagnostics = "\n".join(report["diagnostics"])
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertTrue(report["payload_cleaned"])
            self.assertEqual(report["loader_manifest"], None)
            self.assertEqual(report["materialized_packages"], [])
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())
            self.assertIn(
                "native dynamic package animation direct manifest "
                "id must be a non-empty trimmed string",
                diagnostics,
            )
            self.assertNotIn("does not match selected package", diagnostics)
            self.assertNotIn("no plugin.toml was found", diagnostics)

    def test_native_dynamic_stage_rejects_non_string_source_manifest_id_before_missing_id(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root, package_id="animation")
            animation_manifest = repo_root / "zircon_plugins" / "animation" / "plugin.toml"
            animation_manifest.write_text(
                "\n".join(
                    [
                        "id = 42",
                        'version = "0.1.0"',
                        'default_packaging = ["native_dynamic"]',
                    ]
                ),
                encoding="utf-8",
            )
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(out)
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            diagnostics = "\n".join(report["diagnostics"])
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertTrue(report["payload_cleaned"])
            self.assertEqual(report["loader_manifest"], None)
            self.assertEqual(report["materialized_packages"], [])
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())
            self.assertIn(
                "native dynamic package animation direct manifest id must be a string",
                diagnostics,
            )
            self.assertNotIn("direct manifest id must be a non-empty string", diagnostics)
            self.assertNotIn("no plugin.toml was found", diagnostics)

    def test_native_dynamic_stage_rejects_padded_recursive_source_manifest_id_before_missing_manifest(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            nested_package = repo_root / "zircon_plugins" / "nested" / "animation"
            nested_package.mkdir(parents=True)
            nested_package.joinpath("plugin.toml").write_text(
                "\n".join(
                    [
                        'id = " animation "',
                        'version = "0.1.0"',
                        'default_packaging = ["native_dynamic"]',
                    ]
                ),
                encoding="utf-8",
            )
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(out)
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            diagnostics = "\n".join(report["diagnostics"])
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertTrue(report["payload_cleaned"])
            self.assertEqual(report["loader_manifest"], None)
            self.assertEqual(report["materialized_packages"], [])
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())
            self.assertIn(
                "native dynamic package animation source manifest",
                diagnostics,
            )
            self.assertIn("id must be a non-empty trimmed string", diagnostics)
            self.assertNotIn("does not match selected package", diagnostics)
            self.assertNotIn("no plugin.toml was found", diagnostics)

    def test_native_dynamic_stage_rejects_recursive_source_manifest_parse_error_before_missing_manifest(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            nested_package = repo_root / "zircon_plugins" / "nested" / "animation"
            nested_package.mkdir(parents=True)
            nested_package.joinpath("plugin.toml").write_text(
                'id = "animation"\n[broken\n',
                encoding="utf-8",
            )
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(out)
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            diagnostics = "\n".join(report["diagnostics"])
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertTrue(report["payload_cleaned"])
            self.assertEqual(report["loader_manifest"], None)
            self.assertEqual(report["materialized_packages"], [])
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())
            self.assertIn(
                "native dynamic package animation source manifest",
                diagnostics,
            )
            self.assertIn("could not be parsed", diagnostics)
            self.assertNotIn("no plugin.toml was found", diagnostics)

    def test_native_dynamic_stage_rejects_source_manifest_parse_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root, package_id="animation")
            animation_manifest = repo_root / "zircon_plugins" / "animation" / "plugin.toml"
            animation_manifest.write_text(
                'id = "animation"\n[broken\n',
                encoding="utf-8",
            )
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(out)
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertTrue(report["payload_cleaned"])
            self.assertEqual(report["loader_manifest"], None)
            self.assertEqual(report["materialized_packages"], [])
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())
            self.assertTrue(
                any(
                    "direct manifest could not be parsed" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse(
                any(
                    "no plugin.toml was found" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_native_dynamic_stage_rejects_source_manifest_directory(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root, package_id="animation")
            animation_manifest = repo_root / "zircon_plugins" / "animation" / "plugin.toml"
            animation_manifest.unlink()
            animation_manifest.mkdir()
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(out)

            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(report["payload_cleaned"])
            self.assertEqual(report["loader_manifest"], None)
            self.assertEqual(report["materialized_packages"], [])
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())
            self.assertTrue(
                any(
                    "direct manifest"
                    in diagnostic
                    and "is not a file" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse(
                any(
                    "no plugin.toml was found" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_native_dynamic_stage_rejects_source_manifest_missing_id(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root, package_id="animation")
            animation_manifest = repo_root / "zircon_plugins" / "animation" / "plugin.toml"
            animation_manifest.write_text(
                "\n".join(
                    [
                        'version = "0.1.0"',
                        'default_packaging = ["native_dynamic"]',
                    ]
                ),
                encoding="utf-8",
            )
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(out)
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertTrue(report["payload_cleaned"])
            self.assertEqual(report["loader_manifest"], None)
            self.assertEqual(report["materialized_packages"], [])
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())
            self.assertTrue(
                any(
                    "direct manifest id must be a non-empty string" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse(
                any(
                    "no plugin.toml was found" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )


if __name__ == "__main__":
    unittest.main()
