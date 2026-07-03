import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
NATIVE_BUILD = REPO_ROOT / "tools/zircon_export/native_build.py"
NATIVE_BUILD_EXECUTION = REPO_ROOT / "tools/zircon_export/native_build_execution.py"
NATIVE_DYNAMIC = REPO_ROOT / "tools/zircon_export/native_dynamic.py"


class ZirconExportNativeBuildExecutionOwnerBoundaryTests(unittest.TestCase):
    def test_build_execution_helpers_live_in_dedicated_owner(self):
        self.assertTrue(
            NATIVE_BUILD_EXECUTION.exists(),
            "NativeBuild Cargo execution/artifact copy needs a dedicated owner",
        )
        native_build_text = NATIVE_BUILD.read_text(encoding="utf-8")
        execution_text = NATIVE_BUILD_EXECUTION.read_text(encoding="utf-8")

        for function_name in (
            "execute_native_dynamic_build_plan",
            "execute_native_dynamic_package_build",
            "native_dynamic_materialized_package_dirs",
            "copy_native_dynamic_build_sidecars",
            "native_dynamic_build_execution_report",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                native_build_text,
                f"{function_name} belongs in the NativeBuild execution owner",
            )
            self.assertIn(f"def {function_name}(", execution_text)

        self.assertNotIn(
            "import subprocess",
            native_build_text,
            "NativeBuild plan owner must not launch Cargo processes",
        )
        self.assertNotIn(
            "import shutil",
            native_build_text,
            "NativeBuild plan owner must not copy build artifacts",
        )
        self.assertIn("import subprocess", execution_text)
        self.assertIn("import shutil", execution_text)
        self.assertIn("from .native_build_workspace import", execution_text)
        self.assertNotIn(
            "from .native_build import",
            execution_text,
            "NativeBuild execution owner must not import the plan owner",
        )

    def test_native_dynamic_stage_imports_plan_and_execution_owners_directly(self):
        native_dynamic_text = NATIVE_DYNAMIC.read_text(encoding="utf-8")

        self.assertIn(
            "from .native_build import native_dynamic_build_plan",
            native_dynamic_text,
            "NativeDynamic stage should import build-plan assembly from plan owner",
        )
        self.assertIn(
            "from .native_build_execution import",
            native_dynamic_text,
            "NativeDynamic stage should import build execution from execution owner",
        )
        self.assertNotIn(
            "execute_native_dynamic_build_plan,\n    native_dynamic_build_plan,",
            native_dynamic_text,
            "NativeDynamic stage should not borrow execution through native_build.py",
        )

    def test_native_build_plan_and_execution_owners_stay_below_split_thresholds(self):
        native_build_line_count = len(
            NATIVE_BUILD.read_text(encoding="utf-8").splitlines()
        )
        execution_line_count = len(
            NATIVE_BUILD_EXECUTION.read_text(encoding="utf-8").splitlines()
        )
        self.assertLess(
            native_build_line_count,
            190,
            "NativeBuild plan owner should stay below 190 lines after execution split",
        )
        self.assertLess(
            execution_line_count,
            280,
            "NativeBuild execution owner should stay focused below 280 lines",
        )


if __name__ == "__main__":
    unittest.main()
