"""Explicit, proof-bound bootstrap of abandoned exact-path ownership."""

from __future__ import annotations

import hashlib
import json
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from sqlite3 import Connection, Row

from .baselines import hash_file
from .database import Database
from .leases import LeaseService, lease_paths_overlap
from .models import CoordinatorError, parse_utc, utc_now, utc_text
from .sessions import SessionService


_EXECUTABLE_SESSION_STATUSES = frozenset(
    {
        "registered",
        "active",
        "waiting_lease",
        "resolving_failure",
        "waiting_validation",
        "finalizing",
    }
)


@dataclass(frozen=True, slots=True)
class OwnershipTransferPath:
    path: str
    current_hash: str | None
    baseline_hash: str | None
    source_session_id: str | None
    source_status: str | None
    source_baseline_epoch: int | None
    source_content_hash: str | None
    path_state: str
    eligible: bool
    blocking_reasons: tuple[str, ...]

    def to_dict(self) -> dict[str, object]:
        return {
            "path": self.path,
            "currentHash": self.current_hash,
            "baselineHash": self.baseline_hash,
            "sourceSessionId": self.source_session_id,
            "sourceStatus": self.source_status,
            "sourceBaselineEpoch": self.source_baseline_epoch,
            "sourceContentHash": self.source_content_hash,
            "pathState": self.path_state,
            "eligible": self.eligible,
            "blockingReasons": list(self.blocking_reasons),
        }


@dataclass(frozen=True, slots=True)
class OwnershipTransferPreview:
    fingerprint: str
    target_session_id: str
    baseline_epoch: int
    paths: tuple[OwnershipTransferPath, ...]

    def to_dict(self) -> dict[str, object]:
        return {
            "fingerprint": self.fingerprint,
            "targetSessionId": self.target_session_id,
            "baselineEpoch": self.baseline_epoch,
            "paths": [item.to_dict() for item in self.paths],
        }


@dataclass(frozen=True, slots=True)
class OwnershipTransferResult:
    fingerprint: str
    target_session_id: str
    paths: tuple[str, ...]
    already_applied: bool

    def to_dict(self) -> dict[str, object]:
        return {
            "fingerprint": self.fingerprint,
            "targetSessionId": self.target_session_id,
            "paths": list(self.paths),
            "alreadyApplied": self.already_applied,
        }


