from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
WINDOWING = ROOT / "zircon_runtime/src/ui/component/state_reducer/windowing.rs"
METADATA_BATCH = (
    ROOT / "zircon_runtime/src/ui/surface/property_mutation/metadata_batch.rs"
)


class RuntimeWindowingStateUpdatePerformanceContractTests(unittest.TestCase):
    def test_repeated_window_updates_reuse_static_property_keys(self) -> None:
        source = WINDOWING.read_text(encoding="utf-8")

        self.assertIn("fn set_static_value(", source)
        helper_start = source.index("fn set_static_value(")
        helper = source[helper_start:]
        self.assertIn("state.values.get_mut(property)", helper)
        self.assertIn("state.values.insert(property.to_string(), value)", helper)
        self.assertIn("state.reference_sources.remove(property)", helper)

    def test_visible_range_and_page_window_do_not_allocate_keys_unconditionally(self) -> None:
        source = WINDOWING.read_text(encoding="utf-8")
        helper_start = source.index("fn set_static_value(")
        reducers = source[:helper_start]

        self.assertNotIn("super::set_value", reducers)
        self.assertGreaterEqual(reducers.count("set_static_value("), 19)

    def test_metadata_batch_reuses_existing_static_property_keys(self) -> None:
        source = METADATA_BATCH.read_text(encoding="utf-8")

        self.assertIn("fn set_metadata_value(", source)
        helper_start = source.index("fn set_metadata_value(")
        mutation = source[:helper_start]
        helper = source[helper_start:]
        self.assertIn(
            "set_metadata_value(&mut metadata.attributes, property_name, next)",
            mutation,
        )
        self.assertNotIn("metadata.attributes.insert(property.to_string(), next)", mutation)
        self.assertIn("attributes.get_mut(property)", helper)
        self.assertIn("attributes.insert(property.to_string(), value)", helper)


if __name__ == "__main__":
    unittest.main()
