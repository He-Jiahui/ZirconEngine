from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
RESUME_FLOW_ROOT = (
    REPO_ROOT / "tools/zircon_export/tests/test_pipeline_resume_flow.py"
)
PLATFORM_BUNDLE_HANDOFF_OWNER = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_pipeline_resume_platform_bundle_handoff.py"
)

PLATFORM_BUNDLE_HANDOFF_METHODS = (
    "test_pipeline_platform_bundle_uses_compile_host_report_host",
    "test_pipeline_platform_bundle_rejects_invalid_compile_host_report_host_field",
    "test_pipeline_platform_bundle_rejects_compile_host_report_host_resolve_error",
    "test_pipeline_platform_bundle_uses_pack_report_pack_path",
    "test_stage_platform_bundle_uses_report_handoff_paths",
    "test_stage_platform_bundle_uses_report_delta_pack_path",
    "test_stage_platform_bundle_uses_native_dynamic_report_plugins",
    "test_pipeline_platform_bundle_ignores_pack_report_without_profile",
    "test_pipeline_platform_bundle_rejects_invalid_pack_report_pack_field",
    "test_pipeline_platform_bundle_rejects_invalid_pack_report_delta_pack_field",
)

ROOT_RESUME_FLOW_METHODS = (
    "test_cli_stage_choices_match_shared_pipeline_order",
    "test_resume_from_rejects_explicit_stage",
    "test_resume_from_platform_bundle_stops_before_report_on_failure",
    "test_pipeline_pack_uses_cook_assets_report_manifest",
    "test_pipeline_pack_rejects_invalid_cook_assets_report_manifest_field",
)


def _line_count(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    return len(text.splitlines())


class ZirconExportPipelineResumeFlowTestOwnerBoundaryTests(unittest.TestCase):
    def test_platform_bundle_handoff_tests_live_in_dedicated_owner(self) -> None:
        self.assertTrue(
            PLATFORM_BUNDLE_HANDOFF_OWNER.exists(),
            "PlatformBundle handoff test owner is missing",
        )
        root_text = RESUME_FLOW_ROOT.read_text(encoding="utf-8")
        handoff_text = PLATFORM_BUNDLE_HANDOFF_OWNER.read_text(encoding="utf-8")

        for method_name in PLATFORM_BUNDLE_HANDOFF_METHODS:
            with self.subTest(method=method_name):
                self.assertNotIn(
                    f"def {method_name}",
                    root_text,
                    f"{method_name} should not return to the broad resume flow owner",
                )
                self.assertIn(
                    f"def {method_name}",
                    handoff_text,
                    f"{method_name} belongs in the PlatformBundle handoff owner",
                )

        self.assertNotIn(
            "native_dynamic_export_test_support",
            root_text,
            "NativeDynamic fixture helpers should stay in the handoff owner",
        )

    def test_resume_flow_root_keeps_pipeline_and_pack_resume_coverage(self) -> None:
        root_text = RESUME_FLOW_ROOT.read_text(encoding="utf-8")
        handoff_text = (
            PLATFORM_BUNDLE_HANDOFF_OWNER.read_text(encoding="utf-8")
            if PLATFORM_BUNDLE_HANDOFF_OWNER.exists()
            else ""
        )

        for method_name in ROOT_RESUME_FLOW_METHODS:
            with self.subTest(method=method_name):
                self.assertIn(f"def {method_name}", root_text)
                self.assertNotIn(
                    f"def {method_name}",
                    handoff_text,
                    f"{method_name} should remain in the broad resume flow owner",
                )

    def test_pipeline_resume_flow_test_owners_stay_under_line_budgets(self) -> None:
        budgets = {
            RESUME_FLOW_ROOT: 620,
            PLATFORM_BUNDLE_HANDOFF_OWNER: 460,
        }
        failures: list[str] = []
        for path, budget in budgets.items():
            if not path.exists():
                failures.append(f"{path.relative_to(REPO_ROOT)} is missing")
                continue
            line_count = _line_count(path)
            if line_count > budget:
                failures.append(f"{path.relative_to(REPO_ROOT)}: {line_count} > {budget}")

        if failures:
            self.fail(
                "Pipeline resume flow test owners exceeded boundaries:\n"
                + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
