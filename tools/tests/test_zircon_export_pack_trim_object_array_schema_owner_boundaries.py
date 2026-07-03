import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PACK_TRIM_SCHEMA = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_pack_trim_schema.py"
)
PACK_TRIM_OBJECT_ARRAY_SCHEMA = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_pack_trim_object_array_schema.py"
)


class ZirconExportPackTrimObjectArraySchemaOwnerBoundaryTests(unittest.TestCase):
    def test_trim_object_array_schema_lives_in_dedicated_owner(self):
        self.assertTrue(
            PACK_TRIM_OBJECT_ARRAY_SCHEMA.exists(),
            "Pack trim_report object-array schema needs a dedicated owner",
        )
        parent_text = PACK_TRIM_SCHEMA.read_text(encoding="utf-8")
        object_array_text = PACK_TRIM_OBJECT_ARRAY_SCHEMA.read_text(encoding="utf-8")

        moved_functions = (
            "pack_trimmed_assets_are_schema_clean",
            "trim_reason_is_schema_clean",
            "trim_report_missing_dependencies_are_schema_clean",
            "pack_trimmed_assets_schema_diagnostics",
            "pack_missing_dependencies_schema_diagnostics",
            "pack_optional_asset_path_schema_diagnostics",
            "validate_trim_reason_schema_diagnostics",
        )
        for function_name in moved_functions:
            self.assertNotIn(
                f"def {function_name}(",
                parent_text,
                f"{function_name} belongs in the Pack trim object-array schema owner",
            )
            self.assertIn(f"def {function_name}(", object_array_text)

        moved_constants = (
            "PACK_TRIMMED_ASSET_FIELDS",
            "PACK_MISSING_DEPENDENCY_FIELDS",
            "PACK_MISSING_DEPENDENCY_STRING_FIELDS",
            "PACK_TRIM_REASON_OBJECT_FIELDS",
        )
        for constant_name in moved_constants:
            self.assertNotIn(
                f"{constant_name} =",
                parent_text,
                f"{constant_name} belongs in the Pack trim object-array schema owner",
            )
            self.assertIn(f"{constant_name} =", object_array_text)

        self.assertIn(
            "from .pipeline_report_pack_trim_object_array_schema import (",
            parent_text,
            "Pack trim schema should consume the object-array schema owner directly",
        )
        self.assertNotIn(
            "pipeline_report_pack_trim_schema",
            object_array_text,
            "Pack trim object-array schema owner must not import the parent schema owner",
        )

    def test_pack_trim_schema_owners_stay_under_line_budgets(self):
        self.assertTrue(
            PACK_TRIM_OBJECT_ARRAY_SCHEMA.exists(),
            "Pack trim_report object-array schema needs a dedicated owner",
        )
        parent_line_count = len(PACK_TRIM_SCHEMA.read_text(encoding="utf-8").splitlines())
        object_array_line_count = len(
            PACK_TRIM_OBJECT_ARRAY_SCHEMA.read_text(encoding="utf-8").splitlines()
        )
        self.assertLess(
            parent_line_count,
            310,
            "Pack trim schema parent should stay below 310 lines after object-array split",
        )
        self.assertLess(
            object_array_line_count,
            190,
            "Pack trim object-array schema owner should stay below 190 lines",
        )


if __name__ == "__main__":
    unittest.main()
