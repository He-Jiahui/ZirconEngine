import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
CONVERGENCE_TEST = (
    REPO_ROOT / "tools/tests/test_plugin_docs_current_status_convergence.py"
)
PLUGIN_VALIDATE_DOCS_TEST = (
    REPO_ROOT
    / "tools/tests/test_plugin_docs_current_status_plugin_validate_owner_splits.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_DOCS_TEST = (
    REPO_ROOT
    / "tools/tests/test_plugin_docs_current_status_plugin_validate_feature_provider_status_owner_splits.py"
)
PLUGIN_VALIDATE_DISTRIBUTION_DOCS_TEST = (
    REPO_ROOT
    / "tools/tests/test_plugin_docs_current_status_plugin_validate_distribution_status_owner_splits.py"
)
PLUGIN_VALIDATE_ENTRY_DOCS_TEST = (
    REPO_ROOT
    / "tools/tests/test_plugin_docs_current_status_plugin_validate_entry_status_owner_splits.py"
)


class PluginDocsCurrentStatusPluginValidateOwnerBoundaryTests(unittest.TestCase):
    def test_plugin_validate_docs_status_guards_live_in_dedicated_owner(self):
        convergence_text = CONVERGENCE_TEST.read_text(encoding="utf-8")

        owner_markers = {
            PLUGIN_VALIDATE_DOCS_TEST: (
                "test_current_plugin_docs_reflect_distribution_modules_test_owner",
            ),
            PLUGIN_VALIDATE_FEATURE_PROVIDER_DOCS_TEST: (
                "test_current_export_plan_reflects_feature_provider_projection_compare_owner_split",
                "test_current_plugin_docs_reflect_feature_provider_dependencies_owner_split",
                "test_current_plugin_docs_reflect_projection_optional_owner_split",
                "test_current_plugin_docs_reflect_feature_provider_capabilities_owner_split",
                "test_current_plugin_docs_reflect_feature_provider_distribution_owner_split",
                "test_current_plugin_docs_reflect_feature_provider_extension_owner_split",
            ),
            PLUGIN_VALIDATE_DISTRIBUTION_DOCS_TEST: (
                "test_current_plugin_docs_reflect_dist_crate_dependency_owner_split",
                "test_current_plugin_docs_reflect_distribution_assets_owner_split",
                "test_current_plugin_docs_reflect_distribution_engine_compat_owner_split",
                "test_current_plugin_docs_reflect_distribution_descriptor_symbol_owner_split",
                "test_current_plugin_docs_reflect_distribution_entries_owner_split",
                "test_current_plugin_docs_reflect_distribution_packaging_owner_split",
                "test_current_plugin_docs_reflect_distribution_scalars_owner_split",
                "test_current_plugin_docs_reflect_distribution_module_target_modes_owner_split",
            ),
            PLUGIN_VALIDATE_ENTRY_DOCS_TEST: (
                "test_current_plugin_docs_reflect_plugin_validate_single_target_owner_split",
            ),
        }
        for owner_path, moved_markers in owner_markers.items():
            self.assertTrue(
                owner_path.exists(),
                f"{owner_path} should cover a PluginValidate docs status owner",
            )
            owner_text = owner_path.read_text(encoding="utf-8")
            for marker in moved_markers:
                self.assertNotIn(
                    marker,
                    convergence_text,
                    f"{marker} should move out of the broad convergence test owner",
                )
                self.assertIn(
                    marker,
                    owner_text,
                    f"{marker} should be covered by {owner_path}",
                )
            self.assertLessEqual(
                len(owner_text.splitlines()),
                700,
                f"{owner_path} should stay data-driven and focused",
            )

        self.assertLessEqual(
            len(convergence_text.splitlines()),
            6600,
            "the broad current-status docs convergence test must shrink after the owner split",
        )


if __name__ == "__main__":
    unittest.main()
