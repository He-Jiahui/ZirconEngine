from __future__ import annotations

import contextlib
import io
import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from tools.dev import local_cargo


@unittest.skipUnless(os.name == "nt", "Windows storage contract")
class LocalCargoTests(unittest.TestCase):
    def test_only_physical_drive_roots_are_accepted(self) -> None:
        for path in (r"D:\cargo-targets\zircon-local\test", r"E:\cargo-targets\zircon-local\test", r"F:\cargo-targets\zircon-local\test"):
            self.assertEqual(Path(path), local_cargo.validate_output_path(path))
        for path in (r"C:\cargo-targets\test", r"D:\targets\test", r"E:\ZirconBuilds\test", r"E:\nested\cargo-targets\test", r"D:\cargo-targets", r"D:\cargo-targets\..\elsewhere", "target"):
            with self.subTest(path=path), self.assertRaises(ValueError):
                local_cargo.validate_output_path(path)

    def test_junction_alias_cannot_admit_an_outside_directory(self) -> None:
        with patch.object(Path, "resolve", return_value=Path(r"C:\outside")):
            with self.assertRaises(ValueError):
                local_cargo.validate_output_path(r"D:\cargo-targets\aliased\test")

    def test_dry_run_never_creates_output_or_contacts_coordinator(self) -> None:
        out = io.StringIO()
        with patch.object(Path, "mkdir", side_effect=AssertionError("directory created")), patch.object(local_cargo.subprocess, "run", side_effect=AssertionError("process started")), contextlib.redirect_stdout(out):
            code = local_cargo.main(["--target-dir", r"D:\cargo-targets\zircon-local\dry-test", "--dry-run", "--", "check", "-p", "zircon_runtime", "--locked"])
        self.assertEqual(0, code)
        report = json.loads(out.getvalue())
        self.assertEqual("dry_run", report["status"])
        self.assertIn("--target-dir", report["command"])
        self.assertFalse(report["formalAcceptance"])

    def test_target_override_and_cargo_configuration_cannot_escape_preflight(self) -> None:
        for args in (["check", "--target-dir", "C:/bad"], ["check", "--target-dir=C:/bad"], ["--config", "build.target-dir='C:/bad'", "check"], ["check", "--config=build.build-dir='C:/bad'"]):
            with self.subTest(args=args), self.assertRaises(ValueError):
                local_cargo.prepare_command(args, Path(r"D:\cargo-targets\zircon-local\test"))

    def test_runtime_cdylib_command_uses_same_physical_preflight(self) -> None:
        command = local_cargo.prepare_command(['rustc', '-p', 'zircon_runtime', '--lib', '--crate-type', 'cdylib'], Path(r'D:\cargo-targets\zircon-local\runtime-dll'))
        self.assertEqual(['cargo', 'rustc', '-p', 'zircon_runtime', '--lib', '--crate-type', 'cdylib'], command[:7])
        self.assertIn('--locked', command)
        self.assertIn('--target-dir', command)
        for arguments in (['rustc', '--', '-o', 'C:/escape.exe'],
                          ['rustc', '--artifact-dir', 'C:/escape']):
            with self.assertRaises(ValueError):
                local_cargo.prepare_command(arguments, Path(r'D:\cargo-targets\zircon-local\runtime-dll'))

    def test_test_binary_arguments_are_preserved_after_separator(self) -> None:
        command = local_cargo.prepare_command(["test", "--lib", "--", "--exact", "--nocapture"], Path(r"D:\cargo-targets\zircon-local\test"))
        self.assertEqual(["--exact", "--nocapture"], command[command.index("--") + 1:])
        self.assertLess(command.index("--target-dir"), command.index("--"))

    def test_low_disk_rejects_before_creating_directories_or_running_cargo(self) -> None:
        with patch.object(local_cargo, "free_bytes", return_value=1024), patch.object(Path, "mkdir", side_effect=AssertionError("directory created")), patch.object(local_cargo.subprocess, "run", side_effect=AssertionError("Cargo ran")), contextlib.redirect_stderr(io.StringIO()):
            self.assertEqual(2, local_cargo.main(["--target-dir", r"D:\cargo-targets\zircon-local\low-disk", "--", "check"]))

    def test_powershell_wrapper_preserves_package_and_test_scope(self) -> None:
        wrapper = Path(__file__).resolve().parents[1] / 'dev/local-cargo.ps1'
        shell = Path(os.environ['SystemRoot']) / 'System32/WindowsPowerShell/v1.0/powershell.exe'
        result = subprocess.run([str(shell), '-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass', '-File', str(wrapper),
                                 '-DryRun', 'test', '-p', 'zircon_runtime', '--lib', 'focused_filter',
                                 '--locked', '--', '--exact'], capture_output=True, text=True,
                                encoding='utf-8', errors='replace', check=False)
        self.assertEqual(0, result.returncode, result.stderr)
        command = json.loads(result.stdout)['command']
        self.assertEqual('zircon_runtime', command[command.index('-p') + 1])
        self.assertIn('focused_filter', command)
        self.assertEqual(['--exact'], command[command.index('--') + 1:])

    def test_retired_conventions_do_not_delegate_to_old_managed_validator(self) -> None:
        from tools.audits.check_conventions import convention_commands_for_repo

        with tempfile.TemporaryDirectory() as folder:
            repo = Path(folder)
            marker = repo / '.codex/coordinator-retirement.json'
            marker.parent.mkdir()
            marker.write_text('{"schemaVersion":1,"status":"retired"}', encoding='utf-8')
            with patch('tools.audits.check_conventions._requires_managed_cargo_delegation', side_effect=AssertionError('old delegation used')):
                commands = convention_commands_for_repo(repo)
            for command in commands:
                if command.name in {'structure', 'clippy'}:
                    self.assertIn('tools.dev.local_cargo', command.argv)
                    self.assertNotIn('validate-matrix.ps1', ' '.join(command.argv))
                    self.assertIn('--locked', command.argv)


if __name__ == "__main__":
    unittest.main()
