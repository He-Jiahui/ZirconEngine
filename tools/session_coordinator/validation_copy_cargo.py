from __future__ import annotations

import hashlib
import json
import os
import platform
import stat
from pathlib import Path
from typing import Callable, Mapping

from .cargo_jobs import CargoCompatibility, CargoJobService, CargoLaneKind
from .cargo_command_policy import (
    cargo_config_file_arguments,
    cargo_target_argument,
    cargo_toolchain_selector,
    is_direct_cargo_command,
)
from .cargo_runner import CargoJobRunner
from .database import Database
from .models import CoordinatorError

_RETRYABLE_ADMISSION_ERRORS = frozenset(
    {
        "cargo_cpu_reservation_not_fifo_head",
        "cargo_cpu_lane_reserved",
        "cargo_lane_occupied",
        "cargo_reuse_pool_busy",
        "cargo_process_tree_alive",
        "cargo_lane_cleanup_reserved",
        "cargo_cpu_burst_occupied",
        "cargo_cpu_burst_resource_denied",
        "cargo_cpu_burst_admission_stale",
        "cargo_cpu_session_reservation_pending",
        "cargo_start_rollover_pending",
    }
)


class ValidationCopyCargoExecution:
    """Bind one materialized validation copy to the managed Cargo lane."""

    def __init__(
        self,
        database: Database,
        cargo_jobs: CargoJobService,
        cargo_runner: CargoJobRunner,
        *,
        preflight: Callable[[], None] | None = None,
        reservation_lookup: Callable[[str, str], Mapping[str, object] | None]
        | None = None,
        source_manifest_lookup: Callable[[str, str], Mapping[str, str] | None]
        | None = None,
        toolchain_lookup: Callable[[str, str], Mapping[str, object] | None]
        | None = None,
    ) -> None:
        self.database = database
        self.cargo_jobs = cargo_jobs
        self.cargo_runner = cargo_runner
        self.preflight = preflight
        self.reservation_lookup = reservation_lookup or self._latest_reservation
        self._source_manifest_required = (
            source_manifest_lookup is not None or reservation_lookup is None
        )
        if source_manifest_lookup is not None:
            self.source_manifest_lookup = source_manifest_lookup
        elif reservation_lookup is None:
            self.source_manifest_lookup = self._validation_source_manifest
        else:
            # Existing state-machine tests inject reservation state without a DB.
            # The real service never replaces only this half of the durable lookup.
            self.source_manifest_lookup = lambda _session_id, _run_id: None
        if toolchain_lookup is not None:
            self.toolchain_lookup = toolchain_lookup
        elif reservation_lookup is None:
            self.toolchain_lookup = self._validation_toolchain
        else:
            self.toolchain_lookup = lambda _session_id, _run_id: None
        self.runtime_toolchain_identity = (
            cargo_runner.toolchain_identity
            if isinstance(cargo_runner, CargoJobRunner)
            else None
        )

    def advance(
        self,
        *,
        session_id: str,
        copy_job_id: str,
        source_root: Path,
        input_manifest_hash: str | None,
        command: tuple[str, ...],
        validation_run_id: str,
    ) -> dict[str, object]:
        source_manifest = self.source_manifest_lookup(session_id, validation_run_id)
        toolchain = self.toolchain_lookup(session_id, validation_run_id)
        if not source_manifest and self._source_manifest_required:
            raise CoordinatorError(
                "validation_copy_cargo_source_manifest_missing",
                "Managed Cargo execution requires the validation ticket source manifest",
            )
        compatibility = self._compatibility(
            copy_job_id=copy_job_id,
            source_root=source_root,
            input_manifest_hash=input_manifest_hash,
            source_manifest=source_manifest,
            command=command,
            validation_run_id=validation_run_id,
            toolchain=toolchain,
        )
        reservation = self.reservation_lookup(session_id, copy_job_id)
        if reservation is None or (
            not reservation.get("jobId")
            and str(reservation.get("status") or "") != "pending"
        ):
            try:
                reservation = self.cargo_jobs.reserve_cpu(
                    session_id,
                    compatibility=compatibility,
                    command=command,
                    ttl_seconds=900,
                )
            except CoordinatorError as error:
                if error.code not in _RETRYABLE_ADMISSION_ERRORS:
                    raise
                details = error.details if isinstance(error.details, Mapping) else {}
                return self._progress(
                    "waiting",
                    reservation_id=str(details.get("reservationId") or ""),
                    blocker=error.code,
                )
        reservation_id = str(reservation.get("reservationId") or "")
        if not reservation_id:
            raise CoordinatorError(
                "validation_copy_cargo_reservation_invalid",
                "Managed Cargo reservation did not return an identity",
            )
        cargo_job_id = str(reservation.get("jobId") or "")
        if not cargo_job_id:
            if self.preflight is not None:
                self.preflight()
            try:
                job = self.cargo_jobs.consume_cpu_reservation(
                    reservation_id,
                    session_id=session_id,
                    lane_kind=CargoLaneKind.WORKSPACE,
                )
            except CoordinatorError as error:
                if error.code not in _RETRYABLE_ADMISSION_ERRORS:
                    raise
                return self._progress(
                    "waiting", reservation_id=reservation_id, blocker=error.code
                )
            cargo_job_id = str(job.job_id)
        else:
            job = self.cargo_jobs.get(cargo_job_id)

        job_status = self._status_value(job.status)
        if job_status == "leased":
            if (
                self._input_manifest_hash_for_source_root(source_root)
                != compatibility.source_copy_manifest_hash
            ):
                raise CoordinatorError(
                    "validation_copy_cargo_manifest_stale",
                    "Materialized Cargo inputs changed after their immutable identity was recorded",
                    details={"copyJobId": copy_job_id},
                )
            try:
                run = self.cargo_runner.start(
                    session_id=session_id,
                    job_id=cargo_job_id,
                    command=command,
                    working_directory=source_root,
                )
            except CoordinatorError as error:
                if error.code not in _RETRYABLE_ADMISSION_ERRORS:
                    raise
                return self._progress(
                    "waiting",
                    reservation_id=reservation_id,
                    cargo_job_id=cargo_job_id,
                    blocker=error.code,
                )
            return self._progress(
                "running",
                reservation_id=reservation_id,
                cargo_job_id=cargo_job_id,
                cargo_run_id=str(run.run_id),
                pid=int(run.pid) if run.pid is not None else None,
            )
        if job_status not in {
            "running",
            "succeeded",
            "failed",
            "released",
            "orphaned",
        }:
            raise CoordinatorError(
                "validation_copy_cargo_job_state_invalid",
                f"Managed Cargo job {cargo_job_id} is {job_status}",
            )

        run = self.cargo_runner.status(cargo_job_id, session_id=session_id)
        run_status = str(run.get("status") or "")
        if run_status == "running":
            return self._progress(
                "running",
                reservation_id=reservation_id,
                cargo_job_id=cargo_job_id,
                cargo_run_id=str(run.get("runId") or ""),
                pid=int(run["pid"]) if run.get("pid") is not None else None,
            )
        if run_status not in {"completed", "finish_blocked", "launch_failed"}:
            raise CoordinatorError(
                "validation_copy_cargo_run_state_invalid",
                f"Managed Cargo run for {cargo_job_id} is {run_status or 'unknown'}",
            )
        exit_code = run.get("exitCode")
        return {
            **self._progress(
                "completed",
                reservation_id=reservation_id,
                cargo_job_id=cargo_job_id,
                cargo_run_id=str(run.get("runId") or ""),
            ),
            "exitCode": int(exit_code) if exit_code is not None else -1,
            "stdoutTail": str(run.get("stdoutTail") or ""),
            "stderrTail": str(run.get("stderrTail") or ""),
            "startedAt": str(run.get("startedAt") or ""),
            "completedAt": str(run.get("completedAt") or ""),
            "cargoRunStatus": run_status,
            "cargoErrorCode": run.get("errorCode"),
        }

    def _latest_reservation(
        self, session_id: str, copy_job_id: str
    ) -> Mapping[str, object] | None:
        with self.database.connect() as connection:
            row = connection.execute(
                """SELECT reservation_id, status, job_id
                   FROM cargo_lane_reservations
                   WHERE lane_scope='cpu' AND session_id=? AND source_copy_job_id=?
                   ORDER BY created_at DESC, reservation_id DESC LIMIT 1""",
                (session_id, copy_job_id),
            ).fetchone()
        if row is None:
            return None
        return {
            "reservationId": str(row["reservation_id"]),
            "status": str(row["status"]),
            "jobId": str(row["job_id"]) if row["job_id"] else None,
        }

    def _validation_source_manifest(
        self, session_id: str, validation_run_id: str
    ) -> Mapping[str, str] | None:
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT source_manifest_json FROM validation_tickets "
                "WHERE ticket_id=? AND session_id=?",
                (validation_run_id, session_id),
            ).fetchone()
        if row is None:
            return None
        try:
            manifest = json.loads(str(row["source_manifest_json"]))
        except (TypeError, ValueError, json.JSONDecodeError) as error:
            raise CoordinatorError(
                "validation_copy_cargo_source_manifest_invalid",
                "Validation ticket source manifest is not valid JSON",
            ) from error
        if not isinstance(manifest, dict) or not manifest:
            raise CoordinatorError(
                "validation_copy_cargo_source_manifest_invalid",
                "Validation ticket source manifest must be a non-empty object",
            )
        return manifest

    def _validation_toolchain(
        self, session_id: str, validation_run_id: str
    ) -> Mapping[str, object] | None:
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT toolchain_json FROM validation_tickets "
                "WHERE ticket_id=? AND session_id=?",
                (validation_run_id, session_id),
            ).fetchone()
        if row is None:
            return None
        try:
            toolchain = json.loads(str(row["toolchain_json"]))
        except (TypeError, ValueError, json.JSONDecodeError) as error:
            raise CoordinatorError(
                "validation_copy_cargo_toolchain_invalid",
                "Validation ticket toolchain identity is not valid JSON",
            ) from error
        if not isinstance(toolchain, dict) or not toolchain:
            raise CoordinatorError(
                "validation_copy_cargo_toolchain_invalid",
                "Validation ticket toolchain identity must be a non-empty object",
            )
        return toolchain

    @staticmethod
    def _input_manifest_hash_for_source_root(source_root: Path) -> str:
        resolved_source = source_root.resolve()
        job_root = resolved_source.parent
        target_root = job_root / "target"
        if resolved_source != job_root / "source":
            raise CoordinatorError(
                "validation_copy_cargo_source_root_invalid",
                "Managed Cargo source root does not match the validation-copy layout",
            )
        entries: dict[str, str] = {}
        pending = [job_root]
        while pending:
            directory = pending.pop()
            try:
                children = tuple(os.scandir(directory))
            except OSError as error:
                raise CoordinatorError(
                    "validation_copy_cargo_manifest_unavailable",
                    "Materialized Cargo inputs could not be inspected",
                    details={"path": str(directory)},
                ) from error
            for child in children:
                path = Path(child.path)
                if path == target_root:
                    continue
                relative = path.relative_to(job_root).as_posix()
                try:
                    current = child.stat(follow_symlinks=False)
                except OSError as error:
                    raise CoordinatorError(
                        "validation_copy_cargo_manifest_unavailable",
                        "Materialized Cargo input changed while it was inspected",
                        details={"path": relative},
                    ) from error
                is_junction = bool(
                    getattr(path, "is_junction", lambda: False)()
                )
                if child.is_symlink() or is_junction:
                    raise CoordinatorError(
                        "validation_copy_manifest_symlink_forbidden",
                        "Validation inputs cannot contain filesystem links",
                        details={"path": relative},
                    )
                if stat.S_ISDIR(current.st_mode):
                    pending.append(path)
                    continue
                if not stat.S_ISREG(current.st_mode):
                    continue
                try:
                    content = path.read_bytes()
                    refreshed = child.stat(follow_symlinks=False)
                except OSError as error:
                    raise CoordinatorError(
                        "validation_copy_cargo_manifest_unavailable",
                        "Materialized Cargo input changed while it was hashed",
                        details={"path": relative},
                    ) from error
                before = (
                    current.st_dev,
                    current.st_ino,
                    current.st_size,
                    current.st_mtime_ns,
                    current.st_ctime_ns,
                )
                after = (
                    refreshed.st_dev,
                    refreshed.st_ino,
                    refreshed.st_size,
                    refreshed.st_mtime_ns,
                    refreshed.st_ctime_ns,
                )
                if before != after:
                    raise CoordinatorError(
                        "validation_copy_cargo_manifest_unavailable",
                        "Materialized Cargo input changed while it was hashed",
                        details={"path": relative},
                    )
                entries[relative] = hashlib.sha256(content).hexdigest()
        payload = [
            {"path": path, "sha256": entries[path]}
            for path in sorted(entries, key=str.casefold)
        ]
        return hashlib.sha256(
            json.dumps(
                payload, sort_keys=True, separators=(",", ":")
            ).encode("utf-8")
        ).hexdigest()

    def _compatibility(
        self,
        *,
        copy_job_id: str,
        source_root: Path,
        input_manifest_hash: str | None,
        source_manifest: Mapping[str, str] | None,
        command: tuple[str, ...],
        validation_run_id: str,
        toolchain: Mapping[str, object] | None = None,
    ) -> CargoCompatibility:
        digest = str(input_manifest_hash or "").strip()
        if len(digest) != 64 or any(character not in "0123456789abcdefABCDEF" for character in digest):
            raise CoordinatorError(
                "validation_copy_cargo_manifest_invalid",
                "Managed Cargo execution requires the immutable copy manifest identity",
            )
        build_config = json.dumps(
            {
                "debug": 0,
                "incremental": False,
                "policy": "managed-validation-v2",
                "cargoConfigFiles": ValidationCopyCargoExecution._config_identity(
                    source_root, command
                ),
            },
            sort_keys=True,
            separators=(",", ":"),
        )
        return CargoCompatibility(
            platform="windows" if os.name == "nt" else "wsl",
            toolchain=ValidationCopyCargoExecution._toolchain_identity(
                source_root,
                command,
                toolchain,
                runtime_identity=(
                    self.runtime_toolchain_identity(command, source_root)
                    if self.runtime_toolchain_identity is not None
                    else None
                ),
            ),
            target_architecture=(
                cargo_target_argument(command) or platform.machine() or "unknown"
            ),
            workspace="validation-copy",
            build_config=build_config,
            source_manifest=source_manifest,
            source_copy_job_id=copy_job_id,
            source_copy_manifest_hash=digest,
        )

    @staticmethod
    def _toolchain_identity(
        source_root: Path,
        command: tuple[str, ...],
        declared: Mapping[str, object] | None = None,
        runtime_identity: str | None = None,
    ) -> str:
        selector = cargo_toolchain_selector(command)
        if selector is not None:
            payload: dict[str, object] = {
                "selector": selector,
                "declared": dict(declared or {}),
                "runtime": runtime_identity,
            }
        elif not is_direct_cargo_command(command):
            payload = {
                "declared": dict(declared or {}),
                "legacyOpaqueCommandSha256": hashlib.sha256(
                    json.dumps(
                        command, separators=(",", ":"), ensure_ascii=True
                    ).encode("utf-8")
                ).hexdigest(),
                "runtime": runtime_identity,
            }
        else:
            files: dict[str, str] = {}
            for name in ("rust-toolchain.toml", "rust-toolchain"):
                path = source_root / name
                try:
                    content = path.read_bytes()
                except FileNotFoundError:
                    continue
                except OSError as error:
                    raise CoordinatorError(
                        "validation_copy_cargo_toolchain_unavailable",
                        "Pinned Rust toolchain identity could not be read",
                        details={"path": str(path)},
                    ) from error
                files[name] = hashlib.sha256(content).hexdigest()
            payload = {
                "selector": "workspace-default",
                "declared": dict(declared or {}),
                "toolchainFiles": files,
                "runtime": runtime_identity,
            }
        return json.dumps(payload, sort_keys=True, separators=(",", ":"))

    @staticmethod
    def _config_identity(
        source_root: Path, command: tuple[str, ...]
    ) -> dict[str, str]:
        candidates = [".cargo/config", ".cargo/config.toml"]
        candidates.extend(cargo_config_file_arguments(command))
        identities: dict[str, str] = {}
        for value in candidates:
            candidate = Path(value)
            if not candidate.is_absolute():
                candidate = source_root / candidate
            candidate = candidate.resolve(strict=False)
            try:
                relative = candidate.relative_to(source_root.resolve()).as_posix()
            except ValueError as error:
                raise CoordinatorError(
                    "validation_copy_cargo_config_unsealed",
                    "Cargo config path escaped the immutable validation source",
                    details={"path": value},
                ) from error
            try:
                content = candidate.read_bytes()
            except FileNotFoundError:
                continue
            except OSError as error:
                raise CoordinatorError(
                    "validation_copy_cargo_config_unavailable",
                    "Cargo config identity could not be read",
                    details={"path": str(candidate)},
                ) from error
            identities[relative] = hashlib.sha256(content).hexdigest()
        return dict(sorted(identities.items(), key=lambda item: item[0].casefold()))

    @staticmethod
    def _status_value(status: object) -> str:
        return str(getattr(status, "value", status))

    @staticmethod
    def _progress(
        status: str,
        *,
        reservation_id: str,
        cargo_job_id: str | None = None,
        cargo_run_id: str | None = None,
        pid: int | None = None,
        blocker: str | None = None,
    ) -> dict[str, object]:
        return {
            "status": status,
            "cargoReservationId": reservation_id,
            "cargoJobId": cargo_job_id,
            "cargoRunId": cargo_run_id,
            "pid": pid,
            "blockerCode": blocker,
        }
