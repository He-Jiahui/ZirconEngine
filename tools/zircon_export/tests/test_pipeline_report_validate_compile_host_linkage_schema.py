from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.export_test_support import (
    _compile_host_plan,
    _write_compile_host_report,
    _write_pack_report,
    _write_stage_report,
    _write_validate_report_with_strategies,
)


class PipelineReportValidateCompileHostLinkageSchemaTests(unittest.TestCase):
    def test_report_stage_rejects_validate_compile_host_missing_expected_plugin_provider(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["library_embed"])
            validate_report_path = out / "stages" / "validate" / "report.json"
            validate_report = json.loads(validate_report_path.read_text(encoding="utf-8"))
            compile_host_plan = _compile_host_plan()
            compile_host_plan["expected_runtime_plugins"] = ["rendering"]
            compile_host_plan["linked_runtime_crates"] = []
            validate_report["plan_summary"][
                "library_embed_compile_host"
            ] = compile_host_plan
            validate_report_path.write_text(
                json.dumps(validate_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self._assert_validate_fatal(report)
            self._assert_diagnostic_contains(
                report,
                "validate report plan_summary.library_embed_compile_host."
                "linked_runtime_crates must include provider_package_id "
                "rendering for expected_runtime_plugins[0]",
            )

    def test_report_stage_rejects_compile_host_link_plan_missing_expected_plugin_provider(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            linked_crate = self._linked_crate()
            _write_validate_report_with_strategies(out, ["library_embed"])
            _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, out / "pack-output" / "assets.zrpack")
            _write_stage_report(out, "platform_bundle", fatal=False)

            validate_report_path = out / "stages" / "validate" / "report.json"
            validate_report = json.loads(validate_report_path.read_text(encoding="utf-8"))
            compile_host_plan = _compile_host_plan()
            compile_host_plan["expected_runtime_plugins"] = ["rendering"]
            compile_host_plan["linked_runtime_crates"] = [linked_crate]
            validate_report["plan_summary"][
                "library_embed_compile_host"
            ] = compile_host_plan
            validate_report_path.write_text(
                json.dumps(validate_report, indent=2),
                encoding="utf-8",
            )

            compile_host_report_path = out / "stages" / "compile_host" / "report.json"
            compile_host_report = json.loads(
                compile_host_report_path.read_text(encoding="utf-8")
            )
            compile_host_report["link_plan"]["expected_runtime_plugins"] = ["rendering"]
            compile_host_report["link_plan"]["linked_runtime_crates"] = []
            compile_host_report_path.write_text(
                json.dumps(compile_host_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("CompileHost", report["fatal_stages"])
            self._assert_diagnostic_contains(
                report,
                "compile_host report link_plan.linked_runtime_crates must "
                "include provider_package_id rendering for "
                "expected_runtime_plugins[0]",
            )

    def test_report_stage_rejects_validate_compile_host_linked_crate_non_object(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            self._write_validate_report_with_linked_crates(
                out,
                ["zircon_plugin_rendering_runtime"],
            )

            report = build_pipeline_report(out, "windows-release")

            self._assert_validate_fatal(report)
            self._assert_diagnostic_contains(
                report,
                "validate report plan_summary.library_embed_compile_host."
                "linked_runtime_crates[0] must be an object",
            )

    def test_report_stage_rejects_validate_compile_host_linked_crate_unknown_field(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            self._write_validate_report_with_linked_crates(
                out,
                [
                    {
                        "crate_name": "zircon_plugin_rendering_runtime",
                        "path": "zircon_plugins/rendering/runtime",
                        "registration_kind": "runtime_plugin",
                        "provider_package_id": "rendering",
                        "unsigned_sidecar": "sidecar.bin",
                    }
                ],
            )

            report = build_pipeline_report(out, "windows-release")

            self._assert_validate_fatal(report)
            self._assert_diagnostic_contains(
                report,
                "validate report plan_summary.library_embed_compile_host."
                "linked_runtime_crates[0] unknown field unsigned_sidecar",
            )

    def test_report_stage_rejects_validate_compile_host_linked_crate_string_fields_non_string(
        self,
    ) -> None:
        linked_crate_string_fields = (
            "crate_name",
            "path",
            "provider_package_id",
            "registration_kind",
        )
        for field in linked_crate_string_fields:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    linked_crate = self._linked_crate()
                    linked_crate[field] = 42
                    self._write_validate_report_with_linked_crates(
                        out,
                        [linked_crate],
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self._assert_validate_fatal(report)
                    self._assert_diagnostic_contains(
                        report,
                        "validate report plan_summary.library_embed_compile_host."
                        f"linked_runtime_crates[0].{field} must be a string",
                    )

    def test_report_stage_rejects_validate_compile_host_linked_crate_path_invalid(
        self,
    ) -> None:
        cases = (
            (
                "",
                "validate report plan_summary.library_embed_compile_host."
                "linked_runtime_crates[0].path must be a non-empty trimmed string",
            ),
            (
                "   ",
                "validate report plan_summary.library_embed_compile_host."
                "linked_runtime_crates[0].path must be a non-empty trimmed string",
            ),
            (
                " zircon_plugins/rendering/runtime ",
                "validate report plan_summary.library_embed_compile_host."
                "linked_runtime_crates[0].path must be a non-empty trimmed string",
            ),
            (
                "../rendering/runtime",
                "validate report plan_summary.library_embed_compile_host."
                "linked_runtime_crates[0].path must be a safe relative path",
            ),
            (
                "/zircon_plugins/rendering",
                "validate report plan_summary.library_embed_compile_host."
                "linked_runtime_crates[0].path must be a safe relative path",
            ),
        )
        for path, expected_diagnostic in cases:
            with self.subTest(path=path):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    linked_crate = self._linked_crate()
                    linked_crate["path"] = path
                    self._write_validate_report_with_linked_crates(
                        out,
                        [linked_crate],
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self._assert_diagnostic_contains(report, expected_diagnostic)

    def test_report_stage_rejects_validate_compile_host_linked_crate_missing_field(
        self,
    ) -> None:
        linked_crate_string_fields = (
            "crate_name",
            "path",
            "provider_package_id",
            "registration_kind",
        )
        for field in linked_crate_string_fields:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    linked_crate = self._linked_crate()
                    linked_crate.pop(field)
                    self._write_validate_report_with_linked_crates(
                        out,
                        [linked_crate],
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self._assert_validate_fatal(report)
                    self._assert_diagnostic_contains(
                        report,
                        "validate report plan_summary.library_embed_compile_host."
                        f"linked_runtime_crates[0].{field} must be a string",
                    )

    def test_report_stage_rejects_compile_host_linked_crate_missing_field(
        self,
    ) -> None:
        linked_crate_string_fields = (
            "crate_name",
            "path",
            "provider_package_id",
            "registration_kind",
        )
        for field in linked_crate_string_fields:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    linked_crate = self._linked_crate()
                    self._write_validate_report_with_linked_crates(
                        out,
                        [linked_crate],
                    )
                    linked_crate.pop(field)
                    self._write_compile_host_report_with_linked_crates(
                        out,
                        [linked_crate],
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertIn("CompileHost", report["fatal_stages"])
                    self._assert_diagnostic_contains(
                        report,
                        "compile_host report link_plan.linked_runtime_crates[0]."
                        f"{field} must be a string",
                    )

    def test_report_stage_rejects_compile_host_linked_crate_path_invalid(
        self,
    ) -> None:
        cases = (
            (
                "",
                "compile_host report link_plan.linked_runtime_crates[0]."
                "path must be a non-empty trimmed string",
            ),
            (
                "   ",
                "compile_host report link_plan.linked_runtime_crates[0]."
                "path must be a non-empty trimmed string",
            ),
            (
                " zircon_plugins/rendering/runtime ",
                "compile_host report link_plan.linked_runtime_crates[0]."
                "path must be a non-empty trimmed string",
            ),
            (
                "../rendering/runtime",
                "compile_host report link_plan.linked_runtime_crates[0]."
                "path must be a safe relative path",
            ),
            (
                "/zircon_plugins/rendering",
                "compile_host report link_plan.linked_runtime_crates[0]."
                "path must be a safe relative path",
            ),
        )
        for path, expected_diagnostic in cases:
            with self.subTest(path=path):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    self._write_validate_report_with_linked_crates(
                        out,
                        [self._linked_crate()],
                    )
                    linked_crate = self._linked_crate()
                    linked_crate["path"] = path
                    self._write_compile_host_report_with_linked_crates(
                        out,
                        [linked_crate],
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self._assert_diagnostic_contains(report, expected_diagnostic)

    def test_report_stage_rejects_validate_compile_host_linked_crate_names_invalid(
        self,
    ) -> None:
        cases = (
            (
                "",
                (
                    "validate report plan_summary.library_embed_compile_host."
                    "linked_runtime_crates[0].crate_name must be a non-empty "
                    "trimmed runtime crate name"
                ),
            ),
            (
                "zircon_plugin_rendering_runtime ",
                (
                    "validate report plan_summary.library_embed_compile_host."
                    "linked_runtime_crates[0].crate_name must be a non-empty "
                    "trimmed runtime crate name"
                ),
            ),
            (
                "rendering_runtime",
                (
                    "validate report plan_summary.library_embed_compile_host."
                    "linked_runtime_crates[0].crate_name must use "
                    "zircon_plugin_ crate prefix or builtin_ runtime-domain "
                    "prefix and contain only lowercase ASCII letters, digits, "
                    "and underscores"
                ),
            ),
            (
                "zircon-plugin-rendering-runtime",
                (
                    "validate report plan_summary.library_embed_compile_host."
                    "linked_runtime_crates[0].crate_name must use "
                    "zircon_plugin_ crate prefix or builtin_ runtime-domain "
                    "prefix and contain only lowercase ASCII letters, digits, "
                    "and underscores"
                ),
            ),
            (
                "zircon_plugin_rendering__",
                (
                    "validate report plan_summary.library_embed_compile_host."
                    "linked_runtime_crates[0].crate_name must not end with an "
                    "underscore or contain repeated underscores"
                ),
            ),
        )
        for crate_name, expected_diagnostic in cases:
            with self.subTest(crate_name=crate_name):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    linked_crate = self._linked_crate()
                    linked_crate["crate_name"] = crate_name
                    self._write_validate_report_with_linked_crates(
                        out,
                        [linked_crate],
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self._assert_validate_fatal(report)
                    self._assert_diagnostic_contains(report, expected_diagnostic)

    def test_report_stage_rejects_validate_compile_host_linked_crate_provider_ids_invalid(
        self,
    ) -> None:
        cases = (
            (
                "",
                (
                    "validate report plan_summary.library_embed_compile_host."
                    "linked_runtime_crates[0].provider_package_id must be a "
                    "non-empty trimmed project plugin id"
                ),
            ),
            (
                "rendering ",
                (
                    "validate report plan_summary.library_embed_compile_host."
                    "linked_runtime_crates[0].provider_package_id must be a "
                    "non-empty trimmed project plugin id"
                ),
            ),
            (
                "Rendering",
                (
                    "validate report plan_summary.library_embed_compile_host."
                    "linked_runtime_crates[0].provider_package_id must start "
                    "with a lowercase ASCII letter"
                ),
            ),
            (
                "rendering-plugin",
                (
                    "validate report plan_summary.library_embed_compile_host."
                    "linked_runtime_crates[0].provider_package_id must contain "
                    "only lowercase ASCII letters, digits, and underscores"
                ),
            ),
            (
                "rendering__",
                (
                    "validate report plan_summary.library_embed_compile_host."
                    "linked_runtime_crates[0].provider_package_id must not end "
                    "with an underscore or contain repeated underscores"
                ),
            ),
        )
        for provider_package_id, expected_diagnostic in cases:
            with self.subTest(provider_package_id=provider_package_id):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    linked_crate = self._linked_crate()
                    linked_crate["provider_package_id"] = provider_package_id
                    self._write_validate_report_with_linked_crates(
                        out,
                        [linked_crate],
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self._assert_validate_fatal(report)
                    self._assert_diagnostic_contains(report, expected_diagnostic)

    def test_report_stage_rejects_validate_compile_host_duplicate_linked_crate_name(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            first_linked_crate = self._linked_crate()
            duplicate_linked_crate = self._linked_crate()
            duplicate_linked_crate["path"] = "zircon_plugins/rendering/runtime_copy"
            self._write_validate_report_with_linked_crates(
                out,
                [first_linked_crate, duplicate_linked_crate],
            )

            report = build_pipeline_report(out, "windows-release")

            self._assert_validate_fatal(report)
            self._assert_diagnostic_contains(
                report,
                "validate report plan_summary.library_embed_compile_host."
                "linked_runtime_crates[1].crate_name duplicates entry 0",
            )

    def test_report_stage_rejects_validate_compile_host_linked_crate_identity_mismatch(
        self,
    ) -> None:
        cases = (
            (
                "crate_name",
                "zircon_plugin_physics_runtime",
                "validate report plan_summary.library_embed_compile_host."
                "linked_runtime_crates[0].crate_name must match "
                "provider_package_id rendering as zircon_plugin_rendering_runtime",
            ),
            (
                "path",
                "zircon_plugins/physics/runtime",
                "validate report plan_summary.library_embed_compile_host."
                "linked_runtime_crates[0].path must match "
                "provider_package_id rendering",
            ),
        )
        for field, value, expected_diagnostic in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    linked_crate = self._linked_crate()
                    linked_crate[field] = value
                    self._write_validate_report_with_linked_crates(
                        out,
                        [linked_crate],
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self._assert_validate_fatal(report)
                    self._assert_diagnostic_contains(report, expected_diagnostic)

    def test_report_stage_rejects_validate_compile_host_unexpected_linked_crate_provider(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            self._write_validate_report_with_linked_crates(
                out,
                [self._linked_crate(), self._linked_crate("physics")],
                expected_runtime_plugins=["rendering"],
            )

            report = build_pipeline_report(out, "windows-release")

            self._assert_validate_fatal(report)
            self._assert_diagnostic_contains(
                report,
                "validate report plan_summary.library_embed_compile_host."
                "linked_runtime_crates[1].provider_package_id must be listed "
                "in expected_runtime_plugins",
            )

    def test_report_stage_rejects_compile_host_duplicate_linked_crate_name(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            first_linked_crate = self._linked_crate()
            duplicate_linked_crate = self._linked_crate()
            duplicate_linked_crate["path"] = "zircon_plugins/rendering/runtime_copy"
            self._write_validate_report_with_linked_crates(
                out,
                [first_linked_crate],
            )
            self._write_compile_host_report_with_linked_crates(
                out,
                [first_linked_crate, duplicate_linked_crate],
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("CompileHost", report["fatal_stages"])
            self._assert_diagnostic_contains(
                report,
                "compile_host report link_plan.linked_runtime_crates[1]."
                "crate_name duplicates entry 0",
            )

    def test_report_stage_rejects_compile_host_linked_crate_identity_mismatch(
        self,
    ) -> None:
        cases = (
            (
                "crate_name",
                "zircon_plugin_physics_runtime",
                "compile_host report link_plan.linked_runtime_crates[0]."
                "crate_name must match provider_package_id rendering as "
                "zircon_plugin_rendering_runtime",
            ),
            (
                "path",
                "zircon_plugins/physics/runtime",
                "compile_host report link_plan.linked_runtime_crates[0]."
                "path must match provider_package_id rendering",
            ),
        )
        for field, value, expected_diagnostic in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    self._write_validate_report_with_linked_crates(
                        out,
                        [self._linked_crate()],
                    )
                    linked_crate = self._linked_crate()
                    linked_crate[field] = value
                    self._write_compile_host_report_with_linked_crates(
                        out,
                        [linked_crate],
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertIn("CompileHost", report["fatal_stages"])
                    self._assert_diagnostic_contains(report, expected_diagnostic)

    def test_report_stage_rejects_compile_host_link_plan_unexpected_linked_crate_provider(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            self._write_validate_report_with_linked_crates(
                out,
                [self._linked_crate()],
                expected_runtime_plugins=["rendering"],
            )
            self._write_compile_host_report_with_linked_crates(
                out,
                [self._linked_crate(), self._linked_crate("physics")],
                expected_runtime_plugins=["rendering"],
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("CompileHost", report["fatal_stages"])
            self._assert_diagnostic_contains(
                report,
                "compile_host report link_plan.linked_runtime_crates[1]."
                "provider_package_id must be listed in expected_runtime_plugins",
            )

    def test_report_stage_rejects_validate_compile_host_linked_crate_registration_kind_invalid(
        self,
    ) -> None:
        cases = (
            (
                "",
                "validate report plan_summary.library_embed_compile_host."
                "linked_runtime_crates[0].registration_kind must be a "
                "non-empty trimmed string",
            ),
            (
                "runtime_plugin ",
                "validate report plan_summary.library_embed_compile_host."
                "linked_runtime_crates[0].registration_kind must be a "
                "non-empty trimmed string",
            ),
            (
                " runtime_plugin",
                "validate report plan_summary.library_embed_compile_host."
                "linked_runtime_crates[0].registration_kind must be a "
                "non-empty trimmed string",
            ),
            (
                "runtime_feature",
                "validate report plan_summary.library_embed_compile_host."
                "linked_runtime_crates[0].registration_kind must be "
                "runtime_plugin",
            ),
            (
                "native_dynamic",
                "validate report plan_summary.library_embed_compile_host."
                "linked_runtime_crates[0].registration_kind must be "
                "runtime_plugin",
            ),
        )
        for registration_kind, expected_diagnostic in cases:
            with self.subTest(registration_kind=registration_kind):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    linked_crate = self._linked_crate()
                    linked_crate["registration_kind"] = registration_kind
                    self._write_validate_report_with_linked_crates(
                        out,
                        [linked_crate],
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self._assert_validate_fatal(report)
                    self._assert_diagnostic_contains(
                        report,
                        expected_diagnostic,
                    )

    def test_report_stage_rejects_compile_host_linked_crate_registration_kind_padded(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            self._write_validate_report_with_linked_crates(
                out,
                [self._linked_crate()],
            )
            linked_crate = self._linked_crate()
            linked_crate["registration_kind"] = " runtime_plugin "
            self._write_compile_host_report_with_linked_crates(
                out,
                [linked_crate],
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("CompileHost", report["fatal_stages"])
            self._assert_diagnostic_contains(
                report,
                "compile_host report link_plan.linked_runtime_crates[0]."
                "registration_kind must be a non-empty trimmed string",
            )

    def _write_validate_report_with_linked_crates(
        self,
        out: Path,
        linked_runtime_crates: list[object],
        *,
        expected_runtime_plugins: list[str] | None = None,
    ) -> None:
        _write_validate_report_with_strategies(out, ["library_embed"])
        _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
        _write_stage_report(out, "cook_assets", fatal=False)
        _write_pack_report(out, out / "pack-output" / "assets.zrpack")
        _write_stage_report(out, "platform_bundle", fatal=False)
        validate_report_path = out / "stages" / "validate" / "report.json"
        validate_report = json.loads(validate_report_path.read_text(encoding="utf-8"))
        compile_host_plan = _compile_host_plan()
        compile_host_plan["linked_runtime_crates"] = linked_runtime_crates
        compile_host_plan["expected_runtime_plugins"] = (
            self._expected_runtime_plugins_for_linked_crates(linked_runtime_crates)
            if expected_runtime_plugins is None
            else expected_runtime_plugins
        )
        validate_report["plan_summary"][
            "library_embed_compile_host"
        ] = compile_host_plan
        validate_report_path.write_text(
            json.dumps(validate_report, indent=2),
            encoding="utf-8",
        )

    def _write_compile_host_report_with_linked_crates(
        self,
        out: Path,
        linked_runtime_crates: list[object],
        *,
        expected_runtime_plugins: list[str] | None = None,
    ) -> None:
        compile_host_report_path = out / "stages" / "compile_host" / "report.json"
        compile_host_report = json.loads(
            compile_host_report_path.read_text(encoding="utf-8")
        )
        compile_host_report["link_plan"]["linked_runtime_crates"] = (
            linked_runtime_crates
        )
        if expected_runtime_plugins is not None:
            compile_host_report["link_plan"][
                "expected_runtime_plugins"
            ] = expected_runtime_plugins
        compile_host_report_path.write_text(
            json.dumps(compile_host_report, indent=2),
            encoding="utf-8",
        )

    def _linked_crate(self, provider_package_id: str = "rendering") -> dict[str, object]:
        return {
            "crate_name": f"zircon_plugin_{provider_package_id}_runtime",
            "path": f"zircon_plugins/{provider_package_id}/runtime",
            "registration_kind": "runtime_plugin",
            "provider_package_id": provider_package_id,
        }

    def _expected_runtime_plugins_for_linked_crates(
        self,
        linked_runtime_crates: list[object],
    ) -> list[str]:
        provider_ids: list[str] = []
        for linked_crate in linked_runtime_crates:
            if not isinstance(linked_crate, dict):
                continue
            provider_package_id = linked_crate.get("provider_package_id")
            if not isinstance(provider_package_id, str):
                continue
            if provider_package_id not in provider_ids:
                provider_ids.append(provider_package_id)
        return provider_ids

    def _assert_validate_fatal(self, report: dict[str, object]) -> None:
        self.assertTrue(report["fatal"])
        self.assertEqual(report["missing_stages"], [])
        self.assertEqual(report["fatal_stages"], ["Validate"])

    def _assert_diagnostic_contains(
        self,
        report: dict[str, object],
        expected_diagnostic: str,
    ) -> None:
        self.assertTrue(
            any(
                expected_diagnostic in diagnostic
                for diagnostic in report["diagnostics"]
            ),
            report["diagnostics"],
        )


if __name__ == "__main__":
    unittest.main()
