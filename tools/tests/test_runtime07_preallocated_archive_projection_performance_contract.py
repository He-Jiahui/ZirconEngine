from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/plugin/export_build_plan/materialize/archive.rs"


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


class PreallocatedArchiveProjectionPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.materialize = function_body(cls.source, "pub(super) fn materialize_zip_archive(")
        cls.preview = function_body(cls.source, "pub(super) fn preview_zip_archive(")
        cls.write_packages = function_body(cls.source, "fn write_native_package_entries")
        cls.preview_packages = function_body(cls.source, "fn preview_native_package_entries")
        cls.reuse_regression = function_body(
            cls.source,
            "fn archive_materialization_does_not_preview_then_rescan_each_package()",
        )

    def test_reports_preallocate_known_projection_bounds(self) -> None:
        for body in (self.materialize, self.preview):
            self.assertIn(
                "Vec::with_capacity(plan.generated_files.len())",
                body,
            )
            self.assertIn(
                "Vec::with_capacity(plan.native_dynamic_packages.len())",
                body,
            )

    def test_archive_dedup_sets_preallocate_known_bounds(self) -> None:
        self.assertIn("HashSet::with_capacity(archive_entry_capacity(plan))", self.materialize)
        for body in (self.write_packages, self.preview_packages):
            self.assertIn(
                "HashSet::with_capacity(plan.native_dynamic_packages.len())",
                body,
            )

    def test_inventory_reuse_regression_tracks_current_helper(self) -> None:
        self.assertIn("inventory.file_inventory(package_id)", self.write_packages)
        self.assertIn("inventory.file_inventory(package_id)", self.reuse_regression)
        self.assertNotIn("native_dynamic_package_file_inventory", self.reuse_regression)


if __name__ == "__main__":
    unittest.main()
