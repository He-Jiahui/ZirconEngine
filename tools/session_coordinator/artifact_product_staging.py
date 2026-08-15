from __future__ import annotations

import json
import os
import re
import uuid
from dataclasses import dataclass
from pathlib import Path
from sqlite3 import Connection
from typing import Callable

from .cargo_jobs import target_identity
from .database import Database
from .models import CoordinatorError, utc_text
from .processes import process_creation_time, process_is_alive
from .windows_tree_delete import filesystem_identity


_LEASE_ID = re.compile(r"^[0-9a-f]{32}$")
_SUPPORTED_PURPOSES = frozenset({"build-editor"})


@dataclass(frozen=True, slots=True)
class ArtifactProductStagingLease:
    lease_id: str
    purpose: str
    staging_path: str
    final_path: str
    owner_pid: int
    owner_process_creation_time: str
    status: str
    staging_filesystem_identity: str | None
    published_filesystem_identity: str | None
    created_at: str
    publishing_at: str | None
    published_at: str | None
    released_at: str | None

    def to_dict(self) -> dict[str, object]:
        return {
            "leaseId": self.lease_id,
            "purpose": self.purpose,
            "stagingPath": self.staging_path,
            "finalPath": self.final_path,
            "ownerPid": self.owner_pid,
            "ownerProcessCreationTime": self.owner_process_creation_time,
            "status": self.status,
            "stagingFilesystemIdentity": self.staging_filesystem_identity,
            "publishedFilesystemIdentity": self.published_filesystem_identity,
            "createdAt": self.created_at,
            "publishingAt": self.publishing_at,
            "publishedAt": self.published_at,
            "releasedAt": self.released_at,
        }


