import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
NATIVE_DYNAMIC_STAGE = REPO_ROOT / "tools/zircon_export/native_dynamic.py"
NATIVE_DYNAMIC_MATERIALIZE = (
    REPO_ROOT / "tools/zircon_export/native_dynamic_materialize.py"
)
NATIVE_DYNAMIC_MATERIALIZE_IO = (
    REPO_ROOT / "tools/zircon_export/native_dynamic_materialize_io.py"
)


class ZirconExportNativeDynamicMaterializeIoOwnerBoundaryTests(unittest.TestCase):
    def test_native_dynamic_materialize_io_helpers_live_in_dedicated_owner(self):
        self.assertTrue(
            NATIVE_DYNAMIC_MATERIALIZE_IO.exists(),
            "NativeDynamic materialization IO/path helpers need a dedicated owner",
        )
        stage_text = NATIVE_DYNAMIC_STAGE.read_text(encoding="utf-8")
        materialize_text = NATIVE_DYNAMIC_MATERIALIZE.read_text(encoding="utf-8")
        materialize_io_text = NATIVE_DYNAMIC_MATERIALIZE_IO.read_text(
            encoding="utf-8"
        )

        for function_name in (
            "reset_native_dynamic_plugins_dir",
            "remove_native_dynamic_dir",
            "list_native_dynamic_dir",
            "copy_native_dynamic_file",
            "copy_native_dynamic_tree",
            "resolve_stage_child",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                materialize_text,
                f"{function_name} belongs in the materialize IO/path owner",
            )
            self.assertIn(f"def {function_name}(", materialize_io_text)

        self.assertIn(
            "from .native_dynamic_materialize_io import",
            materialize_text,
            "package materialization should consume the IO/path owner directly",
        )
        self.assertIn(
            "from .native_dynamic_materialize_io import",
            stage_text,
            "stage orchestration should consume reset helpers from the IO/path owner",
        )
        self.assertNotIn(
            "from .native_dynamic_materialize import",
            materialize_io_text,
            "materialize IO/path owner must not import package materialization",
        )
        self.assertNotIn(
            "from .native_dynamic import",
            materialize_io_text,
            "materialize IO/path owner must not import stage orchestration",
        )

    def test_native_dynamic_materialize_owners_stay_under_split_thresholds(self):
        self.assertTrue(
            NATIVE_DYNAMIC_MATERIALIZE_IO.exists(),
            "NativeDynamic materialization IO/path helpers need a dedicated owner",
        )
        materialize_line_count = len(
            NATIVE_DYNAMIC_MATERIALIZE.read_text(encoding="utf-8").splitlines()
        )
        materialize_io_line_count = len(
            NATIVE_DYNAMIC_MATERIALIZE_IO.read_text(encoding="utf-8").splitlines()
        )
        self.assertLess(
            materialize_line_count,
            340,
            "NativeDynamic materialize owner should stay below 340 lines after IO/path split",
        )
        self.assertLess(
            materialize_io_line_count,
            150,
            "NativeDynamic materialize IO/path owner should stay below 150 lines",
        )


if __name__ == "__main__":
    unittest.main()
