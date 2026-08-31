from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
CATALOG_ROOT = ROOT / "zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs"
CATALOG_DIR = ROOT / "zircon_runtime/src/plugin/runtime_plugin/builtin_catalog"
PLUGIN_TESTS = ROOT / "zircon_runtime/src/tests/plugin_extensions"


class Runtime42BorrowedBuiltinCatalogPackageIdPerformanceContractTests(unittest.TestCase):
    def test_catalog_pipeline_carries_the_static_row_package_id(self) -> None:
        source = CATALOG_ROOT.read_text(encoding="utf-8")

        self.assertIn(
            "type IdentifiedBuiltinCatalogDescriptorBuilder = "
            "(&'static str, BuiltinCatalogDescriptorBuilder);",
            source,
        )
        row_id = source.index("row.package_id,")
        builder = source.index("Self::builder(", row_id)
        augment = source.index(".map(augment_descriptor)", builder)
        optional = source.index(".map(attach_optional_features)", augment)
        classify = source.index(".map(classify_descriptor)", optional)
        unwrap = source.index(".map(|(_, descriptor)| descriptor)", classify)
        build = source.index(".map(RuntimePluginDescriptorBuilder::build)", unwrap)

        self.assertLess(row_id, builder)
        self.assertLess(augment, optional)
        self.assertLess(optional, classify)
        self.assertLess(classify, unwrap)
        self.assertLess(unwrap, build)

    def test_dispatch_stages_borrow_package_id_without_string_clones(self) -> None:
        stages = [
            CATALOG_DIR / "classification.rs",
            CATALOG_DIR / "optional_features.rs",
            CATALOG_DIR / "augmentation.rs",
            CATALOG_DIR / "augmentation/capabilities.rs",
            CATALOG_DIR / "augmentation/categories.rs",
        ]
        combined = "\n".join(path.read_text(encoding="utf-8") for path in stages)

        self.assertGreaterEqual(
            combined.count("package_id: &str"),
            2,
            "leaf augmentation stages should receive the borrowed row id",
        )
        self.assertGreaterEqual(
            combined.count("IdentifiedBuiltinCatalogDescriptorBuilder"),
            6,
            "the three pipeline stages should consume and return identified builders",
        )
        self.assertNotIn("descriptor.package_id().to_string()", combined)
        self.assertNotIn("package_id.as_str()", combined)

    def test_existing_rust_catalog_oracles_cover_all_dispatch_outputs(self) -> None:
        sources = "\n".join(
            path.read_text(encoding="utf-8")
            for path in PLUGIN_TESTS.rglob("*.rs")
        )

        for test_name in [
            "fn builtin_catalog_classifies_bevy_parity_runtime_plugins()",
            "fn builtin_catalog_statuses_match_importer_and_physics_capability_metadata()",
            "fn builtin_sound_optional_features_declare_editor_capabilities()",
            "fn builtin_rendering_catalog_declares_owner_features_and_defaults()",
        ]:
            self.assertIn(test_name, sources)


if __name__ == "__main__":
    unittest.main()
