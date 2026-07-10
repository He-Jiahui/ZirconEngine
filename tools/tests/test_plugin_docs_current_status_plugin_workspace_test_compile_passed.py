import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path


REPO_ROOT = Path(__file__).resolve().parents[2]
STATUS_ID = "plugins_13_m5_t1_plugin_workspace_locked_all_targets_test_compile_passed"
REQUIRED_STATUS_PHRASES = [
    STATUS_ID,
    "cargo test --manifest-path zircon_plugins\\Cargo.toml --workspace --locked --all-targets --no-run",
    "E:\\cargo-targets\\zircon-plugin-workspace-test-0703-codex-rerun2",
    "E:\\cargo-targets\\zircon-plugin-workspace-test-0703-codex-rerun2.status.json",
    "E:\\cargo-targets\\zircon-plugin-workspace-test-0703-codex-rerun2.err.log",
    "ExitCode=0",
    "ElapsedSeconds=2225",
    "FinishedAt=2026-07-03T14:50:07.0568604+08:00",
    "Finished `test` profile",
    "ExecutableLines=123",
    "tools/tests/test_plugin_docs_current_status_plugin_workspace_test_compile_passed.py",
    "不声明完整测试执行",
]

STATUS_DOCS = [
    "docs/plans/zircon_plugins/09-export-publishing.md",
    "docs/plans/zircon_plugins/13-standalone-plugin-build.md",
    "docs/zircon_plugins/plugin-standalone-build.md",
    "docs/plans/engine-code-structure-convention.md",
    "docs/plans/engine-code-review-findings-2026-06.md",
    ".codex/sessions/20260628-0317-zui-migration-validation.md",
]


class PluginDocsCurrentStatusPluginWorkspaceTestCompilePassedTests(
    unittest.TestCase
):
    def test_current_docs_record_passed_test_compile_gate(self) -> None:
        failures: list[str] = []
        for relative_path in STATUS_DOCS:
            text = (REPO_ROOT / relative_path).read_text(encoding="utf-8")
            for phrase in REQUIRED_STATUS_PHRASES:
                if phrase not in text:
                    failures.append(f"{relative_path}: missing {phrase}")

        if failures:
            self.fail(
                "Current plugin docs do not record the passed test compile gate:\n"
                + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
