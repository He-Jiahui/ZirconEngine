"""Durable receipts for artifacts produced by managed validation-copy runs."""

from __future__ import annotations

import hashlib
import json
import os
import re
import shutil
from dataclasses import dataclass
from pathlib import Path
from sqlite3 import Row

from .database import Database
from .models import CoordinatorError, utc_text


_RECORD_ID = re.compile(r"^[0-9a-f]{32}$")
_SHA256 = re.compile(r"^[0-9a-f]{64}$")
_VIEWER_KIND = "shader-pbr-viewer"
_VIEWER_PACKAGE = "zircon_app"
_VIEWER_BINARY = "zircon_shader_pbr_viewer"


@dataclass(frozen=True, slots=True)
class ManagedArtifactReceipt:
    receipt_id: str
    session_id: str
    job_id: str
    validation_ticket_id: str
    artifact_kind: str
    status: str
    input_manifest_hash: str
    source_manifest_hash: str
    run_id: str | None
    target_relative_path: str | None
    artifact_path: str | None
    sha256: str | None
    byte_length: int | None
    command: tuple[str, ...]
    command_sha256: str | None
    error_code: str | None
    requested_at: str
    completed_at: str | None

    def to_dict(self) -> dict[str, object]:
        return {
            "receiptId": self.receipt_id,
            "sessionId": self.session_id,
            "jobId": self.job_id,
            "validationTicketId": self.validation_ticket_id,
            "artifactKind": self.artifact_kind,
            "status": self.status,
            "inputManifestHash": self.input_manifest_hash,
            "sourceManifestHash": self.source_manifest_hash,
            "runId": self.run_id,
            "targetRelativePath": self.target_relative_path,
            "artifactPath": self.artifact_path,
            "sha256": self.sha256,
            "byteLength": self.byte_length,
            "command": list(self.command),
            "commandSha256": self.command_sha256,
            "errorCode": self.error_code,
            "requestedAt": self.requested_at,
            "completedAt": self.completed_at,
        }


@dataclass(frozen=True, slots=True)
class _ArtifactRequestSnapshot:
    copy_signature: tuple[str, str, str, str]
    ticket_signature: tuple[str, str, str, str]
    input_manifest_hash: str
    source_manifest_hash: str
    source_root: Path
    copy_paths: frozenset[str]
    source_manifest: dict[str, str | None]


