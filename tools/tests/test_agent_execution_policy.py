"""Regression boundaries for independent command execution; never compile Cargo."""
from __future__ import annotations

import contextlib
import io
import json
import os
from pathlib import Path, PureWindowsPath
import tempfile
import unittest
from unittest.mock import patch

from tools.build import zircon_build_cargo_environment as staged
from tools.dev import local_cargo


class IndependentExecutionPolicyTests(unittest.TestCase):
    def test_native_artifact_mode_stays_unavailable_before_any_source_or_process_access(self):
        with patch.object(local_cargo, "plain_path", side_effect=AssertionError("source read")), \
                patch.object(local_cargo, "_run_with_evidence", side_effect=AssertionError("native runner")), \
                patch.object(local_cargo.subprocess, "run", side_effect=AssertionError("process started")), \
                contextlib.redirect_stdout(io.StringIO()) as output:
            result = local_cargo.main([
                "--evidence-directory", "unused-output", "--evidence-input-manifest", "unused-input",
                "--evidence-timeout-seconds", "10", "--", "build", "-p", "selected", "--no-default-features",
            ])
        self.assertEqual(2, result)
        receipt = json.loads(output.getvalue())
        self.assertIn("not supported", receipt["message"])
        self.assertFalse(receipt["commandExecuted"])
        self.assertFalse(receipt["formalAcceptance"])

    @unittest.skipUnless(os.name == "nt", "Pinned dependency has a Windows MAX_PATH budget")
    def test_default_cargo_home_fits_pinned_dependency_checkout_without_private_pool_copies(self):
        relative = (
            "git/checkouts/zr_vm-91d161ef2b98ea29/3c1352e/tests/language_server/"
            "test_lsp_numeric_loop_self_dependency_conditional_delta_negative_offset_"
            "zero_inclusive_subtractive_semantic_query.c"
        )
        with tempfile.TemporaryDirectory() as folder:
            target = local_cargo._default_target(Path(folder).resolve(), [], dry_run=True)
        home = PureWindowsPath(str(target.parent / "cargo-home"))
        self.assertLessEqual(len(str(home / relative)), 259)
        self.assertNotIn(".jenkins", str(home))

    @unittest.skipUnless(os.name == "nt", "Physical Windows path admission")
    def test_staged_alias_rejection_precedes_resolution(self):
        with patch.object(local_cargo, "validate_output_path", side_effect=ValueError("path alias")), \
                patch.object(Path, "resolve", side_effect=AssertionError("resolved an unadmitted alias")):
            with self.assertRaisesRegex(ValueError, "approved build root"):
                staged.assert_managed_windows_build_root(Path(r"C:\alias-to-approved-root\product"))

    def test_staged_environment_drops_ambient_compiler_output_overrides(self):
        hostile = {
            "CARGO_BUILD_BUILD_DIR": "C:/escape", "CARGO_HOME": "C:/foreign-home",
            "RUSTFLAGS": "-Cincremental=C:/escape", "CARGO_ENCODED_RUSTFLAGS": "escape",
            "RUSTC": "C:/foreign-rustc.exe", "RUSTDOC": "C:/foreign-rustdoc.exe",
            "RUSTC_WRAPPER": "C:/foreign-wrapper.exe", "ZIRCON_JENKINS_IDENTITY_FILE": "not-read",
        }
        with tempfile.TemporaryDirectory() as folder:
            cache = Path(folder).resolve()
            target = cache / "targets" / "runtime"
            # Synthetic directories are only the environment unit boundary. The
            # separate physical-root tests keep real admission fully enabled.
            with patch.dict(os.environ, hostile), \
                    patch.object(local_cargo, "validate_output_path", side_effect=Path), \
                    patch.object(local_cargo, "free_bytes", return_value=100 * 1024**3), \
                    patch.object(staged, "assert_managed_windows_build_root"), \
                    patch.object(local_cargo.subprocess, "run", side_effect=AssertionError("process started")):
                environment = staged.managed_cargo_environment(target, cache)
            self.assertEqual(str(target / "build-intermediates"), environment["CARGO_BUILD_BUILD_DIR"])
            self.assertEqual("", environment["RUSTC_WRAPPER"])
            for name in ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTC", "RUSTDOC", "ZIRCON_JENKINS_IDENTITY_FILE"):
                self.assertNotIn(name, environment)
            for name in ("CARGO_TARGET_DIR", "CARGO_BUILD_BUILD_DIR", "CARGO_HOME", "SCCACHE_DIR", "TEMP", "TMP", "TMPDIR"):
                self.assertTrue(Path(environment[name]).is_relative_to(cache), name)

    def test_staged_low_capacity_is_rejected_before_directory_creation(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder).resolve()
            with patch.object(local_cargo, "validate_output_path", side_effect=Path), \
                    patch.object(staged, "assert_managed_windows_build_root"), \
                    patch.object(local_cargo, "free_bytes", return_value=1), \
                    patch.object(Path, "mkdir", side_effect=AssertionError("created outputs on low capacity")):
                with self.assertRaisesRegex(ValueError, "35 GiB"):
                    staged.managed_cargo_environment(root / "runtime", root)


if __name__ == "__main__":
    unittest.main()
