from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime, timedelta
from pathlib import Path

from .baselines import hash_file
from .database import Database
from .models import CoordinatorError, parse_utc, utc_now, utc_text


@dataclass(frozen=True, slots=True)
class NormalizedPath:
    key: str
    display: str
    absolute: Path


class PathPolicy:
    def __init__(self, repo_root: str | Path):
        self.repo_root = Path(repo_root).resolve()

    def normalize(self, value: str | Path) -> NormalizedPath:
        raw = Path(value)
        candidate = raw.resolve() if raw.is_absolute() else (self.repo_root / raw).resolve()
        try:
            relative = candidate.relative_to(self.repo_root)
        except ValueError as error:
            raise CoordinatorError("path_outside_repo", f"Path is outside repository: {value}") from error
        display = relative.as_posix()
        if not display or display == ".":
            raise CoordinatorError("invalid_path", "Repository root cannot be leased as a file")
        if display == ".git" or display.startswith(".git/"):
            raise CoordinatorError("protected_path", "Git internals cannot be leased")
        if display == ".codex/state" or display.startswith(".codex/state/"):
            raise CoordinatorError("protected_path", "Coordinator state cannot be leased")
        return NormalizedPath(display.casefold(), display, candidate)


@dataclass(frozen=True, slots=True)
class LeaseAcquisition:
    acquired: bool
    paths: tuple[str, ...]
    conflicts: tuple[str, ...]


def lease_paths_overlap(left_key: str, right_key: str) -> bool:
    """Return whether two repository-relative lease scopes intersect.

    A Session may lease a directory to own a coherent module subtree, or lease
    one exact file.  Treating only byte-identical keys as conflicting lets a
    second Session claim a child beneath a live directory lease (and vice
    versa), which makes shared-main writes non-exclusive.  Keys are normalized
    repository-relative paths, so a separator-bounded prefix is the required
    hierarchy test; `input` must not overlap an unrelated `input_state`.
    """
    return (
        left_key == right_key
        or left_key.startswith(right_key + "/")
        or right_key.startswith(left_key + "/")
    )


