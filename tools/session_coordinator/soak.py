from __future__ import annotations

import argparse
import ctypes
import json
import math
import os
import shutil
import sqlite3
import stat
import subprocess
import tempfile
import time
import urllib.request
from dataclasses import asdict, dataclass
from pathlib import Path

from .client import CoordinatorClient
from .config import CoordinatorConfig
from .control_plane.events import EventStreamService
from .database import Database
from .models import utc_text
from .server import RunningCoordinator


@dataclass(frozen=True, slots=True)
class ResourceSample:
    sampled_at: str
    elapsed_seconds: float
    instance_id: str
    event_cursor: int
    rss_bytes: int
    handle_count: int


@dataclass(frozen=True, slots=True)
class InstanceResourceSummary:
    instance_id: str
    sample_count: int
    rss_growth_bytes: int
    rss_peak_growth_bytes: int
    handle_growth: int
    handle_peak_growth: int
    max_rss_bytes: int
    max_handle_count: int


@dataclass(frozen=True, slots=True)
class SoakSummary:
    status: str
    started_at: str
    completed_at: str
    duration_seconds: float
    sample_count: int
    restart_count: int
    browser_disconnect_count: int
    maintenance_tick_count: int
    first_event_cursor: int
    last_event_cursor: int
    rss_growth_bytes: int
    handle_growth: int
    max_rss_bytes: int
    max_handle_count: int
    instance_count: int
    max_sample_gap_seconds: float
    instances: tuple[InstanceResourceSummary, ...]
    errors: tuple[str, ...]


