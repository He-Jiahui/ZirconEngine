"""One transactional authority for requests, operations and domain records."""

from __future__ import annotations

import json
import sqlite3
import time
from contextlib import contextmanager
from pathlib import Path
from typing import Iterator

from ..contracts import JenkinsError, canonical_json, digest, identifier

SCHEMA = """
CREATE TABLE IF NOT EXISTS metadata (key TEXT PRIMARY KEY,value TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS records (
 domain TEXT NOT NULL,key TEXT NOT NULL,payload TEXT NOT NULL,version INTEGER NOT NULL,
 updated_at REAL NOT NULL,PRIMARY KEY(domain,key));
CREATE TABLE IF NOT EXISTS requests (
 repository_id TEXT NOT NULL,session_id TEXT NOT NULL,request_id TEXT NOT NULL,
 payload_digest TEXT NOT NULL,payload TEXT NOT NULL,generation INTEGER NOT NULL,
 status TEXT NOT NULL,created_at REAL NOT NULL,updated_at REAL NOT NULL,
 PRIMARY KEY(repository_id,session_id,request_id));
CREATE TABLE IF NOT EXISTS operations (
 operation_id TEXT PRIMARY KEY,kind TEXT NOT NULL,payload_digest TEXT NOT NULL,
 payload TEXT NOT NULL,status TEXT NOT NULL,result TEXT,version INTEGER NOT NULL,
 created_at REAL NOT NULL,updated_at REAL NOT NULL);
CREATE TABLE IF NOT EXISTS authorizations (
 repository_id TEXT NOT NULL,session_id TEXT NOT NULL,principal TEXT NOT NULL,
 paths TEXT NOT NULL,actions TEXT NOT NULL,evidence TEXT NOT NULL,snapshot_digest TEXT NOT NULL,
 status TEXT NOT NULL,created_at REAL NOT NULL,PRIMARY KEY(repository_id,session_id));
CREATE TABLE IF NOT EXISTS events (
 id INTEGER PRIMARY KEY AUTOINCREMENT,kind TEXT NOT NULL,data TEXT NOT NULL,created_at REAL NOT NULL);
CREATE INDEX IF NOT EXISTS requests_status ON requests(status,created_at);
CREATE INDEX IF NOT EXISTS records_domain ON records(domain,updated_at);
"""


