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

    def test_report_stage_rejects_validate_compile_host_linked_crate_registration_kind_invalid(
        self,
    ) -> None:
        for registration_kind in (
            "",
            "runtime_plugin ",
            " runtime_plugin",
            "runtime_feature",
            "native_dynamic",
        ):
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
                        "validate report plan_summary.library_embed_compile_host."
                        "linked_runtime_crates[0].registration_kind must be "
                        "runtime_plugin",
                    )

    def _write_validate_report_with_linked_crates(
        self,
        out: Path,
        linked_runtime_crates: list[object],
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
        validate_report["plan_summary"] = {
            "library_embed_compile_host": compile_host_plan
        }
        validate_report_path.write_text(
            json.dumps(validate_report, indent=2),
            encoding="utf-8",
        )

    def _write_compile_host_report_with_linked_crates(
        self,
        out: Path,
        linked_runtime_crates: list[object],
    ) -> None:
        compile_host_report_path = out / "stages" / "compile_host" / "report.json"
        compile_host_report = json.loads(
            compile_host_report_path.read_text(encoding="utf-8")
        )
        compile_host_report["link_plan"]["linked_runtime_crates"] = (
            linked_runtime_crates
        )
        compile_host_report_path.write_text(
            json.dumps(compile_host_report, indent=2),
            encoding="utf-8",
        )

    def _linked_crate(self) -> dict[str, object]:
        return {
            "crate_name": "zircon_plugin_rendering_runtime",
            "path": "zircon_plugins/rendering/runtime",
            "registration_kind": "runtime_plugin",
            "provider_package_id": "rendering",
        }

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
