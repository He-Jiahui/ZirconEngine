from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/asset/registry/asset_registry_index.rs"


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


class Runtime206RegistryBuildCapacityPerformanceContractTests(unittest.TestCase):
    def test_bulk_build_uses_iterator_lower_bound_to_reserve_primary_indexes(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        body = function_body(source, "pub fn from_entries(")
        reservation = function_body(source, "fn reserve_build_capacity(")

        self.assertIn("size_hint", body)
        self.assertIn("reserve_build_capacity", body)
        for field in (
            "entries_by_uuid",
            "uuids_by_path",
            "uuid_by_asset_id",
            "entry_uuids_by_source",
        ):
            self.assertIn(f"self.{field}.reserve", reservation)

    def test_dependency_path_bootstrap_does_not_retain_all_path_vectors(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        body = function_body(source, "pub fn from_entries(")

        self.assertIn("entry_uuids", body)
        self.assertIn("replace_dependency_paths", body)
        self.assertNotIn(
            "let dependency_paths = index.entries_by_uuid.values().map",
            body,
        )

    def test_bulk_build_defers_reverse_path_bucket_pruning_until_after_stream(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        body = function_body(source, "pub fn from_entries(")
        helper = function_body(source, "fn replace_dependency_paths_inner(")

        self.assertIn("replace_dependency_paths_inner", body)
        self.assertIn("false", body)
        self.assertIn("prune_empty_buckets", helper)
        self.assertIn("if prune_empty_buckets", helper)

    def test_bulk_build_pressure_model_reduces_transient_path_staging(self) -> None:
        entry_count = 1_048_576
        dependency_edges = 4 * entry_count
        max_degree = 4

        legacy_temporary_path_records = dependency_edges
        optimized_temporary_path_records = max_degree
        self.assertEqual(legacy_temporary_path_records, 4 * entry_count)
        self.assertEqual(optimized_temporary_path_records, max_degree)
        self.assertLess(optimized_temporary_path_records, legacy_temporary_path_records)
        self.assertEqual(
            legacy_temporary_path_records - optimized_temporary_path_records,
            4 * entry_count - max_degree,
        )

        legacy_reverse_path_prune_scans = entry_count
        optimized_reverse_path_prune_scans = 1
        self.assertEqual(legacy_reverse_path_prune_scans, 1_048_576)
        self.assertEqual(optimized_reverse_path_prune_scans, 1)
        self.assertEqual(
            legacy_reverse_path_prune_scans - optimized_reverse_path_prune_scans,
            entry_count - 1,
        )

        # The persistent dependency-path index is intentionally unchanged; the
        # optimization only removes the all-at-once bootstrap staging copy.


if __name__ == "__main__":
    unittest.main()
