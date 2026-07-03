from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.stage_handoff import (
    pack_report_delta_pack_file,
    pack_report_pack_file,
    stage_report_optional_path_handoff_diagnostic,
    stage_report_path_handoff_diagnostic,
    validate_report_asset_filter,
    validate_report_asset_filter_diagnostic,
)
from tools.zircon_export.stage_handoff_strategy import (
    export_strategy_diagnostics,
    export_strategy_list_is_empty,
    export_strategy_list_is_invalid,
    export_strategies_from_validate_report,
    native_dynamic_payload_allowed,
    unsupported_export_strategies_from_validate_report,
)


class StageHandoffTests(unittest.TestCase):
    def test_required_path_field_reports_invalid_value(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir)
            write_stage_report(out, "pack", {"pack": []})

            self.assertIsNone(pack_report_pack_file(out, "windows-release"))
            self.assertEqual(
                stage_report_path_handoff_diagnostic(
                    out,
                    "pack",
                    "windows-release",
                    "pack",
                ),
                "Pack report field pack must be a non-empty string",
            )

    def test_required_path_field_reports_stage_identity_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir)
            write_stage_report(
                out,
                "pack",
                {"stage": "CompileHost", "pack": str(out / "assets.zrpack")},
            )

            self.assertIsNone(pack_report_pack_file(out, "windows-release"))
            self.assertEqual(
                stage_report_path_handoff_diagnostic(
                    out,
                    "pack",
                    "windows-release",
                    "pack",
                ),
                "Pack report stage CompileHost does not match expected stage Pack",
            )

    def test_required_path_field_reports_invalid_fatal_metadata(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir)
            write_stage_report(
                out,
                "pack",
                {"fatal": [], "pack": str(out / "assets.zrpack")},
            )

            self.assertIsNone(pack_report_pack_file(out, "windows-release"))
            self.assertEqual(
                stage_report_path_handoff_diagnostic(
                    out,
                    "pack",
                    "windows-release",
                    "pack",
                ),
                "Pack report fatal must be a boolean",
            )

    def test_required_path_field_reports_stage_report_directory(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir)
            report_path = out / "stages" / "pack" / "report.json"
            report_path.mkdir(parents=True)

            self.assertIsNone(pack_report_pack_file(out, "windows-release"))
            self.assertEqual(
                stage_report_path_handoff_diagnostic(
                    out,
                    "pack",
                    "windows-release",
                    "pack",
                ),
                f"Pack report {report_path} is not a file",
            )

    def test_required_path_field_reports_invalid_diagnostics_metadata(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir)
            write_stage_report(
                out,
                "pack",
                {
                    "diagnostics": "not-a-list",
                    "pack": str(out / "assets.zrpack"),
                },
            )

            self.assertIsNone(pack_report_pack_file(out, "windows-release"))
            self.assertEqual(
                stage_report_path_handoff_diagnostic(
                    out,
                    "pack",
                    "windows-release",
                    "pack",
                ),
                "Pack report diagnostics must be a string array",
            )

    def test_required_path_field_reports_path_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir)
            pack = out / "assets.zrpack"
            write_stage_report(out, "pack", {"pack": str(pack)})
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if str(path) == str(pack):
                    raise OSError("simulated required handoff path failure")
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                self.assertIsNone(pack_report_pack_file(out, "windows-release"))
                self.assertEqual(
                    stage_report_path_handoff_diagnostic(
                        out,
                        "pack",
                        "windows-release",
                        "pack",
                    ),
                    (
                        f"Pack report field pack {pack} could not be resolved: "
                        "simulated required handoff path failure"
                    ),
                )

    def test_optional_path_field_reports_invalid_present_value(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir)
            write_stage_report(
                out,
                "pack",
                {
                    "pack": str(out / "assets.zrpack"),
                    "delta_pack": [],
                },
            )

            self.assertIsNone(pack_report_delta_pack_file(out, "windows-release"))
            self.assertEqual(
                stage_report_optional_path_handoff_diagnostic(
                    out,
                    "pack",
                    "windows-release",
                    "delta_pack",
                ),
                "Pack report field delta_pack must be a non-empty string",
            )

    def test_optional_path_field_reports_blank_present_value(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir)
            write_stage_report(
                out,
                "pack",
                {
                    "pack": str(out / "assets.zrpack"),
                    "delta_pack": " ",
                },
            )

            self.assertIsNone(pack_report_delta_pack_file(out, "windows-release"))
            self.assertEqual(
                stage_report_optional_path_handoff_diagnostic(
                    out,
                    "pack",
                    "windows-release",
                    "delta_pack",
                ),
                "Pack report field delta_pack must be a non-empty string",
            )

    def test_optional_path_field_reports_path_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir)
            delta_pack = out / "assets.delta.zrpd"
            write_stage_report(
                out,
                "pack",
                {
                    "pack": str(out / "assets.zrpack"),
                    "delta_pack": str(delta_pack),
                },
            )
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if str(path) == str(delta_pack):
                    raise OSError("simulated optional handoff path failure")
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                self.assertIsNone(pack_report_delta_pack_file(out, "windows-release"))
                self.assertEqual(
                    stage_report_optional_path_handoff_diagnostic(
                        out,
                        "pack",
                        "windows-release",
                        "delta_pack",
                    ),
                    (
                        f"Pack report field delta_pack {delta_pack} "
                        "could not be resolved: "
                        "simulated optional handoff path failure"
                    ),
                )

    def test_optional_path_field_ignores_stage_report_directory(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir)
            (out / "stages" / "pack" / "report.json").mkdir(parents=True)

            self.assertIsNone(pack_report_delta_pack_file(out, "windows-release"))
            self.assertIsNone(
                stage_report_optional_path_handoff_diagnostic(
                    out,
                    "pack",
                    "windows-release",
                    "delta_pack",
                )
            )

    def test_optional_path_field_allows_absent_value(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir)
            write_stage_report(out, "pack", {"pack": str(out / "assets.zrpack")})

            self.assertIsNone(
                stage_report_optional_path_handoff_diagnostic(
                    out,
                    "pack",
                    "windows-release",
                    "delta_pack",
                )
            )

    def test_validate_asset_filter_reports_invalid_present_value(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir)
            write_stage_report(
                out,
                "validate",
                {
                    "profile_summary": {
                        "asset_filter": [],
                    }
                },
            )

            self.assertIsNone(validate_report_asset_filter(out, "windows-release"))
            self.assertEqual(
                validate_report_asset_filter_diagnostic(out, "windows-release"),
                (
                    "Validate report field profile_summary.asset_filter must be a "
                    "non-empty string"
                ),
            )

    def test_validate_asset_filter_reports_invalid_validate_metadata(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir)
            write_stage_report(
                out,
                "validate",
                {
                    "fatal": [],
                    "profile_summary": {
                        "asset_filter": "shipping",
                    },
                },
            )

            self.assertIsNone(validate_report_asset_filter(out, "windows-release"))
            self.assertEqual(
                validate_report_asset_filter_diagnostic(out, "windows-release"),
                "Validate report fatal must be a boolean",
            )

    def test_export_strategy_helpers_normalize_and_report_metadata(self) -> None:
        report = {
            "profile_summary": {
                "strategies": [
                    "LibraryEmbed",
                    "native-dynamic",
                    "future_export_path",
                    "future_export_path",
                    42,
                ],
            },
        }

        self.assertFalse(export_strategy_list_is_invalid(report))
        self.assertFalse(export_strategy_list_is_empty(report))
        self.assertEqual(
            export_strategies_from_validate_report(report),
            {"library_embed", "native_dynamic"},
        )
        self.assertEqual(
            unsupported_export_strategies_from_validate_report(report),
            ["future_export_path", "42"],
        )
        self.assertEqual(
            export_strategy_diagnostics(report),
            [
                "unsupported export strategy future_export_path",
                "unsupported export strategy 42",
            ],
        )
        self.assertTrue(
            export_strategy_list_is_invalid(
                {"profile_summary": {"strategies": "library_embed"}}
            )
        )
        self.assertTrue(
            export_strategy_list_is_empty({"profile_summary": {"strategies": []}})
        )

    def test_native_dynamic_payload_allowed_uses_strategy_evidence(self) -> None:
        self.assertTrue(native_dynamic_payload_allowed(None))
        self.assertTrue(native_dynamic_payload_allowed({"profile_summary": {}}))
        self.assertTrue(
            native_dynamic_payload_allowed(
                {"profile_summary": {"strategies": ["native_dynamic"]}}
            )
        )
        self.assertFalse(
            native_dynamic_payload_allowed(
                {"profile_summary": {"strategies": ["library_embed"]}}
            )
        )
        self.assertFalse(
            native_dynamic_payload_allowed(
                {"profile_summary": {"strategies": "native_dynamic"}}
            )
        )


def write_stage_report(
    out: Path,
    stage: str,
    extra_fields: dict[str, object],
) -> None:
    report_dir = out / "stages" / stage
    report_dir.mkdir(parents=True, exist_ok=True)
    report = {
        "stage": "".join(part.capitalize() for part in stage.split("_")),
        "profile": "windows-release",
        "fatal": False,
        "diagnostics": [],
    }
    report.update(extra_fields)
    report_dir.joinpath("report.json").write_text(
        json.dumps(report),
        encoding="utf-8",
    )


if __name__ == "__main__":
    unittest.main()
