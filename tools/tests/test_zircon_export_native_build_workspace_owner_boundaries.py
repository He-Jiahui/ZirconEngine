import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
NATIVE_BUILD = REPO_ROOT / "tools/zircon_export/native_build.py"
NATIVE_BUILD_EXECUTION = REPO_ROOT / "tools/zircon_export/native_build_execution.py"
NATIVE_BUILD_WORKSPACE = REPO_ROOT / "tools/zircon_export/native_build_workspace.py"


class ZirconExportNativeBuildWorkspaceOwnerBoundaryTests(unittest.TestCase):
    def test_workspace_metadata_helpers_live_in_dedicated_owner(self):
        self.assertTrue(
            NATIVE_BUILD_WORKSPACE.exists(),
            "NativeBuild TOML/workspace crate metadata needs a dedicated owner",
        )
        native_build_text = NATIVE_BUILD.read_text(encoding="utf-8")
        native_build_execution_text = NATIVE_BUILD_EXECUTION.read_text(encoding="utf-8")
        workspace_text = NATIVE_BUILD_WORKSPACE.read_text(encoding="utf-8")

        for function_name in (
            "resolve_native_build_path",
            "native_dynamic_workspace_crate_index",
            "native_dynamic_cdylib_crate_index",
            "native_dynamic_cdylib_crate_index_from_workspace",
            "native_dynamic_crate_type_schema_invalid",
            "native_dynamic_source_cdylib_crate_name",
            "read_toml",
            "dedupe",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                native_build_text,
                f"{function_name} belongs in the workspace metadata owner",
            )
            self.assertIn(f"def {function_name}(", workspace_text)

        self.assertNotIn(
            "NATIVE_BUILD_CDYLIB_CRATE_TYPE =",
            native_build_text,
            "cdylib crate-type constant belongs with workspace metadata parsing",
        )
        self.assertIn("NATIVE_BUILD_CDYLIB_CRATE_TYPE =", workspace_text)
        self.assertIn(
            "from .native_build_workspace import",
            native_build_text,
            "native build-plan owner should consume the workspace metadata owner",
        )
        self.assertIn(
            "from .native_build_workspace import",
            native_build_execution_text,
            "NativeBuild execution owner should consume the workspace metadata owner",
        )
        self.assertNotIn(
            "from .native_build import",
            workspace_text,
            "workspace metadata owner must not import NativeBuild plan or execution owners",
        )

    def test_workspace_metadata_consumers_import_owner_directly(self):
        for relative_path in (
            "tools/zircon_export/plugin_build.py",
            "tools/zircon_export/plugin_package_source.py",
            "tools/zircon_export/plugin_validate.py",
            "tools/zircon_export/plugin_validate_dist_crate.py",
            "tools/zircon_export/plugin_validate_distribution_modules.py",
            "tools/zircon_export/plugin_validate_engine_version.py",
            "tools/zircon_export/plugin_validate_target_discovery.py",
        ):
            text = (REPO_ROOT / relative_path).read_text(encoding="utf-8")
            self.assertIn(
                "from .native_build_workspace import",
                text,
                f"{relative_path} should consume workspace metadata directly",
            )

    def test_native_build_execution_owner_stays_under_large_file_threshold(self):
        line_count = len(NATIVE_BUILD.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            line_count,
            560,
            "NativeBuild execution owner should stay below 560 lines after workspace metadata split",
        )


if __name__ == "__main__":
    unittest.main()
