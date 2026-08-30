import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PLUGIN_DIAGNOSTICS = (
    REPO_ROOT / "zircon_runtime_interface" / "src" / "plugin_diagnostics.rs"
)


def function_body(source: str, function_name: str) -> str:
    match = re.search(rf"\bfn\s+{re.escape(function_name)}\s*\(", source)
    if match is None:
        raise AssertionError(f"missing function {function_name}")
    opening = source.find("{", match.end())
    if opening < 0:
        raise AssertionError(f"missing body for {function_name}")
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index]
    raise AssertionError(f"unterminated body for {function_name}")


class PluginDiagnosticPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = PLUGIN_DIAGNOSTICS.read_text(encoding="utf-8")

    def test_missing_capability_builds_one_reserved_message_before_moving_plugin_id(self) -> None:
        body = function_body(self.source, "missing_capability")
        self.assertIn("String::with_capacity", body)
        self.assertIn('message.push_str("editor plugin `")', body)
        self.assertIn("message.push_str(&plugin_id)", body)
        self.assertIn("message.push_str(&capability)", body)
        self.assertIn("plugin_id,", body)
        self.assertIn("message,", body)
        self.assertNotIn("plugin_id.clone()", body)
        self.assertNotIn("format!", body)
        self.assertLess(body.index("String::with_capacity"), body.index("Self::new("))

    def test_release_evidence_tracks_plugin_id_clone_elision(self) -> None:
        self.assertIn(
            "PERF_RESULT runtime_interface04_registration_diagnostic",
            self.source,
        )
        self.assertIn("legacy_plugin_id_clones=1", self.source)
        self.assertIn("optimized_plugin_id_clones=0", self.source)


if __name__ == "__main__":
    unittest.main()
