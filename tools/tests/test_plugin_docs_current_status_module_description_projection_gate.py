from tools.tests.plugin_status_document import StatusDocumentPath as Path
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
FRAMEWORKS_STATUS_ID = (
    "frameworks_02_m3_plugin_validate_module_description_projection_gate_passed_full_integration_pending"
)
PLUGIN_STATUS_ID = "plugins_13_m5_t1_plugin_validate_module_description_projection_gate"


class PluginValidateModuleDescriptionProjectionStatusTests(unittest.TestCase):
    def test_current_status_docs_record_module_description_projection_gate(self) -> None:
        common_docs = [
            "docs/plans/zircon_plugins/09-export-publishing.md",
            "docs/plans/zircon_plugins/13-standalone-plugin-build.md",
            "docs/zircon_plugins/plugin-standalone-build.md",
            "docs/cli-and-tooling/zircon-export-tool.md",
        ]
        common_required = [
            PLUGIN_STATUS_ID,
            FRAMEWORKS_STATUS_ID,
            "PLUGIN_VALIDATE_MODULE_FIELDS",
            "description",
            "plugin_validate_optional_trimmed_string",
            "test_plugin_validate_modules",
            "plugin validate --all",
            "不声明 Cargo build/test/export",
        ]

        missing: list[str] = []
        for relative_path in common_docs:
            source = (REPO_ROOT / relative_path).read_text(encoding="utf-8")
            for phrase in common_required:
                if phrase not in source:
                    missing.append(f"{relative_path}: {phrase}")

        framework_docs = [
            "docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md",
            "docs/plans/engine-code-structure-convention.md",
            "docs/plans/engine-code-review-findings-2026-06.md",
            ".codex/sessions/20260702-2358-runtime-frameworks-foundation.md",
        ]
        framework_required = [
            FRAMEWORKS_STATUS_ID,
            "plugin_validate_modules.py",
            "description",
            "unknown",
            "full integration",
        ]
        for relative_path in framework_docs:
            source = (REPO_ROOT / relative_path).read_text(encoding="utf-8")
            for phrase in framework_required:
                if phrase not in source:
                    missing.append(f"{relative_path}: {phrase}")

        self.assertFalse(
            missing,
            "PluginValidate module description projection status is incomplete:\n"
            + "\n".join(missing),
        )


if __name__ == "__main__":
    unittest.main()