class OwnershipTransferService:
    """Transfer abandoned exact paths only through a reviewed, durable preview."""

    def __init__(
        self,
        database: Database,
        repo_root: str | Path,
        leases: LeaseService,
        sessions: SessionService,
    ) -> None:
        self.database = database
        self.repo_root = Path(repo_root).resolve()
        self.leases = leases
        self.sessions = sessions

    def preview(
        self,
        *,
        target_session_id: str,
        paths: tuple[str, ...] | list[str],
        now: datetime | None = None,
    ) -> OwnershipTransferPreview:
        target_session_id = self._text("target_session_id", target_session_id)
        normalized = self._paths(paths)
        current_time = now or utc_now()
        with self.database.connect() as connection:
            self._target_session(connection, target_session_id)
            baseline_epoch = self._baseline_epoch(connection)
            baseline_hashes = self._baseline_hashes(connection, baseline_epoch, normalized)
            attributions = self._attributions(connection, normalized)
            leases = self._live_leases(connection, current_time)
        entries_list: list[OwnershipTransferPath] = []
        for path in normalized:
            path_key = path.casefold()
            entries_list.append(
                self._preview_path(
                    path,
                    baseline_hashes.get(path_key),
                    attributions.get(path_key),
                    leases,
                    target_session_id,
                )
            )
        entries = tuple(entries_list)
        fingerprint = self._fingerprint(target_session_id, baseline_epoch, entries)
        preview = OwnershipTransferPreview(
            fingerprint, target_session_id, baseline_epoch, entries
        )
        with self.database.transaction() as connection:
            self._target_session(connection, target_session_id)
            connection.execute(
                """
                INSERT INTO ownership_transfer_previews(
                    fingerprint, target_session_id, baseline_epoch, candidates_json, created_at
                ) VALUES (?, ?, ?, ?, ?)
                ON CONFLICT(fingerprint) DO NOTHING
                """,
                (
                    fingerprint,
                    target_session_id,
                    baseline_epoch,
                    json.dumps(preview.to_dict(), sort_keys=True),
                    utc_text(current_time),
                ),
            )
        return preview

    def apply(self, fingerprint: str, *, actor: str) -> OwnershipTransferResult:
        fingerprint = self._text("fingerprint", fingerprint)
        actor = self._text("actor", actor)
        with self.database.connect() as connection:
            preview = self._load_preview(connection, fingerprint)
        ineligible = tuple(item.path for item in preview.paths if not item.eligible)
        if ineligible:
            raise CoordinatorError(
                "ownership_transfer_ineligible_paths",
                "Ownership transfer apply requires every reviewed exact path to be eligible",
                details={"fingerprint": fingerprint, "paths": list(ineligible)},
            )
        requested = preview.paths
        with self.database.transaction() as connection:
            already = connection.execute(
                "SELECT display_path FROM ownership_transfers WHERE fingerprint=? ORDER BY path_key",
                (fingerprint,),
            ).fetchall()
            if already:
                if len(already) != len(requested):
                    raise CoordinatorError(
                        "ownership_transfer_partial_apply",
                        "Ownership transfer has an incomplete audit record and requires investigation",
                        details={"fingerprint": fingerprint},
                    )
                return OwnershipTransferResult(
                    fingerprint,
                    preview.target_session_id,
                    tuple(str(row["display_path"]) for row in already),
                    True,
                )

            self._validate_apply_preconditions(connection, preview, requested)
            acquisition = self.leases.acquire_in_connection(
                connection,
                preview.target_session_id,
                tuple(item.path for item in requested),
            )
            if not acquisition.acquired:
                raise CoordinatorError(
                    "ownership_transfer_lease_conflict",
                    "Ownership transfer could not acquire every reviewed exact lease",
                    details={"paths": list(acquisition.conflicts)},
                )
            transferred_paths = tuple(item.path for item in requested)
            self.sessions.extend_write_scope_in_connection(
                connection,
                preview.target_session_id,
                transferred_paths,
                transfer_fingerprint=fingerprint,
            )
            now = utc_text()
            for item in requested:
                path_key = item.path.casefold()
                connection.execute(
                    """
                    INSERT INTO attributions(
                        path_key, display_path, session_id, baseline_epoch, content_hash, attributed_at
                    ) VALUES (?, ?, ?, ?, ?, ?)
                    ON CONFLICT(path_key) DO UPDATE SET
                        display_path=excluded.display_path,
                        session_id=excluded.session_id,
                        baseline_epoch=excluded.baseline_epoch,
                        content_hash=excluded.content_hash,
                        attributed_at=excluded.attributed_at
                    """,
                    (
                        path_key,
                        item.path,
                        preview.target_session_id,
                        preview.baseline_epoch,
                        item.current_hash,
                        now,
                    ),
                )
                connection.execute(
                    """
                    INSERT INTO ownership_transfers(
                        fingerprint, path_key, display_path, target_session_id, source_session_id,
                        baseline_epoch, content_hash, path_state, actor, transferred_at
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                    """,
                    (
                        fingerprint,
                        path_key,
                        item.path,
                        preview.target_session_id,
                        item.source_session_id,
                        preview.baseline_epoch,
                        item.current_hash,
                        item.path_state,
                        actor,
                        now,
                    ),
                )
            connection.execute(
                "UPDATE ownership_transfer_previews SET applied_at=? WHERE fingerprint=?",
                (now, fingerprint),
            )
        return OwnershipTransferResult(
            fingerprint, preview.target_session_id, transferred_paths, False
        )

    def _validate_apply_preconditions(
        self,
        connection: Connection,
        preview: OwnershipTransferPreview,
        requested: tuple[OwnershipTransferPath, ...],
    ) -> None:
        self._target_session(connection, preview.target_session_id)
        if self._baseline_epoch(connection) != preview.baseline_epoch:
            raise CoordinatorError(
                "ownership_transfer_baseline_changed",
                "Ownership transfer baseline changed after preview",
                details={"fingerprint": preview.fingerprint},
            )
        current_time = utc_now()
        paths = tuple(item.path for item in requested)
        baseline_hashes = self._baseline_hashes(connection, preview.baseline_epoch, paths)
        attributions = self._attributions(connection, paths)
        leases = self._live_leases(connection, current_time)
        for item in requested:
            path_key = item.path.casefold()
            current = self._preview_path(
                item.path,
                baseline_hashes.get(path_key),
                attributions.get(path_key),
                leases,
                preview.target_session_id,
            )
            if (
                not current.eligible
                or current.current_hash != item.current_hash
                or current.baseline_hash != item.baseline_hash
                or current.source_session_id != item.source_session_id
                or current.source_status != item.source_status
                or current.source_baseline_epoch != item.source_baseline_epoch
                or current.source_content_hash != item.source_content_hash
                or current.path_state != item.path_state
            ):
                raise CoordinatorError(
                    "ownership_transfer_preview_stale",
                    "Ownership transfer path changed or regained a live owner after preview",
                    details={"fingerprint": preview.fingerprint, "path": item.path},
                )

    def _target_session(self, connection: Connection, session_id: str) -> Row:
        row = connection.execute(
            "SELECT session_id, status, session_role FROM sessions WHERE session_id=?",
            (session_id,),
        ).fetchone()
        if row is None:
            raise CoordinatorError("session_not_found", f"Unknown Session {session_id}")
        if str(row["session_role"] or "primary") != "primary":
            raise CoordinatorError(
                "ownership_transfer_target_not_primary",
                "Ownership transfer target must be an executable primary Session",
                details={"sessionId": session_id},
            )
        if str(row["status"]) not in _EXECUTABLE_SESSION_STATUSES:
            raise CoordinatorError(
                "ownership_transfer_target_not_executable",
                "Ownership transfer target Session is not executable",
                details={"sessionId": session_id, "status": str(row["status"])},
            )
        return row

    @staticmethod
    def _baseline_epoch(connection: Connection) -> int:
        epoch = connection.execute("SELECT MAX(epoch_id) FROM baseline_epochs").fetchone()[0]
        if epoch is None:
            raise CoordinatorError(
                "baseline_not_initialized",
                "Ownership transfer requires an initialized baseline",
            )
        return int(epoch)

    @staticmethod
    def _baseline_hashes(
        connection: Connection, epoch: int, paths: tuple[str, ...]
    ) -> dict[str, str]:
        placeholders = ", ".join("?" for _ in paths)
        rows = connection.execute(
            f"""
            SELECT manifest.key AS path, manifest.value AS content_hash
            FROM baseline_epochs, json_each(baseline_epochs.manifest_json) AS manifest
            WHERE baseline_epochs.epoch_id=? AND manifest.key IN ({placeholders})
            """,
            (epoch, *paths),
        ).fetchall()
        return {str(row["path"]).casefold(): str(row["content_hash"]) for row in rows}

    @staticmethod
    def _attributions(connection: Connection, paths: tuple[str, ...]) -> dict[str, Row]:
        placeholders = ", ".join("?" for _ in paths)
        rows = connection.execute(
            f"""
            SELECT attributions.path_key, attributions.session_id, attributions.baseline_epoch,
                   attributions.content_hash, sessions.status AS source_status
            FROM attributions
            JOIN sessions ON sessions.session_id=attributions.session_id
            WHERE attributions.path_key IN ({placeholders})
            """,
            tuple(path.casefold() for path in paths),
        ).fetchall()
        return {str(row["path_key"]): row for row in rows}

    @staticmethod
    def _live_leases(connection: Connection, now: datetime) -> tuple[Row, ...]:
        rows = connection.execute(
            "SELECT path_key, session_id, expires_at FROM leases"
        ).fetchall()
        return tuple(row for row in rows if now <= parse_utc(str(row["expires_at"])))

    def _preview_path(
        self,
        path: str,
        baseline_hash: str | None,
        attribution: Row | None,
        leases: tuple[Row, ...],
        target_session_id: str,
    ) -> OwnershipTransferPath:
        current_hash = hash_file(self.repo_root / path)
        source_session_id = str(attribution["session_id"]) if attribution else None
        source_status = str(attribution["source_status"]) if attribution else None
        source_baseline_epoch = int(attribution["baseline_epoch"]) if attribution else None
        source_content_hash = (
            str(attribution["content_hash"])
            if attribution is not None and attribution["content_hash"] is not None
            else None
        )
        path_state = (
            "future"
            if current_hash is None and baseline_hash is None and attribution is None
            else "existing"
        )
        archived_clean_handoff = (
            current_hash is not None
            and current_hash == baseline_hash
            and source_status == "archived"
            and source_content_hash == current_hash
        )
        reasons: list[str] = []
        if current_hash is None and path_state != "future":
            reasons.append("path_missing")
        elif (
            current_hash is not None
            and current_hash == baseline_hash
            and not archived_clean_handoff
        ):
            reasons.append("path_matches_baseline")
        if source_session_id == target_session_id:
            reasons.append("path_already_owned_by_target")
        elif source_status in _EXECUTABLE_SESSION_STATUSES:
            reasons.append("source_owner_executable")
        path_key = path.casefold()
        foreign_leases = [
            row
            for row in leases
            if str(row["session_id"]) != target_session_id
            and lease_paths_overlap(str(row["path_key"]), path_key)
        ]
        if foreign_leases:
            reasons.append("live_foreign_lease")
        return OwnershipTransferPath(
            path,
            current_hash,
            baseline_hash,
            source_session_id,
            source_status,
            source_baseline_epoch,
            source_content_hash,
            path_state,
            not reasons,
            tuple(reasons),
        )

    def _paths(self, paths: tuple[str, ...] | list[str]) -> tuple[str, ...]:
        normalized = {
            item.key: item.display
            for item in (self.leases.path_policy.normalize(path) for path in paths)
        }
        if not normalized:
            raise CoordinatorError(
                "ownership_transfer_paths_required",
                "Ownership transfer requires at least one exact repository path",
            )
        return tuple(normalized[key] for key in sorted(normalized))

    @staticmethod
    def _fingerprint(
        target_session_id: str,
        baseline_epoch: int,
        paths: tuple[OwnershipTransferPath, ...],
    ) -> str:
        payload = {
            "targetSessionId": target_session_id,
            "baselineEpoch": baseline_epoch,
            "paths": [item.to_dict() for item in paths],
        }
        encoded = json.dumps(payload, sort_keys=True, separators=(",", ":")).encode("utf-8")
        return hashlib.sha256(encoded).hexdigest()

    @staticmethod
    def _text(name: str, value: str) -> str:
        result = value.strip()
        if not result:
            raise ValueError(f"{name} cannot be empty")
        return result

    @staticmethod
    def _load_preview(connection: Connection, fingerprint: str) -> OwnershipTransferPreview:
        row = connection.execute(
            "SELECT candidates_json FROM ownership_transfer_previews WHERE fingerprint=?",
            (fingerprint,),
        ).fetchone()
        if row is None:
            raise CoordinatorError(
                "ownership_transfer_preview_missing",
                "Ownership transfer apply requires a recorded preview",
                details={"fingerprint": fingerprint},
            )
        payload = json.loads(row["candidates_json"])
        paths = tuple(
            OwnershipTransferPath(
                path=str(item["path"]),
                current_hash=item.get("currentHash"),
                baseline_hash=item.get("baselineHash"),
                source_session_id=item.get("sourceSessionId"),
                source_status=item.get("sourceStatus"),
                source_baseline_epoch=item.get("sourceBaselineEpoch"),
                source_content_hash=item.get("sourceContentHash"),
                path_state=str(
                    item.get("pathState")
                    or (
                        "future"
                        if item.get("currentHash") is None
                        and item.get("baselineHash") is None
                        and item.get("sourceSessionId") is None
                        else "existing"
                    )
                ),
                eligible=bool(item["eligible"]),
                blocking_reasons=tuple(str(reason) for reason in item["blockingReasons"]),
            )
            for item in payload["paths"]
        )
        return OwnershipTransferPreview(
            fingerprint=str(payload["fingerprint"]),
            target_session_id=str(payload["targetSessionId"]),
            baseline_epoch=int(payload["baselineEpoch"]),
            paths=paths,
        )
