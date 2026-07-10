import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path


STATUS_ID = "plugins_13_m5_t1_retired_ui_asset_repo_audit_guard"


def _section(text: str, start: str, end: str) -> str:
    start_index = text.index(start)
    end_index = text.index(end, start_index)
    return text[start_index:end_index]


def _tail_section(text: str, start: str) -> str:
    return text[text.index(start) :]


class PluginDocsCurrentStatusRetiredUiAssetAuditGuardTests(unittest.TestCase):
    def test_current_status_records_retired_ui_asset_repo_audit_guard(self):
        repo_root = Path(__file__).resolve().parents[2]
        plan_13_text = (
            repo_root / "docs/plans/zircon_plugins/13-standalone-plugin-build.md"
        ).read_text(encoding="utf-8")
        plan_09_text = (
            repo_root / "docs/plans/zircon_plugins/09-export-publishing.md"
        ).read_text(encoding="utf-8")
        standalone_text = (
            repo_root / "docs/zircon_plugins/plugin-standalone-build.md"
        ).read_text(encoding="utf-8")
        sections = {
            "Plugins 13 status": _tail_section(plan_13_text, "## 9. 审查和验收记录"),
            "Plugins 09 status": _section(
                plan_09_text, "## 状态与产出记录", "## 5. 里程碑与任务分解"
            ),
            "standalone current status": _tail_section(
                standalone_text, "## 9. 当前落地状态"
            ),
            "export tool docs": (
                repo_root / "docs/cli-and-tooling/zircon-export-tool.md"
            ).read_text(encoding="utf-8"),
            "structure convention": (
                repo_root / "docs/plans/engine-code-structure-convention.md"
            ).read_text(encoding="utf-8"),
            "review findings": (
                repo_root / "docs/plans/engine-code-review-findings-2026-06.md"
            ).read_text(encoding="utf-8"),
            "active session": (
                repo_root / ".codex/sessions/20260628-0317-zui-migration-validation.md"
            ).read_text(encoding="utf-8"),
        }
        required_phrases = [
            STATUS_ID,
            "tools/plugin_structure_audits/retired_ui_assets.py",
            "audit_retired_ui_asset_conformance",
            "RETIRED_UI_ASSET_SCAN_ROOTS",
            "RETIRED_UI_ASSET_SUFFIXES",
            "retired_ui_asset_files=0",
            "zui_only_layout_status=zui-only-clean",
            "tools/tests/test_plugin_structure_audit_retired_ui_assets.py",
            "test_retired_ui_asset_audit_reports_legacy_ui_toml_files",
            "test_retired_ui_asset_audit_reports_zui_only_clean",
            "test_report_exposes_retired_ui_asset_status",
            "不声明 Hub/editor E2E、完整 export matrix 或 startup-to-first-frame",
        ]
        failures: list[str] = []
        for section_name, section in sections.items():
            for phrase in required_phrases:
                if phrase not in section:
                    failures.append(f"{section_name}: missing {phrase}")
        if failures:
            self.fail(
                "Current plugin docs do not record retired UI asset repo audit "
                "guard:\n" + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
