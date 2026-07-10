import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path


REPO_ROOT = Path(__file__).resolve().parents[2]
STATUS_ID = "plugins_13_m5_t1_plugin_workspace_test_compile_attempt_unaccepted"
REQUIRED_STATUS_PHRASES = [
    STATUS_ID,
    "cargo test --manifest-path zircon_plugins\\Cargo.toml --workspace --locked --all-targets --no-run",
    "D:\\cargo-targets\\zircon-plugin-workspace-test-0703-codex-rerun",
    "D:\\cargo-targets\\zircon-plugin-workspace-test-0703-codex-rerun.out.log",
    "D:\\cargo-targets\\zircon-plugin-workspace-test-0703-codex-rerun.err.log",
    "command timed out after 1201s",
    "50GB",
    "Cargo/rustc",
    "tools/tests/test_plugin_docs_current_status_plugin_workspace_test_compile_attempt_unaccepted.py",
    "cargo test --no-run",
]

STATUS_DOCS = [
    "docs/plans/zircon_plugins/09-export-publishing.md",
    "docs/plans/zircon_plugins/13-standalone-plugin-build.md",
    "docs/zircon_plugins/plugin-standalone-build.md",
    "docs/plans/engine-code-structure-convention.md",
    "docs/plans/engine-code-review-findings-2026-06.md",
    ".codex/sessions/20260628-0317-zui-migration-validation.md",
]


class PluginDocsCurrentStatusPluginWorkspaceTestCompileAttemptTests(
    unittest.TestCase
):
    def test_current_docs_record_unaccepted_test_compile_attempt(self) -> None:
        failures: list[str] = []
        for relative_path in STATUS_DOCS:
            text = (REPO_ROOT / relative_path).read_text(encoding="utf-8")
            for phrase in REQUIRED_STATUS_PHRASES:
                if phrase not in text:
                    failures.append(f"{relative_path}: missing {phrase}")

        if failures:
            self.fail(
                "Current plugin docs do not record the unaccepted test compile attempt:\n"
                + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