def run_fixture_soak(
    *,
    duration_seconds: float,
    interval_seconds: float,
    output_path: Path,
    restart_fraction: float = 0.5,
    work_root: Path | None = None,
) -> SoakSummary:
    if duration_seconds <= 0 or interval_seconds <= 0:
        raise ValueError("soak duration and interval must be positive")
    if not 0 < restart_fraction < 1:
        raise ValueError("restart fraction must be between zero and one")
    output_path = output_path.resolve()
    output_path.parent.mkdir(parents=True, exist_ok=True)
    workspace = _create_soak_workspace(work_root)
    expected_duration_seconds = duration_seconds
    rollover_allowance_seconds = min(30.0, max(10.0, interval_seconds * 4))
    disconnect_cadence = max(60.0, interval_seconds * 15)
    maintenance_cadence = max(3600.0, interval_seconds * 60)
    started_at = utc_text()
    started = time.monotonic()
    samples: list[ResourceSample] = []
    errors: list[str] = []
    restart_count = 0
    disconnect_count = 0
    maintenance_count = 0
    next_disconnect = 0.0
    next_maintenance = 0.0
    next_sample = 0.0
    restart_at = 0.0
    restarted = False
    config: CoordinatorConfig | None = None
    running: RunningCoordinator | None = None

    try:
        with workspace.open_marker():
            root = workspace.path
            repo = _initialize_fixture_repo(root / "repo")
            config = CoordinatorConfig.for_repo(
                repo,
                state_root=root / "state",
                port=0,
                watch_interval_seconds=max(60.0, interval_seconds * 4),
                maintenance_interval_seconds=max(3600.0, duration_seconds * 2),
            )
            running = RunningCoordinator.start(config)
            client = CoordinatorClient.from_runtime(config)
            client.command("session.register", {"session_id": "m6-soak"})
            client.command(
                "session.set_status",
                {
                    "session_id": "m6-soak",
                    "status": "active",
                    "reason": "M6 soak started",
                },
            )
            started_at = utc_text()
            started = time.monotonic()
            minimum_sample_count = _minimum_sample_count(
                expected_duration_seconds, interval_seconds
            )
            evidence_deadline = (
                started + duration_seconds + rollover_allowance_seconds
            )
            next_disconnect = started
            next_maintenance = started
            next_sample = started
            restart_at = started + duration_seconds * restart_fraction
            try:
                while True:
                    now = time.monotonic()
                    if now >= started + duration_seconds:
                        if not restarted or _transition_evidence_complete(
                            samples, minimum_sample_count=minimum_sample_count
                        ):
                            break
                    if now >= evidence_deadline:
                        break
                    if not restarted and now >= restart_at and len(samples) >= 2:
                        running, client = _controlled_rollover(running, client, config)
                        restart_count += 1
                        restarted = True
                    client.command("session.heartbeat", {"session_id": "m6-soak"})
                    if samples:
                        with Database(config.database_path).transaction() as connection:
                            connection.execute(
                                """
                                INSERT INTO events(session_id, event_type, payload_json, created_at)
                                VALUES ('m6-soak', 'soak.sample', ?, ?)
                                """,
                                (
                                    json.dumps({"sequence": len(samples)}, sort_keys=True),
                                    utc_text(),
                                ),
                            )
                    health = client.health()
                    snapshot = client.control_snapshot()
                    cursor = int(snapshot["eventCursor"])
                    rss, handles = _process_resources(int(health["pid"]))
                    samples.append(
                        ResourceSample(
                            sampled_at=utc_text(),
                            elapsed_seconds=round(now - started, 3),
                            instance_id=str(health["instance_id"]),
                            event_cursor=cursor,
                            rss_bytes=rss,
                            handle_count=handles,
                        )
                    )
                    if now >= next_disconnect:
                        _disconnect_event_stream(client, max(0, cursor - 1))
                        disconnect_count += 1
                        next_disconnect = now + disconnect_cadence
                    if now >= next_maintenance:
                        client.command(
                            "maintenance.tick",
                            {
                                "apply_cleanup": False,
                                "apply_retention": False,
                                "apply_legacy_archive": False,
                                "apply_lifecycle": False,
                            },
                        )
                        maintenance_count += 1
                        next_maintenance = now + maintenance_cadence
                    now_after_sample = time.monotonic()
                    next_sample = _next_sample_deadline(
                        next_sample, now_after_sample, interval_seconds
                    )
                    remaining = evidence_deadline - now_after_sample
                    if remaining > 0:
                        time.sleep(
                            min(max(0.0, next_sample - now_after_sample), remaining)
                        )
            finally:
                running.stop()
                running = None

            if samples:
                continuity_error = _event_continuity_error(
                    config.database_path,
                    samples[0].event_cursor,
                    samples[-1].event_cursor,
                )
                if continuity_error:
                    errors.append(continuity_error)
    except BaseException as error:
        errors.append(f"{type(error).__name__}: {error}")
        if running is not None:
            try:
                running.stop()
            except BaseException as stop_error:
                errors.append(
                    f"coordinator shutdown failed: {type(stop_error).__name__}: {stop_error}"
                )
            running = None

    completed = time.monotonic()
    summary = summarize_samples(
        samples,
        started_at=started_at,
        completed_at=utc_text(),
        duration_seconds=completed - started,
        expected_duration_seconds=expected_duration_seconds,
        minimum_sample_count=_minimum_sample_count(
            expected_duration_seconds, interval_seconds
        ),
        minimum_browser_disconnect_count=max(
            1, math.ceil(expected_duration_seconds / disconnect_cadence)
        ),
        minimum_maintenance_tick_count=max(
            1, math.ceil(expected_duration_seconds / maintenance_cadence)
        ),
        maximum_sample_gap_seconds=max(5.0, interval_seconds * 3),
        maximum_transition_gap_seconds=rollover_allowance_seconds,
        restart_count=restart_count,
        browser_disconnect_count=disconnect_count,
        maintenance_tick_count=maintenance_count,
        errors=errors,
    )
    _write_soak_report(output_path, workspace.path, True, summary, samples)
    if summary.status == "passed":
        cleanup_error = _remove_workspace_when_released(workspace.path)
        if cleanup_error:
            errors.append(cleanup_error)
            summary = summarize_samples(
                samples,
                started_at=started_at,
                completed_at=utc_text(),
                duration_seconds=completed - started,
                expected_duration_seconds=expected_duration_seconds,
                minimum_sample_count=_minimum_sample_count(
                    expected_duration_seconds, interval_seconds
                ),
                minimum_browser_disconnect_count=max(
                    1, math.ceil(expected_duration_seconds / disconnect_cadence)
                ),
                minimum_maintenance_tick_count=max(
                    1, math.ceil(expected_duration_seconds / maintenance_cadence)
                ),
                maximum_sample_gap_seconds=max(5.0, interval_seconds * 3),
                maximum_transition_gap_seconds=rollover_allowance_seconds,
                restart_count=restart_count,
                browser_disconnect_count=disconnect_count,
                maintenance_tick_count=maintenance_count,
                errors=errors,
            )
            _write_soak_report(output_path, workspace.path, True, summary, samples)
        else:
            _write_soak_report(output_path, workspace.path, False, summary, samples)
    return summary


