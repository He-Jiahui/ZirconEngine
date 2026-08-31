from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


class RuntimeAllocationRegistryStorageContract(unittest.TestCase):
    def test_registration_preserves_the_producer_vec_without_capacity_shrink(self) -> None:
        registry = (
            ROOT
            / "zircon_runtime/src/dynamic_api/session/registry/allocation_registry.rs"
        ).read_text(encoding="utf-8")
        registration = registry[registry.index("fn register_runtime_allocation_in_action") :]

        self.assertIn("bytes: Vec<u8>", registry)
        self.assertNotIn("into_boxed_slice", registry)
        self.assertLess(
            registration.index("let data = bytes.as_ptr();"),
            registration.index("lock_registry()"),
        )


if __name__ == "__main__":
    unittest.main()
