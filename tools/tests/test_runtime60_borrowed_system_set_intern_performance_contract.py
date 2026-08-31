from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/scene/ecs/system_set.rs"


def function_body(source: str, name: str) -> str:
    match = re.search(rf"\bfn\s+{re.escape(name)}\b", source)
    if match is None:
        raise AssertionError(f"missing function {name}")
    start = source.find("{", match.end())
    depth = 0
    for index in range(start, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[start : index + 1]
    raise AssertionError(f"unterminated body for {name}")


class BorrowedSystemSetInternPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.intern = function_body(cls.source, "intern")

    def test_intern_accepts_borrowed_or_owned_names(self) -> None:
        self.assertIn("borrow::Cow", self.source)
        self.assertRegex(
            self.source,
            r"pub fn intern<'a>\(\s*&mut self,\s*name: impl Into<Cow<'a, str>>,?\s*\)",
        )

    def test_existing_name_lookup_precedes_owned_materialization(self) -> None:
        lookup = "self.ids_by_name.get(name.as_ref())"
        materialize = "let name = name.into_owned();"
        self.assertIn(lookup, self.intern)
        self.assertIn(materialize, self.intern)
        self.assertLess(self.intern.index(lookup), self.intern.index(materialize))
        self.assertNotIn("get(&name)", self.intern)

    def test_rust_regressions_cover_borrowed_owned_and_invalid_inputs(self) -> None:
        self.assertIn(
            "fn runtime60_batch_borrowed_system_set_intern_reuses_dense_id()",
            self.source,
        )
        self.assertIn(
            "fn runtime60_batch_owned_system_set_intern_preserves_name()", self.source
        )
        self.assertIn(
            "fn runtime60_batch_invalid_borrowed_system_set_does_not_mutate_registry()",
            self.source,
        )


if __name__ == "__main__":
    unittest.main()
