from __future__ import annotations

import json
import os
import uuid
from collections.abc import Callable, Iterable
from datetime import datetime, timedelta, timezone, tzinfo
from pathlib import Path

from ..database import Database
from .history import CodexHistoricalEvidenceCollector, repository_evidence_key


class CodexEvidenceProjector:
    """Publish a bounded, prompt-free live evidence view for the local user."""

    _LIVE_WINDOW = timedelta(hours=4)
    _MAX_CODEX_SESSIONS = 50
    _MAX_COORDINATOR_SESSIONS = 100
    _MAX_OPEN_FAILURES = 40
    _MAX_RECENT_ACTIONS = 20
    _MAX_RECENT_RESERVATIONS = 20
    _MAX_CURRENT_RESERVATIONS = 20
    _MAX_CURRENT_HEALTH_TIMEOUTS = 20
    _STALE_PROJECTION_TEMP_GRACE = timedelta(hours=1)

    def __init__(
        self,
        database: Database,
        *,
        codex_home: str | Path,
        repo_root: str | Path,
        now: Callable[[], datetime] | None = None,
        local_timezone: tzinfo | None = None,
    ) -> None:
        self.database = database
        self.codex_home = Path(codex_home).resolve()
        self.repo_root = Path(repo_root).resolve()
        self._now = now or (lambda: datetime.now(timezone.utc))
        self._local_timezone = (
            local_timezone or datetime.now().astimezone().tzinfo or timezone.utc
        )
        self._history = CodexHistoricalEvidenceCollector(
            database,
            codex_home=self.codex_home,
            repo_root=self.repo_root,
        )

    def project(self, *, run_id: str, include_history: bool = False) -> Path:
        generated_at = self._now().astimezone(timezone.utc)
        local_generated_at = generated_at.astimezone(self._local_timezone)
        live_since = (generated_at - self._LIVE_WINDOW).isoformat()
        self._history.advance_month(generated_at)
        if include_history:
            self._history.render_month_history(generated_at)
        with self.database.transaction(immediate=False) as connection:
            coordinator_event_cursor = int(
                connection.execute("SELECT COALESCE(MAX(event_id), 0) FROM events").fetchone()[0]
            )
            codex_sessions = connection.execute(
                """SELECT thread_id, state, source_location, rollout_path,
                          last_event, last_activity_at, diagnostic_code
                   FROM codex_sessions
                   WHERE source_location='active' AND last_activity_at >= ?
                   ORDER BY last_activity_at DESC, thread_id ASC LIMIT ?""",
                (live_since, self._MAX_CODEX_SESSIONS),
            ).fetchall()
            coordinator_sessions = connection.execute(
                """SELECT session_id, status, plan_path, last_heartbeat_at, status_reason
                   FROM sessions
                   WHERE (status IN ('active', 'resolving_failure', 'waiting_lease',
                                     'waiting_validation')
                          AND last_heartbeat_at >= ?)
                      OR (status='registered' AND last_heartbeat_at >= ?)
                   ORDER BY last_heartbeat_at DESC, session_id ASC LIMIT ?""",
                (live_since, live_since, self._MAX_COORDINATOR_SESSIONS),
            ).fetchall()
            cargo_jobs = connection.execute(
                """SELECT job_id, session_id, status, lane_kind, started_at, exit_code,
                          last_heartbeat_at
                   FROM cargo_jobs
                   WHERE status IN ('leased', 'running')
                   ORDER BY started_at, job_id LIMIT 100"""
            ).fetchall()
            current_reservations = connection.execute(
                """SELECT reservations.reservation_id, reservations.session_id,
                          reservations.lane_scope, reservations.status,
                          reservations.execution_mode, reservations.created_at,
                          reservations.job_id, reservations.expires_at,
                          jobs.status AS job_status
                   FROM cargo_lane_reservations AS reservations
                   LEFT JOIN cargo_jobs AS jobs ON jobs.job_id=reservations.job_id
                   WHERE reservations.status IN ('pending', 'leased', 'running', 'finished')
                   ORDER BY reservations.lane_scope,
                            CASE WHEN reservations.lane_scope='cpu'
                                 THEN reservations.execution_mode ELSE 'shared' END,
                            reservations.created_at,
                            reservations.reservation_id
                   LIMIT ?""",
                (self._MAX_CURRENT_RESERVATIONS,),
            ).fetchall()
            current_reservations = _with_fifo_positions(current_reservations)
            health_timeout_events = connection.execute(
                """SELECT session_id, payload_json, created_at
                   FROM events
                   WHERE event_type='cargo.health_timeout'
                     AND created_at >= ?
                   ORDER BY created_at DESC, event_id DESC LIMIT ?""",
                (live_since, self._MAX_CURRENT_HEALTH_TIMEOUTS * 4),
            ).fetchall()
            reservations = connection.execute(
                """SELECT reservation_id, session_id, lane_scope, status, completed_at
                   FROM cargo_lane_reservations
                   WHERE lane_scope='cpu'
                     AND status IN ('finished', 'released', 'expired')
                     AND completed_at IS NOT NULL
                     AND completed_at >= ?
                   ORDER BY completed_at DESC, reservation_id DESC LIMIT ?""",
                (live_since, self._MAX_RECENT_RESERVATIONS),
            ).fetchall()
            failures = connection.execute(
                """SELECT summary_slug, fixing_plan, origin_plan, priority, artifact_path
                   FROM failure_nodes
                   WHERE kind='failure' AND status='open'
                   ORDER BY priority ASC, created_at DESC, node_id DESC LIMIT ?""",
                (self._MAX_OPEN_FAILURES,),
            ).fetchall()
            actions = connection.execute(
                """SELECT action_id, action_kind, status, error_code, result_json, completed_at
                   FROM action_requests
                   WHERE status='executing'
                      OR (status NOT IN ('previewed', 'expired') AND completed_at >= ?)
                   ORDER BY created_at DESC, action_id DESC LIMIT ?""",
                (live_since, self._MAX_RECENT_ACTIONS),
            ).fetchall()
            history_progress = connection.execute(
                """SELECT COUNT(*) AS source_count,
                          COALESCE(SUM(CASE WHEN scan_complete=1 THEN 1 ELSE 0 END), 0)
                              AS complete_count
                   FROM codex_evidence_sources"""
            ).fetchone()
            history_record_count = connection.execute(
                "SELECT COUNT(*) FROM codex_evidence_records"
            ).fetchone()[0]
            external_evidence = connection.execute(
                """SELECT thread_id, rollout_name, event_key_hash, kind, outcome,
                          exit_code, event_at
                   FROM codex_evidence_records
                   WHERE event_at >= ?
                   ORDER BY event_at DESC, evidence_id DESC
                   LIMIT 50""",
                (live_since,),
            ).fetchall()
        health_timeouts = _current_health_timeouts(
            health_timeout_events,
            cargo_jobs,
            limit=self._MAX_CURRENT_HEALTH_TIMEOUTS,
        )

        lines = [
            "# ZirconEngine Session Evidence（实时）",
            "",
            f"- 生成时间：`{generated_at.isoformat()}`",
            f"- 同步运行：`{_cell(run_id)}`",
            f"- 仓库：`{_cell(self.repo_root.name)}`",
            f"- 协调器快照游标：`{coordinator_event_cursor}`",
            "- 隐私边界：不写入会话提示词、命令行、日志正文、CWD、绝对路径或 webhook。",
            "",
            "## 历史回填进度",
            "",
            "| 已发现来源 | 已完成来源 | 脱敏事件 |",
            "| ---: | ---: | ---: |",
            "| {sources} | {complete} | {records} |".format(
                sources=history_progress["source_count"],
                complete=history_progress["complete_count"],
                records=history_record_count,
            ),
            "",
            "## Codex 会话",
            "",
            "| Thread | 状态 | 来源 | 最近活动 | Rollout |",
            "| --- | --- | --- | --- | --- |",
        ]
        lines.extend(
            "| {thread} | {state} | {source} | {activity} | {rollout} |".format(
                thread=_cell(row["thread_id"]),
                state=_cell(row["state"]),
                source=_cell(row["source_location"]),
                activity=_cell(row["last_activity_at"]),
                rollout=_cell(_basename(row["rollout_path"])),
            )
            for row in codex_sessions
        )
        if not codex_sessions:
            lines.append("| — | — | — | — | — |")

        lines.extend(
            [
                "",
                "## 最近外部会话证据",
                "",
                "| 时间 | Thread | Rollout | 类型 | 结果 | Exit | 事件键 |",
                "| --- | --- | --- | --- | --- | --- | --- |",
            ]
        )
        lines.extend(
            "| {event_at} | {thread_id} | {rollout_name} | {kind} | {outcome} | {exit_code} | {event_key} |".format(
                event_at=_cell(row["event_at"]),
                thread_id=_cell(row["thread_id"]),
                rollout_name=_cell(row["rollout_name"]),
                kind=_cell(row["kind"]),
                outcome=_cell(row["outcome"]),
                exit_code=_cell(row["exit_code"] if row["exit_code"] is not None else "—"),
                event_key=_cell(str(row["event_key_hash"])[:12]),
            )
            for row in external_evidence
        )
        if not external_evidence:
            lines.append("| — | — | — | — | — | — | — |")

        lines.extend(
            [
                "",
                "## 协调器 Session",
                "",
                "| Session | 状态 | 计划 | 最近心跳 | 摘要 |",
                "| --- | --- | --- | --- | --- |",
            ]
        )
        lines.extend(
            "| {session} | {state} | {plan} | {heartbeat} | {reason} |".format(
                session=_cell(row["session_id"]),
                state=_cell(row["status"]),
                plan=_cell(row["plan_path"] or "—"),
                heartbeat=_cell(row["last_heartbeat_at"]),
                reason=_cell(row["status_reason"] or "—"),
            )
            for row in coordinator_sessions
        )
        if not coordinator_sessions:
            lines.append("| — | — | — | — | — |")

        lines.extend(
            [
                "",
                "## 当前托管 Cargo",
                "",
                "| Job | Session | 阶段 | 状态 | 开始时间 | Exit |",
                "| --- | --- | --- | --- | --- |",
            ]
        )
        lines.extend(
            "| {job} | {session} | {lane} | {status} | {started} | {exit_code} |".format(
                job=_cell(row["job_id"]),
                session=_cell(row["session_id"]),
                lane=_cell(row["lane_kind"]),
                status=_cell(row["status"]),
                started=_cell(row["started_at"] or "—"),
                exit_code=_cell(row["exit_code"] if row["exit_code"] is not None else "—"),
            )
            for row in cargo_jobs
        )
        if not cargo_jobs:
            lines.append("| — | — | — | — | — | — |")

        lines.extend(
            [
                "",
                "## 当前预约队列",
                "",
                "只显示仍会占用调度车道的预约；FIFO 顺位来自同一协调器快照。",
                "历史终态另列，且不暴露命令、兼容性或目标路径。",
                "",
                "| 预约 | FIFO 顺位 | Job | Session | Lane | 预约状态 | Job 状态 | 到期时间 |",
                "| --- | --- | --- | --- | --- | --- | --- | --- |",
            ]
        )
        lines.extend(
            "| {reservation} | {fifo} | {job} | {session} | {lane} | {status} | {job_status} | {expires} |".format(
                reservation=_cell(row["reservation_id"]),
                fifo=_cell(row["fifo_position"]),
                job=_cell(row["job_id"] or "—"),
                session=_cell(row["session_id"]),
                lane=_cell(row["lane_scope"]),
                status=_cell(row["status"]),
                job_status=_cell(row["job_status"] or "—"),
                expires=_cell(row["expires_at"]),
            )
            for row in current_reservations
        )
        if not current_reservations:
            lines.append("| — | — | — | — | — | — | — | — |")

        lines.extend(
            [
                "",
                "## 当前任务健康告警",
                "",
                "只显示仍在运行且自告警后没有新心跳的任务；告警不会关闭其他任务准入。",
                "",
                "| Job | Session | Lane | 心跳超时 / 阈值 | 存活 PID 数 | 观察时间 |",
                "| --- | --- | --- | --- | ---: | --- |",
            ]
        )
        lines.extend(
            "| {job} | {session} | {lane} | {age}s / {timeout}s | {pid_count} | {observed} |".format(
                job=_cell(row["job_id"]),
                session=_cell(row["session_id"]),
                lane=_cell(row["lane_kind"]),
                age=_cell(row["heartbeat_age_seconds"]),
                timeout=_cell(row["timeout_seconds"]),
                pid_count=_cell(row["live_pid_count"]),
                observed=_cell(row["observed_at"]),
            )
            for row in health_timeouts
        )
        if not health_timeouts:
            lines.append("| — | — | — | — | — | — |")

        lines.extend(
            [
                "",
                "## 最近调度转换",
                "",
                "| 预约 | Session | Lane | 终态 | 完成时间 |",
                "| --- | --- | --- | --- | --- |",
            ]
        )
        lines.extend(
            "| {reservation} | {session} | {lane} | {status} | {completed} |".format(
                reservation=_cell(row["reservation_id"]),
                session=_cell(row["session_id"]),
                lane=_cell(row["lane_scope"]),
                status=_cell(row["status"]),
                completed=_cell(row["completed_at"]),
            )
            for row in reservations
        )
        if not reservations:
            lines.append("| — | — | — | — | — |")

        lines.extend(
            [
                "",
                "## 开放 Failure",
                "",
                "| 摘要 | 修复计划 | 来源计划 | 优先级 | 记录 |",
                "| --- | --- | --- | --- | --- |",
            ]
        )
        lines.extend(
            "| {summary} | {fixing} | {origin} | {priority} | {artifact} |".format(
                summary=_cell(row["summary_slug"]),
                fixing=_cell(row["fixing_plan"]),
                origin=_cell(row["origin_plan"]),
                priority=_cell(row["priority"]),
                artifact=_cell(row["artifact_path"]),
            )
            for row in failures
        )
        if not failures:
            lines.append("| — | — | — | — | — |")

        lines.extend(
            [
                "",
                "## 最近受控动作",
                "",
                "| Action | 类型 | 状态 | Commit | 错误码 | 完成时间 |",
                "| --- | --- | --- | --- | --- | --- |",
            ]
        )
        lines.extend(
            "| {action} | {kind} | {status} | {commit} | {error} | {completed} |".format(
                action=_cell(row["action_id"]),
                kind=_cell(row["action_kind"]),
                status=_cell(row["status"]),
                commit=_cell(_commit_sha(row["result_json"]) or "—"),
                error=_cell(row["error_code"] or "—"),
                completed=_cell(row["completed_at"] or "—"),
            )
            for row in actions
        )
        if not actions:
            lines.append("| — | — | — | — | — | — |")

        target = (
            self.codex_home
            / "sessions"
            / local_generated_at.strftime("%Y")
            / local_generated_at.strftime("%m")
            / (
                f"zircon-engine-evidence-live-{local_generated_at.date().isoformat()}-"
                f"{repository_evidence_key(self.repo_root)}.md"
            )
        )
        target.parent.mkdir(parents=True, exist_ok=True)
        temporary = target.with_name(f".{target.name}.{uuid.uuid4().hex}.tmp")
        temporary.write_text("\n".join(lines) + "\n", encoding="utf-8")
        os.replace(temporary, target)
        _remove_stale_projection_temporaries(
            target.parent,
            generated_at,
            grace=self._STALE_PROJECTION_TEMP_GRACE,
        )
        return target


