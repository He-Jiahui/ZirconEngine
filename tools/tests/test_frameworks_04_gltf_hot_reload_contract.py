from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
GLTF_DIST_SOURCE = REPO_ROOT / "zircon_plugins/gltf_importer/dist/src/lib.rs"


class Frameworks04GltfHotReloadContractTests(unittest.TestCase):
    def test_state_callbacks_are_abi_owned_and_panic_free(self) -> None:
        source = GLTF_DIST_SOURCE.read_text(encoding="utf-8")
        production = source.split("#[cfg(test)]", maxsplit=1)[0]

        self.assertIn("save_state: Some(gltf_importer_save_state)", production)
        self.assertIn("restore_state: Some(gltf_importer_restore_state)", production)
        self.assertIn("unload: Some(gltf_importer_unload)", production)
        self.assertIn("is_stateless: false", production)
        self.assertIn("state_schema_version: STATE_SCHEMA_VERSION", production)
        self.assertEqual(
            production.count("native::catch_native_callback_panic("),
            3,
        )
        self.assertIn(
            ".filter(|bytes| bytes.len() == std::mem::size_of::<u64>())",
            production,
        )
        self.assertIn("epoch_bytes.copy_from_slice", production)
        for panic_path in (".expect(", ".unwrap(", "panic!("):
            self.assertNotIn(panic_path, production)


if __name__ == "__main__":
    unittest.main()
