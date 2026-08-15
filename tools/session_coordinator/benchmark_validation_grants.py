from __future__ import annotations

from dataclasses import dataclass
import json
import os
import sqlite3
import uuid
from pathlib import Path
from typing import Callable, Mapping

from .database import Database
from .models import CoordinatorError, utc_text


_PROFILES = frozenset({"release", "profiling"})
BENCHMARK_SOURCE_MANIFEST_ENV = "ZR_BENCHMARK_SOURCE_MANIFEST"
BENCHMARK_CARGO_PROFILE_ENV = "ZR_BENCHMARK_CARGO_PROFILE"
BENCHMARK_ENVIRONMENT_KEYS = frozenset(
    {BENCHMARK_SOURCE_MANIFEST_ENV, BENCHMARK_CARGO_PROFILE_ENV}
)


def _sha256(value: object, *, code: str, label: str) -> str:
    if not isinstance(value, str) or len(value) != 64 or any(
        character not in "0123456789abcdef" for character in value
    ):
        raise CoordinatorError(code, f"{label} must be an exact lowercase SHA-256")
    return value


def benchmark_child_environment(
    target_root: Path, *, benchmark_environment: Mapping[str, str] | None = None
) -> dict[str, str]:
    target_root = target_root.resolve()
    cargo_home = target_root / "cargo-home"
    sccache = target_root / "sccache"
    temporary = target_root / "temporary"
    for directory in (target_root, cargo_home, sccache, temporary):
        directory.mkdir(parents=True, exist_ok=True)

    environment = os.environ.copy()
    for key in BENCHMARK_ENVIRONMENT_KEYS:
        environment.pop(key, None)
    if benchmark_environment is not None:
        environment.update(benchmark_environment)
    environment.update(
        {
            "CARGO_TARGET_DIR": str(target_root),
            "CARGO_HOME": str(cargo_home),
            "SCCACHE_DIR": str(sccache),
            "TEMP": str(temporary),
            "TMP": str(temporary),
            "TMPDIR": str(temporary),
        }
    )
    return environment


def benchmark_run_environment(
    environment: Mapping[str, str] | None, *, input_manifest_hash: object
) -> dict[str, str]:
    if environment is None:
        return {}
    if set(environment) != BENCHMARK_ENVIRONMENT_KEYS or any(
        not isinstance(value, str) for value in environment.values()
    ):
        raise CoordinatorError(
            "validation_copy_benchmark_environment_invalid",
            "Benchmark child environment must contain exactly the source manifest and profile",
        )
    source_manifest = environment[BENCHMARK_SOURCE_MANIFEST_ENV]
    cargo_profile = environment[BENCHMARK_CARGO_PROFILE_ENV]
    _sha256(
        source_manifest,
        code="validation_copy_benchmark_environment_invalid",
        label="Benchmark source manifest",
    )
    if cargo_profile not in _PROFILES:
        raise CoordinatorError(
            "validation_copy_benchmark_environment_invalid",
            "Benchmark child identity requires an optimized Cargo profile",
        )
    if input_manifest_hash is None:
        raise CoordinatorError(
            "validation_copy_benchmark_manifest_missing",
            "Materialized benchmark copy has no input manifest hash",
        )
    if source_manifest != input_manifest_hash:
        raise CoordinatorError(
            "validation_copy_benchmark_manifest_mismatch",
            "Benchmark child manifest does not match the materialized validation copy",
        )
    return dict(environment)