class LeaseService:
    def __init__(
        self,
        database: Database,
        path_policy: PathPolicy,
        *,
        ttl_seconds: int,
        grace_seconds: int,
    ):
        self.database = database
        self.path_policy = path_policy
        self.ttl_seconds = ttl_seconds
        self.grace_seconds = grace_seconds

    def acquire(
        self,
        session_id: str,
        paths: list[str] | tuple[str, ...],
        *,
        now: datetime | None = None,
    ) -> LeaseAcquisition:
        current_time = now or utc_now()
        normalized_by_key = {
            item.key: item for item in (self.path_policy.normalize(path) for path in paths)
        }
        normalized = [normalized_by_key[key] for key in sorted(normalized_by_key)]
        if not normalized:
            raise ValueError("at least one lease path is required")
        with self.database.transaction() as connection:
            session = connection.execute(
                "SELECT status FROM sessions WHERE session_id = ?", (session_id,)
            ).fetchone()
            if session is None:
                raise CoordinatorError("session_not_found", f"Unknown Session {session_id}")
            if session["status"] in {
                "finalizing",
                "completed",
                "stale",
                "archived",
                "cancelled",
            }:
                raise CoordinatorError(
                    "session_not_writable",
                    f"Session {session_id} cannot acquire leases while {session['status']}",
                )
            self._remove_expired(connection, current_time)
            rows = connection.execute("SELECT * FROM leases").fetchall()
            conflicts = tuple(
                sorted(
                    {
                        row["display_path"]
                        for row in rows
                        if row["session_id"] != session_id
                        and any(lease_paths_overlap(row["path_key"], item.key) for item in normalized)
                    },
                    key=str.casefold,
                )
            )
            if conflicts:
                return LeaseAcquisition(False, (), conflicts)
            acquired_at = utc_text(current_time)
            expires_at = utc_text(current_time + timedelta(seconds=self.ttl_seconds))
            for item in normalized:
                connection.execute(
                    """
                    INSERT INTO leases(
                        path_key, display_path, session_id, base_hash,
                        acquired_at, last_heartbeat_at, expires_at
                    ) VALUES (?, ?, ?, ?, ?, ?, ?)
                    ON CONFLICT(path_key) DO UPDATE SET
                        display_path = excluded.display_path,
                        base_hash = COALESCE(leases.base_hash, excluded.base_hash),
                        last_heartbeat_at = excluded.last_heartbeat_at,
                        expires_at = excluded.expires_at
                    WHERE leases.session_id = excluded.session_id
                    """,
                    (
                        item.key,
                        item.display,
                        session_id,
                        hash_file(item.absolute),
                        acquired_at,
                        acquired_at,
                        expires_at,
                    ),
                )
        return LeaseAcquisition(True, tuple(item.display for item in normalized), ())

    def heartbeat(self, session_id: str, *, now: datetime | None = None) -> int:
        current_time = now or utc_now()
        heartbeat_at = utc_text(current_time)
        expires_at = utc_text(current_time + timedelta(seconds=self.ttl_seconds))
        with self.database.transaction() as connection:
            self._remove_expired(connection, current_time)
            cursor = connection.execute(
                """
                UPDATE leases
                SET last_heartbeat_at = ?, expires_at = ?
                WHERE session_id = ?
                """,
                (heartbeat_at, expires_at, session_id),
            )
            return cursor.rowcount

    def release(
        self,
        session_id: str,
        paths: list[str] | tuple[str, ...] | None = None,
    ) -> int:
        with self.database.transaction() as connection:
            if paths is None:
                cursor = connection.execute(
                    "DELETE FROM leases WHERE session_id = ?", (session_id,)
                )
                return cursor.rowcount
            normalized = [self.path_policy.normalize(path) for path in paths]
            if not normalized:
                return 0
            placeholders = ",".join("?" for _ in normalized)
            cursor = connection.execute(
                f"DELETE FROM leases WHERE session_id = ? AND path_key IN ({placeholders})",
                (session_id, *(item.key for item in normalized)),
            )
            return cursor.rowcount

    def owned_paths(self, session_id: str) -> list[str]:
        with self.database.connect() as connection:
            rows = connection.execute(
                "SELECT display_path FROM leases WHERE session_id = ? ORDER BY path_key",
                (session_id,),
            ).fetchall()
        return [row["display_path"] for row in rows]

    def require_owned_live(
        self,
        session_id: str,
        paths: list[str] | tuple[str, ...],
        *,
        error_code: str,
        message: str,
        now: datetime | None = None,
    ) -> None:
        """Require every exact path to retain an unexpired lease for one Session."""
        current_time = now or utc_now()
        normalized = [self.path_policy.normalize(path) for path in paths]
        if not normalized:
            raise ValueError("at least one lease path is required")
        with self.database.connect() as connection:
            rows = connection.execute(
                "SELECT path_key, session_id, expires_at FROM leases"
            ).fetchall()
        missing = [
            item.display
            for item in normalized
            if not any(
                row["session_id"] == session_id
                and (
                    row["path_key"] == item.key
                    or item.key.startswith(row["path_key"] + "/")
                )
                and current_time <= parse_utc(row["expires_at"])
                for row in rows
            )
        ]
        if missing:
            raise CoordinatorError(error_code, message, details={"paths": missing})

    def list(self) -> list[dict[str, str | None]]:
        with self.database.connect() as connection:
            rows = connection.execute("SELECT * FROM leases ORDER BY path_key").fetchall()
        return [dict(row) for row in rows]

    def _remove_expired(self, connection, now: datetime) -> None:
        rows = connection.execute("SELECT path_key, expires_at FROM leases").fetchall()
        expired = [
            row["path_key"]
            for row in rows
            if now > parse_utc(row["expires_at"]) + timedelta(seconds=self.grace_seconds)
        ]
        if expired:
            placeholders = ",".join("?" for _ in expired)
            connection.execute(
                f"DELETE FROM leases WHERE path_key IN ({placeholders})", tuple(expired)
            )
