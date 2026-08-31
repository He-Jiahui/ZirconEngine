from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/dynamic_api/session/construction.rs"


class RuntimeSessionLinkedModuleRegistrationM0PerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_linked_modules_are_not_deep_cloned_into_a_temporary_vector(self) -> None:
        self.assertNotIn(
            "linked_extensions.registry.modules().to_vec()", self.source
        )

    def test_registration_borrows_the_composition_owned_descriptor_slice(self) -> None:
        compact = "".join(self.source.split())
        self.assertIn(
            "fordescriptorinmodules.module_descriptors()", compact
        )
        self.assertIn("runtime.register_module(descriptor.clone())", compact)


if __name__ == "__main__":
    unittest.main()
