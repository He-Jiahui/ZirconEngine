from __future__ import annotations

import unittest

from tools.zircon_export.command_plan import (
    command_option_value_diagnostic,
    command_with_option,
)


class CommandPlanTests(unittest.TestCase):
    def test_command_with_option_rewrites_existing_value(self) -> None:
        command = ["cargo", "build", "--target-dir", "old-target"]

        rewritten = command_with_option(command, "--target-dir", "new-target")

        self.assertEqual(rewritten, ["cargo", "build", "--target-dir", "new-target"])

    def test_command_with_option_appends_missing_option(self) -> None:
        command = ["cargo", "build"]

        rewritten = command_with_option(command, "--target-dir", "target")

        self.assertEqual(rewritten, ["cargo", "build", "--target-dir", "target"])

    def test_command_option_value_diagnostic_rejects_missing_value(self) -> None:
        diagnostic = command_option_value_diagnostic(
            ["cargo", "build", "--target-dir"],
            "--target-dir",
            "Export command",
        )

        self.assertEqual(
            diagnostic,
            "Export command --target-dir must include a value",
        )

    def test_command_option_value_diagnostic_rejects_option_value(self) -> None:
        diagnostic = command_option_value_diagnostic(
            ["cargo", "build", "--target-dir", "--release"],
            "--target-dir",
            "Export command",
        )

        self.assertEqual(
            diagnostic,
            "Export command --target-dir value must not be another option",
        )

    def test_command_option_value_diagnostic_rejects_duplicate_option(self) -> None:
        diagnostic = command_option_value_diagnostic(
            [
                "cargo",
                "build",
                "--manifest-path",
                "Cargo.toml",
                "--manifest-path",
                "other/Cargo.toml",
            ],
            "--manifest-path",
            "Export command",
        )

        self.assertEqual(
            diagnostic,
            "Export command --manifest-path must appear only once",
        )

    def test_command_option_value_diagnostic_accepts_single_value(self) -> None:
        diagnostic = command_option_value_diagnostic(
            ["cargo", "build", "--manifest-path", "Cargo.toml"],
            "--manifest-path",
            "Export command",
        )

        self.assertIsNone(diagnostic)


if __name__ == "__main__":
    unittest.main()
