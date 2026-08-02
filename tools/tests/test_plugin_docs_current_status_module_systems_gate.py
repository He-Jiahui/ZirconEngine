import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path


def _section(text: str, start: str, end: str) -> str:
    start_index = text.index(start)
    end_index = text.index(end, start_index)
    return text[start_index:end_index]


def _tail_section(text: str, start: str) -> str:
    return text[text.index(start) :]


class PluginDocsCurrentStatusModuleSystemsGateTests(unittest.TestCase):
    def test_current_status_records_module_systems_gate(self):
        repo_root = Path(__file__).resolve().parents[2]
        status_id = "plugins_13_m5_t1_plugin_validate_module_systems_gate"

        plan_13_text = (
            repo_root / "docs/plans/zircon_plugins/13-standalone-plugin-build.md"
        ).read_text(encoding="utf-8")
        plan_13_status = _tail_section(plan_13_text, "## 9. 审查和验收记录")
        plan_09_text = (
            repo_root / "docs/plans/zircon_plugins/09-export-publishing.md"
        ).read_text(encoding="utf-8")
        plan_09_status = _section(
            plan_09_text, "## 状态与产出记录", "## 5. 里程碑与任务分解"
        )
        standalone_text = (
            repo_root / "docs/zircon_plugins/plugin-standalone-build.md"
        ).read_text(encoding="utf-8")
        standalone_status = _tail_section(
            standalone_text, "## 9. 当前落地状态"
        )
        export_tool_text = (
            repo_root / "docs/cli-and-tooling/zircon-export-tool.md"
        ).read_text(encoding="utf-8")
        structure_text = (
            repo_root / "docs/plans/engine-code-structure-convention.md"
        ).read_text(encoding="utf-8")
        review_text = (
            repo_root / "docs/plans/engine-code-review-findings-2026-06.md"
        ).read_text(encoding="utf-8")

        sections = {
            "Plugins 13 status": plan_13_status,
            "Plugins 09 status": plan_09_status,
            "standalone current status": standalone_status,
            "export tool docs": export_tool_text,
            "structure convention": structure_text,
            "review findings": review_text,
        }
        required_phrases = [
            status_id,
            "plugin_validate_module_systems.py",
            "validate_plugin_module_system_contracts",
            "validate_plugin_module_system_names",
            "system_sets",
            "system_anchors",
            "may only be declared by runtime modules",
            "duplicates system_sets",
            "duplicates system_anchors",
            "should stay under namespace",
            "test_plugin_validate_rejects_malformed_module_system_names",
            "test_plugin_validate_rejects_duplicate_module_system_names",
            "test_plugin_validate_rejects_non_runtime_module_system_names",
            "test_module_system_contracts_live_in_module_systems_owner",
        ]

        failures: list[str] = []
        for section_name, section in sections.items():
            for phrase in required_phrases:
                if phrase not in section:
                    failures.append(f"{section_name}: missing {phrase}")

        if failures:
            self.fail(
                "Current plugin docs do not record module systems gate:\n"
                + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
