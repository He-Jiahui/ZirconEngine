import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path


REPO_ROOT = Path(__file__).resolve().parents[2]
STATUS_ID = "plugins_13_m5_t1_plugin_build_matrix_real_build_rerun_passed"
REQUIRED_STATUS_PHRASES = [
    STATUS_ID,
    "plugin build <id>",
    "39 isolated plugin builds",
    "D:\\cargo-targets\\zircon-plugin-build-matrix-isolated-0703-codex-summary.jsonl",
    "D:\\cargo-targets\\zircon-plugin-build-matrix-isolated-0703-codex.log",
    "summary_count=39",
    "failed_exit_count=0",
    "artifact_failures=[]",
    "isolated_out_roots=39",
    "package_dir_count=39",
    "native_plugins.toml",
    "plugins/native_plugins.toml",
    "native_dynamic_package.toml",
    "plugin validate --all",
    "target_count=39",
    "failed_count=0",
    "diagnostics=0",
    "manifest_schema_violations=0",
    "retired_ui_asset_files=0",
    "zui-only-clean",
    "tools/tests/test_plugin_docs_current_status_plugin_build_matrix_real_build_rerun.py",
    "不声明 Hub/editor E2E",
]

STATUS_DOCS = [
    "docs/plans/zircon_plugins/09-export-publishing.md",
    "docs/plans/zircon_plugins/13-standalone-plugin-build.md",
    "docs/zircon_plugins/plugin-standalone-build.md",
    "docs/plans/engine-code-structure-convention.md",
    "docs/plans/engine-code-review-findings-2026-06.md",
]


class PluginDocsCurrentStatusPluginBuildMatrixRealBuildRerunTests(unittest.TestCase):
    def test_current_docs_record_plugin_build_matrix_real_build_rerun(self) -> None:
        failures: list[str] = []
        for relative_path in STATUS_DOCS:
            text = (REPO_ROOT / relative_path).read_text(encoding="utf-8")
            for phrase in REQUIRED_STATUS_PHRASES:
                if phrase not in text:
                    failures.append(f"{relative_path}: missing {phrase}")

        if failures:
            self.fail(
                "Current plugin docs do not record plugin build matrix real build rerun:\n"
                + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
