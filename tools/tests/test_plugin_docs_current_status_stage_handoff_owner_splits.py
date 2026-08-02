import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path

from tools.tests.plugin_docs_current_status_export_template_cook_assets_support import (
    assert_required_phrases,
    load_export_template_cook_assets_sections,
)


class PluginDocsCurrentStatusStageHandoffOwnerSplitTests(unittest.TestCase):
    def test_current_export_plan_reflects_stage_handoff_strategy_owner_split(self):
        sections = load_export_template_cook_assets_sections(
            Path(__file__).resolve().parents[2]
        )

        assert_required_phrases(
            self,
            sections,
            {
                "09 export status": [
                    "plugins_13_m5_t1_stage_handoff_strategy_owner_split",
                    "stage_handoff_strategy.py",
                    "Stage handoff strategy owner",
                ],
                "13 standalone status": [
                    "plugins_13_m5_t1_stage_handoff_strategy_owner_split",
                    "stage_handoff_strategy.py",
                    "Stage handoff strategy owner",
                ],
                "standalone current contract": [
                    "plugins_13_m5_t1_stage_handoff_strategy_owner_split",
                    "stage_handoff_strategy.py",
                    "Validate strategy report diagnostics",
                ],
                "export tooling docs": [
                    "stage_handoff_strategy.py",
                    "Stage handoff strategy owner",
                    "Validate strategy report diagnostics",
                ],
            },
            "Current export/plugin docs do not reflect stage handoff strategy owner split",
        )


if __name__ == "__main__":
    unittest.main()
