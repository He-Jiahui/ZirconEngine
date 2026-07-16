from __future__ import annotations

import hashlib
import json
import re
import subprocess
from pathlib import Path
from sqlite3 import Connection

from ...baselines import hash_file
from ...database import Database
from ...failures import failure_artifact_snapshot
from ...models import CoordinatorError
from .models import ActionFingerprint, ActionKind, ActionParameters, ActionSpec, SessionParameters


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
        self._git_dir = Path(self._git("rev-parse", "--absolute-git-dir")).resolve()
        common_dir_file = self._git_dir / "commondir"
        if common_dir_file.is_file():
            common_value = common_dir_file.read_text(encoding="utf-8").strip()
            self._git_common_dir = (self._git_dir / common_value).resolve()
        else:
            self._git_common_dir = self._git_dir
        self._index_path = self._git_dir / "index"

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
        session_id = self._session_id(spec, parameters, bound_session_id)
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
            "head": self._head_oid(),
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
        if spec.kind is ActionKind.CODEX_RECONCILE:
            latest_codex_run = connection.execute(
                "SELECT run_id, status, source_revision, completed_at "
                "FROM codex_sync_runs ORDER BY created_at DESC, run_id DESC LIMIT 1"
            ).fetchone()
            payload["codexSync"] = {
                "latestRun": dict(latest_codex_run) if latest_codex_run is not None else None,
                "sessionCount": connection.execute(
                    "SELECT COUNT(*) FROM codex_sessions"
                ).fetchone()[0],
            }
        canonical = json.dumps(
            payload, sort_keys=True, separators=(",", ":"), ensure_ascii=False
        ).encode("utf-8")
        return ActionFingerprint(hashlib.sha256(canonical).hexdigest(), payload)

    def impact(
        self, spec: ActionSpec, parameters: ActionParameters, *, bound_session_id: str | None
    ) -> tuple[str, ...]:
        session_id = self._session_id(spec, parameters, bound_session_id)
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

    @staticmethod
    def _session_id(
        spec: ActionSpec, parameters: ActionParameters, bound_session_id: str | None
    ) -> str | None:
        if (
            spec.kind is ActionKind.SESSION_ACTIVATE
            and isinstance(parameters, SessionParameters)
            and parameters.maintenance_session_id is not None
        ):
            return parameters.maintenance_session_id
        return getattr(parameters, "session_id", None) or bound_session_id

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
        if spec.kind is ActionKind.MILESTONE_RECONCILE:
            source_run_id = getattr(parameters, "source_run_id", "")
            target_run_id = getattr(parameters, "target_run_id", "")
            rows = [
                connection.execute(
                    """SELECT run.run_id, run.session_id, run.plan_path, run.topology_hash,
                              run.state, run.updated_at, run.current_topology_version_id,
                              version.content_hash, version.topology_json
                       FROM workflow_runs run
                       LEFT JOIN workflow_topology_versions version
                         ON version.topology_version_id=run.current_topology_version_id
                       WHERE run.run_id=?""",
                    (run_id,),
                ).fetchone()
                for run_id in (source_run_id, target_run_id)
            ]
            if any(row is None for row in rows):
                raise CoordinatorError(
                    "workflow_reconcile_run_not_found",
                    "Both reconciliation workflow runs must exist",
                )
            reconciliation_runs: list[dict[str, object]] = []
            plan_paths: set[str] = set()
            for row in rows:
                assert row is not None
                run = dict(row)
                plan_path = str(run["plan_path"])
                plan_paths.add(plan_path)
                run_id = str(run["run_id"])
                reconciliation_runs.append(
                    {
                        "run": run,
                        "nodes": [
                            dict(item)
                            for item in connection.execute(
                                """SELECT node_id, node_key, kind, title, stage, state,
                                          attempt_count, updated_at
                                   FROM workflow_nodes WHERE run_id=? ORDER BY node_key""",
                                (run_id,),
                            )
                        ],
                        "attempts": [
                            dict(item)
                            for item in connection.execute(
                                """SELECT attempt_id, node_id, attempt_number, state, accepted,
                                          evidence_json, completed_at
                                   FROM workflow_attempts WHERE run_id=? ORDER BY node_id, attempt_number""",
                                (run_id,),
                            )
                        ],
                        "manifests": [
                            dict(item)
                            for item in connection.execute(
                                """SELECT manifest_id, topology_version_id, node_id, paths_json,
                                          manifest_hash, created_at
                                   FROM workflow_milestone_manifests
                                   WHERE run_id=? ORDER BY node_id, created_at""",
                                (run_id,),
                            )
                        ],
                        "intents": [
                            dict(item)
                            for item in connection.execute(
                                """SELECT intent_id, topology_version_id, node_id, gate_fingerprint,
                                          paths_json, status, commit_sha, updated_at
                                   FROM workflow_commit_intents
                                   WHERE run_id=? ORDER BY node_id, created_at""",
                                (run_id,),
                            )
                        ],
                    }
                )
            plan_values = tuple(sorted(plan_paths))
            placeholders = ",".join("?" for _ in plan_values)
            failure_nodes = [
                dict(row)
                for row in connection.execute(
                    f"""SELECT lifecycle_key, artifact_path, kind, status, origin_plan,
                               fixing_plan, imported_at
                        FROM failure_nodes
                        WHERE origin_plan IN ({placeholders}) OR fixing_plan IN ({placeholders})
                        ORDER BY lifecycle_key""",
                    (*plan_values, *plan_values),
                )
            ]
            workflow = {"reconciliationRuns": reconciliation_runs}
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
        if not self._index_path.is_file():
            return hashlib.sha256(b"").hexdigest()
        return hash_file(self._index_path)

    def _head_oid(self) -> str:
        try:
            value = (self._git_dir / "HEAD").read_text(encoding="ascii").strip()
            for _ in range(5):
                if re.fullmatch(r"[0-9a-fA-F]{40}|[0-9a-fA-F]{64}", value):
                    return value.lower()
                if not value.startswith("ref: "):
                    break
                reference = value.removeprefix("ref: ").strip()
                loose = self._read_loose_ref(reference)
                if loose is not None:
                    value = loose
                    continue
                packed = self._read_packed_ref(reference)
                if packed is not None:
                    return packed
                break
        except (OSError, UnicodeError):
            pass
        return self._git("rev-parse", "HEAD")

    def _read_loose_ref(self, reference: str) -> str | None:
        if not reference.startswith("refs/") or ".." in Path(reference).parts:
            return None
        for root in (self._git_dir, self._git_common_dir):
            candidate = root / reference
            try:
                if candidate.is_file():
                    return candidate.read_text(encoding="ascii").strip()
            except (OSError, UnicodeError):
                return None
        return None

    def _read_packed_ref(self, reference: str) -> str | None:
        try:
            lines = (self._git_common_dir / "packed-refs").read_text(
                encoding="ascii"
            ).splitlines()
        except (OSError, UnicodeError):
            return None
        suffix = f" {reference}"
        for line in lines:
            if line.startswith(("#", "^")) or not line.endswith(suffix):
                continue
            oid = line.split(" ", 1)[0]
            if re.fullmatch(r"[0-9a-fA-F]{40}|[0-9a-fA-F]{64}", oid):
                return oid.lower()
        return None

    def _git(self, *arguments: str) -> str:
        return subprocess.run(
            ["git", *arguments],
            cwd=self.repo_root,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()
