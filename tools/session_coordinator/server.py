from __future__ import annotations

import json
import os
import secrets
import sqlite3
import subprocess
import threading
import time
import uuid
from dataclasses import asdict, dataclass
from datetime import date
from http import HTTPStatus
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any

from .config import CoordinatorConfig
from .baselines import BaselineService
from .database import Database
from .leases import LeaseService, PathPolicy, lease_paths_overlap
from .migrations import LATEST_SCHEMA_VERSION, migrate
from .models import CoordinatorError, SessionStatus, SupervisionState, utc_text
from .sessions import SessionService
from .patches import PatchService, PatchStatus
from .failures import FailureGraphService, FailureResolution
from .plans import PlanRepository
from .snapshots import ObjectStore, SnapshotService
from .watch import WorkspaceWatcher
from .cargo_jobs import CargoCompatibility, CargoJobService, CargoLaneKind, TargetPathPolicy
from .cargo_runner import CargoJobRunner
from .cleanup import CleanupService, RetentionService
from .artifact_governance import ArtifactGovernanceService
from .processes import current_process_identity, process_is_alive
from .git_finalize import GitFinalizeService
from .git_guard import remove_commit_guard
from .workspace_copy import WorkspaceCopyService
from .legacy import LegacyMigrationService
from .audit import RolloutAuditService
from .control_plane.auth import WebControlAuth
from .control_plane.actions.executor import ActionExecutor
from .control_plane.actions.fingerprint import ActionFingerprinter
from .control_plane.actions.service import ActionService
from .control_plane.artifact_downloads import ArtifactDownloadService
from .control_plane.assets import StaticAssetService
from .control_plane.events import EventStreamService
from .control_plane.http import ControlPlaneHttp
from .control_plane.router import ControlPlaneRouter
from .control_plane.snapshot import ControlSnapshotService
from .workflows.projections import WorkflowProjectionService
from .workflows.store import WorkflowStore
from .workflows.plan_import import TopologyImporter
from .workflows.milestones import MilestoneWorkflowService
from .notifications import WeComNotificationService
from .supervision.lifecycle import LifecycleService
from .supervision.repository_identity import repository_identity
from .supervision.runtime_descriptor import RuntimeDescriptor
from .supervision.service import SupervisionService
from .codex_sync.discovery import CodexSessionDiscovery
from .codex_sync.evidence import CodexEvidenceProjector
from .codex_sync.spool import CodexTriggerSpool
from .codex_sync.store import CodexSessionStore
from .codex_sync.worker import CodexSyncWorker


