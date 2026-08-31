import tempfile
import unittest
import hashlib
from pathlib import Path
from unittest import mock

from tools.session_coordinator.legacy import LegacyMigrationService


class Tooling06LegacyCargoDiagnosticPrefilterPerformanceContractTests(
    unittest.TestCase
):
    def test_only_legacy_named_candidates_require_file_metadata(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            codex = root / ".codex"
            ordinary = codex / "state"
            ordinary.mkdir(parents=True)
            for index in range(2048):
                (ordinary / f"record-{index}.json").write_text("{}", encoding="utf-8")
            candidates = (
                ordinary / "cargo-lease.json",
                ordinary / "target-build-slot.json",
            )
            for path in candidates:
                path.write_text("{}", encoding="utf-8")
            excluded = codex / "sessions" / "cargo-lease.json"
            excluded.parent.mkdir()
            excluded.write_text("{}", encoding="utf-8")

            service = LegacyMigrationService(mock.Mock(), root, mock.Mock())
            metadata_paths: list[Path] = []
            original_is_file = Path.is_file

            def is_file(path: Path) -> bool:
                if path.is_relative_to(codex):
                    metadata_paths.append(path)
                return original_is_file(path)

            with mock.patch.object(Path, "is_file", is_file):
                diagnostics = service.legacy_cargo_diagnostics()

            self.assertEqual(
                (".codex/state/cargo-lease.json", ".codex/state/target-build-slot.json"),
                diagnostics,
            )
            self.assertEqual(set((*candidates, excluded)), set(metadata_paths))
            self.assertEqual(3, len(metadata_paths))

    def test_legacy_note_hashing_does_not_materialize_the_file(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            note = Path(temporary_directory) / "legacy.md"
            note.write_bytes(b"legacy note")
            expected = hashlib.sha256(b"legacy note").hexdigest()

            with mock.patch.object(
                Path,
                "read_bytes",
                side_effect=AssertionError("legacy note hashing must stream"),
            ):
                actual = LegacyMigrationService._hash(note)

            self.assertEqual(expected, actual)


if __name__ == "__main__":
    unittest.main()
