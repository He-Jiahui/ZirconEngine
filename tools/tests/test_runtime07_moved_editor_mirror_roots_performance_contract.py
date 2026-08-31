from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_plugins/plugin_sdk/src/editor.rs"


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


class MovedEditorMirrorRootsPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.body = function_body(
            cls.source,
            "pub fn mirrors_runtime_manifest(",
        )

    def test_asset_roots_move_out_before_manifest_replacement(self) -> None:
        self.assertIn(
            "std::mem::take(&mut self.base_manifest.asset_roots)",
            self.body,
        )
        self.assertNotIn("self.base_manifest.asset_roots.clone()", self.body)

    def test_content_roots_move_out_before_manifest_replacement(self) -> None:
        self.assertIn(
            "std::mem::take(&mut self.base_manifest.content_roots)",
            self.body,
        )
        self.assertNotIn("self.base_manifest.content_roots.clone()", self.body)

    def test_rust_regression_locks_root_buffer_identity(self) -> None:
        self.assertIn("mirrored_manifest_moves_editor_root_buffers", self.source)
        self.assertIn("asset_root_buffer", self.source)
        self.assertIn("content_root_buffer", self.source)


if __name__ == "__main__":
    unittest.main()
