from __future__ import annotations

import hashlib
import json
import re
import shutil
import threading
import uuid
from contextlib import contextmanager
from dataclasses import dataclass
from datetime import datetime, timedelta
from enum import StrEnum
from pathlib import Path
from sqlite3 import Connection
from typing import Callable, Iterator, Mapping

from .cargo_reservations import (
    NORMAL_CPU_RESERVATION_PRIORITY,
    expire_invalid_pending_cpu_reservations,
    expire_invalid_pending_lane_reservations,
    failure_priority_yield_barrier,
    lane_fifo_head,
    reconcile_cpu_fifo_eligibility,
    reconcile_terminal_finished_lane_reservations,
    require_executable_cargo_session,
)
from .cargo_run_registration import (
    SpawnObservation,
    persist_authorized_spawn_observation,
    persist_authorized_managed_run,
    persist_cleanup_unproven_spawn,
    persist_spawn_authorization,
    mark_managed_run_resumed,
    rollback_spawn_authorization,
)
from .cpu_burst import (
    BURST_TARGET_ROOT,
    CpuBurstRequest,
    CpuBurstSelection,
    is_burst_eligible_cpu_check,
    select_cpu_burst,
)
from .database import Database
from .models import CoordinatorError, parse_utc, utc_now, utc_text
from .processes import (
    live_cargo_process_tree_pids,
    live_process_tree_pids,
    process_creation_time as read_process_creation_time,
    process_is_alive,
)
from .resource_budget import ResourceSample, WindowsResourceProbe, burst_decision


class CargoLaneKind(StrEnum):
    CHECK = "check"
    TEST = "test"
    WORKSPACE = "workspace"
    GPU = "gpu"


class CargoJobStatus(StrEnum):
    LEASED = "leased"
    RUNNING = "running"
    SUCCEEDED = "succeeded"
    FAILED = "failed"
    RELEASED = "released"
    ORPHANED = "orphaned"


class CargoProcessRootKind(StrEnum):
    CARGO = "cargo"
    SUPERVISOR = "supervisor"


class CargoCleanupPolicy(StrEnum):
    RETAINED = "retained"
    DELETE_ON_RELEASE = "delete_on_release"


class CargoCleanupStatus(StrEnum):
    RETAINED = "retained"
    PENDING = "pending"
    DELETED = "deleted"
    FAILED = "failed"


ACTIVE_CARGO_STATUSES = (CargoJobStatus.LEASED.value, CargoJobStatus.RUNNING.value)
ALLOWED_TARGET_ROOT_NAMES = frozenset({"cargo-targets", "targets", "zirconbuilds"})
RECOVERED_RESERVATION_TTL_SECONDS = 900
# Full hard-cutovers can legitimately own a large subtree. Keep an explicit
# bound while allowing the current 1,275-file Sound source manifest to remain
# complete and rechecked at reservation consumption.
MAX_SOURCE_MANIFEST_ENTRIES = 4096
MAX_SOURCE_MANIFEST_BYTES = 256 * 1024
def reservation_scope_for_lane(lane_kind: CargoLaneKind) -> str:
    return "gpu" if lane_kind is CargoLaneKind.GPU else "cpu"


def reservation_code(lane_scope: str, suffix: str) -> str:
    return f"cargo_{lane_scope}_reservation_{suffix}"


def target_identity(value: str | Path) -> str:
    return str(value).replace("/", "\\").casefold()


def targets_overlap(left: str, right: str) -> bool:
    left_key = left.rstrip("\\")
    right_key = right.rstrip("\\")
    return (
        left_key == right_key
        or left_key.startswith(right_key + "\\")
        or right_key.startswith(left_key + "\\")
    )


def overlapping_cleanup_reservation(connection, target_key: str):
    """Return the first cleanup reservation whose tree intersects the target."""
    rows = connection.execute(
        "SELECT target_key, target_dir FROM cleanup_reservations ORDER BY reserved_at"
    ).fetchall()
    return next(
        (row for row in rows if targets_overlap(target_key, row["target_key"])),
        None,
    )


@dataclass(frozen=True, slots=True)
class CargoCompatibility:
    platform: str
    toolchain: str
    target_architecture: str
    workspace: str
    build_config: str
    source_manifest: Mapping[str, str] | None = None
    source_copy_job_id: str | None = None
    source_copy_manifest_hash: str | None = None

    def canonical(self) -> dict[str, object]:
        values = {
            "platform": self.platform.strip().casefold(),
            "toolchain": self.toolchain.strip(),
            "target_architecture": self.target_architecture.strip().casefold(),
            "workspace": self.workspace.strip().replace("\\", "/"),
            "build_config": self.build_config.strip(),
        }
        if values["platform"] not in {"windows", "wsl"}:
            raise CoordinatorError(
                "invalid_cargo_compatibility",
                "Cargo compatibility platform must be windows or wsl",
            )
        for name, value in values.items():
            if not value or len(value) > 4096 or any(char in value for char in ("\0", "\r", "\n")):
                raise CoordinatorError(
                    "invalid_cargo_compatibility",
                    f"Cargo compatibility field {name} is empty or invalid",
                )
        workspace_parts = tuple(part for part in values["workspace"].split("/") if part)
        if (
            not workspace_parts
            or values["workspace"].startswith("/")
            or ":" in workspace_parts[0]
            or any(part in {".", ".."} for part in workspace_parts)
        ):
            raise CoordinatorError(
                "invalid_cargo_compatibility",
                "Cargo compatibility workspace must be a repository-relative path",
            )
        values["workspace"] = "/".join(workspace_parts)
        if self.source_manifest is not None:
            values["source_manifest"] = self._canonical_source_manifest(self.source_manifest)
        copy_job_id = self.source_copy_job_id.strip() if self.source_copy_job_id else None
        copy_hash = (
            self.source_copy_manifest_hash.strip().upper()
            if self.source_copy_manifest_hash
            else None
        )
        if (copy_job_id is None) != (copy_hash is None):
            raise CoordinatorError(
                "invalid_cargo_source_copy",
                "Cargo source copy job id and manifest hash must be supplied together",
            )
        if copy_job_id is not None:
            if re.fullmatch(r"[0-9A-Za-z_-]{1,128}", copy_job_id) is None or re.fullmatch(
                r"[0-9A-F]{64}", copy_hash or ""
            ) is None:
                raise CoordinatorError(
                    "invalid_cargo_source_copy",
                    "Cargo source copy identity is invalid",
                )
            values["source_copy_job_id"] = copy_job_id
            values["source_copy_manifest_hash"] = copy_hash
        return values

    @staticmethod
    def _canonical_source_manifest(source_manifest: Mapping[str, str]) -> dict[str, str]:
        """Normalize the first-class source payload independently of build_config."""
        if not isinstance(source_manifest, Mapping) or not source_manifest:
            raise CoordinatorError(
                "invalid_cargo_source_manifest",
                "Cargo source_manifest must be a non-empty path-to-SHA256 object",
            )
        if len(source_manifest) > MAX_SOURCE_MANIFEST_ENTRIES:
            raise CoordinatorError(
                "invalid_cargo_source_manifest",
                f"Cargo source_manifest exceeds {MAX_SOURCE_MANIFEST_ENTRIES} entries",
            )
        normalized: dict[str, str] = {}
        for raw_path, raw_hash in source_manifest.items():
            if not isinstance(raw_path, str) or not isinstance(raw_hash, str):
                raise CoordinatorError(
                    "invalid_cargo_source_manifest",
                    "Cargo source_manifest entries must contain text paths and SHA-256 values",
                )
            path = raw_path.strip().replace("\\", "/")
            digest = raw_hash.strip().upper()
            if not path or any(char in path for char in ("\0", "\r", "\n")):
                raise CoordinatorError(
                    "invalid_cargo_source_manifest",
                    "Cargo source_manifest paths must be non-empty single-line text",
                )
            if not re.fullmatch(r"[0-9A-F]{64}", digest):
                raise CoordinatorError(
                    "invalid_cargo_source_manifest",
                    "Cargo source_manifest values must be SHA-256 hex digests",
                )
            if path in normalized:
                raise CoordinatorError(
                    "invalid_cargo_source_manifest",
                    "Cargo source_manifest contains the same source file more than once",
                )
            normalized[path] = digest
        canonical = dict(sorted(normalized.items()))
        if len(json.dumps(canonical, sort_keys=True, separators=(",", ":")).encode("utf-8")) > MAX_SOURCE_MANIFEST_BYTES:
            raise CoordinatorError(
                "invalid_cargo_source_manifest",
                f"Cargo source_manifest exceeds {MAX_SOURCE_MANIFEST_BYTES} serialized bytes",
            )
        return canonical


@dataclass(frozen=True, slots=True)
class CargoJob:
    job_id: str
    session_id: str
    lane_kind: CargoLaneKind
    target_dir: str
    status: CargoJobStatus
    dry_run: bool
    pid: int | None
    command: tuple[str, ...]
    exit_code: int | None
    created_at: datetime
    last_heartbeat_at: datetime
    started_at: datetime | None
    finished_at: datetime | None
    released_at: datetime | None
    reuse_key: str | None
    compatibility_json: str | None
    compatibility_key: str | None
    reuse_profile: str | None
    cleanup_policy: CargoCleanupPolicy
    cleanup_status: CargoCleanupStatus
    reused_from_job_id: str | None
    source_copy_job_id: str | None
    source_copy_manifest_hash: str | None
    cleanup_error: str | None
    process_tree_observed_at: datetime | None
    live_process_pids: tuple[int, ...]
    process_tree_exited_at: datetime | None
    root_process_creation_time: str | None
    root_process_kind: CargoProcessRootKind

    def to_dict(self) -> dict[str, object]:
        return {
            "job_id": self.job_id,
            "session_id": self.session_id,
            "lane_kind": self.lane_kind.value,
            "target_dir": self.target_dir,
            "status": self.status.value,
            "dry_run": self.dry_run,
            "pid": self.pid,
            "root_process_creation_time": self.root_process_creation_time,
            "root_process_kind": self.root_process_kind.value,
            "command": list(self.command),
            "exit_code": self.exit_code,
            "created_at": self.created_at.isoformat(),
            "last_heartbeat_at": self.last_heartbeat_at.isoformat(),
            "started_at": self.started_at.isoformat() if self.started_at else None,
            "finished_at": self.finished_at.isoformat() if self.finished_at else None,
            "released_at": self.released_at.isoformat() if self.released_at else None,
            "reuse_key": self.reuse_key,
            "compatibility": (
                json.loads(self.compatibility_json) if self.compatibility_json else None
            ),
            "compatibility_key": self.compatibility_key,
            "reuse_profile": self.reuse_profile,
            "cleanup_policy": self.cleanup_policy.value,
            "cleanup_status": self.cleanup_status.value,
            "reused_from_job_id": self.reused_from_job_id,
            "source_copy_job_id": self.source_copy_job_id,
            "source_copy_manifest_hash": self.source_copy_manifest_hash,
            "cleanup_error": self.cleanup_error,
            "process_tree_observed_at": (
                self.process_tree_observed_at.isoformat()
                if self.process_tree_observed_at
                else None
            ),
            "live_process_pids": list(self.live_process_pids),
            "process_tree_exited_at": (
                self.process_tree_exited_at.isoformat()
                if self.process_tree_exited_at
                else None
            ),
        }


@dataclass(frozen=True, slots=True)
class CargoRunContext:
    environment: dict[str, str]
    working_directory: Path


class TargetPathPolicy:
    def __init__(self, roots: list[str | Path] | tuple[str | Path, ...]):
        resolved: list[Path] = []
        for value in roots:
            root = Path(value).resolve()
            if root.name.casefold() not in ALLOWED_TARGET_ROOT_NAMES:
                raise CoordinatorError(
                    "invalid_target_root",
                    "Managed target root must end in cargo-targets, targets, or ZirconBuilds: "
                    f"{root}",
                )
            resolved.append(root)
        if not resolved:
            raise CoordinatorError("target_root_unavailable", "No managed target root is configured")
        self.roots = tuple(dict.fromkeys(resolved))

    def validate(self, value: str | Path) -> Path:
        candidate = Path(value).resolve()
        for root in self.roots:
            if candidate != root and candidate.is_relative_to(root):
                return candidate
        raise CoordinatorError(
            "cargo_target_not_managed",
            "Cargo target must be below a configured D/E/F cargo-targets, targets, "
            f"or ZirconBuilds root: {candidate}",
        )

    def choose_root(self, free_space: Callable[[Path], int]) -> Path:
        available = [
            root
            for root in self.roots
            if root.parent.exists() or (root.anchor and Path(root.anchor).exists())
        ]
        if not available:
            raise CoordinatorError("target_root_unavailable", "No configured target drive is available")
        return max(available, key=free_space)


