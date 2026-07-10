import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path


def _tail_section(text: str, start: str) -> str:
    return text[text.index(start) :]


class PluginDocsCurrentStatusExportTestSupportOwnerSplitTests(unittest.TestCase):
    def test_current_status_records_export_test_support_owner_split(self):
        repo_root = Path(__file__).resolve().parents[2]
        status_id = "plugins_09_e1_export_test_support_owner_split"

        plan_09_text = (
            repo_root / "docs/plans/zircon_plugins/09-export-publishing.md"
        ).read_text(encoding="utf-8")
        plan_13_text = (
            repo_root / "docs/plans/zircon_plugins/13-standalone-plugin-build.md"
        ).read_text(encoding="utf-8")
        standalone_text = (
            repo_root / "docs/zircon_plugins/plugin-standalone-build.md"
        ).read_text(encoding="utf-8")
        structure_text = (
            repo_root / "docs/plans/engine-code-structure-convention.md"
        ).read_text(encoding="utf-8")
        review_text = (
            repo_root / "docs/plans/engine-code-review-findings-2026-06.md"
        ).read_text(encoding="utf-8")
        session_text = (
            repo_root / ".codex/sessions/20260628-0317-zui-migration-validation.md"
        ).read_text(encoding="utf-8")

        sections = {
            "Plugins 09 status": _tail_section(
                plan_09_text, "## 状态与产出记录"
            ),
            "Plugins 13 status": _tail_section(
                plan_13_text, "## 9. 审查和验收记录"
            ),
            "standalone current status": _tail_section(
                standalone_text, "## 9. 当前落地状态"
            ),
            "structure convention": structure_text,
            "review findings": review_text,
            "active session": session_text,
        }
        required_phrases = [
            status_id,
            "tools/zircon_export/tests/export_test_support.py=893",
            "tools/zircon_export/tests/native_dynamic_export_test_support.py=373",
            "tools/zircon_export/tests/platform_bundle_export_test_support.py=223",
            "test_native_dynamic_helpers_have_dedicated_owner",
            "test_platform_bundle_helpers_have_dedicated_owner",
            "test_moved_helpers_are_not_imported_from_root_support",
            "python -m unittest tools.tests.test_zircon_export_export_test_support_owner_boundaries",
            "4/4",
            "python -m unittest tools.zircon_export.tests.test_native_dynamic_stage tools.zircon_export.tests.test_native_dynamic_stage_source_manifest tools.zircon_export.tests.test_native_dynamic_stage_selection_strategy tools.zircon_export.tests.test_pipeline_report_native_dynamic_stage_schema tools.zircon_export.tests.test_pipeline_report_native_dynamic_stage_operation_audit_schema tools.zircon_export.tests.test_pipeline_report_stage_location",
            "92/92",
            "python -m unittest tools.zircon_export.tests.test_platform_bundle_native_dynamic tools.zircon_export.tests.test_platform_bundle_native_dynamic_pipeline_payload tools.zircon_export.tests.test_platform_bundle_strategy_validation tools.zircon_export.tests.test_platform_bundle_native_dynamic_operation_audit tools.zircon_export.tests.test_platform_bundle_native_payload_loader_manifest",
            "58/58",
            "python -m unittest tools.zircon_export.tests.test_pipeline_report_stage tools.zircon_export.tests.test_pipeline_report_cook_assets_pack_handoff tools.zircon_export.tests.test_pipeline_resume_flow tools.zircon_export.tests.test_pipeline_report_validate_native_dynamic_schema",
            "80/80",
            "py_compile",
        ]

        failures: list[str] = []
        for section_name, section in sections.items():
            for phrase in required_phrases:
                if phrase not in section:
                    failures.append(f"{section_name}: missing {phrase}")

        if failures:
            self.fail(
                "Current plugin status docs do not record the export test "
                "support owner split:\n" + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