@dataclass(frozen=True, slots=True)
class SoakWorkspace:
    path: Path
    marker_path: Path

    def open_marker(self):
        return self.marker_path.open("w", encoding="utf-8")


def _create_soak_workspace(work_root: Path | None) -> SoakWorkspace:
    if work_root is None:
        path = Path(tempfile.mkdtemp(prefix="zircon-control-soak-")).resolve()
    else:
        path = work_root.resolve()
        _validate_work_root(path)
        path.mkdir(parents=True, exist_ok=False)
    marker = path / "active.lock"
    return SoakWorkspace(path=path, marker_path=marker)


def _event_continuity_error(
    database_path: Path,
    first_cursor: int,
    final_cursor: int,
) -> str | None:
    if not database_path.is_file():
        return "fixture state database is missing before event continuity verification"
    try:
        stream = EventStreamService(Database(database_path))
        cursor = first_cursor
        while cursor < final_cursor:
            replay = stream.read_after(cursor)
            if replay.resync_required:
                return "event continuity replay required a full resynchronization"
            if not replay.events:
                return (
                    "event continuity replay ended before reaching the final sampled cursor "
                    f"{final_cursor}"
                )
            event_ids = [event.event_id for event in replay.events]
            if event_ids != sorted(event_ids) or event_ids[0] <= cursor:
                return "event continuity replay was not strictly ordered"
            cursor = event_ids[-1]
    except (OSError, sqlite3.DatabaseError) as error:
        return f"event continuity verification failed: {type(error).__name__}"
    return None


def _write_soak_report(
    output_path: Path,
    workspace: Path,
    workspace_retained: bool,
    summary: SoakSummary,
    samples: list[ResourceSample],
) -> None:
    payload = {
        "summary": asdict(summary),
        "samples": [asdict(sample) for sample in samples],
        "workspace": str(workspace),
        "workspaceRetained": workspace_retained,
    }
    temporary = output_path.with_suffix(output_path.suffix + ".tmp")
    temporary.write_text(json.dumps(payload, indent=2, sort_keys=True), encoding="utf-8")
    os.replace(temporary, output_path)


def _remove_workspace_when_released(
    workspace: Path,
    *,
    timeout_seconds: float = 10.0,
) -> str | None:
    deadline = time.monotonic() + timeout_seconds
    while True:
        try:
            shutil.rmtree(workspace, onexc=_retry_readonly_removal)
            return None
        except FileNotFoundError:
            return None
        except OSError as error:
            if time.monotonic() >= deadline:
                return f"workspace cleanup failed: {type(error).__name__}"
            time.sleep(0.1)


def _validate_work_root(path: Path) -> None:
    temporary_root = Path(tempfile.gettempdir()).resolve()
    local_app_data = Path(os.environ.get("LOCALAPPDATA", temporary_root)).resolve()
    durable_root = (local_app_data / "Zircon Session Coordinator" / "soak-runs").resolve()
    if not (
        path.is_relative_to(temporary_root)
        or path.is_relative_to(durable_root)
    ):
        raise ValueError("soak workspace must be below the managed temporary or LocalAppData root")


def _retry_readonly_removal(function, path: str, _error) -> None:
    os.chmod(path, stat.S_IWRITE)
    function(path)


def _minimum_sample_count(duration_seconds: float, interval_seconds: float) -> int:
    expected = math.floor(duration_seconds / interval_seconds)
    restart_allowance = math.ceil(
        min(30.0, duration_seconds * 0.5) / interval_seconds
    )
    floor = 4 if expected >= 4 else 2
    return max(floor, expected - restart_allowance - 1)


def _transition_evidence_complete(
    samples: list[ResourceSample], *, minimum_sample_count: int
) -> bool:
    if len(samples) < minimum_sample_count:
        return False
    transitions: list[str] = []
    counts: dict[str, int] = {}
    for sample in samples:
        counts[sample.instance_id] = counts.get(sample.instance_id, 0) + 1
        if not transitions or transitions[-1] != sample.instance_id:
            transitions.append(sample.instance_id)
    return (
        len(transitions) == 2
        and len(counts) == 2
        and all(count >= 2 for count in counts.values())
    )


def _next_sample_deadline(
    previous_deadline: float,
    current_time: float,
    interval_seconds: float,
) -> float:
    scheduled = previous_deadline + interval_seconds
    if scheduled <= current_time:
        return current_time + interval_seconds
    return scheduled


