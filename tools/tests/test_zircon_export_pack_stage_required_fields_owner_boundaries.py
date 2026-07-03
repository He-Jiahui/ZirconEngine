"""Boundary tests for Pack stage report required-field ownership."""

from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PACK_STAGE_SCHEMA = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_pack_stage_schema.py"
)
PACK_STAGE_REQUIRED_FIELDS = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_pack_stage_required_fields.py"
)

MOVED_CONSTANTS = (
    "PACK_REPORT_REQUIRED_NON_FATAL_STRING_FIELDS",
    "PACK_REPORT_REQUIRED_NON_FATAL_INTEGER_FIELDS",
    "PACK_REPORT_REQUIRED_NON_FATAL_STRING_ARRAY_FIELDS",
    "PACK_REPORT_REQUIRED_NON_FATAL_BOOL_FIELDS",
    "PACK_REPORT_REQUIRED_NON_FATAL_OBJECT_FIELDS",
    "PACK_REPORT_REQUIRED_DELTA_INTEGER_FIELDS",
    "PACK_REPORT_REQUIRED_DELTA_STRING_FIELDS",
    "PACK_REPORT_REQUIRED_DELTA_STRING_ARRAY_FIELDS",
    "PACK_REPORT_REQUIRED_DELTA_TRUE_BOOL_FIELDS",
)
MOVED_FUNCTIONS = ("pack_report_required_field_schema_diagnostics",)


def _line_count(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    return text.count("\n") + int(bool(text))


class PackStageRequiredFieldsOwnerBoundaryTests(unittest.TestCase):
    def test_pack_stage_required_fields_owner_exists(self):
        self.assertTrue(
            PACK_STAGE_REQUIRED_FIELDS.exists(),
            "Pack stage required-fields owner file is missing",
        )

    def test_required_field_members_are_owned_by_required_fields_module(self):
        schema_text = PACK_STAGE_SCHEMA.read_text(encoding="utf-8")
        required_text = (
            PACK_STAGE_REQUIRED_FIELDS.read_text(encoding="utf-8")
            if PACK_STAGE_REQUIRED_FIELDS.exists()
            else ""
        )

        failures: list[str] = []
        for constant_name in MOVED_CONSTANTS:
            definition = f"{constant_name} ="
            if definition in schema_text:
                failures.append(f"Pack stage schema still owns {constant_name}")
            if definition not in required_text:
                failures.append(f"required-fields owner missing {constant_name}")
        for function_name in MOVED_FUNCTIONS:
            definition = f"def {function_name}("
            if definition in schema_text:
                failures.append(f"Pack stage schema still owns {function_name}")
            if definition not in required_text:
                failures.append(f"required-fields owner missing {function_name}")

        if failures:
            self.fail("\n".join(failures))

    def test_pack_stage_schema_imports_required_fields_without_reverse_import(self):
        schema_text = PACK_STAGE_SCHEMA.read_text(encoding="utf-8")
        required_text = (
            PACK_STAGE_REQUIRED_FIELDS.read_text(encoding="utf-8")
            if PACK_STAGE_REQUIRED_FIELDS.exists()
            else ""
        )

        self.assertIn(
            "from .pipeline_report_pack_stage_required_fields import (",
            schema_text,
        )
        self.assertNotIn(
            ".pipeline_report_pack_stage_schema",
            required_text,
        )

    def test_pack_stage_schema_and_required_fields_owner_stay_small(self):
        self.assertLess(_line_count(PACK_STAGE_SCHEMA), 360)
        self.assertTrue(
            PACK_STAGE_REQUIRED_FIELDS.exists(),
            "Pack stage required-fields owner file is missing",
        )
        self.assertLess(_line_count(PACK_STAGE_REQUIRED_FIELDS), 150)


if __name__ == "__main__":
    unittest.main()
