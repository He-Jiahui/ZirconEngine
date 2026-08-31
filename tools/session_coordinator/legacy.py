from __future__ import annotations

import hashlib
import json
import os
import re
import sqlite3
import subprocess
from dataclasses import dataclass
from datetime import UTC, datetime, timedelta
from pathlib import Path
from typing import Callable

from .database import Database
from .models import CoordinatorError, SessionStatus, utc_text
from .processes import process_is_alive
from .sessions import SessionService


STATUS_ALIASES: dict[str, SessionStatus] = {
    status.value: status for status in SessionStatus
}
STATUS_ALIASES.update(
    {
        "in_progress": SessionStatus.ACTIVE,
        "in-progress": SessionStatus.ACTIVE,
        "working": SessionStatus.ACTIVE,
        "implementing": SessionStatus.ACTIVE,
        "done": SessionStatus.COMPLETED,
        "complete": SessionStatus.COMPLETED,
        "canceled": SessionStatus.CANCELLED,
    }
)
TERMINAL_ARCHIVE_STATUSES = {
    SessionStatus.STALE,
    SessionStatus.COMPLETED,
    SessionStatus.CANCELLED,
}


@dataclass(frozen=True, slots=True)
class LegacyNoteRecord:
    note_path: str
    session_id: str
    source_status: str | None
    mapped_status: SessionStatus
    status_reason: str | None
    plan_path: str | None
    pid: int | None
    modified_at: datetime
    content_hash: str
    activity_reasons: tuple[str, ...]
    archive_eligible: bool

    def to_dict(self) -> dict[str, object]:
        return {
            "note_path": self.note_path,
            "session_id": self.session_id,
            "source_status": self.source_status,
            "mapped_status": self.mapped_status.value,
            "status_reason": self.status_reason,
            "plan_path": self.plan_path,
            "pid": self.pid,
            "modified_at": self.modified_at.isoformat(),
            "content_hash": self.content_hash,
            "activity_reasons": list(self.activity_reasons),
            "archive_eligible": self.archive_eligible,
        }


@dataclass(frozen=True, slots=True)
class LegacyMigrationReport:
    notes: tuple[LegacyNoteRecord, ...]

    def to_dict(self) -> dict[str, object]:
        return {
            "note_count": len(self.notes),
            "archive_eligible_count": sum(item.archive_eligible for item in self.notes),
            "notes": [item.to_dict() for item in self.notes],
        }


@dataclass(frozen=True, slots=True)
class ArchiveManifestEntry:
    source_path: str
    destination_path: str
    session_id: str
    before_hash: str
    after_hash: str

    def to_dict(self) -> dict[str, str]:
        return {
            "source_path": self.source_path,
            "destination_path": self.destination_path,
            "session_id": self.session_id,
            "before_hash": self.before_hash,
            "after_hash": self.after_hash,
        }


@dataclass(frozen=True, slots=True)
class ArchiveReport:
    run_id: str
    candidates: tuple[str, ...]
    manifest: tuple[ArchiveManifestEntry, ...]
    applied: bool

    def to_dict(self) -> dict[str, object]:
        return {
            "run_id": self.run_id,
            "candidates": list(self.candidates),
            "manifest": [item.to_dict() for item in self.manifest],
            "applied": self.applied,
        }