def summarize_samples(
    samples: list[ResourceSample],
    *,
    started_at: str,
    completed_at: str,
    duration_seconds: float,
    expected_duration_seconds: float | None = None,
    minimum_sample_count: int = 0,
    minimum_browser_disconnect_count: int = 1,
    minimum_maintenance_tick_count: int = 1,
    maximum_sample_gap_seconds: float | None = None,
    maximum_transition_gap_seconds: float | None = None,
    restart_count: int,
    browser_disconnect_count: int,
    maintenance_tick_count: int,
    errors: list[str],
) -> SoakSummary:
    errors = list(errors)
    if expected_duration_seconds is not None and duration_seconds < expected_duration_seconds:
        errors.append(
            f"duration {duration_seconds:.3f}s was shorter than required "
            f"{expected_duration_seconds:.3f}s"
        )
    if len(samples) < minimum_sample_count:
        errors.append(
            f"sample count {len(samples)} was below required {minimum_sample_count}"
        )
    if restart_count != 1:
        errors.append(f"exactly one restart is required; observed {restart_count}")
    if browser_disconnect_count < minimum_browser_disconnect_count:
        errors.append(
            f"browser disconnect count {browser_disconnect_count} was below required "
            f"{minimum_browser_disconnect_count}"
        )
    if maintenance_tick_count < minimum_maintenance_tick_count:
        errors.append(
            f"maintenance tick count {maintenance_tick_count} was below required "
            f"{minimum_maintenance_tick_count}"
        )

    instance_summaries: list[InstanceResourceSummary] = []
    maximum_gap = 0.0
    if not samples:
        errors.append("no resource samples were recorded")
        first_cursor = last_cursor = rss_growth = handle_growth = max_rss = max_handles = 0
    else:
        first = samples[0]
        last = samples[-1]
        first_cursor = first.event_cursor
        last_cursor = last.event_cursor
        max_rss = max(sample.rss_bytes for sample in samples)
        max_handles = max(sample.handle_count for sample in samples)
        if last_cursor <= first_cursor:
            errors.append("event cursor did not advance")
        if any(
            current.event_cursor < previous.event_cursor
            for previous, current in zip(samples, samples[1:])
        ):
            errors.append("event cursor regressed between resource samples")

        sample_pairs = list(zip(samples, samples[1:]))
        gaps = [
            current.elapsed_seconds - previous.elapsed_seconds
            for previous, current in sample_pairs
        ]
        maximum_gap = max(gaps, default=0.0)
        if any(gap < 0 for gap in gaps):
            errors.append("resource sample elapsed time regressed")
        if maximum_sample_gap_seconds is not None:
            for previous, current in sample_pairs:
                gap = current.elapsed_seconds - previous.elapsed_seconds
                is_transition = previous.instance_id != current.instance_id
                limit = (
                    maximum_transition_gap_seconds
                    if is_transition and maximum_transition_gap_seconds is not None
                    else maximum_sample_gap_seconds
                )
                if gap > limit:
                    kind = "rollover sample gap" if is_transition else "sample gap"
                    errors.append(f"{kind} {gap:.3f}s exceeded {limit:.3f}s")
                    break

        transitions: list[str] = []
        grouped: dict[str, list[ResourceSample]] = {}
        for sample in samples:
            grouped.setdefault(sample.instance_id, []).append(sample)
            if not transitions or transitions[-1] != sample.instance_id:
                transitions.append(sample.instance_id)
        if len(transitions) != 2 or len(grouped) != 2:
            errors.append(
                "instance transition must contain exactly one predecessor and one successor; "
                f"observed {transitions}"
            )

        for instance_id, instance_samples in grouped.items():
            if len(instance_samples) < 2:
                errors.append(
                    f"instance {instance_id} recorded fewer than two resource samples"
                )
            instance_first = instance_samples[0]
            instance_last = instance_samples[-1]
            instance_max_rss = max(sample.rss_bytes for sample in instance_samples)
            instance_max_handles = max(sample.handle_count for sample in instance_samples)
            instance_rss_growth = instance_last.rss_bytes - instance_first.rss_bytes
            instance_rss_peak_growth = instance_max_rss - instance_first.rss_bytes
            instance_handle_growth = instance_last.handle_count - instance_first.handle_count
            instance_handle_peak_growth = instance_max_handles - instance_first.handle_count
            instance_summaries.append(
                InstanceResourceSummary(
                    instance_id=instance_id,
                    sample_count=len(instance_samples),
                    rss_growth_bytes=instance_rss_growth,
                    rss_peak_growth_bytes=instance_rss_peak_growth,
                    handle_growth=instance_handle_growth,
                    handle_peak_growth=instance_handle_peak_growth,
                    max_rss_bytes=instance_max_rss,
                    max_handle_count=instance_max_handles,
                )
            )
            rss_limit = max(64 * 1024 * 1024, instance_first.rss_bytes // 4)
            if instance_rss_growth > rss_limit:
                errors.append(
                    f"instance {instance_id} RSS growth {instance_rss_growth} exceeded {rss_limit}"
                )
            if instance_rss_peak_growth > rss_limit:
                errors.append(
                    f"instance {instance_id} RSS peak growth {instance_rss_peak_growth} "
                    f"exceeded {rss_limit}"
                )
            if instance_handle_growth > 128:
                errors.append(
                    f"instance {instance_id} handle growth {instance_handle_growth} exceeded 128"
                )
            if instance_handle_peak_growth > 128:
                errors.append(
                    f"instance {instance_id} handle peak growth "
                    f"{instance_handle_peak_growth} exceeded 128"
                )
        rss_growth = max(
            (item.rss_growth_bytes for item in instance_summaries), default=0
        )
        handle_growth = max(
            (item.handle_growth for item in instance_summaries), default=0
        )
    return SoakSummary(
        status="passed" if not errors else "failed",
        started_at=started_at,
        completed_at=completed_at,
        duration_seconds=round(duration_seconds, 3),
        sample_count=len(samples),
        restart_count=restart_count,
        browser_disconnect_count=browser_disconnect_count,
        maintenance_tick_count=maintenance_tick_count,
        first_event_cursor=first_cursor,
        last_event_cursor=last_cursor,
        rss_growth_bytes=rss_growth,
        handle_growth=handle_growth,
        max_rss_bytes=max_rss,
        max_handle_count=max_handles,
        instance_count=len(instance_summaries),
        max_sample_gap_seconds=round(maximum_gap, 3),
        instances=tuple(instance_summaries),
        errors=tuple(errors),
    )


def _controlled_rollover(
    running: RunningCoordinator,
    client: CoordinatorClient,
    config: CoordinatorConfig,
) -> tuple[RunningCoordinator, CoordinatorClient]:
    preview = client.control_request(
        "POST",
        "/control/v1/actions/preview",
        {"kind": "service.rollover", "parameters": {"timeoutSeconds": 30}},
    )["action"]
    confirmed = client.control_request(
        "POST",
        f"/control/v1/actions/{preview['actionId']}/confirm",
        {
            "phrase": preview["confirmationPhrase"],
            "reason": "M6 deterministic soak controlled rollover",
        },
    )["action"]
    if confirmed["status"] != "executing":
        raise RuntimeError(
            f"controlled rollover action was not executing: {confirmed['status']}"
        )
    action_id = str(preview["actionId"])
    _wait_for_rollover_handoff(
        running, config.database_path, action_id, timeout_seconds=30
    )
    running.stop()
    successor = RunningCoordinator.start(config)
    _assert_rollover_completed(
        config.database_path,
        action_id,
        successor.instance_id,
    )
    return successor, CoordinatorClient.from_runtime(config)


def _wait_for_rollover_handoff(
    running: RunningCoordinator,
    database_path: Path,
    action_id: str,
    *,
    timeout_seconds: float,
) -> None:
    deadline = time.monotonic() + timeout_seconds
    last_state = "intent=missing, action=unknown"
    while time.monotonic() < deadline:
        with Database(database_path).connect() as connection:
            row = connection.execute(
                """
                SELECT intent.status AS intent_status, action.status AS action_status,
                       intent.error_code AS intent_error, action.error_code AS action_error
                FROM action_requests AS action
                LEFT JOIN service_lifecycle_intents AS intent
                  ON intent.action_id = action.action_id
                WHERE action.action_id = ?
                """,
                (action_id,),
            ).fetchone()
        if row is not None:
            last_state = (
                f"intent={row['intent_status']}, action={row['action_status']}, "
                f"intent_error={row['intent_error']}, action_error={row['action_error']}"
            )
            if row["intent_status"] == "failed" or row["action_status"] == "failed":
                raise RuntimeError(f"controlled rollover failed before handoff: {last_state}")
            if (
                row["intent_status"] == "awaiting_restart"
                and row["action_status"] == "executing"
                and not running.thread.is_alive()
            ):
                return
        time.sleep(0.05)
    raise TimeoutError(f"controlled rollover handoff timed out: {last_state}")


def _assert_rollover_completed(
    database_path: Path,
    action_id: str,
    successor_instance_id: str,
) -> None:
    with Database(database_path).connect() as connection:
        row = connection.execute(
            """
            SELECT intent.status AS intent_status, action.status AS action_status,
                   intent.successor_daemon_instance_id AS successor_instance_id
            FROM action_requests AS action
            JOIN service_lifecycle_intents AS intent ON intent.action_id = action.action_id
            WHERE action.action_id = ?
            """,
            (action_id,),
        ).fetchone()
    if row is None:
        raise RuntimeError("controlled rollover action or lifecycle intent disappeared")
    if (
        row["intent_status"] != "succeeded"
        or row["action_status"] != "succeeded"
        or row["successor_instance_id"] != successor_instance_id
    ):
        raise RuntimeError(
            "controlled rollover did not reach a terminal succeeded state: "
            f"intent={row['intent_status']}, action={row['action_status']}, "
            f"successor={row['successor_instance_id']}"
        )


def _disconnect_event_stream(client: CoordinatorClient, cursor: int) -> None:
    request = urllib.request.Request(
        f"{client.base_url}/control/v1/events/stream?cursor={cursor}",
        headers={"Authorization": f"Bearer {client.token}"},
    )
    with urllib.request.urlopen(request, timeout=5) as response:
        response.read(1)


def _initialize_fixture_repo(repo: Path) -> Path:
    repo.mkdir(parents=True)
    (repo / "README.md").write_text("workflow control soak fixture\n", encoding="utf-8")
    commands = (
        ("init", "-q", "-b", "main"),
        ("config", "user.email", "zircon-soak@example.invalid"),
        ("config", "user.name", "ZirconSoak"),
        ("add", "README.md"),
        ("commit", "-q", "-m", "chore: initialize soak fixture"),
    )
    for arguments in commands:
        subprocess.run(["git", *arguments], cwd=repo, check=True, capture_output=True)
    return repo.resolve()


def _process_resources(pid: int) -> tuple[int, int]:
    if os.name != "nt":
        return 0, 0

    class ProcessMemoryCounters(ctypes.Structure):
        _fields_ = [
            ("cb", ctypes.c_ulong),
            ("PageFaultCount", ctypes.c_ulong),
            ("PeakWorkingSetSize", ctypes.c_size_t),
            ("WorkingSetSize", ctypes.c_size_t),
            ("QuotaPeakPagedPoolUsage", ctypes.c_size_t),
            ("QuotaPagedPoolUsage", ctypes.c_size_t),
            ("QuotaPeakNonPagedPoolUsage", ctypes.c_size_t),
            ("QuotaNonPagedPoolUsage", ctypes.c_size_t),
            ("PagefileUsage", ctypes.c_size_t),
            ("PeakPagefileUsage", ctypes.c_size_t),
        ]

    query = 0x1000
    kernel32 = ctypes.windll.kernel32
    psapi = ctypes.windll.psapi
    handle = kernel32.OpenProcess(query, False, pid)
    if not handle:
        raise OSError(ctypes.get_last_error(), "OpenProcess failed")
    try:
        counters = ProcessMemoryCounters()
        counters.cb = ctypes.sizeof(counters)
        if not psapi.GetProcessMemoryInfo(
            handle, ctypes.byref(counters), ctypes.sizeof(counters)
        ):
            raise OSError(ctypes.get_last_error(), "GetProcessMemoryInfo failed")
        handle_count = ctypes.c_ulong()
        if not kernel32.GetProcessHandleCount(handle, ctypes.byref(handle_count)):
            raise OSError(ctypes.get_last_error(), "GetProcessHandleCount failed")
        return int(counters.WorkingSetSize), int(handle_count.value)
    finally:
        kernel32.CloseHandle(handle)


def main() -> int:
    parser = argparse.ArgumentParser(description="Run the isolated workflow-control soak")
    parser.add_argument("--hours", type=float, default=24.0)
    parser.add_argument("--interval-seconds", type=float, default=60.0)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--work-root", type=Path)
    args = parser.parse_args()
    summary = run_fixture_soak(
        duration_seconds=args.hours * 3600,
        interval_seconds=args.interval_seconds,
        output_path=args.output,
        work_root=args.work_root,
    )
    print(json.dumps(asdict(summary), sort_keys=True))
    return 0 if summary.status == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
