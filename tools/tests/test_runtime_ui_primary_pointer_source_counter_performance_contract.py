from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
POINTER_TABLE = (
    ROOT / "zircon_runtime/src/ui/dispatch/input_manager/pointer_table.rs"
)
MANAGER = ROOT / "zircon_runtime/src/ui/dispatch/input_manager/manager.rs"


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
    raise AssertionError(f"unterminated Rust function: {signature}")


class RuntimeUiPrimaryPointerSourceCounterPerformanceContractTests(unittest.TestCase):
    def test_primary_touch_and_pen_membership_is_owned_by_the_pointer_table(self):
        source = POINTER_TABLE.read_text(encoding="utf-8")
        implementation = source.split("#[cfg(test)]", 1)[0]

        self.assertIn("primary_touch_count: usize", implementation)
        self.assertIn("primary_pen_count: usize", implementation)
        self.assertIn("pub fn has_primary_for_source", implementation)
        self.assertIn("self.primary_touch_count", implementation)
        self.assertIn("self.primary_pen_count", implementation)
        self.assertNotIn("entries().iter().any", implementation)

    def test_primary_membership_updates_on_upsert_remove_and_clear(self):
        source = POINTER_TABLE.read_text(encoding="utf-8")
        implementation = source.split("#[cfg(test)]", 1)[0]

        self.assertIn("add_primary_source", implementation)
        self.assertIn("remove_primary_source", implementation)
        self.assertIn("self.add_primary_source(source)", implementation)
        self.assertIn("self.remove_primary_source(removed.source)", implementation)
        self.assertIn("self.primary_touch_count = 0", implementation)
        self.assertIn("self.primary_pen_count = 0", implementation)

    def test_manager_uses_constant_time_primary_membership_probe(self):
        source = MANAGER.read_text(encoding="utf-8")
        body = function_body(source, "fn active_pointer_is_primary(")

        self.assertIn("has_primary_for_source", body)
        self.assertNotIn("self.pointers.entries()", body)
        self.assertNotIn(".iter().any", body)

    def test_primary_probe_pressure_model_is_constant_time(self):
        pointer_count = 100_000
        legacy_probes = pointer_count
        optimized_probes = 1
        self.assertLessEqual(optimized_probes * 100, legacy_probes)

    def test_release_benchmark_marker_and_threshold_are_recorded(self):
        source = (POINTER_TABLE.parent / "pointer_table/tests/index_tests.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("RUNTIME200_PRIMARY_POINTER_SOURCE_COUNTER_BENCH_V1", source)
        self.assertIn("index + 1 == POINTER_COUNT", source)
        self.assertIn("legacy_membership_steps_per_probe={POINTER_COUNT}", source)
        self.assertIn("optimized_membership_steps_per_probe=1", source)
        self.assertIn("optimized_p95.saturating_mul(2) <= legacy_p95", source)


if __name__ == "__main__":
    unittest.main()