def require_benchmark_launch_grant(
    connection,
    *,
    grant_id: str,
    session_id: str,
    job_id: str,
    copy_row,
    command: tuple[str, ...],
    environment: Mapping[str, str],
    validation_run_id: str,
    required_copy_status: str,
) -> None:
    if not environment:
        raise CoordinatorError(
            "validation_copy_benchmark_environment_invalid",
            "Benchmark grant launch requires its exact child identity",
        )
    grant = connection.execute(
        "SELECT * FROM benchmark_validation_grants WHERE grant_id=?", (grant_id,)
    ).fetchone()
    binding = connection.execute(
        """SELECT binding.*, exact.template AS exact_template,
                  node.node_key AS milestone_key
           FROM workflow_validation_bindings binding
           JOIN workflow_validation_template_bindings exact
             ON exact.validation_run_id=binding.validation_run_id
           JOIN workflow_nodes node
             ON node.run_id=binding.run_id AND node.node_id=binding.node_id
           WHERE binding.validation_run_id=?
             AND binding.benchmark_grant_id=?""",
        (validation_run_id, grant_id),
    ).fetchone()
    source = connection.execute(
        "SELECT plan_path FROM sessions WHERE session_id=?",
        (grant["source_session_id"],) if grant is not None else ("",),
    ).fetchone()
    target = connection.execute(
        "SELECT plan_path FROM sessions WHERE session_id=?",
        (grant["target_session_id"],) if grant is not None else ("",),
    ).fetchone()
    workflow_run = connection.execute(
        "SELECT session_id FROM workflow_runs WHERE run_id=?",
        (grant["run_id"],) if grant is not None else ("",),
    ).fetchone()
    try:
        granted_command = (
            tuple(json.loads(grant["command_json"])) if grant is not None else ()
        )
    except (TypeError, ValueError, json.JSONDecodeError):
        granted_command = ()
    if (
        grant is None
        or grant["status"] != "launching"
        or grant["validation_run_id"] is not None
        or grant["root_pid"] is not None
        or grant["root_process_creation_time"] is not None
        or int(grant["job_isolated"]) != 0
        or copy_row is None
        or grant["target_session_id"] != session_id
        or grant["source_session_id"] != copy_row["session_id"]
        or grant["job_id"] != job_id
        or grant["input_manifest_hash"] != copy_row["input_manifest_hash"]
        or copy_row["status"] != required_copy_status
        or granted_command != command
        or environment[BENCHMARK_SOURCE_MANIFEST_ENV] != grant["input_manifest_hash"]
        or environment[BENCHMARK_CARGO_PROFILE_ENV] != grant["cargo_profile"]
        or source is None
        or target is None
        or not source["plan_path"]
        or source["plan_path"] != target["plan_path"]
        or workflow_run is None
        or workflow_run["session_id"] != session_id
    ):
        raise CoordinatorError(
            "validation_copy_benchmark_grant_invalid",
            "Benchmark launch does not match its one-shot Coordinator grant",
        )
    if (
        binding is None
        or binding["job_id"] != job_id
        or binding["run_id"] != grant["run_id"]
        or binding["milestone_key"] != grant["milestone_id"]
        or binding["session_id"] != session_id
        or binding["exact_template"] != "native-plugin-benchmark"
        or binding["source_manifest_hash"] != grant["scoped_manifest_hash"]
        or binding["copy_input_manifest_hash"] != grant["input_manifest_hash"]
        or binding["benchmark_name"] != grant["benchmark_name"]
        or binding["cargo_profile"] != grant["cargo_profile"]
        or binding["root_pid"] is not None
        or binding["root_process_creation_time"] is not None
    ):
        raise CoordinatorError(
            "validation_copy_benchmark_binding_invalid",
            "Benchmark launch does not match its pre-created workflow binding",
        )


@dataclass(frozen=True, slots=True)
class BenchmarkValidationCandidate:
    job_id: str
    source_session_id: str
    target_session_id: str
    input_manifest_hash: str


