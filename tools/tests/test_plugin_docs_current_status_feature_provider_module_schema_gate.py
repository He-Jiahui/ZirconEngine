import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path


STATUS_ID = "plugins_13_m5_t1_plugin_validate_feature_provider_module_schema_gate"


def _section(text: str, start: str, end: str) -> str:
    start_index = text.index(start)
    end_index = text.index(end, start_index)
    return text[start_index:end_index]


def _tail_section(text: str, start: str) -> str:
    return text[text.index(start) :]


class PluginDocsCurrentStatusFeatureProviderModuleSchemaGateTests(unittest.TestCase):
    def test_current_status_docs_record_feature_provider_module_schema_gate(self):
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
            "plugin_validate_feature_provider_module_schema.py",
            "validate_plugin_feature_provider_module_schema",
            "validate_plugin_module_name",
            "validate_plugin_module_kind",
            "validate_plugin_module_crate_name",
            "validate_plugin_module_target_modes",
            "validate_plugin_module_capabilities",
            "PLUGIN_VALIDATE_TARGET_MODES",
            "generated feature_extensions[0].modules[0].name Sound.Timeline.Runtime should contain only lowercase ASCII letters, digits, underscores, and dots",
            "generated feature_extensions[0].modules[0].kind tooling should be one of runtime, editor, native, vm",
            "generated feature_extensions[0].modules[0].crate_name Bad_Crate should use the zircon_plugin_ prefix",
            'generated feature_extensions[0].modules[0].target_modes[0] "nightly_runtime" is unsupported; expected one of client_runtime, server_runtime, editor_host',
            "test_plugin_validate_rejects_generated_feature_provider_module_schema_drift",
            "test_feature_provider_module_schema_stays_in_schema_leaf",
            "不声明 Hub/editor E2E、完整 export matrix 或 startup-to-first-frame",
        ]
        failures: list[str] = []
        for section_name, section in sections.items():
            for phrase in required_phrases:
                if phrase not in section:
                    failures.append(f"{section_name}: missing {phrase}")
        if failures:
            self.fail(
                "Current plugin docs do not record feature-provider module "
                "schema gate:\n" + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
