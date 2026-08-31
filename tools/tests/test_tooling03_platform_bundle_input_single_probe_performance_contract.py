from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.platform_bundle_materialize import (
    platform_bundle_file_input_diagnostic,
)


class Tooling03PlatformBundleInputSingleProbePerformanceContractTests(
    unittest.TestCase
):
    def test_valid_file_uses_one_metadata_probe(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            path = Path(temp_dir) / "assets.zrpack"
            path.write_bytes(b"pack")
            original_stat = Path.stat
            probes = 0

            def observed_stat(candidate: Path, *args: object, **kwargs: object):
                nonlocal probes
                if candidate == path:
                    probes += 1
                return original_stat(candidate, *args, **kwargs)

            with mock.patch.object(Path, "stat", new=observed_stat):
                diagnostic = platform_bundle_file_input_diagnostic(
                    "pack file",
                    path,
                )

        self.assertIsNone(diagnostic)
        self.assertEqual(probes, 1)

    def test_missing_file_preserves_diagnostic(self) -> None:
        path = Path("missing-assets.zrpack")

        self.assertEqual(
            platform_bundle_file_input_diagnostic("pack file", path),
            f"pack file {path} does not exist",
        )

    def test_directory_preserves_not_a_file_diagnostic(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            path = Path(temp_dir)

            diagnostic = platform_bundle_file_input_diagnostic("pack file", path)

        self.assertEqual(diagnostic, f"pack file {path} is not a file")

    def test_empty_file_preserves_diagnostic(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            path = Path(temp_dir) / "empty.zrpack"
            path.touch()

            diagnostic = platform_bundle_file_input_diagnostic("pack file", path)

        self.assertEqual(diagnostic, f"pack file {path} is empty")

    def test_inspection_error_preserves_diagnostic(self) -> None:
        path = Path("unreadable-assets.zrpack")
        with mock.patch.object(
            Path,
            "stat",
            side_effect=PermissionError("simulated permission failure"),
        ):
            diagnostic = platform_bundle_file_input_diagnostic("pack file", path)

        self.assertEqual(
            diagnostic,
            f"pack file {path} could not be inspected: simulated permission failure",
        )


if __name__ == "__main__":
    unittest.main()
