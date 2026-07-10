import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path


REPO_ROOT = Path(__file__).resolve().parents[2]
STATUS_ID = "plugins_13_m5_t1_native_dynamic_fixture_real_plugin_build_rerun_passed"
REQUIRED_STATUS_PHRASES = [
    STATUS_ID,
    "python -m tools.zircon_export plugin build native_dynamic_fixture --form dist --platform windows-x86_64 --mode debug",
    "zircon_plugin_native_dynamic_fixture_native",
    "D:\\cargo-targets\\zircon-plugin-build-native-dynamic-fixture-0703-codex-out",
    "native_dynamic_package.toml",
    "native_plugins.toml",
    "native_dynamic_fixture.sig",
    "native_dynamic_fixture.zrpack",
    "payload file_count=5",
    "fatal=false",
    "diagnostics=[]",
    "tools/tests/test_plugin_docs_current_status_native_dynamic_fixture_real_build_rerun.py",
    "不声明 Hub/editor E2E、完整 export matrix",
]

STATUS_DOCS = [
    "docs/plans/zircon_plugins/09-export-publishing.md",
    "docs/plans/zircon_plugins/13-standalone-plugin-build.md",
    "docs/zircon_plugins/plugin-standalone-build.md",
    "docs/plans/engine-code-structure-convention.md",
    "docs/plans/engine-code-review-findings-2026-06.md",
    ".codex/sessions/20260628-0317-zui-migration-validation.md",
]


class PluginDocsCurrentStatusNativeDynamicFixtureRealBuildRerunTests(unittest.TestCase):
    def test_current_docs_record_native_dynamic_fixture_real_build_rerun(self) -> None:
        failures: list[str] = []
        for relative_path in STATUS_DOCS:
            text = (REPO_ROOT / relative_path).read_text(encoding="utf-8")
            for phrase in REQUIRED_STATUS_PHRASES:
                if phrase not in text:
                    failures.append(f"{relative_path}: missing {phrase}")

        if failures:
            self.fail(
                "Current plugin docs do not record NativeDynamic fixture real build rerun:\n"
                + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
