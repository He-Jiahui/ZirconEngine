import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
DESCRIPTOR_FILES = (
    ROOT / "zircon_runtime/src/core/runtime/descriptors/driver_descriptor.rs",
    ROOT / "zircon_runtime/src/core/runtime/descriptors/manager_descriptor.rs",
    ROOT / "zircon_runtime/src/core/runtime/descriptors/plugin_descriptor.rs",
)
ENGINE_SERVICE = ROOT / "zircon_runtime/src/engine_module/engine_service.rs"


class Runtime46SharedServiceDependenciesPerformanceContract(unittest.TestCase):
    def test_service_descriptors_freeze_dependencies_in_shared_slices(self) -> None:
        for path in DESCRIPTOR_FILES:
            source = path.read_text(encoding="utf-8")
            self.assertIn("Arc<[DependencySpec]>", source, path)
            self.assertIsNotNone(
                re.search(
                    r"dependencies:\s*Vec<DependencySpec>.*?dependencies:\s*dependencies\.into\(\)",
                    source,
                    flags=re.DOTALL,
                ),
                path,
            )
            self.assertNotIn("pub dependencies: Vec<DependencySpec>", source, path)

    def test_contract_construction_shares_descriptor_dependencies(self) -> None:
        source = ENGINE_SERVICE.read_text(encoding="utf-8")
        self.assertIn("dependencies: Arc<[DependencySpec]>", source)
        self.assertEqual(source.count("Arc::clone(&descriptor.dependencies)"), 3)
        self.assertNotIn("descriptor.dependencies.clone()", source)

    def test_rust_contract_checks_all_service_kinds_share_the_same_slice(self) -> None:
        source = ENGINE_SERVICE.read_text(encoding="utf-8")
        test_match = re.search(
            r"fn service_contracts_share_descriptor_dependency_slices\(\).*?\n\}",
            source,
            flags=re.DOTALL,
        )
        self.assertIsNotNone(test_match)
        body = test_match.group(0)
        self.assertEqual(body.count("Arc::ptr_eq"), 3)
        for contract in ("driver_contract", "manager_contract", "plugin_contract"):
            self.assertIn(contract, body)


if __name__ == "__main__":
    unittest.main()
