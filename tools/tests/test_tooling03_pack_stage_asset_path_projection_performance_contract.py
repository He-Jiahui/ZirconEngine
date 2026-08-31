"""Performance contract for Pack report asset path arrays."""

from __future__ import annotations

import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report_pack_stage_schema import (
    pack_report_asset_path_array_projection,
)
from tools.zircon_export.pipeline_report_pack_stage_schema import (
    pack_string_array_entry_type_schema_diagnostics,
)
from tools.zircon_export.pipeline_report_pack_trim_schema import (
    pack_asset_path_array_schema_diagnostics,
)
from tools.zircon_export.pipeline_report_schema_string_array import (
    string_array_no_blank_entries_schema_diagnostics,
)


REPO_ROOT = Path(__file__).resolve().parents[2]
OWNER = REPO_ROOT / "tools/zircon_export/pipeline_report_pack_stage_schema.py"


def _legacy(label: str, value: object) -> list[str]:
    return (
        pack_string_array_entry_type_schema_diagnostics(label, value)
        + string_array_no_blank_entries_schema_diagnostics(label, value)
        + pack_asset_path_array_schema_diagnostics(label, value)
    )


class PackStageAssetPathProjectionPerformanceContractTests(unittest.TestCase):
    def test_mixed_values_match_legacy(self) -> None:
        value = ["", " assets/a.bin ", "../unsafe.bin", "assets/a.bin", 7]
        self.assertEqual(
            pack_report_asset_path_array_projection("pack report assets", value),
            _legacy("pack report assets", value),
        )

    def test_projection_has_one_array_loop(self) -> None:
        source = OWNER.read_text(encoding="utf-8")
        helper = source[source.index("def pack_report_asset_path_array_projection("):source.index("def pack_report_schema_diagnostics(")]
        self.assertEqual(helper.count("for index, item in enumerate(value):"), 1)


if __name__ == "__main__":
    unittest.main()
