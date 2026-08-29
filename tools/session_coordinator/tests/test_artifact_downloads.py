from __future__ import annotations

import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.control_plane.artifact_downloads import ArtifactDownloadService
from tools.session_coordinator.models import CoordinatorError, utc_text
from tools.session_coordinator.server import CoordinatorApplication
from tools.session_coordinator.tests.helpers import init_repo


class ArtifactDownloadTests(unittest.TestCase):
    def test_extremely_long_range_numbers_are_typed(self) -> None:
        for header in (
            f"bytes={'9' * 5000}-",
            f"bytes=-{'9' * 5000}",
            f"bytes=0-{'9' * 5000}",
        ):
            with self.subTest(header=header[:32]):
                with self.assertRaises(CoordinatorError) as rejected:
                    ArtifactDownloadService._range(10, header)

                self.assertEqual("invalid_range", rejected.exception.code)

    def test_non_ascii_range_digits_are_typed(self) -> None:
        with self.assertRaises(CoordinatorError) as rejected:
            ArtifactDownloadService._range(10, "bytes=２-３")

        self.assertEqual("invalid_range", rejected.exception.code)

    def test_download_does_not_reopen_path_after_metadata_validation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state")
            app = CoordinatorApplication(config)
            app.sessions.register(session_id="session-a")
            with app.database.connect() as connection:
                run_id = connection.execute("SELECT run_id FROM workflow_runs").fetchone()[0]
            config.workflow_artifact_root.mkdir(parents=True)
            artifact = config.workflow_artifact_root / "report.txt"
            artifact.write_bytes(b"safe")
            replacement = root / "replacement.txt"
            replacement.write_bytes(b"evil")
            with app.database.transaction() as connection:
                connection.execute(
                    """INSERT INTO workflow_artifacts(
                        artifact_id, run_id, artifact_kind, display_name, storage_path,
                        byte_count, metadata_json, created_at
                    ) VALUES (?, ?, 'report', ?, ?, ?, ?, ?)""",
                    (
                        "opaque-race",
                        run_id,
                        "report.txt",
                        str(artifact),
                        4,
                        json.dumps({}),
                        utc_text(),
                    ),
                )
            service = ArtifactDownloadService(app.database, config.workflow_artifact_root)
            original_stat = Path.stat
            replaced = False

            def replace_after_metadata(path: Path, *args, **kwargs):
                nonlocal replaced
                metadata = original_stat(path, *args, **kwargs)
                if path == artifact and not replaced:
                    replaced = True
                    replacement.replace(artifact)
                return metadata

            with mock.patch.object(Path, "stat", replace_after_metadata):
                response = service.download("opaque-race", None)

        self.assertFalse(replaced)
        self.assertEqual(b"safe", response.body)

    def test_download_is_opaque_bounded_and_range_capable(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state")
            app = CoordinatorApplication(config)
            app.sessions.register(session_id="session-a")
            with app.database.connect() as connection:
                run_id = connection.execute("SELECT run_id FROM workflow_runs").fetchone()[0]
            config.workflow_artifact_root.mkdir(parents=True)
            artifact = config.workflow_artifact_root / "report.txt"
            artifact.write_bytes(b"0123456789")
            with app.database.transaction() as connection:
                connection.execute(
                    """INSERT INTO workflow_artifacts(
                        artifact_id, run_id, artifact_kind, display_name, storage_path,
                        byte_count, metadata_json, created_at
                    ) VALUES (?, ?, 'report', ?, ?, ?, ?, ?)""",
                    ("opaque-a", run_id, "report.txt", str(artifact), 10, json.dumps({}), utc_text()),
                )
            service = ArtifactDownloadService(app.database, config.workflow_artifact_root)

            response = service.download("opaque-a", "bytes=2-5")
            suffix = service.download("opaque-a", "bytes=-3")
            open_ended = service.download("opaque-a", "bytes=7-")
            invalid = service.download("opaque-a", "bytes=99-100")
            oversized_numeric = service.download("opaque-a", f"bytes={'9' * 5000}-")

        self.assertEqual(206, response.status)
        self.assertEqual(b"2345", response.body)
        self.assertEqual("bytes 2-5/10", response.headers["Content-Range"])
        self.assertNotIn(str(artifact), str(response.headers))
        self.assertEqual(b"789", suffix.body)
        self.assertEqual(b"789", open_ended.body)
        self.assertEqual(416, invalid.status)
        self.assertEqual("bytes */10", invalid.headers["Content-Range"])
        self.assertEqual(416, oversized_numeric.status)
        self.assertEqual("bytes */10", oversized_numeric.headers["Content-Range"])

    def test_empty_artifact_download_is_valid_but_range_is_unsatisfiable(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state")
            app = CoordinatorApplication(config)
            app.sessions.register(session_id="session-a")
            with app.database.connect() as connection:
                run_id = connection.execute("SELECT run_id FROM workflow_runs").fetchone()[0]
            config.workflow_artifact_root.mkdir(parents=True)
            artifact = config.workflow_artifact_root / "empty.txt"
            artifact.write_bytes(b"")
            with app.database.transaction() as connection:
                connection.execute(
                    """INSERT INTO workflow_artifacts(
                        artifact_id, run_id, artifact_kind, display_name, storage_path,
                        byte_count, metadata_json, created_at
                    ) VALUES (?, ?, 'report', 'empty.txt', ?, 0, '{}', ?)""",
                    ("opaque-empty", run_id, str(artifact), utc_text()),
                )
            service = ArtifactDownloadService(app.database, config.workflow_artifact_root)

            response = service.download("opaque-empty", None)
            invalid = service.download("opaque-empty", "bytes=0-0")

        self.assertEqual(200, response.status)
        self.assertEqual(b"", response.body)
        self.assertEqual(416, invalid.status)
        self.assertEqual("bytes */0", invalid.headers["Content-Range"])

    def test_metadata_cannot_escape_artifact_root(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state")
            app = CoordinatorApplication(config)
            app.sessions.register(session_id="session-a")
            outside = root / "secret.txt"
            outside.write_text("secret", encoding="utf-8")
            with app.database.connect() as connection:
                run_id = connection.execute("SELECT run_id FROM workflow_runs").fetchone()[0]
            with app.database.transaction() as connection:
                connection.execute(
                    """INSERT INTO workflow_artifacts(
                        artifact_id, run_id, artifact_kind, display_name, storage_path,
                        byte_count, metadata_json, created_at
                    ) VALUES (?, ?, 'report', 'secret', ?, 6, '{}', ?)""",
                    ("opaque-b", run_id, str(outside), utc_text()),
                )
            service = ArtifactDownloadService(app.database, config.workflow_artifact_root)
            with self.assertRaises(CoordinatorError):
                service.download("opaque-b", None)

    def test_durable_byte_count_must_match_the_open_handle(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state")
            app = CoordinatorApplication(config)
            app.sessions.register(session_id="session-a")
            with app.database.connect() as connection:
                run_id = connection.execute("SELECT run_id FROM workflow_runs").fetchone()[0]
            config.workflow_artifact_root.mkdir(parents=True)
            artifact = config.workflow_artifact_root / "report.txt"
            artifact.write_bytes(b"safe")
            with app.database.transaction() as connection:
                connection.execute(
                    """INSERT INTO workflow_artifacts(
                        artifact_id, run_id, artifact_kind, display_name, storage_path,
                        byte_count, metadata_json, created_at
                    ) VALUES (?, ?, 'report', 'report.txt', ?, 99, '{}', ?)""",
                    ("opaque-size", run_id, str(artifact), utc_text()),
                )
            service = ArtifactDownloadService(app.database, config.workflow_artifact_root)

            with self.assertRaises(CoordinatorError) as rejected:
                service.download("opaque-size", None)

        self.assertEqual("artifact_not_found", rejected.exception.code)

    def test_multiply_linked_artifact_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state")
            app = CoordinatorApplication(config)
            app.sessions.register(session_id="session-a")
            with app.database.connect() as connection:
                run_id = connection.execute("SELECT run_id FROM workflow_runs").fetchone()[0]
            config.workflow_artifact_root.mkdir(parents=True)
            outside = root / "secret.txt"
            outside.write_bytes(b"secret")
            artifact = config.workflow_artifact_root / "report.txt"
            try:
                os.link(outside, artifact)
            except OSError as error:
                self.skipTest(f"hardlinks are unavailable: {error}")
            with app.database.transaction() as connection:
                connection.execute(
                    """INSERT INTO workflow_artifacts(
                        artifact_id, run_id, artifact_kind, display_name, storage_path,
                        byte_count, metadata_json, created_at
                    ) VALUES (?, ?, 'report', 'report.txt', ?, 6, '{}', ?)""",
                    ("opaque-hardlink", run_id, str(artifact), utc_text()),
                )
            service = ArtifactDownloadService(app.database, config.workflow_artifact_root)

            with self.assertRaises(CoordinatorError) as rejected:
                service.download("opaque-hardlink", None)

        self.assertEqual("artifact_not_found", rejected.exception.code)

if __name__ == "__main__":
    unittest.main()
