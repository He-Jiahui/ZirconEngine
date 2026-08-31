from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
OWNER = ROOT / "zircon_editor/src/ui/retained_host/host_contract/workbench_context_menu"
PATH = OWNER / "path.rs"
PROVIDER = OWNER / "provider.rs"
REQUEST = OWNER / "request.rs"


class EditorWorkbenchContextMenuPerformanceContractTests(unittest.TestCase):
    def test_target_uri_is_written_into_one_final_buffer(self) -> None:
        path = PATH.read_text(encoding="utf-8")
        provider = PROVIDER.read_text(encoding="utf-8")

        self.assertIn("fn push_path_segment(", path)
        self.assertIn("path: &mut String", path)
        self.assertNotIn(".collect::<String>()", path)
        self.assertNotIn(".trim_matches('-')", path)
        self.assertIn("String::with_capacity(prefix.len() + target.len())", provider)
        self.assertIn("push_path_segment(&mut target_path, target);", provider)
        self.assertNotIn("format!(", provider)

    def test_request_resolves_target_value_once(self) -> None:
        source = REQUEST.read_text(encoding="utf-8")

        self.assertEqual(source.count("target_value_text(hit)"), 1)
        self.assertIn("let target_path = provider.target_path(hit, target_value.as_str());", source)
        self.assertIn("target_value_text: target_value,", source)


if __name__ == "__main__":
    unittest.main()
