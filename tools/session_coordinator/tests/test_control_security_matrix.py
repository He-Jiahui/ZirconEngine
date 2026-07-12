from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.control_plane.artifact_downloads import ArtifactDownloadService
from tools.session_coordinator.control_plane.http_security import (
    validate_browser_read_origin,
    validate_loopback_host,
    validate_loopback_origin,
)
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import CoordinatorError


class ControlSecurityMatrixTests(unittest.TestCase):
    def test_malicious_host_origin_and_referrer_matrix_fails_closed(self) -> None:
        for value in (
            "0.0.0.0:14250",
            "[::1]:14250",
            "127.0.0.1:14250@example.invalid",
            "localhost.:14250",
            "127.0.0.1:14250\r\nX-Evil: yes",
        ):
            with self.subTest(host=value), self.assertRaises(CoordinatorError):
                validate_loopback_host(value, 14250)
        for value in (
            "file:///control",
            "http://127.0.0.1:14250.evil.invalid",
            "http://user@127.0.0.1:14250",
            "http://127.0.0.1:14250/control?token=1",
            "http://127.0.0.1:14250/#fragment",
        ):
            with self.subTest(origin=value), self.assertRaises(CoordinatorError):
                validate_loopback_origin(value, 14250)
        with self.assertRaises(CoordinatorError):
            validate_browser_read_origin(
                None,
                "http://127.0.0.1:14250.evil.invalid/ui/",
                "same-origin",
                14250,
            )

    def test_artifact_identifier_and_path_traversal_matrix_is_indistinguishable(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            database = Database(root / "state.sqlite3")
            migrate(database)
            downloads = ArtifactDownloadService(database, root / "artifacts")
            for value in (
                "../runtime.json",
                "..\\runtime.json",
                "%2e%2e%2fruntime.json",
                "artifact/child",
                "artifact?token=1",
                "<script>alert(1)</script>",
                "a" * 129,
            ):
                with self.subTest(artifact=value), self.assertRaises(CoordinatorError) as rejected:
                    downloads.download(value, None)
                self.assertEqual("artifact_not_found", rejected.exception.code)


if __name__ == "__main__":
    unittest.main()
