from __future__ import annotations

import gzip
import hashlib
import json
import os
import shutil
import sqlite3
from dataclasses import dataclass
from datetime import UTC, datetime, timedelta
from pathlib import Path

from .database import Database
from .models import CoordinatorError, parse_utc, utc_text


@dataclass(frozen=True, slots=True)
class ManifestRetentionCandidate:
    table: str
    identity: str
    sha256: str
    entry_count: int
    byte_count: int

    def to_dict(self) -> dict[str, object]:
        return {
            "table": self.table,
            "identity": self.identity,
            "sha256": self.sha256,
            "entryCount": self.entry_count,
            "byteCount": self.byte_count,
        }


@dataclass(frozen=True, slots=True)
class ManifestRetentionPreview:
    fingerprint: str
    candidates: tuple[ManifestRetentionCandidate, ...]
    created_at: datetime

    @classmethod
    def create(
        cls,
        candidates: tuple[ManifestRetentionCandidate, ...],
        created_at: datetime,
    ) -> "ManifestRetentionPreview":
        ordered = tuple(sorted(candidates, key=lambda item: (item.table, item.identity)))
        encoded = json.dumps(
            [candidate.to_dict() for candidate in ordered],
            sort_keys=True,
            separators=(",", ":"),
        ).encode("utf-8")
        return cls(
            f"manifest-retention-{hashlib.sha256(encoded).hexdigest()[:24]}",
            ordered,
            created_at,
        )

    def require_fingerprint(self, fingerprint: str) -> None:
        if fingerprint != self.fingerprint:
            raise CoordinatorError(
                "manifest_retention_preview_stale",
                "Manifest retention apply must use its exact preview fingerprint",
            )

    def to_dict(self) -> dict[str, object]:
        return {
            "fingerprint": self.fingerprint,
            "candidates": [candidate.to_dict() for candidate in self.candidates],
            "createdAt": self.created_at.isoformat(),
        }


@dataclass(frozen=True, slots=True)
class ManifestRetentionResult:
    batch_id: str
    archive_path: Path
    backup_path: Path
    retired_count: int


@dataclass(frozen=True, slots=True)
class ManifestCompactionResult:
    batch_id: str
    quick_check: str
    size_before: int
    size_after: int


@dataclass(frozen=True, slots=True)
class ManifestCompactionReceipt:
    batch_id: str
    status: str


