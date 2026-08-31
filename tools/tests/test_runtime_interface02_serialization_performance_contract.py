import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
CANONICAL = (
    REPO_ROOT
    / "zircon_runtime_interface"
    / "src"
    / "serialization"
    / "text"
    / "canonical.rs"
)
INTO_JSON = (
    REPO_ROOT
    / "zircon_runtime_interface"
    / "src"
    / "serialization"
    / "binary"
    / "value"
    / "into_json.rs"
)


def function_body(source: str, function_name: str) -> str:
    match = re.search(rf"\bfn\s+{re.escape(function_name)}\s*\(", source)
    if match is None:
        raise AssertionError(f"missing function {function_name}")
    opening = source.find("{", match.end())
    if opening < 0:
        raise AssertionError(f"missing body for {function_name}")
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index]
    raise AssertionError(f"unterminated body for {function_name}")


class SerializationPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.canonical = CANONICAL.read_text(encoding="utf-8")
        cls.into_json = INTO_JSON.read_text(encoding="utf-8")

    def test_canonical_objects_reuse_the_input_map(self) -> None:
        body = function_body(self.canonical, "canonicalize_value")
        self.assertIn("canonicalize_value_in_place(&mut value)", body)
        in_place = function_body(self.canonical, "canonicalize_value_in_place")
        self.assertIn("Value::Object(values)", in_place)
        self.assertIn("values.values_mut()", in_place)
        self.assertIn("values.sort_keys()", in_place)
        nested = function_body(self.canonical, "canonicalize_nested_value")
        self.assertIn(
            "matches!(value, Value::Array(_) | Value::Object(_))", nested
        )
        self.assertIn("canonicalize_value_in_place(value)", nested)
        self.assertNotIn("std::mem::take", in_place)
        self.assertNotIn("collect::<Vec<_>>()", in_place)
        self.assertNotIn("Map::from_iter", in_place)

    def test_binary_object_decode_uses_one_entry_lookup(self) -> None:
        attach = function_body(self.into_json, "attach_value")
        self.assertNotIn("contains_key", attach)
        self.assertIn("insert_unique_object_value(values, key, value)?", attach)
        helper = function_body(self.into_json, "insert_unique_object_value")
        self.assertIn("values.entry(key)", helper)
        self.assertIn("Entry::Vacant", helper)
        self.assertIn("Entry::Occupied", helper)

    def test_release_evidence_tracks_both_serialization_tasks(self) -> None:
        self.assertIn(
            "PERF_RESULT runtime_interface02_canonical_object_reuse",
            self.canonical,
        )
        self.assertIn("legacy_transient_collections_per_object=2", self.canonical)
        self.assertIn("optimized_transient_collections_per_object=0", self.canonical)
        self.assertIn("legacy_value_slot_rewrites_per_child=2", self.canonical)
        self.assertIn("optimized_value_slot_rewrites_per_child=0", self.canonical)
        self.assertIn(
            "PERF_RESULT runtime_interface02_binary_object_entry",
            self.into_json,
        )
        self.assertIn("legacy_index_lookups_per_entry=2", self.into_json)
        self.assertIn("optimized_index_lookups_per_entry=1", self.into_json)


if __name__ == "__main__":
    unittest.main()
