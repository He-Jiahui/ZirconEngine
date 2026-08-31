from __future__ import annotations

import hashlib
import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.session_coordinator.workflows.milestones import MilestoneWorkflowService


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


if __name__ == "__main__":
    unittest.main()