def _basename(value: object) -> str:
    text = str(value or "")
    return text.replace("\\", "/").rsplit("/", 1)[-1] or "—"


def _cell(value: object) -> str:
    return str(value).replace("|", "\\|").replace("\n", " ").replace("\r", " ")


def _with_fifo_positions(rows: Iterable[object]) -> list[dict[str, object]]:
    """Annotate reservation rows using the same lane/mode FIFO partition as admission."""
    positions: dict[tuple[str, str], int] = {}
    projected: list[dict[str, object]] = []
    for row in rows:
        lane_scope = str(row["lane_scope"])
        execution_mode = (
            str(row["execution_mode"] or "warm")
            if lane_scope == "cpu"
            else "shared"
        )
        lane_key = (lane_scope, execution_mode)
        position = positions.get(lane_key, 0) + 1
        positions[lane_key] = position
        projected_row = dict(row)
        projected_row["fifo_position"] = f"{lane_scope}/{execution_mode} #{position}"
        projected.append(projected_row)
    return projected


def _remove_stale_projection_temporaries(
    directory: Path,
    generated_at: datetime,
    *,
    grace: timedelta,
) -> None:
    """Remove only abandoned atomic-write files after a successful replacement."""
    cutoff = generated_at.timestamp() - grace.total_seconds()
    try:
        candidates = tuple(directory.glob(".zircon-engine-evidence-*.md.*.tmp"))
    except OSError:
        return
    for temporary in candidates:
        try:
            if temporary.stat().st_mtime > cutoff:
                continue
            temporary.unlink()
        except OSError:
            continue


