from __future__ import annotations

import os
import unittest
from pathlib import Path
from unittest import mock

from tools import zircon_build_cargo_environment as cargo_environment


class Tooling08CargoApprovedRootPerformanceContractTests(unittest.TestCase):
    @unittest.skipUnless(os.name == "nt", "Windows build roots are Windows-only")
    def test_repeated_checks_resolve_approved_roots_once(self) -> None:
        resolve_counts: dict[str, int] = {}
        original_resolve = Path.resolve

        def counted_resolve(path: Path, *args, **kwargs):
            key = str(path)
            resolve_counts[key] = resolve_counts.get(key, 0) + 1
            return original_resolve(path, *args, **kwargs)

        cached_roots = getattr(
            cargo_environment,
            "_resolved_approved_windows_build_roots",
            None,
        )
        if cached_roots is not None:
            cached_roots.cache_clear()
        with mock.patch.object(Path, "resolve", counted_resolve):
            cargo_environment.assert_managed_windows_build_root(
                Path(r"E:\ZirconBuilds\performance-a")
            )
            cargo_environment.assert_managed_windows_build_root(
                Path(r"E:\ZirconBuilds\performance-b")
            )

        approved_resolves = sum(
            resolve_counts.get(root, 0)
            for root in cargo_environment.APPROVED_WINDOWS_BUILD_ROOTS
        )
        approved_root_count = len(cargo_environment.APPROVED_WINDOWS_BUILD_ROOTS)
        self.assertEqual(approved_root_count, approved_resolves)
        self.assertEqual(1, resolve_counts[r"E:\ZirconBuilds\performance-a"])
        self.assertEqual(1, resolve_counts[r"E:\ZirconBuilds\performance-b"])

    def test_approved_root_cache_is_narrow_and_clearable(self) -> None:
        cached_roots = cargo_environment._resolved_approved_windows_build_roots

        cached_roots.cache_clear()
        parameters = cached_roots.cache_parameters()

        self.assertEqual(1, parameters["maxsize"])
        self.assertFalse(parameters["typed"])


if __name__ == "__main__":
    unittest.main()
