import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
CONVERGENCE_TEST = (
    REPO_ROOT / "tools/tests/test_plugin_docs_current_status_convergence.py"
)
PLUGIN_BUILD_DOCS_TEST = (
    REPO_ROOT
    / "tools/tests/test_plugin_docs_current_status_plugin_build_owner_splits.py"
)


class PluginDocsCurrentStatusPluginBuildOwnerBoundaryTests(unittest.TestCase):
    def test_plugin_build_docs_status_guards_live_in_dedicated_owner(self):
        self.assertTrue(
            PLUGIN_BUILD_DOCS_TEST.exists(),
            "Plugin build docs status guards belong in a focused test owner",
        )
        convergence_text = CONVERGENCE_TEST.read_text(encoding="utf-8")
        plugin_build_docs_text = PLUGIN_BUILD_DOCS_TEST.read_text(encoding="utf-8")

        moved_markers = (
            "test_current_plugin_docs_reflect_build_package_owner_layering",
            "test_current_export_plan_reflects_plugin_build_command_owner_split",
            "test_current_export_plan_reflects_plugin_build_preflight_owner_split",
        )
        for marker in moved_markers:
            self.assertNotIn(
                marker,
                convergence_text,
                f"{marker} should move out of the broad convergence test owner",
            )
            self.assertIn(
                marker,
                plugin_build_docs_text,
                f"{marker} should be covered by the PluginBuild docs owner",
            )

        self.assertLessEqual(
            len(convergence_text.splitlines()),
            4300,
            "the broad current-status docs convergence test must shrink after the owner split",
        )
        self.assertLessEqual(
            len(plugin_build_docs_text.splitlines()),
            260,
            "the PluginBuild docs status owner should stay data-driven and focused",
        )


if __name__ == "__main__":
    unittest.main()
