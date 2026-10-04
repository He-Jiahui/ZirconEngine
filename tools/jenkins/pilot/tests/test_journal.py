from __future__ import annotations

import concurrent.futures
import os
import sqlite3
import unittest
from pathlib import Path

from tools.jenkins.pilot.contracts import PilotError, RequestIdentity
from tools.jenkins.pilot.journal import SubmissionJournal
from tools.jenkins.pilot.tests.test_native_process import native_directory


@unittest.skipUnless(os.name == "nt", "requires native physical journal storage")
class SubmissionJournalTests(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = native_directory()
        self.root = self.directory.__enter__()
        self.addCleanup(self.directory.__exit__, None, None, None)
        self.journal = SubmissionJournal(self.root / "submissions.sqlite3")
        self.identity = RequestIdentity("session", "request", "attempt", 1, "a" * 64, "python-static")

    def test_concurrent_identical_submissions_have_one_dispatch_owner(self) -> None:
        def reserve(_: int) -> bool:
            return self.journal.reserve(self.identity, "b" * 64, "pilot")[1]
        with concurrent.futures.ThreadPoolExecutor(max_workers=8) as executor:
            owners = list(executor.map(reserve, range(16)))
        self.assertEqual(owners.count(True), 1)

    def test_request_cannot_change_bundle_job_or_generation(self) -> None:
        self.journal.reserve(self.identity, "b" * 64, "pilot")
        altered = RequestIdentity("session", "request", "attempt", 2, "a" * 64, "python-static")
        for identity, bundle, job in [
            (self.identity, "c" * 64, "pilot"), (self.identity, "b" * 64, "other"),
            (altered, "b" * 64, "pilot"),
        ]:
            with self.subTest(job=job, generation=identity.generation):
                with self.assertRaises(PilotError):
                    self.journal.reserve(identity, bundle, job)

    def test_uncertain_submission_survives_restart_without_new_dispatch(self) -> None:
        self.journal.reserve(self.identity, "b" * 64, "pilot")
        self.journal.update(self.identity, "uncertain")
        reopened = SubmissionJournal(self.journal.path)
        record, dispatch = reopened.reserve(self.identity, "b" * 64, "pilot")
        self.assertFalse(dispatch)
        self.assertEqual(record["state"], "uncertain")

    def test_terminal_and_exact_build_binding_cannot_be_overwritten(self) -> None:
        self.journal.reserve(self.identity, "b" * 64, "pilot")
        self.journal.update(self.identity, "running", build_number=3)
        with self.assertRaises(PilotError):
            self.journal.update(self.identity, "running", build_number=4)
        self.journal.update(self.identity, "completed", result="FAILURE")
        with self.assertRaises(PilotError):
            self.journal.update(self.identity, "running")

    def test_boolean_generation_is_rejected(self) -> None:
        with self.assertRaises(PilotError):
            RequestIdentity("session", "request", "attempt", True, "a" * 64, "python-static")

    def test_live_connection_pins_database_and_parent_against_rename(self):
        with self.journal._connect() as connection:
            with self.assertRaises(OSError):
                os.rename(self.journal.path, self.root / "renamed.sqlite3")
            with self.assertRaises(OSError):
                os.rename(self.root, self.root.with_name(self.root.name + "-renamed"))
            self.assertEqual(connection.execute("PRAGMA journal_mode").fetchone()[0], "persist")
            self.assertEqual(connection.execute("PRAGMA temp_store").fetchone()[0], 2)

    def test_hardlinked_database_and_sidecar_preserve_foreign_file(self):
        foreign = self.root / "foreign-data.txt"
        foreign.write_bytes(b"foreign data must survive")
        for name in ("linked.sqlite3", "linked-sidecar.sqlite3-journal"):
            with self.subTest(name=name):
                os.link(foreign, self.root / name)
                database = self.root / ("linked-sidecar.sqlite3" if name.endswith("-journal") else name)
                with self.assertRaises((ValueError, OSError)):
                    SubmissionJournal(database)
                self.assertEqual(foreign.read_bytes(), b"foreign data must survive")
                self.assertEqual((self.root / name).read_bytes(), b"foreign data must survive")

    def test_junction_parent_rejected_without_mutating_foreign_database(self):
        import subprocess
        foreign = self.root / "foreign"
        foreign.mkdir()
        database = foreign / "data.sqlite3"
        database.write_bytes(b"foreign database bytes")
        alias = self.root / "junction"
        subprocess.run(["cmd.exe", "/d", "/c", "mklink", "/J", str(alias), str(foreign)],
                       check=True, capture_output=True)
        with self.assertRaises((ValueError, OSError)):
            SubmissionJournal(alias / "data.sqlite3")
        self.assertEqual(database.read_bytes(), b"foreign database bytes")

    def test_existing_wal_database_is_rejected_without_migration(self):
        path = self.root / "unsupported.sqlite3"
        with sqlite3.connect(path) as connection:
            self.assertEqual(connection.execute("PRAGMA journal_mode=WAL").fetchone()[0], "wal")
            connection.execute("CREATE TABLE historical(value TEXT)")
            connection.execute("INSERT INTO historical VALUES ('preserved')")
        before = path.read_bytes()
        with self.assertRaises(PilotError):
            SubmissionJournal(path)
        self.assertEqual(path.read_bytes(), before)

    def test_existing_wal_sidecar_is_preserved(self):
        path = self.root / "sidecar.sqlite3"
        sidecar = self.root / "sidecar.sqlite3-wal"
        sidecar.write_bytes(b"unreconciled historical WAL")
        with self.assertRaises(PilotError):
            SubmissionJournal(path)
        self.assertEqual(sidecar.read_bytes(), b"unreconciled historical WAL")
        self.assertFalse(path.exists())

    def test_foreign_plain_rollback_sidecar_is_never_overwritten(self):
        path = self.root / "foreign-sidecar.sqlite3"
        sidecar = self.root / "foreign-sidecar.sqlite3-journal"
        sidecar.write_bytes(b"plain foreign history")
        with self.assertRaises(PilotError):
            SubmissionJournal(path)
        self.assertEqual(sidecar.read_bytes(), b"plain foreign history")
        self.assertEqual(path.read_bytes(), b"")


if __name__ == "__main__":
    unittest.main()
