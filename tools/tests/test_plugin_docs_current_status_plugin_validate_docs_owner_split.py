import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path


class PluginValidateDocsStatusOwnerSplitDocsTests(unittest.TestCase):
    def test_plugin_validate_docs_status_owner_split_is_recorded(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        required_files = [
            "docs/plans/zircon_plugins/13-standalone-plugin-build.md",
            "docs/plans/zircon_plugins/09-export-publishing.md",
            "docs/zircon_plugins/plugin-standalone-build.md",
            "docs/cli-and-tooling/zircon-export-tool.md",
            "docs/plans/engine-code-structure-convention.md",
            "docs/plans/engine-code-review-findings-2026-06.md",
            ".codex/sessions/20260628-0317-zui-migration-validation.md",
        ]
        required_phrases = [
            "plugins_13_m5_t1_plugin_validate_docs_status_owner_split",
            "test_plugin_docs_current_status_plugin_validate_owner_splits.py",
            "test_plugin_docs_current_status_convergence.py",
            "PluginValidate",
            "current-status",
            "6346",
            "666",
        ]

        missing: list[str] = []
        for relative_path in required_files:
            text = (repo_root / relative_path).read_text(encoding="utf-8")
            for phrase in required_phrases:
                if phrase not in text:
                    missing.append(f"{relative_path}: {phrase}")

        self.assertEqual([], missing)


if __name__ == "__main__":
    unittest.main()
