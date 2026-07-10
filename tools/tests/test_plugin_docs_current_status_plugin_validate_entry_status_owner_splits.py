import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path

from tools.tests.plugin_docs_current_status_plugin_validate_support import (
    assert_required_phrases,
    current_doc_sections,
    plugin_validate_status_requirements,
)


class PluginDocsCurrentStatusPluginValidateEntryOwnerTests(unittest.TestCase):
    def test_current_plugin_docs_reflect_plugin_validate_single_target_owner_split(self):
        assert_required_phrases(
            self,
            current_doc_sections(Path(__file__).resolve().parents[2]),
            plugin_validate_status_requirements(
                "plugins_13_m5_t1_plugin_validate_single_target_owner_split",
                "plugin_validate_single_target.py",
                "PluginValidate single-target owner",
                "single-target validation orchestration",
            ),
            "Current plugin docs do not reflect PluginValidate single-target owner split",
        )


if __name__ == "__main__":
    unittest.main()
