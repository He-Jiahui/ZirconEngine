from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.export_test_support import (
    _write_compile_host_report,
    _write_pack_report,
    _write_stage_report,
    _write_validate_report_with_strategies,
)


class PipelineReportValidateRuntimeAvailabilitySchemaTests(unittest.TestCase):
    def _valid_availability_entry(self, **overrides: object) -> dict[str, object]:
        entry: dict[str, object] = {
            "id": "rendering",
            "runtime_id": "rendering",
            "required": True,
            "maturity": "stable",
            "reason": "plugin descriptor satisfies profile gates",
        }
        entry.update(overrides)
        return entry

    def _valid_availability(
        self,
        *,
        category: str = "available",
        entry: dict[str, object] | None = None,
    ) -> dict[str, object]:
        availability: dict[str, object] = {
            "available": [],
            "linked": [],
            "native_dynamic": [],
            "externalized_missing": [],
            "stub": [],
            "blocked_by_target": [],
            "blocked_by_maturity": [],
            "missing_required": [],
        }
        availability[category] = [entry or self._valid_availability_entry()]
        return availability

    def _build_report_for_availability(
        self,
        runtime_plugin_availability: object,
    ) -> dict[str, object]:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["library_embed"])
            _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, out / "pack-output" / "assets.zrpack")
            _write_stage_report(out, "platform_bundle", fatal=False)
            validate_report_path = out / "stages" / "validate" / "report.json"
            validate_report = json.loads(
                validate_report_path.read_text(encoding="utf-8")
            )
            validate_report["plan_summary"] = {
                "runtime_plugin_availability": runtime_plugin_availability
            }
            validate_report_path.write_text(
                json.dumps(validate_report, indent=2),
                encoding="utf-8",
            )

            return build_pipeline_report(out, "windows-release")

    def _assert_validate_runtime_availability_diagnostic(
        self,
        report: dict[str, object],
        expected_diagnostic: str,
    ) -> None:
        self.assertTrue(report["fatal"])
        self.assertEqual(report["missing_stages"], [])
        self.assertEqual(report["fatal_stages"], ["Validate"])
        self.assertTrue(
            any(
                expected_diagnostic in diagnostic
                for diagnostic in report["diagnostics"]
            ),
            report["diagnostics"],
        )

    def test_report_stage_rejects_validate_runtime_availability_unknown_field(
        self,
    ) -> None:
        report = self._build_report_for_availability(
            {
                "available": [],
                "linked": [],
                "native_dynamic": [],
                "externalized_missing": [],
                "stub": [],
                "blocked_by_target": [],
                "blocked_by_maturity": [],
                "missing_required": [],
                "unsigned_sidecar": "sidecar.bin",
            }
        )

        self._assert_validate_runtime_availability_diagnostic(
            report,
            "validate report plan_summary.runtime_plugin_availability unknown field unsigned_sidecar",
        )

    def test_report_stage_rejects_validate_runtime_availability_non_object(
        self,
    ) -> None:
        report = self._build_report_for_availability(["available"])

        self._assert_validate_runtime_availability_diagnostic(
            report,
            "validate report plan_summary.runtime_plugin_availability must be an object",
        )

    def test_report_stage_rejects_validate_runtime_availability_entry_unknown_field(
        self,
    ) -> None:
        report = self._build_report_for_availability(
            {
                "available": [
                    {
                        "id": "rendering",
                        "runtime_id": "rendering",
                        "required": True,
                        "maturity": "stable",
                        "reason": "",
                        "unsigned_sidecar": "sidecar.bin",
                    }
                ],
                "linked": [],
                "native_dynamic": [],
                "externalized_missing": [],
                "stub": [],
                "blocked_by_target": [],
                "blocked_by_maturity": [],
                "missing_required": [],
            }
        )

        self._assert_validate_runtime_availability_diagnostic(
            report,
            "validate report plan_summary.runtime_plugin_availability.available[0] unknown field unsigned_sidecar",
        )

    def test_report_stage_rejects_validate_runtime_availability_bucket_non_array(
        self,
    ) -> None:
        report = self._build_report_for_availability(
            {
                "available": {"id": "rendering"},
                "linked": [],
                "native_dynamic": [],
                "externalized_missing": [],
                "stub": [],
                "blocked_by_target": [],
                "blocked_by_maturity": [],
                "missing_required": [],
            }
        )

        self._assert_validate_runtime_availability_diagnostic(
            report,
            "validate report plan_summary.runtime_plugin_availability.available must be an array",
        )

    def test_report_stage_rejects_validate_runtime_availability_entry_non_object(
        self,
    ) -> None:
        report = self._build_report_for_availability(
            {
                "available": ["rendering"],
                "linked": [],
                "native_dynamic": [],
                "externalized_missing": [],
                "stub": [],
                "blocked_by_target": [],
                "blocked_by_maturity": [],
                "missing_required": [],
            }
        )

        self._assert_validate_runtime_availability_diagnostic(
            report,
            "validate report plan_summary.runtime_plugin_availability.available[0] must be an object",
        )

    def test_report_stage_rejects_validate_runtime_availability_entry_string_fields_non_string(
        self,
    ) -> None:
        for field in ("id", "runtime_id", "maturity", "reason"):
            with self.subTest(field=field):
                entry = {
                    "id": "rendering",
                    "runtime_id": "rendering",
                    "required": True,
                    "maturity": "stable",
                    "reason": "",
                }
                entry[field] = 42
                report = self._build_report_for_availability(
                    {
                        "available": [entry],
                        "linked": [],
                        "native_dynamic": [],
                        "externalized_missing": [],
                        "stub": [],
                        "blocked_by_target": [],
                        "blocked_by_maturity": [],
                        "missing_required": [],
                    }
                )

                self._assert_validate_runtime_availability_diagnostic(
                    report,
                    "validate report plan_summary.runtime_plugin_availability."
                    f"available[0].{field} must be a string",
                )

    def test_report_stage_rejects_validate_runtime_availability_required_non_bool(
        self,
    ) -> None:
        report = self._build_report_for_availability(
            {
                "available": [
                    {
                        "id": "rendering",
                        "runtime_id": "rendering",
                        "required": "true",
                        "maturity": "stable",
                        "reason": "",
                    }
                ],
                "linked": [],
                "native_dynamic": [],
                "externalized_missing": [],
                "stub": [],
                "blocked_by_target": [],
                "blocked_by_maturity": [],
                "missing_required": [],
            }
        )

        self._assert_validate_runtime_availability_diagnostic(
            report,
            "validate report plan_summary.runtime_plugin_availability.available[0].required must be a boolean",
        )

    def test_report_stage_rejects_validate_runtime_availability_missing_bucket(
        self,
    ) -> None:
        availability = self._valid_availability()
        del availability["missing_required"]

        report = self._build_report_for_availability(availability)

        self._assert_validate_runtime_availability_diagnostic(
            report,
            "validate report plan_summary.runtime_plugin_availability missing field missing_required",
        )

    def test_report_stage_rejects_validate_runtime_availability_plugin_ids_invalid(
        self,
    ) -> None:
        cases = (
            (
                "",
                "must be a non-empty trimmed project plugin id",
            ),
            (
                "rendering ",
                "must be a non-empty trimmed project plugin id",
            ),
            (
                "Rendering",
                "must start with a lowercase ASCII letter",
            ),
            (
                "rendering-plugin",
                "must contain only lowercase ASCII letters, digits, and underscores",
            ),
            (
                "rendering__",
                "must not end with an underscore or contain repeated underscores",
            ),
        )
        for plugin_id, expected_diagnostic in cases:
            with self.subTest(plugin_id=plugin_id):
                report = self._build_report_for_availability(
                    self._valid_availability(
                        entry=self._valid_availability_entry(id=plugin_id)
                    )
                )

                self._assert_validate_runtime_availability_diagnostic(
                    report,
                    "validate report plan_summary.runtime_plugin_availability."
                    f"available[0].id {expected_diagnostic}",
                )

    def test_report_stage_rejects_validate_runtime_availability_runtime_ids_invalid(
        self,
    ) -> None:
        cases = (
            ("", "must be a non-empty trimmed string"),
            ("rendering ", "must be a non-empty trimmed string"),
            ("Rendering", "must be a known runtime plugin id"),
            ("renderer", "must be a known runtime plugin id"),
        )
        for runtime_id, expected_diagnostic in cases:
            with self.subTest(runtime_id=runtime_id):
                report = self._build_report_for_availability(
                    self._valid_availability(
                        entry=self._valid_availability_entry(runtime_id=runtime_id)
                    )
                )

                self._assert_validate_runtime_availability_diagnostic(
                    report,
                    "validate report plan_summary.runtime_plugin_availability."
                    f"available[0].runtime_id {expected_diagnostic}",
                )

    def test_report_stage_rejects_validate_runtime_availability_id_runtime_mismatch(
        self,
    ) -> None:
        report = self._build_report_for_availability(
            self._valid_availability(
                entry=self._valid_availability_entry(
                    id="sound",
                    runtime_id="rendering",
                )
            )
        )

        self._assert_validate_runtime_availability_diagnostic(
            report,
            "validate report plan_summary.runtime_plugin_availability."
            "available[0].id must match runtime_id",
        )

    def test_report_stage_rejects_validate_runtime_availability_maturity_invalid(
        self,
    ) -> None:
        cases = (
            ("", "must be a non-empty trimmed string"),
            ("stable ", "must be a non-empty trimmed string"),
            ("Stable", "must be a known plugin maturity"),
            ("alpha", "must be a known plugin maturity"),
        )
        for maturity, expected_diagnostic in cases:
            with self.subTest(maturity=maturity):
                report = self._build_report_for_availability(
                    self._valid_availability(
                        entry=self._valid_availability_entry(maturity=maturity)
                    )
                )

                self._assert_validate_runtime_availability_diagnostic(
                    report,
                    "validate report plan_summary.runtime_plugin_availability."
                    f"available[0].maturity {expected_diagnostic}",
                )

    def test_report_stage_rejects_validate_runtime_availability_reason_not_trimmed(
        self,
    ) -> None:
        for reason in ("", " supplied by linked registration "):
            with self.subTest(reason=reason):
                report = self._build_report_for_availability(
                    self._valid_availability(
                        entry=self._valid_availability_entry(reason=reason)
                    )
                )

                self._assert_validate_runtime_availability_diagnostic(
                    report,
                    "validate report plan_summary.runtime_plugin_availability."
                    "available[0].reason must be a non-empty trimmed string",
                )

    def test_report_stage_rejects_validate_runtime_availability_missing_required_false(
        self,
    ) -> None:
        report = self._build_report_for_availability(
            self._valid_availability(
                category="missing_required",
                entry=self._valid_availability_entry(required=False),
            )
        )

        self._assert_validate_runtime_availability_diagnostic(
            report,
            "validate report plan_summary.runtime_plugin_availability."
            "missing_required[0].required must be true",
        )



if __name__ == "__main__":
    unittest.main()
