from pathlib import Path
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
LIBRARY_EMBED_PLAN = (
    REPO_ROOT
    / "zircon_runtime"
    / "src"
    / "plugin"
    / "export_build_plan"
    / "library_embed_compile_plan.rs"
)


class RuntimeExportLibraryEmbedCommandPolicyTests(unittest.TestCase):
    def test_library_embed_compile_host_command_carries_manifest_path(self) -> None:
        source = LIBRARY_EMBED_PLAN.read_text(encoding="utf-8")

        self.assertIn('"--manifest-path".to_string()', source)
        self.assertIn("manifest_path.clone()", source)


if __name__ == "__main__":
    unittest.main()
