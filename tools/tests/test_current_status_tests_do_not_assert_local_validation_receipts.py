"""Reject machine-local validation receipts in current-status contract tests."""

from pathlib import Path
import unittest


TEST_ROOT = Path(__file__).resolve().parent
FORBIDDEN_FRAGMENTS = (
    "cargo check --manifest-path zircon_plugins\\Cargo.toml --workspace",
    "cargo test --manifest-path zircon_plugins\\Cargo.toml --workspace",
    "cargo-targets\\zircon-plugin-workspace",
    "StartedAt=",
    "FinishedAt=",
    "ElapsedSeconds=",
    "ExecutableLines=",
    "command timed out after",
)


class CurrentStatusValidationReceiptBoundaryTests(unittest.TestCase):
    def test_current_status_tests_do_not_assert_local_validation_receipts(self) -> None:
        violations: list[str] = []
        for path in sorted(TEST_ROOT.glob("test_plugin_docs_current_status*.py")):
            source = path.read_text(encoding="utf-8")
            for fragment in FORBIDDEN_FRAGMENTS:
                if fragment in source:
                    violations.append(f"{path.name}: {fragment}")

        self.assertEqual([], violations)


if __name__ == "__main__":
    unittest.main()
