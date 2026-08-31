import tempfile
import unittest
import hashlib
from pathlib import Path
from unittest import mock

from tools.session_coordinator.failures import failure_artifact_snapshot


class Tooling06FailureArtifactSnapshotPrefilterPerformanceContractTests(
    unittest.TestCase
):
    def test_only_failure_named_markdown_requires_file_metadata(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            plans = root / "docs" / "plans" / "runtime"
            plans.mkdir(parents=True)
            for index in range(2048):
                (plans / f"plan-{index}.md").write_text("# Plan\n", encoding="utf-8")
            failures = (
                plans / "failure-runtime.md",
                plans / "2026-09-01-runtime-fixed-handoff.md",
            )
            for path in failures:
                path.write_text("# Failure\n", encoding="utf-8")

            metadata_paths: list[Path] = []
            original_is_file = Path.is_file

            def is_file(path: Path) -> bool:
                if path.suffix == ".md" and path.parent == plans:
                    metadata_paths.append(path)
                return original_is_file(path)

            with mock.patch.object(Path, "is_file", is_file):
                snapshot = failure_artifact_snapshot(root)

            self.assertEqual(2, len(snapshot))
            self.assertEqual(set(failures), set(metadata_paths))
            self.assertEqual(2, len(metadata_paths))

    def test_failure_hashing_does_not_materialize_the_artifact(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            artifact = root / "docs" / "plans" / "failure-runtime.md"
            artifact.parent.mkdir(parents=True)
            artifact.write_bytes(b"failure evidence")
            expected = hashlib.sha256(b"failure evidence").hexdigest()

            with mock.patch.object(
                Path,
                "read_bytes",
                side_effect=AssertionError("failure snapshot must stream hashes"),
            ):
                snapshot = failure_artifact_snapshot(root)

            self.assertEqual(expected, snapshot[0]["hash"])


if __name__ == "__main__":
    unittest.main()
