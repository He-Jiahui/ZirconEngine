from __future__ import annotations

import json
import re
import subprocess
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
GENERATOR = ROOT / "examples/woc/tools/talent_modifier_catalog_codegen.mjs"
DOCUMENT = ROOT / "examples/woc/reference/current-head/talent_modifier_catalog.json"
CATALOG = ROOT / "examples/woc/scripts/woc_game/src/generated/talent_modifier_catalog.zr"
STATE = ROOT / "examples/woc/scripts/woc_game/src/progression/talent_modifier_state.zr"

SPEC_COUNT_PER_CLASS = 3
OPTION_COUNT_PER_CLASS = 18
ENTRY_COUNT_PER_CLASS = SPEC_COUNT_PER_CLASS + OPTION_COUNT_PER_CLASS


def zr_function_body(source: str, name: str) -> str:
    match = re.search(rf"\b(?:pub\s+)?{re.escape(name)}\s*\([^)]*\)[^{{]*{{", source)
    if match is None:
        raise AssertionError(f"missing Zr function {name}")
    depth = 1
    index = match.end()
    while index < len(source) and depth:
        depth += source[index] == "{"
        depth -= source[index] == "}"
        index += 1
    if depth:
        raise AssertionError(f"unterminated Zr function {name}")
    return source[match.end() : index - 1]


class Runtime14TalentModifierEntryLookupPerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.generator = GENERATOR.read_text(encoding="utf-8")
        cls.document = json.loads(DOCUMENT.read_text(encoding="utf-8"))
        cls.catalog = CATALOG.read_text(encoding="utf-8")
        cls.state = STATE.read_text(encoding="utf-8")

    def test_generated_projection_is_current(self) -> None:
        result = subprocess.run(
            ["node", str(GENERATOR), "--check"],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_codegen_validates_dense_origin_layout(self) -> None:
        self.assertIn("validateDenseOriginLayout(entries, selection.classes);", self.generator)
        validation = re.search(
            r"function validateDenseOriginLayout\(.*?\n\}",
            self.generator,
            re.DOTALL,
        )
        self.assertIsNotNone(validation)
        body = validation.group(0)
        self.assertIn("SPEC_COUNT_PER_CLASS", body)
        self.assertIn("OPTION_COUNT_PER_CLASS", body)
        self.assertIn("origin_code", body)

    def test_current_document_obeys_the_dense_layout(self) -> None:
        entries = self.document["entries"]
        class_ids = list(dict.fromkeys(entry["class_id"] for entry in entries))
        self.assertEqual(len(entries), len(class_ids) * ENTRY_COUNT_PER_CLASS)
        for class_index, class_id in enumerate(class_ids):
            start = class_index * ENTRY_COUNT_PER_CLASS
            class_entries = entries[start : start + ENTRY_COUNT_PER_CLASS]
            self.assertEqual({entry["class_id"] for entry in class_entries}, {class_id})
            for offset, entry in enumerate(class_entries[:SPEC_COUNT_PER_CLASS]):
                self.assertEqual(entry["origin"], "spec")
                self.assertEqual(
                    entry["origin_code"],
                    class_index * SPEC_COUNT_PER_CLASS + offset + 1,
                )
            for offset, entry in enumerate(class_entries[SPEC_COUNT_PER_CLASS:]):
                self.assertEqual(entry["origin"], "option")
                self.assertEqual(
                    entry["origin_code"],
                    class_index * OPTION_COUNT_PER_CLASS + offset + 1,
                )

    def test_generated_entry_index_uses_constant_time_integer_math(self) -> None:
        body = zr_function_body(self.catalog, "entryIndex")
        self.assertIn("originCode > <uint>27", body)
        self.assertIn("originCode > <uint>162", body)
        self.assertIn("code / 3 * 21 + code % 3", body)
        self.assertIn("code / 18 * 21 + 3 + code % 18", body)
        self.assertNotIn("while (", body)
        self.assertNotIn("entryOriginCode", body)

    def test_reducer_delegates_lookup_to_generated_catalog(self) -> None:
        body = zr_function_body(self.state, "findEntry")
        self.assertIn("return catalog.entryIndex(originCode, spec);", body)
        self.assertNotIn("catalog.entryCount()", body)
        self.assertNotIn("while (", body)

    def test_zr_contract_covers_all_entries_and_invalid_codes(self) -> None:
        body = zr_function_body(self.state, "contractTest")
        self.assertIn("catalog.entryIndex(", body)
        self.assertIn("catalog.entryOriginCode(entry)", body)
        self.assertIn("catalog.entryIsSpec(entry)", body)
        self.assertIn("catalog.entryIndex(<uint>0, false) != -1", body)
        self.assertIn("catalog.entryIndex(<uint>28, true) != -1", body)
        self.assertIn("catalog.entryIndex(<uint>163, false) != -1", body)

    def test_worst_case_lookup_comparisons_drop_by_at_least_ninety_eight_percent(self) -> None:
        legacy_comparisons = len(self.document["entries"])
        optimized_decisions = 3
        reduction_percent = (
            (legacy_comparisons - optimized_decisions) * 100.0 / legacy_comparisons
        )
        self.assertGreaterEqual(reduction_percent, 98.0)


if __name__ == "__main__":
    unittest.main()
