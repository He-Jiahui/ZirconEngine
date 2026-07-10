import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path


STATUS_ID = "plugins_13_m5_t1_structure_audit_event_catalogs_guard"


def _section(text: str, start: str, end: str) -> str:
    start_index = text.index(start)
    end_index = text.index(end, start_index)
    return text[start_index:end_index]


def _tail_section(text: str, start: str) -> str:
    return text[text.index(start) :]


class PluginDocsCurrentStatusStructureAuditEventCatalogsGuardTests(unittest.TestCase):
    def test_current_status_records_event_catalogs_guard(self):
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
            "tools/plugin_structure_audits/manifest_schema.py",
            "tools/plugin_structure_audits/manifest_schema_event_catalogs.py",
            "collect_event_catalogs_schema_violations",
            "EVENT_CATALOG_FIELDS",
            "EVENT_FIELDS",
            "event_catalogs",
            "payload_schema",
            "is not a known event catalog field",
            "is not a known event field",
            "duplicates event catalog namespace",
            "should stay under package namespace",
            "should be a positive u32",
            "should end with a version segment like v1",
            "test_manifest_schema_rejects_malformed_event_catalog_row",
            "test_manifest_schema_rejects_duplicate_event_catalog_namespace",
            "test_manifest_schema_rejects_event_payload_schema_version",
            "manifest_schema_violations=0",
            "不声明 Hub/editor E2E、完整 export matrix 或 startup-to-first-frame",
        ]
        failures: list[str] = []
        for section_name, section in sections.items():
            for phrase in required_phrases:
                if phrase not in section:
                    failures.append(f"{section_name}: missing {phrase}")
        if failures:
            self.fail(
                "Current plugin docs do not record structure-audit "
                "event_catalogs guard:\n" + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
