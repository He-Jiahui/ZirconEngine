from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
NATIVE = ROOT / "zircon_runtime/src/plugin/export_build_plan/materialize/native.rs"
MATERIALIZE = ROOT / "zircon_runtime/src/plugin/export_build_plan/materialize/mod.rs"


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


class PreallocatedNativePackageReportsPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.native = NATIVE.read_text(encoding="utf-8")
        cls.materialize = MATERIALIZE.read_text(encoding="utf-8")

    def test_package_directory_sets_use_selected_package_capacity(self) -> None:
        capacity = "HashSet::with_capacity(plan.native_dynamic_packages.len())"
        for signature in (
            "pub(super) fn materialize_native_dynamic_packages(",
            "pub(super) fn preview_native_dynamic_packages(",
        ):
            body = function_body(self.native, signature)
            self.assertIn(capacity, body)
            self.assertNotIn("HashSet::new()", body)

    def test_reports_reserve_copied_package_upper_bound(self) -> None:
        for signature in (
            "pub fn materialize_with_native_packages(",
            "pub fn preview_materialize_with_native_packages(",
        ):
            body = function_body(self.materialize, signature)
            self.assertIn("report.copied_packages.reserve(", body)
            self.assertIn("self.native_dynamic_packages.len()", body)

    def test_rust_contract_tracks_both_capacity_layers(self) -> None:
        self.assertIn("native_package_reports_preallocate_plan_bounds", self.native)
        self.assertIn('include_str!("mod.rs")', self.native)


if __name__ == "__main__":
    unittest.main()
