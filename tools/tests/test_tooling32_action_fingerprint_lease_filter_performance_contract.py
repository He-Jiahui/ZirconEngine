from __future__ import annotations

import inspect
import json
import sqlite3
import unittest
from types import SimpleNamespace

from tools.session_coordinator.control_plane.actions.fingerprint import (
    ActionFingerprinter,
)
from tools.session_coordinator.control_plane.actions.models import ActionKind


class ActionFingerprintLeaseFilterPerformanceContractTests(unittest.TestCase):
    @staticmethod
    def _fingerprinter() -> ActionFingerprinter:
        fingerprinter = object.__new__(ActionFingerprinter)
        fingerprinter.supervision = None
        return fingerprinter

    def _connection(self) -> sqlite3.Connection:
        connection = sqlite3.connect(":memory:")
        self.addCleanup(connection.close)
        connection.row_factory = sqlite3.Row
        connection.executescript(
            """
            CREATE TABLE leases(
                path_key TEXT PRIMARY KEY,
                display_path TEXT NOT NULL,
                session_id TEXT NOT NULL,
                expires_at TEXT NOT NULL
            );
            CREATE TABLE patches(
                patch_id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                patch_object_hash TEXT NOT NULL,
                targets_json TEXT NOT NULL,
                base_hashes_json TEXT NOT NULL,
                status TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE sessions(
                session_id TEXT PRIMARY KEY,
                write_scope_json TEXT NOT NULL
            );
            """
        )
        connection.executemany(
            "INSERT INTO leases VALUES (?, ?, ?, ?)",
            (
                ("crates/a.rs", "Crates/A.rs", "session-a", "2026-09-01T00:00:00Z"),
                ("crates/b.rs", "Crates/B.rs", "session-b", "2026-09-01T00:00:00Z"),
                ("crates/c.rs", "Crates/C.rs", "session-c", "2026-09-01T00:00:00Z"),
            ),
        )
        return connection

    def test_patch_and_claim_filter_leases_inside_sql(self) -> None:
        source = inspect.getsource(ActionFingerprinter._action_resources)

        self.assertEqual(source.count("self._leases_for_path_keys("), 2)
        self.assertNotIn(
            '"SELECT display_path, session_id, expires_at FROM leases ORDER BY path_key"',
            source,
        )

    def test_indexed_filter_preserves_selected_lease_order_and_shape(self) -> None:
        connection = self._connection()

        leases = ActionFingerprinter._leases_for_path_keys(
            connection, {"crates/c.rs", "crates/a.rs"}
        )

        self.assertEqual(
            leases,
            [
                {
                    "display_path": "Crates/A.rs",
                    "session_id": "session-a",
                    "expires_at": "2026-09-01T00:00:00Z",
                },
                {
                    "display_path": "Crates/C.rs",
                    "session_id": "session-c",
                    "expires_at": "2026-09-01T00:00:00Z",
                },
            ],
        )
        self.assertEqual(ActionFingerprinter._leases_for_path_keys(connection, set()), [])

    def test_patch_process_projects_only_target_leases(self) -> None:
        connection = self._connection()
        connection.execute(
            "INSERT INTO patches VALUES (?, ?, ?, ?, ?, ?, ?)",
            (
                "patch-a",
                "owner",
                "a" * 64,
                json.dumps(["Crates/C.rs", "Crates/A.rs"]),
                "{}",
                "queued",
                "2026-08-31T00:00:00Z",
            ),
        )

        resources = self._fingerprinter()._action_resources(
            connection,
            SimpleNamespace(kind=ActionKind.PATCH_PROCESS),
            SimpleNamespace(),
            "owner",
        )

        self.assertEqual(
            [row["display_path"] for row in resources["leases"]],
            ["Crates/A.rs", "Crates/C.rs"],
        )

    def test_lease_claim_projects_only_write_scope_leases(self) -> None:
        connection = self._connection()
        connection.execute(
            "INSERT INTO sessions VALUES (?, ?)",
            ("owner", json.dumps(["Crates/B.rs"])),
        )

        resources = self._fingerprinter()._action_resources(
            connection,
            SimpleNamespace(kind=ActionKind.LEASE_CLAIM),
            SimpleNamespace(),
            "owner",
        )

        self.assertEqual(
            [row["display_path"] for row in resources["leases"]],
            ["Crates/B.rs"],
        )


if __name__ == "__main__":
    unittest.main()
