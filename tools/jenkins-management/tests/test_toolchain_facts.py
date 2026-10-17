"""Trusted toolchain resolution for actual-checkout management reads."""
from __future__ import annotations

import hashlib
import importlib.util
from pathlib import Path
import tempfile
import sys
import unittest

ROOT = Path(__file__).resolve().parents[3]
PREPARATION_PATH = ROOT / "tools/jenkins-management/source_preparation.py"
_SPEC = importlib.util.spec_from_file_location("toolchain_facts_test_preparation", PREPARATION_PATH)
preparation = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(preparation)


def file_fact(path):
    raw = Path(path).read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "byteLength": len(raw)}


class TrustedToolchainFactsTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.rustup = Path(self.temporary.name) / "rustup"
        self.root = self.rustup / "toolchains" / "1.94.1-x86_64-pc-windows-msvc"
        self.host = "x86_64-pc-windows-msvc"
        self.write(self.rustup / "settings.toml",
                   'default_toolchain = "stable-x86_64-pc-windows-msvc"\n'
                   'default_host_triple = "x86_64-pc-windows-msvc"\n')
        self.write(self.root / "bin/cargo.exe", b"trusted-cargo")
        self.write(self.root / "bin/rustc.exe", b"trusted-rustc")
        self.write(self.root / "bin/rustdoc.exe", b"trusted-rustdoc")
        self.write(self.root / "bin/host.dll", b"compiler-dll")
        self.write(self.root / "lib/rustlib" / self.host / "lib/std.rlib", b"host-sysroot")
        self.write(self.root / "lib/rustlib" / self.host / "bin/rust-lld.exe", b"host-linker")
        self.write(self.root / "lib/rustlib/bin/rustdoc.exe", b"rustdoc-helper")
        self.trusted = {
            "host": self.host,
            "cargoPath": str(self.root / "bin/cargo.exe"),
            "cargoSha256": file_fact(self.root / "bin/cargo.exe")["sha256"],
            "rustcPath": str(self.root / "bin/rustc.exe"),
            "rustcSha256": file_fact(self.root / "bin/rustc.exe")["sha256"],
        }

    def tearDown(self):
        self.temporary.cleanup()

    def write(self, path, value):
        path = Path(path)
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(value if isinstance(value, bytes) else value.encode("utf-8"))
        return path

    def capture(self, trusted=None, environment=None):
        return preparation.trusted_toolchain_facts(
            self.trusted if trusted is None else trusted, file_fact, environment=environment
        )

    def test_policy_pinned_1_94_1_wins_over_rustup_stable_default(self):
        facts = self.capture(environment={"RUSTUP_HOME": str(self.rustup)})
        self.assertEqual(self.root.name, facts["channel"])
        self.assertEqual(self.trusted["cargoPath"], facts["binaries"]["cargo"]["path"])
        self.assertEqual(self.trusted["rustcPath"], facts["binaries"]["rustc"]["path"])
        self.assertEqual(self.trusted["rustcSha256"], facts["binaries"]["rustc"]["sha256"])
        self.assertEqual(file_fact(self.rustup / "settings.toml")["sha256"], facts["settingsSha256"])

    def test_capture_contains_binary_dll_sysroot_settings_and_python_closure(self):
        facts = self.capture()
        captured = facts["plainFileFacts"]
        expected = {
            self.root / "bin/cargo.exe", self.root / "bin/rustc.exe", self.root / "bin/rustdoc.exe",
            self.root / "bin/host.dll", self.root / "lib/rustlib" / self.host / "lib/std.rlib",
            self.root / "lib/rustlib" / self.host / "bin/rust-lld.exe",
            self.root / "lib/rustlib/bin/rustdoc.exe", self.rustup / "settings.toml", Path(sys.executable),
        }
        self.assertTrue({str(path) for path in expected}.issubset(captured))
        expected_sysroot = {
            f"lib/rustlib/{self.host}/lib/std.rlib":
                file_fact(self.root / "lib/rustlib" / self.host / "lib/std.rlib")["sha256"],
            f"lib/rustlib/{self.host}/bin/rust-lld.exe":
                file_fact(self.root / "lib/rustlib" / self.host / "bin/rust-lld.exe")["sha256"],
            "lib/rustlib/bin/rustdoc.exe":
                file_fact(self.root / "lib/rustlib/bin/rustdoc.exe")["sha256"],
        }
        self.assertEqual(expected_sysroot, facts["sysrootInputs"])
        self.assertEqual(file_fact(self.root / "bin/host.dll")["sha256"],
                         facts["libraries"]["bin/host.dll"])

    def test_path_hash_and_rustup_overrides_are_rejected(self):
        wrong_hash = dict(self.trusted, cargoSha256="0" * 64)
        with self.assertRaisesRegex(ValueError, "trusted toolchain"):
            self.capture(wrong_hash)
        stable_override = dict(self.trusted, cargoPath=str(self.rustup / "toolchains/stable/bin/cargo.exe"))
        with self.assertRaisesRegex(ValueError, "compiler paths"):
            self.capture(stable_override)
        with self.assertRaisesRegex(ValueError, "RUSTUP_HOME"):
            self.capture(environment={"RUSTUP_HOME": str(self.rustup / "other")})
        with self.assertRaisesRegex(ValueError, "RUSTUP_TOOLCHAIN"):
            self.capture(environment={"RUSTUP_TOOLCHAIN": "stable-x86_64-pc-windows-msvc"})

    def test_dll_sysroot_and_settings_changes_change_the_captured_input_map(self):
        before = self.capture()
        self.write(self.root / "bin/host.dll", b"changed-dll")
        self.write(self.root / "lib/rustlib" / self.host / "lib/std.rlib", b"changed-sysroot")
        self.write(self.rustup / "settings.toml", 'default_toolchain = "stable"\nchanged = true\n')
        after = self.capture()
        self.assertNotEqual(before["plainFileFacts"], after["plainFileFacts"])
        self.assertNotEqual(before["libraries"], after["libraries"])
        self.assertNotEqual(before["sysrootInputs"], after["sysrootInputs"])
        self.assertNotEqual(before["settingsSha256"], after["settingsSha256"])

    def test_sysroot_relative_index_rejects_case_alias_and_allows_exact_repeat(self):
        names = {}
        relative = f"lib/rustlib/{self.host}/lib/std.rlib"
        preparation._index_sysroot_path(names, relative)
        preparation._index_sysroot_path(names, relative)
        self.assertEqual({relative.casefold(): relative}, names)
        with self.assertRaisesRegex(ValueError, "sysroot contains a case alias"):
            preparation._index_sysroot_path(names, relative.swapcase())

    def test_sysroot_inventory_tracks_added_changed_and_removed_files(self):
        before = self.capture()
        relative = f"lib/rustlib/{self.host}/lib/extra.rmeta"
        path = self.root / "lib/rustlib" / self.host / "lib/extra.rmeta"
        self.write(path, b"initial sysroot bytes")
        added = self.capture()
        self.assertNotIn(relative, before["sysrootInputs"])
        self.assertEqual(hashlib.sha256(b"initial sysroot bytes").hexdigest(),
                         added["sysrootInputs"][relative])
        self.write(path, b"changed sysroot bytes")
        changed = self.capture()
        self.assertNotEqual(added["sysrootInputs"][relative],
                            changed["sysrootInputs"][relative])
        path.unlink()
        removed = self.capture()
        self.assertNotIn(relative, removed["sysrootInputs"])


if __name__ == "__main__":
    unittest.main()
