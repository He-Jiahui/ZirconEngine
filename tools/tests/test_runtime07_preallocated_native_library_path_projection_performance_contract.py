from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT
    / "zircon_runtime/src/plugin/native_plugin_loader/candidate_from_manifest.rs"
)


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


class PreallocatedNativeLibraryPathProjectionPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.body = function_body(
            cls.source,
            "pub(super) fn native_library_paths_for_candidate(",
        )

    def test_path_projection_uses_module_kind_upper_bound(self) -> None:
        self.assertIn(
            "Vec::<(PathBuf, Vec<PluginModuleKind>)>::with_capacity(module_kinds.len())",
            self.body,
        )
        self.assertNotIn(
            "Vec::<(PathBuf, Vec<PluginModuleKind>)>::new()",
            self.body,
        )

    def test_existing_path_dedup_still_merges_module_kinds(self) -> None:
        self.assertIn("existing_kinds.push(*module_kind);", self.body)
        self.assertIn("paths.push((library_path, vec![*module_kind]));", self.body)

    def test_rust_regression_locks_returned_capacity_and_dedup(self) -> None:
        self.assertIn(
            "native_library_path_projection_preallocates_module_kind_bound",
            self.source,
        )
        self.assertIn("assert_eq!(paths.capacity(), module_kinds.len());", self.source)


if __name__ == "__main__":
    unittest.main()
