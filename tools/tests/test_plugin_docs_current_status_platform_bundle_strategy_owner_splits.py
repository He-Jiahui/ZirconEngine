import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path

from tools.tests.plugin_docs_current_status_platform_bundle_support import (
    assert_required_phrases,
    load_platform_bundle_sections,
)


class PluginDocsCurrentStatusPlatformBundleStrategyOwnerTests(unittest.TestCase):
    def setUp(self) -> None:
        self.sections = load_platform_bundle_sections(
            Path(__file__).resolve().parents[2]
        )

    def test_current_export_plan_reflects_platform_bundle_argument_path_owner_split(
        self,
    ):
        assert_required_phrases(
            self,
            self.sections,
            {
                "09 export status": [
                    "plugins_13_m5_t1_platform_bundle_argument_path_owner_split",
                    "platform_bundle_arguments.py",
                    "PlatformBundle argument/path owner",
                ],
                "13 standalone status": [
                    "plugins_13_m5_t1_platform_bundle_argument_path_owner_split",
                    "platform_bundle_arguments.py",
                    "PlatformBundle argument/path owner",
                ],
                "standalone current contract": [
                    "plugins_13_m5_t1_platform_bundle_argument_path_owner_split",
                    "platform_bundle_arguments.py",
                    "argument origin/path resolution diagnostics",
                ],
                "export tooling docs": [
                    "platform_bundle_arguments.py",
                    "PlatformBundle argument/path owner",
                    "argument origin/path resolution diagnostics",
                ],
                "active session notes": [
                    "plugins_13_m5_t1_platform_bundle_argument_path_owner_split",
                    "platform_bundle_arguments.py",
                    "PlatformBundle argument/path owner",
                ],
            },
            "Current export/plugin docs do not reflect PlatformBundle argument/path owner split",
        )

    def test_current_export_plan_reflects_platform_bundle_strategy_handoff_owner_split(
        self,
    ):
        assert_required_phrases(
            self,
            self.sections,
            {
                "09 export status": [
                    "plugins_13_m5_t1_platform_bundle_strategy_handoff_owner_split",
                    "platform_bundle_strategy_handoff.py",
                    "PlatformBundle strategy handoff owner",
                ],
                "13 standalone status": [
                    "plugins_13_m5_t1_platform_bundle_strategy_handoff_owner_split",
                    "platform_bundle_strategy_handoff.py",
                    "PlatformBundle strategy handoff owner",
                ],
                "standalone current contract": [
                    "plugins_13_m5_t1_platform_bundle_strategy_handoff_owner_split",
                    "platform_bundle_strategy_handoff.py",
                    "Validate strategy/native-dynamic handoff diagnostics",
                ],
                "export tooling docs": [
                    "platform_bundle_strategy_handoff.py",
                    "PlatformBundle strategy handoff owner",
                    "Validate strategy/native-dynamic handoff diagnostics",
                ],
                "active session notes": [
                    "plugins_13_m5_t1_platform_bundle_strategy_handoff_owner_split",
                    "platform_bundle_strategy_handoff.py",
                    "PlatformBundle strategy handoff owner",
                ],
            },
            "Current export/plugin docs do not reflect PlatformBundle strategy handoff owner split",
        )


if __name__ == "__main__":
    unittest.main()
