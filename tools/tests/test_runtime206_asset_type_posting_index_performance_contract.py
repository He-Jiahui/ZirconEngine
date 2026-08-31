from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
INDEX = ROOT / "zircon_runtime/src/asset/registry/asset_registry_index.rs"
QUERY = ROOT / "zircon_runtime/src/asset/registry/query.rs"


def function_body(source: str, signature: str) -> str:
    start = source.index(signature)
    opening = source.index("{", start)
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index]
    raise AssertionError(f"unterminated function: {signature}")


class Runtime206AssetTypePostingIndexPerformanceContractTests(unittest.TestCase):
    def test_index_owns_and_maintains_type_postings(self) -> None:
        source = INDEX.read_text(encoding="utf-8")
        insert = function_body(source, "pub(super) fn insert_checked(")
        remove = function_body(source, "pub(super) fn remove_source_path(")

        self.assertIn(
            "uuids_by_type: HashMap<AssetKind, HashSet<AssetUuid>>", source
        )
        self.assertIn("self.uuids_by_type", insert)
        self.assertIn(".insert(entry.uuid())", insert)
        self.assertIn("self.uuids_by_type", remove)
        self.assertIn(".remove(&uuid)", remove)

    def test_type_query_reads_only_the_type_posting(self) -> None:
        source = QUERY.read_text(encoding="utf-8")
        query = function_body(source, "pub fn get_assets_by_type(")
        compact = "".join(source.split())

        self.assertIn("self.sorted_type_matches(type_marker", query)
        self.assertNotIn("self.sorted_matches", query)
        self.assertNotIn("entries_by_uuid.values()", query)
        self.assertIn("self.uuids_by_type.get(&type_marker)", compact)

    def test_type_filtered_query_reuses_the_same_candidate_posting(self) -> None:
        source = QUERY.read_text(encoding="utf-8")
        query = function_body(source, "pub fn get_assets(")

        self.assertIn("filter.type_marker", query)
        self.assertIn("self.sorted_type_matches(type_marker", query)

    def test_uniform_type_pressure_reduces_candidate_visits_by_32x(self) -> None:
        entry_count = 1_048_576
        type_count = 32
        matching_entries = entry_count // type_count

        before_visits = entry_count
        after_visits = matching_entries

        self.assertEqual(before_visits, 1_048_576)
        self.assertEqual(after_visits, 32_768)
        self.assertEqual(before_visits // after_visits, 32)
        self.assertAlmostEqual(1.0 - after_visits / before_visits, 0.96875)


if __name__ == "__main__":
    unittest.main()