class LegacyMigrationService:
    def __init__(
        self,
        database: Database,
        repo_root: str | Path,
        sessions: SessionService,
        *,
        process_alive: Callable[[int], bool] = process_is_alive,
        recent_seconds: int = 600,
        archive_after_hours: int = 24,
    ):
        self.database = database
        self.repo_root = Path(repo_root).resolve()
        self.sessions = sessions
        self.process_alive = process_alive
        self.recent_seconds = recent_seconds
        self.archive_after_hours = archive_after_hours
        self.session_root = self.repo_root / ".codex" / "sessions"
        self.archive_root = self.session_root / "archive"

    def report(self, *, now: datetime | None = None) -> LegacyMigrationReport:
        current_time = now or datetime.now(UTC)
        notes = tuple(
            self._read_note(path, now=current_time)
            for path in sorted(self._active_notes(), key=lambda item: item.name.casefold())
        )
        return LegacyMigrationReport(notes)

    def import_notes(self, *, now: datetime | None = None) -> LegacyMigrationReport:
        current_time = now or datetime.now(UTC)
        report = self.report(now=current_time)
        head = self._git("rev-parse", "HEAD")
        with self.database.transaction() as connection:
            for note in report.notes:
                existing = connection.execute(
                    "SELECT * FROM sessions WHERE session_id = ?", (note.session_id,)
                ).fetchone()
                previously_imported = connection.execute(
                    "SELECT 1 FROM legacy_note_imports WHERE note_path = ?",
                    (note.note_path,),
                ).fetchone()
                if existing is None:
                    connection.execute(
                        """
                        INSERT INTO sessions(
                            session_id, display_name, plan_path, status, status_reason,
                            base_head, write_scope_json, created_at, updated_at,
                            last_heartbeat_at, completed_at, archived_at
                        ) VALUES (?, ?, ?, ?, ?, ?, '[]', ?, ?, ?, ?, ?)
                        """,
                        (
                            note.session_id,
                            note.session_id,
                            note.plan_path,
                            note.mapped_status.value,
                            note.status_reason,
                            head,
                            note.modified_at.isoformat(),
                            note.modified_at.isoformat(),
                            note.modified_at.isoformat(),
                            note.modified_at.isoformat()
                            if note.mapped_status
                            in {SessionStatus.COMPLETED, SessionStatus.CANCELLED}
                            else None,
                            note.modified_at.isoformat()
                            if note.mapped_status is SessionStatus.ARCHIVED
                            else None,
                        ),
                    )
                elif previously_imported is not None and datetime.fromisoformat(
                    existing["updated_at"]
                ) <= note.modified_at:
                    connection.execute(
                        """
                        UPDATE sessions
                        SET plan_path = COALESCE(?, plan_path), status = ?, status_reason = ?,
                            updated_at = ?, last_heartbeat_at = ?,
                            completed_at = ?, archived_at = ?
                        WHERE session_id = ?
                        """,
                        (
                            note.plan_path,
                            note.mapped_status.value,
                            note.status_reason,
                            note.modified_at.isoformat(),
                            note.modified_at.isoformat(),
                            note.modified_at.isoformat()
                            if note.mapped_status
                            in {SessionStatus.COMPLETED, SessionStatus.CANCELLED}
                            else None,
                            note.modified_at.isoformat()
                            if note.mapped_status is SessionStatus.ARCHIVED
                            else None,
                            note.session_id,
                        ),
                    )
                connection.execute(
                    """
                    INSERT INTO legacy_note_imports(
                        note_path, content_hash, session_id, source_status,
                        mapped_status, imported_at, last_seen_at
                    ) VALUES (?, ?, ?, ?, ?, ?, ?)
                    ON CONFLICT(note_path) DO UPDATE SET
                        content_hash = excluded.content_hash,
                        session_id = excluded.session_id,
                        source_status = excluded.source_status,
                        mapped_status = excluded.mapped_status,
                        last_seen_at = excluded.last_seen_at
                    """,
                    (
                        note.note_path,
                        note.content_hash,
                        note.session_id,
                        note.source_status,
                        note.mapped_status.value,
                        utc_text(current_time),
                        utc_text(current_time),
                    ),
                )
            connection.execute(
                "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, ?)",
                (
                    "legacy.notes_imported",
                    json.dumps({"count": len(report.notes)}, sort_keys=True),
                    utc_text(current_time),
                ),
            )
        return report

    def archive_notes(
        self, *, now: datetime | None = None, apply: bool = False
    ) -> ArchiveReport:
        current_time = now or datetime.now(UTC)
        candidates = tuple(
            item for item in self.report(now=current_time).notes if item.archive_eligible
        )
        candidate_paths = tuple(item.note_path for item in candidates)
        reserved: set[str] = set()
        planned_manifest: list[ArchiveManifestEntry] = []
        for note in candidates:
            source = self.repo_root / note.note_path
            destination = self._archive_destination(
                source, note.content_hash, reserved=reserved
            )
            reserved.add(str(destination).casefold())
            planned_manifest.append(
                ArchiveManifestEntry(
                    note.note_path,
                    self._relative(destination),
                    note.session_id,
                    note.content_hash,
                    note.content_hash,
                )
            )
        digest = hashlib.sha256(
            json.dumps(
                [
                    (entry.source_path, entry.destination_path, entry.before_hash)
                    for entry in planned_manifest
                ],
                separators=(",", ":"),
            ).encode("utf-8")
        ).hexdigest()[:24]
        run_id = f"legacy-archive-{digest}"
        if not apply:
            return ArchiveReport(run_id, candidate_paths, (), False)
        if not candidates:
            return ArchiveReport(run_id, (), (), True)
        self.archive_root.mkdir(parents=True, exist_ok=True)
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO legacy_archive_runs(
                    run_id, candidates_json, manifest_json, status, created_at, applied_at
                ) VALUES (?, ?, ?, 'planned', ?, NULL)
                ON CONFLICT(run_id) DO UPDATE SET
                    candidates_json = excluded.candidates_json,
                    manifest_json = excluded.manifest_json,
                    status = CASE
                        WHEN legacy_archive_runs.status = 'applied' THEN 'applied'
                        ELSE 'planned'
                    END,
                    created_at = CASE
                        WHEN legacy_archive_runs.status = 'applied'
                        THEN legacy_archive_runs.created_at
                        ELSE excluded.created_at
                    END
                """,
                (
                    run_id,
                    json.dumps(candidate_paths),
                    json.dumps(
                        [item.to_dict() for item in planned_manifest], sort_keys=True
                    ),
                    utc_text(current_time),
                ),
            )
            status = connection.execute(
                "SELECT status FROM legacy_archive_runs WHERE run_id = ?",
                (run_id,),
            ).fetchone()["status"]
            if status == "applied":
                raise CoordinatorError(
                    "legacy_archive_run_already_applied",
                    f"Legacy archive intent was already applied: {run_id}",
                )
        manifest: list[ArchiveManifestEntry] = []
        moved: list[tuple[Path, Path]] = []
        try:
            with self.database.transaction() as connection:
                for note, planned in zip(candidates, planned_manifest, strict=True):
                    self._assert_archive_still_eligible(
                        note, connection=connection, now=current_time
                    )
                    source = self.repo_root / planned.source_path
                    destination = self.repo_root / planned.destination_path
                    destination.parent.mkdir(parents=True, exist_ok=True)
                    if destination.exists():
                        raise CoordinatorError(
                            "legacy_archive_destination_changed",
                            f"Archive destination appeared after planning: {planned.destination_path}",
                        )
                    os.replace(source, destination)
                    moved.append((source, destination))
                    after_hash = self._hash(destination)
                    if after_hash != note.content_hash:
                        raise CoordinatorError(
                            "legacy_archive_hash_mismatch",
                            f"Archived note hash changed: {note.note_path}",
                        )
                    manifest.append(
                        ArchiveManifestEntry(
                            planned.source_path,
                            planned.destination_path,
                            planned.session_id,
                            planned.before_hash,
                            after_hash,
                        )
                    )
                for entry in manifest:
                    connection.execute(
                        """
                        UPDATE sessions
                        SET status = 'archived', status_reason = ?, archived_at = ?, updated_at = ?
                        WHERE session_id = ?
                        """,
                        (
                            "legacy note archived after stale retention",
                            utc_text(current_time),
                            utc_text(current_time),
                            entry.session_id,
                        ),
                    )
                    connection.execute(
                        "UPDATE legacy_note_imports SET archived_path = ? WHERE note_path = ?",
                        (entry.destination_path, entry.source_path),
                    )
                connection.execute(
                    """
                    UPDATE legacy_archive_runs
                    SET manifest_json = ?, status = 'applied', applied_at = ?
                    WHERE run_id = ? AND status = 'planned'
                    """,
                    (
                        json.dumps([item.to_dict() for item in manifest], sort_keys=True),
                        utc_text(current_time),
                        run_id,
                    ),
                )
        except BaseException:
            for source, destination in reversed(moved):
                if destination.exists() and not source.exists():
                    os.replace(destination, source)
            with self.database.transaction() as connection:
                connection.execute(
                    """
                    UPDATE legacy_archive_runs SET status = 'failed'
                    WHERE run_id = ? AND status = 'planned'
                    """,
                    (run_id,),
                )
            raise
        return ArchiveReport(run_id, candidate_paths, tuple(manifest), True)

    def recover_interrupted_archives(self) -> tuple[str, ...]:
        """Restore source notes for durable archive intents that never committed."""
        with self.database.connect() as connection:
            rows = connection.execute(
                """
                SELECT run_id, manifest_json FROM legacy_archive_runs
                WHERE status = 'planned' ORDER BY created_at, run_id
                """
            ).fetchall()
        recovered: list[str] = []
        for row in rows:
            entries = json.loads(row["manifest_json"])
            for raw in reversed(entries):
                source = self._validated_archive_path(
                    str(raw["source_path"]), expected_root=self.session_root
                )
                destination = self._validated_archive_path(
                    str(raw["destination_path"]), expected_root=self.archive_root
                )
                expected_hash = str(raw["before_hash"])
                if source.exists() and not destination.exists():
                    if self._hash(source) != expected_hash:
                        raise CoordinatorError(
                            "legacy_archive_recovery_hash_mismatch",
                            f"Source note changed during recovery: {raw['source_path']}",
                        )
                    continue
                if destination.exists() and not source.exists():
                    if self._hash(destination) != expected_hash:
                        raise CoordinatorError(
                            "legacy_archive_recovery_hash_mismatch",
                            f"Archive note changed during recovery: {raw['destination_path']}",
                        )
                    os.replace(destination, source)
                    continue
                raise CoordinatorError(
                    "legacy_archive_recovery_conflict",
                    f"Cannot safely recover archive entry {raw['source_path']}",
                )
            with self.database.transaction() as connection:
                connection.execute(
                    "UPDATE legacy_archive_runs SET status = 'failed' WHERE run_id = ? AND status = 'planned'",
                    (row["run_id"],),
                )
            recovered.append(row["run_id"])
        return tuple(recovered)

    def legacy_cargo_diagnostics(self) -> tuple[str, ...]:
        found: set[str] = set()
        target_root = self.repo_root / "target"
        if target_root.is_dir():
            for path in target_root.iterdir():
                if path.is_dir() and path.name.casefold().startswith("codex-shared"):
                    found.add(self._relative(path))
        codex_root = self.repo_root / ".codex"
        if codex_root.is_dir():
            legacy_targets = codex_root / "targets"
            if legacy_targets.is_dir():
                for path in legacy_targets.iterdir():
                    found.add(self._relative(path))
            for path in codex_root.rglob("*"):
                name = path.name.casefold()
                if not (
                    ("cargo" in name and "lease" in name)
                    or (
                        path.suffix.casefold() == ".json"
                        and "target" in name
                        and "slot" in name
                    )
                ):
                    continue
                if not path.is_file():
                    continue
                relative = self._relative(path)
                if relative.startswith(".codex/sessions/"):
                    continue
                if relative.startswith(".codex/targets/"):
                    continue
                found.add(relative)
        return tuple(sorted(found, key=str.casefold))

    def _archive_destination(
        self, source: Path, content_hash: str, *, reserved: set[str]
    ) -> Path:
        candidate = self.archive_root / source.name
        if not candidate.exists() and str(candidate).casefold() not in reserved:
            return candidate
        stem = f"{source.stem}-{content_hash[:12]}"
        candidate = self.archive_root / f"{stem}{source.suffix}"
        counter = 2
        while candidate.exists() or str(candidate).casefold() in reserved:
            candidate = self.archive_root / f"{stem}-{counter}{source.suffix}"
            counter += 1
        return candidate

    def _assert_archive_still_eligible(
        self,
        note: LegacyNoteRecord,
        *,
        connection: sqlite3.Connection,
        now: datetime,
    ) -> None:
        source = self.repo_root / note.note_path
        if not source.is_file() or self._hash(source) != note.content_hash:
            raise CoordinatorError(
                "legacy_archive_source_changed",
                f"Legacy note changed after archive planning: {note.note_path}",
            )
        modified_at = datetime.fromtimestamp(source.stat().st_mtime, tz=UTC)
        reasons: list[str] = []
        if note.pid and self.process_alive(note.pid):
            reasons.append("live_pid")
        if now - modified_at <= timedelta(seconds=self.recent_seconds):
            reasons.append("recent_note")
        reasons.extend(
            self._database_activity_reasons(
                connection,
                note.session_id,
                now=now,
            )
        )
        if reasons:
            raise CoordinatorError(
                "legacy_archive_became_active",
                f"Legacy note became active after archive planning: {note.note_path}",
                details={"activity_reasons": sorted(set(reasons))},
            )

    def _validated_archive_path(self, relative: str, *, expected_root: Path) -> Path:
        candidate = (self.repo_root / relative).resolve()
        root = expected_root.resolve()
        if candidate.parent != root:
            raise CoordinatorError(
                "legacy_archive_recovery_path_invalid",
                f"Archive recovery path is outside its expected root: {relative}",
            )
        return candidate

    def _read_note(self, path: Path, *, now: datetime) -> LegacyNoteRecord:
        content = path.read_bytes()
        text = content.decode("utf-8", errors="replace")
        metadata = self._parse_frontmatter(text)
        session_id = str(metadata.get("session") or metadata.get("session_id") or path.stem)
        source_status = self._optional_text(metadata.get("status"))
        pid = self._optional_int(metadata.get("pid"))
        plan_path = self._plan_path(metadata)
        modified_at = datetime.fromtimestamp(path.stat().st_mtime, tz=UTC)
        activity_reasons = self._activity_reasons(
            session_id, pid=pid, modified_at=modified_at, now=now
        )
        alias = STATUS_ALIASES.get((source_status or "").strip().casefold())
        status_reason = None if alias is not None else source_status
        if activity_reasons:
            mapped = SessionStatus.ACTIVE
        elif alias in {SessionStatus.COMPLETED, SessionStatus.CANCELLED, SessionStatus.ARCHIVED}:
            mapped = alias
        elif alias is not None and alias not in {
            SessionStatus.ACTIVE,
            SessionStatus.REGISTERED,
            SessionStatus.WAITING_LEASE,
            SessionStatus.RESOLVING_FAILURE,
            SessionStatus.WAITING_VALIDATION,
            SessionStatus.FINALIZING,
        }:
            mapped = alias
        else:
            mapped = SessionStatus.STALE
        age = now - modified_at
        archive_eligible = (
            mapped in TERMINAL_ARCHIVE_STATUSES
            and not activity_reasons
            and age >= timedelta(hours=self.archive_after_hours)
        )
        return LegacyNoteRecord(
            self._relative(path),
            session_id,
            source_status,
            mapped,
            status_reason,
            plan_path,
            pid,
            modified_at,
            hashlib.sha256(content).hexdigest(),
            activity_reasons,
            archive_eligible,
        )

    def _activity_reasons(
        self,
        session_id: str,
        *,
        pid: int | None,
        modified_at: datetime,
        now: datetime,
    ) -> tuple[str, ...]:
        reasons: list[str] = []
        if pid and self.process_alive(pid):
            reasons.append("live_pid")
        if now - modified_at <= timedelta(seconds=self.recent_seconds):
            reasons.append("recent_note")
        with self.database.connect() as connection:
            reasons.extend(
                self._database_activity_reasons(
                    connection, session_id, now=now
                )
            )
        return tuple(reasons)

    def _database_activity_reasons(
        self,
        connection: sqlite3.Connection,
        session_id: str,
        *,
        now: datetime,
    ) -> list[str]:
        reasons: list[str] = []
        service_session = connection.execute(
            "SELECT status, last_heartbeat_at FROM sessions WHERE session_id = ?",
            (session_id,),
        ).fetchone()
        if service_session is not None and service_session["status"] not in {
            "completed",
            "archived",
            "cancelled",
        }:
            heartbeat = datetime.fromisoformat(service_session["last_heartbeat_at"])
            if now - heartbeat <= timedelta(seconds=self.recent_seconds):
                reasons.append("service_heartbeat")
        if connection.execute(
            "SELECT 1 FROM leases WHERE session_id = ? LIMIT 1", (session_id,)
        ).fetchone():
            reasons.append("active_lease")
        if connection.execute(
            """
            SELECT 1 FROM patches
            WHERE session_id = ? AND status IN ('queued', 'applying', 'needs_rebase')
            LIMIT 1
            """,
            (session_id,),
        ).fetchone():
            reasons.append("pending_patch")
        # Failure priority belongs to the Failure graph. It must never become
        # a synthetic legacy-session heartbeat that prevents stale collection.
        return reasons

    def _active_notes(self) -> list[Path]:
        if not self.session_root.is_dir():
            return []
        return [path for path in self.session_root.glob("*.md") if path.is_file()]

    @staticmethod
    def _parse_frontmatter(text: str) -> dict[str, object]:
        lines = text.splitlines()
        if not lines or lines[0].strip() != "---":
            return LegacyMigrationService._parse_loose_fields(text)
        metadata: dict[str, object] = {}
        current_list: str | None = None
        for line in lines[1:]:
            if line.strip() == "---":
                break
            stripped = line.strip()
            if stripped.startswith("- ") and current_list:
                values = metadata.setdefault(current_list, [])
                if isinstance(values, list):
                    values.append(stripped[2:].strip().strip("\"'"))
                continue
            match = re.match(r"^([A-Za-z0-9_-]+):\s*(.*)$", stripped)
            if not match:
                continue
            key, value = match.groups()
            if value:
                metadata[key] = value.strip().strip("\"'")
                current_list = None
            else:
                metadata[key] = []
                current_list = key
        return metadata

    @staticmethod
    def _parse_loose_fields(text: str) -> dict[str, object]:
        metadata: dict[str, object] = {}
        for key in ("session", "session_id", "status", "pid", "plan_path"):
            match = re.search(rf"(?im)^\s*[-*]?\s*{key}\s*:\s*`?([^`\r\n]+)", text)
            if match:
                metadata[key] = match.group(1).strip()
        return metadata

    @staticmethod
    def _plan_path(metadata: dict[str, object]) -> str | None:
        direct = LegacyMigrationService._optional_text(metadata.get("plan_path"))
        if direct:
            return direct.replace("\\", "/")
        plans = metadata.get("related_plans")
        if isinstance(plans, list):
            normalized = [str(item).replace("\\", "/") for item in plans]
            numbered = [
                item
                for item in normalized
                if re.search(r"/\d{2}[a-z]?-[^/]+\.md$", item, re.IGNORECASE)
            ]
            return (numbered or normalized or [None])[0]
        return None

    def _relative(self, path: Path) -> str:
        return path.resolve().relative_to(self.repo_root).as_posix()

    @staticmethod
    def _hash(path: Path) -> str:
        with path.open("rb") as handle:
            return hashlib.file_digest(handle, "sha256").hexdigest()

    @staticmethod
    def _optional_text(value: object) -> str | None:
        return str(value).strip() if value is not None and str(value).strip() else None

    @staticmethod
    def _optional_int(value: object) -> int | None:
        try:
            return int(str(value)) if value is not None else None
        except ValueError:
            return None

    def _git(self, *arguments: str) -> str:
        return subprocess.run(
            ["git", *arguments],
            cwd=self.repo_root,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()
