import ast
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
ORACLE_PATH = REPO_ROOT / "tools" / "visual" / "zircon_editor_ui_visual_oracle.py"


class EditorVisualOracleProfileSingleReadPerformanceContractTests(unittest.TestCase):
    def test_profile_json_reuses_the_verified_payload(self) -> None:
        module = ast.parse(ORACLE_PATH.read_text(encoding="utf-8"))
        functions = {
            node.name: node
            for node in module.body
            if isinstance(node, ast.FunctionDef)
        }
        loader_source = ast.unparse(functions["_load_verified_json"])
        capture_source = ast.unparse(functions["_analyze_capture"])

        self.assertEqual(loader_source.count("read_bytes()"), 1)
        self.assertIn("hashlib.sha256(payload)", loader_source)
        self.assertIn("json.loads(payload.decode('utf-8-sig'))", loader_source)
        self.assertNotIn("_verify_file_sha256(profile_path", capture_source)
        self.assertNotIn("_load_json(profile_path)", capture_source)

    def test_screenshot_decoder_reuses_the_verified_payload(self) -> None:
        module = ast.parse(ORACLE_PATH.read_text(encoding="utf-8"))
        functions = {
            node.name: node
            for node in module.body
            if isinstance(node, ast.FunctionDef)
        }
        capture_source = ast.unparse(functions["_analyze_capture"])

        self.assertIn("_load_verified_bytes(", capture_source)
        self.assertIn("_RgbImage.from_png_bytes(", capture_source)
        self.assertNotIn("_verify_file_sha256(screenshot_path", capture_source)
        self.assertNotIn("_RgbImage.load_png(screenshot_path)", capture_source)


if __name__ == "__main__":
    unittest.main()
