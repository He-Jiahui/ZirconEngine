from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/plugin/export_build_plan/from_project_manifest.rs"
SIGNATURE = "fn linked_runtime_plugin_projection("


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


class SinglePassLinkedRuntimeProjectionPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_plugin_projection_scans_enabled_plugins_once(self) -> None:
        self.assertIn(SIGNATURE, self.source)
        body = function_body(self.source, SIGNATURE)
        self.assertEqual(body.count("for &selection in enabled_plugins"), 1)
        self.assertNotIn("enabled_plugins.iter().filter", body)
        self.assertIn("crate_names.contains(crate_name)", body)
        self.assertIn("let crate_name = crate_name.to_string();", body)

    def test_projection_outputs_reserve_enabled_plugin_upper_bound(self) -> None:
        self.assertIn(SIGNATURE, self.source)
        body = function_body(self.source, SIGNATURE)
        self.assertGreaterEqual(body.count("HashSet::with_capacity(enabled_plugins.len())"), 2)
        self.assertIn("Vec::with_capacity(enabled_plugins.len())", body)

    def test_rust_regression_preserves_duplicate_crate_package_ids(self) -> None:
        self.assertIn("single_pass_linked_runtime_projection_preserves_contract", self.source)
        self.assertIn('"duplicate-runtime-a"', self.source)
        self.assertIn('"duplicate-runtime-b"', self.source)


if __name__ == "__main__":
    unittest.main()
