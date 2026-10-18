from __future__ import annotations

import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
TRUST_SOURCE = (
    ROOT
    / "zircon_runtime"
    / "src"
    / "plugin"
    / "native_plugin_loader"
    / "native_artifact_trust.rs"
)


class NativeArtifactAuthorityContractTests(unittest.TestCase):
    def test_local_authority_only_captures_import_reachable_sibling_dlls(self) -> None:
        source = TRUST_SOURCE.read_text(encoding="utf-8")

        self.assertIn("fn capture_native_dependency_closure", source)
        self.assertIn("goblin::pe::PE::parse", source)
        self.assertIn("native dependency source has ambiguous DLL filename", source)
        self.assertNotIn(
            "for entry in std::fs::read_dir(path.parent().expect(\"native source has parent\"))",
            source,
        )


if __name__ == "__main__":
    unittest.main()
