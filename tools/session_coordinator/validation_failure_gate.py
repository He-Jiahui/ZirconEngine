"""Failure-graph admission for queued validation tickets."""

from __future__ import annotations

import json
from collections import deque
from collections.abc import Mapping
from sqlite3 import Connection, Row
from typing import Any


_BLOCKED_EVENT = "validation.ticket_dependency_blocked"


def blockers(connection: Connection, ticket: object) -> tuple[dict[str, object], ...]:
    """Return open cross-plan Failures that affect ``ticket``.

    The caller supplies its existing transaction. This function intentionally reads
    only the imported ``failure_nodes`` projection; admission must not rescan or parse
    Failure artifacts while the validation queue transaction is held.
    """

    plan_path = _portable_path(_ticket_value(ticket, "plan_path", ""))
    if not plan_path:
        return ()

    rows = connection.execute(
        """
        SELECT lifecycle_key, artifact_path, created_at, summary_slug,
               origin_plan, origin_workflow_node, fixing_plan, priority,
               related_code_json
        FROM failure_nodes
        WHERE kind='failure' AND status='open' AND origin_plan <> fixing_plan
        ORDER BY priority, created_at, summary_slug, artifact_path
        """
    ).fetchall()
    by_origin: dict[str, list[Row]] = {}
    for row in rows:
        origin_key = _plan_key(row["origin_plan"])
        if origin_key != _plan_key(row["fixing_plan"]):
            by_origin.setdefault(origin_key, []).append(row)

    workflow_nodes = _workflow_nodes(ticket)
    ticket_paths = _ticket_paths(ticket)
    full_coverage = _full_coverage(ticket)
    cargo_ticket = _cargo_ticket(ticket)
    pending = deque([(plan_path, (plan_path,), True)])
    expanded = {_plan_key(plan_path)}
    emitted: set[str] = set()
    result: list[dict[str, object]] = []

    while pending:
        origin, dependency_path, scope_ticket = pending.popleft()
        for row in by_origin.get(_plan_key(origin), ()):
            related_paths = _related_paths(row["related_code_json"])
            if scope_ticket and not _applies_to_ticket(
                row["origin_workflow_node"],
                related_paths,
                workflow_nodes,
                ticket_paths,
                full_coverage,
                cargo_ticket,
            ):
                continue

            artifact_path = str(row["artifact_path"])
            fixing_plan = _portable_path(row["fixing_plan"])
            summary_slug = str(row["summary_slug"])
            next_path = (*dependency_path, fixing_plan)
            if artifact_path not in emitted:
                emitted.add(artifact_path)
                result.append(
                    {
                        "code": "validation_dependency_failed",
                        "message": (
                            f"Validation depends on open Failure {summary_slug}"
                        ),
                        "repairCondition": (
                            f"Fixing plan {fixing_plan} must resolve and import "
                            f"the fixed lifecycle for {artifact_path}"
                        ),
                        "artifactPath": artifact_path,
                        "lifecycleKey": str(row["lifecycle_key"]),
                        "summarySlug": summary_slug,
                        "originPlan": str(row["origin_plan"]),
                        "originWorkflowNode": (
                            str(row["origin_workflow_node"])
                            if row["origin_workflow_node"] is not None
                            else None
                        ),
                        "fixingPlan": str(row["fixing_plan"]),
                        "relatedPaths": list(related_paths),
                        "dependencyPath": list(next_path),
                        "priority": int(row["priority"]),
                    }
                )
            fixing_key = _plan_key(fixing_plan)
            if fixing_key not in expanded:
                expanded.add(fixing_key)
                pending.append((fixing_plan, next_path, False))

    return tuple(result)


def record_blockers(
    connection: Connection,
    ticket_id: str,
    blockers: tuple[Mapping[str, object], ...] | list[Mapping[str, object]],
    created_at: str,
) -> bool:
    """Append the ticket's blocker projection when it changed.

    An empty projection is retained only when it resolves an earlier non-empty
    projection, so repeated queue scans do not create unbounded no-op events.
    """

    payload = {"blockers": [dict(item) for item in blockers]}
    previous = connection.execute(
        """
        SELECT payload_json FROM validation_ticket_events
        WHERE ticket_id=? AND event_type=?
        ORDER BY event_id DESC
        LIMIT 1
        """,
        (ticket_id, _BLOCKED_EVENT),
    ).fetchone()
    if previous is None:
        if not payload["blockers"]:
            return False
    else:
        try:
            previous_payload = json.loads(str(previous["payload_json"]))
        except (TypeError, ValueError):
            previous_payload = None
        if previous_payload == payload:
            return False

    connection.execute(
        """
        INSERT INTO validation_ticket_events(
            ticket_id, event_type, payload_json, created_at
        ) VALUES (?, ?, ?, ?)
        """,
        (
            ticket_id,
            _BLOCKED_EVENT,
            json.dumps(
                payload,
                sort_keys=True,
                separators=(",", ":"),
                ensure_ascii=True,
                allow_nan=False,
            ),
            created_at,
        ),
    )
    return True


