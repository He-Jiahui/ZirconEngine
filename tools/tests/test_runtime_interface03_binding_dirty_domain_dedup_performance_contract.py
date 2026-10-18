import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/binding/model/update.rs"


def function_body(source: str, signature: str) -> str:
    start = source.index(signature)
    brace = source.index("{", start)
    depth = 0
    for index in range(brace, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[brace : index + 1]
    raise AssertionError(f"unterminated function body for {signature}")


class RuntimeInterface03BindingDirtyDomainDedupPerformanceContract(unittest.TestCase):
    def test_dirty_flag_projection_presizes_the_complete_domain_set(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        body = function_body(source, "pub fn from_dirty_flags(flags: UiDirtyFlags)")

        self.assertIn("Vec::with_capacity(7)", body)
        self.assertNotIn("Vec::new()", body)
        self.assertIn(
            "RUNTIME_INTERFACE03_PRESIZED_BINDING_DIRTY_DOMAINS_BENCH_V1",
            source,
        )

    def test_update_report_uses_constant_time_dirty_domain_membership(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        body = function_body(source, "pub fn recompute(&mut self)")

        self.assertIn("let mut dirty_mask = 0u16;", body)
        self.assertIn("let domain_mask = domain.bit();", body)
        self.assertIn("dirty_mask & domain_mask == 0", body)
        self.assertIn("dirty_mask |= domain_mask;", body)
        self.assertNotIn("self.dirty.contains(domain)", body)
        self.assertIn("const fn bit(self) -> u16", source)
        self.assertIn(
            "RUNTIME_INTERFACE03_BINDING_DIRTY_DOMAIN_BITSET_DEDUP_BENCH_V1",
            source,
        )


if __name__ == "__main__":
    unittest.main()
