from __future__ import annotations

import hashlib
import json
import unittest
from pathlib import Path

from tools.session_coordinator.artifact_receipts import ManagedArtifactReceiptService


SCRIPT = (
    Path(__file__).resolve().parents[1]
    / "session_coordinator"
    / "artifact_receipts.py"
)


class ArtifactReceiptManifestHashProjectionPerformanceContractTests(unittest.TestCase):
    def test_source_manifest_folds_each_hash_once(self) -> None:
        source = SCRIPT.read_text(encoding="utf-8")
        function = source[source.index("    def _source_manifest(") :]

        self.assertIn("folded_source_hash = source_hash.casefold()", function)
        self.assertEqual(1, function.count("source_hash.casefold()"))
        self.assertIn("_SHA256.fullmatch(folded_source_hash)", function)
        self.assertIn("normalized[relative_path] = folded_source_hash", function)

    def test_uppercase_hash_is_still_canonicalized(self) -> None:
        source_hash = "A" * 64
        canonical = json.dumps(
            {"src/lib.rs": source_hash.casefold()},
            sort_keys=True,
            separators=(",", ":"),
            ensure_ascii=True,
        )
        expected_hash = hashlib.sha256(canonical.encode("utf-8")).hexdigest()

        normalized = ManagedArtifactReceiptService._source_manifest(
            json.dumps({"src/lib.rs": source_hash}),
            expected_hash,
        )

        self.assertEqual({"src/lib.rs": "a" * 64}, normalized)


if __name__ == "__main__":
    unittest.main()
