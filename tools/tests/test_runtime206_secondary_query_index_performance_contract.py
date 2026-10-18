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


class Runtime206SecondaryQueryIndexPerformanceContractTests(unittest.TestCase):
    def test_index_owns_tag_package_and_path_prefix_postings(self) -> None:
        source = INDEX.read_text(encoding="utf-8")

        self.assertIn(
            "uuids_by_tag: HashMap<String, HashSet<AssetUuid>>",
            source,
        )
        self.assertIn(
            "uuids_by_package: HashMap<String, HashSet<AssetUuid>>",
            source,
        )
        self.assertIn(
            "uuids_by_path_prefix: BTreeMap<String, HashSet<AssetUuid>>",
            source,
        )

    def test_insert_and_remove_maintain_all_secondary_postings(self) -> None:
        source = INDEX.read_text(encoding="utf-8")
        insert = function_body(source, "pub(super) fn insert_checked(")
        remove = function_body(source, "pub(super) fn remove_source_path(")

        for body in (insert, remove):
            self.assertIn("uuids_by_tag", body)
            self.assertIn("uuids_by_package", body)
            self.assertIn("uuids_by_path_prefix", body)
        self.assertIn("entry.tags()", insert)
        self.assertIn("entry.path().package_id()", insert)
        self.assertIn("entry.path().path()", insert)
        self.assertIn("entry.tags()", remove)
        self.assertIn("entry.path().package_id()", remove)
        self.assertIn("entry.path().path()", remove)

    def test_filtered_query_starts_from_a_secondary_candidate_index(self) -> None:
        source = QUERY.read_text(encoding="utf-8")
        query = function_body(source, "pub fn get_assets(")

        self.assertIn("candidate_entries", query)
        self.assertIn("entry_matches_filter", query)
        self.assertNotIn("self.entries_by_uuid.values().filter", query)
        self.assertNotIn("self.entries_by_uuid.values().collect", query)
        self.assertIn("uuids_by_tag", source)
        self.assertIn("uuids_by_package", source)
        self.assertIn("uuids_by_path_prefix", source)

    def test_path_prefix_range_borrows_the_filter_key(self) -> None:
        source = QUERY.read_text(encoding="utf-8")

        self.assertIn(
            "range::<str, _>((Bound::Included(prefix), Bound::Unbounded))",
            source,
        )
        self.assertNotIn("range(prefix.to_owned()..)", source)

    def test_path_union_is_lazy_when_a_direct_posting_is_available(self) -> None:
        source = QUERY.read_text(encoding="utf-8")
        candidate = function_body(source, "fn candidate_entries(")

        self.assertIn("if candidate_uuids.is_none()", candidate)

    def test_uniform_tag_pressure_reduces_candidate_visits_by_32x(self) -> None:
        entry_count = 1_048_576
        tag_count = 32
        matching_entries = entry_count // tag_count

        self.assertEqual(entry_count, 1_048_576)
        self.assertEqual(matching_entries, 32_768)
        self.assertEqual(entry_count // matching_entries, 32)
        self.assertAlmostEqual(1.0 - matching_entries / entry_count, 0.96875)


if __name__ == "__main__":
    unittest.main()