def _commit_sha(result_json: object) -> str | None:
    try:
        parsed = json.loads(str(result_json or "{}"))
    except json.JSONDecodeError:
        return None
    return next(_commit_values(parsed), None)


def _current_health_timeouts(
    events: Iterable[object],
    cargo_jobs: Iterable[object],
    *,
    limit: int,
) -> list[dict[str, object]]:
    """Return only still-stale active jobs, never a historical alert ledger."""

    active_jobs = {str(row["job_id"]): row for row in cargo_jobs}
    current: list[dict[str, object]] = []
    reported_job_ids: set[str] = set()
    for event in events:
        try:
            payload = json.loads(str(event["payload_json"]))
        except (json.JSONDecodeError, TypeError):
            continue
        if not isinstance(payload, dict):
            continue
        job_id = payload.get("jobId")
        if not isinstance(job_id, str) or job_id in reported_job_ids:
            continue
        job = active_jobs.get(job_id)
        if job is None:
            continue
        if str(job["last_heartbeat_at"]) > str(event["created_at"]):
            continue
        live_pids = payload.get("livePids")
        if not isinstance(live_pids, list):
            live_pids = []
        current.append(
            {
                "job_id": job_id,
                "session_id": str(job["session_id"]),
                "lane_kind": str(job["lane_kind"]),
                "heartbeat_age_seconds": _nonnegative_int(
                    payload.get("heartbeatAgeSeconds")
                ),
                "timeout_seconds": _nonnegative_int(payload.get("timeoutSeconds")),
                "live_pid_count": len(live_pids),
                "observed_at": str(event["created_at"]),
            }
        )
        reported_job_ids.add(job_id)
        if len(current) >= limit:
            break
    return current


def _nonnegative_int(value: object) -> int:
    try:
        return max(0, int(value))
    except (TypeError, ValueError):
        return 0


def _commit_values(value: object) -> Iterable[str]:
    if isinstance(value, dict):
        candidate = value.get("commitSha")
        if isinstance(candidate, str) and candidate:
            yield candidate
        for nested in value.values():
            yield from _commit_values(nested)
    elif isinstance(value, list):
        for nested in value:
            yield from _commit_values(nested)
