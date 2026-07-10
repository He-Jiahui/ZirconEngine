import unittest
from pathlib import Path


class RuntimeAssetSchemaNamingTests(unittest.TestCase):
    def test_dds_cubemap_flags_use_protocol_names_not_legacy_labels(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        source = (
            repo_root
            / "zircon_runtime/src/asset/assets/texture/external_source_cubemap.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("caps2_cubemap", source)
        self.assertIn("DDS caps2", source)
        self.assertNotIn("legacy_cubemap", source)
        self.assertNotIn("legacy cubemap", source)
        self.assertNotIn("legacy caps2", source)


if __name__ == "__main__":
    unittest.main()
