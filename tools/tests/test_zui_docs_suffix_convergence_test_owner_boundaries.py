from pathlib import Path
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
ROOT_GUARD = REPO_ROOT / "tools/tests/test_zui_docs_suffix_convergence.py"
STATUS_GUARD = REPO_ROOT / "tools/tests/test_zui_docs_suffix_status_guards.py"

STATUS_METHODS = (
    "test_structure_and_review_status_include_latest_zui_authority_guards",
    "test_production_zui_asset_text_suffix_guard_status_is_recorded",
    "test_editor_ui_asset_editing_fixture_zui_guard_status_is_recorded",
    "test_editor_host_manager_ui_asset_fixture_zui_guard_status_is_recorded",
    "test_editor_ui_asset_editor_ui_tests_fixture_zui_guard_status_is_recorded",
    "test_editor_host_theme_tooling_fixture_zui_guard_status_is_recorded",
    "test_runtime_ui_prototype_store_zui_guard_status_is_recorded",
    "test_editor_ui_component_adapter_fixture_zui_guard_status_is_recorded",
    "test_editor_retained_host_projection_zui_guard_status_is_recorded",
    "test_editor_extension_contract_zui_guard_status_is_recorded",
    "test_editor_view_projection_zui_guard_status_is_recorded",
    "test_runtime_extension_component_zui_guard_status_is_recorded",
    "test_runtime_asset_ui_reference_zui_guard_status_is_recorded",
)


class ZuiDocsSuffixConvergenceTestOwnerBoundaryTests(unittest.TestCase):
    def test_status_guards_live_in_focused_owner(self) -> None:
        root_source = ROOT_GUARD.read_text(encoding="utf-8")
        self.assertLessEqual(
            len(root_source.splitlines()),
            380,
            "root zui docs suffix convergence guard should only own current authority docs",
        )

        self.assertTrue(
            STATUS_GUARD.exists(),
            "status-recording guards should live in a focused owner file",
        )
        status_source = STATUS_GUARD.read_text(encoding="utf-8")
        self.assertLessEqual(
            len(status_source.splitlines()),
            620,
            "zui docs suffix status guard should stay below the focused owner budget",
        )
        for method in STATUS_METHODS:
            self.assertNotIn(method, root_source)
            self.assertIn(method, status_source)


if __name__ == "__main__":
    unittest.main()
