from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/script/vm/reflection/catalog.rs"
RUST_TEST_SOURCE = (
    ROOT / "zircon_runtime/src/script/vm/reflection/tests/schema_invariants.rs"
)
SIGNATURE = "fn validate_package_owner("


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


class BorrowedReflectionOwnerValidationPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.body = function_body(cls.source, SIGNATURE)
        cls.rust_tests = RUST_TEST_SOURCE.read_text(encoding="utf-8")

    def test_success_path_borrows_type_path_without_preemptive_clone(self) -> None:
        loop_prefix = self.body.split("if type_path_owner != expected_owner", 1)[0]
        self.assertIn("for registration in registrations", loop_prefix)
        self.assertNotIn("type_path.clone()", loop_prefix)
        self.assertNotIn("type_path.type_path.clone()", loop_prefix)
        self.assertNotIn("type_path().to_string()", loop_prefix)
        self.assertNotIn("let type_path =", self.body)

    def test_each_error_branch_owns_type_path_only_when_constructing_error(self) -> None:
        owned_error_field = "type_path: registration.type_path.type_path().to_string(),"
        self.assertEqual(self.body.count(owned_error_field), 1)
        self.assertIn("type_path_owner != expected_owner", self.body)

    def test_existing_rust_guard_preserves_foreign_namespace_error_payload(self) -> None:
        self.assertIn(
            "trusted_package_owner_rejects_self_consistent_foreign_namespaces",
            self.rust_tests,
        )
        self.assertIn("VmReflectionError::PackageOwnerMismatch", self.rust_tests)
        self.assertIn("type_path,", self.rust_tests)


if __name__ == "__main__":
    unittest.main()
