from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT
    / "zircon_runtime/src/scene/dynamic_scene/session/selected_retention/tag/preview.rs"
)


def function_body(source: str, name: str) -> str:
    match = re.search(rf"\bfn\s+{re.escape(name)}\b", source)
    if match is None:
        raise AssertionError(f"missing function {name}")
    start = source.find("{", match.end())
    depth = 0
    for index in range(start, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[start : index + 1]
    raise AssertionError(f"unterminated body for {name}")


class IncrementalSelectedRetentionPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.owner = function_body(cls.source, "report_with_selected_tag_slot")
        cls.incremental = (
            function_body(cls.source, "protect_selected_slot")
            if re.search(r"\bfn\s+protect_selected_slot\b", cls.source)
            else ""
        )

    def test_selected_protection_updates_both_sorted_partitions_incrementally(self) -> None:
        self.assertGreaterEqual(self.incremental.count("binary_search_by"), 2)
        self.assertIn("report.removed_slot_ids.remove(removed_index);", self.incremental)
        self.assertRegex(
            self.incremental,
            r"report\s*\.retained_slot_ids\s*\.insert\(retained_index,\s*selected_slot_id\.to_owned\(\)\)",
        )

    def test_owner_no_longer_rebuilds_report_from_the_archive(self) -> None:
        self.assertNotIn("archive.slot_ids()", self.owner)
        self.assertNotIn("removed_slot_ids.iter().any", self.owner.replace("\n", ""))
        self.assertNotIn("retained_slot_ids.sort", self.owner)
        self.assertIn("protect_selected_slot(report, selected_slot_id)", self.owner)

    def test_rust_regressions_cover_order_noop_and_partition_uniqueness(self) -> None:
        self.assertIn(
            "fn runtime52_batch_incremental_selected_protection_preserves_canonical_order()",
            self.source,
        )
        self.assertIn(
            "fn runtime52_batch_incremental_selected_protection_is_noop_when_not_removed()",
            self.source,
        )
        self.assertIn(
            "fn runtime52_batch_incremental_selected_protection_keeps_partition_unique()",
            self.source,
        )


if __name__ == "__main__":
    unittest.main()
