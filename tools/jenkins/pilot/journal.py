"""Submission reconciliation, without a second task scheduler or execution queue."""

from __future__ import annotations

import json
import sqlite3
from contextlib import contextmanager
from pathlib import Path
from typing import Mapping

from .contracts import PilotError, RequestIdentity, canonical_json, identifier, sha256_identity
from .storage import ManagedStorage, require_managed_root

_STATES = frozenset({"reserved", "uncertain", "queued", "running", "completed", "cancelled"})


class SubmissionJournal:
    def __init__(self, path: Path) -> None:
        self.path = require_managed_root(path, allow_root=False)
        with self._connect() as connection:
            connection.execute(
                """CREATE TABLE IF NOT EXISTS submissions (
                   session_id TEXT NOT NULL, request_id TEXT NOT NULL,
                   identity_json TEXT NOT NULL, bundle_hash TEXT NOT NULL,
                   job TEXT NOT NULL, state TEXT NOT NULL,
                   queue_id INTEGER, build_number INTEGER, result TEXT,
                   PRIMARY KEY(session_id, request_id))"""
            )

    @contextmanager
    def _connect(self):
        storage = ManagedStorage(self.path.parent)
        with storage.backend() as backend:
            # No WAL migration or cleanup is authorized. These leaves must
            # not become an alternate SQLite write/recovery path.
            for suffix in ("-wal", "-shm"):
                if backend.exists(self.path.name + suffix):
                    raise PilotError("private submission journal has unsupported WAL sidecars; preserve and reconcile them")
            with backend.open_lock_file(self.path.name) as database, \
                 backend.open_lock_file(self.path.name + "-journal") as rollback:
                database.seek(0)
                header = database.read(100)
                if header and (len(header) < 100 or header[:16] != b"SQLite format 3\0"):
                    raise PilotError("private submission journal is not a plain supported SQLite database")
                if header and (header[18] != 1 or header[19] != 1):
                    raise PilotError("private submission journal uses unsupported WAL format; preserve its history")
                rollback.seek(0)
                journal_magic = rollback.read(8)
                if journal_magic and journal_magic not in {b"\0" * 8, b"\xd9\xd5\x05\xf9\x20\xa1\x63\xd7"}:
                    raise PilotError("private rollback sidecar is not a supported SQLite journal; preserve its data")
                connection = sqlite3.connect(self.path.as_uri() + "?mode=rw", uri=True, timeout=10)
                connection.row_factory = sqlite3.Row
                try:
                    # PERSIST commits clear the journal header instead of
                    # unlinking our pinned sidecar. SQLite's own locks still
                    # serialize concurrent BEGIN IMMEDIATE reservations.
                    mode = connection.execute("PRAGMA journal_mode=PERSIST").fetchone()[0]
                    if mode.lower() != "persist":
                        raise PilotError("private submission journal cannot retain its pinned rollback sidecar")
                    connection.execute("PRAGMA temp_store=MEMORY")
                    with connection:
                        yield connection
                finally:
                    connection.close()

    @staticmethod
    def _row(row: sqlite3.Row) -> dict[str, object]:
        result = dict(row)
        result["identity"] = json.loads(result.pop("identity_json"))
        return result

    def reserve(
        self, identity: RequestIdentity, bundle_hash: str, job: str
    ) -> tuple[dict[str, object], bool]:
        sha256_identity(bundle_hash, "bundle_hash")
        identifier(job, "job")
        encoded = canonical_json(identity.to_dict()).decode("ascii")
        with self._connect() as connection:
            connection.execute("BEGIN IMMEDIATE")
            row = connection.execute(
                "SELECT * FROM submissions WHERE session_id=? AND request_id=?",
                (identity.session_id, identity.request_id),
            ).fetchone()
            if row is not None:
                if (row["identity_json"], row["bundle_hash"], row["job"]) != (
                    encoded, bundle_hash, job
                ):
                    raise PilotError("request ID is already bound to different immutable inputs")
                return self._row(row), False
            connection.execute(
                """INSERT INTO submissions
                   (session_id,request_id,identity_json,bundle_hash,job,state)
                   VALUES(?,?,?,?,?,'reserved')""",
                (identity.session_id, identity.request_id, encoded, bundle_hash, job),
            )
        return self.get(identity), True

    def get(self, identity: RequestIdentity) -> dict[str, object]:
        with self._connect() as connection:
            row = connection.execute(
                "SELECT * FROM submissions WHERE session_id=? AND request_id=?",
                (identity.session_id, identity.request_id),
            ).fetchone()
        if row is None:
            raise PilotError("request has not been submitted")
        result = self._row(row)
        if result["identity"] != identity.to_dict():
            raise PilotError("request identity differs from its journal binding")
        return result

    def update(
        self, identity: RequestIdentity, state: str, *, queue_id: int | None = None,
        build_number: int | None = None, result: str | None = None,
    ) -> Mapping[str, object]:
        if state not in _STATES:
            raise PilotError("submission state is unsupported")
        for value in (queue_id, build_number):
            if value is not None and (type(value) is not int or value < 1):
                raise PilotError("Jenkins queue/build identity must be a positive integer")
        with self._connect() as connection:
            connection.execute("BEGIN IMMEDIATE")
            row = connection.execute(
                "SELECT * FROM submissions WHERE session_id=? AND request_id=?",
                (identity.session_id, identity.request_id),
            ).fetchone()
            if row is None or json.loads(row["identity_json"]) != identity.to_dict():
                raise PilotError("submission update has no matching immutable request")
            if row["state"] in {"completed", "cancelled"} and row["state"] != state:
                raise PilotError("terminal submission cannot return to an active state")
            if row["build_number"] is not None and build_number not in {None, row["build_number"]}:
                raise PilotError("request cannot be rebound to a different Jenkins build")
            if row["queue_id"] is not None and queue_id not in {None, row["queue_id"]}:
                raise PilotError("request cannot be rebound to a different Jenkins queue entry")
            if row["state"] in {"completed", "cancelled"} and row["result"] is not None and result not in {None, row["result"]}:
                raise PilotError("terminal Jenkins result cannot be rewritten")
            connection.execute(
                """UPDATE submissions SET state=?, queue_id=COALESCE(?,queue_id),
                   build_number=COALESCE(?,build_number), result=COALESCE(?,result)
                   WHERE session_id=? AND request_id=?""",
                (state, queue_id, build_number, result, identity.session_id, identity.request_id),
            )
        return self.get(identity)
