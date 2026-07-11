from __future__ import annotations

import subprocess
from dataclasses import dataclass
from pathlib import Path

from .baselines import BaselineService
from .cargo_jobs import TargetPathPolicy
from .database import Database
from .failures import FailureGraphService
from .legacy import LegacyMigrationService
from .models import CoordinatorError, SessionStatus
from .plans import PlanRepository
from .sessions import SessionService


@dataclass(frozen=True, slots=True)
class RolloutAudit:
    branch: str
    baseline_health: str
    session_count: int
    invalid_session_statuses: tuple[str, ...]
    formal_plan_count: int
    legacy_plan_count: int
    handoff_artifact_count: int
    failure_diagnostics: tuple[str, ...]
    target_roots: tuple[str, ...]
    unsafe_cleanup_candidates: tuple[str, ...]
    legacy_note_count: int
    legacy_archive_eligible_count: int
    legacy_cargo_targets: tuple[str, ...]
    maintenance_tick_count: int

    def to_dict(self) -> dict[str, object]:
        return {
            "branch": self.branch,
            "baseline_health": self.baseline_health,
            "session_count": self.session_count,
            "invalid_session_statuses": list(self.invalid_session_statuses),
            "formal_plan_count": self.formal_plan_count,
            "legacy_plan_count": self.legacy_plan_count,
            "handoff_artifact_count": self.handoff_artifact_count,
            "failure_diagnostics": list(self.failure_diagnostics),
            "target_roots": list(self.target_roots),
            "unsafe_cleanup_candidates": list(self.unsafe_cleanup_candidates),
            "legacy_note_count": self.legacy_note_count,
            "legacy_archive_eligible_count": self.legacy_archive_eligible_count,
            "legacy_cargo_targets": list(self.legacy_cargo_targets),
            "maintenance_tick_count": self.maintenance_tick_count,
        }


class RolloutAuditService:
    """Build a deterministic, read-only rollout report from current state."""

    def __init__(
        self,
        database: Database,
        repo_root: str | Path,
        *,
        sessions: SessionService,
        baselines: BaselineService,
        plans: PlanRepository,
        failures: FailureGraphService,
        legacy: LegacyMigrationService,
        target_roots: tuple[str | Path, ...],
    ):
        self.database = database
        self.repo_root = Path(repo_root).resolve()
        self.sessions = sessions
        self.baselines = baselines
        self.plans = plans
        self.failures = failures
        self.legacy = legacy
        self.target_roots = tuple(Path(root).resolve() for root in target_roots)
        self.target_policy = (
            TargetPathPolicy(self.target_roots) if self.target_roots else None
        )

    def audit_all(self) -> RolloutAudit:
        inventory = self.plans.scan()
        legacy_report = self.legacy.report()
        valid_statuses = {status.value for status in SessionStatus}
        with self.database.connect() as connection:
            status_rows = connection.execute(
                "SELECT DISTINCT status FROM sessions ORDER BY status"
            ).fetchall()
            session_count = int(connection.execute("SELECT COUNT(*) FROM sessions").fetchone()[0])
            maintenance_tick_count = int(
                connection.execute(
                    "SELECT COUNT(*) FROM maintenance_ticks WHERE status = 'succeeded'"
                ).fetchone()[0]
            )
            cargo_rows = connection.execute(
                "SELECT DISTINCT target_dir FROM cargo_jobs ORDER BY target_dir"
            ).fetchall()
        invalid_statuses = tuple(
            row["status"] for row in status_rows if row["status"] not in valid_statuses
        )
        unsafe_targets = tuple(
            row["target_dir"]
            for row in cargo_rows
            if not self._target_is_managed(Path(row["target_dir"]))
        )
        handoff_artifacts = sorted(
            [*self.repo_root.glob("docs/plans/**/failure-*.md"), *self.repo_root.glob("docs/plans/**/fixed-*.md")],
            key=lambda path: path.as_posix().casefold(),
        )
        try:
            failure_diagnostics = tuple(sorted(self.failures.validator_errors()))
        except (FileNotFoundError, ImportError):
            failure_diagnostics = ("failure_validator_unavailable",)
        try:
            baseline_health = self.baselines.current().health.value
        except CoordinatorError:
            baseline_health = "uninitialized"
        return RolloutAudit(
            branch=self._git("branch", "--show-current"),
            baseline_health=baseline_health,
            session_count=session_count,
            invalid_session_statuses=invalid_statuses,
            formal_plan_count=len(inventory.formal_plans),
            legacy_plan_count=len(inventory.legacy_documents),
            handoff_artifact_count=len(handoff_artifacts),
            failure_diagnostics=failure_diagnostics,
            target_roots=tuple(str(root) for root in self.target_roots),
            unsafe_cleanup_candidates=unsafe_targets,
            legacy_note_count=len(legacy_report.notes),
            legacy_archive_eligible_count=sum(
                note.archive_eligible for note in legacy_report.notes
            ),
            legacy_cargo_targets=self.legacy.legacy_cargo_diagnostics(),
            maintenance_tick_count=maintenance_tick_count,
        )

    def _target_is_managed(self, value: Path) -> bool:
        if self.target_policy is None:
            return False
        try:
            self.target_policy.validate(value)
        except Exception:
            return False
        return True

    def _git(self, *arguments: str) -> str:
        return subprocess.run(
            ["git", *arguments],
            cwd=self.repo_root,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()
