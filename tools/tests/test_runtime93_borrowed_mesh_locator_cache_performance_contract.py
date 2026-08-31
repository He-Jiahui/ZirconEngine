from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
MODEL_GEOMETRY = ROOT / (
    "zircon_runtime/src/graphics/scene/resources/resource_streamer/"
    "model_geometry_resolution.rs"
)


def function_region(source: str, start: str, end: str) -> str:
    offset = source.index(start)
    return source[offset : source.index(end, offset)]


class Runtime93BorrowedMeshLocatorCachePerformanceContractTests(unittest.TestCase):
    def test_model_geometry_cache_borrows_asset_uri_keys(self) -> None:
        source = MODEL_GEOMETRY.read_text(encoding="utf-8")
        resolve = function_region(
            source,
            "pub(super) fn resolve_model_geometry(",
            "pub(super) fn model_dependencies_are_current(",
        )

        self.assertIn(
            "HashMap::<&crate::asset::AssetUri, Option<MeshAsset>>::new()",
            resolve,
        )
        self.assertIn("mesh_assets.entry(&reference.locator)", resolve)
        self.assertNotIn("reference.locator.to_string()", resolve)
        self.assertNotIn("HashMap::<String, Option<MeshAsset>>", resolve)

    def test_cache_miss_still_records_dependency_before_loading(self) -> None:
        source = MODEL_GEOMETRY.read_text(encoding="utf-8")
        resolve = function_region(
            source,
            "pub(super) fn resolve_model_geometry(",
            "pub(super) fn model_dependencies_are_current(",
        )
        dependency = resolve.index("dependency_states.push(dependency_state(asset_manager, reference));")
        load = resolve.index("load_referenced_mesh_asset(asset_manager, reference)")

        self.assertLess(dependency, load)
        self.assertEqual(resolve.count("or_insert_with(||"), 1)

    def test_equal_borrowed_locator_semantics_are_covered_by_rust(self) -> None:
        source = MODEL_GEOMETRY.read_text(encoding="utf-8")

        self.assertIn(
            "fn runtime93_borrowed_mesh_locator_cache_deduplicates_equal_locators()",
            source,
        )
        self.assertIn("assert_eq!(mesh_assets.len(), 1);", source)
        self.assertIn("assert_eq!(mesh_assets[&first], 11);", source)


if __name__ == "__main__":
    unittest.main()
