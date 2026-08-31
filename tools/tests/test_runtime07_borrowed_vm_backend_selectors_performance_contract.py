from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
TRAIT_SOURCE = ROOT / "zircon_runtime/src/script/vm/backend/vm_backend_family.rs"
REGISTRY_SOURCE = ROOT / "zircon_runtime/src/script/vm/backend/backend_registry.rs"
BUILTIN_SOURCE = (
    ROOT / "zircon_runtime/src/script/vm/backend/builtin_vm_backend_family.rs"
)
ZR_VM_SOURCE = ROOT / "zircon_plugins/zr_vm_language/runtime/src/backend.rs"
IMPLEMENTATION_SOURCES = (
    REGISTRY_SOURCE,
    BUILTIN_SOURCE,
    ROOT
    / "zircon_runtime/src/script/vm/backend/backend_registry/qualified_lookup_tests.rs",
    ROOT / "zircon_runtime/src/script/vm/tests/lifecycle_failures.rs",
    ROOT / "zircon_runtime/src/script/vm/tests/support.rs",
    ZR_VM_SOURCE,
    ROOT / "zircon_plugins/zr_vm_language/runtime/src/tests/registration.rs",
)


def function_body(source: str, signature: str) -> str:
    start = source.index(signature)
    opening = source.index("{", start)
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index]
    raise AssertionError(f"unterminated function: {signature}")


class BorrowedVmBackendSelectorsPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.trait_source = TRAIT_SOURCE.read_text(encoding="utf-8")
        cls.registry_source = REGISTRY_SOURCE.read_text(encoding="utf-8")
        cls.builtin_source = BUILTIN_SOURCE.read_text(encoding="utf-8")
        cls.zr_vm_source = ZR_VM_SOURCE.read_text(encoding="utf-8")

    def test_family_contract_visits_borrowed_selectors(self) -> None:
        self.assertIn(
            "fn visit_selectors(&self, visitor: &mut dyn FnMut(&str));",
            self.trait_source,
        )
        self.assertNotIn("fn selectors(&self) -> Vec<String>", self.trait_source)

    def test_registry_collects_directly_into_the_result_vector(self) -> None:
        names = function_body(self.registry_source, "pub fn names(")
        self.assertIn("family.visit_selectors", names)
        self.assertIn("selectors.push(selector.to_owned())", names)
        self.assertNotIn(".flat_map", names)
        self.assertNotIn("family.selectors()", names)

    def test_builtin_and_zrvm_families_visit_static_selectors(self) -> None:
        for source, selectors in (
            (
                self.builtin_source,
                (
                    "builtin:mock",
                    "mock",
                    "builtin:unavailable",
                    "unavailable",
                ),
            ),
            (self.zr_vm_source, ("zr_vm:project", "project")),
        ):
            visit = function_body(source, "fn visit_selectors(")
            for selector in selectors:
                self.assertIn(f'visitor("{selector}")', visit)
            self.assertNotIn("vec![", visit)
            self.assertNotIn("to_string()", visit)

    def test_every_family_implementation_uses_the_borrowed_contract(self) -> None:
        for path in IMPLEMENTATION_SOURCES:
            source = path.read_text(encoding="utf-8")
            self.assertNotIn(
                "fn selectors(&self) -> Vec<String>",
                source,
                msg=str(path.relative_to(ROOT)),
            )


if __name__ == "__main__":
    unittest.main()
