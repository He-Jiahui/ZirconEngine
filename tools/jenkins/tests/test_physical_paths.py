import os
import shutil
import subprocess
import unittest
from pathlib import Path

from tools.jenkins.resources.paths import canonical_build_root, build_namespace, physical_path_under

REPO_ROOT = Path(__file__).resolve().parents[3]
BUILD_ROOT = REPO_ROOT / ".jenkins" / "builds"


class PhysicalPathTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        if os.name != "nt":
            raise unittest.SkipTest("Windows physical identity APIs required")
        try:
            cls.root = canonical_build_root(str(BUILD_ROOT))
        except Exception as exc:
            raise unittest.SkipTest(f"approved build root unavailable: {exc}")
        cls.fixture = cls.root.path / "zircon-jenkins" / "_physical_path_tests"
        cls.fixture.mkdir(parents=True, exist_ok=True)

    @classmethod
    def tearDownClass(cls):
        shutil.rmtree(cls.fixture, ignore_errors=True)

    def test_exact_drive_roots_only(self):
        self.assertEqual(str(canonical_build_root(str(BUILD_ROOT)).path).casefold(), str(BUILD_ROOT).casefold())
        for value in (r"C:\cargo-targets", r"D:\targets", str(BUILD_ROOT / "nested"), str(Path.cwd() / "target")):
            with self.assertRaises(Exception):
                canonical_build_root(value)

    def test_missing_descendant_is_allowed_only_under_verified_root(self):
        candidate = self.fixture / "new" / "object.bin"
        admitted = physical_path_under(self.root, candidate, allow_missing=True)
        self.assertEqual(admitted, candidate)
        with self.assertRaises(Exception):
            physical_path_under(self.root, self.root.path.parent / "escape", allow_missing=True)

    def test_namespace_rejects_alias_components(self):
        with self.assertRaises(Exception):
            build_namespace(self.root, "..")
        with self.assertRaises(Exception):
            build_namespace(self.root, r"nested\child")

    def test_descendant_junction_is_rejected(self):
        target = self.fixture / "real"
        junction = self.fixture / "junction"
        target.mkdir(exist_ok=True)
        result = subprocess.run(["cmd.exe", "/d", "/c", "mklink", "/J", str(junction), str(target)], capture_output=True, text=True)
        if result.returncode != 0:
            self.skipTest("mklink /J unavailable without elevated Windows privileges")
        try:
            with self.assertRaises(Exception):
                physical_path_under(self.root, junction / "file", allow_missing=True)
        finally:
            subprocess.run(["cmd.exe", "/d", "/c", "rmdir", str(junction)], capture_output=True)


if __name__ == "__main__":
    unittest.main()
