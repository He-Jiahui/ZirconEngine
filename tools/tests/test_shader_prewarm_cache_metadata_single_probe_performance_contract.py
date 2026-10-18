import ast
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
TOOL_PATH = REPO_ROOT / "tools" / "build" / "zircon_build_shader_prewarm_cache_artifacts.py"


class ShaderPrewarmCacheMetadataSingleProbePerformanceContractTests(unittest.TestCase):
    def test_cache_scan_uses_one_metadata_probe_per_path(self) -> None:
        module = ast.parse(TOOL_PATH.read_text(encoding="utf-8"))
        functions = {
            node.name: node
            for node in module.body
            if isinstance(node, ast.FunctionDef)
        }
        scan_source = ast.unparse(functions["_shader_cache_artifact_pairs"])

        self.assertEqual(scan_source.count(".stat()"), 2)
        self.assertNotIn(".exists()", scan_source)
        self.assertNotIn(".is_file()", scan_source)
        self.assertNotIn(".is_dir()", scan_source)


if __name__ == "__main__":
    unittest.main()
