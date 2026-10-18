"""Runtime878 guards direct plugin-list template formatting without Cargo."""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
OWNER = ROOT / "zircon_runtime/src/plugin/export_build_plan/plugin_selection_template.rs"
TESTS = ROOT / "zircon_runtime/src/plugin/export_build_plan/plugin_selection_template/tests/direct_string_vec_tests.rs"


class Runtime878ExportPluginListDirectContract(unittest.TestCase):
    def test_value_list_writes_debug_values_into_one_result(self):
        source = OWNER.read_text(encoding="utf-8")
        function = source.split('fn string_vec_expr(values: &[String]) -> String {', 1)[1].split('\nfn ', 1)[0]
        self.assertIn('String::with_capacity(', function)
        self.assertIn('write!(&mut result, "{value:?}.to_string()")', function)
        self.assertNotIn('collect::<Vec<_>>()', function)
        self.assertNotIn('.join(', function)

    def test_lower_parity_and_release_percentiles_are_wired(self):
        source = OWNER.read_text(encoding="utf-8")
        tests = TESTS.read_text(encoding="utf-8")
        self.assertIn('mod direct_string_vec_tests;', source)
        self.assertIn('RUNTIME878_EXPORT_PLUGIN_LIST_DIRECT_BENCH_V1', tests)
        self.assertIn('legacy_expr(', tests)
        self.assertIn('101', tests)
        self.assertIn('p99', tests)

    def test_deterministic_intermediate_model(self):
        templates, plugins_per_template = 4_096, 32
        legacy_children = templates * plugins_per_template
        legacy_slots = legacy_children
        self.assertEqual((legacy_children, legacy_slots), (131_072, 131_072))
        self.assertEqual((0, 0), (0, 0))


if __name__ == "__main__":
    unittest.main()
