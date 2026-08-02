from tools.tests.plugin_status_document import StatusDocumentPath as Path
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
STATUS_ID = "plugins_13_m5_t1_plugin_validate_component_closed_fields_gate"

STATUS_DOCS = [
    "docs/plans/zircon_plugins/09-export-publishing.md",
    "docs/plans/zircon_plugins/13-standalone-plugin-build.md",
    "docs/zircon_plugins/plugin-standalone-build.md",
    "docs/cli-and-tooling/zircon-export-tool.md",
    "docs/plans/engine-code-structure-convention.md",
    "docs/plans/engine-code-review-findings-2026-06.md",
]


class PluginValidateComponentClosedFieldsStatusTests(unittest.TestCase):
    def test_current_status_docs_record_component_closed_fields_gate(self):
        required_phrases = [
            STATUS_ID,
            "validate_plugin_component_known_fields",
            "PLUGIN_VALIDATE_COMPONENT_FIELDS",
            "PLUGIN_VALIDATE_COMPONENT_PROPERTY_FIELDS",
            "PLUGIN_VALIDATE_UI_COMPONENT_FIELDS",
            "test_plugin_validate_rejects_unknown_component_fields",
            "is not a known component field",
            "is not a known component property field",
            "is not a known ui_component field",
            "test_components_lives_in_components_owner",
            "test_component_tests_live_in_component_test_owner",
            "不声明 Hub/editor E2E、完整 export matrix 或 startup-to-first-frame",
        ]

        missing: list[str] = []
        for relative_path in STATUS_DOCS:
            source = (REPO_ROOT / relative_path).read_text(encoding="utf-8")
            for phrase in required_phrases:
                if phrase not in source:
                    missing.append(f"{relative_path}: {phrase}")

        self.assertFalse(
            missing,
            "PluginValidate component closed-fields status is incomplete:\n"
            + "\n".join(missing),
        )


if __name__ == "__main__":
    unittest.main()