class CargoJobService:
    def __init__(
        self,
        database: Database,
        target_policy: TargetPathPolicy,
        *,
        repo_root: str | Path | None = None,
        free_space: Callable[[Path], int] | None = None,
        process_alive: Callable[[int], bool] | None = None,
        process_tree_pids: Callable[[int], tuple[int, ...]] | None = None,
        supervisor_cargo_pids: Callable[[int], tuple[int, ...]] | None = None,
        process_creation_time: Callable[[int], str] | None = None,
        burst_target_root: str | Path = BURST_TARGET_ROOT,
        burst_samples: Callable[[], tuple[ResourceSample, ...]] | None = None,
        admission_guard: Callable[[Connection, str, str], None] | None = None,
        reservation_consume_guard: Callable[[Connection, str, str, str | None], None]
        | None = None,
    ):
        self.database = database
        self.target_policy = target_policy
        self.repo_root = Path(repo_root).resolve() if repo_root is not None else Path.cwd().resolve()
        self.free_space = free_space or (
            lambda path: shutil.disk_usage(path.anchor or path.parent).free
        )
        self.process_alive = process_alive or process_is_alive
        self.process_tree_pids = process_tree_pids or (
            live_process_tree_pids
            if process_alive is None
            else lambda pid: (pid,) if self.process_alive(pid) else ()
        )
        self.supervisor_cargo_pids = supervisor_cargo_pids or (
            live_cargo_process_tree_pids
            if process_alive is None
            else self.process_tree_pids
        )
        self.process_creation_time = process_creation_time or read_process_creation_time
        self.burst_target_root = Path(burst_target_root)
        self.burst_samples = burst_samples or self._sample_burst_resources
        self._admission_guard = admission_guard
        self._reservation_consume_guard = reservation_consume_guard
        self._reported_health_timeouts: set[str] = set()
        self._start_reconcile_lock = threading.RLock()
        self._managed_collectors: set[str] = set()

    @contextmanager
    def managed_start_registration(self) -> Iterator[None]:
        """Keep one managed launch indivisible from local orphan reconciliation."""
        with self._start_reconcile_lock:
            yield

    def register_managed_collector(self, job_id: str) -> None:
        with self._start_reconcile_lock:
            self._managed_collectors.add(job_id)

    def unregister_managed_collector(self, job_id: str) -> None:
        with self._start_reconcile_lock:
            self._managed_collectors.discard(job_id)

    def record_cleanup_unproven_spawn(
        self,
        *,
        run_id: str,
        job_id: str,
        session_id: str,
        command: tuple[str, ...],
        environment: Mapping[str, str],
        stdout_path: Path,
        stderr_path: Path,
        started_at: str,
        pid: int,
        rejection_code: str,
    ) -> CargoJob:
        creation_time = self._read_process_creation_time(pid)
        try:
            live_pids = tuple(sorted({int(value) for value in self.supervisor_cargo_pids(pid)}))
        except (OSError, ValueError):
            live_pids = ()
        persist_cleanup_unproven_spawn(
            self.database,
            run_id=run_id,
            job_id=job_id,
            session_id=session_id,
            command=command,
            environment=environment,
            stdout_path=stdout_path,
            stderr_path=stderr_path,
            started_at=started_at,
            observation=SpawnObservation(
                pid=pid,
                creation_time=creation_time,
                root_kind=CargoProcessRootKind.SUPERVISOR.value,
                live_pids=live_pids,
            ),
            rejection_code=rejection_code,
        )
        return self.get(job_id)

    def set_admission_guard(self, guard: Callable[[Connection, str, str], None]) -> None:
        """Attach the coordinator's durable at-commit admission fence."""
        self._admission_guard = guard

    def set_reservation_consume_guard(
        self, guard: Callable[[Connection, str, str, str | None], None]
    ) -> None:
        """Validate a consumed reservation inside its job-binding transaction."""
        self._reservation_consume_guard = guard

    def _require_admission_checkpoint(
        self, connection: Connection, operation: str, checkpoint: str | None
    ) -> None:
        if checkpoint is not None and self._admission_guard is not None:
            self._admission_guard(connection, operation, checkpoint)

    def _require_reservation_consume_guard(
        self,
        connection: Connection,
        reservation_id: str,
        session_id: str,
        job_id: str | None,
    ) -> None:
        if self._reservation_consume_guard is not None:
            self._reservation_consume_guard(connection, reservation_id, session_id, job_id)

    def reserve_cpu(
        self,
        session_id: str,
        *,
        compatibility: CargoCompatibility,
        target_dir: str | Path | None = None,
        command: list[str] | tuple[str, ...],
        ttl_seconds: int = 900,
        burst_eligible: bool | None = None,
        dependency_lifecycle_key: str | None = None,
        dependency_fixed_sha256: str | None = None,
        admission_checkpoint: str | None = None,
    ) -> dict[str, object]:
        """Reserve the next CPU Cargo lane for one exact managed command.

        ``target_dir`` is optional, but when supplied it is validated before
        the reservation is written and becomes the only target its later
        consume operation may bind.  This supports audited warm-pool reuse
        without letting the caller change target identity after FIFO admission.
        """
        return self._reserve_lane(
            session_id,
            lane_scope="cpu",
            compatibility=compatibility,
            target_dir=target_dir,
            command=command,
            ttl_seconds=ttl_seconds,
            burst_eligible=burst_eligible,
            dependency_lifecycle_key=dependency_lifecycle_key,
            dependency_fixed_sha256=dependency_fixed_sha256,
            admission_checkpoint=admission_checkpoint,
        )

    def reserve_gpu(
        self,
        session_id: str,
        *,
        compatibility: CargoCompatibility,
        target_dir: str | Path,
        command: list[str] | tuple[str, ...],
        ttl_seconds: int = 900,
        admission_checkpoint: str | None = None,
    ) -> dict[str, object]:
        """Reserve the sole GPU lane for one exact managed command."""
        return self._reserve_lane(
            session_id,
            lane_scope="gpu",
            compatibility=compatibility,
            target_dir=target_dir,
            command=command,
            ttl_seconds=ttl_seconds,
            burst_eligible=False,
            dependency_lifecycle_key=None,
            dependency_fixed_sha256=None,
            admission_checkpoint=admission_checkpoint,
        )

    def _reserve_lane(
        self,
        session_id: str,
        *,
        lane_scope: str,
        compatibility: CargoCompatibility,
        target_dir: str | Path | None = None,
        command: list[str] | tuple[str, ...],
        ttl_seconds: int,
        burst_eligible: bool,
        dependency_lifecycle_key: str | None,
        dependency_fixed_sha256: str | None,
        admission_checkpoint: str | None,
    ) -> dict[str, object]:
        if not 30 <= ttl_seconds <= 3600:
            raise CoordinatorError(
                "cargo_reservation_ttl_invalid",
                "Cargo lane reservation TTL must be between 30 and 3600 seconds",
            )
        command_tuple = tuple(str(part) for part in command if str(part))
        if not command_tuple:
            raise CoordinatorError("cargo_reservation_command_empty", "Reservation command cannot be empty")
        self._reject_coordinator_output_flags(command_tuple)
        canonical = compatibility.canonical()
        source_manifest = self._source_manifest_from_compatibility(canonical, lane_scope=lane_scope)
        if canonical.get("source_copy_job_id") is None:
            self._verify_source_manifest(source_manifest, lane_scope=lane_scope)
        compatibility_json = json.dumps(canonical, sort_keys=True, separators=(",", ":"))
        compatibility_key = self._compatibility_fingerprint(compatibility_json)
        canonical_target = (
            str(self.target_policy.validate(target_dir)) if target_dir is not None else None
        )
        # Target-free `cargo check` and targeted library tests can use an
        # isolated, disposable target when the warm lane is busy. Make those
        # safe shapes automatic so callers do not have to remember a
        # throughput-only opt-in. An explicit false remains an operator
        # opt-out, and every other Cargo command stays in the warm FIFO.
        if burst_eligible is None:
            burst_eligible = is_burst_eligible_cpu_check(
                lane_scope=lane_scope,
                burst_eligible=True,
                command=command_tuple,
                target_dir=canonical_target,
            )
        if burst_eligible and not is_burst_eligible_cpu_check(
            lane_scope=lane_scope,
            burst_eligible=burst_eligible,
            command=command_tuple,
            target_dir=canonical_target,
        ):
            raise CoordinatorError(
                "cargo_cpu_burst_eligibility_invalid",
                "Burst eligibility is limited to a target-free CPU cargo check or targeted library test",
            )
        if lane_scope == "gpu" and canonical_target is None:
            raise CoordinatorError(
                "cargo_gpu_reservation_target_required",
                "A GPU reservation requires a coordinator-approved target directory",
            )
        dependency_lifecycle_key = (
            dependency_lifecycle_key.strip() if dependency_lifecycle_key else None
        )
        dependency_fixed_sha256 = (
            dependency_fixed_sha256.strip().upper() if dependency_fixed_sha256 else None
        )
        if lane_scope != "cpu" and (
            dependency_lifecycle_key is not None or dependency_fixed_sha256 is not None
        ):
            raise CoordinatorError(
                "cargo_reservation_dependency_scope_invalid",
                "Only CPU reservations may declare a Failure dependency barrier",
            )
        if dependency_fixed_sha256 is not None and dependency_lifecycle_key is None:
            raise CoordinatorError(
                "cargo_reservation_dependency_invalid",
                "A required fixed digest must name its Failure lifecycle",
            )
        if dependency_fixed_sha256 is not None and re.fullmatch(
            r"[0-9A-F]{64}", dependency_fixed_sha256
        ) is None:
            raise CoordinatorError(
                "cargo_reservation_dependency_invalid",
                "Failure dependency fixed digest must be SHA-256 hex",
            )
        command_fingerprint = self._command_fingerprint(command_tuple)
        now = utc_now()
        now_text = utc_text(now)
        expires_at = utc_text(now + timedelta(seconds=ttl_seconds))
        reservation_id = uuid.uuid4().hex
        with self.database.transaction() as connection:
            self._require_admission_checkpoint(
                connection, f"cargo.reserve_{lane_scope}@{session_id}", admission_checkpoint
            )
            require_executable_cargo_session(connection, session_id)
            source_copy = self._require_source_copy(
                connection,
                session_id=session_id,
                compatibility=canonical,
                source_manifest=source_manifest,
                lane_scope=lane_scope,
            )
            if dependency_lifecycle_key is not None:
                dependency = connection.execute(
                    "SELECT 1 FROM failure_nodes WHERE lifecycle_key=? LIMIT 1",
                    (dependency_lifecycle_key,),
                ).fetchone()
                if dependency is None:
                    raise CoordinatorError(
                        "cargo_cpu_reservation_dependency_not_found",
                        "CPU reservation dependency lifecycle is unknown",
                        details={"lifecycleKey": dependency_lifecycle_key},
                    )
            expire_invalid_pending_lane_reservations(
                connection, lane_scope=lane_scope, now=now_text
            )
            reconcile_terminal_finished_lane_reservations(
                connection, lane_scope=lane_scope, now=now_text
            )
            # Each active Session may hold one durable successor per lane.
            # Multiple distinct Sessions form an exact FIFO queue, which
            # closes release-to-acquire gaps without allowing any entry to
            # consume ahead of the queue head.
            existing_query = (
                """SELECT * FROM cargo_lane_reservations
                   WHERE lane_scope=? AND session_id=? AND status='pending'
                   ORDER BY created_at LIMIT 1"""
                if lane_scope == "cpu"
                else """SELECT * FROM cargo_lane_reservations
                         WHERE lane_scope=? AND status='pending'
                         ORDER BY created_at LIMIT 1"""
            )
            existing_arguments = (lane_scope, session_id) if lane_scope == "cpu" else (lane_scope,)
            existing = connection.execute(existing_query, existing_arguments).fetchone()
            if existing is not None:
                if (
                    existing["session_id"] == session_id
                    and existing["compatibility_key"] == compatibility_key
                    and existing["target_dir"] == canonical_target
                    and existing["dependency_lifecycle_key"] == dependency_lifecycle_key
                    and existing["dependency_fixed_sha256"] == dependency_fixed_sha256
                ):
                    if (
                        existing["command_fingerprint"] != command_fingerprint
                        or bool(existing["burst_eligible"]) != burst_eligible
                    ):
                        connection.execute(
                            """
                            UPDATE cargo_lane_reservations
                            SET command_fingerprint=?, burst_eligible=?, expires_at=?
                            WHERE reservation_id=? AND status='pending'
                            """,
                            (
                                command_fingerprint,
                                int(burst_eligible),
                                expires_at,
                                existing["reservation_id"],
                            ),
                        )
                        self._record_event(
                            connection,
                            session_id,
                            "cargo.reservation_command_corrected",
                            {
                                "reservationId": existing["reservation_id"],
                                "laneScope": lane_scope,
                                "previousCommandFingerprint": existing["command_fingerprint"],
                                "commandFingerprint": command_fingerprint,
                            },
                        )
                        existing = connection.execute(
                            "SELECT * FROM cargo_lane_reservations WHERE reservation_id=?",
                            (existing["reservation_id"],),
                        ).fetchone()
                    return self._reservation_dict(existing)
                if lane_scope == "gpu" and existing["session_id"] != session_id:
                    raise CoordinatorError(
                        "cargo_gpu_lane_reserved",
                        "The next managed GPU lane is reserved for another exact job",
                        details={
                            "sessionId": existing["session_id"],
                            "reservationId": existing["reservation_id"],
                        },
                    )
                raise CoordinatorError(
                    f"cargo_{lane_scope}_session_reservation_pending",
                    f"Session {session_id} already has a pending exact {lane_scope.upper()} reservation",
                    details={"reservationId": existing["reservation_id"]},
                )
            connection.execute(
                """
                INSERT INTO cargo_lane_reservations(
                    reservation_id, session_id, lane_scope, compatibility_key,
                    compatibility_json, target_dir, command_fingerprint, execution_mode,
                    burst_eligible, status, created_at, expires_at,
                    dependency_lifecycle_key, dependency_fixed_sha256,
                    source_copy_job_id
                ) VALUES (?, ?, ?, ?, ?, ?, ?, 'warm', ?, 'pending', ?, ?, ?, ?, ?)
                """,
                (
                    reservation_id,
                    session_id,
                    lane_scope,
                    compatibility_key,
                    compatibility_json,
                    canonical_target,
                    command_fingerprint,
                    int(burst_eligible),
                    now_text,
                    expires_at,
                    dependency_lifecycle_key,
                    dependency_fixed_sha256,
                    source_copy["job_id"] if source_copy is not None else None,
                ),
            )
            row = connection.execute(
                "SELECT * FROM cargo_lane_reservations WHERE reservation_id=?", (reservation_id,)
            ).fetchone()
        return self._reservation_dict(row)

    def release_cpu_reservation(
        self, reservation_id: str, *, session_id: str
    ) -> dict[str, object]:
        """Release a completed or unstarted CPU reservation for the next queued Session."""
        now = utc_text()
        with self.database.transaction() as connection:
            expire_invalid_pending_cpu_reservations(connection, now=now)
            row = connection.execute(
                "SELECT * FROM cargo_lane_reservations WHERE reservation_id=?",
                (reservation_id,),
            ).fetchone()
            if row is None:
                raise CoordinatorError(
                    "cargo_cpu_reservation_not_found",
                    f"Unknown CPU reservation {reservation_id}",
                )
            if row["session_id"] != session_id:
                raise CoordinatorError(
                    "cargo_cpu_reservation_owner_mismatch",
                    f"CPU reservation {reservation_id} belongs to Session {row['session_id']}",
                )
            if row["status"] == "running":
                raise CoordinatorError(
                    "cargo_cpu_reservation_running",
                    "A running CPU reservation cannot be handed off",
                    details={"reservationId": reservation_id, "jobId": row["job_id"]},
                )
            if row["status"] in {"released", "expired"}:
                return self._reservation_dict(row)
            connection.execute(
                """
                UPDATE cargo_lane_reservations
                SET status='released', completed_at=COALESCE(completed_at, ?)
                WHERE reservation_id=?
                """,
                (now, reservation_id),
            )
            row = connection.execute(
                "SELECT * FROM cargo_lane_reservations WHERE reservation_id=?",
                (reservation_id,),
            ).fetchone()
        return self._reservation_dict(row)

    def renew_cpu_reservation(
        self,
        reservation_id: str,
        *,
        session_id: str,
        ttl_seconds: int = 900,
    ) -> dict[str, object]:
        """Extend one pending CPU reservation without changing its FIFO identity."""
        if not 30 <= ttl_seconds <= 3600:
            raise CoordinatorError(
                "cargo_reservation_ttl_invalid",
                "CPU lane reservation TTL must be between 30 and 3600 seconds",
            )
        now = utc_now()
        now_text = utc_text(now)
        expires_at = utc_text(now + timedelta(seconds=ttl_seconds))
        with self.database.transaction() as connection:
            require_executable_cargo_session(connection, session_id)
            expire_invalid_pending_cpu_reservations(connection, now=now_text)
            row = connection.execute(
                "SELECT * FROM cargo_lane_reservations WHERE reservation_id=?",
                (reservation_id,),
            ).fetchone()
            if row is None:
                raise CoordinatorError(
                    "cargo_cpu_reservation_not_found",
                    f"Unknown CPU reservation {reservation_id}",
                )
            if row["session_id"] != session_id:
                raise CoordinatorError(
                    "cargo_cpu_reservation_owner_mismatch",
                    f"CPU reservation {reservation_id} belongs to Session {row['session_id']}",
                )
            if row["status"] != "pending":
                raise CoordinatorError(
                    "cargo_cpu_reservation_not_pending",
                    "Only a pending CPU reservation can be renewed",
                    details={"reservationId": reservation_id, "status": row["status"]},
                )
            connection.execute(
                "UPDATE cargo_lane_reservations SET expires_at=? WHERE reservation_id=?",
                (expires_at, reservation_id),
            )
            row = connection.execute(
                "SELECT * FROM cargo_lane_reservations WHERE reservation_id=?",
                (reservation_id,),
            ).fetchone()
        return self._reservation_dict(row)

    def promote_cpu_reservation_for_failure(
        self,
        reservation_id: str,
        *,
        session_id: str,
        failure_lifecycle_key: str,
        admission_checkpoint: str | None = None,
    ) -> dict[str, object]:
        """Promote one source-bound CPU reservation for its open fixing failure.

        This is deliberately an exception to normal FIFO, not a status-based
        shortcut.  It can only move a still-pending reservation for the exact
        fixing plan of an open failure whose complete ``related_code`` is
        source-bound by the reservation.  Leased and running jobs are never
        reordered or preempted.
        """
        now = utc_text()
        with self.database.transaction() as connection:
            self._require_admission_checkpoint(
                connection,
                "cargo.promote_failure_reservation",
                admission_checkpoint,
            )
            require_executable_cargo_session(connection, session_id)
            expire_invalid_pending_cpu_reservations(connection, now=now)
            reconcile_terminal_finished_lane_reservations(
                connection, lane_scope="cpu", now=now
            )
            reservation = connection.execute(
                """
                SELECT * FROM cargo_lane_reservations
                WHERE reservation_id=? AND lane_scope='cpu'
                """,
                (reservation_id,),
            ).fetchone()
            if reservation is None:
                raise CoordinatorError(
                    "cargo_cpu_reservation_not_found",
                    f"Unknown CPU reservation {reservation_id}",
                )
            if reservation["session_id"] != session_id:
                raise CoordinatorError(
                    "cargo_cpu_reservation_owner_mismatch",
                    f"CPU reservation {reservation_id} belongs to Session {reservation['session_id']}",
                )
            if reservation["status"] != "pending" or reservation["job_id"] is not None:
                raise CoordinatorError(
                    "cargo_cpu_reservation_failure_promotion_unavailable",
                    "Only an unstarted pending CPU reservation may receive failure priority",
                    details={"reservationId": reservation_id, "status": reservation["status"]},
                )
            yield_barrier = failure_priority_yield_barrier(
                connection,
                session_id=session_id,
                failure_lifecycle_key=failure_lifecycle_key,
                created_at=str(reservation["created_at"]),
                reservation_id=str(reservation["reservation_id"]),
            )
            if yield_barrier is not None:
                raise CoordinatorError(
                    "cargo_cpu_reservation_failure_yield_required",
                    "Failure priority must yield once to the older normal CPU reservation",
                    details={
                        "reservationId": reservation_id,
                        "barrierReservationId": yield_barrier["reservation_id"],
                        "priorPriorityReservationId": yield_barrier["prior_priority_reservation_id"],
                    },
                )
            failure = connection.execute(
                """
                SELECT fixing_plan, priority, related_code_json
                FROM failure_nodes
                WHERE lifecycle_key=? AND kind='failure' AND status='open'
                ORDER BY node_id DESC
                LIMIT 1
                """,
                (failure_lifecycle_key,),
            ).fetchone()
            if failure is None:
                raise CoordinatorError(
                    "cargo_cpu_reservation_failure_not_open",
                    "Failure priority requires an open canonical failure node",
                    details={"failureLifecycleKey": failure_lifecycle_key},
                )
            session = connection.execute(
                "SELECT plan_path FROM sessions WHERE session_id=?", (session_id,)
            ).fetchone()
            if session is None or not session["plan_path"]:
                raise CoordinatorError(
                    "cargo_cpu_reservation_failure_plan_mismatch",
                    "Failure priority requires the reservation owner to declare its fixing plan",
                    details={"sessionId": session_id},
                )
            if self._canonical_repo_path(str(session["plan_path"])) != self._canonical_repo_path(
                str(failure["fixing_plan"])
            ):
                raise CoordinatorError(
                    "cargo_cpu_reservation_failure_plan_mismatch",
                    "Failure priority is limited to the canonical fixing-plan owner",
                    details={
                        "sessionId": session_id,
                        "failureLifecycleKey": failure_lifecycle_key,
                    },
                )
            try:
                related_code = json.loads(failure["related_code_json"])
            except (TypeError, json.JSONDecodeError) as error:
                raise CoordinatorError(
                    "cargo_cpu_reservation_failure_related_code_invalid",
                    "Failure priority requires canonical related_code metadata",
                    details={"failureLifecycleKey": failure_lifecycle_key},
                ) from error
            if not isinstance(related_code, list) or not related_code or any(
                not isinstance(path, str) or not path for path in related_code
            ):
                raise CoordinatorError(
                    "cargo_cpu_reservation_failure_related_code_invalid",
                    "Failure priority requires at least one canonical related_code path",
                    details={"failureLifecycleKey": failure_lifecycle_key},
                )
            try:
                compatibility = json.loads(reservation["compatibility_json"])
                source_manifest = self._source_manifest_from_compatibility(
                    compatibility,
                    lane_scope="cpu",
                    reservation_id=reservation_id,
                )
                self._verify_source_manifest(source_manifest, lane_scope="cpu")
                required_paths = {self._canonical_repo_path(path) for path in related_code}
            except CoordinatorError:
                raise
            except (TypeError, ValueError, json.JSONDecodeError) as error:
                raise CoordinatorError(
                    "cargo_cpu_reservation_failure_manifest_mismatch",
                    "Failure priority requires a valid source-bound reservation",
                    details={"reservationId": reservation_id},
                ) from error
            missing_paths = sorted(required_paths.difference(source_manifest))
            promotion_scope = "complete_failure_manifest"
            if missing_paths:
                if not self._is_dependency_lock_preflight(
                    connection,
                    session_id=session_id,
                    compatibility=compatibility,
                    source_manifest=source_manifest,
                    required_paths=required_paths,
                    now=now,
                ):
                    raise CoordinatorError(
                        "cargo_cpu_reservation_failure_manifest_mismatch",
                        "Failure priority requires the reservation source manifest to cover every related path",
                        details={"reservationId": reservation_id, "missingPaths": missing_paths},
                    )
                promotion_scope = "dependency_lock_preflight"
            priority_rank = min(max(int(failure["priority"]), 0), NORMAL_CPU_RESERVATION_PRIORITY)
            connection.execute(
                """
                UPDATE cargo_lane_reservations
                SET priority_rank=?, failure_lifecycle_key=?
                WHERE reservation_id=? AND status='pending' AND job_id IS NULL
                """,
                (priority_rank, failure_lifecycle_key, reservation_id),
            )
            self._record_event(
                connection,
                session_id,
                "cargo.reservation_failure_priority_promoted",
                {
                    "reservationId": reservation_id,
                    "failureLifecycleKey": failure_lifecycle_key,
                    "priorityRank": priority_rank,
                    "relatedCode": sorted(required_paths),
                    "promotionScope": promotion_scope,
                },
            )
            reservation = connection.execute(
                "SELECT * FROM cargo_lane_reservations WHERE reservation_id=?",
                (reservation_id,),
            ).fetchone()
        return self._reservation_dict(reservation)

    def consume_cpu_reservation(
        self,
        reservation_id: str,
        *,
        session_id: str,
        lane_kind: CargoLaneKind,
        admission_checkpoint: str | None = None,
    ) -> CargoJob:
        """Bind one pending CPU reservation to exactly one unstarted managed job.

        The reservation, not the caller, supplies the canonical compatibility
        payload.  ``acquire`` rechecks ownership, FIFO position, expiry and the
        exact compatibility JSON in its single write transaction.
        """
        if lane_kind is CargoLaneKind.GPU:
            raise CoordinatorError(
                "cargo_cpu_reservation_lane_invalid",
                "A CPU reservation cannot create a GPU Cargo job",
            )
        return self._consume_reservation(
            reservation_id,
            session_id=session_id,
            lane_kind=lane_kind,
            lane_scope="cpu",
            admission_checkpoint=admission_checkpoint,
        )

    def consume_gpu_reservation(
        self,
        reservation_id: str,
        *,
        session_id: str,
    ) -> CargoJob:
        """Bind one pending GPU reservation to its sole unstarted job."""
        return self._consume_reservation(
            reservation_id,
            session_id=session_id,
            lane_kind=CargoLaneKind.GPU,
            lane_scope="gpu",
        )

    def recover_expired_reservation(
        self,
        reservation_id: str,
        *,
        job_id: str,
        session_id: str,
    ) -> CargoJob:
        """Restore one orphaned pre-start lease without allocating a new job.

        A daemon handoff may outlive the five-minute pre-start watchdog.  This
        recovery is deliberately narrower than acquire: it accepts only the
        same owner-bound reservation/job pair, no process identity, and no
        command, target or compatibility input from the caller.
        """
        now = utc_now()
        now_text = utc_text(now)
        expires_at = utc_text(now + timedelta(seconds=RECOVERED_RESERVATION_TTL_SECONDS))
        with self.database.transaction() as connection:
            require_executable_cargo_session(connection, session_id)
            reservation = connection.execute(
                "SELECT * FROM cargo_lane_reservations WHERE reservation_id=?",
                (reservation_id,),
            ).fetchone()
            if reservation is None:
                raise CoordinatorError(
                    "cargo_reservation_not_found",
                    f"Unknown Cargo reservation {reservation_id}",
                )
            lane_scope = str(reservation["lane_scope"])
            if reservation["session_id"] != session_id or reservation["job_id"] != job_id:
                raise CoordinatorError(
                    reservation_code(lane_scope, "binding_invalid"),
                    f"{lane_scope.upper()} reservation is not bound to the requested job",
                    details={"reservationId": reservation_id, "jobId": reservation["job_id"]},
                )
            if reservation["status"] != "expired":
                raise CoordinatorError(
                    reservation_code(lane_scope, "recovery_not_allowed"),
                    "Only an expired reservation can be restored without a new acquire",
                    details={"reservationId": reservation_id, "status": reservation["status"]},
                )
            job = connection.execute(
                "SELECT * FROM cargo_jobs WHERE job_id=?", (job_id,)
            ).fetchone()
            if (
                job is None
                or job["session_id"] != session_id
                or job["status"] != CargoJobStatus.ORPHANED.value
                or job["pid"] is not None
                or job["started_at"] is not None
                or job["released_at"] is not None
                or job["command_json"] != "[]"
                or job["process_tree_live_pids_json"] != "[]"
            ):
                raise CoordinatorError(
                    reservation_code(lane_scope, "recovery_not_allowed"),
                    "Recovery requires the same orphaned job to have never started a process",
                    details={"reservationId": reservation_id, "jobId": job_id},
                )
            connection.execute(
                """
                UPDATE cargo_jobs
                SET status='leased', exit_code=NULL, finished_at=NULL,
                    last_heartbeat_at=?, process_tree_observed_at=NULL,
                    process_tree_exited_at=NULL
                WHERE job_id=?
                """,
                (now_text, job_id),
            )
            connection.execute(
                """
                UPDATE cargo_lane_reservations
                SET status='leased', expires_at=?, started_at=NULL, completed_at=NULL
                WHERE reservation_id=? AND status='expired'
                """,
                (expires_at, reservation_id),
            )
            self._record_event(
                connection,
                session_id,
                "cargo.reservation_recovered",
                {
                    "reservationId": reservation_id,
                    "jobId": job_id,
                    "laneScope": lane_scope,
                    "reason": "orphaned_unstarted_lease",
                },
            )
        return self.get(job_id)

    def _consume_reservation(
        self,
        reservation_id: str,
        *,
        session_id: str,
        lane_kind: CargoLaneKind,
        lane_scope: str,
        admission_checkpoint: str | None = None,
    ) -> CargoJob:
        with self.database.connect() as connection:
            require_executable_cargo_session(connection, session_id)
            reservation = connection.execute(
                """SELECT * FROM cargo_lane_reservations
                   WHERE reservation_id=? AND lane_scope=?""",
                (reservation_id, lane_scope),
            ).fetchone()
        if reservation is None:
            raise CoordinatorError(
                reservation_code(lane_scope, "not_found"),
                f"Unknown {lane_scope.upper()} reservation {reservation_id}",
            )
        try:
            payload = json.loads(reservation["compatibility_json"])
            compatibility = CargoCompatibility(**payload)
        except (TypeError, ValueError, json.JSONDecodeError) as error:
            raise CoordinatorError(
                reservation_code(lane_scope, "compatibility_invalid"),
                f"{lane_scope.upper()} reservation has no usable canonical compatibility payload",
                details={"reservationId": reservation_id},
            ) from error
        burst_selection = self._choose_cpu_execution_mode(
            reservation,
            session_id=session_id,
            lane_scope=lane_scope,
        )
        arguments = {
            "compatibility": compatibility,
            "requested_target": reservation["target_dir"],
            "expected_cpu_reservation_id": reservation_id if lane_scope == "cpu" else None,
            "expected_gpu_reservation_id": reservation_id if lane_scope == "gpu" else None,
            "cpu_burst_selection": burst_selection if lane_scope == "cpu" else None,
        }
        return self.acquire(
            session_id,
            lane_kind,
            admission_checkpoint=admission_checkpoint,
            admission_operation=f"cargo.consume_{lane_scope}_reservation@{session_id}",
            **arguments,
        )

    def _sample_burst_resources(self) -> tuple[ResourceSample, ...]:
        probe = WindowsResourceProbe()
        return tuple(probe.sample() for _ in range(3))

    def _choose_cpu_execution_mode(
        self,
        reservation,
        *,
        session_id: str,
        lane_scope: str,
    ) -> CpuBurstSelection:
        if lane_scope != "cpu" or reservation["session_id"] != session_id:
            return CpuBurstSelection("warm", None, "not_eligible")
        if not bool(reservation["burst_eligible"]):
            return CpuBurstSelection("warm", None, "not_eligible")
        with self.database.connect() as connection:
            warm_active = connection.execute(
                """
                SELECT 1 FROM cargo_lane_reservations
                WHERE lane_scope='cpu' AND execution_mode='warm'
                  AND status IN ('leased', 'running')
                LIMIT 1
                """
            ).fetchone()
            burst_active = connection.execute(
                """
                SELECT 1 FROM cargo_lane_reservations
                WHERE lane_scope='cpu' AND execution_mode='burst'
                  AND status IN ('leased', 'running', 'finished')
                LIMIT 1
                """
            ).fetchone()
        if warm_active is None:
            return CpuBurstSelection("warm", None, "not_eligible")
        try:
            target_root = self.target_policy.validate(self.burst_target_root)
            free_bytes = self.free_space(target_root)
        except (OSError, ValueError, CoordinatorError):
            return CpuBurstSelection("warm", None, "disk_headroom")
        try:
            samples = self.burst_samples()
        except (OSError, ValueError):
            return CpuBurstSelection("warm", None, "cpu_headroom")
        decision = burst_decision(
            samples,
            free_bytes=free_bytes,
            burst_active=burst_active is not None,
        )
        return select_cpu_burst(
            CpuBurstRequest(
                reservation_id=str(reservation["reservation_id"]),
                lane_scope=lane_scope,
                burst_eligible=bool(reservation["burst_eligible"]),
                # reserve_cpu admitted burst_eligible only for this exact command
                # shape; the fingerprint is rechecked before the process starts.
                command=("cargo", "check"),
                target_dir=reservation["target_dir"],
            ),
            decision,
            target_root=target_root,
        )

    @staticmethod
    def _admit_cpu_execution_mode(
        connection,
        *,
        reservation_id: str,
        session_id: str,
        selection: CpuBurstSelection,
    ) -> bool:
        """Choose warm or burst while the exact job binding write lock is held."""
        reservation = connection.execute(
            "SELECT * FROM cargo_lane_reservations WHERE reservation_id=? AND lane_scope='cpu'",
            (reservation_id,),
        ).fetchone()
        if (
            reservation is None
            or reservation["session_id"] != session_id
            or reservation["status"] != "pending"
            or not bool(reservation["burst_eligible"])
        ):
            return False
        warm_active = connection.execute(
            """SELECT reservation_id FROM cargo_lane_reservations
               WHERE lane_scope='cpu' AND execution_mode='warm'
                 AND status IN ('leased', 'running', 'finished')
                 AND reservation_id<>? LIMIT 1""",
            (reservation_id,),
        ).fetchone()
        if selection.mode != "burst":
            if warm_active is None:
                return False
            if selection.reason == "burst_active":
                code = "cargo_cpu_burst_occupied"
            elif selection.reason in {
                "disk_headroom",
                "cpu_headroom",
                "memory_headroom",
            }:
                code = "cargo_cpu_burst_resource_denied"
            else:
                code = "cargo_cpu_burst_admission_stale"
            raise CoordinatorError(
                code,
                "CPU warm lane is occupied and isolated burst admission was not accepted",
                details={
                    "reservationId": reservation_id,
                    "reason": selection.reason,
                    "warmReservationId": warm_active["reservation_id"],
                },
            )
        burst_active = connection.execute(
            """SELECT reservation_id FROM cargo_lane_reservations
               WHERE lane_scope='cpu' AND execution_mode='burst'
                 AND status IN ('leased', 'running', 'finished')
                 AND reservation_id<>? LIMIT 1""",
            (reservation_id,),
        ).fetchone()
        if burst_active is not None:
            raise CoordinatorError(
                "cargo_cpu_burst_occupied",
                "The isolated CPU burst lane is already occupied",
                details={
                    "reservationId": reservation_id,
                    "burstReservationId": burst_active["reservation_id"],
                },
            )
        updated = connection.execute(
            """UPDATE cargo_lane_reservations SET execution_mode='burst'
               WHERE reservation_id=? AND lane_scope='cpu' AND status='pending'
                 AND execution_mode='warm' AND burst_eligible=1""",
            (reservation_id,),
        )
        if updated.rowcount != 1:
            raise CoordinatorError(
                "cargo_cpu_burst_admission_stale",
                "CPU reservation changed during atomic execution-mode admission",
                details={"reservationId": reservation_id},
            )
        return True

    def _require_dependency_barrier(
        self, connection, reservation, *, lane_scope: str
    ) -> None:
        lifecycle_key = reservation["dependency_lifecycle_key"]
        if lane_scope != "cpu" or not lifecycle_key:
            return
        fixed = connection.execute(
            """SELECT artifact_path FROM failure_nodes
               WHERE lifecycle_key=? AND kind='fixed' AND status='fixed'
               ORDER BY resolved_at DESC, node_id DESC LIMIT 1""",
            (lifecycle_key,),
        ).fetchone()
        if fixed is None:
            raise CoordinatorError(
                "cargo_cpu_reservation_dependency_pending",
                "CPU reservation is waiting for its required Failure fixed return",
                details={
                    "reservationId": reservation["reservation_id"],
                    "failureLifecycleKey": lifecycle_key,
                },
            )
        artifact = (self.repo_root / str(fixed["artifact_path"])).resolve()
        if not artifact.is_relative_to(self.repo_root) or not artifact.is_file():
            raise CoordinatorError(
                "cargo_cpu_reservation_dependency_fixed_missing",
                "CPU reservation fixed return is not a repository file",
                details={"failureLifecycleKey": lifecycle_key},
            )
        required_digest = reservation["dependency_fixed_sha256"]
        if required_digest:
            actual_digest = hashlib.sha256(artifact.read_bytes()).hexdigest().upper()
            if actual_digest != required_digest:
                raise CoordinatorError(
                    "cargo_cpu_reservation_dependency_fixed_digest_mismatch",
                    "CPU reservation fixed return does not match its required SHA-256",
                    details={
                        "reservationId": reservation["reservation_id"],
                        "failureLifecycleKey": lifecycle_key,
                        "expectedSha256": required_digest,
                        "actualSha256": actual_digest,
                    },
                )

    def reserved_run_environment(
        self,
        reservation_id: str,
        *,
        session_id: str,
        job_id: str,
        command: list[str] | tuple[str, ...],
    ) -> dict[str, str]:
        """Validate the reserved command before the daemon creates its child process."""
        command_tuple = tuple(str(part) for part in command if str(part))
        if not command_tuple:
            raise CoordinatorError("cargo_run_command_empty", "Managed Cargo command cannot be empty")
        self._reject_coordinator_output_flags(command_tuple)
        with self.database.connect() as connection:
            require_executable_cargo_session(connection, session_id)
            reservation = connection.execute(
                "SELECT * FROM cargo_lane_reservations WHERE reservation_id=?",
                (reservation_id,),
            ).fetchone()
            if reservation is None:
                raise CoordinatorError(
                    "cargo_reservation_not_found",
                    f"Unknown Cargo reservation {reservation_id}",
                )
            lane_scope = str(reservation["lane_scope"])
            hold = connection.execute(
                "SELECT 1 FROM service_recovery_state WHERE maintenance_hold=1 LIMIT 1"
            ).fetchone()
            if hold is not None and lane_scope != "cpu":
                raise CoordinatorError(
                    "maintenance_hold_cpu_reservation_required",
                    "A proof-bound maintenance hold may run only its existing CPU reservation",
                    details={
                        "reservationId": reservation_id,
                        "laneScope": lane_scope,
                    },
                )
            if reservation["session_id"] != session_id:
                raise CoordinatorError(
                    reservation_code(lane_scope, "owner_mismatch"),
                    f"{lane_scope.upper()} reservation {reservation_id} belongs to Session {reservation['session_id']}",
                )
            if reservation["status"] != "leased" or reservation["job_id"] != job_id:
                raise CoordinatorError(
                    reservation_code(lane_scope, "binding_invalid"),
                    f"{lane_scope.upper()} reservation is not bound to the requested leased job",
                    details={"reservationId": reservation_id, "jobId": reservation["job_id"]},
                )
            self._require_dependency_barrier(connection, reservation, lane_scope=lane_scope)
            job = connection.execute(
                "SELECT status, session_id FROM cargo_jobs WHERE job_id=?",
                (job_id,),
            ).fetchone()
        if job is None or job["session_id"] != session_id or job["status"] != CargoJobStatus.LEASED.value:
            raise CoordinatorError(
                reservation_code(lane_scope, "binding_invalid"),
                f"{lane_scope.upper()} reservation job is not available for a managed start",
                details={"reservationId": reservation_id, "jobId": job_id},
            )
        if reservation["command_fingerprint"] != self._command_fingerprint(command_tuple):
            raise CoordinatorError(
                reservation_code(lane_scope, "command_mismatch"),
                f"The reserved {lane_scope.upper()} job must run its exact approved command",
                details={"reservationId": reservation_id},
            )
        try:
            compatibility = json.loads(reservation["compatibility_json"])
        except (TypeError, ValueError, json.JSONDecodeError) as error:
            raise CoordinatorError(
                reservation_code(lane_scope, "compatibility_invalid"),
                f"{lane_scope.upper()} reservation has no usable canonical compatibility payload",
                details={"reservationId": reservation_id},
            ) from error
        source_manifest = self._source_manifest_from_compatibility(
            compatibility,
            lane_scope=lane_scope,
            reservation_id=reservation_id,
        )
        with self.database.connect() as connection:
            source_copy = self._require_source_copy(
                connection,
                session_id=session_id,
                compatibility=compatibility,
                source_manifest=source_manifest,
                lane_scope=lane_scope,
                reservation_id=reservation_id,
            )
        if source_copy is None:
            self._verify_source_manifest(
                source_manifest,
                lane_scope=lane_scope,
                reservation_id=reservation_id,
            )
        return self._environment_from_reservation_compatibility(reservation)

    def reserved_run_context(
        self,
        reservation_id: str,
        *,
        session_id: str,
        job_id: str,
        command: list[str] | tuple[str, ...],
    ) -> CargoRunContext:
        environment = self.reserved_run_environment(
            reservation_id,
            session_id=session_id,
            job_id=job_id,
            command=command,
        )
        with self.database.connect() as connection:
            row = connection.execute(
                """SELECT copy.source_root
                   FROM cargo_jobs job
                   LEFT JOIN validation_copies copy
                     ON copy.job_id=job.source_copy_job_id
                   WHERE job.job_id=? AND job.session_id=?""",
                (job_id, session_id),
            ).fetchone()
        if row is None:
            raise CoordinatorError(
                "cargo_job_not_found", f"Unknown Cargo job {job_id}"
            )
        working_directory = (
            Path(str(row["source_root"])).resolve()
            if row["source_root"] is not None
            else self.repo_root
        )
        if not working_directory.is_dir():
            raise CoordinatorError(
                "cargo_run_source_root_invalid",
                "Managed Cargo source root is unavailable",
                details={"sourceRoot": str(working_directory)},
            )
        return CargoRunContext(environment, working_directory)

    def _verify_source_manifest(
        self,
        source_manifest: dict[str, str],
        *,
        lane_scope: str,
        reservation_id: str | None = None,
        source_root: Path | None = None,
    ) -> None:
        """Reject a managed start when a reservation's claimed source bytes drift.

        A source manifest is optional for historical reservations.  When a
        new exact reservation declares one, both reservation creation and
        start must observe precisely those bytes.  This closes the shared-main
        race where a valid test command could otherwise run after another
        writer reverted its owned source file.
        """
        root = (source_root or self.repo_root).resolve()
        for relative_path, expected_hash in source_manifest.items():
            source = (root / relative_path).resolve()
            if not source.is_relative_to(root) or not source.is_file():
                raise CoordinatorError(
                    reservation_code(lane_scope, "source_manifest_stale"),
                    "Reservation source manifest path is no longer a regular file",
                    details={
                        "reservationId": reservation_id,
                        "path": relative_path,
                        "expectedHash": expected_hash,
                        "actualHash": None,
                    },
                )
            actual_hash = hashlib.sha256(source.read_bytes()).hexdigest().upper()
            if actual_hash != expected_hash:
                raise CoordinatorError(
                    reservation_code(lane_scope, "source_manifest_stale"),
                    "Reservation source manifest no longer matches its bound source root",
                    details={
                        "reservationId": reservation_id,
                        "path": relative_path,
                        "expectedHash": expected_hash,
                        "actualHash": actual_hash,
                    },
                )

    def _require_source_copy(
        self,
        connection,
        *,
        session_id: str,
        compatibility: dict[str, object],
        source_manifest: dict[str, str],
        lane_scope: str,
        reservation_id: str | None = None,
    ):
        copy_job_id = compatibility.get("source_copy_job_id")
        if copy_job_id is None:
            return None
        if not source_manifest:
            raise CoordinatorError(
                reservation_code(lane_scope, "source_copy_manifest_missing"),
                "Immutable Cargo source copy requires a selected source manifest",
                details={"reservationId": reservation_id},
            )
        row = connection.execute(
            """SELECT job_id, session_id, source_root, status, input_manifest_hash
               FROM validation_copies WHERE job_id=?""",
            (copy_job_id,),
        ).fetchone()
        expected_hash = compatibility.get("source_copy_manifest_hash")
        materialized_hash = (
            str(row["input_manifest_hash"]).upper()
            if row is not None and row["input_manifest_hash"] is not None
            else None
        )
        if (
            row is None
            or row["session_id"] != session_id
            or row["status"] != "materialized"
            or materialized_hash != expected_hash
        ):
            raise CoordinatorError(
                reservation_code(lane_scope, "source_copy_invalid"),
                "Cargo reservation source copy is missing, foreign, stale, or incomplete",
                details={"reservationId": reservation_id, "sourceCopyJobId": copy_job_id},
            )
        source_root = Path(str(row["source_root"])).resolve()
        if not source_root.is_dir():
            raise CoordinatorError(
                reservation_code(lane_scope, "source_copy_invalid"),
                "Cargo reservation source copy root is unavailable",
                details={"sourceCopyJobId": copy_job_id},
            )
        self._verify_source_manifest(
            source_manifest,
            lane_scope=lane_scope,
            reservation_id=reservation_id,
            source_root=source_root,
        )
        return row

    def _canonical_repo_path(self, raw_path: str) -> str:
        candidate = Path(raw_path)
        resolved = (
            candidate.resolve()
            if candidate.is_absolute()
            else (self.repo_root / candidate).resolve()
        )
        if not resolved.is_relative_to(self.repo_root) or resolved == self.repo_root:
            raise CoordinatorError(
                "cargo_cpu_reservation_failure_path_invalid",
                "Failure priority paths must stay within the coordinator repository",
                details={"path": raw_path},
            )
        return resolved.relative_to(self.repo_root).as_posix()

    def _is_dependency_lock_preflight(
        self,
        connection,
        *,
        session_id: str,
        compatibility: dict[str, object],
        source_manifest: dict[str, str],
        required_paths: set[str],
        now: str,
    ) -> bool:
        """Allow only a bounded lock-refresh preflight to precede locked work.

        The preflight may bind only the manifest/lock inputs themselves.  The
        owner must nonetheless retain live leases for every related source
        path, so its later source validation cannot be silently detached from
        the open failure it is unblocking.
        """
        try:
            build_config = self._build_config_from_compatibility(
                compatibility,
                lane_scope="cpu",
            )
        except CoordinatorError:
            return False
        operation = build_config.get("operation")
        if not isinstance(operation, str) or not operation.casefold().endswith("lock-refresh"):
            return False
        if str(build_config.get("profile", "")).casefold() != "metadata":
            return False
        if str(build_config.get("locked", "")).casefold() not in {"false", "0"}:
            return False
        if str(build_config.get("no_deps", "")).casefold() not in {"true", "1"}:
            return False
        dependency_paths = {
            path for path in required_paths if Path(path).name in {"Cargo.toml", "Cargo.lock"}
        }
        if not dependency_paths or dependency_paths == required_paths:
            return False
        if set(source_manifest) != dependency_paths:
            return False
        leased_paths = {
            self._canonical_repo_path(str(row["display_path"]))
            for row in connection.execute(
                """
                SELECT display_path FROM leases
                WHERE session_id=? AND expires_at>?
                """,
                (session_id, now),
            ).fetchall()
        }
        return required_paths.issubset(leased_paths)

    def _source_manifest_from_compatibility(
        self,
        compatibility: dict[str, object],
        *,
        lane_scope: str,
        reservation_id: str | None = None,
    ) -> dict[str, str]:
        raw_manifest = compatibility.get("source_manifest")
        if raw_manifest is None:
            build_config = self._build_config_from_compatibility(
                compatibility,
                lane_scope=lane_scope,
                reservation_id=reservation_id,
            )
            raw_manifest = build_config.get("source_manifest")
        if raw_manifest is None:
            return {}
        if not isinstance(raw_manifest, dict) or not raw_manifest:
            raise CoordinatorError(
                reservation_code(lane_scope, "source_manifest_invalid"),
                "Reservation source_manifest must be a non-empty path-to-SHA256 object",
                details={"reservationId": reservation_id},
            )
        manifest: dict[str, str] = {}
        for raw_path, raw_hash in raw_manifest.items():
            if not isinstance(raw_path, str) or not isinstance(raw_hash, str):
                raise CoordinatorError(
                    reservation_code(lane_scope, "source_manifest_invalid"),
                    "Reservation source_manifest entries must contain text paths and SHA-256 values",
                    details={"reservationId": reservation_id},
                )
            candidate = (self.repo_root / raw_path).resolve()
            if (
                Path(raw_path).is_absolute()
                or not candidate.is_relative_to(self.repo_root)
                or candidate == self.repo_root
            ):
                raise CoordinatorError(
                    reservation_code(lane_scope, "source_manifest_invalid"),
                    "Reservation source_manifest paths must be repository-relative files",
                    details={"reservationId": reservation_id, "path": raw_path},
                )
            normalized_path = candidate.relative_to(self.repo_root).as_posix()
            normalized_hash = raw_hash.upper()
            if not re.fullmatch(r"[0-9A-F]{64}", normalized_hash):
                raise CoordinatorError(
                    reservation_code(lane_scope, "source_manifest_invalid"),
                    "Reservation source_manifest values must be SHA-256 hex digests",
                    details={"reservationId": reservation_id, "path": normalized_path},
                )
            if normalized_path in manifest:
                raise CoordinatorError(
                    reservation_code(lane_scope, "source_manifest_invalid"),
                    "Reservation source_manifest contains the same source file more than once",
                    details={"reservationId": reservation_id, "path": normalized_path},
                )
            manifest[normalized_path] = normalized_hash
        canonical = dict(sorted(manifest.items()))
        if len(canonical) > MAX_SOURCE_MANIFEST_ENTRIES:
            raise CoordinatorError(
                reservation_code(lane_scope, "source_manifest_invalid"),
                f"Reservation source_manifest exceeds {MAX_SOURCE_MANIFEST_ENTRIES} entries",
                details={"reservationId": reservation_id},
            )
        if len(json.dumps(canonical, sort_keys=True, separators=(",", ":")).encode("utf-8")) > MAX_SOURCE_MANIFEST_BYTES:
            raise CoordinatorError(
                reservation_code(lane_scope, "source_manifest_invalid"),
                f"Reservation source_manifest exceeds {MAX_SOURCE_MANIFEST_BYTES} serialized bytes",
                details={"reservationId": reservation_id},
            )
        return canonical

    @staticmethod
    def _build_config_from_compatibility(
        compatibility: dict[str, object],
        *,
        lane_scope: str,
        reservation_id: str | None = None,
    ) -> dict[str, object]:
        """Parse a canonical build configuration without changing legacy profiles."""
        try:
            build_config_raw = compatibility["build_config"]
        except (KeyError, TypeError) as error:
            raise CoordinatorError(
                reservation_code(lane_scope, "compatibility_invalid"),
                f"{lane_scope.upper()} reservation does not contain a canonical build configuration",
                details={"reservationId": reservation_id},
            ) from error
        if not isinstance(build_config_raw, str):
            raise CoordinatorError(
                reservation_code(lane_scope, "compatibility_invalid"),
                f"{lane_scope.upper()} reservation build configuration must be text",
                details={"reservationId": reservation_id},
            )
        try:
            build_config = json.loads(build_config_raw)
        except json.JSONDecodeError:
            # Historical compatible pools use the established semicolon profile
            # form. It has no privileged environment unless it explicitly names
            # one of the two allowlisted keys.
            build_config = {
                key.strip().casefold(): value.strip()
                for part in build_config_raw.split(";")
                if (key_value := part.partition("="))[1]
                for key, _, value in (key_value,)
            }
        if not isinstance(build_config, dict):
            raise CoordinatorError(
                reservation_code(lane_scope, "compatibility_invalid"),
                f"{lane_scope.upper()} reservation build configuration must be an object or profile string",
                details={"reservationId": reservation_id},
            )
        return build_config

    @classmethod
    def _environment_from_reservation_compatibility(cls, reservation) -> dict[str, str]:
        """Derive the allowlisted process environment from canonical reservation data."""
        try:
            compatibility = json.loads(reservation["compatibility_json"])
        except (TypeError, ValueError, json.JSONDecodeError) as error:
            raise CoordinatorError(
                reservation_code(str(reservation["lane_scope"]), "compatibility_invalid"),
                f"{reservation['lane_scope'].upper()} reservation has no usable canonical compatibility payload",
                details={"reservationId": reservation["reservation_id"]},
            ) from error
        build_config = cls._build_config_from_compatibility(
            compatibility,
            lane_scope=str(reservation["lane_scope"]),
            reservation_id=reservation["reservation_id"],
        )
        environment: dict[str, str] = {}
        rustflags = build_config.get("rustflags")
        if isinstance(rustflags, str) and rustflags:
            environment["RUSTFLAGS"] = rustflags
        cargo_incremental = build_config.get("cargo_incremental", build_config.get("incremental"))
        if isinstance(cargo_incremental, str) and cargo_incremental:
            environment["CARGO_INCREMENTAL"] = cargo_incremental
        return environment

    def acquire(
        self,
        session_id: str,
        lane_kind: CargoLaneKind,
        *,
        requested_target: str | Path | None = None,
        dry_run: bool = False,
        owner_pid: int | None = None,
        ephemeral: bool = False,
        compatibility: CargoCompatibility | None = None,
        expected_cpu_reservation_id: str | None = None,
        expected_gpu_reservation_id: str | None = None,
        isolated_cpu_burst: bool = False,
        cpu_burst_selection: CpuBurstSelection | None = None,
        admission_checkpoint: str | None = None,
        admission_operation: str | None = None,
    ) -> CargoJob:
        if not isinstance(lane_kind, CargoLaneKind):
            raise ValueError("lane_kind must be a CargoLaneKind")
        if ephemeral and compatibility is not None:
            raise CoordinatorError(
                "cargo_compatibility_conflict",
                "An ephemeral Cargo job cannot also request a reusable compatibility pool",
            )
        job_id = uuid.uuid4().hex
        canonical_compatibility = (
            compatibility.canonical() if compatibility is not None else None
        )
        compatibility_json = (
            json.dumps(canonical_compatibility, sort_keys=True, separators=(",", ":"))
            if canonical_compatibility is not None
            else None
        )
        reuse_key = (
            self._compatibility_fingerprint(compatibility_json)
            if compatibility_json is not None
            else None
        )
        ephemeral = ephemeral or compatibility_json is None
        cleanup_policy = (
            CargoCleanupPolicy.DELETE_ON_RELEASE if ephemeral else CargoCleanupPolicy.RETAINED
        )
        requested = self.target_policy.validate(requested_target) if requested_target else None
        target: Path | None = requested
        reused_from_job_id: str | None = None
        if isinstance(owner_pid, bool) or (owner_pid is not None and owner_pid <= 0):
            raise CoordinatorError(
                "cargo_owner_pid_invalid", "Cargo lease owner PID must be a positive integer"
            )
        owner_creation_time = (
            self._read_process_creation_time(owner_pid) if owner_pid is not None else None
        )
        owner_root_kind = (
            CargoProcessRootKind.SUPERVISOR
            if owner_pid is not None
            else CargoProcessRootKind.CARGO
        )
        now = utc_text()
        with self.database.transaction() as connection:
            self._require_admission_checkpoint(
                connection,
                admission_operation or f"cargo.acquire@{session_id}",
                admission_checkpoint,
            )
            require_executable_cargo_session(connection, session_id)
            if expected_cpu_reservation_id is not None and expected_gpu_reservation_id is not None:
                raise CoordinatorError(
                    "cargo_reservation_consume_arguments_invalid",
                    "A managed Cargo job can consume exactly one lane reservation",
                )
            if isolated_cpu_burst and expected_cpu_reservation_id is None:
                raise CoordinatorError(
                    "cargo_cpu_burst_unavailable",
                    "An isolated CPU burst requires its exact CPU reservation",
                )
            if expected_cpu_reservation_id is not None and lane_kind is CargoLaneKind.GPU:
                raise CoordinatorError(
                    "cargo_cpu_reservation_lane_invalid",
                    "A CPU reservation cannot create a GPU Cargo job",
                )
            if expected_gpu_reservation_id is not None and lane_kind is not CargoLaneKind.GPU:
                raise CoordinatorError(
                    "cargo_gpu_reservation_lane_invalid",
                    "A GPU reservation can create only a GPU Cargo job",
                )
            expected_reservation_id = expected_cpu_reservation_id or expected_gpu_reservation_id
            expected_scope = "cpu" if expected_cpu_reservation_id else "gpu"
            if expected_reservation_id is not None:
                existing_job = self._bound_reservation_job(
                    connection,
                    reservation_id=expected_reservation_id,
                    session_id=session_id,
                    lane_kind=lane_kind,
                    lane_scope=expected_scope,
                )
                if existing_job is not None:
                    if expected_cpu_reservation_id is not None:
                        self._require_reservation_consume_guard(
                            connection,
                            expected_cpu_reservation_id,
                            session_id,
                            existing_job.job_id,
                        )
                    return existing_job
            if expected_cpu_reservation_id is not None:
                self._require_reservation_consume_guard(
                    connection,
                    expected_cpu_reservation_id,
                    session_id,
                    None,
                )
            admitted_burst = False
            if expected_cpu_reservation_id is not None and cpu_burst_selection is not None:
                admitted_burst = self._admit_cpu_execution_mode(
                    connection,
                    reservation_id=expected_cpu_reservation_id,
                    session_id=session_id,
                    selection=cpu_burst_selection,
                )
                if admitted_burst:
                    if cpu_burst_selection.target_dir is None:
                        raise CoordinatorError(
                            "cargo_cpu_burst_unavailable",
                            "Atomic burst admission has no managed isolated target",
                        )
                    requested = self.target_policy.validate(cpu_burst_selection.target_dir)
                    target = requested
                    compatibility_json = None
                    reuse_key = None
                    ephemeral = True
                    cleanup_policy = CargoCleanupPolicy.DELETE_ON_RELEASE
            gpu_reservation = self._require_gpu_reservation(
                connection,
                session_id,
                lane_kind,
                compatibility_key=reuse_key,
                compatibility_json=compatibility_json,
                now=now,
                expected_reservation_id=expected_gpu_reservation_id,
            )
            self._require_gpu_lane_available(connection, lane_kind)
            cpu_reservation = self._require_cpu_reservation(
                connection,
                session_id=session_id,
                lane_kind=lane_kind,
                compatibility_key=reuse_key,
                compatibility_json=compatibility_json,
                now=now,
                expected_reservation_id=expected_cpu_reservation_id,
                allow_isolated_burst=isolated_cpu_burst or admitted_burst,
            )
            if cpu_reservation is not None:
                self._require_dependency_barrier(
                    connection, cpu_reservation, lane_scope="cpu"
                )
            lane_reservation = gpu_reservation or cpu_reservation
            if reuse_key is not None:
                active_pool = connection.execute(
                    """
                    SELECT job_id, target_dir FROM cargo_jobs
                    WHERE reuse_key=? AND status IN ('leased', 'running')
                    ORDER BY created_at DESC LIMIT 1
                    """,
                    (reuse_key,),
                ).fetchone()
                if active_pool is not None:
                    raise CoordinatorError(
                        "cargo_reuse_pool_busy",
                        f"Compatible Cargo pool is owned by job {active_pool['job_id']}",
                    )
                reusable_rows = connection.execute(
                    """
                    SELECT job_id, target_dir FROM cargo_jobs
                    WHERE reuse_key=? AND cleanup_status='retained'
                      AND status IN ('released', 'orphaned')
                    ORDER BY released_at DESC, created_at DESC
                    """,
                    (reuse_key,),
                ).fetchall()
                reusable = next(
                    (
                        row
                        for row in reusable_rows
                        if self.target_policy.validate(row["target_dir"]).is_dir()
                    ),
                    None,
                )
                missing_job_ids = [
                    row["job_id"]
                    for row in reusable_rows
                    if not self.target_policy.validate(row["target_dir"]).is_dir()
                ]
                if missing_job_ids:
                    connection.executemany(
                        """
                        UPDATE cargo_jobs
                        SET cleanup_status='deleted', cleanup_error=NULL
                        WHERE job_id=? AND cleanup_status='retained'
                        """,
                        ((missing_job_id,) for missing_job_id in missing_job_ids),
                    )
                if reusable is not None:
                    candidate = self.target_policy.validate(reusable["target_dir"])
                    candidate_key = target_identity(candidate)
                    duplicate_job_ids = [
                        row["job_id"]
                        for row in reusable_rows
                        if row["job_id"] != reusable["job_id"]
                        and self.target_policy.validate(row["target_dir"]).is_dir()
                        and target_identity(row["target_dir"]) != candidate_key
                    ]
                    if duplicate_job_ids:
                        # Historical or manually imported rows can predate the single-pool
                        # invariant. Keep the newest existing directory authoritative and
                        # send every other retained directory through prompt cleanup.
                        connection.executemany(
                            """
                            UPDATE cargo_jobs
                            SET cleanup_policy='delete_on_release', cleanup_status='pending',
                                cleanup_error=NULL
                            WHERE job_id=? AND cleanup_status='retained'
                            """,
                            ((duplicate_job_id,) for duplicate_job_id in duplicate_job_ids),
                        )
                    reserved_candidate = overlapping_cleanup_reservation(
                        connection, candidate_key
                    )
                    if reserved_candidate is not None:
                        raise CoordinatorError(
                            "cargo_lane_cleanup_reserved",
                            f"Compatible Cargo pool is reserved for cleanup: {candidate}",
                        )
                    if requested is not None and target_identity(requested) != candidate_key:
                        raise CoordinatorError(
                            "cargo_reuse_target_mismatch",
                            "The requested target differs from the existing primary pool for this "
                            "compatibility key",
                        )
                    target = candidate
                    reused_from_job_id = reusable["job_id"]
            if target is None:
                root = self.target_policy.choose_root(self.free_space)
                if reuse_key is not None:
                    target = self.target_policy.validate(
                        root / "zircon-engine" / "pool" / reuse_key
                    )
                else:
                    target = self.target_policy.validate(
                        root / "zircon-engine" / "ephemeral" / lane_kind.value / job_id
                    )
            target_key = target_identity(target)
            reservation = overlapping_cleanup_reservation(connection, target_key)
            if reservation is not None:
                raise CoordinatorError(
                    "cargo_lane_cleanup_reserved",
                    f"Cargo target is reserved for cleanup: {reservation['target_dir']}",
                )
            cleanup_failed = next(
                (
                    row
                    for row in connection.execute(
                        """
                        SELECT job_id, target_key, target_dir, cleanup_error
                        FROM cargo_jobs
                        WHERE cleanup_status='failed'
                        ORDER BY created_at DESC, job_id DESC
                        """
                    ).fetchall()
                    if targets_overlap(target_key, row["target_key"])
                ),
                None,
            )
            if cleanup_failed is not None:
                raise CoordinatorError(
                    "cargo_lane_cleanup_failed",
                    "Cargo target overlaps a failed deletion and cannot be reused until cleanup succeeds",
                    details={
                        "jobId": cleanup_failed["job_id"],
                        "targetDir": cleanup_failed["target_dir"],
                        "cleanupError": cleanup_failed["cleanup_error"],
                    },
                )
            active_rows = connection.execute(
                """
                SELECT job_id, target_key FROM cargo_jobs
                WHERE status IN ('leased', 'running')
                """
            ).fetchall()
            occupied = next(
                (row for row in active_rows if targets_overlap(target_key, row["target_key"])),
                None,
            )
            if occupied is not None:
                raise CoordinatorError(
                    "cargo_reuse_pool_busy" if reuse_key is not None else "cargo_lane_occupied",
                    f"Cargo target is already owned by job {occupied['job_id']}",
                )
            tree_blocker = self._live_tree_target_blocker(connection, target_key)
            if tree_blocker is not None:
                job, live_pids = tree_blocker
                raise self._process_tree_alive_error(job, live_pids)
            source_copy_job_id = (
                lane_reservation["source_copy_job_id"]
                if lane_reservation is not None
                else None
            )
            source_copy_manifest_hash = None
            if source_copy_job_id is not None:
                try:
                    reservation_compatibility = json.loads(
                        lane_reservation["compatibility_json"]
                    )
                    source_copy_manifest_hash = reservation_compatibility[
                        "source_copy_manifest_hash"
                    ]
                except (KeyError, TypeError, json.JSONDecodeError) as error:
                    raise CoordinatorError(
                        "cargo_cpu_reservation_source_copy_invalid",
                        "CPU reservation lost its immutable source-copy identity",
                    ) from error
            connection.execute(
                """
                INSERT INTO cargo_jobs(
                    job_id, session_id, lane_kind, target_dir, status, dry_run,
                    created_at, last_heartbeat_at, target_key, pid, reuse_key,
                    compatibility_json, compatibility_key, reuse_profile, cleanup_policy,
                    cleanup_status, reused_from_job_id, source_copy_job_id,
                    source_copy_manifest_hash, root_process_creation_time,
                    root_process_kind
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                """,
                (
                    job_id,
                    session_id,
                    lane_kind.value,
                    str(target),
                    CargoJobStatus.LEASED.value,
                    1 if dry_run else 0,
                    now,
                    now,
                    target_key,
                    owner_pid,
                    reuse_key,
                    compatibility_json,
                    reuse_key,
                    compatibility_json,
                    cleanup_policy.value,
                    (
                        CargoCleanupStatus.PENDING.value
                        if ephemeral
                        else CargoCleanupStatus.RETAINED.value
                    ),
                    reused_from_job_id,
                    source_copy_job_id,
                    source_copy_manifest_hash,
                    owner_creation_time,
                    owner_root_kind.value,
                ),
            )
            if lane_reservation is not None:
                connection.execute(
                    """
                    UPDATE cargo_lane_reservations
                    SET job_id=?, status='leased'
                    WHERE reservation_id=? AND status='pending'
                    """,
                    (job_id, lane_reservation["reservation_id"]),
                )
        if not dry_run:
            try:
                target.mkdir(parents=True, exist_ok=True)
            except BaseException:
                with self.database.transaction() as connection:
                    connection.execute(
                        """
                        UPDATE cargo_lane_reservations
                        SET job_id=NULL, status='pending',
                            execution_mode=CASE WHEN execution_mode='burst' THEN 'warm'
                                                ELSE execution_mode END
                        WHERE job_id=? AND status='leased'
                        """,
                        (job_id,),
                    )
                    connection.execute("DELETE FROM cargo_jobs WHERE job_id = ?", (job_id,))
                raise
        return self.get(job_id)

    def _require_cpu_reservation(
        self,
        connection,
        *,
        session_id: str,
        lane_kind: CargoLaneKind,
        compatibility_key: str | None,
        compatibility_json: str | None,
        now: str,
        expected_reservation_id: str | None = None,
        allow_isolated_burst: bool = False,
    ):
        if lane_kind is CargoLaneKind.GPU:
            return None
        return self._require_lane_reservation(
            connection,
            session_id=session_id,
            lane_scope="cpu",
            compatibility_key=compatibility_key,
            compatibility_json=compatibility_json,
            now=now,
            expected_reservation_id=expected_reservation_id,
            allow_isolated_burst=allow_isolated_burst,
        )

    @staticmethod
    def _require_lane_reservation(
        connection,
        *,
        session_id: str,
        lane_scope: str,
        compatibility_key: str | None,
        compatibility_json: str | None,
        now: str,
        expected_reservation_id: str | None,
        allow_isolated_burst: bool = False,
    ):
        if lane_scope == "cpu":
            reconcile_cpu_fifo_eligibility(connection, now=now)
        else:
            expire_invalid_pending_lane_reservations(connection, lane_scope=lane_scope, now=now)
            reconcile_terminal_finished_lane_reservations(connection, lane_scope=lane_scope, now=now)
        if expected_reservation_id is None:
            reservation = lane_fifo_head(
                connection,
                lane_scope=lane_scope,
                execution_mode="warm" if lane_scope == "cpu" else None,
            )
        else:
            reservation = connection.execute(
                "SELECT * FROM cargo_lane_reservations WHERE reservation_id=? AND lane_scope=?",
                (expected_reservation_id, lane_scope),
            ).fetchone()
            if reservation is None:
                raise CoordinatorError(
                    reservation_code(lane_scope, "not_found"),
                    f"Unknown {lane_scope.upper()} reservation {expected_reservation_id}",
                )
            burst_reservation = (
                lane_scope == "cpu"
                and reservation["execution_mode"] == "burst"
                and allow_isolated_burst
            )
            fifo_head = lane_fifo_head(
                connection,
                lane_scope=lane_scope,
                execution_mode=(
                    "burst"
                    if burst_reservation
                    else "warm"
                    if lane_scope == "cpu"
                    else None
                ),
            )
            if fifo_head is None or fifo_head["reservation_id"] != expected_reservation_id:
                predecessor = (
                    None
                    if fifo_head is None
                    else {
                        "reservationId": fifo_head["reservation_id"],
                        "sessionId": fifo_head["session_id"],
                        "status": fifo_head["status"],
                        "jobId": fifo_head["job_id"],
                        "executionMode": fifo_head["execution_mode"],
                        "priorityRank": fifo_head["priority_rank"],
                        "createdAt": fifo_head["created_at"],
                    }
                )
                raise CoordinatorError(
                    reservation_code(lane_scope, "not_fifo_head"),
                    f"{lane_scope.upper()} reservation is no longer the next eligible FIFO entry",
                    details={
                        "reservationId": expected_reservation_id,
                        "predecessor": predecessor,
                    },
                )
        if reservation is None:
            return None
        if reservation["session_id"] != session_id:
            raise CoordinatorError(
                f"cargo_{lane_scope}_lane_reserved",
                f"The next managed {lane_scope.upper()} lane is reserved for another Session",
                details={"sessionId": reservation["session_id"], "reservationId": reservation["reservation_id"]},
            )
        if reservation["status"] != "pending":
            raise CoordinatorError(
                reservation_code(lane_scope, "consumed"),
                f"The {lane_scope.upper()} reservation already owns a Cargo job",
                details={"reservationId": reservation["reservation_id"], "jobId": reservation["job_id"]},
            )
        if not (lane_scope == "cpu" and reservation["execution_mode"] == "burst" and allow_isolated_burst) and compatibility_key != reservation["compatibility_key"]:
            raise CoordinatorError(
                reservation_code(lane_scope, "compatibility_mismatch"),
                f"The reserved {lane_scope.upper()} job requires its exact compatibility pool",
                details={"reservationId": reservation["reservation_id"]},
            )
        if not (lane_scope == "cpu" and reservation["execution_mode"] == "burst" and allow_isolated_burst) and compatibility_json != reservation["compatibility_json"]:
            raise CoordinatorError(
                reservation_code(lane_scope, "compatibility_mismatch"),
                f"The reserved {lane_scope.upper()} job requires its exact canonical compatibility payload",
                details={"reservationId": reservation["reservation_id"]},
            )
        return reservation

    def _bound_reservation_job(
        self,
        connection,
        *,
        reservation_id: str,
        session_id: str,
        lane_kind: CargoLaneKind,
        lane_scope: str,
    ) -> CargoJob | None:
        """Return an existing exact binding so retrying consume is idempotent."""
        reservation = connection.execute(
            "SELECT * FROM cargo_lane_reservations WHERE reservation_id=? AND lane_scope=?",
            (reservation_id, lane_scope),
        ).fetchone()
        if reservation is None:
            raise CoordinatorError(
                reservation_code(lane_scope, "not_found"),
                f"Unknown {lane_scope.upper()} reservation {reservation_id}",
            )
        if reservation["session_id"] != session_id:
            raise CoordinatorError(
                reservation_code(lane_scope, "owner_mismatch"),
                f"{lane_scope.upper()} reservation {reservation_id} belongs to Session {reservation['session_id']}",
            )
        if reservation["status"] == "pending":
            return None
        if reservation["status"] != "leased" or not reservation["job_id"]:
            raise CoordinatorError(
                reservation_code(lane_scope, "consumed"),
                f"The {lane_scope.upper()} reservation is no longer available for a new Cargo job",
                details={"reservationId": reservation_id, "jobId": reservation["job_id"]},
            )
        row = connection.execute(
            "SELECT * FROM cargo_jobs WHERE job_id=?",
            (reservation["job_id"],),
        ).fetchone()
        if row is None or row["session_id"] != session_id or row["lane_kind"] != lane_kind.value:
            raise CoordinatorError(
                reservation_code(lane_scope, "binding_invalid"),
                f"{lane_scope.upper()} reservation binding does not match the requested managed lane",
                details={"reservationId": reservation_id, "jobId": reservation["job_id"]},
            )
        return self._from_row(row)

    @staticmethod
    def _require_gpu_reservation(
        connection,
        session_id: str,
        lane_kind: CargoLaneKind,
        *,
        compatibility_key: str | None,
        compatibility_json: str | None,
        now: str,
        expected_reservation_id: str | None,
    ):
        """Honor the one-shot GPU lane reservation carried by the latest resume action.

        The action record is already durable and auditable.  The reservation is
        consumed only after its nominated Session starts and reaches a terminal
        state; a lease that is released before launch remains retryable and
        cannot lose FIFO priority to another Session.
        """
        if lane_kind is not CargoLaneKind.GPU:
            return None
        durable = CargoJobService._require_lane_reservation(
            connection,
            session_id=session_id,
            lane_scope="gpu",
            compatibility_key=compatibility_key,
            compatibility_json=compatibility_json,
            now=now,
            expected_reservation_id=expected_reservation_id,
        )
        if durable is not None:
            if expected_reservation_id is None:
                raise CoordinatorError(
                    "cargo_gpu_reservation_requires_consume",
                    "A pending GPU reservation must be consumed through its exact typed path",
                    details={"reservationId": durable["reservation_id"]},
                )
            return durable
        if expected_reservation_id is not None:
            return None
        action = connection.execute(
            """
            SELECT parameters_json, completed_at
            FROM action_requests
            WHERE action_kind='service.resume' AND status='succeeded'
              AND completed_at IS NOT NULL
            ORDER BY completed_at DESC, action_id DESC
            LIMIT 1
            """
        ).fetchone()
        if action is None:
            return None
        try:
            parameters = json.loads(action["parameters_json"])
        except (TypeError, json.JSONDecodeError):
            return None
        reserved_session_id = parameters.get("gpuReservationSessionId")
        if not isinstance(reserved_session_id, str) or not reserved_session_id:
            return None
        active = connection.execute(
            """
            SELECT job_id FROM cargo_jobs
            WHERE session_id=? AND lane_kind='gpu' AND created_at>=?
              AND status IN ('leased', 'running')
            LIMIT 1
            """,
            (reserved_session_id, action["completed_at"]),
        ).fetchone()
        if active is not None:
            if session_id == reserved_session_id:
                return None
            raise CoordinatorError(
                "cargo_gpu_lane_reserved",
                "The next managed GPU lane is reserved for another Session",
                details={"sessionId": reserved_session_id},
            )
        terminal_started = connection.execute(
            """
            SELECT 1 FROM cargo_jobs
            WHERE session_id=? AND lane_kind='gpu' AND created_at>=?
              AND started_at IS NOT NULL
              AND status NOT IN ('leased', 'running')
            LIMIT 1
            """,
            (reserved_session_id, action["completed_at"]),
        ).fetchone()
        if terminal_started is not None or session_id == reserved_session_id:
            return None
        raise CoordinatorError(
            "cargo_gpu_lane_reserved",
            "The next managed GPU lane is reserved for another Session",
            details={"sessionId": reserved_session_id},
        )

    @staticmethod
    def _require_gpu_lane_available(connection, lane_kind: CargoLaneKind) -> None:
        if lane_kind is not CargoLaneKind.GPU:
            return
        occupied = connection.execute(
            """SELECT job_id, session_id, status, target_dir FROM cargo_jobs
               WHERE lane_kind='gpu' AND status IN ('leased', 'running')
               ORDER BY created_at, job_id LIMIT 1"""
        ).fetchone()
        if occupied is None:
            return
        raise CoordinatorError(
            "cargo_gpu_lane_occupied",
            "The single managed GPU lane is occupied",
            details={
                "jobId": occupied["job_id"],
                "sessionId": occupied["session_id"],
                "status": occupied["status"],
                "targetDir": occupied["target_dir"],
            },
        )

    def audit_active_gpu_jobs(self) -> tuple[CargoJob, ...]:
        """Return the startup-visible leases that occupy the single GPU lane."""
        with self.database.connect() as connection:
            rows = connection.execute(
                """SELECT * FROM cargo_jobs
                   WHERE lane_kind='gpu' AND status IN ('leased', 'running')
                   ORDER BY created_at, job_id"""
            ).fetchall()
        return tuple(self._from_row(row) for row in rows)

    def authorize_managed_start(
        self,
        job_id: str,
        *,
        session_id: str,
        command: list[str] | tuple[str, ...],
    ) -> CargoJob:
        """Durably consume launch authorization before a managed child exists."""

        command_tuple = tuple(command)
        now = utc_text()
        try:
            with self.database.transaction() as connection:
                reservation = self._require_start_eligibility(
                    connection,
                    job_id=job_id,
                    session_id=session_id,
                    command=command_tuple,
                    now=now,
                )
                persist_spawn_authorization(
                    connection,
                    job_id=job_id,
                    session_id=session_id,
                    command=command_tuple,
                    authorized_at=now,
                    reservation_id=(
                        str(reservation["reservation_id"])
                        if reservation is not None
                        else None
                    ),
                )
        except CoordinatorError as error:
            with self.database.transaction() as connection:
                self._record_event(
                    connection,
                    session_id,
                    "cargo.start_rejected",
                    {
                        "jobId": job_id,
                        "pid": None,
                        "rootIsSupervisor": True,
                        "code": error.code,
                    },
                )
            raise
        return self.get(job_id)

    def register_authorized_managed_run(
        self,
        job_id: str,
        *,
        session_id: str,
        pid: int,
        command: list[str] | tuple[str, ...],
        run_id: str,
        environment: Mapping[str, str],
        stdout_path: Path,
        stderr_path: Path,
        started_at: str,
        root_process_creation_time: str | None = None,
    ) -> CargoJob:
        """Bind a suspended supervisor and run in one durable transaction."""

        if pid <= 0:
            raise ValueError("Cargo process PID must be positive")
        command_tuple = tuple(command)
        now = utc_text()
        observed_creation_time, live_pids = self._observe_started_process(
            pid, root_is_supervisor=True
        )
        creation_time = root_process_creation_time or observed_creation_time
        with self.database.transaction() as connection:
            persist_authorized_managed_run(
                connection,
                run_id=run_id,
                job_id=job_id,
                session_id=session_id,
                command=command_tuple,
                environment=environment,
                stdout_path=stdout_path,
                stderr_path=stderr_path,
                started_at=started_at,
                observed_at=now,
                observation=SpawnObservation(
                    pid,
                    creation_time,
                    CargoProcessRootKind.SUPERVISOR.value,
                    live_pids,
                ),
            )
        return self.get(job_id)

    def rollback_managed_start_authorization(
        self,
        job_id: str,
        *,
        session_id: str,
        command: list[str] | tuple[str, ...],
    ) -> CargoJob:
        """Restore an authorization whose child was never durably registered."""

        command_tuple = tuple(command)
        now = utc_text()
        with self.database.transaction() as connection:
            rollback_spawn_authorization(
                connection,
                job_id=job_id,
                session_id=session_id,
                command=command_tuple,
                rolled_back_at=now,
            )
        return self.get(job_id)

    def mark_authorized_managed_run_resumed(
        self, run_id: str, *, job_id: str, session_id: str
    ) -> None:
        with self.database.transaction() as connection:
            mark_managed_run_resumed(
                connection,
                run_id=run_id,
                job_id=job_id,
                session_id=session_id,
            )

    def _require_start_eligibility(
        self,
        connection: Connection,
        *,
        job_id: str,
        session_id: str,
        command: tuple[str, ...],
        now: str,
    ):
        require_executable_cargo_session(connection, session_id)
        self._require_status(
            connection, job_id, {CargoJobStatus.LEASED}, session_id=session_id
        )
        reservation = connection.execute(
            "SELECT * FROM cargo_lane_reservations WHERE job_id=?", (job_id,)
        ).fetchone()
        if reservation is not None:
            lane_scope = str(reservation["lane_scope"])
            expire_invalid_pending_lane_reservations(
                connection, lane_scope=lane_scope, now=now
            )
            reservation = connection.execute(
                "SELECT * FROM cargo_lane_reservations WHERE reservation_id=?",
                (reservation["reservation_id"],),
            ).fetchone()
            if reservation["status"] != "leased":
                raise CoordinatorError(
                    reservation_code(lane_scope, "expired"),
                    f"The {lane_scope.upper()} reservation is no longer active",
                    details={"reservationId": reservation["reservation_id"]},
                )
            if reservation["command_fingerprint"] != self._command_fingerprint(command):
                raise CoordinatorError(
                    reservation_code(lane_scope, "command_mismatch"),
                    f"The reserved {lane_scope.upper()} job must start its exact approved command",
                    details={"reservationId": reservation["reservation_id"]},
                )
            return reservation

        # A legacy acquire cannot bypass a newer exact CPU FIFO reservation.
        expire_invalid_pending_lane_reservations(connection, lane_scope="cpu", now=now)
        reconcile_terminal_finished_lane_reservations(
            connection, lane_scope="cpu", now=now
        )
        priority = connection.execute(
            """SELECT reservation_id, session_id, status
               FROM cargo_lane_reservations
               WHERE lane_scope='cpu'
                 AND status IN ('pending', 'leased', 'running', 'finished')
               ORDER BY created_at, reservation_id LIMIT 1"""
        ).fetchone()
        if priority is not None:
            raise CoordinatorError(
                "cargo_cpu_lane_reserved",
                "An exact CPU reservation must start before an unreserved job",
                details={
                    "reservationId": priority["reservation_id"],
                    "sessionId": priority["session_id"],
                    "status": priority["status"],
                },
            )
        return None

    def _observe_started_process(
        self, pid: int, *, root_is_supervisor: bool
    ) -> tuple[str | None, tuple[int, ...]]:
        creation_time = self._read_process_creation_time(pid)
        process_tree = (
            self.supervisor_cargo_pids if root_is_supervisor else self.process_tree_pids
        )
        try:
            live_pids = tuple(
                sorted({int(value) for value in process_tree(pid) if int(value) > 0})
            )
        except (OSError, ValueError):
            live_pids = ()
        return creation_time, live_pids

    def start(
        self,
        job_id: str,
        *,
        session_id: str,
        pid: int,
        command: list[str] | tuple[str, ...],
        root_is_supervisor: bool = False,
    ) -> CargoJob:
        if pid <= 0:
            raise ValueError("Cargo process PID must be positive")
        now = utc_text()
        root_process_creation_time, initial_live_pids = self._observe_started_process(
            pid, root_is_supervisor=root_is_supervisor
        )
        root_process_kind = (
            CargoProcessRootKind.SUPERVISOR
            if root_is_supervisor
            else CargoProcessRootKind.CARGO
        )
        try:
            with self.database.transaction() as connection:
                reservation = self._require_start_eligibility(
                    connection,
                    job_id=job_id,
                    session_id=session_id,
                    command=tuple(command),
                    now=now,
                )
                connection.execute(
                    """
                    UPDATE cargo_jobs
                    SET status = ?, pid = ?, root_process_creation_time = ?, root_process_kind = ?,
                        command_json = ?, started_at = ?, last_heartbeat_at = ?
                        , process_tree_observed_at = ?, process_tree_live_pids_json = ?,
                        process_tree_exited_at = CASE WHEN ? THEN NULL ELSE ? END
                    WHERE job_id = ?
                    """,
                    (
                        CargoJobStatus.RUNNING.value,
                        pid,
                        root_process_creation_time,
                        root_process_kind.value,
                        json.dumps(tuple(command)),
                        now,
                        now,
                        now,
                        json.dumps(initial_live_pids),
                        1 if initial_live_pids else 0,
                        now,
                        job_id,
                    ),
                )
                if reservation is not None:
                    connection.execute(
                        """
                        UPDATE cargo_lane_reservations
                        SET status='running', started_at=?
                        WHERE reservation_id=? AND status='leased'
                        """,
                        (now, reservation["reservation_id"]),
                    )
                self._record_event(
                    connection,
                    session_id,
                    "cargo.start_accepted",
                    {
                        "jobId": job_id,
                        "pid": pid,
                        "rootIsSupervisor": root_is_supervisor,
                    },
                )
        except CoordinatorError as error:
            with self.database.transaction() as connection:
                self._record_event(
                    connection,
                    session_id,
                    "cargo.start_rejected",
                    {
                        "jobId": job_id,
                        "pid": pid,
                        "rootIsSupervisor": root_is_supervisor,
                        "code": error.code,
                    },
                )
            raise
        return self.get(job_id)

    @staticmethod
    def _record_event(
        connection,
        session_id: str,
        event_type: str,
        payload: dict[str, object],
    ) -> None:
        connection.execute(
            "INSERT INTO events(session_id, event_type, payload_json, created_at) VALUES (?, ?, ?, ?)",
            (session_id, event_type, json.dumps(payload, sort_keys=True), utc_text()),
        )

    def heartbeat(self, job_id: str, *, session_id: str) -> CargoJob:
        with self.database.transaction() as connection:
            self._require_status(
                connection,
                job_id,
                {CargoJobStatus.LEASED, CargoJobStatus.RUNNING},
                session_id=session_id,
            )
            connection.execute(
                "UPDATE cargo_jobs SET last_heartbeat_at = ? WHERE job_id = ?",
                (utc_text(), job_id),
            )
        return self.get(job_id)

    def finish(self, job_id: str, *, session_id: str, exit_code: int) -> CargoJob:
        now = utc_text()
        job = self.get(job_id)
        live_pids = self._live_process_pids(job, include_supervisor_root=False)
        blocked = False
        with self.database.transaction() as connection:
            self._require_status(
                connection,
                job_id,
                {CargoJobStatus.RUNNING, CargoJobStatus.ORPHANED},
                session_id=session_id,
            )
            self._record_process_tree_observation(connection, job_id, live_pids, now)
            if live_pids:
                blocked = True
            else:
                connection.execute(
                    """
                    UPDATE cargo_jobs
                    SET status = ?, exit_code = ?, finished_at = ?, last_heartbeat_at = ?
                    WHERE job_id = ?
                    """,
                    (
                        CargoJobStatus.SUCCEEDED.value if exit_code == 0 else CargoJobStatus.FAILED.value,
                        exit_code,
                        now,
                        now,
                        job_id,
                    ),
                )
                connection.execute(
                    """
                    UPDATE cargo_lane_reservations
                    SET status='finished', completed_at=?
                    WHERE job_id=? AND status='running'
                    """,
                    (now, job_id),
                )
        if blocked:
            raise self._process_tree_alive_error(job, live_pids)
        return self.get(job_id)

    def finish_from_atomic_job_terminal(
        self, job_id: str, *, session_id: str, exit_code: int
    ) -> CargoJob:
        """Finish and release a managed run after its atomic Job Object is empty.

        The Windows Job Object contains every descendant of the atomic Cargo
        launch, so its terminal observation is stronger than a subsequent PID
        projection that can briefly retain an exited process.
        """

        now = utc_text()
        with self._start_reconcile_lock:
            if job_id not in self._managed_collectors:
                raise CoordinatorError(
                    "cargo_managed_collector_not_registered",
                    "Atomic Job terminal evidence requires the active managed collector",
                )
            with self.database.transaction() as connection:
                self._require_status(
                    connection,
                    job_id,
                    {CargoJobStatus.RUNNING, CargoJobStatus.ORPHANED},
                    session_id=session_id,
                )
                self._record_process_tree_observation(connection, job_id, (), now)
                connection.execute(
                    """
                    UPDATE cargo_jobs
                    SET status = ?, exit_code = ?, finished_at = ?, released_at = ?,
                        last_heartbeat_at = ?,
                        cleanup_status = CASE
                            WHEN cleanup_policy='delete_on_release' THEN 'pending'
                            ELSE cleanup_status
                        END,
                        cleanup_error = NULL
                    WHERE job_id = ?
                    """,
                    (
                        CargoJobStatus.RELEASED.value,
                        exit_code,
                        now,
                        now,
                        now,
                        job_id,
                    ),
                )
                connection.execute(
                    """
                    UPDATE cargo_lane_reservations
                    SET status='released', completed_at=COALESCE(completed_at, ?)
                    WHERE job_id=? AND status IN ('leased', 'running', 'finished')
                    """,
                    (now, job_id),
                )
        return self.get(job_id)

    def release(self, job_id: str, *, session_id: str) -> CargoJob:
        now = utc_text()
        job = self.get(job_id)
        live_pids = self._live_process_pids(job, include_supervisor_root=False)
        blocked = False
        with self.database.transaction() as connection:
            row = connection.execute(
                "SELECT status, session_id FROM cargo_jobs WHERE job_id = ?", (job_id,)
            ).fetchone()
            if row is None:
                raise CoordinatorError("cargo_job_not_found", f"Unknown Cargo job {job_id}")
            self._require_owner(row, job_id, session_id)
            if row["status"] == CargoJobStatus.RUNNING.value:
                raise CoordinatorError("cargo_job_running", "A running Cargo job cannot be released")
            self._record_process_tree_observation(connection, job_id, live_pids, now)
            if live_pids:
                blocked = True
            else:
                connection.execute(
                    """
                    UPDATE cargo_jobs
                    SET status = ?, released_at = ?, last_heartbeat_at = ?,
                        cleanup_status = CASE
                            WHEN cleanup_policy='delete_on_release' THEN 'pending'
                            ELSE cleanup_status
                        END,
                        cleanup_error = NULL
                    WHERE job_id = ?
                    """,
                    (CargoJobStatus.RELEASED.value, now, now, job_id),
                )
                connection.execute(
                    """
                    UPDATE cargo_lane_reservations
                    SET status='released', completed_at=COALESCE(completed_at, ?)
                    WHERE job_id=? AND status IN ('leased', 'finished')
                    """,
                    (now, job_id),
                )
        if blocked:
            raise self._process_tree_alive_error(job, live_pids)
        return self.get(job_id)

    def _live_process_pids(
        self, job: CargoJob, *, include_supervisor_root: bool = True
    ) -> tuple[int, ...]:
        if job.pid is None:
            return ()
        root_identity_alive = True
        if job.root_process_creation_time is None:
            if job.status not in {
                CargoJobStatus.LEASED,
                CargoJobStatus.RUNNING,
            }:
                return ()
        else:
            observed_creation_time = self._read_process_creation_time(job.pid)
            if observed_creation_time is None:
                root_identity_alive = False
                if (
                    job.status
                    not in {CargoJobStatus.LEASED, CargoJobStatus.RUNNING}
                    and job.process_tree_exited_at is not None
                ):
                    # The terminal job already produced one authoritative empty-tree
                    # observation. A later access-denied PID cannot re-establish the old
                    # Cargo identity; same-user Cargo roots have readable creation times,
                    # while a reused protected system PID may not.
                    return ()
            elif observed_creation_time != job.root_process_creation_time:
                return ()
        cargo_descendants_only = (
            job.status is CargoJobStatus.LEASED
            or not include_supervisor_root
            or not root_identity_alive
        ) and (
            job.root_process_kind is CargoProcessRootKind.SUPERVISOR
            or job.status is CargoJobStatus.LEASED
        )
        process_tree = (
            self.supervisor_cargo_pids
            if cargo_descendants_only
            else self.process_tree_pids
        )
        live_pids = {int(value) for value in process_tree(job.pid) if int(value) > 0}
        if cargo_descendants_only:
            live_pids.discard(job.pid)
        return tuple(sorted(live_pids))

    def _read_process_creation_time(self, pid: int) -> str | None:
        try:
            return self.process_creation_time(pid)
        except (OSError, ValueError):
            return None

    def _root_process_identity_changed(self, job: CargoJob) -> bool:
        if job.pid is None or job.root_process_creation_time is None:
            return False
        observed_creation_time = self._read_process_creation_time(job.pid)
        return (
            observed_creation_time is not None
            and observed_creation_time != job.root_process_creation_time
        )

    def _live_tree_target_blocker(self, connection, target_key: str):
        rows = connection.execute(
            """
            SELECT * FROM cargo_jobs
            WHERE pid IS NOT NULL AND status NOT IN ('leased', 'running')
            ORDER BY created_at DESC, job_id DESC
            """
        ).fetchall()
        for row in rows:
            if not targets_overlap(target_key, row["target_key"]):
                continue
            job = self._from_row(row)
            live_pids = self._live_process_pids(job, include_supervisor_root=False)
            self._record_process_tree_observation(connection, job.job_id, live_pids, utc_text())
            if live_pids:
                return job, live_pids
        return None

    @staticmethod
    def _record_process_tree_observation(connection, job_id: str, live_pids: tuple[int, ...], now: str) -> None:
        connection.execute(
            """
            UPDATE cargo_jobs
            SET process_tree_observed_at = ?, process_tree_live_pids_json = ?,
                process_tree_exited_at = CASE
                    WHEN ? THEN NULL
                    ELSE COALESCE(process_tree_exited_at, ?)
                END
            WHERE job_id = ?
            """,
            (now, json.dumps(live_pids), 1 if live_pids else 0, now, job_id),
        )

    @staticmethod
    def _process_tree_alive_error(job: CargoJob, live_pids: tuple[int, ...]) -> CoordinatorError:
        return CoordinatorError(
            "cargo_process_tree_alive",
            "Cargo process tree still owns the target; wait for its recorded descendants to exit",
            details={
                "jobId": job.job_id,
                "targetDir": job.target_dir,
                "rootPid": job.pid,
                "livePids": list(live_pids),
            },
        )

    def _compatibility_fingerprint(self, compatibility_json: str) -> str:
        digest = hashlib.sha256()
        digest.update(target_identity(self.repo_root).encode("utf-8"))
        digest.update(b"\0")
        digest.update(compatibility_json.encode("utf-8"))
        return digest.hexdigest()

    @staticmethod
    def _command_fingerprint(command: tuple[str, ...]) -> str:
        digest = hashlib.sha256()
        digest.update(json.dumps(command, ensure_ascii=False, separators=(",", ":")).encode("utf-8"))
        return digest.hexdigest()

    @staticmethod
    def _reject_coordinator_output_flags(command: tuple[str, ...]) -> None:
        """Keep coordinator transport flags out of a managed process command.

        The CLI's JSON formatting belongs to the coordinator envelope.  Passing
        it through a Cargo test separator reaches the test binary instead and
        can yield a misleading successful compile followed by zero executed
        tests.  Reject it before a reservation is recorded and again before an
        older reservation is allowed to start.
        """
        invalid_flags = tuple(
            part for part in command if part.strip().casefold() in {"-json", "--json"}
        )
        if invalid_flags:
            raise CoordinatorError(
                "cargo_command_contains_coordinator_flag",
                "Coordinator JSON output flags must be outside the managed Cargo command",
                details={"flags": list(invalid_flags)},
            )

    @classmethod
    def _reservation_dict(cls, row) -> dict[str, object]:
        compatibility = json.loads(row["compatibility_json"]) if row["compatibility_json"] else None
        source_manifest = (
            cls._source_manifest_from_reservation_row(compatibility)
            if compatibility is not None
            else {}
        )
        return {
            "reservationId": row["reservation_id"],
            "sessionId": row["session_id"],
            "laneScope": row["lane_scope"],
            "executionMode": row["execution_mode"],
            "burstEligible": bool(row["burst_eligible"]),
            "compatibilityKey": row["compatibility_key"],
            "compatibility": compatibility,
            "sourceManifest": source_manifest,
            "sourceManifestFingerprint": cls._source_manifest_fingerprint(source_manifest),
            "priorityRank": row["priority_rank"],
            "failureLifecycleKey": row["failure_lifecycle_key"],
            "dependencyLifecycleKey": row["dependency_lifecycle_key"],
            "dependencyFixedSha256": row["dependency_fixed_sha256"],
            "sourceCopyJobId": row["source_copy_job_id"],
            "targetDir": row["target_dir"],
            "commandFingerprint": row["command_fingerprint"],
            "jobId": row["job_id"],
            "status": row["status"],
            "createdAt": row["created_at"],
            "expiresAt": row["expires_at"],
            "startedAt": row["started_at"],
            "completedAt": row["completed_at"],
        }

    @staticmethod
    def _source_manifest_from_reservation_row(compatibility: dict[str, object]) -> dict[str, str]:
        """Expose an auditable manifest without touching workspace paths."""
        raw_manifest = compatibility.get("source_manifest")
        if isinstance(raw_manifest, dict):
            return {
                str(path): str(digest).upper()
                for path, digest in sorted(raw_manifest.items())
                if isinstance(path, str) and isinstance(digest, str)
            }
        try:
            raw_build_config = compatibility["build_config"]
            build_config = json.loads(raw_build_config)
            raw_manifest = build_config.get("source_manifest")
        except (AttributeError, KeyError, TypeError, ValueError, json.JSONDecodeError):
            return {}
        if not isinstance(raw_manifest, dict):
            return {}
        return {
            str(path): str(digest).upper()
            for path, digest in sorted(raw_manifest.items())
            if isinstance(path, str) and isinstance(digest, str)
        }

    @staticmethod
    def _source_manifest_fingerprint(source_manifest: dict[str, str]) -> str | None:
        if not source_manifest:
            return None
        payload = "\n".join(
            f"{path.casefold()}={digest.casefold()}"
            for path, digest in sorted(source_manifest.items())
        )
        return hashlib.sha256(payload.encode("utf-8")).hexdigest()

    def get(self, job_id: str) -> CargoJob:
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT * FROM cargo_jobs WHERE job_id = ?", (job_id,)
            ).fetchone()
        if row is None:
            raise CoordinatorError("cargo_job_not_found", f"Unknown Cargo job {job_id}")
        return self._from_row(row)

    def list(self) -> list[CargoJob]:
        with self.database.connect() as connection:
            rows = connection.execute(
                "SELECT * FROM cargo_jobs ORDER BY created_at, job_id"
            ).fetchall()
        return [self._from_row(row) for row in rows]

    def reconcile_orphans(
        self,
        *,
        now: datetime | None = None,
        leased_timeout_seconds: int = 300,
        running_health_timeout_seconds: int = 300,
    ) -> tuple[CargoJob, ...]:
        with self._start_reconcile_lock:
            return self._reconcile_orphans(
                now=now,
                leased_timeout_seconds=leased_timeout_seconds,
                running_health_timeout_seconds=running_health_timeout_seconds,
            )

    def _reconcile_orphans(
        self,
        *,
        now: datetime | None = None,
        leased_timeout_seconds: int = 300,
        running_health_timeout_seconds: int = 300,
    ) -> tuple[CargoJob, ...]:
        orphaned_ids: list[str] = []
        current_time = now or utc_now()
        now_text = utc_text(current_time)

        # Process-tree discovery can take seconds on a busy Windows host.  Do
        # not hold SQLite's write transaction while asking the OS for that
        # information, or normal control requests (tray, lease, reservation)
        # queue behind the watcher.
        with self.database.connect() as connection:
            snapshots = connection.execute(
                """
                SELECT *
                FROM cargo_jobs WHERE status IN ('leased', 'running')
                """
            ).fetchall()

        for snapshot in snapshots:
            snapshot_job = self._from_row(snapshot)
            if snapshot_job.job_id in self._managed_collectors:
                continue
            live_pids = self._live_process_pids(
                snapshot_job,
                include_supervisor_root=snapshot_job.status is not CargoJobStatus.LEASED,
            )
            root_identity_changed = (
                not live_pids and self._root_process_identity_changed(snapshot_job)
            )

            # Re-read under a short write transaction.  A job may have
            # finished, been replaced, or acquired a new supervisor while the
            # OS scan was running; in that case discard the stale observation.
            with self.database.transaction() as connection:
                row = connection.execute(
                    """
                    SELECT * FROM cargo_jobs
                    WHERE job_id=? AND status IN ('leased', 'running')
                    """,
                    (snapshot_job.job_id,),
                ).fetchone()
                if row is None:
                    continue
                if (
                    row["pid"] != snapshot["pid"]
                    or row["root_process_creation_time"]
                    != snapshot["root_process_creation_time"]
                    or row["root_process_kind"] != snapshot["root_process_kind"]
                ):
                    continue

                job = self._from_row(row)
                self._record_process_tree_observation(
                    connection, job.job_id, live_pids, now_text
                )
                if live_pids:
                    heartbeat_age_seconds = (
                        current_time - parse_utc(row["last_heartbeat_at"])
                    ).total_seconds()
                    if (
                        row["status"] == CargoJobStatus.RUNNING.value
                        and heartbeat_age_seconds > running_health_timeout_seconds
                        and job.job_id not in self._reported_health_timeouts
                    ):
                        # A live child cannot be safely preempted or have its
                        # lane reused.  Record the timeout once, then leave
                        # global admission open for all unrelated work.
                        connection.execute(
                            "INSERT INTO events(session_id, event_type, payload_json, created_at) "
                            "VALUES (?, ?, ?, ?)",
                            (
                                job.session_id,
                                "cargo.health_timeout",
                                json.dumps(
                                    {
                                        "jobId": job.job_id,
                                        "laneKind": job.lane_kind.value,
                                        "livePids": list(live_pids),
                                        "heartbeatAgeSeconds": int(heartbeat_age_seconds),
                                        "timeoutSeconds": running_health_timeout_seconds,
                                    },
                                    sort_keys=True,
                                ),
                                now_text,
                            ),
                        )
                        self._reported_health_timeouts.add(job.job_id)
                    elif heartbeat_age_seconds <= running_health_timeout_seconds:
                        self._reported_health_timeouts.discard(job.job_id)
                    continue
                self._reported_health_timeouts.discard(job.job_id)
                if (
                    row["status"] == CargoJobStatus.LEASED.value
                    and (current_time - parse_utc(row["last_heartbeat_at"])).total_seconds()
                    <= leased_timeout_seconds
                ):
                    continue
                if (
                    row["status"] == CargoJobStatus.RUNNING.value
                    and row["process_tree_exited_at"] is None
                    and not root_identity_changed
                ):
                    continue
                cursor = connection.execute(
                    """
                    UPDATE cargo_jobs
                    SET status = ?, finished_at = ?, last_heartbeat_at = ?
                    WHERE job_id = ? AND status IN ('leased', 'running')
                    """,
                    (CargoJobStatus.ORPHANED.value, now_text, now_text, row["job_id"]),
                )
                if cursor.rowcount != 1:
                    continue
                connection.execute(
                    """
                    UPDATE cargo_lane_reservations
                    SET status='expired', completed_at=COALESCE(completed_at, ?)
                    WHERE job_id=? AND status IN ('leased', 'running')
                    """,
                    (now_text, row["job_id"]),
                )
                orphaned_ids.append(row["job_id"])
        return tuple(self.get(job_id) for job_id in orphaned_ids)

    def reconcile_pending_reservations(
        self, *, now: datetime | None = None
    ) -> dict[str, int]:
        """Advance FIFO lanes past expired claims without touching managed jobs."""
        now_text = utc_text(now or utc_now())
        with self.database.transaction() as connection:
            expired_cpu, released_cpu, _ = reconcile_cpu_fifo_eligibility(
                connection, now=now_text
            )
            expired_gpu = expire_invalid_pending_lane_reservations(
                connection, lane_scope="gpu", now=now_text
            )
            released_gpu = reconcile_terminal_finished_lane_reservations(
                connection, lane_scope="gpu", now=now_text
            )
        return {
            "expiredCpu": expired_cpu,
            "expiredGpu": expired_gpu,
            "releasedCpu": released_cpu,
            "releasedGpu": released_gpu,
        }

    @staticmethod
    def _require_status(
        connection,
        job_id: str,
        allowed: set[CargoJobStatus],
        *,
        session_id: str,
    ):
        row = connection.execute(
            "SELECT * FROM cargo_jobs WHERE job_id = ?", (job_id,)
        ).fetchone()
        if row is None:
            raise CoordinatorError("cargo_job_not_found", f"Unknown Cargo job {job_id}")
        CargoJobService._require_owner(row, job_id, session_id)
        current = CargoJobStatus(row["status"])
        if current not in allowed:
            raise CoordinatorError(
                "invalid_cargo_job_status",
                f"Cargo job {job_id} is {current.value}; expected {sorted(item.value for item in allowed)}",
            )
        return row

    @staticmethod
    def _require_owner(row, job_id: str, session_id: str) -> None:
        if row["session_id"] != session_id:
            raise CoordinatorError(
                "cargo_job_owner_mismatch",
                f"Cargo job {job_id} belongs to Session {row['session_id']}",
            )

    @staticmethod
    def _from_row(row) -> CargoJob:
        def parsed(value):
            return parse_utc(value) if value else None

        return CargoJob(
            job_id=row["job_id"],
            session_id=row["session_id"],
            lane_kind=CargoLaneKind(row["lane_kind"]),
            target_dir=row["target_dir"],
            status=CargoJobStatus(row["status"]),
            dry_run=bool(row["dry_run"]),
            pid=row["pid"],
            command=tuple(json.loads(row["command_json"])),
            exit_code=row["exit_code"],
            created_at=parse_utc(row["created_at"]),
            last_heartbeat_at=parse_utc(row["last_heartbeat_at"]),
            started_at=parsed(row["started_at"]),
            finished_at=parsed(row["finished_at"]),
            released_at=parsed(row["released_at"]),
            reuse_key=row["reuse_key"],
            compatibility_json=row["compatibility_json"],
            compatibility_key=row["compatibility_key"],
            reuse_profile=row["reuse_profile"],
            cleanup_policy=CargoCleanupPolicy(row["cleanup_policy"]),
            cleanup_status=CargoCleanupStatus(row["cleanup_status"]),
            reused_from_job_id=row["reused_from_job_id"],
            source_copy_job_id=row["source_copy_job_id"],
            source_copy_manifest_hash=row["source_copy_manifest_hash"],
            cleanup_error=row["cleanup_error"],
            process_tree_observed_at=parsed(row["process_tree_observed_at"]),
            live_process_pids=tuple(json.loads(row["process_tree_live_pids_json"])),
            process_tree_exited_at=parsed(row["process_tree_exited_at"]),
            root_process_creation_time=row["root_process_creation_time"],
            root_process_kind=CargoProcessRootKind(row["root_process_kind"]),
        )
