"""Boundary tests for pack delta semantic diagnostics ownership."""

from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PACK_DELTA_SCHEMA = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_pack_delta_schema.py"
)
PACK_STAGE_SCHEMA = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_pack_stage_schema.py"
)
PACK_DELTA_SEMANTICS = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_pack_delta_semantics.py"
)
PACK_DELTA_ASSET_SET_SEMANTICS = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_pack_delta_asset_set_semantics.py"
)

MOVED_FUNCTIONS = (
    "pack_report_delta_publication_diagnostics",
    "report_path_is_present",
    "pack_report_delta_manifest_count_diagnostics",
    "pack_report_delta_target_manifest_diagnostics",
)


def _line_count(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    return text.count("\n") + int(bool(text))


class PackDeltaSemanticsOwnerBoundaryTests(unittest.TestCase):
    def test_pack_delta_semantics_owner_exists(self):
        self.assertTrue(
            PACK_DELTA_SEMANTICS.exists(),
            "Pack delta semantic diagnostics owner file is missing",
        )

    def test_delta_semantics_are_owned_by_semantics_module(self):
        schema_text = PACK_DELTA_SCHEMA.read_text(encoding="utf-8")
        owner_text = (
            PACK_DELTA_SEMANTICS.read_text(encoding="utf-8")
            if PACK_DELTA_SEMANTICS.exists()
            else ""
        )

        failures: list[str] = []
        for function_name in MOVED_FUNCTIONS:
            definition = f"def {function_name}("
            if definition in schema_text:
                failures.append(f"delta schema still owns {function_name}")
            if definition not in owner_text:
                failures.append(f"semantics owner missing {function_name}")

        if failures:
            self.fail("\n".join(failures))

    def test_schema_imports_leaf_owner_without_reverse_import(self):
        schema_text = PACK_DELTA_SCHEMA.read_text(encoding="utf-8")
        stage_text = PACK_STAGE_SCHEMA.read_text(encoding="utf-8")
        owner_text = (
            PACK_DELTA_SEMANTICS.read_text(encoding="utf-8")
            if PACK_DELTA_SEMANTICS.exists()
            else ""
        )
        asset_set_text = (
            PACK_DELTA_ASSET_SET_SEMANTICS.read_text(encoding="utf-8")
            if PACK_DELTA_ASSET_SET_SEMANTICS.exists()
            else ""
        )

        self.assertIn(
            "from .pipeline_report_pack_delta_asset_set_semantics import (",
            schema_text,
        )
        self.assertIn(
            "from .pipeline_report_pack_delta_semantics import (",
            stage_text,
        )
        self.assertNotIn(
            ".pipeline_report_pack_delta_schema",
            owner_text,
        )
        self.assertNotIn(
            ".pipeline_report_pack_delta_schema",
            asset_set_text,
        )

    def test_pack_delta_schema_and_semantics_owner_stay_small(self):
        self.assertLess(_line_count(PACK_DELTA_SCHEMA), 360)
        self.assertTrue(
            PACK_DELTA_SEMANTICS.exists(),
            "Pack delta semantic diagnostics owner file is missing",
        )
        self.assertLess(_line_count(PACK_DELTA_SEMANTICS), 190)


if __name__ == "__main__":
    unittest.main()
