import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path


REPO_ROOT = Path(__file__).resolve().parents[2]
STATUS_ID = "plugins_13_m5_t1_plugin_workspace_locked_all_targets_test_execution_passed"
FINAL_COMMAND = (
    "CARGO_PROFILE_DEV_DEBUG=0 cargo test --manifest-path "
    "zircon_plugins\\Cargo.toml --workspace --locked --all-targets --jobs 1 "
    "--target-dir E:\\cargo-targets\\zircon-plugin-net-runtime-worker-rerun-0703 "
    "--message-format short --color never -- --nocapture --test-threads=1"
)
STATUS_DOCS = [
    "docs/plans/zircon_plugins/09-export-publishing.md",
    "docs/plans/zircon_plugins/13-standalone-plugin-build.md",
    "docs/zircon_plugins/plugin-standalone-build.md",
    "docs/plans/engine-code-structure-convention.md",
    "docs/plans/engine-code-review-findings-2026-06.md",
    "docs/plans/zircon_editor/editor_ui/11-zui-suffix-convergence-and-ui-toml-retirement.md",
    ".codex/sessions/20260628-0317-zui-migration-validation.md",
]
MODULE_DOCS = [
    "docs/zircon_plugins/sound/runtime.md",
    "docs/zircon_plugins/net/runtime.md",
    "docs/assets-and-rendering/render-framework-architecture.md",
]
REQUIRED_COMBINED_PHRASES = [
    FINAL_COMMAND,
    "E:\\cargo-targets\\zircon-plugin-workspace-test-0703-codex-rerun13-final.status.json",
    "E:\\cargo-targets\\zircon-plugin-workspace-test-0703-codex-rerun13-final.out.log",
    "E:\\cargo-targets\\zircon-plugin-workspace-test-0703-codex-rerun13-final.err.log",
    "ExitCode=0",
    "StartedAt=2026-07-04T00:16:01.5688471+08:00",
    "FinishedAt=2026-07-04T00:47:52.6590709+08:00",
    "ElapsedSeconds=1911.09",
    "zircon_plugin_sound_runtime --lib",
    "368/368",
    "zircon_plugin_texture_importer_runtime --all-targets",
    "144/144",
    "21/21",
    "env!(\"CARGO_MANIFEST_DIR\")",
    "final master gain",
    "SynthParameter",
    "HRTF preview ownership",
    "ShaderIdePreviewVariant",
    "tools/tests/test_plugin_docs_current_status_plugin_workspace_test_execution_passed.py",
    "不声明 Hub/editor E2E",
]


class PluginDocsCurrentStatusPluginWorkspaceTestExecutionPassedTests(
    unittest.TestCase
):
    def test_current_docs_record_passed_workspace_test_execution(self) -> None:
        failures: list[str] = []
        combined_text_parts: list[str] = []
        for relative_path in [*STATUS_DOCS, *MODULE_DOCS]:
            text = (REPO_ROOT / relative_path).read_text(encoding="utf-8")
            combined_text_parts.append(text)
            if STATUS_ID not in text:
                failures.append(f"{relative_path}: missing {STATUS_ID}")

        combined_text = "\n".join(combined_text_parts)
        for phrase in REQUIRED_COMBINED_PHRASES:
            if phrase not in combined_text:
                failures.append(f"combined docs: missing {phrase}")

        self.assertFalse(
            failures,
            "Current plugin docs do not record the passed workspace test execution:\n"
            + "\n".join(failures),
        )


if __name__ == "__main__":
    unittest.main()
