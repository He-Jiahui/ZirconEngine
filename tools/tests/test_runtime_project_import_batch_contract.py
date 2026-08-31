from __future__ import annotations

import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
TARGETED = ROOT / "zircon_runtime/src/asset/project/manager/scan_and_import/targeted.rs"
SCAN = ROOT / "zircon_runtime/src/asset/project/manager/scan_and_import.rs"
ERROR = ROOT / "zircon_runtime/src/asset/importer/error.rs"


class RuntimeProjectImportBatchContractTests(unittest.TestCase):
    def test_batch_retains_only_one_final_registry_write(self) -> None:
        source = TARGETED.read_text(encoding="utf-8")

        self.assertIn("struct PreparedProjectImportBatch", source)
        self.assertIn("registry_write: PreparedFileWrite", source)
        self.assertIn("self.registry_write = registry_write", source)
        self.assertIn("writes.push(self.registry_write)", source)
        self.assertIn(".sort_by_key", source)
        self.assertIn("lock_meta_document_paths(&self.meta_paths)", source)
        self.assertIn("append_prepared_file_writes", source)

    def test_batch_preparation_is_candidate_scoped_and_rejects_duplicates(self) -> None:
        source = TARGETED.read_text(encoding="utf-8")

        self.assertIn("fn prepare_targeted_import_batch", source)
        self.assertIn("AssetImportError::EmptyProjectImportBatch", source)
        self.assertIn("AssetImportError::DuplicateProjectAssetUri", source)
        self.assertIn("self.prepare_targeted_generation(&source_uri, path)?", source)
        self.assertIn("PreparedProjectImportBatch::from_targeted_generations", source)
        self.assertNotIn("fn import_targeted_batch", source)
        self.assertIn("PreparedProjectImportBatch", SCAN.read_text(encoding="utf-8"))
        self.assertIn("EmptyProjectImportBatch", ERROR.read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
