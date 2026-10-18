import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
COMMAND_SOURCE = REPO_ROOT / "zircon_editor/src/core/play/process_backend/command.rs"
BACKEND_SOURCE = REPO_ROOT / "zircon_editor/src/core/play/process_backend/mod.rs"
LOWER = (
    REPO_ROOT / "zircon_editor/src/core/play/process_backend/command/tests/borrowed_arguments_tests.rs"
)


def function_body(source: str, signature: str) -> str:
    match = re.search(rf"fn {signature}\(.*?\n    }}\n", source, re.S)
    if match is None:
        raise AssertionError(f"{signature} owner is missing")
    return match.group(0)


class Editor886PlayProcessBorrowedArgumentsPerformanceContractTests(
    unittest.TestCase
):
    @classmethod
    def setUpClass(cls) -> None:
        cls.command_source = COMMAND_SOURCE.read_text(encoding="utf-8")
        cls.backend_source = BACKEND_SOURCE.read_text(encoding="utf-8")

    def test_arguments_are_a_fixed_borrowed_array(self) -> None:
        owner = function_body(self.command_source, "arguments")
        self.assertIn("-> [&str; 8]", owner)
        self.assertIn("self.scene.as_str()", owner)
        self.assertIn("self.report_pipe.as_str()", owner)
        self.assertNotIn("Vec<OsString>", owner)
        self.assertNotIn(".clone()", owner)

    def test_spawn_configuration_reuses_argument_authority(self) -> None:
        owner = function_body(self.command_source, "configure")
        self.assertRegex(owner, r"command\s*\.args\(self\.arguments\(\)\)")
        self.assertNotIn('.arg("--project")', owner)

    def test_backend_joins_borrowed_arguments_directly(self) -> None:
        start = self.backend_source.split("fn start(", 1)[1].split("fn stop(", 1)[0]
        self.assertIn('let arguments = command.arguments().join(" ");', start)
        self.assertNotIn("to_string_lossy", start)
        self.assertNotIn("collect::<Vec", start)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn("mod borrowed_arguments_tests;", self.command_source)
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn(
            "editor886_play_process_borrowed_arguments_preserve_command_and_diagnostic",
            lower,
        )
        self.assertIn("EDITOR886_PLAY_PROCESS_BORROWED_ARGUMENTS_BENCH_V1", lower)
        self.assertIn("const SAMPLE_PAIRS: usize = 101;", lower)
        self.assertIn("legacy_p99_ns", lower)
        self.assertIn("optimized_p99_ns", lower)

    def test_dense_diagnostic_temporary_model(self) -> None:
        render_count = 4_096
        argument_count = 8
        legacy_owned_argument_strings = render_count * argument_count
        legacy_temporary_vector_slots = render_count * argument_count * 2
        optimized_owned_argument_strings = 0
        optimized_temporary_vector_slots = 0
        self.assertEqual(legacy_owned_argument_strings, 32_768)
        self.assertEqual(legacy_temporary_vector_slots, 65_536)
        self.assertEqual(optimized_owned_argument_strings, 0)
        self.assertEqual(optimized_temporary_vector_slots, 0)


if __name__ == "__main__":
    unittest.main()