class ArtifactProductStagingService:
    """Bind an ephemeral product staging tree to its identity-preserving publication."""

    def __init__(
        self,
        database: Database,
        *,
        roots: tuple[Path, ...],
        managed_overlap: Callable[[Connection, str], bool],
    ) -> None:
        self.database = database
        self.roots = roots
        self._managed_overlap = managed_overlap

    def acquire(
        self, purpose: str, *, final_path: str | Path, owner_pid: int
    ) -> ArtifactProductStagingLease:
        if not isinstance(purpose, str) or purpose not in _SUPPORTED_PURPOSES:
            raise CoordinatorError(
                "artifact_product_staging_purpose_invalid",
                "Product staging purpose is not an allowlisted producer",
            )
        owner_identity = self._live_owner_identity(owner_pid)
        final, root = self._resolve_final_path(final_path)
        if final.exists() or final.is_symlink():
            raise CoordinatorError(
                "artifact_product_staging_final_exists",
                "Product staging final path must not already exist",
                details={"finalPath": str(final)},
            )
        if not final.parent.is_dir():
            raise CoordinatorError(
                "artifact_product_staging_parent_missing",
                "Product staging final parent must already exist",
                details={"finalPath": str(final)},
            )
        self.recover()
        lease_id = uuid.uuid4().hex
        staging = root / f"mvp-product-inputs-{purpose}-{lease_id}"
        staging_key = target_identity(staging)
        final_key = target_identity(final)
        created_at = utc_text()
        with self.database.transaction() as connection:
            for path, key in ((staging, staging_key), (final, final_key)):
                if self._managed_overlap(connection, key):
                    raise CoordinatorError(
                        "artifact_product_staging_path_managed",
                        "Product staging path overlaps existing Coordinator-managed work",
                        details={"path": str(path)},
                    )
            connection.execute(
                """INSERT INTO artifact_product_staging_leases(
                       lease_id, purpose, staging_target_key, staging_dir,
                       final_target_key, final_dir, owner_pid,
                       owner_process_creation_time, status, created_at
                   ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'active', ?)""",
                (
                    lease_id,
                    purpose,
                    staging_key,
                    str(staging),
                    final_key,
                    str(final),
                    owner_pid,
                    owner_identity,
                    created_at,
                ),
            )
            self._event(
                connection,
                "artifact.product_staging_acquired",
                lease_id,
                {"purpose": purpose, "stagingPath": str(staging), "finalPath": str(final)},
            )
        return ArtifactProductStagingLease(
            lease_id,
            purpose,
            str(staging),
            str(final),
            owner_pid,
            owner_identity,
            "active",
            None,
            None,
            created_at,
            None,
            None,
            None,
        )

    def begin_publish(
        self, lease_id: str, *, owner_pid: int
    ) -> ArtifactProductStagingLease:
        with self.database.connect() as connection:
            row = self._owned_row(connection, lease_id, owner_pid=owner_pid)
        if row["status"] == "publishing":
            return self._lease(row)
        if row["status"] != "active":
            self._raise_transition(row, "begin publication")
        staging = Path(str(row["staging_dir"]))
        final = Path(str(row["final_dir"]))
        if final.exists() or final.is_symlink():
            raise CoordinatorError(
                "artifact_product_staging_final_exists",
                "Final path appeared before product publication was authorized",
                details={"finalPath": str(final)},
            )
        try:
            identity = filesystem_identity(staging)
        except (FileNotFoundError, NotADirectoryError, OSError) as error:
            raise CoordinatorError(
                "artifact_product_staging_path_unverifiable",
                "Product staging directory could not be sealed before publication",
                details={"stagingPath": str(staging)},
            ) from error
        publishing_at = utc_text()
        with self.database.transaction() as connection:
            self._require_unchanged(connection, row, expected_status="active")
            connection.execute(
                """UPDATE artifact_product_staging_leases
                   SET status='publishing', staging_filesystem_identity=?, publishing_at=?
                   WHERE lease_id=? AND status='active'""",
                (identity, publishing_at, lease_id),
            )
            self._event(
                connection,
                "artifact.product_staging_publish_started",
                lease_id,
                {"filesystemIdentity": identity},
            )
        values = dict(row)
        values.update(
            status="publishing",
            staging_filesystem_identity=identity,
            publishing_at=publishing_at,
        )
        return self._lease(values)

    def complete_publish(
        self, lease_id: str, *, owner_pid: int
    ) -> ArtifactProductStagingLease:
        with self.database.connect() as connection:
            row = self._owned_row(connection, lease_id, owner_pid=owner_pid)
        if row["status"] == "published":
            return self._lease(row)
        if row["status"] != "publishing":
            self._raise_transition(row, "complete publication")
        identity = self._moved_identity(row)
        published_at = utc_text()
        with self.database.transaction() as connection:
            self._require_unchanged(connection, row, expected_status="publishing")
            connection.execute(
                """UPDATE artifact_product_staging_leases
                   SET status='published', published_filesystem_identity=?, published_at=?
                   WHERE lease_id=? AND status='publishing'""",
                (identity, published_at, lease_id),
            )
            self._event(
                connection,
                "artifact.product_staging_published",
                lease_id,
                {"filesystemIdentity": identity, "finalPath": str(row["final_dir"])},
            )
        values = dict(row)
        values.update(
            status="published",
            published_filesystem_identity=identity,
            published_at=published_at,
        )
        return self._lease(values)

    def release(
        self, lease_id: str, *, owner_pid: int
    ) -> ArtifactProductStagingLease:
        with self.database.connect() as connection:
            row = self._owned_row(connection, lease_id, owner_pid=owner_pid)
        if row["status"] == "released":
            return self._lease(row)
        if row["status"] not in {"active", "publishing"}:
            self._raise_transition(row, "release staging")
        existing = [
            str(path)
            for path in (Path(str(row["staging_dir"])), Path(str(row["final_dir"])))
            if path.exists() or path.is_symlink()
        ]
        if existing:
            raise CoordinatorError(
                "artifact_product_staging_path_still_exists",
                "Staging and final paths must be absent before a failed product staging lease is released",
                details={"paths": existing},
            )
        released_at = utc_text()
        with self.database.transaction() as connection:
            self._require_unchanged(
                connection, row, expected_status=str(row["status"])
            )
            connection.execute(
                """UPDATE artifact_product_staging_leases
                   SET status='released', released_at=?
                   WHERE lease_id=? AND status IN ('active', 'publishing')""",
                (released_at, lease_id),
            )
            self._event(
                connection, "artifact.product_staging_released", lease_id, {}
            )
        values = dict(row)
        values.update(status="released", released_at=released_at)
        return self._lease(values)

    def recover(self) -> None:
        with self.database.connect() as connection:
            rows = connection.execute(
                """SELECT * FROM artifact_product_staging_leases
                   WHERE status IN ('active', 'publishing', 'published')"""
            ).fetchall()
        changes: list[tuple[str, str, str | None]] = []
        for row in rows:
            status = str(row["status"])
            if status in {"active", "publishing"} and self.owner_matches(row):
                continue
            if status == "publishing":
                try:
                    identity = self._moved_identity(row)
                except CoordinatorError:
                    changes.append((str(row["lease_id"]), "recovered", None))
                else:
                    changes.append((str(row["lease_id"]), "published", identity))
                continue
            if status == "published" and self._published_identity_matches(row):
                continue
            changes.append((str(row["lease_id"]), "recovered", None))
        if not changes:
            return
        changed_at = utc_text()
        with self.database.transaction() as connection:
            for lease_id, status, identity in changes:
                if status == "published":
                    updated = connection.execute(
                        """UPDATE artifact_product_staging_leases
                           SET status='published', published_filesystem_identity=?,
                               published_at=COALESCE(published_at, ?)
                           WHERE lease_id=? AND status='publishing'""",
                        (identity, changed_at, lease_id),
                    ).rowcount
                    event_type = "artifact.product_staging_publish_recovered"
                else:
                    updated = connection.execute(
                        """UPDATE artifact_product_staging_leases
                           SET status='recovered', released_at=?
                           WHERE lease_id=? AND status IN ('active', 'publishing', 'published')""",
                        (changed_at, lease_id),
                    ).rowcount
                    event_type = "artifact.product_staging_recovered"
                if updated:
                    self._event(connection, event_type, lease_id, {})

    def managed_paths(self) -> tuple[Path, ...]:
        with self.database.connect() as connection:
            rows = connection.execute(
                """SELECT * FROM artifact_product_staging_leases
                   WHERE status IN ('active', 'publishing', 'published')"""
            ).fetchall()
        paths: list[Path] = []
        for row in rows:
            status = str(row["status"])
            if status in {"active", "publishing"} and self.owner_matches(row):
                paths.append(Path(str(row["staging_dir"])))
                if status == "publishing":
                    paths.append(Path(str(row["final_dir"])))
            elif status == "published" and self._published_identity_matches(row):
                paths.append(Path(str(row["final_dir"])))
        return tuple(paths)

    def owner_matches(self, row) -> bool:
        pid = int(row["owner_pid"])
        if not process_is_alive(pid):
            return False
        try:
            return process_creation_time(pid) == str(row["owner_process_creation_time"])
        except (OSError, ValueError):
            return False

    def _resolve_final_path(self, value: str | Path) -> tuple[Path, Path]:
        if not isinstance(value, (str, Path)):
            raise CoordinatorError(
                "artifact_product_staging_final_invalid",
                "Product staging final path must be absolute",
            )
        final = Path(os.path.abspath(value))
        try:
            resolved = final.resolve(strict=False)
        except OSError as error:
            raise CoordinatorError(
                "artifact_product_staging_final_invalid",
                "Product staging final path could not be resolved",
            ) from error
        roots = [
            root
            for root in self.roots
            if root.name.casefold() == "zirconbuilds"
            and resolved != root
            and resolved.is_relative_to(root)
        ]
        if not roots or resolved != final:
            raise CoordinatorError(
                "artifact_product_staging_final_invalid",
                "Product staging final path must resolve below a governed ZirconBuilds root",
                details={"finalPath": str(final)},
            )
        return final, max(roots, key=lambda item: len(item.parts))

    def _live_owner_identity(self, owner_pid: int) -> str:
        if isinstance(owner_pid, bool) or not isinstance(owner_pid, int) or owner_pid <= 0:
            raise CoordinatorError(
                "artifact_product_staging_owner_invalid",
                "Product staging owner PID must be positive",
            )
        if not process_is_alive(owner_pid):
            raise CoordinatorError(
                "artifact_product_staging_owner_not_alive",
                "Product staging owner process is not alive",
                details={"ownerPid": owner_pid},
            )
        try:
            return process_creation_time(owner_pid)
        except OSError as error:
            raise CoordinatorError(
                "artifact_product_staging_owner_not_alive",
                "Product staging owner process identity could not be read",
                details={"ownerPid": owner_pid},
            ) from error

    def _owned_row(self, connection: Connection, lease_id: str, *, owner_pid: int):
        if not isinstance(lease_id, str) or not _LEASE_ID.fullmatch(lease_id):
            raise CoordinatorError(
                "artifact_product_staging_lease_invalid",
                "Product staging lease ID must be 32 lowercase hex digits",
            )
        row = connection.execute(
            "SELECT * FROM artifact_product_staging_leases WHERE lease_id=?",
            (lease_id,),
        ).fetchone()
        if row is None:
            raise CoordinatorError(
                "artifact_product_staging_lease_not_found",
                "Product staging lease does not exist",
            )
        if owner_pid != int(row["owner_pid"]) or not self.owner_matches(row):
            raise CoordinatorError(
                "artifact_product_staging_owner_mismatch",
                "Product staging lease belongs to another process identity",
            )
        return row

    @staticmethod
    def _require_unchanged(
        connection: Connection, snapshot, *, expected_status: str
    ) -> None:
        current = connection.execute(
            "SELECT * FROM artifact_product_staging_leases WHERE lease_id=?",
            (snapshot["lease_id"],),
        ).fetchone()
        if (
            current is None
            or current["status"] != expected_status
            or current["owner_pid"] != snapshot["owner_pid"]
            or current["owner_process_creation_time"]
            != snapshot["owner_process_creation_time"]
            or current["staging_dir"] != snapshot["staging_dir"]
            or current["final_dir"] != snapshot["final_dir"]
        ):
            raise CoordinatorError(
                "artifact_product_staging_changed",
                "Product staging lease changed before its lifecycle update committed",
                details={"leaseId": str(snapshot["lease_id"])},
            )

    @staticmethod
    def _raise_transition(row, operation: str) -> None:
        raise CoordinatorError(
            "artifact_product_staging_transition_invalid",
            f"Cannot {operation} from product staging state {row['status']}",
            details={"leaseId": str(row["lease_id"]), "status": str(row["status"])},
        )

    @staticmethod
    def _lease(row) -> ArtifactProductStagingLease:
        return ArtifactProductStagingLease(
            lease_id=str(row["lease_id"]),
            purpose=str(row["purpose"]),
            staging_path=str(row["staging_dir"]),
            final_path=str(row["final_dir"]),
            owner_pid=int(row["owner_pid"]),
            owner_process_creation_time=str(row["owner_process_creation_time"]),
            status=str(row["status"]),
            staging_filesystem_identity=(
                str(row["staging_filesystem_identity"])
                if row["staging_filesystem_identity"]
                else None
            ),
            published_filesystem_identity=(
                str(row["published_filesystem_identity"])
                if row["published_filesystem_identity"]
                else None
            ),
            created_at=str(row["created_at"]),
            publishing_at=str(row["publishing_at"]) if row["publishing_at"] else None,
            published_at=str(row["published_at"]) if row["published_at"] else None,
            released_at=str(row["released_at"]) if row["released_at"] else None,
        )

    @staticmethod
    def _moved_identity(row) -> str:
        staging = Path(str(row["staging_dir"]))
        final = Path(str(row["final_dir"]))
        if staging.exists() or staging.is_symlink():
            raise CoordinatorError(
                "artifact_product_staging_move_incomplete",
                "Staging path still exists after product publication move",
                details={"stagingPath": str(staging)},
            )
        expected = row["staging_filesystem_identity"]
        if not expected:
            raise CoordinatorError(
                "artifact_product_staging_identity_missing",
                "Product staging publication has no sealed source identity",
            )
        try:
            actual = filesystem_identity(final)
        except (FileNotFoundError, NotADirectoryError, OSError) as error:
            raise CoordinatorError(
                "artifact_product_staging_final_unverifiable",
                "Published product path could not be verified",
                details={"finalPath": str(final)},
            ) from error
        if actual != str(expected):
            raise CoordinatorError(
                "artifact_product_staging_identity_mismatch",
                "Published product path is not the sealed staging directory",
                details={"finalPath": str(final)},
            )
        return actual

    @staticmethod
    def _published_identity_matches(row) -> bool:
        expected = row["published_filesystem_identity"]
        if not expected:
            return False
        try:
            return filesystem_identity(Path(str(row["final_dir"]))) == str(expected)
        except (FileNotFoundError, NotADirectoryError, OSError):
            return False

    @staticmethod
    def _event(
        connection: Connection,
        event_type: str,
        lease_id: str,
        details: dict[str, object],
    ) -> None:
        payload = {"leaseId": lease_id, **details}
        connection.execute(
            "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, ?)",
            (event_type, json.dumps(payload, sort_keys=True), utc_text()),
        )
