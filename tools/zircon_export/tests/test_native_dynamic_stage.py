from __future__ import annotations

import tempfile
import tomllib
import unittest
from pathlib import Path

from tools.zircon_export.tests.export_test_support import (
    _export_args,
    _file_sha256,
    _native_dynamic_content_hash,
    _native_dynamic_package_export,
    _native_dynamic_package_payload_file_manifest,
    _run_stage_quiet,
    _write_native_dynamic_package_fixture,
    _write_validate_report_with_native_dynamic_exports,
    json_dumps,
    json_loads,
)


class NativeDynamicStageTests(unittest.TestCase):
    def test_native_dynamic_stage_writes_package_export_report(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root)
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(out)
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            report = json_loads(
                (out / "stages" / "native_dynamic" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 0)
            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertEqual(report["stage"], "NativeDynamic")
            self.assertEqual(report["profile"], "windows-release")
            self.assertEqual(report["package_count"], 1)
            self.assertEqual(report["package_exports"][0]["package_id"], "animation")
            self.assertEqual(report["package_exports"][0]["directory"], "animation")
            self.assertEqual(report["package_exports"][0]["path"], "plugins/animation")
            self.assertEqual(report["package_exports"][0]["manifest"], "plugins/animation/plugin.toml")

    def test_native_dynamic_stage_materializes_package_and_loader_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root)
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(out)
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            materialized_package = stage_dir / "plugins" / "animation"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            package_report = (materialized_package / "native_dynamic_package.toml").read_text(
                encoding="utf-8"
            )
            loader_manifest = (stage_dir / "plugins" / "native_plugins.toml").read_text(
                encoding="utf-8"
            )
            self.assertEqual(exit_code, 0)
            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertEqual(report["loader_manifest"], str(stage_dir / "plugins" / "native_plugins.toml"))
            self.assertEqual(len(report["materialized_packages"]), 1)
            self.assertFalse(report["payload_cleaned"])
            self.assertEqual(report["cleanup_reason"], None)
            self.assertTrue((materialized_package / "plugin.toml").exists())
            self.assertTrue((materialized_package / "native" / "zircon_plugin_animation.dll").exists())
            self.assertTrue((materialized_package / "resources" / "animation.asset").exists())
            self.assertFalse((materialized_package / "src" / "lib.rs").exists())
            self.assertIn('package_id = "animation"', package_report)
            self.assertIn('[abi]', package_report)
            self.assertIn('package_report = "plugins/animation/native_dynamic_package.toml"', loader_manifest)
            self.assertIn("[plugins.abi]", loader_manifest)

    def test_native_dynamic_stage_reports_materialized_file_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root)
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(out)
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            file_manifest = report["file_manifest"]
            manifest_paths = [entry["path"] for entry in file_manifest]
            self.assertEqual(exit_code, 0)
            self.assertEqual(
                manifest_paths,
                [
                    "plugins/animation/native/zircon_plugin_animation.dll",
                    "plugins/animation/native_dynamic_package.toml",
                    "plugins/animation/plugin.toml",
                    "plugins/animation/resources/animation.asset",
                    "plugins/native_plugins.toml",
                ],
            )
            self.assertEqual(manifest_paths, sorted(manifest_paths))
            for entry in file_manifest:
                self.assertGreater(entry["bytes"], 0)
                self.assertEqual(len(entry["sha256"]), 64)
                self.assertEqual(entry["sha256"], _file_sha256(stage_dir / entry["path"]))
            self.assertEqual(len(report["content_hash"]), 64)
            self.assertEqual(report["content_hash"], _native_dynamic_content_hash(file_manifest))

    def test_native_dynamic_package_report_records_package_payload_hash(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root)
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(out)
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            package = out / "stages" / "native_dynamic" / "plugins" / "animation"
            package_report_path = package / "native_dynamic_package.toml"
            expected_files = _native_dynamic_package_payload_file_manifest(package)
            with package_report_path.open("rb") as package_report_file:
                package_report = tomllib.load(package_report_file)
            self.assertEqual(exit_code, 0)
            self.assertEqual(package_report["payload"]["file_count"], len(expected_files))
            self.assertEqual(
                package_report["payload"]["content_hash"],
                _native_dynamic_content_hash(expected_files),
            )
            self.assertEqual(package_report["payload"]["files"], expected_files)

    def test_native_dynamic_stage_removes_stale_unselected_packages(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root)
            out = root / "out"
            stale_package = out / "stages" / "native_dynamic" / "plugins" / "stale"
            (stale_package / "native").mkdir(parents=True)
            (stale_package / "plugin.toml").write_text('id = "stale"\n', encoding="utf-8")
            (stale_package / "native" / "zircon_plugin_stale.dll").write_text(
                "stale native payload",
                encoding="utf-8",
            )
            _write_validate_report_with_native_dynamic_exports(out)
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            manifest_paths = [entry["path"] for entry in report["file_manifest"]]
            self.assertEqual(exit_code, 0)
            self.assertFalse(stale_package.exists())
            self.assertFalse(any(path.startswith("plugins/stale/") for path in manifest_paths))
            self.assertTrue((stage_dir / "plugins" / "animation").exists())

    def test_native_dynamic_stage_filters_artifacts_by_target_platform(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root)
            package_native_dir = repo_root / "zircon_plugins" / "animation" / "native"
            (package_native_dir / "zircon_plugin_animation.pdb").write_text(
                "windows debug symbols",
                encoding="utf-8",
            )
            (package_native_dir / "libzircon_plugin_animation.so").write_text(
                "linux dynamic payload",
                encoding="utf-8",
            )
            (package_native_dir / "libzircon_plugin_animation.dbg").write_text(
                "linux debug symbols",
                encoding="utf-8",
            )
            (package_native_dir / "libzircon_plugin_animation.dylib").write_text(
                "macos dynamic payload",
                encoding="utf-8",
            )
            (package_native_dir / "zircon_plugin_animation.dsym").write_text(
                "macos debug symbols",
                encoding="utf-8",
            )
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(out)
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            manifest_paths = [entry["path"] for entry in report["file_manifest"]]
            self.assertEqual(exit_code, 0)
            self.assertIn("plugins/animation/native/zircon_plugin_animation.dll", manifest_paths)
            self.assertIn("plugins/animation/native/zircon_plugin_animation.pdb", manifest_paths)
            self.assertNotIn("plugins/animation/native/libzircon_plugin_animation.so", manifest_paths)
            self.assertNotIn("plugins/animation/native/libzircon_plugin_animation.dbg", manifest_paths)
            self.assertNotIn("plugins/animation/native/libzircon_plugin_animation.dylib", manifest_paths)
            self.assertNotIn("plugins/animation/native/zircon_plugin_animation.dsym", manifest_paths)

    def test_native_dynamic_stage_requires_platform_loadable_artifact(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            package = repo_root / "zircon_plugins" / "animation"
            (package / "native").mkdir(parents=True)
            (package / "resources").mkdir()
            (package / "plugin.toml").write_text('id = "animation"\n', encoding="utf-8")
            (package / "native" / "zircon_plugin_animation.pdb").write_text(
                "windows debug symbols without runtime library",
                encoding="utf-8",
            )
            (package / "resources" / "animation.asset").write_text(
                "asset",
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
            self.assertEqual(report["loader_manifest"], None)
            self.assertEqual(report["materialized_packages"], [])
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())
            self.assertFalse((stage_dir / "plugins" / "animation").exists())
            self.assertTrue(
                any(
                    "has no loadable native library artifacts" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_native_dynamic_stage_removes_partial_package_on_artifact_filter_fatal(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            package = repo_root / "zircon_plugins" / "animation"
            (package / "native").mkdir(parents=True)
            (package / "resources").mkdir()
            (package / "plugin.toml").write_text('id = "animation"\n', encoding="utf-8")
            (package / "native" / "libzircon_plugin_animation.so").write_text(
                "linux payload",
                encoding="utf-8",
            )
            (package / "resources" / "animation.asset").write_text(
                "asset",
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
            self.assertEqual(report["loader_manifest"], None)
            self.assertEqual(report["materialized_packages"], [])
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())
            self.assertFalse((stage_dir / "plugins" / "animation").exists())
            self.assertTrue(
                any(
                    "has no dynamic library artifacts" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_native_dynamic_stage_removes_all_packages_when_any_package_fails(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root, package_id="animation")
            physics = repo_root / "zircon_plugins" / "physics"
            (physics / "native").mkdir(parents=True)
            (physics / "resources").mkdir()
            (physics / "plugin.toml").write_text('id = "physics"\n', encoding="utf-8")
            (physics / "native" / "libzircon_plugin_physics.so").write_text(
                "linux payload",
                encoding="utf-8",
            )
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(
                out,
                native_dynamic_packages=["animation", "physics"],
                extra_package_exports=[
                    _native_dynamic_package_export(
                        {
                            "package_id": "physics",
                            "directory": "physics",
                            "path": "plugins/physics",
                            "manifest": "plugins/physics/plugin.toml",
                            "package_report": "plugins/physics/native_dynamic_package.toml",
                        }
                    )
                ],
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertEqual(report["loader_manifest"], None)
            self.assertEqual(report["materialized_packages"], [])
            self.assertTrue(report["payload_cleaned"])
            self.assertEqual(report["cleanup_reason"], "fatal_diagnostics")
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())
            self.assertFalse((stage_dir / "plugins" / "animation").exists())
            self.assertFalse((stage_dir / "plugins" / "physics").exists())
            self.assertTrue(
                any(
                    "physics" in diagnostic and "has no dynamic library artifacts" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_native_dynamic_stage_rejects_inconsistent_package_paths(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root)
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(
                out,
                package_export_overrides={
                    "path": "plugins/wrong-animation",
                    "manifest": "plugins/wrong-animation/plugin.toml",
                },
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertEqual(report["loader_manifest"], None)
            self.assertEqual(report["materialized_packages"], [])
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())
            self.assertTrue(
                any(
                    "path must be plugins/animation" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_native_dynamic_stage_rejects_inconsistent_package_report_path(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root)
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(
                out,
                package_export_overrides={
                    "package_report": "plugins/wrong-animation/native_dynamic_package.toml",
                },
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertEqual(report["loader_manifest"], None)
            self.assertEqual(report["materialized_packages"], [])
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())
            self.assertTrue(
                any(
                    "package_report must be plugins/animation/native_dynamic_package.toml" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_native_dynamic_stage_derives_missing_package_report_path(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root)
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(out)
            validate_report = out / "stages" / "validate" / "report.json"
            validate_payload = json_loads(validate_report.read_text(encoding="utf-8"))
            validate_payload["plan_summary"]["native_dynamic_package_exports"][0].pop(
                "package_report"
            )
            validate_report.write_text(json_dumps(validate_payload), encoding="utf-8")
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            loader_manifest = (stage_dir / "plugins" / "native_plugins.toml").read_text(
                encoding="utf-8"
            )
            self.assertEqual(exit_code, 0)
            self.assertFalse(report["fatal"])
            self.assertEqual(
                report["package_exports"][0]["package_report"],
                "plugins/animation/native_dynamic_package.toml",
            )
            self.assertIn(
                'package_report = "plugins/animation/native_dynamic_package.toml"',
                loader_manifest,
            )

    def test_native_dynamic_stage_accepts_sanitized_package_directory(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root, package_id="animation.fx")
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(
                out,
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

            stage_dir = out / "stages" / "native_dynamic"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 0)
            self.assertFalse(report["fatal"])
            self.assertTrue((stage_dir / "plugins" / "animation_fx" / "plugin.toml").exists())
            self.assertTrue((stage_dir / "plugins" / "native_plugins.toml").exists())

    def test_native_dynamic_stage_rejects_package_directory_id_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root)
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(
                out,
                package_export_overrides={
                    "directory": "animation_copy",
                    "path": "plugins/animation_copy",
                    "manifest": "plugins/animation_copy/plugin.toml",
                    "package_report": "plugins/animation_copy/native_dynamic_package.toml",
                },
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertEqual(report["loader_manifest"], None)
            self.assertEqual(report["materialized_packages"], [])
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())
            self.assertTrue(
                any(
                    "directory must be animation for package_id animation" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_native_dynamic_stage_rejects_duplicate_package_ids(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root)
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(
                out,
                extra_package_exports=[
                    _native_dynamic_package_export(
                        {
                            "package_id": "animation",
                            "directory": "animation_copy",
                            "path": "plugins/animation_copy",
                            "manifest": "plugins/animation_copy/plugin.toml",
                        }
                    )
                ],
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertEqual(report["loader_manifest"], None)
            self.assertEqual(report["materialized_packages"], [])
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())
            self.assertTrue(
                any(
                    "package_id animation duplicates entry 0" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

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

    def test_native_dynamic_stage_rejects_unselected_package_export(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root)
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(
                out,
                native_dynamic_packages=[],
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertEqual(report["loader_manifest"], None)
            self.assertEqual(report["materialized_packages"], [])
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())
            self.assertTrue(
                any(
                    "package_export animation is not present in native_dynamic_packages" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_native_dynamic_stage_rejects_duplicate_selected_package_ids(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root)
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(
                out,
                native_dynamic_packages=["animation", "animation"],
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertEqual(report["loader_manifest"], None)
            self.assertEqual(report["materialized_packages"], [])
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())
            self.assertTrue(
                any(
                    "native_dynamic_packages entry animation duplicates entry 0" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_native_dynamic_stage_rejects_missing_selected_package_export(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root)
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(
                out,
                native_dynamic_packages=["animation", "physics"],
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertEqual(report["loader_manifest"], None)
            self.assertEqual(report["materialized_packages"], [])
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())
            self.assertTrue(
                any(
                    "native_dynamic_packages entry physics has no package_export" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_native_dynamic_stage_rejects_invalid_validate_metadata(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root)
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(out)
            validate_report = out / "stages" / "validate" / "report.json"
            payload = json_loads(validate_report.read_text(encoding="utf-8"))
            payload["fatal"] = []
            validate_report.write_text(json_dumps(payload), encoding="utf-8")
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertEqual(report["loader_manifest"], None)
            self.assertEqual(report["materialized_packages"], [])
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())
            self.assertTrue(
                any(
                    "Validate report fatal must be a boolean" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_native_dynamic_stage_rejects_validate_report_directory(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root)
            out = root / "out"
            validate_report = out / "stages" / "validate" / "report.json"
            validate_report.mkdir(parents=True)
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            report = json_loads(
                (out / "stages" / "native_dynamic" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    f"validate report {validate_report} is not a file"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_native_dynamic_stage_requires_native_dynamic_strategy(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root)
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(out)
            validate_report = out / "stages" / "validate" / "report.json"
            payload = json_loads(validate_report.read_text(encoding="utf-8"))
            payload["profile_summary"]["strategies"] = ["library_embed"]
            validate_report.write_text(json_dumps(payload), encoding="utf-8")
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["loader_manifest"], None)
            self.assertEqual(report["materialized_packages"], [])
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())
            self.assertTrue(
                any(
                    "NativeDynamic stage requires the native_dynamic strategy"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_native_dynamic_stage_rejects_invalid_strategy_metadata(self) -> None:
        cases = (
            ("native_dynamic", "profile_summary.strategies must be a list"),
            (
                [],
                "profile_summary.strategies must include at least one supported export strategy",
            ),
            (["native_dynamic", "ghost_path"], "unsupported export strategy ghost_path"),
        )
        for strategies, expected_diagnostic in cases:
            with self.subTest(strategies=strategies):
                with tempfile.TemporaryDirectory() as temp_dir:
                    root = Path(temp_dir)
                    repo_root = root / "repo"
                    _write_native_dynamic_package_fixture(repo_root)
                    out = root / "out"
                    _write_validate_report_with_native_dynamic_exports(out)
                    validate_report = out / "stages" / "validate" / "report.json"
                    payload = json_loads(validate_report.read_text(encoding="utf-8"))
                    payload["profile_summary"]["strategies"] = strategies
                    validate_report.write_text(json_dumps(payload), encoding="utf-8")
                    args = _export_args(out=out, stage="native_dynamic", dry_run=False)
                    args.repo_root = str(repo_root)

                    exit_code = _run_stage_quiet(args)

                    stage_dir = out / "stages" / "native_dynamic"
                    report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
                    self.assertEqual(exit_code, 2)
                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertEqual(report["loader_manifest"], None)
                    self.assertEqual(report["materialized_packages"], [])
                    self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())
                    self.assertTrue(
                        any(
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_native_dynamic_stage_reports_missing_package_source_fatal(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            (repo_root / "zircon_plugins").mkdir(parents=True)
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(out)
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            report = json_loads(
                (out / "stages" / "native_dynamic" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertEqual(report["materialized_packages"], [])
            self.assertEqual(report["loader_manifest"], None)
            self.assertFalse(
                (out / "stages" / "native_dynamic" / "plugins" / "native_plugins.toml").exists()
            )
            self.assertTrue(
                any("no plugin.toml was found" in diagnostic for diagnostic in report["diagnostics"]),
                report["diagnostics"],
            )


if __name__ == "__main__":
    unittest.main()