class ManifestRetentionService:
    """Archive old manifests before retiring their SQLite payloads.

    A retention batch has exactly one gzip JSONL archive and one SQLite backup.
    The service re-derives candidates inside its retirement transaction, so a
    preview never grants authority over a manifest that changed meanwhile.
    """

    _TERMINAL_SESSION_STATUSES = ("completed", "archived", "cancelled")
    _ACTIVE_COPY_STATUSES = ("planned", "materialized", "running", "cleanup_pending")
    _TERMINAL_COPY_STATUSES = ("removed", "failed")

    def __init__(
        self,
        database: Database,
        state_root: str | Path,
        *,
        retention_days: int = 7,
    ) -> None:
        if retention_days < 1:
            raise ValueError("retention_days must be positive")
        self.database = database
        self.state_root = Path(state_root).resolve()
        self.retention_days = retention_days
        self.archive_root = self.state_root / "manifest-archives"
        self.backup_root = self.state_root / "backups"

    def preview(self, *, now: datetime | None = None) -> ManifestRetentionPreview:
        current_time = now or datetime.now(UTC)
        with self.database.connect() as connection:
            candidates = self._candidates_with_connection(connection, current_time)
        return ManifestRetentionPreview.create(candidates, current_time)

    def record_preview(
        self,
        preview: ManifestRetentionPreview,
        *,
        actor: str,
        now: datetime | None = None,
    ) -> None:
        current_time = now or datetime.now(UTC)
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO manifest_retention_previews(
                    fingerprint, actor, candidate_count, candidates_json, created_at
                ) VALUES (?, ?, ?, ?, ?)
                ON CONFLICT(fingerprint) DO NOTHING
                """,
                (
                    preview.fingerprint,
                    actor,
                    len(preview.candidates),
                    json.dumps(
                        [candidate.to_dict() for candidate in preview.candidates],
                        sort_keys=True,
                        separators=(",", ":"),
                    ),
                    utc_text(current_time),
                ),
            )

    def load_preview(self, fingerprint: str) -> ManifestRetentionPreview:
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT candidates_json, created_at FROM manifest_retention_previews WHERE fingerprint=?",
                (fingerprint,),
            ).fetchone()
        if row is None:
            raise CoordinatorError(
                "manifest_retention_preview_missing",
                f"Unknown manifest retention preview {fingerprint}",
            )
        candidates = tuple(
            ManifestRetentionCandidate(
                str(item["table"]),
                str(item["identity"]),
                str(item["sha256"]),
                int(item["entryCount"]),
                int(item["byteCount"]),
            )
            for item in json.loads(str(row["candidates_json"]))
        )
        preview = ManifestRetentionPreview.create(
            candidates, parse_utc(str(row["created_at"]))
        )
        if preview.fingerprint != fingerprint:
            raise CoordinatorError(
                "manifest_retention_preview_tampered",
                "Persisted manifest retention preview does not match its fingerprint",
            )
        return preview

    def apply(
        self,
        preview: ManifestRetentionPreview,
        *,
        fingerprint: str,
        actor: str,
        now: datetime | None = None,
    ) -> ManifestRetentionResult:
        preview.require_fingerprint(fingerprint)
        completed = self._completed_result(preview.fingerprint)
        if completed is not None:
            return completed
        current_time = now or datetime.now(UTC)
        current_preview = self.preview(now=current_time)
        if current_preview.fingerprint != preview.fingerprint:
            raise CoordinatorError(
                "manifest_retention_preview_stale",
                "Manifest retention candidates changed after preview",
            )
        if not preview.candidates:
            raise CoordinatorError(
                "manifest_retention_candidates_empty",
                "Manifest retention preview contains no retireable manifests",
            )

        self._require_storage_capacity(preview)
        archive_path = self.archive_root / f"{preview.fingerprint}.jsonl.gz"
        backup_path = self.backup_root / f"{preview.fingerprint}.sqlite3"
        self._begin_batch(preview, actor, archive_path, backup_path, current_time)
        archive_temporary = archive_path.with_suffix(archive_path.suffix + ".tmp")
        archive_published = False
        try:
            self._create_backup(backup_path)
            records = self._read_records(preview)
            self._write_archive(archive_temporary, records)
            self._verify_archive(archive_temporary, preview.candidates)
            archive_path.parent.mkdir(parents=True, exist_ok=True)
            os.replace(archive_temporary, archive_path)
            archive_published = True
            self._retire(preview, archive_path, current_time)
        except BaseException as error:
            archive_temporary.unlink(missing_ok=True)
            if archive_published:
                archive_path.unlink(missing_ok=True)
            self._fail_batch(preview.fingerprint, str(error))
            raise
        return ManifestRetentionResult(
            preview.fingerprint,
            archive_path,
            backup_path,
            len(preview.candidates),
        )

    def _completed_result(self, batch_id: str) -> ManifestRetentionResult | None:
        with self.database.connect() as connection:
            row = connection.execute(
                """
                SELECT status, archive_path, backup_path, candidate_count
                FROM manifest_retention_batches WHERE batch_id=?
                """,
                (batch_id,),
            ).fetchone()
        if row is None or row["status"] not in {"retired", "compacted"}:
            return None
        archive_path = self.state_root / str(row["archive_path"])
        backup_path = self.state_root / str(row["backup_path"])
        if not archive_path.is_file() or not backup_path.is_file():
            raise CoordinatorError(
                "manifest_retention_batch_artifact_missing",
                "A completed manifest retention batch is missing its archive or backup",
                details={"batchId": batch_id},
            )
        return ManifestRetentionResult(
            batch_id,
            archive_path,
            backup_path,
            int(row["candidate_count"]),
        )

    def compact(
        self,
        batch_id: str,
        *,
        actor: str,
        now: datetime | None = None,
    ) -> ManifestCompactionResult:
        current_time = now or datetime.now(UTC)
        with self.database.connect() as connection:
            batch = connection.execute(
                "SELECT status FROM manifest_retention_batches WHERE batch_id=?", (batch_id,)
            ).fetchone()
        if batch is None:
            raise CoordinatorError(
                "manifest_retention_batch_missing", f"Unknown manifest retention batch {batch_id}"
            )
        if batch["status"] not in {"retired", "compact_pending", "compacted"}:
            raise CoordinatorError(
                "manifest_retention_compact_not_ready",
                f"Manifest retention batch {batch_id} is {batch['status']}",
            )
        quick_check = self._quick_check()
        if quick_check != "ok":
            raise CoordinatorError(
                "manifest_retention_database_unhealthy",
                "SQLite quick_check failed before manifest compaction",
                details={"quickCheck": quick_check},
            )
        size_before = self.database.path.stat().st_size if self.database.path.exists() else 0
        if batch["status"] in {"retired", "compact_pending"}:
            with self.database.connect() as connection:
                connection.execute("PRAGMA wal_checkpoint(TRUNCATE)")
                connection.execute("VACUUM")
            quick_check = self._quick_check()
            if quick_check != "ok":
                raise CoordinatorError(
                    "manifest_retention_database_unhealthy",
                    "SQLite quick_check failed after manifest compaction",
                    details={"quickCheck": quick_check},
                )
            with self.database.transaction() as connection:
                cursor = connection.execute(
                    """
                    UPDATE manifest_retention_batches
                    SET status='compacted', compacted_at=?, error_text=NULL
                    WHERE batch_id=? AND status IN ('retired', 'compact_pending')
                    """,
                    (utc_text(current_time), batch_id),
                )
                if cursor.rowcount != 1:
                    raise CoordinatorError(
                        "manifest_retention_compact_conflict",
                        "Manifest retention batch changed during compact",
                    )
                connection.execute(
                    "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, ?)",
                    (
                        "manifest_retention.compacted",
                        json.dumps({"batchId": batch_id, "actor": actor}, sort_keys=True),
                        utc_text(current_time),
                    ),
                )
        size_after = self.database.path.stat().st_size if self.database.path.exists() else 0
        return ManifestCompactionResult(batch_id, quick_check, size_before, size_after)

    def queue_compact(
        self,
        batch_id: str,
        *,
        actor: str,
        now: datetime | None = None,
    ) -> ManifestCompactionReceipt:
        current_time = now or datetime.now(UTC)
        with self.database.transaction() as connection:
            row = connection.execute(
                "SELECT status FROM manifest_retention_batches WHERE batch_id=?", (batch_id,)
            ).fetchone()
            if row is None:
                raise CoordinatorError(
                    "manifest_retention_batch_missing",
                    f"Unknown manifest retention batch {batch_id}",
                )
            status = str(row["status"])
            if status == "compacted":
                return ManifestCompactionReceipt(batch_id, status)
            if status not in {"retired", "compact_pending"}:
                raise CoordinatorError(
                    "manifest_retention_compact_not_ready",
                    f"Manifest retention batch {batch_id} is {status}",
                )
            connection.execute(
                """
                UPDATE manifest_retention_batches
                SET status='compact_pending', error_text=NULL
                WHERE batch_id=? AND status='retired'
                """,
                (batch_id,),
            )
            connection.execute(
                "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, ?)",
                (
                    "manifest_retention.compact_queued",
                    json.dumps({"batchId": batch_id, "actor": actor}, sort_keys=True),
                    utc_text(current_time),
                ),
            )
        return ManifestCompactionReceipt(batch_id, "compact_pending")

    def pending_compactions(self) -> tuple[str, ...]:
        with self.database.connect() as connection:
            rows = connection.execute(
                """
                SELECT batch_id FROM manifest_retention_batches
                WHERE status='compact_pending' ORDER BY created_at, batch_id
                """
            ).fetchall()
        return tuple(str(row["batch_id"]) for row in rows)

    def _candidates_with_connection(
        self, connection: sqlite3.Connection, current_time: datetime
    ) -> tuple[ManifestRetentionCandidate, ...]:
        cutoff = current_time - timedelta(days=self.retention_days)
        protected_epochs = self._protected_epochs(connection)
        candidates: list[ManifestRetentionCandidate] = []
        for row in connection.execute(
            """
            SELECT epoch_id, manifest_json, created_at
            FROM baseline_epochs
            WHERE manifest_archive_path IS NULL
            ORDER BY epoch_id
            """
        ):
            epoch_id = int(row["epoch_id"])
            if epoch_id in protected_epochs or parse_utc(str(row["created_at"])) > cutoff:
                continue
            candidate = self._candidate("baseline_epochs", str(epoch_id), str(row["manifest_json"]))
            if candidate is not None:
                candidates.append(candidate)
        placeholders = ", ".join("?" for _ in self._TERMINAL_COPY_STATUSES)
        for row in connection.execute(
            f"""
            SELECT job_id, manifest_json, created_at, removed_at
            FROM validation_copies
            WHERE manifest_archive_path IS NULL AND status IN ({placeholders})
            ORDER BY job_id
            """,
            self._TERMINAL_COPY_STATUSES,
        ):
            created_at = parse_utc(str(row["created_at"]))
            terminal_at = parse_utc(str(row["removed_at"] or row["created_at"]))
            if created_at > cutoff or terminal_at > cutoff:
                continue
            candidate = self._candidate(
                "validation_copies", str(row["job_id"]), str(row["manifest_json"])
            )
            if candidate is not None:
                candidates.append(candidate)
        return tuple(sorted(candidates, key=lambda item: (item.table, item.identity)))

    def _protected_epochs(self, connection: sqlite3.Connection) -> set[int]:
        protected: set[int] = set()
        latest = connection.execute("SELECT MAX(epoch_id) FROM baseline_epochs").fetchone()[0]
        if latest is not None:
            protected.add(int(latest))
        placeholders = ", ".join("?" for _ in self._TERMINAL_SESSION_STATUSES)
        rows = connection.execute(
            f"""
            SELECT DISTINCT baseline_epoch FROM sessions
            WHERE baseline_epoch IS NOT NULL AND status NOT IN ({placeholders})
            """,
            self._TERMINAL_SESSION_STATUSES,
        ).fetchall()
        protected.update(int(row["baseline_epoch"]) for row in rows)
        return protected

    def _candidate(
        self, table: str, identity: str, manifest_json: str
    ) -> ManifestRetentionCandidate | None:
        payload = self._manifest_payload(manifest_json)
        if not payload:
            return None
        encoded = manifest_json.encode("utf-8")
        return ManifestRetentionCandidate(
            table,
            identity,
            hashlib.sha256(encoded).hexdigest(),
            len(payload) if isinstance(payload, (list, dict)) else 1,
            len(encoded),
        )

    @staticmethod
    def _manifest_payload(manifest_json: str) -> object:
        try:
            payload = json.loads(manifest_json)
        except json.JSONDecodeError as error:
            raise CoordinatorError(
                "manifest_retention_manifest_invalid",
                "Manifest retention requires valid JSON manifests",
            ) from error
        if not isinstance(payload, (list, dict)):
            raise CoordinatorError(
                "manifest_retention_manifest_invalid",
                "Manifest retention only accepts list or object manifests",
            )
        return payload

    def _require_storage_capacity(self, preview: ManifestRetentionPreview) -> None:
        database_bytes = self.database.path.stat().st_size if self.database.path.exists() else 0
        archive_bytes = sum(candidate.byte_count for candidate in preview.candidates)
        required_bytes = database_bytes * 3 + archive_bytes
        self.state_root.mkdir(parents=True, exist_ok=True)
        free_bytes = shutil.disk_usage(self.state_root).free
        if free_bytes < required_bytes:
            raise CoordinatorError(
                "manifest_retention_insufficient_space",
                "Insufficient free space for backup, archive, and SQLite compaction",
                details={"requiredBytes": required_bytes, "freeBytes": free_bytes},
            )

    def _begin_batch(
        self,
        preview: ManifestRetentionPreview,
        actor: str,
        archive_path: Path,
        backup_path: Path,
        current_time: datetime,
    ) -> None:
        archive_relative = str(archive_path.relative_to(self.state_root))
        backup_relative = str(backup_path.relative_to(self.state_root))
        with self.database.transaction() as connection:
            row = connection.execute(
                "SELECT status FROM manifest_retention_batches WHERE batch_id=?",
                (preview.fingerprint,),
            ).fetchone()
            if row is not None and row["status"] in {"retired", "compacted"}:
                raise CoordinatorError(
                    "manifest_retention_batch_already_applied",
                    f"Manifest retention batch {preview.fingerprint} is already {row['status']}",
                )
            if row is not None and row["status"] == "applying":
                raise CoordinatorError(
                    "manifest_retention_batch_busy",
                    f"Manifest retention batch {preview.fingerprint} is already applying",
                )
            connection.execute(
                """
                INSERT INTO manifest_retention_batches(
                    batch_id, actor, status, candidate_count, archive_path, backup_path, created_at,
                    retired_at, compacted_at, error_text
                ) VALUES (?, ?, 'applying', ?, ?, ?, ?, NULL, NULL, NULL)
                ON CONFLICT(batch_id) DO UPDATE SET
                    actor=excluded.actor, status='applying', candidate_count=excluded.candidate_count,
                    archive_path=excluded.archive_path, backup_path=excluded.backup_path,
                    created_at=excluded.created_at, retired_at=NULL, compacted_at=NULL, error_text=NULL
                """,
                (
                    preview.fingerprint,
                    actor,
                    len(preview.candidates),
                    archive_relative,
                    backup_relative,
                    utc_text(current_time),
                ),
            )

    def _create_backup(self, backup_path: Path) -> None:
        backup_path.parent.mkdir(parents=True, exist_ok=True)
        temporary = backup_path.with_suffix(backup_path.suffix + ".tmp")
        temporary.unlink(missing_ok=True)
        destination = sqlite3.connect(temporary)
        try:
            with self.database.connect() as source:
                source.backup(destination)
        finally:
            destination.close()
        os.replace(temporary, backup_path)

    def _read_records(self, preview: ManifestRetentionPreview) -> tuple[dict[str, object], ...]:
        records: list[dict[str, object]] = []
        with self.database.connect() as connection:
            for candidate in preview.candidates:
                manifest_json = self._read_manifest(connection, candidate)
                encoded = manifest_json.encode("utf-8")
                if hashlib.sha256(encoded).hexdigest() != candidate.sha256:
                    raise CoordinatorError(
                        "manifest_retention_preview_stale",
                        "Manifest payload changed after preview",
                    )
                records.append(
                    {
                        "batchId": preview.fingerprint,
                        "table": candidate.table,
                        "identity": candidate.identity,
                        "manifestJson": manifest_json,
                        "sha256": candidate.sha256,
                        "entryCount": candidate.entry_count,
                        "byteCount": candidate.byte_count,
                    }
                )
        return tuple(records)

    def _read_manifest(
        self, connection: sqlite3.Connection, candidate: ManifestRetentionCandidate
    ) -> str:
        if candidate.table == "baseline_epochs":
            row = connection.execute(
                """
                SELECT manifest_json FROM baseline_epochs
                WHERE epoch_id=? AND manifest_archive_path IS NULL
                """,
                (int(candidate.identity),),
            ).fetchone()
        else:
            row = connection.execute(
                """
                SELECT manifest_json FROM validation_copies
                WHERE job_id=? AND manifest_archive_path IS NULL
                """,
                (candidate.identity,),
            ).fetchone()
        if row is None:
            raise CoordinatorError(
                "manifest_retention_preview_stale",
                f"Manifest retention candidate disappeared: {candidate.table}:{candidate.identity}",
            )
        return str(row["manifest_json"])

    def _write_archive(self, temporary: Path, records: tuple[dict[str, object], ...]) -> None:
        temporary.parent.mkdir(parents=True, exist_ok=True)
        temporary.unlink(missing_ok=True)
        with gzip.open(temporary, "xt", encoding="utf-8") as stream:
            for record in records:
                stream.write(json.dumps(record, sort_keys=True, separators=(",", ":")))
                stream.write("\n")

    def _verify_archive(
        self,
        temporary: Path,
        expected: tuple[ManifestRetentionCandidate, ...],
    ) -> None:
        actual: list[ManifestRetentionCandidate] = []
        try:
            with gzip.open(temporary, "rt", encoding="utf-8") as stream:
                for line in stream:
                    record = json.loads(line)
                    manifest_json = record["manifestJson"]
                    if not isinstance(manifest_json, str):
                        raise ValueError("manifestJson is not text")
                    payload = self._manifest_payload(manifest_json)
                    encoded = manifest_json.encode("utf-8")
                    sha256 = hashlib.sha256(encoded).hexdigest()
                    entry_count = len(payload) if isinstance(payload, (list, dict)) else 1
                    if (
                        record.get("sha256") != sha256
                        or record.get("entryCount") != entry_count
                        or record.get("byteCount") != len(encoded)
                    ):
                        raise ValueError("manifest archive summary mismatch")
                    actual.append(
                        ManifestRetentionCandidate(
                            str(record["table"]),
                            str(record["identity"]),
                            sha256,
                            entry_count,
                            len(encoded),
                        )
                    )
        except (OSError, ValueError, KeyError, json.JSONDecodeError) as error:
            raise CoordinatorError(
                "manifest_retention_archive_hash_mismatch",
                "Manifest archive failed its read-back verification",
            ) from error
        if tuple(actual) != expected:
            raise CoordinatorError(
                "manifest_retention_archive_hash_mismatch",
                "Manifest archive does not match the retention preview",
            )

    def _retire(
        self,
        preview: ManifestRetentionPreview,
        archive_path: Path,
        current_time: datetime,
    ) -> None:
        archive_relative = str(archive_path.relative_to(self.state_root))
        with self.database.transaction() as connection:
            current = self._candidates_with_connection(connection, current_time)
            if current != preview.candidates:
                raise CoordinatorError(
                    "manifest_retention_preview_stale",
                    "Manifest retention candidates changed before retirement",
                )
            for candidate in preview.candidates:
                if candidate.table == "baseline_epochs":
                    cursor = connection.execute(
                        """
                        UPDATE baseline_epochs
                        SET manifest_json='{}', manifest_sha256=?, manifest_entry_count=?,
                            manifest_byte_count=?, manifest_archive_path=?, manifest_archived_at=?
                        WHERE epoch_id=? AND manifest_archive_path IS NULL AND manifest_json=?
                        """,
                        (
                            candidate.sha256,
                            candidate.entry_count,
                            candidate.byte_count,
                            archive_relative,
                            utc_text(current_time),
                            int(candidate.identity),
                            self._read_manifest(connection, candidate),
                        ),
                    )
                else:
                    cursor = connection.execute(
                        """
                        UPDATE validation_copies
                        SET manifest_json='[]', manifest_sha256=?, manifest_entry_count=?,
                            manifest_byte_count=?, manifest_archive_path=?, manifest_archived_at=?
                        WHERE job_id=? AND manifest_archive_path IS NULL AND manifest_json=?
                        """,
                        (
                            candidate.sha256,
                            candidate.entry_count,
                            candidate.byte_count,
                            archive_relative,
                            utc_text(current_time),
                            candidate.identity,
                            self._read_manifest(connection, candidate),
                        ),
                    )
                if cursor.rowcount != 1:
                    raise CoordinatorError(
                        "manifest_retention_preview_stale",
                        f"Manifest changed before retirement: {candidate.table}:{candidate.identity}",
                    )
            connection.execute(
                """
                UPDATE manifest_retention_batches
                SET status='retired', retired_at=?, error_text=NULL
                WHERE batch_id=? AND status='applying'
                """,
                (utc_text(current_time), preview.fingerprint),
            )
            connection.execute(
                "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, ?)",
                (
                    "manifest_retention.retired",
                    json.dumps(
                        {
                            "batchId": preview.fingerprint,
                            "candidateCount": len(preview.candidates),
                            "archivePath": archive_relative,
                        },
                        sort_keys=True,
                    ),
                    utc_text(current_time),
                ),
            )

    def _fail_batch(self, batch_id: str, error_text: str) -> None:
        with self.database.transaction() as connection:
            connection.execute(
                """
                UPDATE manifest_retention_batches
                SET status='failed', error_text=?
                WHERE batch_id=? AND status='applying'
                """,
                (error_text, batch_id),
            )

    def _quick_check(self) -> str:
        with self.database.connect() as connection:
            row = connection.execute("PRAGMA quick_check").fetchone()
        return str(row[0]) if row is not None else "unknown"
