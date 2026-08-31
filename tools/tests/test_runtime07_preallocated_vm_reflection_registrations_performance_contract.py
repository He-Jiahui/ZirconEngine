from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/script/vm/reflection/schema.rs"


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


class PreallocatedVmReflectionRegistrationsPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.projection = function_body(
            cls.source,
            "pub fn from_state_schema(schema: &VmStateSchema) -> Result<Self, VmReflectionError>",
        )

    def test_projection_preallocates_from_the_schema_row_upper_bound(self) -> None:
        self.assertIn(
            "Vec::with_capacity(schema.types.len())",
            self.projection,
        )
        self.assertNotIn("let mut registrations = Vec::new()", self.projection)

    def test_projection_does_not_add_a_counting_scan(self) -> None:
        self.assertNotIn(".filter(|type_schema|", self.projection)
        self.assertNotIn(".count();", self.projection)

    def test_projection_reuses_one_visibility_and_role_predicate(self) -> None:
        helper = function_body(
            self.source,
            "fn is_public_component_registration(registration: &ReflectTypeRegistration) -> bool",
        )
        self.assertIn("ReflectScriptVisibility::Public", helper)
        self.assertIn("registration.is_component()", helper)
        self.assertNotIn("registration.is_resource()", helper)
        self.assertIn("if !is_public_component_registration(registration)", self.projection)


if __name__ == "__main__":
    unittest.main()
