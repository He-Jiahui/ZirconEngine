from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT
    / "zircon_runtime/src/plugin/export_build_plan/materialize/package_lookup.rs"
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


class HashPackageResolutionSetsPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.build = function_body(cls.source, "pub(super) fn build(")

    def test_resolution_state_uses_hash_membership_sets(self) -> None:
        self.assertIn("collect::<HashSet<_>>()", self.build)
        self.assertIn("resolved_package_dirs", self.build)
        self.assertIn("HashSet", self.build)

    def test_selected_package_order_remains_deterministic(self) -> None:
        self.assertIn("collect::<BTreeSet<_>>()", self.build)
        self.assertIn("for package_id in &selected_package_ids", self.build)
        self.assertNotIn("let mut unresolved_package_ids = selected_package_ids.clone()", self.build)

    def test_rust_regressions_cover_order_and_duplicate_selection(self) -> None:
        self.assertIn("hash_resolution_sets_preserve_lexical_fallback_order", self.source)
        self.assertIn("hash_resolution_sets_deduplicate_selected_package_ids", self.source)


if __name__ == "__main__":
    unittest.main()
