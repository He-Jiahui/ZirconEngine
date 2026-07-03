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


class NativeDynamicStageSelectionStrategyTests(unittest.TestCase):
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

    def test_native_dynamic_stage_rejects_padded_selected_package_id_before_uniqueness(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root)
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(
                out,
                native_dynamic_packages=[" animation ", " animation "],
            )
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
                    "native_dynamic_packages entry 0 "
                    "must be a non-empty trimmed string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse(
                any(
                    "native_dynamic_packages entry  animation  duplicates entry 0"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_native_dynamic_stage_rejects_non_string_selected_package_id_before_array_shape(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root)
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(
                out,
                native_dynamic_packages=[42, "animation"],
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            diagnostics = report["diagnostics"]
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], diagnostics)
            self.assertEqual(report["loader_manifest"], None)
            self.assertEqual(report["materialized_packages"], [])
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())
            self.assertIn(
                "native_dynamic_packages entry 0 must be a string",
                diagnostics,
            )
            self.assertNotIn(
                "validate report native_dynamic_packages must be a string array",
                diagnostics,
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

    def test_native_dynamic_stage_rejects_padded_target_platform_before_artifact_selection(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root)
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(out)
            validate_report = out / "stages" / "validate" / "report.json"
            payload = json_loads(validate_report.read_text(encoding="utf-8"))
            payload["profile_summary"]["target_platform"] = " windows-x86_64 "
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
                    "validate report profile_summary.target_platform "
                    "must be a non-empty trimmed export target platform"
                    in diagnostic
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
