import ast
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
IDENTITY_PATH = REPO_ROOT / "tools" / "profiling" / "shader_pbr" / "zircon_shader_pbr_evidence_identity.py"


class ShaderPbrIdentityManifestSingleReadPerformanceContractTests(unittest.TestCase):
    def test_identity_manifest_reuses_the_verified_payload(self) -> None:
        module = ast.parse(IDENTITY_PATH.read_text(encoding="utf-8"))
        functions = {
            node.name: node
            for node in module.body
            if isinstance(node, ast.FunctionDef)
        }
        loader_source = ast.unparse(functions["_load_verified_identity"])
        validator_source = ast.unparse(functions["validate_ready_frame_identity"])

        self.assertEqual(loader_source.count("read_bytes()"), 1)
        self.assertIn("hashlib.sha256(payload)", loader_source)
        self.assertIn("len(payload)", loader_source)
        self.assertIn("json.loads(payload.decode('utf-8-sig'))", loader_source)
        self.assertNotIn("identity_path.read_text", validator_source)
        self.assertNotIn("_require_file_fingerprint(identity_path", validator_source)


if __name__ == "__main__":
    unittest.main()
