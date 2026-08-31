"""Static contract for the immutable built-in asset-type registry base."""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
BUILTIN = ROOT / "zircon_editor/src/core/asset/type_registry/builtin.rs"


class EditorAssetBuiltinRegistryBaseCacheContractTests(unittest.TestCase):
    def test_builtin_registry_validates_and_compiles_the_static_base_once(self):
        source = BUILTIN.read_text(encoding="utf-8")

        self.assertIn(
            "fn builtin_registry_base() "
            "-> &'static Result<AssetTypeRegistry, AssetTypeRegistryError>",
            source,
        )
        self.assertIn(
            "static REGISTRY: OnceLock<Result<AssetTypeRegistry, AssetTypeRegistryError>>",
            source,
        )
        self.assertIn("REGISTRY.get_or_init(build_builtin_registry)", source)
        self.assertIn("fn build_builtin_registry()", source)
        self.assertIn("builtin_registry_base().clone()", source)
        self.assertNotIn("OnceLock<Option<AssetTypeRegistry>>", source)


if __name__ == "__main__":
    unittest.main()