def _atomic_json_write(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_text(json.dumps(payload, indent=2, sort_keys=True), encoding="utf-8")
    os.replace(temporary, path)


def _pid_is_alive(pid: int) -> bool:
    return process_is_alive(pid)


class CoordinatorApplication:
    READ_ONLY_COMMANDS = frozenset(
        {
            "session.list",
            "session.show",
            "baseline.status",
            "baseline.diff",
            "watch.scan",
            "lease.list",
            "patch.status",
            "patch.list",
            "plan.audit",
            "plan.owner",
            "failure.audit",
            "failure.open",
            "cargo.list",
            "cargo.run_status",
            "cleanup.plan",
            "validation_copy.status",
            "legacy.report",
            "retention.show",
            "audit.all",
            "artifact.audit",
        }
    )
    # These commands either own an independent short SQLite transition or do
    # their long work after one.  Holding the generic foreground RLock around
    # them turns a disconnected caller into a five-minute outage for Cargo
    # lifecycle and lease writes.  Shared-worktree patch/finalize/failure work
    # deliberately stays serialized by the normal command path.
    NON_BLOCKING_MUTATION_COMMANDS = frozenset(
        {
            "session.register",
            "session.heartbeat",
            "baseline.init",
            "baseline.scan",
            "baseline.attribute",
            "baseline.accept",
            "baseline.reconcile",
            "lease.claim",
            "lease.release",
            "cargo.acquire",
            "cargo.reserve_cpu",
            "cargo.reserve_gpu",
            "cargo.renew_cpu_reservation",
            "cargo.consume_cpu_reservation",
            "cargo.consume_gpu_reservation",
            "cargo.recover_expired_reservation",
            "cargo.run_reserved",
            "cargo.start",
            "cargo.run",
            "cargo.heartbeat",
            "cargo.finish",
            "cargo.release",
            "validation_copy.materialize",
            "validation_copy.run",
            "validation_copy.cleanup",
            "artifact.cleanup",
        }
    )

    def __init__(
        self,
        config: CoordinatorConfig,
        *,
        instance_id: str | None = None,
        started_at: str | None = None,
        automatic_start: bool = False,
    ):
        self.config = config
        self.instance_id = instance_id or uuid.uuid4().hex
        self.started_at = started_at or utc_text()
        self.database = Database(config.database_path)
        migrate(self.database)
        self.workflows = WorkflowStore(self.database)
        self.sessions = SessionService(
            self.database,
            config.repo_root,
            session_change_hook=self.workflows.synchronize_session_in_connection,
        )
        self._mutation_lock = threading.RLock()
        self._maintenance_lock = threading.Lock()
        self.baselines = BaselineService(self.database, config.repo_root)
        self.object_store = ObjectStore(self.database, config.object_root)
        self.snapshots = SnapshotService(
            self.database, config.repo_root, self.object_store
        )
        self.leases = LeaseService(
            self.database,
            PathPolicy(config.repo_root),
            ttl_seconds=config.lease_ttl_seconds,
            grace_seconds=config.lease_grace_seconds,
        )
        self.patches = PatchService(
            self.database,
            config.repo_root,
            self.object_store,
            self.snapshots,
            self.leases,
            self.sessions,
        )
        self.watcher = WorkspaceWatcher(self.baselines)
        self.plans = PlanRepository(config.repo_root)
        self.failures = FailureGraphService(self.database, config.repo_root)
        self.legacy = LegacyMigrationService(
            self.database, config.repo_root, self.sessions, process_alive=process_is_alive
        )
        self.legacy.recover_interrupted_archives()
        self.retention = RetentionService(self.database, self.object_store)
        self.retention.recover_interrupted()
        self.finalize = GitFinalizeService(
            self.database,
            config.repo_root,
            self.baselines,
            self.sessions,
            self.plans,
            self.failures,
        )
        self.finalize.recover_stale_mutex()
        if config.enabled_target_roots:
            self.cargo_jobs: CargoJobService | None = CargoJobService(
                self.database,
                TargetPathPolicy(config.enabled_target_roots),
                repo_root=config.repo_root,
            )
            self.cargo_runner: CargoJobRunner | None = CargoJobRunner(
                self.database,
                self.cargo_jobs,
                repo_root=config.repo_root,
                log_root=config.cargo_run_log_root,
            )
            self.cargo_runner.reconcile_terminal_runs()
            self.cleanup: CleanupService | None = CleanupService(
                self.database,
                self.cargo_jobs,
                process_alive=process_is_alive,
            )
            self.cleanup.recover_reservations()
            self._record_startup_gpu_lane_audit()
            self.workspace_copy: WorkspaceCopyService | None = WorkspaceCopyService(
                self.database,
                config.repo_root,
                tuple(
                    root
                    for root in config.enabled_target_roots
                    if root.name.casefold() == "cargo-targets"
                ),
                mutation_gate=lambda: self._mutation_lock,
            )
            self.workspace_copy.recover_interrupted_jobs()
        else:
            self.cargo_jobs = None
            self.cargo_runner = None
            self.cleanup = None
            self.workspace_copy = None
        self.artifact_governance = (
            ArtifactGovernanceService(self.database, roots=config.enabled_target_roots)
            if config.unmanaged_artifact_sweep_enabled and config.enabled_target_roots
            else None
        )
        self.rollout_audit = RolloutAuditService(
            self.database,
            config.repo_root,
            sessions=self.sessions,
            baselines=self.baselines,
            plans=self.plans,
            failures=self.failures,
            legacy=self.legacy,
            target_roots=config.enabled_target_roots,
        )
        self.branch = self._branch()
        self.process_identity = current_process_identity()
        self.repository_identity = repository_identity(config.repo_root)
        self.supervision = SupervisionService(
            self.database,
            repository_key=self.repository_identity.key,
            daemon_instance_id=self.instance_id,
            process_creation_time=self.process_identity.creation_time,
            maintenance_active=self._maintenance_lock.locked,
            maintenance_session_ids=self._maintenance_session_ids_for_startup(),
        )
        self.supervision.initialize(automatic_start=automatic_start)
        if config.codex_home is None or config.codex_spool_base is None:
            raise CoordinatorError("codex_config_invalid", "Codex sync roots are unavailable")
        self.codex_discovery = CodexSessionDiscovery(config.codex_home, config.repo_root)
        self.codex_spool = CodexTriggerSpool(
            config.codex_spool_base, self.repository_identity.key
        )
        self.codex_store = CodexSessionStore(self.database)
        self.codex_evidence = CodexEvidenceProjector(
            self.database,
            codex_home=config.codex_home or config.state_root,
            repo_root=config.repo_root,
        )
        self.codex_worker = CodexSyncWorker(
            discover=self.codex_discovery.discover,
            store=self.codex_store,
            spool=self.codex_spool,
            writable=self._codex_sync_writable,
            project=lambda result, include_history=False: self.codex_evidence.project(
                run_id=result.run_id,
                include_history=include_history,
            ),
            membership_interval_seconds=config.codex_membership_interval_seconds,
            full_interval_seconds=config.codex_full_interval_seconds,
        )
        self.lifecycle = LifecycleService(self.supervision)
        self.workflow_projections = WorkflowProjectionService()
        self.topology_importer = TopologyImporter(self.database, config.repo_root)
        self.notifications = WeComNotificationService(self.database)
        self.notifications.recover_reserved()
        self.milestone_workflows = MilestoneWorkflowService(
            self.database,
            config.repo_root,
            self.baselines,
            self.finalize,
            self.notifications,
            sessions=self.sessions,
            leases=self.leases,
            failures=self.failures,
        )
        self.milestone_workflows.recover_pending_commits()
        if self.workspace_copy is not None:
            self.workspace_copy.set_completion_hook(
                self.milestone_workflows.import_validation_result
            )
            self.milestone_workflows.recover_validation_results()
        self.workflows.synchronize_sessions(self.sessions.list(include_archived=True))
        self.web_auth = WebControlAuth(self.database)
        self.control_actions = ActionService(
            self.database,
            ActionFingerprinter(
                self.database,
                config.repo_root,
                daemon_instance_id=self.instance_id,
                supervision=self.supervision,
            ),
            ActionExecutor(
                sessions=self.sessions,
                leases=self.leases,
                patches=self.patches,
                failures=self.failures,
                workspace_copy=self.workspace_copy,
                workflows=self.workflows,
                topology_importer=self.topology_importer,
                milestones=self.milestone_workflows,
                lifecycle=self.lifecycle,
                git_finalize=self.finalize,
                codex_wake=self.codex_worker.wake,
            ),
            daemon_instance_id=self.instance_id,
            mutation_gate=self.supervision.require_mutation_allowed,
        )
        self.control_actions.recover_interrupted_actions()
        self.control_events = EventStreamService(self.database)
        self.control_snapshot = ControlSnapshotService(
            self.database,
            self.workflow_projections,
            self.control_service_state,
            repo_root=self.config.repo_root,
        )

    @property
    def read_only(self) -> bool:
        return self.branch != "main"

    def health(self) -> dict[str, Any]:
        try:
            baseline_health = self.baselines.current().health.value
        except CoordinatorError as error:
            if error.code != "baseline_missing":
                raise
            baseline_health = "uninitialized"
        return {
            "status": "ok",
            "branch": self.branch,
            "mode": "read_only" if self.read_only else "read_write",
            "repo_root": str(self.config.repo_root),
            "pid": os.getpid(),
            "baseline": baseline_health,
            "instance_id": self.instance_id,
            "started_at": self.started_at,
            "control_api_versions": [1],
            "supervision_api_versions": [1],
            "schema_version": LATEST_SCHEMA_VERSION,
            "repository_key": self.repository_identity.key,
            "process_creation_time": self.process_identity.creation_time,
            "executable": self.process_identity.executable,
            "supervision": self.supervision.snapshot().to_dict(),
            "codex_sync": self.codex_worker.snapshot(),
        }

    def _codex_sync_writable(self) -> bool:
        if self.read_only:
            return False
        try:
            self.supervision.require_mutation_allowed("codex.sessions.reconcile")
        except CoordinatorError:
            return False
        return True

    @staticmethod
    def _maintenance_session_ids_from_environment() -> tuple[str, ...]:
        raw_values = (
            os.environ.get("ZIRCON_COORDINATOR_MAINTENANCE_SESSION", ""),
            os.environ.get("ZIRCON_COORDINATOR_MAINTENANCE_SESSIONS", ""),
        )
        return tuple(
            session_id
            for value in raw_values
            for session_id in (item.strip() for item in value.split(","))
            if session_id
        )

    def _maintenance_session_ids_for_startup(self) -> tuple[str, ...]:
        environment_ids = self._maintenance_session_ids_from_environment()
        with self.database.connect() as connection:
            state = connection.execute(
                "SELECT maintenance_hold FROM service_recovery_state WHERE repository_key=?",
                (self.repository_identity.key,),
            ).fetchone()
            if state is None or not bool(state["maintenance_hold"]):
                return environment_ids
            row = connection.execute(
                """SELECT parameters_json FROM action_requests
                   WHERE action_kind='service.drain'
                     AND status='succeeded'
                     AND completed_at IS NOT NULL
                   ORDER BY completed_at DESC, action_id DESC
                   LIMIT 1""",
            ).fetchone()
        if row is None:
            return environment_ids
        try:
            payload = json.loads(row["parameters_json"])
            raw_ids = payload.get("maintenanceSessionIds", [])
            if not isinstance(raw_ids, list):
                return environment_ids
            durable_ids = tuple(
                session_id.strip()
                for session_id in raw_ids
                if isinstance(session_id, str) and session_id.strip()
            )
        except (TypeError, ValueError, json.JSONDecodeError):
            return environment_ids
        return tuple(dict.fromkeys((*durable_ids, *environment_ids)))

    def control_service_state(self, connection) -> dict[str, object]:
        baseline = connection.execute(
            "SELECT * FROM baseline_epochs ORDER BY epoch_id DESC LIMIT 1"
        ).fetchone()
        baseline_state = "uninitialized" if baseline is None else baseline["health"]
        supervision = self.supervision.snapshot(connection).to_dict()
        codex_sync = self.codex_worker.snapshot()
        codex_sync["queueDepth"] = self.codex_spool.pending_count()
        return {
            "status": "ok",
            "branch": self.branch,
            "mode": "read_only" if self.read_only else "read_write",
            "baseline": baseline_state,
            "instanceId": self.instance_id,
            "startedAt": self.started_at,
            "sessionTtlSeconds": self.config.session_ttl_seconds,
            "controlApiVersions": [1],
            "supervisionApiVersions": [1],
            "schemaVersion": LATEST_SCHEMA_VERSION,
            "repositoryKey": self.repository_identity.key,
            "processCreationTime": self.process_identity.creation_time,
            "supervision": supervision,
            "codexSync": codex_sync,
        }

    def command(self, name: str, arguments: dict[str, Any]) -> dict[str, Any]:
        if name in self.READ_ONLY_COMMANDS:
            return self._command_unlocked(name, arguments)
        if name in self.NON_BLOCKING_MUTATION_COMMANDS:
            if self.read_only:
                return self._command_unlocked(name, arguments)
            self.supervision.require_mutation_allowed(
                self._mutation_operation(name, arguments)
            )
            return self._command_unlocked(name, arguments)
        if name == "supervision.force_stop_ack":
            with self._mutation_lock:
                return self._command_unlocked(name, arguments)
        if self.read_only:
            return self._command_unlocked(name, arguments)
        if name == "legacy.import" and bool(arguments.get("apply")):
            self._require_maintenance_capability(arguments)
        self.supervision.require_mutation_allowed(self._mutation_operation(name, arguments))
        with self._mutation_lock:
            return self._command_unlocked(name, arguments)

    @staticmethod
    def _mutation_operation(name: str, arguments: dict[str, Any]) -> str:
        session_id = arguments.get("session_id")
        if isinstance(session_id, str) and session_id:
            return f"{name}@{session_id}"
        return name

    def _require_scoped_failure_return_leases(
        self,
        session_id: str,
        lifecycle_key: str,
        resolved_at: date,
    ) -> None:
        """Keep maintenance-mode failure returns bound to their exact artifacts."""
        with self.database.connect() as connection:
            rows = connection.execute(
                "SELECT * FROM failure_nodes WHERE lifecycle_key = ?", (lifecycle_key,)
            ).fetchall()
        if len(rows) != 1:
            raise CoordinatorError(
                "ambiguous_failure_lifecycle",
                f"Expected one open artifact for lifecycle {lifecycle_key}; found {len(rows)}",
            )
        node = self.failures._node_from_row(rows[0])
        if node.kind != "failure" or node.status != "open":
            raise CoordinatorError("failure_not_open", "Only an open failure can be returned")

        source = self.config.repo_root / node.artifact_path
        destination = (
            self.config.repo_root
            / node.origin_child_dir
            / f"fixed-{resolved_at.isoformat()}-{node.summary_slug}.md"
        )
        fixer_paths = [source]
        source_text = source.read_text(encoding="utf-8")
        if self.failures._is_child_record_only(source_text):
            fixer_paths.append(
                self.config.repo_root
                / node.fixing_child_dir
                / f"{resolved_at.isoformat()}-{node.summary_slug}-return.md"
            )
        else:
            fixer_paths.extend(
                (
                    destination,
                    self.config.repo_root / node.origin_plan,
                    self.config.repo_root / node.fixing_plan,
                )
            )
        self.leases.require_owned_live(
            session_id,
            [path.relative_to(self.config.repo_root).as_posix() for path in fixer_paths],
            error_code="failure_return_lease_missing",
            message="Scoped failure return requires live leases for every affected artifact",
        )
        if self.failures._is_child_record_only(source_text):
            try:
                self.leases.require_owned_live(
                    session_id,
                    [destination.relative_to(self.config.repo_root).as_posix()],
                    error_code="failure_return_lease_missing",
                    message="Scoped failure return requires live leases for every affected artifact",
                )
            except CoordinatorError as error:
                if error.code != "failure_return_lease_missing":
                    raise
                origin_owner = self._require_origin_destination_lease(node, destination)
            else:
                origin_owner = None
            if origin_owner is None:
                return
            with self.database.transaction() as connection:
                connection.execute(
                    "INSERT INTO events(session_id, event_type, payload_json, created_at) VALUES (?, ?, ?, ?)",
                    (
                        session_id,
                        "failure.return_origin_destination_authorized",
                        json.dumps(
                            {
                                "lifecycleKey": lifecycle_key,
                                "destination": destination.relative_to(self.config.repo_root).as_posix(),
                                "originOwnerSessionId": origin_owner,
                                "originPlan": node.origin_plan,
                            },
                            sort_keys=True,
                        ),
                        utc_text(),
                    ),
                )

    def _require_origin_destination_lease(self, node, destination: Path) -> str:
        """Allow a child-only fixed record under its active origin-plan lease.

        The fixing Session never receives or releases the origin lease.  This is
        a narrow coordinator lifecycle transfer for the one generated fixed
        artifact, not a general cross-session write exception.
        """
        destination_key = destination.relative_to(self.config.repo_root).as_posix().casefold()
        expected_plan = node.origin_plan.casefold()
        with self.database.connect() as connection:
            rows = connection.execute(
                """
                SELECT leases.path_key, leases.session_id, sessions.plan_path
                FROM leases
                JOIN sessions ON sessions.session_id = leases.session_id
                WHERE sessions.status IN ('active', 'resolving_failure', 'waiting_validation')
                  AND leases.expires_at >= ?
                """,
                (utc_text(),),
            ).fetchall()
        for row in rows:
            plan_path = str(row["plan_path"] or "").replace("\\", "/").casefold()
            if plan_path != expected_plan:
                continue
            if lease_paths_overlap(str(row["path_key"]), destination_key):
                return str(row["session_id"])
        raise CoordinatorError(
            "failure_return_lease_missing",
            "Scoped failure return requires the active origin-plan lease for its fixed destination",
            details={
                "paths": [destination.relative_to(self.config.repo_root).as_posix()],
                "originPlan": node.origin_plan,
            },
        )

    def _command_unlocked(self, name: str, arguments: dict[str, Any]) -> dict[str, Any]:
        if self.read_only and name not in self.READ_ONLY_COMMANDS:
            raise CoordinatorError(
                "not_on_main",
                f"Coordinator mutations require main; current branch is {self.branch}",
            )
        if name == "supervision.recovery_record":
            snapshot = self.supervision.record_recovery(
                failure_count=arguments.get("failureCount"),
                failure_window_started_at=arguments.get("failureWindowStartedAt"),
                next_retry_at=arguments.get("nextRetryAt"),
                circuit_open_until=arguments.get("circuitOpenUntil"),
                healthy_since=arguments.get("healthySince"),
            )
            return {"supervision": snapshot.to_dict()}
        if name == "supervision.force_stop_ack":
            return self.lifecycle.acknowledge_force_stop(str(arguments["actionId"]))
        if name == "session.register":
            session = self.sessions.register(
                session_id=str(arguments["session_id"]),
                display_name=arguments.get("display_name"),
                plan_path=arguments.get("plan_path"),
                write_scope=arguments.get("write_scope") or [],
            )
            if session.plan_path:
                self.failures.import_repository()
                open_failures = self.failures.open_for_plan(session.plan_path)
            else:
                open_failures = []
            if open_failures:
                session = self.sessions.set_status(
                    session.session_id,
                    SessionStatus.RESOLVING_FAILURE,
                    reason=f"{len(open_failures)} open failure handoff(s) require priority",
                )
            return {
                "session": session.to_dict(),
                "open_failures": [asdict(item) for item in open_failures],
            }
        if name == "session.list":
            sessions = self.sessions.list(include_archived=bool(arguments.get("include_archived")))
            return {"sessions": [session.to_dict() for session in sessions]}
        if name == "session.show":
            return {"session": self.sessions.get(str(arguments["session_id"])).to_dict()}
        if name == "session.heartbeat":
            session_id = str(arguments["session_id"])
            session = self.sessions.heartbeat(session_id)
            return {
                "session": session.to_dict(),
                "leases": {"renewed": self.leases.heartbeat(session_id)},
            }
        if name == "session.set_status":
            status = SessionStatus(str(arguments["status"]))
            session_id = str(arguments["session_id"])
            if status is SessionStatus.COMPLETED and self.sessions.get(session_id).plan_path:
                raise CoordinatorError(
                    "session_goal_close_requires_milestone",
                    "Numbered-plan Sessions must use 'milestone close-goal' after "
                    "accepted milestone commits; generic completion cannot bypass Git "
                    "and workflow evidence.",
                )
            session = self.sessions.set_status(
                session_id, status, reason=arguments.get("reason")
            )
            return {"session": session.to_dict()}
        if name == "baseline.init":
            return {"baseline": self._baseline_dict(self.baselines.initialize())}
        if name == "baseline.status":
            return {"baseline": self._baseline_dict(self.baselines.current())}
        if name in {"baseline.diff", "baseline.scan"}:
            changes = self.baselines.scan() if name == "baseline.scan" else self.baselines.diff()
            return {"changes": [asdict(change) for change in changes]}
        if name == "baseline.attribute":
            session_id = str(arguments["session_id"])
            paths = arguments.get("paths") or []
            self.leases.require_owned_live(
                session_id,
                paths,
                error_code="baseline_lease_missing",
                message="Path attribution requires a live lease owned by the Session",
            )
            self.baselines.attribute(session_id, paths)
            return {"status": "attributed"}
        if name == "baseline.accept":
            baseline = self.baselines.accept(reason=str(arguments["reason"]))
            return {"baseline": self._baseline_dict(baseline)}
        if name == "baseline.reconcile":
            baseline = self.baselines.reconcile_health()
            return {"baseline": self._baseline_dict(baseline)}
        if name == "lease.claim":
            result = self.leases.acquire(str(arguments["session_id"]), arguments.get("paths") or [])
            return {"lease": asdict(result)}
        if name == "lease.release":
            released = self.leases.release(
                str(arguments["session_id"]), arguments.get("paths")
            )
            processed = self.patches.process_queue()
            return {
                "released": released,
                "processed_patches": [self._patch_dict(patch) for patch in processed],
            }
        if name == "lease.heartbeat":
            return {"renewed": self.leases.heartbeat(str(arguments["session_id"]))}
        if name == "lease.list":
            return {"leases": self.leases.list()}
        if name == "snapshot.create":
            snapshot = self.snapshots.create(
                session_id=str(arguments["session_id"]),
                paths=arguments.get("paths") or [],
                baseline_epoch=arguments.get("baseline_epoch"),
                purpose=str(arguments["purpose"]),
            )
            return {"snapshot": asdict(snapshot)}
        if name == "snapshot.preview":
            preview = self.snapshots.restore_preview(int(arguments["snapshot_id"]))
            return {"preview": [asdict(item) for item in preview]}
        if name == "patch.enqueue":
            patch = self.patches.submit(
                str(arguments["session_id"]),
                str(arguments["patch_text"]),
                arguments.get("targets") or [],
            )
            return {"patch": self._patch_dict(patch)}
        if name == "patch.status":
            return {"patch": self._patch_dict(self.patches.get(int(arguments["patch_id"])))}
        if name == "patch.list":
            requested_status = arguments.get("status")
            status = PatchStatus(str(requested_status)) if requested_status else None
            return {"patches": [self._patch_dict(item) for item in self.patches.list(status=status)]}
        if name == "patch.process":
            return {"patches": [self._patch_dict(item) for item in self.patches.process_queue()]}
        if name == "watch.scan":
            return {"changes": [asdict(item) for item in self.watcher.scan_once()]}
        if name == "plan.audit":
            inventory = self.plans.scan()
            return {
                "formal_plans": [asdict(item) for item in inventory.formal_plans],
                "legacy_documents": list(inventory.legacy_documents),
            }
        if name == "plan.owner":
            return {"owner": asdict(self.plans.resolve_owner(str(arguments["plan_path"])))}
        if name == "plan.authorize":
            session = self.sessions.get(str(arguments["session_id"]))
            if not session.plan_path:
                raise CoordinatorError(
                    "session_plan_missing", "Session must register a numbered plan before plan writes"
                )
            decision = self.plans.authorize_write(
                session.plan_path,
                str(arguments["target_path"]),
                maintenance=bool(arguments.get("maintenance")),
            )
            return {"decision": asdict(decision)}
        if name == "failure.import":
            return {"audit": self._failure_audit_dict(self.failures.import_repository())}
        if name == "failure.audit":
            return {"audit": self._failure_audit_dict(self.failures.audit())}
        if name == "failure.open":
            nodes = self.failures.open_for_plan(str(arguments["fixing_plan"]))
            return {"failures": [asdict(item) for item in nodes]}
        if name == "failure.return":
            session_id = arguments.get("session_id")
            if isinstance(session_id, str) and session_id:
                self._require_scoped_failure_return_leases(
                    session_id,
                    str(arguments["lifecycle_key"]),
                    date.fromisoformat(str(arguments["resolved_at"])),
                )
            destination = self.failures.return_fixed(
                str(arguments["lifecycle_key"]),
                FailureResolution(
                    root_cause=str(arguments["root_cause"]),
                    architecture_fix=str(arguments["architecture_fix"]),
                    validation=str(arguments["validation"]),
                    return_summary=str(arguments["return_summary"]),
                ),
                resolved_at=date.fromisoformat(str(arguments["resolved_at"])),
            )
            return {"fixed_artifact": destination.relative_to(self.config.repo_root).as_posix()}
        if name == "cargo.acquire":
            self._require_artifact_governance_clean()
            cargo_jobs = self._require_cargo_jobs()
            compatibility_payload = arguments.get("compatibility")
            compatibility = None
            if compatibility_payload is not None:
                if not isinstance(compatibility_payload, dict):
                    raise CoordinatorError(
                        "invalid_cargo_compatibility",
                        "Cargo compatibility must be a JSON object",
                    )
                try:
                    compatibility = CargoCompatibility(**compatibility_payload)
                except TypeError as error:
                    raise CoordinatorError(
                        "invalid_cargo_compatibility",
                        f"Cargo compatibility fields are invalid: {error}",
                    ) from error
            job = cargo_jobs.acquire(
                str(arguments["session_id"]),
                CargoLaneKind(str(arguments["lane_kind"])),
                requested_target=arguments.get("target_dir"),
                dry_run=bool(arguments.get("dry_run")),
                owner_pid=int(arguments["pid"]) if arguments.get("pid") else None,
                ephemeral=bool(arguments.get("ephemeral")),
                compatibility=compatibility,
            )
            cleanup_scheduled = self._require_cleanup().schedule_pending_cleanup()
            return {"job": job.to_dict(), "cleanup_scheduled": cleanup_scheduled}
        if name == "cargo.reserve_cpu":
            compatibility_payload = arguments.get("compatibility")
            if not isinstance(compatibility_payload, dict):
                raise CoordinatorError(
                    "invalid_cargo_compatibility",
                    "CPU lane reservation requires a compatibility JSON object",
                )
            try:
                compatibility = CargoCompatibility(**compatibility_payload)
            except TypeError as error:
                raise CoordinatorError(
                    "invalid_cargo_compatibility",
                    f"Cargo compatibility fields are invalid: {error}",
                ) from error
            target_dir = arguments.get("target_dir")
            if target_dir is not None and not isinstance(target_dir, str):
                raise CoordinatorError(
                    "cargo_cpu_reservation_target_invalid",
                    "CPU lane reservation target_dir must be a text path when supplied",
                )
            burst_eligible = arguments.get("burst_eligible", False)
            if not isinstance(burst_eligible, bool):
                raise CoordinatorError(
                    "cargo_cpu_burst_eligibility_invalid",
                    "CPU burst eligibility must be a boolean when supplied",
                )
            reservation = self._require_cargo_jobs().reserve_cpu(
                str(arguments["session_id"]),
                compatibility=compatibility,
                target_dir=target_dir,
                command=arguments.get("command") or [],
                ttl_seconds=int(arguments.get("ttl_seconds", 900)),
                burst_eligible=burst_eligible,
            )
            return {"reservation": reservation}
        if name == "cargo.reserve_gpu":
            required = {"session_id", "compatibility", "target_dir", "ttl_seconds", "command"}
            if set(arguments) != required or not isinstance(arguments.get("target_dir"), str):
                raise CoordinatorError(
                    "cargo_gpu_reservation_arguments_invalid",
                    "GPU reservation requires only session_id, compatibility, target_dir, ttl_seconds, and command",
                )
            compatibility_payload = arguments.get("compatibility")
            if not isinstance(compatibility_payload, dict):
                raise CoordinatorError(
                    "invalid_cargo_compatibility",
                    "GPU lane reservation requires a compatibility JSON object",
                )
            try:
                compatibility = CargoCompatibility(**compatibility_payload)
            except TypeError as error:
                raise CoordinatorError(
                    "invalid_cargo_compatibility",
                    f"Cargo compatibility fields are invalid: {error}",
                ) from error
            reservation = self._require_cargo_jobs().reserve_gpu(
                str(arguments["session_id"]),
                compatibility=compatibility,
                target_dir=str(arguments["target_dir"]),
                command=arguments.get("command") or [],
                ttl_seconds=int(arguments.get("ttl_seconds", 900)),
            )
            return {"reservation": reservation}
        if name == "cargo.release_cpu_reservation":
            reservation = self._require_cargo_jobs().release_cpu_reservation(
                str(arguments["reservation_id"]),
                session_id=str(arguments["session_id"]),
            )
            return {"reservation": reservation}
        if name == "cargo.renew_cpu_reservation":
            reservation = self._require_cargo_jobs().renew_cpu_reservation(
                str(arguments["reservation_id"]),
                session_id=str(arguments["session_id"]),
                ttl_seconds=int(arguments.get("ttl_seconds", 900)),
            )
            return {"reservation": reservation}
        if name == "cargo.consume_cpu_reservation":
            required = {"session_id", "reservation_id", "lane_kind"}
            if set(arguments) != required or any(
                not isinstance(arguments.get(field), str) or not arguments[field]
                for field in required
            ):
                raise CoordinatorError(
                    "cargo_reservation_consume_arguments_invalid",
                    "CPU reservation consumption accepts only session_id, reservation_id, and lane_kind",
                )
            try:
                lane_kind = CargoLaneKind(str(arguments["lane_kind"]))
            except ValueError as error:
                raise CoordinatorError(
                    "cargo_reservation_consume_arguments_invalid",
                    "CPU reservation consumption requires a known CPU lane kind",
                ) from error
            if lane_kind is CargoLaneKind.GPU:
                raise CoordinatorError(
                    "cargo_reservation_consume_arguments_invalid",
                    "CPU reservation consumption cannot create a GPU job",
                )
            job = self._require_cargo_jobs().consume_cpu_reservation(
                str(arguments["reservation_id"]),
                session_id=str(arguments["session_id"]),
                lane_kind=lane_kind,
            )
            cleanup_scheduled = self._require_cleanup().schedule_pending_cleanup()
            return {"job": job.to_dict(), "cleanup_scheduled": cleanup_scheduled}
        if name == "cargo.consume_gpu_reservation":
            required = {"session_id", "reservation_id"}
            if set(arguments) != required or any(
                not isinstance(arguments.get(field), str) or not arguments[field]
                for field in required
            ):
                raise CoordinatorError(
                    "cargo_reservation_consume_arguments_invalid",
                    "GPU reservation consumption accepts only session_id and reservation_id",
                )
            job = self._require_cargo_jobs().consume_gpu_reservation(
                str(arguments["reservation_id"]),
                session_id=str(arguments["session_id"]),
            )
            cleanup_scheduled = self._require_cleanup().schedule_pending_cleanup()
            return {"job": job.to_dict(), "cleanup_scheduled": cleanup_scheduled}
        if name == "cargo.recover_expired_reservation":
            required = {"session_id", "reservation_id", "job_id"}
            if set(arguments) != required or any(
                not isinstance(arguments.get(field), str) or not arguments[field]
                for field in required
            ):
                raise CoordinatorError(
                    "cargo_reservation_recovery_arguments_invalid",
                    "Reservation recovery accepts only session_id, reservation_id, and job_id",
                )
            job = self._require_cargo_jobs().recover_expired_reservation(
                str(arguments["reservation_id"]),
                job_id=str(arguments["job_id"]),
                session_id=str(arguments["session_id"]),
            )
            return {"job": job.to_dict()}
        if name == "cargo.start":
            job = self._require_cargo_jobs().start(
                str(arguments["job_id"]),
                session_id=str(arguments["session_id"]),
                pid=int(arguments["pid"]),
                command=arguments.get("command") or [],
                root_is_supervisor=bool(arguments.get("root_is_supervisor")),
            )
            return {"job": job.to_dict()}
        if name == "cargo.run":
            runner = self._require_cargo_runner()
            run = runner.start(
                session_id=str(arguments["session_id"]),
                job_id=str(arguments["job_id"]),
                command=arguments.get("command") or [],
                environment=arguments.get("environment"),
            )
            return {"run": run.to_dict()}
        if name == "cargo.run_reserved":
            required = {"session_id", "reservation_id", "job_id", "command"}
            command = arguments.get("command")
            if (
                set(arguments) != required
                or any(
                    not isinstance(arguments.get(field), str) or not arguments[field]
                    for field in ("session_id", "reservation_id", "job_id")
                )
                or not isinstance(command, list)
                or not command
                or any(not isinstance(part, str) or not part for part in command)
            ):
                raise CoordinatorError(
                    "cargo_reservation_run_arguments_invalid",
                    "Reserved Cargo run accepts only session_id, reservation_id, job_id, and command",
                )
            environment = self._require_cargo_jobs().reserved_run_environment(
                str(arguments["reservation_id"]),
                session_id=str(arguments["session_id"]),
                job_id=str(arguments["job_id"]),
                command=command,
            )
            run = self._require_cargo_runner().start(
                session_id=str(arguments["session_id"]),
                job_id=str(arguments["job_id"]),
                command=command,
                environment=environment,
            )
            return {"run": run.to_dict()}
        if name == "cargo.run_status":
            return {
                "run": self._require_cargo_runner().status(
                    str(arguments["job_id"]), session_id=str(arguments["session_id"])
                )
            }
        if name == "cargo.heartbeat":
            return {
                "job": self._require_cargo_jobs().heartbeat(
                    str(arguments["job_id"]),
                    session_id=str(arguments["session_id"]),
                ).to_dict()
            }
        if name == "cargo.finish":
            return {
                "job": self._require_cargo_jobs().finish(
                    str(arguments["job_id"]),
                    session_id=str(arguments["session_id"]),
                    exit_code=int(arguments["exit_code"]),
                ).to_dict()
            }
        if name == "cargo.release":
            job = self._require_cargo_jobs().release(
                str(arguments["job_id"]),
                session_id=str(arguments["session_id"]),
            )
            cleanup_scheduled = self._require_cleanup().schedule_pending_cleanup()
            return {
                "job": job.to_dict(),
                "cleanup_scheduled": cleanup_scheduled,
            }
        if name == "cargo.list":
            return {"jobs": [job.to_dict() for job in self._require_cargo_jobs().list()]}
        if name == "cleanup.plan":
            plan = self._require_cleanup().plan(older_than_hours=int(arguments.get("older_than_hours", 2)))
            return {"plan": self._cleanup_plan_dict(plan)}
        if name == "cleanup.apply":
            self._require_maintenance_capability(arguments)
            cleanup = self._require_cleanup()
            older_than_hours = int(arguments.get("older_than_hours", 2))
            plan = cleanup.get_plan(str(arguments["plan_id"]))
            if plan.older_than_hours != older_than_hours:
                raise CoordinatorError(
                    "cleanup_plan_retention_mismatch",
                    "cleanup.apply retention must match the reviewed plan",
                )
            result = cleanup.apply(plan)
            return {
                "plan": self._cleanup_plan_dict(plan),
                "result": {
                    "deleted": list(result.deleted),
                    "denied": [asdict(item) for item in result.denied],
                },
            }
        if name == "artifact.audit":
            return {"unmanaged": self._artifact_governance_paths()}
        if name == "artifact.cleanup":
            governance = self._require_artifact_governance()
            result = governance.cleanup()
            return {
                "deleted": list(result.deleted),
                "failed": [item.path for item in result.failed],
                "remaining": self._artifact_governance_paths(),
            }
        if name == "finalize.preview":
            maintenance = self._authorize_maintenance(arguments)
            preview = self.finalize.preview(
                str(arguments["session_id"]),
                paths=tuple(str(path) for path in arguments.get("paths") or ()),
                message=str(arguments["message"]),
                validation_commands=tuple(
                    tuple(str(part) for part in command)
                    for command in arguments.get("validation_commands") or ()
                ),
                maintenance=maintenance,
            )
            return {"preview": preview.to_dict()}
        if name == "finalize.commit":
            maintenance = self._authorize_maintenance(arguments)
            result = self.finalize.finalize(
                str(arguments["session_id"]),
                paths=tuple(str(path) for path in arguments.get("paths") or ()),
                message=str(arguments["message"]),
                validation_commands=tuple(
                    tuple(str(part) for part in command)
                    for command in arguments.get("validation_commands") or ()
                ),
                maintenance=maintenance,
            )
            return {"result": result.to_dict()}
        if name == "finalize.milestone":
            raise CoordinatorError(
                "legacy_milestone_finalize_forbidden",
                "Use 'milestone prepare', 'milestone validate', 'milestone review', "
                "and 'milestone commit'; legacy finalize --milestone cannot record "
                "workflow evidence or send WeCom notification.",
            )
        if name == "validation_copy.plan":
            self._require_artifact_governance_clean()
            record = self._require_workspace_copy().plan(
                str(arguments["session_id"]),
                include_paths=tuple(str(path) for path in arguments.get("paths") or ()),
            )
            return {"copy": record.to_dict()}
        if name == "validation_copy.materialize":
            self._require_artifact_governance_clean()
            record = self._require_workspace_copy().materialize_async(
                str(arguments["session_id"]),
                include_paths=tuple(str(path) for path in arguments.get("paths") or ()),
            )
            return {"copy": record.to_dict()}
        if name == "validation_copy.status":
            record = self._require_workspace_copy().status(
                str(arguments["session_id"]), str(arguments["job_id"])
            )
            return {"copy": record.to_dict()}
        if name == "validation_copy.cleanup":
            removed = self._require_workspace_copy().cleanup(
                str(arguments["session_id"]), str(arguments["job_root"])
            )
            return {"removed": str(removed)}
        if name == "validation_copy.run":
            self._require_artifact_governance_clean()
            evidence = self._require_workspace_copy().run(
                str(arguments["session_id"]),
                str(arguments["job_id"]),
                command=tuple(str(part) for part in arguments.get("command") or ()),
            )
            return {"evidence": evidence.to_dict()}
        if name == "legacy.report":
            return {"migration": self.legacy.report().to_dict()}
        if name == "legacy.import":
            if not bool(arguments.get("apply")):
                return {"migration": self.legacy.report().to_dict(), "applied": False}
            self._require_maintenance_capability(arguments)
            report = self.legacy.import_notes()
            failure_audit = self.failures.import_repository()
            inventory = self.plans.scan()
            return {
                "migration": report.to_dict(),
                "applied": True,
                "formal_plan_count": len(inventory.formal_plans),
                "legacy_plan_count": len(inventory.legacy_documents),
                "failure_node_count": failure_audit.node_count,
                "failure_diagnostic_count": len(failure_audit.diagnostics),
                "legacy_cargo_targets": list(self.legacy.legacy_cargo_diagnostics()),
            }
        if name == "legacy.archive":
            apply = bool(arguments.get("apply"))
            if apply:
                self._require_maintenance_capability(arguments)
            result = self.legacy.archive_notes(apply=apply)
            return {"archive": result.to_dict()}
        if name == "retention.plan":
            return {"plan": self.retention.plan().to_dict()}
        if name == "retention.show":
            return {
                "plan": self.retention.get_plan(str(arguments["plan_id"])).to_dict()
            }
        if name == "retention.apply":
            self._require_maintenance_capability(arguments)
            plan = self.retention.get_plan(str(arguments["plan_id"]))
            return {"plan": plan.to_dict(), "result": self.retention.apply(plan).to_dict()}
        if name == "maintenance.tick":
            if bool(arguments.get("apply_cleanup")) or bool(
                arguments.get("apply_retention")
            ) or bool(arguments.get("apply_legacy_archive")):
                self._require_maintenance_capability(arguments)
            if bool(arguments.get("apply_lifecycle")):
                self._require_maintenance_capability(arguments)
            return {"maintenance": self._maintenance_tick(arguments)}
        if name == "audit.all":
            return {"audit": self.rollout_audit.audit_all().to_dict()}
        raise CoordinatorError("unknown_command", f"Unknown coordinator command {name}")

    @staticmethod
    def _baseline_dict(baseline) -> dict[str, Any]:
        return {
            "epoch_id": baseline.epoch_id,
            "head_commit": baseline.head_commit,
            "index_tree": baseline.index_tree,
            "health": baseline.health.value,
            "manifest_count": len(baseline.manifest),
            "degraded_reason": baseline.degraded_reason,
        }

    @staticmethod
    def _patch_dict(patch) -> dict[str, Any]:
        result = asdict(patch)
        result["status"] = patch.status.value
        return result

    @staticmethod
    def _failure_audit_dict(audit) -> dict[str, Any]:
        return {
            "node_count": audit.node_count,
            "nodes": [asdict(item) for item in audit.nodes],
            "diagnostics": [asdict(item) for item in audit.diagnostics],
        }

    @staticmethod
    def _cleanup_plan_dict(plan) -> dict[str, Any]:
        return {
            "plan_id": plan.plan_id,
            "candidates": list(plan.candidates),
            "denied": [asdict(item) for item in plan.denied],
            "generated_at": plan.generated_at.isoformat(),
            "free_bytes_by_root": dict(plan.free_bytes_by_root),
            "pressure_roots": list(plan.pressure_roots),
            "older_than_hours": plan.older_than_hours,
        }

    def _require_cargo_jobs(self) -> CargoJobService:
        if self.cargo_jobs is None:
            raise CoordinatorError(
                "target_root_unavailable", "No D:/E:/F: managed targets root is available"
            )
        return self.cargo_jobs

    def _require_cargo_runner(self) -> CargoJobRunner:
        if self.cargo_runner is None:
            raise CoordinatorError(
                "target_root_unavailable", "No managed targets root is available for Cargo runs"
            )
        return self.cargo_runner

    def _record_startup_gpu_lane_audit(self) -> None:
        """Persist GPU leases that predate the latest resume reservation."""
        active = self._require_cargo_jobs().audit_active_gpu_jobs()
        if not active:
            return
        with self.database.transaction() as connection:
            resume = connection.execute(
                """SELECT completed_at FROM action_requests
                   WHERE action_kind='service.resume' AND status='succeeded'
                     AND completed_at IS NOT NULL
                   ORDER BY completed_at DESC, action_id DESC LIMIT 1"""
            ).fetchone()
            reservation_at = str(resume["completed_at"]) if resume is not None else None
            connection.execute(
                "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, ?)",
                (
                    "cargo.gpu_lane_startup_audit",
                    json.dumps(
                        {
                            "reservationCompletedAt": reservation_at,
                            "jobs": [
                                {
                                    "jobId": job.job_id,
                                    "sessionId": job.session_id,
                                    "status": job.status.value,
                                    "targetDir": job.target_dir,
                                    "createdAt": job.created_at.isoformat(),
                                    "preReservation": (
                                        reservation_at is not None
                                        and job.created_at.isoformat() < reservation_at
                                    ),
                                }
                                for job in active
                            ],
                        },
                        sort_keys=True,
                    ),
                    utc_text(),
                ),
            )

    def _require_cleanup(self) -> CleanupService:
        if self.cleanup is None:
            raise CoordinatorError(
                "target_root_unavailable", "No managed targets root is available for cleanup"
            )
        return self.cleanup

    def _require_workspace_copy(self) -> WorkspaceCopyService:
        if self.workspace_copy is None:
            raise CoordinatorError(
                "target_root_unavailable",
                "No managed targets root is available for validation copies",
            )
        return self.workspace_copy

    def _require_artifact_governance(self) -> ArtifactGovernanceService:
        if self.artifact_governance is None:
            raise CoordinatorError(
                "artifact_governance_unavailable",
                "Unmanaged artifact governance is disabled for this coordinator",
            )
        return self.artifact_governance

    def _artifact_governance_paths(self) -> list[str]:
        if self.artifact_governance is None:
            return []
        return [item.path for item in self.artifact_governance.scan()]

    def _require_artifact_governance_clean(self) -> None:
        if self.artifact_governance is not None:
            self.artifact_governance.require_clean()

    def _maintenance_tick(self, arguments: dict[str, Any]) -> dict[str, Any]:
        # Maintenance has its own non-blocking lock and service-level
        # reservations. It must never occupy the foreground command mutex while
        # doing filesystem or retention work.
        return self._maintenance_tick_serialized(arguments)

    def _maintenance_tick_serialized(self, arguments: dict[str, Any]) -> dict[str, Any]:
        if not self._maintenance_lock.acquire(blocking=False):
            raise CoordinatorError(
                "maintenance_busy", "Another coordinator maintenance tick is already running"
            )
        try:
            return self._maintenance_tick_unlocked(arguments)
        finally:
            self._maintenance_lock.release()

    def _maintenance_tick_unlocked(self, arguments: dict[str, Any]) -> dict[str, Any]:
        tick_id = uuid.uuid4().hex
        created_at = utc_text()
        stale: list[str] = []
        archived: list[str] = []
        orphaned: list[str] = []
        legacy_archive_run_id: str | None = None
        retention_plan_id: str | None = None
        cleanup_plan_id: str | None = None
        unmanaged_artifacts_deleted: list[str] = []
        try:
            legacy_report = self.legacy.report()
            legacy_active_sessions = {
                note.session_id for note in legacy_report.notes if note.activity_reasons
            }
            if bool(arguments.get("apply_lifecycle")):
                stale = self.sessions.mark_stale(
                    older_than_seconds=self.config.session_ttl_seconds,
                    excluded_session_ids=legacy_active_sessions,
                )
            if bool(arguments.get("apply_legacy_archive")):
                self.legacy.import_notes()
                legacy_archive = self.legacy.archive_notes(apply=True)
                legacy_archive_run_id = legacy_archive.run_id
            if bool(arguments.get("apply_lifecycle")):
                archived = self.sessions.archive_stale(
                    older_than_seconds=86400,
                    excluded_session_ids=legacy_active_sessions,
                )
            if self.cargo_jobs is not None:
                orphaned = [
                    job.job_id for job in self.cargo_jobs.reconcile_orphans()
                ]
            if self.workspace_copy is not None:
                self.workspace_copy.recover_interrupted_jobs(startup=False)
            retention_plan = self.retention.plan()
            retention_plan_id = retention_plan.plan_id
            if bool(arguments.get("apply_retention")) and (
                retention_plan.snapshot_ids or retention_plan.object_hashes
            ):
                self.retention.apply(retention_plan)
            if self.cleanup is not None:
                cleanup_plan = self.cleanup.plan()
                cleanup_plan_id = cleanup_plan.plan_id
                if bool(arguments.get("apply_cleanup")) and cleanup_plan.candidates:
                    self.cleanup.apply(cleanup_plan)
            if self.artifact_governance is not None:
                unmanaged_artifacts_deleted = list(self.artifact_governance.cleanup().deleted)
            with self.database.connect() as connection:
                connection.execute("PRAGMA wal_checkpoint(PASSIVE)")
            with self.database.transaction() as connection:
                connection.execute(
                    """
                    INSERT INTO maintenance_ticks(
                        tick_id, stale_sessions_json, archived_sessions_json,
                        orphaned_cargo_json, legacy_archive_run_id,
                        retention_plan_id, cleanup_plan_id, status, created_at
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, 'succeeded', ?)
                    """,
                    (
                        tick_id,
                        json.dumps(stale),
                        json.dumps(archived),
                        json.dumps(orphaned),
                        legacy_archive_run_id,
                        retention_plan_id,
                        cleanup_plan_id,
                        created_at,
                    ),
                )
        except BaseException as error:
            with self.database.transaction() as connection:
                connection.execute(
                    """
                    INSERT INTO maintenance_ticks(
                        tick_id, stale_sessions_json, archived_sessions_json,
                        orphaned_cargo_json, legacy_archive_run_id,
                        retention_plan_id, cleanup_plan_id, status, created_at, error_text
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, 'failed', ?, ?)
                    """,
                    (
                        tick_id,
                        json.dumps(stale),
                        json.dumps(archived),
                        json.dumps(orphaned),
                        legacy_archive_run_id,
                        retention_plan_id,
                        cleanup_plan_id,
                        created_at,
                        str(error),
                    ),
                )
            raise
        return {
            "tick_id": tick_id,
            "status": "succeeded",
            "stale_sessions": stale,
            "archived_sessions": archived,
            "orphaned_cargo_jobs": orphaned,
            "legacy_archive_run_id": legacy_archive_run_id,
            "retention_plan_id": retention_plan_id,
            "cleanup_plan_id": cleanup_plan_id,
            "unmanaged_artifacts_deleted": unmanaged_artifacts_deleted,
        }

    @staticmethod
    def _authorize_maintenance(arguments: dict[str, Any]) -> bool:
        if not bool(arguments.get("maintenance")):
            return False
        CoordinatorApplication._require_maintenance_capability(arguments)
        return True

    @staticmethod
    def _require_maintenance_capability(arguments: dict[str, Any]) -> None:
        configured = os.environ.get("ZIRCON_COORDINATOR_MAINTENANCE_TOKEN")
        # Local coordinator commands already authenticate through the runtime
        # descriptor.  A separate maintenance token is therefore optional for
        # a local-only deployment, but remains an opt-in extra guard when set.
        if not configured:
            return
        supplied = str(arguments.get("maintenance_capability") or "")
        if not secrets.compare_digest(configured, supplied):
            raise CoordinatorError(
                "maintenance_unauthorized",
                "Destructive maintenance requires the configured local maintenance capability",
            )

    def _branch(self) -> str:
        result = subprocess.run(
            ["git", "branch", "--show-current"],
            cwd=self.config.repo_root,
            check=True,
            capture_output=True,
            text=True,
        )
        return result.stdout.strip()


class _CoordinatorHttpServer(ThreadingHTTPServer):
    # The production coordinator owns one fixed loopback endpoint.  Leaving
    # SO_REUSEADDR enabled on Windows permits unrelated repositories to share
    # 6518 and makes clients land on an arbitrary ledger.
    allow_reuse_address = False
    daemon_threads = True

    def __init__(self, address, handler, *, application: CoordinatorApplication):
        super().__init__(address, handler)
        self.application = application
        router = ControlPlaneRouter(
            instance_id=application.instance_id,
            auth=application.web_auth,
            snapshot=application.control_snapshot,
            workflows=application.workflow_projections,
            database=application.database,
            actions=application.control_actions,
            maintenance_authorizer=application._require_maintenance_capability,
            live_workflow_eligibility=application.milestone_workflows.live_eligibility,
            codex_wake=application.codex_worker.wake,
            repository_key=application.repository_identity.key,
        )
        self.control_http = ControlPlaneHttp(
            router,
            application.control_events,
            assets=StaticAssetService(application.config.control_web_dist_root),
            artifact_downloads=ArtifactDownloadService(
                application.database, application.config.workflow_artifact_root
            ),
        )


class CoordinatorRequestHandler(BaseHTTPRequestHandler):
    server: _CoordinatorHttpServer

    def do_GET(self) -> None:
        if self.path == "/":
            self.send_response(HTTPStatus.SEE_OTHER)
            self.send_header("Location", "/ui/")
            self.send_header("Cache-Control", "no-store")
            self.end_headers()
            return
        if self.server.control_http.handles(self.path):
            self.server.control_http.handle(self)
            return
        if not self._authorized():
            return
        if self.path == "/health":
            self._write_json(HTTPStatus.OK, self.server.application.health())
            return
        self._write_error(HTTPStatus.NOT_FOUND, "not_found", "Unknown endpoint")

    def do_POST(self) -> None:
        if self.server.control_http.handles(self.path):
            self.server.control_http.handle(self)
            return
        if not self._authorized():
            return
        if self.path == "/shutdown":
            self._write_error(
                HTTPStatus.GONE,
                "controlled_lifecycle_required",
                "Use the controlled service.stop action",
            )
            return
        if self.path != "/command":
            self._write_error(HTTPStatus.NOT_FOUND, "not_found", "Unknown endpoint")
            return
        try:
            length = int(self.headers.get("Content-Length", "0"))
            payload = json.loads(self.rfile.read(length).decode("utf-8"))
            command = str(payload["command"])
            arguments = payload.get("arguments") or {}
            if not isinstance(arguments, dict):
                raise ValueError("arguments must be an object")
            result = self.server.application.command(command, arguments)
            self._write_json(HTTPStatus.OK, result)
        except (BrokenPipeError, ConnectionAbortedError, ConnectionResetError):
            # The command has already completed or is independently durable.
            # A caller timing out must never turn a response write into another
            # mutation or a noisy handler traceback.
            return
        except CoordinatorError as error:
            self._write_json(HTTPStatus.CONFLICT, {"error": error.to_dict()})
        except (KeyError, TypeError, ValueError, json.JSONDecodeError) as error:
            self._write_error(HTTPStatus.BAD_REQUEST, "invalid_request", str(error))
        except Exception as error:  # pragma: no cover - defensive service boundary
            self._write_error(HTTPStatus.INTERNAL_SERVER_ERROR, "internal_error", str(error))

    def do_PUT(self) -> None:
        self._delegate_control_method()

    def do_PATCH(self) -> None:
        self._delegate_control_method()

    def do_DELETE(self) -> None:
        self._delegate_control_method()

    def do_HEAD(self) -> None:
        self._delegate_control_method()

    def do_OPTIONS(self) -> None:
        self._delegate_control_method()

    def _delegate_control_method(self) -> None:
        if self.server.control_http.handles(self.path):
            self.server.control_http.handle(self)
            return
        self._write_error(HTTPStatus.NOT_FOUND, "not_found", "Unknown endpoint")

    def log_message(self, _format: str, *_args: object) -> None:
        return

    def _authorized(self) -> bool:
        # The service only binds to the exact IPv4 loopback address. Local users
        # intentionally access the coordinator without a browser or CLI token.
        return True

    def _write_error(self, status: HTTPStatus, code: str, message: str) -> None:
        self._write_json(status, {"error": {"code": code, "message": message, "details": {}}})

    def _write_json(self, status: HTTPStatus, payload: dict[str, Any]) -> None:
        encoded = json.dumps(payload, sort_keys=True).encode("utf-8")
        try:
            self.send_response(status)
            self.send_header("Content-Type", "application/json; charset=utf-8")
            self.send_header("Content-Length", str(len(encoded)))
            self.end_headers()
            self.wfile.write(encoded)
        except (BrokenPipeError, ConnectionAbortedError, ConnectionResetError):
            # HTTP clients can leave after a bounded command wait.  The server
            # operation is already independently transactional; response I/O
            # must not turn that condition into a handler traceback or retry.
            return


@dataclass(slots=True)
class RunningCoordinator:
    config: CoordinatorConfig
    httpd: _CoordinatorHttpServer
    thread: threading.Thread
    maintenance_thread: threading.Thread
    maintenance_stop: threading.Event
    token: str
    instance_id: str
    started_at: str

    @classmethod
    def start(
        cls, config: CoordinatorConfig, *, automatic_start: bool = False
    ) -> "RunningCoordinator":
        if config.host != "127.0.0.1":
            raise CoordinatorError(
                "invalid_bind_host",
                "Session coordinator must bind to the 127.0.0.1 loopback address",
            )
        config.state_root.mkdir(parents=True, exist_ok=True)
        cls._acquire_lock(config)
        instance_id = uuid.uuid4().hex
        started_at = utc_text()
        try:
            application = CoordinatorApplication(
                config,
                instance_id=instance_id,
                started_at=started_at,
                automatic_start=automatic_start,
            )
            if not application.read_only:
                remove_commit_guard(config.repo_root)
            httpd = _CoordinatorHttpServer(
                (config.host, config.port),
                CoordinatorRequestHandler,
                application=application,
            )
            application.lifecycle.set_shutdown(lambda _kind: httpd.shutdown())
            thread = threading.Thread(target=httpd.serve_forever, name="zircon-session-coordinator", daemon=True)
            thread.start()
            maintenance_stop = threading.Event()
            maintenance_thread = threading.Thread(
                target=cls._maintenance_loop,
                args=(
                    application,
                    config.watch_interval_seconds,
                    config.maintenance_interval_seconds,
                    maintenance_stop,
                ),
                name="zircon-session-coordinator-watch",
                daemon=True,
            )
            maintenance_thread.start()
            host, port = httpd.server_address[:2]
            descriptor = RuntimeDescriptor(
                host=str(host),
                port=int(port),
                token="",
                repo_root=config.repo_root,
                repository=application.repository_identity,
                instance_id=instance_id,
                started_at=started_at,
                process=application.process_identity,
            )
            _atomic_json_write(config.runtime_path, descriptor.to_payload())
            if application.read_only:
                application.supervision.transition(
                    SupervisionState.READ_ONLY,
                    reason_code="startup.read_only_branch",
                    actor="daemon",
                )
            elif application.supervision.snapshot().maintenance_hold:
                # A controlled reload must not publish a healthy mutation window
                # before its maintainer explicitly releases the durable hold.
                application.supervision.transition(
                    SupervisionState.DRAINING,
                    reason_code="startup.maintenance_hold",
                    actor="daemon",
                )
            else:
                application.supervision.mark_healthy()
            application.lifecycle.recover_restart_intents()
            application.codex_worker.start()
            (config.state_root / "startup-failure.json").unlink(missing_ok=True)
            return cls(
                config=config,
                httpd=httpd,
                thread=thread,
                maintenance_thread=maintenance_thread,
                maintenance_stop=maintenance_stop,
                token="",
                instance_id=instance_id,
                started_at=started_at,
            )
        except BaseException as error:
            if isinstance(error, sqlite3.DatabaseError):
                _atomic_json_write(
                    config.state_root / "startup-failure.json",
                    {
                        "kind": "migration_or_integrity_failure",
                        "errorType": type(error).__name__,
                        "occurredAt": utc_text(),
                    },
                )
            cls._remove_owned_file(config.lock_path, os.getpid())
            raise

    @property
    def base_url(self) -> str:
        host, port = self.httpd.server_address[:2]
        return f"http://{host}:{port}"

    def stop(self) -> None:
        self.maintenance_stop.set()
        self.httpd.application.codex_worker.stop()
        self.httpd.control_http.close()
        self.httpd.shutdown()
        self.httpd.server_close()
        self.thread.join(timeout=5)
        self.maintenance_thread.join(timeout=5)
        self._remove_owned_file(self.config.runtime_path, os.getpid())
        self._remove_owned_file(self.config.lock_path, os.getpid())

    def __enter__(self) -> "RunningCoordinator":
        return self

    def __exit__(self, _exc_type, _exc_value, _traceback) -> None:
        self.stop()

    @staticmethod
    def _acquire_lock(config: CoordinatorConfig) -> None:
        if config.lock_path.exists():
            try:
                existing = json.loads(config.lock_path.read_text(encoding="utf-8"))
                if _pid_is_alive(int(existing.get("pid", 0))):
                    raise CoordinatorError("already_running", "Coordinator is already running")
            except (OSError, ValueError, TypeError, json.JSONDecodeError):
                pass
            config.lock_path.unlink(missing_ok=True)
        descriptor = json.dumps({"pid": os.getpid()})
        descriptor_fd = os.open(config.lock_path, os.O_CREAT | os.O_EXCL | os.O_WRONLY)
        with os.fdopen(descriptor_fd, "w", encoding="utf-8") as stream:
            stream.write(descriptor)

    @staticmethod
    def _remove_owned_file(path: Path, pid: int) -> None:
        try:
            payload = json.loads(path.read_text(encoding="utf-8"))
            if int(payload.get("pid", -1)) == pid:
                path.unlink(missing_ok=True)
        except (OSError, ValueError, TypeError, json.JSONDecodeError):
            return

    @staticmethod
    def _maintenance_loop(
        application: CoordinatorApplication,
        watch_interval_seconds: float,
        maintenance_interval_seconds: float,
        stop_event: threading.Event,
    ) -> None:
        watch_interval = max(watch_interval_seconds, 0.05)
        maintenance_interval = max(maintenance_interval_seconds, watch_interval)
        next_maintenance = time.monotonic() + maintenance_interval
        while not stop_event.wait(watch_interval):
            try:
                observation = application.watcher.prepare_scan()
                application.watcher.apply_scan(observation)
            except Exception as error:  # pragma: no cover - defensive long-lived boundary
                with application.database.transaction() as connection:
                    connection.execute(
                        "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, datetime('now'))",
                        (
                            "watch.scan_failed",
                            json.dumps({"error": str(error)}, sort_keys=True),
                        ),
                    )
            if application.cargo_jobs is not None:
                try:
                    orphaned = application.cargo_jobs.reconcile_orphans()
                    if orphaned:
                        with application.database.transaction() as connection:
                            connection.execute(
                                "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, datetime('now'))",
                                (
                                    "cargo.jobs_orphaned",
                                    json.dumps(
                                        {"job_ids": [job.job_id for job in orphaned]},
                                        sort_keys=True,
                                    ),
                                ),
                            )
                    application.cleanup.retry_pending_jobs()
                    application.cleanup.evict_idle_pools_under_pressure()
                except Exception as error:  # pragma: no cover - defensive long-lived boundary
                    with application.database.transaction() as connection:
                        connection.execute(
                            "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, datetime('now'))",
                            (
                                "cargo.reconcile_failed",
                                json.dumps({"error": str(error)}, sort_keys=True),
                            ),
                        )
            if application.artifact_governance is not None:
                try:
                    application.artifact_governance.cleanup()
                except Exception as error:  # pragma: no cover - defensive maintenance boundary
                    with application.database.transaction() as connection:
                        connection.execute(
                            "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, datetime('now'))",
                            (
                                "artifact.governance_failed",
                                json.dumps({"error": str(error)}, sort_keys=True),
                            ),
                        )
            if application.workspace_copy is not None:
                try:
                    recovered_running, recovered_cleanup = (
                        application.workspace_copy.recover_interrupted_jobs(startup=False)
                    )
                    if recovered_running or recovered_cleanup:
                        with application.database.transaction() as connection:
                            connection.execute(
                                "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, datetime('now'))",
                                (
                                    "validation_copy.recovered",
                                    json.dumps(
                                        {
                                            "running": recovered_running,
                                            "cleanup_pending": recovered_cleanup,
                                        },
                                        sort_keys=True,
                                    ),
                                ),
                            )
                except Exception as error:  # pragma: no cover - defensive long-lived boundary
                    with application.database.transaction() as connection:
                        connection.execute(
                            "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, datetime('now'))",
                            (
                                "validation_copy.recovery_failed",
                                json.dumps({"error": str(error)}, sort_keys=True),
                            ),
                        )
            if not application.read_only and time.monotonic() >= next_maintenance:
                try:
                    application._maintenance_tick(
                        {
                            "apply_cleanup": True,
                            "apply_retention": True,
                            "apply_legacy_archive": True,
                            "apply_lifecycle": True,
                        }
                    )
                except Exception as error:  # pragma: no cover - defensive long-lived boundary
                    with application.database.transaction() as connection:
                        connection.execute(
                            "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, datetime('now'))",
                            (
                                "maintenance.tick_failed",
                                json.dumps({"error": str(error)}, sort_keys=True),
                            ),
                        )
                finally:
                    next_maintenance = time.monotonic() + maintenance_interval


def run_forever(config: CoordinatorConfig, *, automatic_start: bool = False) -> None:
    running = RunningCoordinator.start(config, automatic_start=automatic_start)
    try:
        running.thread.join()
    except KeyboardInterrupt:
        pass
    finally:
        running.stop()
