from pathlib import Path
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
LIB = REPO_ROOT / "examples/woc/native/crates/woc_contract_codegen/src/lib.rs"
PROJECTION = REPO_ROOT / "examples/woc/native/crates/woc_contract_codegen/src/projection.rs"
TESTS = REPO_ROOT / "examples/woc/native/crates/woc_contract_codegen/tests/contract_generation.rs"


class Tooling05WocCodegenIncrementalIoPerformanceContract(unittest.TestCase):
    def test_reference_catalog_bytes_are_read_once_for_hash_and_parse(self) -> None:
        source = LIB.read_text(encoding="utf-8")

        self.assertIn("&source.catalog_sha256", source)
        self.assertIn("Sha256::digest(source.as_bytes())", source)
        self.assertIn("serde_json::from_str(&source)", source)
        self.assertNotIn("validate_catalog_identities(root", source)
        self.assertNotIn("let catalog: Catalog<T> = read_json(root, name)?", source)

    def test_unchanged_projection_preserves_the_incremental_build_chain(self) -> None:
        source = PROJECTION.read_text(encoding="utf-8")
        tests = TESTS.read_text(encoding="utf-8")

        self.assertIn("ProjectionWriteOutcome", source)
        self.assertIn("metadata.len() == contents.len() as u64", source)
        self.assertIn("actual == contents.as_bytes()", source)
        self.assertIn("ProjectionWriteOutcome::Unchanged", source)
        self.assertIn("unchanged_projection_is_not_rewritten", tests)


if __name__ == "__main__":
    unittest.main()
