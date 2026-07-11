from __future__ import annotations

import hashlib
import json
import subprocess
from pathlib import Path
from sqlite3 import Connection

from ...baselines import hash_file
from ...database import Database
from ...failures import failure_artifact_snapshot
from ...models import CoordinatorError
from .models import ActionFingerprint, ActionKind, ActionParameters, ActionSpec


class ActionFingerprinter:
    """Captures every authority surface that can invalidate an action preview."""

    def __init__(
        self,
        database: Database,
        repo_root: str | Path,
        *,
        daemon_instance_id: str,
        supervision=None,
    ):
        self.database = database
        self.repo_root = Path(repo_root).resolve()
        self.daemon_instance_id = daemon_instance_id
        self.supervision = supervision

    def capture(
        self,
        spec: ActionSpec,
        parameters: ActionParameters,
        *,
        bound_session_id: str | None,
        connection: Connection | None = None,
    ) -> ActionFingerprint:
        if connection is None:
            with self.database.connect() as owned_connection:
                return self._capture(
                    owned_connection, spec, parameters, bound_session_id=bound_session_id
                )
        return self._capture(
            connection, spec, parameters, bound_session_id=bound_session_id
        )

    def _capture(
        self,
        connection: Connection,
        spec: ActionSpec,
        parameters: ActionParameters,
        *,
        bound_session_id: str | None,
    ) -> ActionFingerprint:
        session_id = getattr(parameters, "session_id", None) or bound_session_id
        session = (
            connection.execute(
                "SELECT * FROM sessions WHERE session_id = ?", (session_id,)
            ).fetchone()
            if session_id
            else None
        )
        if session_id and session is None:
            raise CoordinatorError("session_not_found", f"Unknown Session {session_id}")

        baseline = connection.execute(
            "SELECT epoch_id, head_commit, health FROM baseline_epochs ORDER BY epoch_id DESC LIMIT 1"
        ).fetchone()
        resources = self._action_resources(connection, spec, parameters, session_id)
        targets = self._target_paths(connection, session_id, session)
        plan_session = session
        executor_session_id = getattr(parameters, "executor_session_id", None)
        if executor_session_id:
            plan_session = connection.execute(
                "SELECT * FROM sessions WHERE session_id=?", (executor_session_id,)
            ).fetchone()
        plan_path = (
            str(plan_session["plan_path"])
            if plan_session is not None and plan_session["plan_path"]
            else None
        )
        payload: dict[str, object] = {
            "actionKind": spec.kind.value,
            "parameters": parameters.to_payload(),
            "daemonInstanceId": self.daemon_instance_id,
            "head": self._git("rev-parse", "HEAD"),
            "index": self._index_digest(),
            "baseline": dict(baseline) if baseline is not None else None,
            "session": dict(session) if session is not None else None,
            "actionResources": resources,
            "plan": {
                "path": plan_path,
                "hash": self._safe_file_hash(plan_path) if plan_path else None,
            },
            "targets": [
                {"path": path, "hash": self._safe_file_hash(path)} for path in targets
            ],
        }
        canonical = json.dumps(
            payload, sort_keys=True, separators=(",", ":"), ensure_ascii=False
        ).encode("utf-8")
        return ActionFingerprint(hashlib.sha256(canonical).hexdigest(), payload)

    def impact(
        self, spec: ActionSpec, parameters: ActionParameters, *, bound_session_id: str | None
    ) -> tuple[str, ...]:
        session_id = getattr(parameters, "session_id", None) or bound_session_id
        with self.database.connect() as connection:
            session = (
                connection.execute(
                    "SELECT * FROM sessions WHERE session_id = ?", (session_id,)
                ).fetchone()
                if session_id
                else None
            )
            targets = self._target_paths(connection, session_id, session)
            resources = self._action_resources(connection, spec, parameters, session_id)
        impact = [f"Session: {session_id}" if session_id else "Service control state"]
        impact.extend(f"Target: {path}" for path in targets[:50])
        if len(targets) > 50:
            impact.append(f"Additional targets: {len(targets) - 50}")
        impact.extend(self._resource_impact(resources))
        return tuple(impact)

    def _action_resources(self, connection, spec, parameters, session_id: str | None) -> dict[str, object]:
        patches: list[dict[str, object]] = []
        validation_copies: list[dict[str, object]] = []
        failure_nodes: list[dict[str, object]] = []
        failure_artifacts: tuple[dict[str, str], ...] = ()
        leases: list[dict[str, object]] = []
        workflow: dict[str, object] | None = None
        supervision: dict[str, object] | None = None
        if spec.kind in {
            ActionKind.DRAIN_PREVIEW,
            ActionKind.SERVICE_DRAIN,
            ActionKind.SERVICE_RESUME,
            ActionKind.SERVICE_STOP,
            ActionKind.SERVICE_RESTART,
            ActionKind.SERVICE_FORCE_STOP,
        } and self.supervision is not None:
            supervision = self.supervision.snapshot(connection).to_dict()
        if spec.kind is ActionKind.PATCH_PROCESS and session_id:
            patches = [dict(row) for row in connection.execute(
                """SELECT patch_id, session_id, patch_object_hash, targets_json,
                          base_hashes_json, status, updated_at
                   FROM patches WHERE session_id = ? AND status = 'queued' ORDER BY patch_id""",
                (session_id,),
            )]
            target_paths = {
                str(path).casefold()
                for patch in patches
                for path in json.loads(str(patch["targets_json"]))
            }
            leases = [dict(row) for row in connection.execute(
                "SELECT display_path, session_id, expires_at FROM leases ORDER BY path_key"
            ) if str(row["display_path"]).casefold() in target_paths]
        if spec.kind is ActionKind.LEASE_CLAIM and session_id:
            row = connection.execute(
                "SELECT write_scope_json FROM sessions WHERE session_id = ?", (session_id,)
            ).fetchone()
            scope = {str(path).casefold() for path in json.loads(row[0] or "[]")} if row else set()
            leases = [dict(item) for item in connection.execute(
                "SELECT display_path, session_id, expires_at FROM leases ORDER BY path_key"
            ) if str(item["display_path"]).casefold() in scope]
        if spec.kind is ActionKind.LEASE_RELEASE and session_id:
            leases = [dict(row) for row in connection.execute(
                """SELECT display_path, session_id, expires_at FROM leases
                   WHERE session_id = ? ORDER BY path_key""", (session_id,)
            )]
        if spec.kind is ActionKind.VALIDATION_CANCEL:
            job_id = getattr(parameters, "job_id", "")
            validation_copies = [dict(row) for row in connection.execute(
                """SELECT job_id, session_id, head_commit, manifest_json, status,
                          run_pid, created_at, removed_at
                   FROM validation_copies WHERE job_id = ? AND session_id = ?""",
                (job_id, session_id),
            )]
        if spec.kind is ActionKind.FAILURE_REFRESH:
            failure_nodes = [dict(row) for row in connection.execute(
                """SELECT lifecycle_key, artifact_path, kind, fixing_plan, origin_plan,
                          status, imported_at
                   FROM failure_nodes
                   WHERE origin_plan = (SELECT plan_path FROM sessions WHERE session_id = ?)
                      OR fixing_plan = (SELECT plan_path FROM sessions WHERE session_id = ?)
                   ORDER BY lifecycle_key""",
                (session_id, session_id),
            )]
            failure_artifacts = failure_artifact_snapshot(self.repo_root)
        if spec.kind in {
            ActionKind.VALIDATION_START,
            ActionKind.TOPOLOGY_REFRESH,
            ActionKind.MILESTONE_COMMIT,
            ActionKind.SESSION_COMPLETE,
        } and getattr(parameters, "run_id", None):
            run_id = getattr(parameters, "run_id", "")
            workflow_session_id = (
                getattr(parameters, "executor_session_id", None) or session_id
            )
            run = connection.execute(
                """SELECT run_id, session_id, topology_hash,
                          current_topology_version_id, state, updated_at
                   FROM workflow_runs WHERE run_id=? AND session_id=?""",
                (run_id, workflow_session_id),
            ).fetchone()
            if run is None:
                raise CoordinatorError(
                    "workflow_run_session_mismatch",
                    "Workflow run does not belong to the requested Session",
                )
            workflow = {
                "run": dict(run),
                "nodes": [
                    dict(row)
                    for row in connection.execute(
                        """SELECT node_id, node_key, kind, state, attempt_count, updated_at
                           FROM workflow_nodes WHERE run_id=? ORDER BY node_key""",
                        (run_id,),
                    )
                ],
                "gates": [
                    dict(row)
                    for row in connection.execute(
                        """SELECT evidence_id, gate_kind, decision,
                                  input_fingerprint, created_at
                           FROM workflow_gate_evidence WHERE run_id=?
                           ORDER BY created_at, evidence_id""",
                        (run_id,),
                    )
                ],
            }
        return {
            "leases": leases,
            "patches": patches,
            "validationCopies": validation_copies,
            "failureNodes": failure_nodes,
            "failureArtifacts": failure_artifacts,
            "workflow": workflow,
            "supervision": supervision,
        }

    @staticmethod
    def _resource_impact(resources: dict[str, object]) -> tuple[str, ...]:
        lines: list[str] = []
        for row in resources["leases"]:
            lines.append(f"Lease: {row['display_path']} [{row['session_id']}]")
        for row in resources["patches"]:
            lines.append(f"Patch: {row['patch_id']} [{row['status']}]")
        for row in resources["validationCopies"]:
            lines.append(f"Validation: {row['job_id']} [{row['status']}] pid={row['run_pid'] or '-'}")
        for row in resources["failureNodes"]:
            lines.append(f"Failure: {row['lifecycle_key']} [{row['status']}]")
        for row in resources["failureArtifacts"]:
            lines.append(f"Failure artifact: {row['path']} sha256={row['hash'][:12]}")
        supervision = resources.get("supervision")
        if isinstance(supervision, dict):
            lines.append(f"Supervision: {supervision.get('state', 'unknown')}")
            for blocker in supervision.get("blockers", []):
                if isinstance(blocker, dict):
                    lines.append(
                        f"Blocker: {blocker.get('kind')} {blocker.get('identity')} "
                        f"[{blocker.get('status')}]"
                    )
        return tuple(lines)

    def _target_paths(self, connection, session_id: str | None, session) -> tuple[str, ...]:
        if not session_id or session is None:
            return ()
        scope = json.loads(session["write_scope_json"] or "[]")
        leases = [
            row[0]
            for row in connection.execute(
                "SELECT display_path FROM leases WHERE session_id = ?", (session_id,)
            ).fetchall()
        ]
        attributions = [
            row[0]
            for row in connection.execute(
                "SELECT display_path FROM attributions WHERE session_id = ?",
                (session_id,),
            ).fetchall()
        ]
        return tuple(sorted({str(path) for path in (*scope, *leases, *attributions)}, key=str.casefold))

    def _safe_file_hash(self, value: str) -> str | None:
        candidate = (self.repo_root / value).resolve()
        if not candidate.is_relative_to(self.repo_root):
            raise CoordinatorError("action_target_outside_repo", "Derived action target escaped repository")
        if not candidate.is_file():
            return None
        return hash_file(candidate)

    def _index_digest(self) -> str:
        result = subprocess.run(
            ["git", "diff", "--cached", "--binary", "--no-ext-diff"],
            cwd=self.repo_root,
            check=True,
            capture_output=True,
        )
        return hashlib.sha256(result.stdout).hexdigest()

    def _git(self, *arguments: str) -> str:
        return subprocess.run(
            ["git", *arguments],
            cwd=self.repo_root,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()
