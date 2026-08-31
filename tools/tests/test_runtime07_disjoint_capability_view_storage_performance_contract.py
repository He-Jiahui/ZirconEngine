from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/plugin/runtime_plugin/capability_view.rs"


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


class DisjointCapabilityViewStoragePerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_has_queries_provided_and_status_indexes(self) -> None:
        body = function_body(self.source, "pub fn has(&self, capability: &str) -> bool")
        self.assertIn("self.provided.contains(capability)", body)
        self.assertIn("self.statuses.contains_key(capability)", body)

    def test_with_status_moves_one_owned_key_into_the_status_index(self) -> None:
        body = function_body(self.source, "pub fn with_status(")
        self.assertIn("self.provided.remove(&capability)", body)
        self.assertIn("self.statuses.insert(capability, status)", body)
        self.assertNotIn("capability.clone()", body)

    def test_package_statuses_are_indexed_before_plain_capabilities(self) -> None:
        body = function_body(self.source, "fn extend_package_manifest(")
        status_loop = body.index("for status in &manifest.capability_statuses")
        capability_extension = body.index("self.extend_capabilities")
        self.assertLess(status_loop, capability_extension)
        self.assertIn("self.provided.remove(status.capability.as_str())", body)
        self.assertNotIn("self.provided.insert(status.capability.clone())", body)

    def test_plain_capabilities_skip_keys_already_owned_by_statuses(self) -> None:
        body = function_body(self.source, "fn extend_capabilities")
        self.assertIn("if !self.statuses.contains_key(capability.as_str())", body)
        self.assertIn("self.provided.insert(capability.clone())", body)
        self.assertIn("disjoint_indexes_preserve_status_only_capability_queries", self.source)


if __name__ == "__main__":
    unittest.main()