def _applies_to_ticket(
    origin_workflow_node: object,
    related_paths: tuple[str, ...],
    workflow_nodes: tuple[str, ...],
    ticket_paths: tuple[str, ...],
    full_coverage: bool,
    cargo_ticket: bool,
) -> bool:
    if origin_workflow_node is not None and workflow_nodes:
        required = str(origin_workflow_node).strip().casefold()
        return any(_workflow_overlap(required, item) for item in workflow_nodes)
    if full_coverage:
        return True
    if related_paths and ticket_paths:
        if cargo_ticket:
            return True
        return any(
            _path_overlap(related, candidate)
            for related in related_paths
            for candidate in ticket_paths
        )
    return True


def _cargo_ticket(ticket: object) -> bool:
    # Ticket persistence calls this gate, so resolve its lane contract lazily.
    from .validation_tickets import validation_uses_cargo_lane

    command = _ticket_value(ticket, "command", ())
    if not isinstance(command, (list, tuple)):
        command = ()
    toolchain = _ticket_value(ticket, "toolchain", {})
    if not isinstance(toolchain, Mapping):
        toolchain = {}
    return validation_uses_cargo_lane(tuple(command), toolchain)


def _full_coverage(ticket: object) -> bool:
    coverage = _ticket_value(ticket, "coverage", {})
    if not isinstance(coverage, Mapping):
        return False
    return coverage.get("fullCoverage") is True or coverage.get("full_coverage") is True


def _workflow_nodes(ticket: object) -> tuple[str, ...]:
    coverage = _ticket_value(ticket, "coverage", {})
    if not isinstance(coverage, Mapping):
        return ()
    values: list[str] = []
    for key in (
        "workflowNodeKeys",
        "workflow_node_keys",
        "workflowNodes",
        "workflow_nodes",
        "milestoneIds",
        "milestone_ids",
    ):
        raw = coverage.get(key)
        if isinstance(raw, (list, tuple)):
            values.extend(item for item in raw if isinstance(item, str) and item.strip())
    for key in (
        "workflowNode",
        "workflow_node",
        "milestoneId",
        "milestone_id",
    ):
        raw = coverage.get(key)
        if isinstance(raw, str) and raw.strip():
            values.append(raw)
    return tuple(dict.fromkeys(item.strip().casefold() for item in values))


def _ticket_paths(ticket: object) -> tuple[str, ...]:
    values: list[str] = []
    manifest = _ticket_value(ticket, "source_manifest", {})
    if isinstance(manifest, Mapping):
        values.extend(key for key in manifest if isinstance(key, str) and key.strip())
    coverage = _ticket_value(ticket, "coverage", {})
    if isinstance(coverage, Mapping):
        for key in ("dependencyRoots", "dependency_roots"):
            raw = coverage.get(key)
            if isinstance(raw, (list, tuple)):
                values.extend(
                    item for item in raw if isinstance(item, str) and item.strip()
                )
    return tuple(dict.fromkeys(_portable_path(item) for item in values))


def _related_paths(encoded: object) -> tuple[str, ...]:
    try:
        decoded = json.loads(str(encoded))
    except (TypeError, ValueError):
        return ()
    if not isinstance(decoded, list):
        return ()
    return tuple(
        dict.fromkeys(
            _portable_path(item)
            for item in decoded
            if isinstance(item, str) and item.strip()
        )
    )


def _ticket_value(ticket: object, key: str, default: Any) -> Any:
    if isinstance(ticket, Mapping):
        return ticket.get(key, default)
    return getattr(ticket, key, default)


def _portable_path(value: object) -> str:
    return str(value).strip().replace("\\", "/").strip("/")


def _plan_key(value: object) -> str:
    return _portable_path(value).casefold()


def _path_overlap(left: str, right: str) -> bool:
    left_key = left.casefold().rstrip("/")
    right_key = right.casefold().rstrip("/")
    return (
        left_key == right_key
        or left_key.startswith(right_key + "/")
        or right_key.startswith(left_key + "/")
    )


def _workflow_overlap(left: str, right: str) -> bool:
    return (
        left == right
        or left.startswith(right + ".")
        or right.startswith(left + ".")
    )
