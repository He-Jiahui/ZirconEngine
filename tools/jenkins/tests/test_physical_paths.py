import os
import shutil
import subprocess
import unittest
from unittest.mock import patch
from pathlib import Path

from tools.jenkins.resources.paths import canonical_build_root, build_namespace, physical_path_under

REPO_ROOT = Path(__file__).resolve().parents[3]
BUILD_ROOT = Path(r"E:\cargo-targets")


class PhysicalPathTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        if os.name != "nt":
            raise unittest.SkipTest("Windows physical identity APIs required")
        try:
            cls.root = canonical_build_root(str(BUILD_ROOT))
        except Exception as exc:
            raise unittest.SkipTest(f"approved build root unavailable: {exc}")
        cls.fixture = cls.root.path / "zircon-local" / "evidence" / "_physical_path_tests"
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


class PhysicalAdmissionRegressions(unittest.TestCase):
    def test_repository_build_root_is_rejected_before_native_access(self):
        from tools.jenkins.resources import paths
        with patch.object(paths, "_drive_identity", side_effect=AssertionError("native access")):
            with self.assertRaises(Exception) as caught:
                paths.canonical_build_root(REPO_ROOT / ".jenkins" / "builds")
        self.assertNotIsInstance(caught.exception, AssertionError)

    def test_windows_alias_names_rejected_before_native_access(self):
        from tools.jenkins.resources import paths
        root = paths.ApprovedBuildRoot(Path(r"E:\cargo-targets"))
        for suffix in ("target.", "target ", "NUL.bin", "target:stream", "a\\.\\b", "a\\\\b"):
            with self.subTest(suffix=suffix), patch.object(paths, "_attrs", side_effect=AssertionError("native access")):
                with self.assertRaises(Exception) as caught:
                    paths.physical_path_under(root, str(root.path) + "\\" + suffix)
                self.assertNotIsInstance(caught.exception, AssertionError)

    def test_reparse_parent_rejected_before_descendant_or_final_resolution(self):
        from tools.jenkins.resources import paths
        root = paths.ApprovedBuildRoot(Path(r"E:\cargo-targets"))
        seen = []
        def attributes(path):
            seen.append(path)
            if path == r"E:\cargo-targets\alias":
                return paths._REPARSE
            if path.endswith("child"):
                raise AssertionError("followed junction")
            return 0
        with patch.object(paths, "_attrs", side_effect=attributes), patch.object(paths, "_final_path", side_effect=AssertionError("resolved junction")):
            with self.assertRaises(Exception) as caught:
                paths.physical_path_under(root, r"E:\cargo-targets\alias\child")
        self.assertNotIsInstance(caught.exception, AssertionError)
        self.assertNotIn(r"E:\cargo-targets\alias\child", seen)

    def test_forged_root_cannot_expand_admission(self):
        from tools.jenkins.resources import paths
        root = paths.ApprovedBuildRoot(REPO_ROOT / ".jenkins" / "builds")
        with patch.object(paths, "_attrs", side_effect=AssertionError("native access")):
            with self.assertRaises(Exception) as caught:
                paths.physical_path_under(root, root.path / "target")
        self.assertNotIsInstance(caught.exception, AssertionError)

    def test_subst_and_duplicate_volume_aliases_are_rejected(self):
        from tools.jenkins.resources import paths
        from tools.jenkins.pilot.native import paths as native
        for mapping in ({"D:": r"\??\E:\somewhere"},
                        {"D:": r"\Device\HarddiskVolume3", "C:": r"\Device\HarddiskVolume3"}):
            with self.subTest(mapping=mapping), patch.object(native, "_query_dos_device", side_effect=lambda drive, **kw: mapping.get(drive)):
                with self.assertRaises(Exception):
                    paths.canonical_build_root(r"D:\cargo-targets")


if __name__ == "__main__":
    unittest.main()
