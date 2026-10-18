import ast
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
TOOL_PATHS = (
    REPO_ROOT / "tools" / "performance" / "ui" / "ui_render_segment_evidence.py",
    REPO_ROOT / "tools" / "performance" / "ui" / "ui_render_dependency_delta_evidence.py",
    REPO_ROOT / "tools" / "performance" / "ui" / "ui_render_dependency_memory_evidence.py",
)


class UiRenderEvidenceSingleReadPerformanceContractTests(unittest.TestCase):
    def test_timeline_and_manifest_reuse_one_payload_snapshot(self) -> None:
        for tool_path in TOOL_PATHS:
            with self.subTest(tool=tool_path.name):
                module = ast.parse(tool_path.read_text(encoding="utf-8"))
                functions = {
                    node.name: node
                    for node in module.body
                    if isinstance(node, ast.FunctionDef)
                }
                loader_source = ast.unparse(functions["_load_json_evidence"])
                main_source = ast.unparse(functions["main"])

                self.assertEqual(loader_source.count("read_bytes()"), 1)
                self.assertIn("hashlib.sha256(payload)", loader_source)
                self.assertNotIn("read_text(", main_source)
                self.assertNotIn("_sha256(timeline_path)", main_source)
                self.assertNotIn("_sha256(source_manifest_path)", main_source)


if __name__ == "__main__":
    unittest.main()