class ManagedArtifactReceiptService:
    """Issue one immutable receipt for a closed, server-known artifact kind."""

    def __init__(self, database: Database, artifact_root: str | Path):
        self.database = database
        self.artifact_root = Path(artifact_root).resolve()

    def request(
        self,
        *,
        session_id: str,
        job_id: str,
        validation_ticket_id: str,
        artifact_kind: str,
    ) -> ManagedArtifactReceipt:
        normalized_session = self._require_session_id(session_id)
        normalized_job = self._require_record_id("job_id", job_id)
        normalized_ticket = self._require_record_id(
            "validation_ticket_id", validation_ticket_id
        )
        if artifact_kind != _VIEWER_KIND:
            raise CoordinatorError(
                "managed_artifact_kind_unknown",
                "Managed artifact kind is not allow-listed",
            )
        receipt_id = hashlib.sha256(
            "\n".join(
                (normalized_session, normalized_job, normalized_ticket, artifact_kind)
            ).encode("utf-8")
        ).hexdigest()[:32]
        snapshot = self._request_snapshot(
            normalized_session,
            normalized_job,
            normalized_ticket,
        )
        self._verify_source_binding(snapshot)
        now = utc_text()
        with self.database.transaction() as connection:
            copy = connection.execute(
                """
                SELECT session_id, status, input_manifest_hash, source_root
                FROM validation_copies WHERE job_id=?
                """,
                (normalized_job,),
            ).fetchone()
            if copy is None:
                raise CoordinatorError(
                    "managed_artifact_job_not_found", "Managed validation job was not found"
                )
            ticket = connection.execute(
                """
                SELECT session_id, status, source_manifest_hash, source_manifest_json
                FROM validation_tickets WHERE ticket_id=?
                """,
                (normalized_ticket,),
            ).fetchone()
            if ticket is None:
                raise CoordinatorError(
                    "managed_artifact_ticket_not_found", "Validation ticket was not found"
                )
            if self._copy_signature(copy) != snapshot.copy_signature or self._ticket_signature(
                ticket
            ) != snapshot.ticket_signature:
                raise CoordinatorError(
                    "managed_artifact_request_snapshot_stale",
                    "Artifact receipt inputs changed after source verification",
                )
            existing = connection.execute(
                "SELECT * FROM managed_artifact_receipts WHERE receipt_id=?",
                (receipt_id,),
            ).fetchone()
            if existing is not None:
                return self._from_row(existing)
            connection.execute(
                """
                INSERT INTO managed_artifact_receipts(
                    receipt_id, session_id, job_id, validation_ticket_id,
                    artifact_kind, status, requested_input_manifest_hash,
                    source_manifest_hash, requested_at
                ) VALUES (?, ?, ?, ?, ?, 'pending', ?, ?, ?)
                """,
                (
                    receipt_id,
                    normalized_session,
                    normalized_job,
                    normalized_ticket,
                    artifact_kind,
                    snapshot.input_manifest_hash,
                    snapshot.source_manifest_hash,
                    now,
                ),
            )
            row = connection.execute(
                "SELECT * FROM managed_artifact_receipts WHERE receipt_id=?",
                (receipt_id,),
            ).fetchone()
        assert row is not None
        return self._from_row(row)

    def _request_snapshot(
        self,
        session_id: str,
        job_id: str,
        validation_ticket_id: str,
    ) -> _ArtifactRequestSnapshot:
        with self.database.connect() as connection:
            copy = connection.execute(
                """
                SELECT session_id, status, input_manifest_hash, source_root,
                       manifest_json
                FROM validation_copies WHERE job_id=?
                """,
                (job_id,),
            ).fetchone()
            if copy is None:
                raise CoordinatorError(
                    "managed_artifact_job_not_found",
                    "Managed validation job was not found",
                )
            ticket = connection.execute(
                """
                SELECT session_id, status, source_manifest_hash,
                       source_manifest_json
                FROM validation_tickets WHERE ticket_id=?
                """,
                (validation_ticket_id,),
            ).fetchone()
            if ticket is None:
                raise CoordinatorError(
                    "managed_artifact_ticket_not_found",
                    "Validation ticket was not found",
                )

        copy_signature = self._copy_signature(copy)
        ticket_signature = self._ticket_signature(ticket)
        if copy_signature[0] != session_id or ticket_signature[0] != session_id:
            raise CoordinatorError(
                "managed_artifact_cross_session",
                "Artifact receipt job, ticket, and requester must share one Session",
            )
        if copy_signature[1] != "materialized":
            raise CoordinatorError(
                "managed_artifact_job_not_ready",
                "Artifact receipt requires a materialized validation job",
            )
        if ticket_signature[1] != "passed":
            raise CoordinatorError(
                "managed_artifact_ticket_not_passed",
                "Artifact receipt requires a passed validation ticket",
            )
        input_manifest_hash = self._require_sha256(
            "input_manifest_hash", copy_signature[2]
        )
        source_manifest_hash = self._require_sha256(
            "source_manifest_hash", ticket_signature[2]
        )
        source_manifest = self._source_manifest(
            ticket_signature[3], source_manifest_hash
        )
        return _ArtifactRequestSnapshot(
            copy_signature=copy_signature,
            ticket_signature=ticket_signature,
            input_manifest_hash=input_manifest_hash,
            source_manifest_hash=source_manifest_hash,
            source_root=Path(copy_signature[3]).resolve(),
            copy_paths=self._copy_manifest(copy["manifest_json"]),
            source_manifest=source_manifest,
        )

    def _verify_source_binding(self, snapshot: _ArtifactRequestSnapshot) -> None:
        for relative_path, expected_hash in snapshot.source_manifest.items():
            candidate = snapshot.source_root.joinpath(*relative_path.split("/"))
            try:
                resolved = candidate.resolve()
                resolved.relative_to(snapshot.source_root)
            except (OSError, ValueError) as error:
                raise CoordinatorError(
                    "managed_artifact_source_escape",
                    "Validation ticket source escaped the materialized copy",
                    details={"path": relative_path},
                ) from error
            listed = relative_path in snapshot.copy_paths
            if expected_hash is None:
                if listed or resolved.exists():
                    raise CoordinatorError(
                        "managed_artifact_source_tombstone_mismatch",
                        "Validation ticket tombstone exists in the materialized copy",
                        details={"path": relative_path},
                    )
                continue
            if not listed or not resolved.is_file():
                raise CoordinatorError(
                    "managed_artifact_source_not_in_copy",
                    "Validation ticket source is absent from the materialized copy",
                    details={"path": relative_path},
                )
            actual_hash, _ = self._fingerprint(resolved)
            if actual_hash != expected_hash:
                raise CoordinatorError(
                    "managed_artifact_source_hash_mismatch",
                    "Validation ticket source does not match the materialized copy",
                    details={"path": relative_path},
                )

    @classmethod
    def _copy_manifest(cls, value: object) -> frozenset[str]:
        try:
            decoded = json.loads(str(value))
        except (TypeError, ValueError) as error:
            raise CoordinatorError(
                "managed_artifact_copy_manifest_invalid",
                "Materialized copy manifest is malformed",
            ) from error
        if not isinstance(decoded, list):
            raise CoordinatorError(
                "managed_artifact_copy_manifest_invalid",
                "Materialized copy manifest must be a path list",
            )
        normalized = [cls._manifest_path(path) for path in decoded]
        if len(set(normalized)) != len(normalized):
            raise CoordinatorError(
                "managed_artifact_copy_manifest_invalid",
                "Materialized copy manifest contains duplicate paths",
            )
        return frozenset(normalized)

    @classmethod
    def _source_manifest(
        cls, value: object, expected_hash: str
    ) -> dict[str, str | None]:
        try:
            decoded = json.loads(str(value))
        except (TypeError, ValueError) as error:
            raise CoordinatorError(
                "managed_artifact_source_manifest_invalid",
                "Validation ticket source manifest is malformed",
            ) from error
        if not isinstance(decoded, dict) or not decoded:
            raise CoordinatorError(
                "managed_artifact_source_manifest_invalid",
                "Validation ticket source manifest must be a non-empty object",
            )
        normalized: dict[str, str | None] = {}
        for path, source_hash in decoded.items():
            relative_path = cls._manifest_path(path)
            if source_hash is None:
                normalized[relative_path] = None
            elif isinstance(source_hash, str):
                folded_source_hash = source_hash.casefold()
                if _SHA256.fullmatch(folded_source_hash) is None:
                    raise CoordinatorError(
                        "managed_artifact_source_manifest_invalid",
                        "Validation ticket source hashes must be SHA-256 or null",
                        details={"path": relative_path},
                    )
                normalized[relative_path] = folded_source_hash
            else:
                raise CoordinatorError(
                    "managed_artifact_source_manifest_invalid",
                    "Validation ticket source hashes must be SHA-256 or null",
                    details={"path": relative_path},
                )
        canonical = json.dumps(
            normalized,
            sort_keys=True,
            separators=(",", ":"),
            ensure_ascii=True,
        )
        if hashlib.sha256(canonical.encode("utf-8")).hexdigest() != expected_hash:
            raise CoordinatorError(
                "managed_artifact_source_manifest_invalid",
                "Validation ticket source manifest hash is inconsistent",
            )
        return dict(sorted(normalized.items(), key=lambda item: item[0].casefold()))

    @staticmethod
    def _manifest_path(value: object) -> str:
        if not isinstance(value, str):
            raise CoordinatorError(
                "managed_artifact_source_manifest_invalid",
                "Artifact manifest paths must be strings",
            )
        normalized = value.replace("\\", "/")
        if (
            normalized != value
            or normalized.startswith("/")
            or re.match(r"^[A-Za-z]:", normalized)
            or any(part in {"", ".", ".."} for part in normalized.split("/"))
        ):
            raise CoordinatorError(
                "managed_artifact_source_manifest_invalid",
                "Artifact manifest path is unsafe",
                details={"path": value},
            )
        return normalized

    @staticmethod
    def _copy_signature(row: Row) -> tuple[str, str, str, str]:
        return (
            str(row["session_id"]),
            str(row["status"]),
            str(row["input_manifest_hash"] or ""),
            str(row["source_root"]),
        )

    @staticmethod
    def _ticket_signature(row: Row) -> tuple[str, str, str, str]:
        return (
            str(row["session_id"]),
            str(row["status"]),
            str(row["source_manifest_hash"] or ""),
            str(row["source_manifest_json"]),
        )

    def finalize_run(self, run_id: str) -> ManagedArtifactReceipt | None:
        normalized_run = self._require_record_id("run_id", run_id)
        with self.database.connect() as connection:
            run = connection.execute(
                """
                SELECT runs.*, copies.target_root, copies.input_manifest_hash
                FROM validation_copy_runs AS runs
                JOIN validation_copies AS copies ON copies.job_id=runs.job_id
                WHERE runs.run_id=?
                """,
                (normalized_run,),
            ).fetchone()
            if run is None:
                raise CoordinatorError(
                    "managed_artifact_run_not_found", "Managed validation run was not found"
                )
            pending = connection.execute(
                """
                SELECT * FROM managed_artifact_receipts
                WHERE job_id=? AND status='pending'
                ORDER BY requested_at, receipt_id
                """,
                (str(run["job_id"]),),
            ).fetchall()
            ticket = (
                connection.execute(
                    """
                    SELECT session_id, status, source_manifest_hash
                    FROM validation_tickets WHERE ticket_id=?
                    """,
                    (str(pending[0]["validation_ticket_id"]),),
                ).fetchone()
                if len(pending) == 1
                else None
            )
        if not pending:
            return None
        if len(pending) != 1:
            raise CoordinatorError(
                "managed_artifact_receipt_ambiguous",
                "Managed validation job has more than one pending artifact receipt",
            )
        receipt = self._from_row(pending[0])
        command = self._command(run["command_json"])
        command_json = self._canonical_command(command)
        command_sha256 = hashlib.sha256(command_json.encode("utf-8")).hexdigest()
        if str(run["session_id"]) != receipt.session_id:
            return self._reject(
                receipt.receipt_id,
                normalized_run,
                command_json,
                command_sha256,
                "managed_artifact_cross_session",
            )
        if int(run["exit_code"]) != 0:
            return self._reject(
                receipt.receipt_id,
                normalized_run,
                command_json,
                command_sha256,
                "managed_artifact_build_failed",
            )
        current_manifest = str(run["input_manifest_hash"] or "")
        if current_manifest != receipt.input_manifest_hash:
            return self._reject(
                receipt.receipt_id,
                normalized_run,
                command_json,
                command_sha256,
                "managed_artifact_input_manifest_mismatch",
            )
        if (
            ticket is None
            or str(ticket["session_id"]) != receipt.session_id
            or str(ticket["status"]) != "passed"
            or str(ticket["source_manifest_hash"]).casefold()
            != receipt.source_manifest_hash
        ):
            return self._reject(
                receipt.receipt_id,
                normalized_run,
                command_json,
                command_sha256,
                "managed_artifact_source_manifest_mismatch",
            )
        target_root = Path(str(run["target_root"])).resolve()
        try:
            candidate = self._artifact_path(target_root, command)
        except CoordinatorError as error:
            return self._reject(
                receipt.receipt_id,
                normalized_run,
                command_json,
                command_sha256,
                error.code,
            )
        try:
            artifact = candidate.resolve()
            target_relative = artifact.relative_to(target_root).as_posix()
        except (OSError, ValueError):
            return self._reject(
                receipt.receipt_id,
                normalized_run,
                command_json,
                command_sha256,
                "managed_artifact_target_escape",
            )
        if not artifact.is_file():
            return self._reject(
                receipt.receipt_id,
                normalized_run,
                command_json,
                command_sha256,
                "managed_artifact_missing",
            )
        source_sha256, source_length = self._fingerprint(artifact)
        try:
            durable_path = self._copy_to_durable_store(
                receipt.receipt_id, artifact, source_sha256, source_length
            )
        except CoordinatorError as error:
            return self._reject(
                receipt.receipt_id,
                normalized_run,
                command_json,
                command_sha256,
                error.code,
            )
        completed_at = utc_text()
        try:
            with self.database.transaction() as connection:
                cursor = connection.execute(
                    """
                    UPDATE managed_artifact_receipts
                    SET status='passed', run_id=?, target_relative_path=?,
                        artifact_path=?, sha256=?, byte_length=?, command_json=?,
                        command_sha256=?, error_code=NULL, completed_at=?
                    WHERE receipt_id=? AND status='pending'
                    """,
                    (
                        normalized_run,
                        target_relative,
                        str(durable_path),
                        source_sha256,
                        source_length,
                        command_json,
                        command_sha256,
                        completed_at,
                        receipt.receipt_id,
                    ),
                )
                if cursor.rowcount != 1:
                    raise CoordinatorError(
                        "managed_artifact_receipt_state_changed",
                        "Managed artifact receipt changed during finalization",
                    )
        except BaseException:
            self._discard_new_artifact(durable_path)
            raise
        return self.get(receipt.receipt_id)

    def get(
        self, receipt_id: str, *, session_id: str | None = None
    ) -> ManagedArtifactReceipt:
        normalized_receipt = self._require_record_id("receipt_id", receipt_id)
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT * FROM managed_artifact_receipts WHERE receipt_id=?",
                (normalized_receipt,),
            ).fetchone()
        if row is None:
            raise CoordinatorError(
                "managed_artifact_receipt_not_found", "Managed artifact receipt was not found"
            )
        receipt = self._from_row(row)
        if session_id is not None and receipt.session_id != self._require_session_id(session_id):
            raise CoordinatorError(
                "managed_artifact_cross_session",
                "Managed artifact receipt belongs to another Session",
            )
        if receipt.status == "passed":
            self._verify_durable_artifact(receipt)
        return receipt

    def _reject(
        self,
        receipt_id: str,
        run_id: str,
        command_json: str,
        command_sha256: str,
        error_code: str,
    ) -> ManagedArtifactReceipt:
        with self.database.transaction() as connection:
            cursor = connection.execute(
                """
                UPDATE managed_artifact_receipts
                SET status='rejected', run_id=?, command_json=?, command_sha256=?,
                    error_code=?, completed_at=?
                WHERE receipt_id=? AND status='pending'
                """,
                (
                    run_id,
                    command_json,
                    command_sha256,
                    error_code,
                    utc_text(),
                    receipt_id,
                ),
            )
            if cursor.rowcount != 1:
                raise CoordinatorError(
                    "managed_artifact_receipt_state_changed",
                    "Managed artifact receipt changed during rejection",
                )
            row = connection.execute(
                "SELECT * FROM managed_artifact_receipts WHERE receipt_id=?",
                (receipt_id,),
            ).fetchone()
        assert row is not None
        return self._from_row(row)

    def _artifact_path(self, target_root: Path, command: tuple[str, ...]) -> Path:
        if not command or Path(command[0]).name.casefold() not in {"cargo", "cargo.exe"}:
            raise CoordinatorError(
                "managed_artifact_command_invalid", "Receipt run is not a Cargo command"
            )
        arguments = list(command[1:])
        if arguments and arguments[0].startswith("+"):
            arguments.pop(0)
        if not arguments or arguments.pop(0) != "build":
            raise CoordinatorError(
                "managed_artifact_command_invalid", "Receipt run is not a Cargo build"
            )
        package = self._option(arguments, "-p", "--package")
        binary = self._option(arguments, "--bin")
        if package != _VIEWER_PACKAGE or binary != _VIEWER_BINARY or "--locked" not in arguments:
            raise CoordinatorError(
                "managed_artifact_command_invalid",
                "Receipt run must build the allow-listed viewer package and binary",
            )
        if self._has_option(arguments, "--target", "--target-dir"):
            raise CoordinatorError(
                "managed_artifact_command_invalid",
                "Receipt build may not override the managed target directory or target triple",
            )
        profile = self._option(arguments, "--profile")
        release = "--release" in arguments
        if profile is not None and release:
            raise CoordinatorError(
                "managed_artifact_command_invalid", "Receipt build profile is ambiguous"
            )
        profile = profile or ("release" if release else "debug")
        if profile not in {"debug", "release", "profiling"}:
            raise CoordinatorError(
                "managed_artifact_command_invalid", "Receipt build profile is not allow-listed"
            )
        suffix = ".exe" if os.name == "nt" else ""
        return target_root / profile / f"{_VIEWER_BINARY}{suffix}"

    def _copy_to_durable_store(
        self, receipt_id: str, source: Path, expected_hash: str, expected_length: int
    ) -> Path:
        self.artifact_root.mkdir(parents=True, exist_ok=True)
        if self.artifact_root.resolve() != self.artifact_root:
            raise CoordinatorError(
                "managed_artifact_store_escape", "Managed artifact store is a reparse point"
            )
        receipt_root = self.artifact_root / receipt_id
        try:
            receipt_root.mkdir()
        except FileExistsError as error:
            raise CoordinatorError(
                "managed_artifact_store_collision",
                "Managed artifact receipt directory already exists",
            ) from error
        resolved_root = receipt_root.resolve()
        try:
            resolved_root.relative_to(self.artifact_root)
        except ValueError as error:
            receipt_root.rmdir()
            raise CoordinatorError(
                "managed_artifact_store_escape", "Managed artifact store escaped its root"
            ) from error
        destination = resolved_root / source.name
        temporary = resolved_root / f".{source.name}.tmp"
        try:
            shutil.copyfile(source, temporary)
            copied_hash, copied_length = self._fingerprint(temporary)
            if copied_hash != expected_hash or copied_length != expected_length:
                raise CoordinatorError(
                    "managed_artifact_copy_mismatch",
                    "Durable artifact copy does not match the managed target artifact",
                )
            os.replace(temporary, destination)
        except BaseException:
            if temporary.exists():
                temporary.unlink()
            if destination.exists():
                destination.unlink()
            resolved_root.rmdir()
            raise
        return destination

    def _verify_durable_artifact(self, receipt: ManagedArtifactReceipt) -> None:
        if (
            receipt.artifact_path is None
            or receipt.sha256 is None
            or receipt.byte_length is None
        ):
            raise CoordinatorError(
                "managed_artifact_receipt_incomplete",
                "Passed managed artifact receipt is incomplete",
            )
        path = Path(receipt.artifact_path).resolve()
        try:
            path.relative_to(self.artifact_root)
        except ValueError as error:
            raise CoordinatorError(
                "managed_artifact_receipt_store_escape",
                "Managed artifact receipt escaped the durable store",
            ) from error
        if not path.is_file():
            raise CoordinatorError(
                "managed_artifact_receipt_missing",
                "Managed artifact receipt file is missing",
            )
        sha256, byte_length = self._fingerprint(path)
        if sha256 != receipt.sha256 or byte_length != receipt.byte_length:
            raise CoordinatorError(
                "managed_artifact_receipt_hash_mismatch",
                "Managed artifact receipt file no longer matches its terminal fingerprint",
            )

    @staticmethod
    def _discard_new_artifact(path: Path) -> None:
        if path.exists():
            path.unlink()
        path.parent.rmdir()

    @staticmethod
    def _fingerprint(path: Path) -> tuple[str, int]:
        digest = hashlib.sha256()
        length = 0
        with path.open("rb") as source:
            while block := source.read(1024 * 1024):
                digest.update(block)
                length += len(block)
        return digest.hexdigest(), length

    @staticmethod
    def _option(arguments: list[str], *names: str) -> str | None:
        values: list[str] = []
        for index, argument in enumerate(arguments):
            if argument in names:
                if index + 1 >= len(arguments):
                    raise CoordinatorError(
                        "managed_artifact_command_invalid", "Cargo option value is missing"
                    )
                values.append(arguments[index + 1])
                continue
            for name in names:
                prefix = name + "="
                if argument.startswith(prefix):
                    values.append(argument[len(prefix) :])
        if len(values) > 1:
            raise CoordinatorError(
                "managed_artifact_command_invalid", "Cargo option is repeated"
            )
        return values[0] if values else None

    @classmethod
    def _has_option(cls, arguments: list[str], *names: str) -> bool:
        return cls._option(arguments, *names) is not None

    @staticmethod
    def _canonical_command(command: tuple[str, ...]) -> str:
        return json.dumps(command, ensure_ascii=True, separators=(",", ":"))

    @staticmethod
    def _command(value: object) -> tuple[str, ...]:
        try:
            decoded = json.loads(str(value))
        except (TypeError, ValueError) as error:
            raise CoordinatorError(
                "managed_artifact_command_invalid", "Managed run command is malformed"
            ) from error
        if not isinstance(decoded, list) or not decoded or any(
            not isinstance(item, str) or not item for item in decoded
        ):
            raise CoordinatorError(
                "managed_artifact_command_invalid", "Managed run command is malformed"
            )
        return tuple(decoded)

    @staticmethod
    def _from_row(row: Row) -> ManagedArtifactReceipt:
        raw_command = row["command_json"]
        command = tuple(json.loads(str(raw_command))) if raw_command else ()
        return ManagedArtifactReceipt(
            receipt_id=str(row["receipt_id"]),
            session_id=str(row["session_id"]),
            job_id=str(row["job_id"]),
            validation_ticket_id=str(row["validation_ticket_id"]),
            artifact_kind=str(row["artifact_kind"]),
            status=str(row["status"]),
            input_manifest_hash=str(row["requested_input_manifest_hash"]),
            source_manifest_hash=str(row["source_manifest_hash"]),
            run_id=str(row["run_id"]) if row["run_id"] is not None else None,
            target_relative_path=(
                str(row["target_relative_path"])
                if row["target_relative_path"] is not None
                else None
            ),
            artifact_path=(
                str(row["artifact_path"]) if row["artifact_path"] is not None else None
            ),
            sha256=str(row["sha256"]) if row["sha256"] is not None else None,
            byte_length=(
                int(row["byte_length"]) if row["byte_length"] is not None else None
            ),
            command=command,
            command_sha256=(
                str(row["command_sha256"])
                if row["command_sha256"] is not None
                else None
            ),
            error_code=(
                str(row["error_code"]) if row["error_code"] is not None else None
            ),
            requested_at=str(row["requested_at"]),
            completed_at=(
                str(row["completed_at"]) if row["completed_at"] is not None else None
            ),
        )

    @staticmethod
    def _require_record_id(field: str, value: object) -> str:
        normalized = str(value or "").strip().casefold()
        if _RECORD_ID.fullmatch(normalized) is None:
            raise CoordinatorError(
                "managed_artifact_identity_invalid", f"{field} must be an exact 32-hex ID"
            )
        return normalized

    @staticmethod
    def _require_session_id(value: object) -> str:
        normalized = str(value or "").strip()
        if not normalized or len(normalized) > 200:
            raise CoordinatorError(
                "managed_artifact_identity_invalid", "session_id is invalid"
            )
        return normalized

    @staticmethod
    def _require_sha256(field: str, value: object) -> str:
        normalized = str(value or "").strip().casefold()
        if _SHA256.fullmatch(normalized) is None:
            raise CoordinatorError(
                "managed_artifact_manifest_invalid", f"{field} must be exact SHA-256"
            )
        return normalized
