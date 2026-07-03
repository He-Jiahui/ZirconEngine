from pathlib import Path
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
STATUS_ID = "plugins_13_m5_t1_plugin_validate_root_test_retirement_status_converged"
ROOT_TEST = REPO_ROOT / "tools/zircon_export/tests/test_plugin_validate.py"

STATUS_DOCS = [
    "docs/plans/zircon_plugins/09-export-publishing.md",
    "docs/plans/zircon_plugins/13-standalone-plugin-build.md",
    "docs/zircon_plugins/plugin-standalone-build.md",
    "docs/cli-and-tooling/zircon-export-tool.md",
    "docs/plans/engine-code-structure-convention.md",
    "docs/plans/engine-code-review-findings-2026-06.md",
    ".codex/sessions/20260628-0317-zui-migration-validation.md",
]


class PluginValidateRootTestRetirementStatusTests(unittest.TestCase):
    def test_root_plugin_validate_test_file_stays_retired_marker(self):
        source = ROOT_TEST.read_text(encoding="utf-8")

        self.assertLessEqual(
            len(source.splitlines()),
            40,
            "PluginValidate root test file must stay as a retired import marker",
        )
        self.assertIn("Retired PluginValidate root behavior tests.", source)
        self.assertNotIn("def test_", source)

    def test_current_status_docs_record_root_test_retirement(self):
        required_phrases = [
            STATUS_ID,
            "tools/zircon_export/tests/test_plugin_validate.py=9",
            "Retired PluginValidate root behavior tests",
            "root marker 0/0",
            "test_plugin_validate_all_target_test_owner_split",
            "test_all_target_tests_live_in_all_target_test_owner",
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
            "PluginValidate root test retirement status is incomplete:\n"
            + "\n".join(missing),
        )


if __name__ == "__main__":
    unittest.main()
