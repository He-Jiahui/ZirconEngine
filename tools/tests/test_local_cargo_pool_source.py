"""Compiler-pool admission tests; real Cargo execution is a separate probe."""
from __future__ import annotations

from concurrent.futures import ThreadPoolExecutor
import json
import os
from pathlib import Path
import tempfile
import threading
import unittest

from tools.dev.local_cargo_pool_source import RECORD_NAME, bind_compiler_pool


class LocalCargoPoolSourceTests(unittest.TestCase):
    def setUp(self):
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        self.root = Path(directory.name).resolve()
        self.a = self.root / "checkout-a"
        self.b = self.root / "checkout-b"
        self.target = self.root / "pool/target"
        self.build = self.root / "pool/build"
        self.home = self.root / "pool/cargo-home"
        self.marker = self.build.parent / RECORD_NAME
        for path in (self.a, self.b, self.target, self.build, self.home):
            path.mkdir(parents=True)

    def test_same_source_reuses_output_and_keeps_cargo_home(self):
        registry = self.home / "registry-index-control"
        registry.write_bytes(b"registry cache")
        bind_compiler_pool(self.a, self.target, self.build)
        before = self.marker.read_bytes()
        self.assertEqual("zircon_cargo_pool_source_v1", json.loads(before)["schema"])
        artifact = self.build / "compiled-test"
        artifact.write_bytes(b"compiled from A")

        bind_compiler_pool(self.a, self.target, self.build)

        self.assertEqual(before, self.marker.read_bytes())
        self.assertEqual(b"compiled from A", artifact.read_bytes())
        self.assertEqual(b"registry cache", registry.read_bytes())

    def test_other_source_is_refused_even_if_no_build_succeeded(self):
        bind_compiler_pool(self.a, self.target, self.build)
        before = self.marker.read_bytes()
        with self.assertRaisesRegex(ValueError, "source or output directories do not match"):
            bind_compiler_pool(self.b, self.target, self.build)
        self.assertEqual(before, self.marker.read_bytes())
        self.assertEqual([], list(self.build.iterdir()))
        bind_compiler_pool(self.a, self.target, self.build)

    def test_unbound_targets_and_build_roots_are_never_adopted(self):
        for name in ("target", "build"):
            with self.subTest(directory=name):
                target, build = self.root / name / "target", self.root / name / "build"
                target.mkdir(parents=True)
                build.mkdir()
                output = (target if name == "target" else build) / "foreign-output"
                output.write_bytes(b"unknown provenance")

                with self.assertRaisesRegex(ValueError, "not empty"):
                    bind_compiler_pool(self.a, target, build)

                self.assertEqual(b"unknown provenance", output.read_bytes())
                self.assertFalse((build.parent / RECORD_NAME).exists())

    def test_another_target_name_does_not_release_the_shared_build_root(self):
        bind_compiler_pool(self.a, self.target, self.build)
        alternate = self.target.with_name("target-other")
        alternate.mkdir()
        with self.assertRaisesRegex(ValueError, "source or output directories do not match"):
            bind_compiler_pool(self.a, alternate, self.build)

    def test_partial_record_is_not_replaced(self):
        self.marker.touch()
        with self.assertRaisesRegex(ValueError, "Incomplete or invalid"):
            bind_compiler_pool(self.a, self.target, self.build)
        self.assertEqual(b"", self.marker.read_bytes())

    def test_linked_record_is_not_an_ownership_fact(self):
        bind_compiler_pool(self.a, self.target, self.build)
        alias = self.root / "other-record-link"
        os.link(self.marker, alias)
        with self.assertRaisesRegex(ValueError, "Incomplete or invalid"):
            bind_compiler_pool(self.a, self.target, self.build)
        self.assertEqual(self.marker.read_bytes(), alias.read_bytes())

    def test_two_simultaneous_first_claims_never_admit_both_sources(self):
        start = threading.Barrier(2)

        def attempt(repo):
            start.wait(timeout=5)
            try:
                bind_compiler_pool(repo, self.target, self.build)
            except ValueError:
                return None
            return repo

        with ThreadPoolExecutor(max_workers=2) as workers:
            futures = [workers.submit(attempt, path) for path in (self.a, self.b)]
            admitted = [path for future in futures if (path := future.result(timeout=10))]

        self.assertEqual(1, len(admitted), admitted)
        self.assertEqual(os.path.normcase(str(admitted[0])), json.loads(self.marker.read_bytes())["source"]["path"])
        bind_compiler_pool(admitted[0], self.target, self.build)


if __name__ == "__main__":
    unittest.main()
