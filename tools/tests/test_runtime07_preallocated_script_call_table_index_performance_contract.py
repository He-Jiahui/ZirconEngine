from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/script/vm/host/script_call_table.rs"


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


class PreallocatedScriptCallTableIndexPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.body = function_body(
            cls.source,
            "pub(crate) fn from_entries(generation: u64, entries: Vec<ScriptCallSite>)",
        )

    def test_outer_module_index_uses_group_count_capacity(self) -> None:
        self.assertIn("let module_count = entries", self.body)
        self.assertIn("HashMap::with_capacity(module_count)", self.body)
        self.assertNotIn("let mut by_name = HashMap::<", self.body)

    def test_each_contiguous_module_group_preallocates_its_function_index(self) -> None:
        self.assertIn("let function_count = group_end - group_start;", self.body)
        self.assertIn("HashMap::with_capacity(function_count)", self.body)
        self.assertIn("for entry in &entries[group_start..group_end]", self.body)

    def test_rust_regression_preserves_repeated_module_groups(self) -> None:
        self.assertIn(
            "from_entries_preserves_non_contiguous_module_groups",
            self.source,
        )
        self.assertIn('table.resolve("runtime.time", "now")', self.source)
        self.assertIn('table.resolve("runtime.time", "delta")', self.source)


if __name__ == "__main__":
    unittest.main()