class State:
    def __init__(self, path: str | Path, *, busy_timeout_ms: int = 3000) -> None:
        self.path = Path(path).absolute()
        self.path.parent.mkdir(parents=True, exist_ok=True)
        self.busy_timeout_ms = busy_timeout_ms
        with self.connect() as connection:
            connection.execute("PRAGMA journal_mode=WAL")
            connection.executescript(SCHEMA)
            version = connection.execute("SELECT value FROM metadata WHERE key='schemaVersion'").fetchone()
            if version and version[0] != "1":
                raise JenkinsError("state_schema_mismatch", "Unsupported coordination state schema")
            connection.execute("INSERT OR IGNORE INTO metadata VALUES ('schemaVersion','1')")

    @contextmanager
    def connect(self) -> Iterator[sqlite3.Connection]:
        connection = sqlite3.connect(self.path, timeout=self.busy_timeout_ms / 1000,
                                     isolation_level=None)
        connection.row_factory = sqlite3.Row
        connection.execute("PRAGMA foreign_keys=ON")
        connection.execute("PRAGMA synchronous=FULL")
        connection.execute(f"PRAGMA busy_timeout={self.busy_timeout_ms}")
        try:
            yield connection
        finally:
            connection.close()

    @contextmanager
    def transaction(self) -> Iterator[sqlite3.Connection]:
        with self.connect() as connection:
            try:
                connection.execute("BEGIN IMMEDIATE")
                yield connection
                connection.commit()
            except sqlite3.OperationalError as error:
                connection.rollback()
                if "locked" in str(error).lower() or "busy" in str(error).lower():
                    raise JenkinsError("state_busy", "Coordination transaction is busy", retryable=True) from error
                raise
            except BaseException:
                connection.rollback()
                raise

    @staticmethod
    def _record(row):
        return None if row is None else {"domain": row["domain"], "key": row["key"],
                                        "payload": json.loads(row["payload"]),
                                        "version": row["version"], "updatedAt": row["updated_at"]}

    def get(self, domain: str, key: str, *, connection=None) -> dict | None:
        if connection is None:
            with self.connect() as current:
                return self.get(domain, key, connection=current)
        return self._record(connection.execute("SELECT * FROM records WHERE domain=? AND key=?",
                                               (domain, key)).fetchone())

    def list(self, domain: str, *, connection=None) -> list[dict]:
        if connection is None:
            with self.connect() as current:
                return self.list(domain, connection=current)
        return [self._record(row) for row in connection.execute(
            "SELECT * FROM records WHERE domain=? ORDER BY key", (domain,))]

    def put(self, domain: str, key: str, payload: dict, *, expected_version: int | None = None,
            connection=None) -> dict:
        if connection is None:
            with self.transaction() as current:
                return self.put(domain, key, payload, expected_version=expected_version, connection=current)
        old = self.get(domain, key, connection=connection)
        version = old["version"] if old else 0
        if expected_version is not None and version != expected_version:
            raise JenkinsError("state_conflict", "Record changed before conditional update", retryable=True)
        now = time.time()
        connection.execute("INSERT INTO records VALUES (?,?,?,?,?) ON CONFLICT(domain,key) "
                           "DO UPDATE SET payload=excluded.payload,version=excluded.version,updated_at=excluded.updated_at",
                           (domain, key, canonical_json(payload).decode(), version + 1, now))
        return {"domain": domain, "key": key, "payload": payload, "version": version + 1, "updatedAt": now}

    def delete(self, domain: str, key: str, *, expected_version: int | None = None, connection=None) -> None:
        if connection is None:
            with self.transaction() as current:
                return self.delete(domain, key, expected_version=expected_version, connection=current)
        old = self.get(domain, key, connection=connection)
        if expected_version is not None and (old is None or old["version"] != expected_version):
            raise JenkinsError("state_conflict", "Record changed before conditional delete", retryable=True)
        connection.execute("DELETE FROM records WHERE domain=? AND key=?", (domain, key))

    def event(self, kind: str, data: dict, *, connection=None) -> int:
        if connection is None:
            with self.transaction() as current:
                return self.event(kind, data, connection=current)
        cursor = connection.execute("INSERT INTO events(kind,data,created_at) VALUES (?,?,?)",
                                    (kind, canonical_json(data).decode(), time.time()))
        return cursor.lastrowid

    def put_operation(self, kind: str, operation_id: str, payload: dict,
                      status: str = "prepared", *, connection=None) -> dict:
        identifier(operation_id, "operationId")
        if connection is None:
            with self.transaction() as current:
                return self.put_operation(kind, operation_id, payload, status, connection=current)
        old = self.get_operation(operation_id, connection=connection)
        if old:
            if old["kind"] != kind or old["payloadDigest"] != digest(payload):
                raise JenkinsError("operation_payload_mismatch", "Operation intent payload changed")
            return old
        now = time.time()
        connection.execute("INSERT INTO operations VALUES (?,?,?,?,?,?,1,?,?)",
                           (operation_id, kind, digest(payload), canonical_json(payload).decode(), status, None, now, now))
        return self.get_operation(operation_id, connection=connection)

    def get_operation(self, operation_id: str, *, connection=None) -> dict | None:
        if connection is None:
            with self.connect() as current:
                return self.get_operation(operation_id, connection=current)
        row = connection.execute("SELECT * FROM operations WHERE operation_id=?", (operation_id,)).fetchone()
        if row is None:
            return None
        return {"operationId": row["operation_id"], "kind": row["kind"],
                "payloadDigest": row["payload_digest"], "payload": json.loads(row["payload"]),
                "status": row["status"], "result": json.loads(row["result"]) if row["result"] else None,
                "version": row["version"], "createdAt": row["created_at"], "updatedAt": row["updated_at"]}

    def transition_operation(self, operation_id: str, expected: str | list[str], status: str,
                             result: dict | None = None, *, connection=None) -> dict:
        if connection is None:
            with self.transaction() as current:
                return self.transition_operation(operation_id, expected, status, result, connection=current)
        old = self.get_operation(operation_id, connection=connection)
        allowed = [expected] if isinstance(expected, str) else expected
        if old is None or old["status"] not in allowed:
            raise JenkinsError("operation_conflict", "Operation phase changed before transition", retryable=True)
        connection.execute("UPDATE operations SET status=?,result=?,version=version+1,updated_at=? WHERE operation_id=?",
                           (status, canonical_json(result).decode() if result is not None else None,
                            time.time(), operation_id))
        self.event("operation_transition", {"operationId": operation_id, "from": old["status"], "to": status},
                   connection=connection)
        return self.get_operation(operation_id, connection=connection)

    def submit_request(self, payload: dict) -> dict:
        repo = identifier(payload.get("repositoryId"), "repositoryId")
        session = identifier(payload.get("sessionId"), "sessionId")
        request = identifier(payload.get("requestId"), "requestId")
        with self.transaction() as connection:
            row = connection.execute("SELECT * FROM requests WHERE repository_id=? AND session_id=? AND request_id=?",
                                     (repo, session, request)).fetchone()
            if row:
                if row["payload_digest"] != digest(payload):
                    raise JenkinsError("request_payload_mismatch", "Duplicate request payload differs")
                return self._request(row)
            now = time.time()
            connection.execute("INSERT INTO requests VALUES (?,?,?,?,?,1,'pending',?,?)",
                               (repo, session, request, digest(payload), canonical_json(payload).decode(), now, now))
            self.event("request_submitted", {"repositoryId": repo, "sessionId": session, "requestId": request},
                       connection=connection)
            return self._request(connection.execute(
                "SELECT * FROM requests WHERE repository_id=? AND session_id=? AND request_id=?", (repo, session, request)).fetchone())

    @staticmethod
    def _request(row) -> dict:
        return {"repositoryId": row["repository_id"], "sessionId": row["session_id"],
                "requestId": row["request_id"], "payloadDigest": row["payload_digest"],
                "payload": json.loads(row["payload"]), "generation": row["generation"],
                "status": row["status"], "createdAt": row["created_at"], "updatedAt": row["updated_at"]}

    def get_request(self, repository_id: str, session_id: str, request_id: str,
                    *, connection=None) -> dict | None:
        if connection is None:
            with self.connect() as current:
                return self.get_request(repository_id, session_id, request_id, connection=current)
        row = connection.execute("SELECT * FROM requests WHERE repository_id=? AND session_id=? AND request_id=?",
                                 (repository_id, session_id, request_id)).fetchone()
        return self._request(row) if row else None

    def transition_request(self, repository_id: str, session_id: str, request_id: str,
                           generation: int, expected: str | list[str], status: str,
                           result: dict | None = None, *, connection=None) -> dict:
        """Persist workflow progress and evidence atomically without rewriting intent.

        Only the authoritative workflow collector calls this method.  Request
        delivery and Jenkins' result are separate records, not acceptance.
        """
        if connection is None:
            with self.transaction() as current:
                return self.transition_request(repository_id, session_id, request_id, generation,
                                               expected, status, result, connection=current)
        if status not in {"pending", "running", "waiting", "accepted", "failed", "cancelled", "blocked", "reconciling"}:
            raise JenkinsError("request_status_invalid", "Unsupported request status")
        old = self.get_request(repository_id, session_id, request_id, connection=connection)
        if old is None or old["generation"] != generation:
            raise JenkinsError("request_generation_mismatch", "Request generation changed")
        key = digest([repository_id, session_id, request_id])
        evidence = {"repositoryId": repository_id, "sessionId": session_id, "requestId": request_id,
                    "generation": generation, "status": status, "result": result}
        previous = self.get("request_result", key, connection=connection)
        if old["status"] == status and previous and previous["payload"] == evidence:
            return old
        allowed = [expected] if isinstance(expected, str) else expected
        if old["status"] not in allowed or old["status"] in {"accepted", "failed", "cancelled"}:
            raise JenkinsError("request_conflict", "Request phase changed before transition", retryable=True)
        if status == "accepted":
            receipt = self.get("acceptance", (result or {}).get("receiptRef", ""), connection=connection)
            if receipt is None or receipt["payload"].get("status") != "accepted":
                raise JenkinsError("request_acceptance_missing", "Request acceptance requires an authoritative receipt")
        connection.execute("UPDATE requests SET status=?,updated_at=? WHERE repository_id=? AND session_id=? AND request_id=?",
                           (status, time.time(), repository_id, session_id, request_id))
        self.put("request_result", key, evidence, expected_version=previous["version"] if previous else 0,
                 connection=connection)
        self.event("request_transition", {"repositoryId": repository_id, "sessionId": session_id,
                                         "requestId": request_id, "generation": generation,
                                         "from": old["status"], "to": status}, connection=connection)
        return self.get_request(repository_id, session_id, request_id, connection=connection)

    def authorize_session(self, repository_id: str, session_id: str, principal: str,
                          paths: list[str], actions: list[str], evidence: dict) -> dict:
        if evidence.get("source") != "user" or not evidence.get("task"):
            raise JenkinsError("authorization_evidence_missing", "Task authorization evidence is required")
        normalized = [_relative_path(path) for path in paths]
        for action in actions:
            if action not in {"implementation", "validation", "commit", "push", "notify", "gc", "deployment"}:
                raise JenkinsError("authorization_action_invalid", "Unsupported authorization action")
        snapshot = {"repositoryId": repository_id, "sessionId": session_id, "principal": principal,
                    "paths": normalized, "actions": sorted(set(actions)), "evidence": evidence}
        with self.transaction() as connection:
            old = connection.execute("SELECT snapshot_digest FROM authorizations WHERE repository_id=? AND session_id=?",
                                     (repository_id, session_id)).fetchone()
            if old and old[0] != digest(snapshot):
                raise JenkinsError("authorization_conflict", "Session authorization already exists with different scope")
            connection.execute("INSERT OR IGNORE INTO authorizations VALUES (?,?,?,?,?,?,?,'active',?)",
                               (repository_id, session_id, principal, canonical_json(normalized).decode(),
                                canonical_json(snapshot["actions"]).decode(), canonical_json(evidence).decode(),
                                digest(snapshot), time.time()))
            self.event("session_authorized", {"repositoryId": repository_id, "sessionId": session_id,
                                               "snapshotDigest": digest(snapshot)}, connection=connection)
        return {**snapshot, "snapshotDigest": digest(snapshot), "status": "active"}

    def check_authorization(self, repository_id: str, session_id: str, action: str,
                            paths: list[str] | None = None, *, connection=None) -> dict:
        if connection is None:
            with self.connect() as current:
                return self.check_authorization(repository_id, session_id, action, paths, connection=current)
        row = connection.execute("SELECT * FROM authorizations WHERE repository_id=? AND session_id=?",
                                 (repository_id, session_id)).fetchone()
        if row is None or row["status"] != "active" or action not in json.loads(row["actions"]):
            raise JenkinsError("action_not_authorized", "Task action lacks an active scoped authorization")
        allowed = json.loads(row["paths"])
        for path in paths or []:
            current = _relative_path(path)
            if not any(current == scope or current.startswith(scope + "/") for scope in allowed):
                raise JenkinsError("path_not_authorized", "Path lies outside task authorization")
        return {"principal": row["principal"], "snapshotDigest": row["snapshot_digest"],
                "actions": json.loads(row["actions"]), "paths": allowed}

    def revoke_session(self, repository_id: str, session_id: str) -> None:
        with self.transaction() as connection:
            connection.execute("UPDATE authorizations SET status='revoked' WHERE repository_id=? AND session_id=?",
                               (repository_id, session_id))
            self.event("session_revoked", {"repositoryId": repository_id, "sessionId": session_id}, connection=connection)


def _relative_path(value: str) -> str:
    if not isinstance(value, str) or value.startswith(("/", "\\")):
        raise JenkinsError("invalid_scope_path", "Authorization scope must be repository-relative")
    path = value.replace("\\", "/").rstrip("/").casefold()
    if not path or ":" in path or any(part in {"", ".", ".."} for part in path.split("/")):
        raise JenkinsError("invalid_scope_path", "Authorization scope must be a repository-relative path")
    return path
