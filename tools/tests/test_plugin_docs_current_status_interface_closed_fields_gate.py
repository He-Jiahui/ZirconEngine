import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path


STATUS_ID = "plugins_13_m5_t1_plugin_validate_interface_closed_fields_gate"


def _section(text: str, start: str, end: str) -> str:
    start_index = text.index(start)
    end_index = text.index(end, start_index)
    return text[start_index:end_index]


def _tail_section(text: str, start: str) -> str:
    return text[text.index(start) :]


class PluginDocsCurrentStatusInterfaceClosedFieldsGateTests(unittest.TestCase):
    def test_current_status_docs_record_interface_closed_fields_gate(self):
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
        }
        required_phrases = [
            STATUS_ID,
            "plugin_validate_interfaces.py",
            "PLUGIN_VALIDATE_PROVIDED_INTERFACE_FIELDS",
            "validate_plugin_provided_interface_known_fields",
            "plugin_validate_interface_methods.py",
            "PLUGIN_VALIDATE_INTERFACE_METHOD_FIELDS",
            "validate_plugin_interface_method_known_fields",
            "plugin_validate_interface_signatures.py",
            "PLUGIN_VALIDATE_INTERFACE_METHOD_PARAMETER_FIELDS",
            "PLUGIN_VALIDATE_INTERFACE_METHOD_TYPE_REF_FIELDS",
            "validate_plugin_interface_signature_known_fields",
            "is not a known provided interface field",
            "is not a known provided interface method field",
            "is not a known interface method parameter field",
            "is not a known interface method type_ref field",
            "test_plugin_validate_rejects_unknown_interface_fields",
            "test_interface_contracts_live_in_interfaces_owner",
            "test_interface_method_contracts_live_in_interface_methods_owner",
            "test_interface_signature_contracts_live_in_interface_signatures_owner",
            "不声明 Hub/editor E2E、完整 export matrix 或 startup-to-first-frame",
        ]
        failures: list[str] = []
        for section_name, section in sections.items():
            for phrase in required_phrases:
                if phrase not in section:
                    failures.append(f"{section_name}: missing {phrase}")
        if failures:
            self.fail(
                "Current plugin docs do not record interface closed-fields gate:\n"
                + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
