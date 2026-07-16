from __future__ import annotations

import json
from dataclasses import dataclass
from enum import StrEnum
from typing import Any, Iterable

from .database import Database
from .models import CoordinatorError, utc_text


AI_WORKDAY_HOURS = 8.0
DEFAULT_BASELINE_ID = "user-2026-07-15"
MAX_PLAN_ID_LENGTH = 512
MAX_BLOCKER_LENGTH = 512
MAX_LEDGER_ID_LENGTH = 128


class AiEffortOutcome(StrEnum):
    ACCEPTED = "accepted"
    FAILED = "failed"
    SUPERSEDED = "superseded"


class AiEffortCostClass(StrEnum):
    DELIVERY_DESIGN = "delivery_design"
    REPAIR_VALIDATION = "repair_validation"


@dataclass(frozen=True, slots=True)
class AiEffortMilestone:
    ledger_id: str
    plan_id: str
    active_ai_hours: float
    outcome: AiEffortOutcome
    blocked_by: tuple[str, ...]
    cost_class: AiEffortCostClass
    source_session_id: str | None = None


class AiEffortService:
    """A privacy-safe ledger for planned AI effort and accepted delivery value."""

    def __init__(self, database: Database) -> None:
        self.database = database

    def seed_user_baseline(self) -> dict[str, object]:
        """Persist the user-confirmed aggregate numbers without inferring milestone time."""
        baseline = {
            "historical": {
                "allEffectiveHours": 6005.0,
                "allEffectiveDays": 750.6,
                "deliveryDesignHours": 5371.0,
                "deliveryDesignDays": 671.3,
                "repairValidationHours": 634.0,
                "repairValidationDays": 79.3,
                "julyTraceableHours": 911.7,
                "julyTraceableDays": 114.0,
                "extrapolationAllowed": False,
            },
            "currentPlan": {
                "scopeCompletePercent": 47.0,
                "acceptedEffectiveDays": {"min": 110.0, "max": 175.0},
                "remainingEffectiveDays": {"min": 125.0, "max": 200.0},
                "totalEffectiveDays": {"min": 235.0, "max": 375.0},
            },
        }
        scenarios = (
            ("single_ai", 1.0, 1.0, 25.0, 40.0),
            ("two_isolated_plus_integration", 1.6, 2.0, 18.0, 28.0),
            ("three_domains_plus_integration", 2.2, 2.8, 12.0, 20.0),
        )
        now = utc_text()
        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO ai_effort_baselines(baseline_id, payload_json, recorded_at)
                   VALUES (?, ?, ?)
                   ON CONFLICT(baseline_id) DO NOTHING""",
                (DEFAULT_BASELINE_ID, json.dumps(baseline, sort_keys=True), now),
            )
            connection.executemany(
                """INSERT INTO ai_effort_forecast_scenarios(
                       scenario_id, effective_parallelism_min, effective_parallelism_max,
                       calendar_weeks_min, calendar_weeks_max, recorded_at
                   ) VALUES (?, ?, ?, ?, ?, ?)
                   ON CONFLICT(scenario_id) DO NOTHING""",
                ((*scenario, now) for scenario in scenarios),
            )
        return self.report()

    def record(self, value: AiEffortMilestone) -> dict[str, object]:
        self._validate(value)
        with self.database.transaction() as connection:
            existing = connection.execute(
                "SELECT * FROM ai_effort_milestones WHERE ledger_id=?", (value.ledger_id,)
            ).fetchone()
            encoded_blockers = json.dumps(value.blocked_by, ensure_ascii=False)
            if existing is not None:
                expected = (
                    value.plan_id,
                    value.active_ai_hours,
                    value.outcome.value,
                    encoded_blockers,
                    value.cost_class.value,
                    value.source_session_id,
                )
                actual = (
                    existing["plan_id"],
                    float(existing["active_ai_hours"]),
                    existing["outcome"],
                    existing["blocked_by_json"],
                    existing["cost_class"],
                    existing["source_session_id"],
                )
                if actual != expected:
                    raise CoordinatorError(
                        "ai_effort_ledger_id_conflict",
                        "AI effort ledger ID already exists with different values",
                        details={"ledgerId": value.ledger_id},
                    )
                return self._row_to_dict(existing)
            connection.execute(
                """INSERT INTO ai_effort_milestones(
                       ledger_id, plan_id, active_ai_hours, outcome, blocked_by_json,
                       cost_class, source_session_id, recorded_at
                   ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)""",
                (
                    value.ledger_id,
                    value.plan_id,
                    value.active_ai_hours,
                    value.outcome.value,
                    encoded_blockers,
                    value.cost_class.value,
                    value.source_session_id,
                    utc_text(),
                ),
            )
            row = connection.execute(
                "SELECT * FROM ai_effort_milestones WHERE ledger_id=?", (value.ledger_id,)
            ).fetchone()
        return self._row_to_dict(row)

    def report(self) -> dict[str, object]:
        with self.database.connect() as connection:
            baseline_row = connection.execute(
                "SELECT payload_json, recorded_at FROM ai_effort_baselines "
                "WHERE baseline_id=?",
                (DEFAULT_BASELINE_ID,),
            ).fetchone()
            milestone_rows = connection.execute(
                "SELECT * FROM ai_effort_milestones ORDER BY recorded_at, ledger_id"
            ).fetchall()
            scenarios = connection.execute(
                """SELECT scenario_id, effective_parallelism_min, effective_parallelism_max,
                          calendar_weeks_min, calendar_weeks_max
                   FROM ai_effort_forecast_scenarios ORDER BY scenario_id"""
            ).fetchall()
        baseline = json.loads(baseline_row["payload_json"]) if baseline_row else None
        by_outcome = {outcome.value: 0.0 for outcome in AiEffortOutcome}
        by_cost_class = {cost.value: 0.0 for cost in AiEffortCostClass}
        blocked: dict[str, dict[str, float | int]] = {}
        for row in milestone_rows:
            hours = float(row["active_ai_hours"])
            by_outcome[row["outcome"]] += hours
            by_cost_class[row["cost_class"]] += hours
            for blocker in json.loads(row["blocked_by_json"]):
                aggregate = blocked.setdefault(blocker, {"hours": 0.0, "milestones": 0})
                aggregate["hours"] = float(aggregate["hours"]) + hours
                aggregate["milestones"] = int(aggregate["milestones"]) + 1
        accepted_hours = by_outcome[AiEffortOutcome.ACCEPTED.value]
        failed_hours = by_outcome[AiEffortOutcome.FAILED.value]
        return {
            "baselineId": DEFAULT_BASELINE_ID if baseline is not None else None,
            "baselineRecordedAt": baseline_row["recorded_at"] if baseline_row else None,
            "historical": baseline["historical"] if baseline else None,
            "currentPlanBudget": baseline["currentPlan"] if baseline else None,
            "ledger": {
                "milestoneCount": len(milestone_rows),
                "acceptedEffectiveHours": accepted_hours,
                "acceptedEffectiveDays": self._days(accepted_hours),
                "failedQualityCostHours": failed_hours,
                "failedQualityCostDays": self._days(failed_hours),
                "supersededHours": by_outcome[AiEffortOutcome.SUPERSEDED.value],
                "hoursByOutcome": by_outcome,
                "hoursByCostClass": by_cost_class,
                "blockedBy": [
                    {"blockedBy": key, **value}
                    for key, value in sorted(blocked.items(), key=lambda item: item[0].casefold())
                ],
            },
            "calendarForecasts": [
                {
                    "scenarioId": row["scenario_id"],
                    "effectiveParallelism": {
                        "min": float(row["effective_parallelism_min"]),
                        "max": float(row["effective_parallelism_max"]),
                    },
                    "calendarWeeks": {
                        "min": float(row["calendar_weeks_min"]),
                        "max": float(row["calendar_weeks_max"]),
                    },
                }
                for row in scenarios
            ],
        }

    @staticmethod
    def _days(hours: float) -> float:
        return round(hours / AI_WORKDAY_HOURS, 3)

    @staticmethod
    def _row_to_dict(row) -> dict[str, object]:
        return {
            "ledgerId": row["ledger_id"],
            "planId": row["plan_id"],
            "activeAiHours": float(row["active_ai_hours"]),
            "outcome": row["outcome"],
            "blockedBy": json.loads(row["blocked_by_json"]),
            "costClass": row["cost_class"],
            "sourceSessionId": row["source_session_id"],
            "recordedAt": row["recorded_at"],
        }

    @staticmethod
    def _validate(value: AiEffortMilestone) -> None:
        if not value.ledger_id.strip() or len(value.ledger_id) > MAX_LEDGER_ID_LENGTH:
            raise CoordinatorError("ai_effort_ledger_id_invalid", "AI effort ledger ID is invalid")
        if not value.plan_id.strip() or len(value.plan_id) > MAX_PLAN_ID_LENGTH:
            raise CoordinatorError("ai_effort_plan_id_invalid", "AI effort plan ID is invalid")
        if value.active_ai_hours < 0:
            raise CoordinatorError("ai_effort_hours_invalid", "AI effort hours cannot be negative")
        if not isinstance(value.outcome, AiEffortOutcome):
            raise CoordinatorError("ai_effort_outcome_invalid", "AI effort outcome is invalid")
        if not isinstance(value.cost_class, AiEffortCostClass):
            raise CoordinatorError("ai_effort_cost_class_invalid", "AI effort cost class is invalid")
        if any(not item.strip() or len(item) > MAX_BLOCKER_LENGTH for item in value.blocked_by):
            raise CoordinatorError("ai_effort_blocked_by_invalid", "AI effort blocker is invalid")


def normalize_blocked_by(value: object) -> tuple[str, ...]:
    if value is None:
        return ()
    if not isinstance(value, list) or any(not isinstance(item, str) for item in value):
        raise CoordinatorError("ai_effort_blocked_by_invalid", "blockedBy must be a list of strings")
    return tuple(item.strip() for item in value)