@dataclass(frozen=True, slots=True)
class BenchmarkValidationGrant:
    grant_id: str
    job_id: str
    source_session_id: str
    target_session_id: str
    run_id: str
    milestone_id: str
    input_manifest_hash: str
    scoped_manifest_hash: str
    benchmark_name: str
    cargo_profile: str
    command: tuple[str, ...]
    fifo_sequence: int
    status: str
    validation_run_id: str | None
    root_pid: int | None
    root_process_creation_time: str | None
    job_isolated: bool
    error_code: str | None

    def to_dict(self, *, include_job: bool = False) -> dict[str, object]:
        result: dict[str, object] = {
            "grantId": self.grant_id,
            "sourceSessionId": self.source_session_id,
            "targetSessionId": self.target_session_id,
            "runId": self.run_id,
            "milestoneId": self.milestone_id,
            "inputManifestHash": self.input_manifest_hash,
            "scopedManifestHash": self.scoped_manifest_hash,
            "benchmarkName": self.benchmark_name,
            "cargoProfile": self.cargo_profile,
            "command": list(self.command),
            "fifoSequence": self.fifo_sequence,
            "status": self.status,
            "validationRunId": self.validation_run_id,
            "rootPid": self.root_pid,
            "rootProcessCreationTime": self.root_process_creation_time,
            "jobIsolated": self.job_isolated,
            "errorCode": self.error_code,
        }
        if include_job:
            result["jobId"] = self.job_id
        return result


