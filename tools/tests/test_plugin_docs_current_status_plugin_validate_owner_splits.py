import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path

from tools.tests.plugin_docs_current_status_plugin_validate_support import (
    assert_required_phrases,
    current_doc_sections,
    section,
)


class PluginDocsCurrentStatusPluginValidateOwnerSplitsTests(unittest.TestCase):
    def test_current_plugin_docs_reflect_distribution_modules_test_owner(self):
        repo_root = Path(__file__).resolve().parents[2]
        sections = current_doc_sections(repo_root)
        standalone_text = (
            repo_root / "docs/zircon_plugins/plugin-standalone-build.md"
        ).read_text(encoding="utf-8")
        export_text = (
            repo_root / "docs/cli-and-tooling/zircon-export-tool.md"
        ).read_text(encoding="utf-8")
        session_text = (
            repo_root
            / ".codex/sessions/20260628-0317-zui-migration-validation.md"
        ).read_text(encoding="utf-8")

        target_sections = {
            "standalone current contract": section(
                standalone_text,
                "## 6. 注册跨 ABI 编组",
                "## 9. 当前落地状态",
            ),
            "export validate owner docs": section(
                export_text,
                "Distribution contract behavior tests live in",
                "Dist crate workspace-member resolution and Cargo-manifest preflight",
            ),
            "active session distribution owner notes": section(
                session_text,
                "- `test_plugin_validate_distribution_contract.py` now owns",
                "- `tools/zircon_build.py` now treats",
            ),
        }
        sections.update(target_sections)

        failures: list[str] = []
        stale_phrases = [
            "dist crate/module binding",
            "module binding 与 all-target",
        ]
        for section_name, section_text in target_sections.items():
            for phrase in stale_phrases:
                if phrase in section_text:
                    failures.append(f"{section_name}: stale {phrase}")

        assert_required_phrases(
            self,
            sections,
            {
                "standalone current contract": [
                    "test_plugin_validate_distribution_modules.py",
                    "root/feature-provider dist_crate module binding",
                    "dist crate Cargo preflight",
                ],
                "export validate owner docs": [
                    "test_plugin_validate_distribution_modules.py",
                    "root and feature-provider module cases",
                    "root/general validation, dist crate Cargo preflight",
                ],
                "active session distribution owner notes": [
                    "test_plugin_validate_distribution_modules.py",
                    "distribution module behavior tests",
                    "`test_plugin_validate.py` is now 284 lines",
                    "`test_plugin_validate_feature_provider.py` is 333 lines",
                ],
            },
            "Current plugin docs do not reflect distribution modules test ownership",
        )
        if failures:
            self.fail(
                "Current plugin docs contain stale distribution modules ownership text:\n"
                + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
