from __future__ import annotations

import re
import unittest
from pathlib import Path

from tools.editor_asset_browser_slot_reuse_pressure import run


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "workbench"
    / "asset_content_layout"
    / "browser_virtualization.rs"
)


def logical_index_for_slot(
    *,
    item_count: int,
    materialized_item_count: int,
    columns: int,
    start_row: int,
    slot_index: int,
) -> int | None:
    if materialized_item_count == 0 or slot_index >= materialized_item_count:
        return None
    row_count = (materialized_item_count + columns - 1) // columns
    max_start_row = max(0, (item_count + columns - 1) // columns - row_count)
    start_row = min(start_row, max_start_row)
    physical_row, column = divmod(slot_index, columns)
    first_physical_row = start_row % row_count
    row_offset = (physical_row + row_count - first_physical_row) % row_count
    logical_row = start_row + row_offset
    logical_index = logical_row * columns + column
    return logical_index if logical_index < item_count else None


class AssetBrowserSlotReusePerformanceContract(unittest.TestCase):
    def test_pressure_model_counts_only_changed_slot_assignments(self) -> None:
        report = run(
            materialized_item_count=54,
            columns=6,
            scroll_update_count=4_096,
            large_seek_count=64,
        )

        self.assertEqual(
            report["retired_window_relative_binding"]["slot_rebinds"],
            221_184,
        )
        self.assertEqual(report["row_modulo_binding"]["slot_rebinds"], 27_648)
        self.assertEqual(report["delta"]["slot_rebind_reduction_ratio"], 8.0)
        self.assertFalse(report["interpretation"]["runtime_cpu_measured"])

    def test_one_row_scroll_rebinds_only_the_entering_physical_row(self) -> None:
        initial = [
            logical_index_for_slot(
                item_count=10_000,
                materialized_item_count=54,
                columns=6,
                start_row=0,
                slot_index=slot,
            )
            for slot in range(54)
        ]
        scrolled = [
            logical_index_for_slot(
                item_count=10_000,
                materialized_item_count=54,
                columns=6,
                start_row=1,
                slot_index=slot,
            )
            for slot in range(54)
        ]

        changed = sum(left != right for left, right in zip(initial, scrolled))
        self.assertEqual(changed, 6)
        self.assertEqual(sorted(scrolled), list(range(6, 60)))
        self.assertTrue(
            all(
                logical is None or logical % 6 == slot % 6
                for slot, logical in enumerate(scrolled)
            )
        )

    def test_bottom_window_backfills_the_complete_retained_pool(self) -> None:
        bound = [
            logical_index_for_slot(
                item_count=10,
                materialized_item_count=6,
                columns=2,
                start_row=100,
                slot_index=slot,
            )
            for slot in range(6)
        ]

        self.assertEqual(sorted(bound), list(range(4, 10)))

    def test_rust_binding_uses_row_modulo_and_relative_slot_geometry(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        binding = source[
            source.index("    pub(super) fn binding(") : source.index(
                "    fn materialized_row_count(&self) -> usize"
            )
        ]

        self.assertIn("fn materialized_row_count(&self) -> usize", source)
        self.assertIn("fn logical_index_for_slot(", source)
        self.assertIn("let first_physical_row = start_row % materialized_row_count;", source)
        self.assertIn("let y_offset_rows = logical_row.checked_sub(physical_row)?;", source)
        self.assertNotRegex(
            binding,
            re.compile(
                r"let logical_index = start_row\s*\.checked_mul\(self\.columns\)\?\s*"
                r"\.checked_add\(slot_index\)\?;",
                re.MULTILINE,
            ),
        )
        self.assertIn(
            "one_row_scroll_rebinds_only_the_entering_physical_row",
            source,
        )
        self.assertIn("bottom_window_backfills_the_materialized_rows", source)


if __name__ == "__main__":
    unittest.main()
