import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
VALIDATION_LOG = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/composites/feedback/"
    "workbench_validation_log.zui"
)
NAVIGATION = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/extension_module_navigation/specs/gameplay_animation.rs"
)
BRIDGE = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/"
    "reference_menu_actions.rs"
)
FEEDBACK = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/"
    "extension_module_feedback.rs"
)


class EditorZuiValidationLogInteractionContractTests(unittest.TestCase):
    def test_validation_controls_have_unique_click_routes(self):
        document = tomllib.loads(VALIDATION_LOG.read_text(encoding="utf-8"))
        nodes = document["nodes"]
        expected = {
            "validation_log_all": "workbench.extension.blend_space.validation.filter_all",
            "validation_log_errors": "workbench.extension.blend_space.validation.filter_errors",
            "validation_log_warnings": "workbench.extension.blend_space.validation.filter_warnings",
            "validation_log_infos": "workbench.extension.blend_space.validation.filter_infos",
            "validation_log_clear": "workbench.extension.blend_space.validation.clear",
        }
        routes = []
        for node_id, route in expected.items():
            events = nodes[node_id].get("events", [])
            self.assertEqual(1, len(events), f"{node_id} must expose one click route")
            self.assertEqual("Click", events[0]["event"])
            self.assertEqual(route, events[0]["route"])
            routes.append(route)
        self.assertEqual(len(routes), len(set(routes)))
        self.assertEqual("0", nodes["validation_log_errors"]["props"]["text"])
        self.assertEqual("1", nodes["validation_log_warnings"]["props"]["text"])
        self.assertEqual("3", nodes["validation_log_infos"]["props"]["text"])

    def test_validation_routes_share_one_navigation_and_feedback_authority(self):
        navigation = NAVIGATION.read_text(encoding="utf-8")
        bridge = BRIDGE.read_text(encoding="utf-8")
        feedback = FEEDBACK.read_text(encoding="utf-8")
        routes = [
            "workbench.extension.blend_space.validation.filter_all",
            "workbench.extension.blend_space.validation.filter_errors",
            "workbench.extension.blend_space.validation.filter_warnings",
            "workbench.extension.blend_space.validation.filter_infos",
            "workbench.extension.blend_space.validation.clear",
        ]
        for route in routes:
            self.assertIn(route, navigation)
            self.assertIn(route, bridge)
            self.assertIn(route, feedback)
        for control_id in [
            "WorkbenchValidationLogAll",
            "WorkbenchValidationLogErrors",
            "WorkbenchValidationLogWarnings",
            "WorkbenchValidationLogInfos",
            "WorkbenchValidationLogClear",
        ]:
            self.assertIn(control_id, navigation)
        for row_id in [
            "WorkbenchValidationLogInfoAxesRow",
            "WorkbenchValidationLogWarningRow",
            "WorkbenchValidationLogInfoRangeRow",
            "WorkbenchValidationLogInfoDuplicatesRow",
        ]:
            self.assertIn(row_id, bridge)


if __name__ == "__main__":
    unittest.main()
