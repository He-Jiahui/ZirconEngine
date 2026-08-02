import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path


class PlatformBundleTemplateDocsStatusOwnerSplitDocsTests(unittest.TestCase):
    def test_platform_bundle_template_docs_status_owner_split_is_recorded(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        required_files = [
            "docs/plans/zircon_plugins/13-standalone-plugin-build.md",
            "docs/plans/zircon_plugins/09-export-publishing.md",
            "docs/zircon_plugins/plugin-standalone-build.md",
            "docs/cli-and-tooling/zircon-export-tool.md",
            "docs/plans/engine-code-structure-convention.md",
            "docs/plans/engine-code-review-findings-2026-06.md",
        ]
        required_phrases = [
            "plugins_13_m5_t1_platform_bundle_template_docs_status_owner_split",
            "test_plugin_docs_current_status_platform_bundle_template_owner_splits.py",
            "test_plugin_docs_current_status_convergence.py",
            "PlatformBundle template",
            "current-status",
            "5392",
            "181",
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
