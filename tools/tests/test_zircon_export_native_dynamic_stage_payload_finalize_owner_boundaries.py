import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
NATIVE_DYNAMIC_STAGE = REPO_ROOT / "tools/zircon_export/native_dynamic.py"
NATIVE_DYNAMIC_MATERIALIZE = (
    REPO_ROOT / "tools/zircon_export/native_dynamic_materialize.py"
)
NATIVE_DYNAMIC_STAGE_PAYLOAD_FINALIZE = (
    REPO_ROOT / "tools/zircon_export/native_dynamic_stage_payload_finalize.py"
)


class ZirconExportNativeDynamicStagePayloadFinalizeOwnerBoundaryTests(
    unittest.TestCase
):
    def test_stage_payload_finalization_lives_in_finalize_owner(self):
        self.assertTrue(
            NATIVE_DYNAMIC_STAGE_PAYLOAD_FINALIZE.exists(),
            "NativeDynamic stage payload finalization needs a dedicated owner",
        )
        stage_text = NATIVE_DYNAMIC_STAGE.read_text(encoding="utf-8")
        materialize_text = NATIVE_DYNAMIC_MATERIALIZE.read_text(encoding="utf-8")
        finalize_text = NATIVE_DYNAMIC_STAGE_PAYLOAD_FINALIZE.read_text(
            encoding="utf-8"
        )

        for function_name in (
            "finalize_native_dynamic_stage_payload",
            "finalize_native_dynamic_package_reports",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                materialize_text,
                f"{function_name} belongs in the stage payload finalize owner",
            )
            self.assertIn(f"def {function_name}(", finalize_text)

        self.assertIn(
            "from .native_dynamic_stage_payload_finalize import",
            stage_text,
            "NativeDynamic stage orchestration should consume the finalize owner",
        )
        self.assertNotIn(
            "from .native_dynamic_materialize import finalize_native_dynamic_stage_payload",
            stage_text,
            "stage orchestration should not borrow finalization through materialize",
        )
        self.assertNotIn(
            "from .native_dynamic_materialize import",
            finalize_text,
            "finalize owner must not import the package materialization owner",
        )

    def test_native_dynamic_materialize_owner_stays_under_split_threshold(self):
        line_count = len(
            NATIVE_DYNAMIC_MATERIALIZE.read_text(encoding="utf-8").splitlines()
        )
        self.assertLess(
            line_count,
            450,
            "NativeDynamic materialize owner should stay below the split threshold",
        )


if __name__ == "__main__":
    unittest.main()
