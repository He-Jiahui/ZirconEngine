from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT
    / "zircon_runtime/src/plugin/runtime_plugin/registration_report/validation/system_anchors.rs"
)


class PreallocatedRegistrationSystemAnchorIndexPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_both_registration_iterators_contribute_capacity_bounds(self) -> None:
        self.assertIn("let plugin_system_rows = extensions.plugin_systems();", self.source)
        self.assertIn(
            "let runtime_system_rows = extensions.plugin_runtime_systems();",
            self.source,
        )
        self.assertEqual(self.source.count(".size_hint()"), 2)

    def test_anchor_index_is_preallocated_before_single_pass_insertion(self) -> None:
        self.assertIn(
            "let mut registered_systems = HashSet::with_capacity(", self.source
        )
        self.assertEqual(self.source.count("registered_systems.insert(("), 2)
        self.assertNotIn("collect::<HashSet<_>>()", self.source)

    def test_rust_guard_preserves_borrowed_anchor_index_contract(self) -> None:
        self.assertIn(
            "preallocated_system_anchor_index_preserves_borrowed_registration_contract",
            self.source,
        )
        self.assertIn("plugin_module_name(owner)", self.source)
        self.assertIn("system.id.as_str()", self.source)


if __name__ == "__main__":
    unittest.main()
