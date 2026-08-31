from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/plugin/export_build_plan/source_template_build_plan.rs"


class PreallocatedSourceTemplateCommandPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_fixed_paths_do_not_materialize_path_buffers(self) -> None:
        self.assertNotIn("use std::path::Path;", self.source)
        self.assertIn('const MANIFEST_PATH: &str = "Cargo.toml";', self.source)
        self.assertIn(
            'const TARGET_DIR: &str = "stages/source_template/target";',
            self.source,
        )
        self.assertNotIn('.replace(\'\\\\\', "/")', self.source)

    def test_release_command_capacity_is_preallocated(self) -> None:
        self.assertIn("const BASE_COMMAND_ARGUMENT_COUNT: usize = 6;", self.source)
        self.assertIn(
            "Vec::with_capacity(BASE_COMMAND_ARGUMENT_COUNT + usize::from(release))",
            self.source,
        )

    def test_rust_regression_covers_complete_release_command(self) -> None:
        self.assertIn("preallocated_source_template_command_preserves_contract", self.source)
        self.assertIn('"--release"', self.source)
        self.assertIn('"stages/source_template/target"', self.source)


if __name__ == "__main__":
    unittest.main()
