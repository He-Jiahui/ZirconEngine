from __future__ import annotations

import unittest
from pathlib import Path

from tools.session_coordinator.cargo_command_policy import (
    cargo_package_specs,
    cargo_target_argument,
    normalize_cargo_ticket_command,
    rewrite_cargo_source_path_arguments,
    validate_cargo_storage_arguments,
)
from tools.session_coordinator.models import CoordinatorError


class CargoCommandPolicyTests(unittest.TestCase):
    def assert_rejected(self, command: tuple[str, ...], code: str) -> None:
        with self.assertRaises(CoordinatorError) as rejected:
            normalize_cargo_ticket_command(command, Path("E:/repo"))
        self.assertEqual(code, rejected.exception.code)

    def test_new_cargo_ticket_rejects_opaque_environment_assignment(self) -> None:
        command = (
            "pwsh.exe",
            "-NoProfile",
            "-Command",
            "$env:CARGO_TARGET_DIR='D:\\unmanaged'; cargo test",
        )

        with self.assertRaises(CoordinatorError) as rejected:
            normalize_cargo_ticket_command(command, Path("E:/repo"))

        self.assertEqual(
            "validation_ticket_cargo_command_opaque", rejected.exception.code
        )

    def test_cargo_policy_stops_at_test_binary_delimiter(self) -> None:
        command = (
            "cargo",
            "test",
            "--locked",
            "-p",
            "app@1.2.3",
            "-p=dep",
            "--",
            "--target-dir",
            "D:/test-fixture",
            "--manifest-path=fixture/Cargo.toml",
            "--target=fixture-target",
        )

        validate_cargo_storage_arguments(command)
        rewritten = rewrite_cargo_source_path_arguments(
            command, lambda option, value: f"mapped:{option}:{value}"
        )

        self.assertEqual(command, rewritten)
        self.assertEqual(("app", "dep"), cargo_package_specs(command))
        self.assertIsNone(cargo_target_argument(command))

    def test_direct_locked_validation_is_accepted(self) -> None:
        command = ("cargo", "+1.94.1", "check", "--locked", "-p", "app@1.2.3")

        self.assertEqual(command, normalize_cargo_ticket_command(command, Path("E:/repo")))

    def test_path_qualified_cargo_is_rejected(self) -> None:
        self.assert_rejected(
            ("E:/repo/cargo.exe", "check", "--locked"),
            "validation_ticket_cargo_command_opaque",
        )

    def test_lock_is_required(self) -> None:
        self.assert_rejected(
            ("cargo", "check"), "validation_ticket_cargo_lock_required"
        )

    def test_toolchain_selector_must_be_exact(self) -> None:
        self.assert_rejected(
            ("cargo", "+nightly", "check", "--locked"),
            "validation_ticket_cargo_toolchain_unpinned",
        )

    def test_mutating_or_non_validation_commands_are_rejected(self) -> None:
        cases = (
            (("cargo", "publish", "--locked"), "validation_ticket_cargo_subcommand_forbidden"),
            (("cargo", "clippy", "--locked", "--fix"), "validation_ticket_cargo_mutation_forbidden"),
            (("cargo", "clippy", "--locked", "--allow-no-vcs"), "validation_ticket_cargo_mutation_forbidden"),
        )
        for command, code in cases:
            with self.subTest(command=command):
                self.assert_rejected(command, code)

    def test_coordinator_owned_and_unstable_arguments_are_rejected(self) -> None:
        cases = (
            (("cargo", "check", "--locked", "--target-dir", "out"), "validation_ticket_cargo_storage_override"),
            (("cargo", "check", "--locked", "--artifact-dir=out"), "validation_ticket_cargo_storage_override"),
            (("cargo", "check", "--locked", "--lockfile-path=other.lock"), "validation_ticket_cargo_storage_override"),
            (("cargo", "check", "--locked", "-Zunstable-options"), "validation_ticket_cargo_unstable_argument_forbidden"),
            (("cargo", "check", "--locked", "--jobs=8"), "validation_ticket_cargo_compute_override"),
        )
        for command, code in cases:
            with self.subTest(command=command):
                self.assert_rejected(command, code)

    def test_ambiguous_selectors_are_rejected(self) -> None:
        cases = (
            (("cargo", "check", "--locked", "--target", "a", "--target=b"), "validation_ticket_cargo_target_duplicate"),
            (("cargo", "check", "--locked", "-papp"), "validation_ticket_cargo_argument_unsupported"),
            (("cargo", "check", "--locked", "-Ffeature"), "validation_ticket_cargo_argument_unsupported"),
            (("cargo", "check", "--locked", "-p", "app*"), "validation_ticket_cargo_package_spec_unsupported"),
            (("cargo", "check", "--locked", "-p", "https://example.invalid/app"), "validation_ticket_cargo_package_spec_unsupported"),
        )
        for command, code in cases:
            with self.subTest(command=command):
                self.assert_rejected(command, code)

    def test_test_binary_arguments_after_delimiter_may_reuse_cargo_flag_names(self) -> None:
        command = (
            "cargo",
            "test",
            "--locked",
            "-p",
            "app",
            "--",
            "--jobs=99",
            "--fix",
            "-Ztest-only",
        )

        self.assertEqual(command, normalize_cargo_ticket_command(command, Path("E:/repo")))


if __name__ == "__main__":
    unittest.main()
