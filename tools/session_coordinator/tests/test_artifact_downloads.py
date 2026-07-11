from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.control_plane.artifact_downloads import ArtifactDownloadService
from tools.session_coordinator.models import CoordinatorError, utc_text
from tools.session_coordinator.server import CoordinatorApplication
from tools.session_coordinator.tests.helpers import init_repo


class ArtifactDownloadTests(unittest.TestCase):
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

        self.assertEqual(206, response.status)
        self.assertEqual(b"2345", response.body)
        self.assertEqual("bytes 2-5/10", response.headers["Content-Range"])
        self.assertNotIn(str(artifact), str(response.headers))
        self.assertEqual(b"789", suffix.body)
        self.assertEqual(b"789", open_ended.body)
        self.assertEqual(416, invalid.status)
        self.assertEqual("bytes */10", invalid.headers["Content-Range"])

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

if __name__ == "__main__":
    unittest.main()
