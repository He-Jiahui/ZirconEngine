from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
PACKAGE_ASSET_REGISTRY = ROOT / (
    "zircon_runtime/src/asset/project/package_asset_registry.rs"
)
PACKAGE_ASSET_TESTS = ROOT / "zircon_runtime/src/asset/tests/project/package_assets.rs"


def function_region(source: str, start: str, end: str) -> str:
    offset = source.index(start)
    return source[offset : source.index(end, offset)]


class Runtime85ProjectRootDedupPerformanceContractTests(unittest.TestCase):
    def test_project_root_registration_uses_capacity_sized_fingerprint_index(self) -> None:
        source = PACKAGE_ASSET_REGISTRY.read_text(encoding="utf-8")
        registration = function_region(
            source,
            "    pub fn register_project_roots(",
            "    pub fn project_roots(",
        )

        self.assertIn(
            "use std::collections::{BTreeMap, HashSet, hash_map::RandomState};",
            source,
        )
        self.assertIn("use std::hash::BuildHasher;", source)
        self.assertIn("let resolved_path_hasher = RandomState::new();", registration)
        self.assertIn(
            "let mut resolved_path_hashes = HashSet::with_capacity(asset_roots.len());",
            registration,
        )
        self.assertIn(
            "let path_hash = resolved_path_hasher.hash_one(&canonical_asset_root);",
            registration,
        )
        self.assertIn(
            "if !resolved_path_hashes.insert(path_hash)"
            " && resolved.contains(&canonical_asset_root)",
            registration,
        )
        self.assertNotIn("insert(canonical_asset_root.clone())", registration)

    def test_indexed_registration_preserves_authored_root_order(self) -> None:
        source = PACKAGE_ASSET_REGISTRY.read_text(encoding="utf-8")
        registration = function_region(
            source,
            "    pub fn register_project_roots(",
            "    pub fn project_roots(",
        )

        dedup = registration.index("resolved_path_hashes.insert(path_hash)")
        publish = registration.index("resolved.push(canonical_asset_root);")
        self.assertLess(dedup, publish)
        self.assertIn("self.project_roots = resolved;", registration)

    def test_canonical_alias_duplicate_semantics_remain_covered_by_rust(self) -> None:
        tests = PACKAGE_ASSET_TESTS.read_text(encoding="utf-8")

        self.assertIn(
            "fn runtime85_project_dedup_recovery_batch_root_alias_semantics()",
            tests,
        )
        self.assertIn("AssetImportError::DuplicateProjectAssetRoot { root }", tests)


if __name__ == "__main__":
    unittest.main()
