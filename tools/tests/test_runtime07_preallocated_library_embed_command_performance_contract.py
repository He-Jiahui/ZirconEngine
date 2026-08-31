from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/plugin/export_build_plan/library_embed_compile_plan.rs"


class PreallocatedLibraryEmbedCommandPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_fixed_paths_do_not_materialize_path_buffers(self) -> None:
        self.assertNotIn("use std::path::{Path, PathBuf};", self.source)
        self.assertIn('const MANIFEST_PATH: &str = "Cargo.toml";', self.source)
        self.assertIn(
            'const TARGET_DIR: &str = "stages/compile_host/target";',
            self.source,
        )
        self.assertNotIn("Path::new", self.source)
        self.assertNotIn("PathBuf::from", self.source)

    def test_release_command_capacity_is_preallocated(self) -> None:
        self.assertIn("const BASE_COMMAND_ARGUMENT_COUNT: usize = 13;", self.source)
        self.assertIn(
            "Vec::with_capacity(BASE_COMMAND_ARGUMENT_COUNT + usize::from(release))",
            self.source,
        )

    def test_rust_regression_covers_complete_release_command(self) -> None:
        self.assertIn("preallocated_library_embed_command_preserves_contract", self.source)
        self.assertIn('"--no-default-features"', self.source)
        self.assertIn('"stages/compile_host/target"', self.source)


if __name__ == "__main__":
    unittest.main()
