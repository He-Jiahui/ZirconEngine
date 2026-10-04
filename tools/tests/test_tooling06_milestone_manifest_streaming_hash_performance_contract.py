from __future__ import annotations

import hashlib
import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.session_coordinator.workflows.milestones import MilestoneWorkflowService
from tools.session_coordinator.models import CoordinatorError


class Tooling06MilestoneManifestStreamingHashPerformanceContractTests(
    unittest.TestCase
):
    def test_manifest_hash_does_not_materialize_file_contents(self) -> None:
        payload = bytes(range(256)) * 32
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            artifact = root / "artifact.bin"
            artifact.write_bytes(payload)
            expected = hashlib.sha256(
                json.dumps(
                    [
                        {
                            "path": "artifact.bin",
                            "kind": "file",
                            "blob": hashlib.sha256(payload).hexdigest(),
                        }
                    ],
                    sort_keys=True,
                    separators=(",", ":"),
                ).encode("utf-8")
            ).hexdigest()

            with mock.patch.object(
                Path,
                "read_bytes",
                side_effect=AssertionError("milestone file was fully materialized"),
            ):
                actual = MilestoneWorkflowService._manifest_hash_at(
                    root,
                    ("artifact.bin",),
                )

        self.assertEqual(actual, expected)

    def test_manifest_hash_preserves_empty_file_digest(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            root.joinpath("empty.bin").touch()

            first = MilestoneWorkflowService._manifest_hash_at(
                root,
                ("empty.bin",),
            )
            second = MilestoneWorkflowService._manifest_hash_at(
                root,
                ("empty.bin",),
            )

        self.assertEqual(first, second)

    def test_file_kind_uses_one_metadata_probe(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            artifact = root / "artifact.bin"
            artifact.write_bytes(b"artifact")
            probes = self._manifest_probe_count(root, artifact, "artifact.bin")

        self.assertEqual(probes, 1)

    def test_directory_kind_uses_one_metadata_probe_and_preserves_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            directory = root / "artifact-dir"
            directory.mkdir()

            path_type = type(directory)
            original_is_file = path_type.is_file
            original_is_dir = path_type.is_dir
            original_stat = path_type.stat
            probes = 0

            def observed_is_file(path: Path) -> bool:
                nonlocal probes
                probes += 1
                return original_is_file(path)

            def observed_is_dir(path: Path) -> bool:
                nonlocal probes
                probes += 1
                return original_is_dir(path)

            def observed_stat(path: Path, *args: object, **kwargs: object):
                nonlocal probes
                probes += 1
                return original_stat(path, *args, **kwargs)

            with mock.patch.object(
                path_type,
                "is_file",
                new=observed_is_file,
            ), mock.patch.object(
                path_type,
                "is_dir",
                new=observed_is_dir,
            ), mock.patch.object(
                path_type,
                "stat",
                new=observed_stat,
            ):
                with self.assertRaises(CoordinatorError) as raised:
                    MilestoneWorkflowService._manifest_hash_at(
                        root,
                        ("artifact-dir",),
                    )

        self.assertEqual(raised.exception.code, "milestone_manifest_directory")
        self.assertEqual(probes, 1)

    def test_missing_kind_uses_one_metadata_probe_and_preserves_deletion(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            missing = root / "missing.bin"
            probes = self._manifest_probe_count(root, missing, "missing.bin")

        self.assertEqual(probes, 1)

    @staticmethod
    def _manifest_probe_count(root: Path, target: Path, relative: str) -> int:
        path_type = type(target)
        original_is_file = path_type.is_file
        original_is_dir = path_type.is_dir
        original_stat = path_type.stat
        probes = 0

        def observed_is_file(path: Path) -> bool:
            nonlocal probes
            probes += 1
            return original_is_file(path)

        def observed_is_dir(path: Path) -> bool:
            nonlocal probes
            probes += 1
            return original_is_dir(path)

        def observed_stat(path: Path, *args: object, **kwargs: object):
            nonlocal probes
            probes += 1
            return original_stat(path, *args, **kwargs)

        with mock.patch.object(
            path_type,
            "is_file",
            new=observed_is_file,
        ), mock.patch.object(
            path_type,
            "is_dir",
            new=observed_is_dir,
        ), mock.patch.object(
            path_type,
            "stat",
            new=observed_stat,
        ):
            MilestoneWorkflowService._manifest_hash_at(root, (relative,))
        return probes


if __name__ == "__main__":
    unittest.main()