class BenchmarkValidationGrantService:
    """Authorize one existing materialized copy for one named benchmark launch."""

    def __init__(self, database: Database):
        self.database = database
        self.reconcile_interrupted_launches()

    def reconcile_interrupted_launches(self) -> tuple[str, ...]:
        with self.database.transaction() as connection:
            rows = connection.execute(
                """SELECT grant_id FROM benchmark_validation_grants
                   WHERE status='launching' AND validation_run_id IS NULL
                     AND root_pid IS NULL
                   ORDER BY fifo_sequence"""
            ).fetchall()
            grant_ids = tuple(str(row["grant_id"]) for row in rows)
            if grant_ids:
                connection.execute(
                    """UPDATE benchmark_validation_grants
                       SET status='denied', denied_at=?,
                           error_code='benchmark_validation_grant_launch_interrupted'
                       WHERE status='launching' AND validation_run_id IS NULL
                         AND root_pid IS NULL""",
                    (utc_text(),),
                )
        return grant_ids

    def reconcile_interrupted_consumed(
        self,
        reject_validation: Callable[..., bool],
        *,
        terminate_interrupted: Callable[..., None],
    ) -> tuple[str, ...]:
        with self.database.connect() as connection:
            rows = connection.execute(
                """SELECT grant_id, job_id, validation_run_id, root_pid,
                          root_process_creation_time, job_isolated
                   FROM benchmark_validation_grants grant
                   WHERE grant.status='consumed'
                     AND grant.validation_run_id IS NOT NULL
                     AND NOT EXISTS (
                         SELECT 1 FROM validation_copy_runs run
                         WHERE run.run_id=grant.validation_run_id
                     )
                   ORDER BY grant.fifo_sequence"""
            ).fetchall()
        recovered: list[str] = []
        error_code = "benchmark_validation_collector_interrupted"
        for row in rows:
            validation_run_id = str(row["validation_run_id"])
            if row["root_pid"] is None or not row["root_process_creation_time"]:
                raise CoordinatorError(
                    "benchmark_validation_recovery_identity_missing",
                    "Interrupted benchmark has no durable process identity",
                )
            terminate_interrupted(
                grant_id=str(row["grant_id"]),
                job_id=str(row["job_id"]),
                root_pid=int(row["root_pid"]),
                process_creation_time=str(row["root_process_creation_time"]),
                job_isolated=bool(row["job_isolated"]),
            )
            reject_validation(validation_run_id, error_code=error_code)
            with self.database.transaction() as connection:
                cursor = connection.execute(
                    """UPDATE benchmark_validation_grants
                       SET error_code=?
                       WHERE grant_id=? AND status='consumed'
                         AND validation_run_id=?
                         AND NOT EXISTS (
                             SELECT 1 FROM validation_copy_runs run
                             WHERE run.run_id=benchmark_validation_grants.validation_run_id
                         )""",
                    (error_code, row["grant_id"], validation_run_id),
                )
            if cursor.rowcount == 1:
                recovered.append(validation_run_id)
        return tuple(recovered)

    def select_candidate(
        self, *, source_session_id: str, target_session_id: str
    ) -> BenchmarkValidationCandidate:
        with self.database.connect() as connection:
            source = connection.execute(
                "SELECT plan_path FROM sessions WHERE session_id=?", (source_session_id,)
            ).fetchone()
            target = connection.execute(
                "SELECT plan_path FROM sessions WHERE session_id=?", (target_session_id,)
            ).fetchone()
            self._require_same_plan(source, target)
            rows = connection.execute(
                """SELECT job_id, input_manifest_hash
                   FROM validation_copies
                   WHERE session_id=? AND status='materialized'
                     AND materialization_kind='cargo'
                     AND materialization_phase='materialized'
                   ORDER BY created_at, job_id""",
                (source_session_id,),
            ).fetchall()
        if not rows:
            raise CoordinatorError(
                "benchmark_validation_grant_copy_unavailable",
                "Source Session has no materialized validation copy",
            )
        if len(rows) != 1:
            raise CoordinatorError(
                "benchmark_validation_grant_copy_ambiguous",
                "Source Session must have exactly one materialized validation copy",
                details={"candidateCount": len(rows)},
            )
        manifest_hash = _sha256(
            rows[0]["input_manifest_hash"],
            code="benchmark_validation_grant_manifest_invalid",
            label="Materialized copy input manifest",
        )
        return BenchmarkValidationCandidate(
            str(rows[0]["job_id"]),
            source_session_id,
            target_session_id,
            manifest_hash,
        )

    def issue(
        self,
        *,
        candidate: BenchmarkValidationCandidate,
        target_session_id: str,
        run_id: str,
        milestone_id: str,
        benchmark_name: str,
        cargo_profile: str,
        command: tuple[str, ...] | list[str],
        scoped_manifest_hash: str,
    ) -> BenchmarkValidationGrant:
        if candidate.target_session_id != target_session_id:
            raise CoordinatorError(
                "benchmark_validation_grant_target_mismatch",
                "Selected copy is bound to another target Session",
            )
        full_hash = _sha256(
            candidate.input_manifest_hash,
            code="benchmark_validation_grant_manifest_invalid",
            label="Materialized copy input manifest",
        )
        scoped_hash = _sha256(
            scoped_manifest_hash,
            code="benchmark_validation_grant_scoped_manifest_invalid",
            label="Milestone-scoped manifest",
        )
        profile = self._profile(cargo_profile)
        name = self._text(benchmark_name, "benchmark name")
        milestone = self._text(milestone_id, "milestone ID")
        command_tuple = self._command(command)
        grant_id = uuid.uuid4().hex
        now = utc_text()
        try:
            with self.database.transaction() as connection:
                source = connection.execute(
                    "SELECT plan_path FROM sessions WHERE session_id=?",
                    (candidate.source_session_id,),
                ).fetchone()
                target = connection.execute(
                    "SELECT plan_path FROM sessions WHERE session_id=?", (target_session_id,)
                ).fetchone()
                self._require_same_plan(source, target)
                run = connection.execute(
                    "SELECT session_id FROM workflow_runs WHERE run_id=?", (run_id,)
                ).fetchone()
                if run is None or run["session_id"] != target_session_id:
                    raise CoordinatorError(
                        "benchmark_validation_grant_workflow_mismatch",
                        "Workflow run does not belong to the target Session",
                    )
                copy = connection.execute(
                    """SELECT session_id, status, input_manifest_hash
                       FROM validation_copies WHERE job_id=?""",
                    (candidate.job_id,),
                ).fetchone()
                if (
                    copy is None
                    or copy["session_id"] != candidate.source_session_id
                    or copy["status"] != "materialized"
                    or copy["input_manifest_hash"] != full_hash
                ):
                    raise CoordinatorError(
                        "benchmark_validation_grant_copy_changed",
                        "Selected validation copy changed before grant issuance",
                    )
                connection.execute(
                    """INSERT INTO benchmark_validation_grants(
                           grant_id, job_id, source_session_id, target_session_id,
                           run_id, milestone_id, input_manifest_hash,
                           scoped_manifest_hash, benchmark_name, cargo_profile,
                           command_json, status, issued_at
                       ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'issued', ?)""",
                    (
                        grant_id,
                        candidate.job_id,
                        candidate.source_session_id,
                        target_session_id,
                        run_id,
                        milestone,
                        full_hash,
                        scoped_hash,
                        name,
                        profile,
                        json.dumps(command_tuple),
                        now,
                    ),
                )
                row = connection.execute(
                    "SELECT * FROM benchmark_validation_grants WHERE grant_id=?",
                    (grant_id,),
                ).fetchone()
        except sqlite3.IntegrityError as error:
            raise CoordinatorError(
                "benchmark_validation_grant_copy_already_bound",
                "Materialized validation copy already has a benchmark grant",
            ) from error
        return self._from_row(row)

    def acquire(
        self,
        *,
        target_session_id: str,
        run_id: str,
        milestone_id: str,
        benchmark_name: str,
        cargo_profile: str,
        command: tuple[str, ...] | list[str],
    ) -> BenchmarkValidationGrant:
        profile = self._profile(cargo_profile)
        name = self._text(benchmark_name, "benchmark name")
        milestone = self._text(milestone_id, "milestone ID")
        command_tuple = self._command(command)
        rejected: CoordinatorError | None = None
        with self.database.transaction() as connection:
            row = connection.execute(
                """SELECT * FROM benchmark_validation_grants
                   WHERE target_session_id=? AND status IN ('issued', 'launching')
                   ORDER BY fifo_sequence LIMIT 1""",
                (target_session_id,),
            ).fetchone()
            if row is None:
                raise CoordinatorError(
                    "benchmark_validation_grant_required",
                    "No Coordinator-issued benchmark validation grant is available",
                )
            if row["status"] != "issued":
                raise CoordinatorError(
                    "benchmark_validation_grant_fifo_wait",
                    "An earlier benchmark validation grant is already launching",
                )
            expected = (
                str(row["run_id"]),
                str(row["milestone_id"]),
                str(row["benchmark_name"]),
                str(row["cargo_profile"]),
                tuple(json.loads(row["command_json"])),
            )
            actual = (run_id, milestone, name, profile, command_tuple)
            if actual != expected:
                raise CoordinatorError(
                    "benchmark_validation_grant_fifo_wait",
                    "Requested benchmark does not match the target Session FIFO head",
                )
            try:
                self._revalidate_in_connection(connection, row)
            except CoordinatorError as error:
                rejected = error
                connection.execute(
                    """UPDATE benchmark_validation_grants
                       SET status='denied', denied_at=?, error_code=?
                       WHERE grant_id=? AND status='issued'""",
                    (utc_text(), error.code, row["grant_id"]),
                )
            else:
                cursor = connection.execute(
                    """UPDATE benchmark_validation_grants
                       SET status='launching', acquired_at=?
                       WHERE grant_id=? AND status='issued'""",
                    (utc_text(), row["grant_id"]),
                )
                if cursor.rowcount != 1:
                    raise CoordinatorError(
                        "benchmark_validation_grant_replayed",
                        "Benchmark validation grant was already acquired",
                    )
            row = connection.execute(
                "SELECT * FROM benchmark_validation_grants WHERE grant_id=?",
                (row["grant_id"],),
            ).fetchone()
        if rejected is not None:
            raise rejected
        return self._from_row(row)

    def deny(self, grant_id: str, *, error_code: str) -> BenchmarkValidationGrant:
        with self.database.transaction() as connection:
            cursor = connection.execute(
                """UPDATE benchmark_validation_grants
                   SET status='denied', denied_at=?, error_code=?
                   WHERE grant_id=? AND status IN ('issued', 'launching')""",
                (utc_text(), self._text(error_code, "error code"), grant_id),
            )
            if cursor.rowcount != 1:
                raise CoordinatorError(
                    "benchmark_validation_grant_terminal",
                    "Benchmark validation grant is already terminal",
                )
            row = connection.execute(
                "SELECT * FROM benchmark_validation_grants WHERE grant_id=?", (grant_id,)
            ).fetchone()
        return self._from_row(row)

    @staticmethod
    def _revalidate_in_connection(connection, row) -> None:
        source = connection.execute(
            "SELECT plan_path FROM sessions WHERE session_id=?",
            (row["source_session_id"],),
        ).fetchone()
        target = connection.execute(
            "SELECT plan_path FROM sessions WHERE session_id=?",
            (row["target_session_id"],),
        ).fetchone()
        BenchmarkValidationGrantService._require_same_plan(source, target)
        run = connection.execute(
            "SELECT session_id FROM workflow_runs WHERE run_id=?", (row["run_id"],)
        ).fetchone()
        copy = connection.execute(
            """SELECT session_id, status, input_manifest_hash
               FROM validation_copies WHERE job_id=?""",
            (row["job_id"],),
        ).fetchone()
        if run is None or run["session_id"] != row["target_session_id"]:
            raise CoordinatorError(
                "benchmark_validation_grant_workflow_mismatch",
                "Workflow run no longer belongs to the grant target Session",
            )
        if (
            copy is None
            or copy["session_id"] != row["source_session_id"]
            or copy["status"] != "materialized"
            or copy["input_manifest_hash"] != row["input_manifest_hash"]
        ):
            raise CoordinatorError(
                "benchmark_validation_grant_copy_changed",
                "Granted validation copy changed before launch",
            )

    @staticmethod
    def _require_same_plan(source, target) -> None:
        if (
            source is None
            or target is None
            or not source["plan_path"]
            or source["plan_path"] != target["plan_path"]
        ):
            raise CoordinatorError(
                "benchmark_validation_grant_plan_mismatch",
                "Source and target Sessions must own the same numbered Plan",
            )

    @staticmethod
    def _profile(value: object) -> str:
        if not isinstance(value, str) or value not in _PROFILES:
            raise CoordinatorError(
                "benchmark_validation_grant_profile_invalid",
                "Benchmark Cargo profile must be release or profiling",
            )
        return value

    @staticmethod
    def _text(value: object, label: str) -> str:
        if not isinstance(value, str) or not value.strip() or len(value.strip()) > 300:
            raise CoordinatorError(
                "benchmark_validation_grant_parameters_invalid", f"Invalid {label}"
            )
        return value.strip()

    @staticmethod
    def _command(value: tuple[str, ...] | list[str]) -> tuple[str, ...]:
        if not isinstance(value, (tuple, list)):
            raise CoordinatorError(
                "benchmark_validation_grant_command_invalid",
                "Benchmark command must be a server-generated argument sequence",
            )
        command = tuple(part for part in value if isinstance(part, str) and part)
        if not command or len(command) != len(value):
            raise CoordinatorError(
                "benchmark_validation_grant_command_invalid",
                "Benchmark command must contain only non-empty string arguments",
            )
        return command

    @staticmethod
    def _from_row(row) -> BenchmarkValidationGrant:
        if row is None:
            raise CoordinatorError(
                "benchmark_validation_grant_not_found", "Benchmark grant was not found"
            )
        return BenchmarkValidationGrant(
            grant_id=str(row["grant_id"]),
            job_id=str(row["job_id"]),
            source_session_id=str(row["source_session_id"]),
            target_session_id=str(row["target_session_id"]),
            run_id=str(row["run_id"]),
            milestone_id=str(row["milestone_id"]),
            input_manifest_hash=str(row["input_manifest_hash"]),
            scoped_manifest_hash=str(row["scoped_manifest_hash"]),
            benchmark_name=str(row["benchmark_name"]),
            cargo_profile=str(row["cargo_profile"]),
            command=tuple(json.loads(row["command_json"])),
            fifo_sequence=int(row["fifo_sequence"]),
            status=str(row["status"]),
            validation_run_id=row["validation_run_id"],
            root_pid=int(row["root_pid"]) if row["root_pid"] is not None else None,
            root_process_creation_time=row["root_process_creation_time"],
            job_isolated=bool(row["job_isolated"]),
            error_code=row["error_code"],
        )
