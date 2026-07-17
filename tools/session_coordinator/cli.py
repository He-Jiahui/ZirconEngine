from __future__ import annotations

import argparse
import json
import os
import shlex
import sys
import time
import uuid
import webbrowser
from pathlib import Path
from typing import Any

from .client import CoordinatorClient, CoordinatorClientError
from .config import CoordinatorConfig
from .models import CoordinatorError
from .offline_queue import OfflineCommandSpool
from .server import run_forever


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="zircon-session")
    parser.add_argument("--repo-root", default=str(Path.cwd()))
    parser.add_argument("--state-root")
    parser.add_argument("--json", action="store_true", dest="json_output")
    commands = parser.add_subparsers(dest="command", required=True)
    serve = commands.add_parser("serve")
    serve.add_argument("--automatic-start", action="store_true")
    commands.add_parser("status")
    commands.add_parser("stop")

    offline_queue = commands.add_parser("offline-queue")
    offline_queue_commands = offline_queue.add_subparsers(
        dest="offline_queue_command", required=True
    )
    offline_queue_commands.add_parser("status")
    offline_queue_commands.add_parser("replay")

    ui = commands.add_parser("ui")
    ui_commands = ui.add_subparsers(dest="ui_command", required=True)
    ui_ticket = ui_commands.add_parser("ticket")
    ui_ticket.add_argument("--role", choices=("observer",), default="observer")
    ui_ticket.add_argument("--actor", default="local-cli")
    ui_open = ui_commands.add_parser("open")
    ui_open.add_argument("--actor", default="local-cli")

    control = commands.add_parser("control")
    control_commands = control.add_subparsers(dest="control_command", required=True)
    control_commands.add_parser("snapshot")
    control_elevate = control_commands.add_parser("elevate")
    control_elevate.add_argument(
        "--role", choices=("operator", "committer", "maintainer"), default="operator"
    )
    control_elevate.add_argument("--session-id")
    control_elevate.add_argument("--actor", default="local-cli")

    session = commands.add_parser("session")
    session_commands = session.add_subparsers(dest="session_command", required=True)

    register = session_commands.add_parser("register")
    register.add_argument("--session-id")
    register.add_argument("--display-name")
    register.add_argument("--plan-path")
    register.add_argument("--write-scope", action="append", default=[])

    listing = session_commands.add_parser("list")
    listing.add_argument("--include-archived", action="store_true")

    show = session_commands.add_parser("show")
    show.add_argument("--session-id")

    heartbeat = session_commands.add_parser("heartbeat")
    heartbeat.add_argument("--session-id")

    set_status = session_commands.add_parser("set-status")
    set_status.add_argument("status")
    set_status.add_argument("--session-id")
    set_status.add_argument("--reason")

    baseline = commands.add_parser("baseline")
    baseline_commands = baseline.add_subparsers(dest="baseline_command", required=True)
    for name in ("init", "status", "diff", "scan", "reconcile"):
        baseline_commands.add_parser(name)
    baseline_accept = baseline_commands.add_parser("accept")
    baseline_accept.add_argument("--reason", required=True)
    baseline_attribute = baseline_commands.add_parser("attribute")
    baseline_attribute.add_argument("paths", nargs="+")
    baseline_attribute.add_argument("--session-id")

    lease = commands.add_parser("lease")
    lease_commands = lease.add_subparsers(dest="lease_command", required=True)
    lease_claim = lease_commands.add_parser("claim")
    lease_claim.add_argument("paths", nargs="+")
    lease_claim.add_argument("--session-id")
    lease_release = lease_commands.add_parser("release")
    lease_release.add_argument("paths", nargs="*")
    lease_release.add_argument("--session-id")
    lease_heartbeat = lease_commands.add_parser("heartbeat")
    lease_heartbeat.add_argument("--session-id")
    lease_commands.add_parser("list")

    snapshot = commands.add_parser("snapshot")
    snapshot_commands = snapshot.add_subparsers(dest="snapshot_command", required=True)
    snapshot_create = snapshot_commands.add_parser("create")
    snapshot_create.add_argument("paths", nargs="+")
    snapshot_create.add_argument("--session-id")
    snapshot_create.add_argument("--baseline-epoch", type=int)
    snapshot_create.add_argument("--purpose", required=True)
    snapshot_preview = snapshot_commands.add_parser("preview")
    snapshot_preview.add_argument("snapshot_id", type=int)

    patch = commands.add_parser("patch")
    patch_commands = patch.add_subparsers(dest="patch_command", required=True)
    patch_enqueue = patch_commands.add_parser("enqueue")
    patch_enqueue.add_argument("--file", required=True)
    patch_enqueue.add_argument("--target", action="append", required=True)
    patch_enqueue.add_argument("--session-id")
    patch_status = patch_commands.add_parser("status")
    patch_status.add_argument("patch_id", type=int)
    patch_list = patch_commands.add_parser("list")
    patch_list.add_argument("--status")
    patch_commands.add_parser("process")

    watch = commands.add_parser("watch")
    watch_commands = watch.add_subparsers(dest="watch_command", required=True)
    watch_commands.add_parser("scan")

    plan = commands.add_parser("plan")
    plan_commands = plan.add_subparsers(dest="plan_command", required=True)
    plan_commands.add_parser("audit")
    plan_owner = plan_commands.add_parser("owner")
    plan_owner.add_argument("plan_path")
    plan_authorize = plan_commands.add_parser("authorize")
    plan_authorize.add_argument("target_path")
    plan_authorize.add_argument("--session-id")
    plan_authorize.add_argument("--maintenance", action="store_true")

    failure = commands.add_parser("failure")
    failure_commands = failure.add_subparsers(dest="failure_command", required=True)
    failure_commands.add_parser("import")
    failure_commands.add_parser("audit")
    failure_open = failure_commands.add_parser("open")
    failure_open.add_argument("fixing_plan")
    failure_return = failure_commands.add_parser("return")
    failure_return.add_argument("lifecycle_key")
    failure_return.add_argument("--session-id")
    failure_return.add_argument("--resolved-at", required=True)
    failure_return.add_argument("--root-cause", required=True)
    failure_return.add_argument("--architecture-fix", required=True)
    failure_return.add_argument("--validation", required=True)
    failure_return.add_argument("--return-summary", required=True)

    cargo = commands.add_parser("cargo")
    cargo_commands = cargo.add_subparsers(dest="cargo_command", required=True)
    cargo_acquire = cargo_commands.add_parser("acquire")
    cargo_acquire.add_argument("lane_kind", choices=("check", "test", "workspace", "gpu"))
    cargo_acquire.add_argument("--session-id")
    cargo_acquire.add_argument("--target-dir")
    cargo_acquire.add_argument("--dry-run", action="store_true")
    cargo_acquire.add_argument("--pid", type=int)
    cargo_acquire.add_argument("--ephemeral", action="store_true")
    cargo_acquire.add_argument("--compatibility-json")
    cargo_reserve_cpu = cargo_commands.add_parser("reserve-cpu")
    cargo_reserve_cpu.add_argument("--session-id")
    cargo_reserve_cpu.add_argument("--compatibility-json", required=True)
    cargo_reserve_cpu.add_argument("--target-dir")
    cargo_reserve_cpu.add_argument("--ttl-seconds", type=int, default=900)
    cargo_reserve_cpu.add_argument("--burst-eligible", action="store_true")
    cargo_reserve_cpu.add_argument("command_args", nargs=argparse.REMAINDER)
    cargo_reserve_gpu = cargo_commands.add_parser("reserve-gpu")
    cargo_reserve_gpu.add_argument("--session-id", required=True)
    cargo_reserve_gpu.add_argument("--compatibility-json", required=True)
    cargo_reserve_gpu.add_argument("--target-dir", required=True)
    cargo_reserve_gpu.add_argument("--ttl-seconds", type=int, default=900)
    cargo_reserve_gpu.add_argument("command_args", nargs=argparse.REMAINDER)
    cargo_release_cpu_reservation = cargo_commands.add_parser("release-cpu-reservation")
    cargo_release_cpu_reservation.add_argument("reservation_id")
    cargo_release_cpu_reservation.add_argument("--session-id")
    cargo_renew_cpu_reservation = cargo_commands.add_parser("renew-cpu-reservation")
    cargo_renew_cpu_reservation.add_argument("reservation_id")
    cargo_renew_cpu_reservation.add_argument("--session-id")
    cargo_renew_cpu_reservation.add_argument("--ttl-seconds", type=int, default=900)
    cargo_consume_cpu_reservation = cargo_commands.add_parser("consume-cpu-reservation")
    cargo_consume_cpu_reservation.add_argument("reservation_id")
    cargo_consume_cpu_reservation.add_argument("--session-id", required=True)
    cargo_consume_cpu_reservation.add_argument(
        "--lane-kind", required=True, choices=("check", "test", "workspace")
    )
    cargo_consume_gpu_reservation = cargo_commands.add_parser("consume-gpu-reservation")
    cargo_consume_gpu_reservation.add_argument("reservation_id")
    cargo_consume_gpu_reservation.add_argument("--session-id", required=True)
    cargo_recover_reservation = cargo_commands.add_parser("recover-reservation")
    cargo_recover_reservation.add_argument("reservation_id")
    cargo_recover_reservation.add_argument("job_id")
    cargo_recover_reservation.add_argument("--session-id", required=True)
    cargo_start = cargo_commands.add_parser("start")
    cargo_start.add_argument("job_id")
    cargo_start.add_argument("--pid", type=int, required=True)
    cargo_start.add_argument("--supervisor", action="store_true")
    cargo_start.add_argument("--session-id")
    cargo_start.add_argument("command_args", nargs="*")
    cargo_run = cargo_commands.add_parser("run")
    cargo_run.add_argument("job_id")
    cargo_run.add_argument("--session-id")
    cargo_run.add_argument("--env", action="append", default=[])
    cargo_run.add_argument("command_args", nargs=argparse.REMAINDER)
    cargo_run_reserved = cargo_commands.add_parser("run-reserved")
    cargo_run_reserved.add_argument("reservation_id")
    cargo_run_reserved.add_argument("job_id")
    cargo_run_reserved.add_argument("--session-id", required=True)
    cargo_run_reserved.add_argument("command_args", nargs=argparse.REMAINDER)
    cargo_run_status = cargo_commands.add_parser("run-status")
    cargo_run_status.add_argument("job_id")
    cargo_run_status.add_argument("--session-id")
    cargo_heartbeat = cargo_commands.add_parser("heartbeat")
    cargo_heartbeat.add_argument("job_id")
    cargo_heartbeat.add_argument("--session-id")
    cargo_finish = cargo_commands.add_parser("finish")
    cargo_finish.add_argument("job_id")
    cargo_finish.add_argument("--exit-code", type=int, required=True)
    cargo_finish.add_argument("--session-id")
    cargo_release = cargo_commands.add_parser("release")
    cargo_release.add_argument("job_id")
    cargo_release.add_argument("--session-id")
    cargo_commands.add_parser("list")

    milestone = commands.add_parser("milestone")
    milestone_commands = milestone.add_subparsers(dest="milestone_command", required=True)
    milestone_prepare = milestone_commands.add_parser("prepare")
    milestone_prepare.add_argument("--session-id", required=True)
    milestone_prepare.add_argument("--milestone", required=True)
    milestone_review = milestone_commands.add_parser("review")
    milestone_review.add_argument("--session-id", required=True)
    milestone_review.add_argument("--executor-session-id", required=True)
    milestone_review.add_argument("--run-id", required=True)
    milestone_review.add_argument("--milestone", required=True)
    milestone_review.add_argument("--critical-count", type=int, required=True)
    milestone_review.add_argument("--important-count", type=int, required=True)
    milestone_review.add_argument("--summary", required=True)
    milestone_validate = milestone_commands.add_parser("validate")
    milestone_validate.add_argument("--session-id", required=True)
    milestone_validate.add_argument("--run-id", required=True)
    milestone_validate.add_argument("--milestone", required=True)
    milestone_validate.add_argument(
        "--template", choices=("coordinator-actions", "web-check"), required=True
    )
    milestone_commit = milestone_commands.add_parser("commit")
    milestone_commit.add_argument("--session-id", required=True)
    milestone_commit.add_argument("--run-id", required=True)
    milestone_commit.add_argument("--milestone", required=True)
    milestone_commit.add_argument("--summary", required=True)
    milestone_goal = milestone_commands.add_parser("close-goal")
    milestone_goal.add_argument("--session-id", required=True)
    milestone_goal.add_argument("--run-id", required=True)

    cleanup = commands.add_parser("cleanup")
    cleanup_commands = cleanup.add_subparsers(dest="cleanup_command", required=True)
    cleanup_plan = cleanup_commands.add_parser("plan")
    cleanup_plan.add_argument("--older-than-hours", type=int, default=2)
    cleanup_apply = cleanup_commands.add_parser("apply")
    cleanup_apply.add_argument("--older-than-hours", type=int, default=2)
    cleanup_apply.add_argument("--plan-id", required=True)

    artifact = commands.add_parser("artifact")
    artifact_commands = artifact.add_subparsers(dest="artifact_command", required=True)
    artifact_commands.add_parser("audit")
    artifact_commands.add_parser("cleanup")

    finalize = commands.add_parser("finalize")
    finalize.add_argument("--commit", dest="finalize_commit", action="store_true")
    finalize.add_argument("--session-id", dest="direct_session_id")
    finalize.add_argument("--message", dest="direct_message")
    finalize.add_argument("--path", dest="direct_paths", action="append")
    finalize.add_argument(
        "--validation-command", dest="direct_validation_commands", action="append", default=[]
    )
    finalize.add_argument("--maintenance", dest="direct_maintenance", action="store_true")
    finalize.add_argument("--milestone", dest="direct_milestone", action="store_true")
    finalize_commands = finalize.add_subparsers(dest="finalize_command", required=False)
    for finalize_name in ("preview", "commit"):
        finalize_parser = finalize_commands.add_parser(finalize_name)
        finalize_parser.add_argument("--session-id")
        finalize_parser.add_argument("--message", required=True)
        finalize_parser.add_argument("--path", action="append", required=True)
        finalize_parser.add_argument("--validation-command", action="append", default=[])
        finalize_parser.add_argument("--maintenance", action="store_true")
        finalize_parser.add_argument("--milestone", action="store_true")

    validation_copy = commands.add_parser("validation-copy")
    validation_copy_commands = validation_copy.add_subparsers(
        dest="validation_copy_command", required=True
    )
    for copy_name in ("plan", "materialize"):
        copy_parser = validation_copy_commands.add_parser(copy_name)
        copy_parser.add_argument("--session-id")
        copy_parser.add_argument("--path", action="append", required=True)
    copy_status = validation_copy_commands.add_parser("status")
    copy_status.add_argument("job_id")
    copy_status.add_argument("--session-id")
    copy_cleanup = validation_copy_commands.add_parser("cleanup")
    copy_cleanup.add_argument("job_root")
    copy_cleanup.add_argument("--session-id")
    copy_run = validation_copy_commands.add_parser("run")
    copy_run.add_argument("job_id")
    copy_run.add_argument("--session-id")
    copy_run.add_argument("command_args", nargs=argparse.REMAINDER)

    legacy = commands.add_parser("legacy")
    legacy_commands = legacy.add_subparsers(dest="legacy_command", required=True)
    legacy_report = legacy_commands.add_parser("report")
    legacy_report.add_argument("--report")
    for legacy_name in ("import", "archive"):
        legacy_action = legacy_commands.add_parser(legacy_name)
        legacy_action.add_argument("--apply", action="store_true")
        legacy_action.add_argument("--dry-run", action="store_true")
        legacy_action.add_argument("--report")

    retention = commands.add_parser("retention")
    retention_commands = retention.add_subparsers(
        dest="retention_command", required=True
    )
    retention_plan = retention_commands.add_parser("plan")
    retention_plan.add_argument("--report")
    retention_apply = retention_commands.add_parser("apply")
    retention_apply.add_argument("--plan-id", required=True)
    retention_apply.add_argument("--dry-run", action="store_true")
    retention_apply.add_argument("--report")

    maintenance = commands.add_parser("maintenance")
    maintenance_commands = maintenance.add_subparsers(
        dest="maintenance_command", required=True
    )
    maintenance_tick = maintenance_commands.add_parser("tick")
    maintenance_tick.add_argument("--apply-cleanup", action="store_true")
    maintenance_tick.add_argument("--apply-retention", action="store_true")
    maintenance_tick.add_argument("--apply-legacy-archive", action="store_true")
    maintenance_tick.add_argument("--apply-lifecycle", action="store_true")
    maintenance_tick.add_argument("--report")

    audit = commands.add_parser("audit")
    audit_commands = audit.add_subparsers(dest="audit_command", required=True)
    audit_all = audit_commands.add_parser("all")
    audit_all.add_argument("--report")
    return parser


def _session_id(value: str | None) -> str:
    return value or os.environ.get("CODEX_THREAD_ID") or f"manual-{uuid.uuid4()}"


def _split_command(value: str) -> list[str]:
    parts = shlex.split(value, posix=False)
    return [
        part[1:-1]
        if len(part) >= 2 and part[0] == part[-1] and part[0] in {'"', "'"}
        else part
        for part in parts
    ]


def _config(arguments: argparse.Namespace) -> CoordinatorConfig:
    return CoordinatorConfig.for_repo(
        arguments.repo_root,
        state_root=arguments.state_root,
    )


def _offline_spool(config: CoordinatorConfig) -> OfflineCommandSpool:
    return OfflineCommandSpool(
        config.offline_command_queue_root,
        repository_key=config.repository_key,
    )


def _offline_queue_intent(arguments: argparse.Namespace) -> tuple[str, dict[str, Any]] | None:
    """Return only replay-safe CLI work; process and lifecycle work never queues."""
    if arguments.command == "session":
        explicit_session_id = getattr(arguments, "session_id", None) or os.environ.get(
            "CODEX_THREAD_ID"
        )
        if arguments.session_command == "register":
            if explicit_session_id is None:
                return None
            return (
                "session.register",
                {
                    "session_id": explicit_session_id,
                    "display_name": arguments.display_name,
                    "plan_path": arguments.plan_path,
                    "write_scope": arguments.write_scope,
                },
            )
        session_id = _session_id(explicit_session_id)
        if arguments.session_command == "heartbeat":
            return "session.heartbeat", {"session_id": session_id}
    if arguments.command == "lease" and arguments.lease_command == "heartbeat":
        return "lease.heartbeat", {"session_id": _session_id(arguments.session_id)}
    return None


def _replay_offline_queue(
    config: CoordinatorConfig, client: CoordinatorClient
) -> dict[str, int]:
    return _offline_spool(config).replay(client.command).to_dict()


def _write_report(path: str | None, payload: dict[str, Any]) -> None:
    if not path:
        return
    destination = Path(path).resolve()
    destination.parent.mkdir(parents=True, exist_ok=True)
    temporary = destination.with_suffix(destination.suffix + ".tmp")
    temporary.write_text(
        json.dumps(payload, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    os.replace(temporary, destination)


def _run(arguments: argparse.Namespace) -> dict[str, Any]:
    config = _config(arguments)
    if arguments.command == "serve":
        run_forever(config, automatic_start=arguments.automatic_start)
        return {"status": "stopped"}
    if arguments.command == "offline-queue":
        spool = _offline_spool(config)
        if arguments.offline_queue_command == "status":
            return {"offlineQueue": spool.snapshot().to_dict()}
        client = CoordinatorClient.from_runtime(config)
        client.health()
        return {"offlineReplay": spool.replay(client.command).to_dict()}
    client = CoordinatorClient.from_runtime(config)
    if arguments.command == "status":
        result = client.health()
        result["offlineReplay"] = _replay_offline_queue(config, client)
        return result
    if arguments.command == "stop":
        result = client.shutdown()
        for _ in range(50):
            if not config.runtime_path.exists():
                break
            time.sleep(0.05)
        return result
    if arguments.command == "ui":
        ticket = client.issue_ui_ticket(
            actor=arguments.actor,
            role=getattr(arguments, "role", "observer"),
        )
        if arguments.ui_command == "ticket":
            return ticket
        bootstrap_url = f"{client.base_url}{ticket['bootstrapPath']}"
        if not webbrowser.open(bootstrap_url, new=2):
            raise CoordinatorClientError(
                "browser_open_failed", "The system browser could not be opened"
            )
        return {"status": "opened", "url": f"{client.base_url}/ui/"}
    if arguments.command == "control" and arguments.control_command == "snapshot":
        return client.control_snapshot()
    if arguments.command == "control" and arguments.control_command == "elevate":
        return client.issue_elevation_grant(
            actor=arguments.actor,
            role=arguments.role,
            session_id=arguments.session_id,
            maintenance_capability=(
                os.environ.get("ZIRCON_COORDINATOR_MAINTENANCE_TOKEN")
                if arguments.role == "maintainer"
                else None
            ),
        )
    if arguments.command == "session" and arguments.session_command == "register":
        return client.command(
            "session.register",
            {
                "session_id": _session_id(arguments.session_id),
                "display_name": arguments.display_name,
                "plan_path": arguments.plan_path,
                "write_scope": arguments.write_scope,
            },
        )
    if arguments.command == "session" and arguments.session_command == "list":
        return client.command(
            "session.list", {"include_archived": arguments.include_archived}
        )
    if arguments.command == "session" and arguments.session_command == "show":
        return client.command("session.show", {"session_id": _session_id(arguments.session_id)})
    if arguments.command == "session" and arguments.session_command == "heartbeat":
        return client.command(
            "session.heartbeat", {"session_id": _session_id(arguments.session_id)}
        )
    if arguments.command == "session" and arguments.session_command == "set-status":
        return client.command(
            "session.set_status",
            {
                "session_id": _session_id(arguments.session_id),
                "status": arguments.status,
                "reason": arguments.reason,
            },
        )
    if arguments.command == "baseline":
        if arguments.baseline_command in {"init", "status", "diff", "scan", "reconcile"}:
            return client.command(f"baseline.{arguments.baseline_command}")
        if arguments.baseline_command == "accept":
            return client.command("baseline.accept", {"reason": arguments.reason})
        if arguments.baseline_command == "attribute":
            return client.command(
                "baseline.attribute",
                {"session_id": _session_id(arguments.session_id), "paths": arguments.paths},
            )
    if arguments.command == "lease":
        if arguments.lease_command == "claim":
            return client.command(
                "lease.claim",
                {"session_id": _session_id(arguments.session_id), "paths": arguments.paths},
            )
        if arguments.lease_command == "release":
            return client.command(
                "lease.release",
                {
                    "session_id": _session_id(arguments.session_id),
                    "paths": arguments.paths or None,
                },
            )
        if arguments.lease_command == "heartbeat":
            return client.command(
                "lease.heartbeat", {"session_id": _session_id(arguments.session_id)}
            )
        if arguments.lease_command == "list":
            return client.command("lease.list")
    if arguments.command == "snapshot":
        if arguments.snapshot_command == "create":
            return client.command(
                "snapshot.create",
                {
                    "session_id": _session_id(arguments.session_id),
                    "paths": arguments.paths,
                    "baseline_epoch": arguments.baseline_epoch,
                    "purpose": arguments.purpose,
                },
            )
        if arguments.snapshot_command == "preview":
            return client.command("snapshot.preview", {"snapshot_id": arguments.snapshot_id})
    if arguments.command == "patch":
        if arguments.patch_command == "enqueue":
            patch_text = Path(arguments.file).read_text(encoding="utf-8")
            return client.command(
                "patch.enqueue",
                {
                    "session_id": _session_id(arguments.session_id),
                    "patch_text": patch_text,
                    "targets": arguments.target,
                },
            )
        if arguments.patch_command == "status":
            return client.command("patch.status", {"patch_id": arguments.patch_id})
        if arguments.patch_command == "list":
            return client.command("patch.list", {"status": arguments.status})
        if arguments.patch_command == "process":
            return client.command("patch.process")
    if arguments.command == "watch" and arguments.watch_command == "scan":
        return client.command("watch.scan")
    if arguments.command == "plan":
        if arguments.plan_command == "audit":
            return client.command("plan.audit")
        if arguments.plan_command == "owner":
            return client.command("plan.owner", {"plan_path": arguments.plan_path})
        if arguments.plan_command == "authorize":
            return client.command(
                "plan.authorize",
                {
                    "session_id": _session_id(arguments.session_id),
                    "target_path": arguments.target_path,
                    "maintenance": arguments.maintenance,
                },
            )
    if arguments.command == "failure":
        if arguments.failure_command in {"import", "audit"}:
            return client.command(f"failure.{arguments.failure_command}")
        if arguments.failure_command == "open":
            return client.command("failure.open", {"fixing_plan": arguments.fixing_plan})
        if arguments.failure_command == "return":
            return client.command(
                "failure.return",
                {
                    "lifecycle_key": arguments.lifecycle_key,
                    "session_id": _session_id(arguments.session_id),
                    "resolved_at": arguments.resolved_at,
                    "root_cause": arguments.root_cause,
                    "architecture_fix": arguments.architecture_fix,
                    "validation": arguments.validation,
                    "return_summary": arguments.return_summary,
                },
            )
    if arguments.command == "cargo":
        if arguments.cargo_command == "reserve-cpu":
            command = list(arguments.command_args)
            if command and command[0] == "--":
                command = command[1:]
            return client.command(
                "cargo.reserve_cpu",
                {
                    "session_id": _session_id(arguments.session_id),
                    "compatibility": json.loads(arguments.compatibility_json),
                    "target_dir": arguments.target_dir,
                    "ttl_seconds": arguments.ttl_seconds,
                    "burst_eligible": arguments.burst_eligible,
                    "command": command,
                },
            )
        if arguments.cargo_command == "reserve-gpu":
            command = list(arguments.command_args)
            if command and command[0] == "--":
                command = command[1:]
            return client.command(
                "cargo.reserve_gpu",
                {
                    "session_id": _session_id(arguments.session_id),
                    "compatibility": json.loads(arguments.compatibility_json),
                    "target_dir": arguments.target_dir,
                    "ttl_seconds": arguments.ttl_seconds,
                    "command": command,
                },
            )
        if arguments.cargo_command == "release-cpu-reservation":
            return client.command(
                "cargo.release_cpu_reservation",
                {
                    "reservation_id": arguments.reservation_id,
                    "session_id": _session_id(arguments.session_id),
                },
            )
        if arguments.cargo_command == "renew-cpu-reservation":
            return client.command(
                "cargo.renew_cpu_reservation",
                {
                    "reservation_id": arguments.reservation_id,
                    "session_id": _session_id(arguments.session_id),
                    "ttl_seconds": arguments.ttl_seconds,
                },
            )
        if arguments.cargo_command == "consume-cpu-reservation":
            return client.command(
                "cargo.consume_cpu_reservation",
                {
                    "reservation_id": arguments.reservation_id,
                    "session_id": _session_id(arguments.session_id),
                    "lane_kind": arguments.lane_kind,
                },
            )
        if arguments.cargo_command == "consume-gpu-reservation":
            return client.command(
                "cargo.consume_gpu_reservation",
                {
                    "reservation_id": arguments.reservation_id,
                    "session_id": _session_id(arguments.session_id),
                },
            )
        if arguments.cargo_command == "recover-reservation":
            return client.command(
                "cargo.recover_expired_reservation",
                {
                    "reservation_id": arguments.reservation_id,
                    "job_id": arguments.job_id,
                    "session_id": _session_id(arguments.session_id),
                },
            )
        if arguments.cargo_command == "acquire":
            return client.command(
                "cargo.acquire",
                {
                    "session_id": _session_id(arguments.session_id),
                    "lane_kind": arguments.lane_kind,
                    "target_dir": arguments.target_dir,
                    "dry_run": arguments.dry_run,
                    "pid": arguments.pid,
                    "ephemeral": arguments.ephemeral,
                    "compatibility": (
                        json.loads(arguments.compatibility_json)
                        if arguments.compatibility_json
                        else None
                    ),
                },
            )
        if arguments.cargo_command == "start":
            return client.command(
                "cargo.start",
                {
                    "job_id": arguments.job_id,
                    "pid": arguments.pid,
                    "session_id": _session_id(arguments.session_id),
                    "command": arguments.command_args,
                    "root_is_supervisor": arguments.supervisor,
                },
            )
        if arguments.cargo_command == "run":
            command = list(arguments.command_args)
            if command and command[0] == "--":
                command = command[1:]
            environment: dict[str, str] = {}
            for value in arguments.env:
                key, separator, setting = value.partition("=")
                if not separator or not key or not setting:
                    raise CoordinatorError(
                        "cargo_run_environment_invalid",
                        "Cargo --env values must use NAME=VALUE",
                    )
                environment[key] = setting
            return client.command(
                "cargo.run",
                {
                    "job_id": arguments.job_id,
                    "session_id": _session_id(arguments.session_id),
                    "command": command,
                    "environment": environment,
                },
            )
        if arguments.cargo_command == "run-reserved":
            command = list(arguments.command_args)
            if command and command[0] == "--":
                command = command[1:]
            return client.command(
                "cargo.run_reserved",
                {
                    "reservation_id": arguments.reservation_id,
                    "job_id": arguments.job_id,
                    "session_id": _session_id(arguments.session_id),
                    "command": command,
                },
            )
        if arguments.cargo_command == "run-status":
            return client.command(
                "cargo.run_status",
                {"job_id": arguments.job_id, "session_id": _session_id(arguments.session_id)},
            )
        if arguments.cargo_command == "heartbeat":
            return client.command(
                "cargo.heartbeat",
                {
                    "job_id": arguments.job_id,
                    "session_id": _session_id(arguments.session_id),
                },
            )
        if arguments.cargo_command == "finish":
            return client.command(
                "cargo.finish",
                {
                    "job_id": arguments.job_id,
                    "session_id": _session_id(arguments.session_id),
                    "exit_code": arguments.exit_code,
                },
            )
        if arguments.cargo_command == "release":
            return client.command(
                "cargo.release",
                {
                    "job_id": arguments.job_id,
                    "session_id": _session_id(arguments.session_id),
                },
            )
        if arguments.cargo_command == "list":
            return client.command("cargo.list")
    if arguments.command == "milestone":
        if arguments.milestone_command == "prepare":
            action = client.execute_control_action(
                "topology.refresh",
                {"sessionId": arguments.session_id},
                reason=f"prepare milestone {arguments.milestone.strip().upper()}",
            )
        elif arguments.milestone_command == "validate":
            action = client.execute_control_action(
                "validation.start",
                {
                    "sessionId": arguments.session_id,
                    "runId": arguments.run_id,
                    "milestoneId": arguments.milestone.strip().upper(),
                    "template": arguments.template,
                },
                reason=f"start managed validation for {arguments.milestone.strip().upper()}",
            )
        elif arguments.milestone_command == "review":
            action = client.execute_control_action(
                "topology.refresh",
                {
                    "sessionId": arguments.session_id,
                    "executorSessionId": arguments.executor_session_id,
                    "runId": arguments.run_id,
                    "milestoneId": arguments.milestone.strip().upper(),
                    "criticalCount": arguments.critical_count,
                    "importantCount": arguments.important_count,
                    "summary": arguments.summary,
                },
                reason=f"submit independent review for {arguments.milestone.strip().upper()}",
            )
        elif arguments.milestone_command == "commit":
            action = client.execute_control_action(
                "milestone.commit",
                {
                    "sessionId": arguments.session_id,
                    "runId": arguments.run_id,
                    "milestoneId": arguments.milestone.strip().upper(),
                    "summary": arguments.summary,
                },
                reason=(
                    f"commit {arguments.milestone.strip().upper()} with context: "
                    f"{arguments.summary}"
                ),
            )
        else:
            action = client.execute_control_action(
                "session.complete",
                {"sessionId": arguments.session_id, "runId": arguments.run_id},
                reason="close accepted workflow goal",
            )
        result = action.get("result")
        if not isinstance(result, dict):
            raise CoordinatorClientError(
                "invalid_response", "Coordinator action completed without a result payload"
            )
        return result
    if arguments.command == "cleanup":
        payload = {"older_than_hours": arguments.older_than_hours}
        if arguments.cleanup_command == "apply":
            payload.update(
                {
                    "plan_id": arguments.plan_id,
                    "maintenance_capability": os.environ.get(
                        "ZIRCON_COORDINATOR_MAINTENANCE_TOKEN"
                    ),
                }
            )
        return client.command(f"cleanup.{arguments.cleanup_command}", payload)
    if arguments.command == "artifact":
        return client.command(f"artifact.{arguments.artifact_command}")
    if arguments.command == "finalize":
        direct = arguments.finalize_command is None
        if direct and not arguments.finalize_commit:
            raise CoordinatorError(
                "finalize_arguments_missing",
                "Use finalize preview, finalize commit, or finalize --commit with finalize arguments",
            )
        session_id = arguments.direct_session_id if direct else arguments.session_id
        message = arguments.direct_message if direct else arguments.message
        paths = arguments.direct_paths if direct else arguments.path
        validation_commands = (
            arguments.direct_validation_commands
            if direct
            else arguments.validation_command
        )
        maintenance = arguments.direct_maintenance if direct else arguments.maintenance
        milestone = arguments.direct_milestone if direct else arguments.milestone
        if milestone and not (
            (direct and arguments.finalize_commit)
            or (not direct and arguments.finalize_command == "commit")
            or arguments.finalize_commit
        ):
            raise CoordinatorError(
                "milestone_commit_required",
                "Milestone mode is mutating and requires finalize commit or finalize --commit",
            )
        if not message or not paths:
            raise CoordinatorError(
                "finalize_arguments_missing", "Finalize requires --message and at least one --path"
            )
        command = (
            "finalize.milestone"
            if milestone
            else "finalize.preview"
            if not direct and arguments.finalize_command == "preview" and not arguments.finalize_commit
            else "finalize.commit"
        )
        return client.command(
            command,
            {
                "session_id": _session_id(session_id),
                "message": message,
                "paths": paths,
                "validation_commands": [
                    _split_command(command)
                    for command in validation_commands
                ],
                "maintenance": maintenance,
                "maintenance_capability": (
                    os.environ.get("ZIRCON_COORDINATOR_MAINTENANCE_TOKEN")
                    if maintenance
                    else None
                ),
            },
        )
    if arguments.command == "validation-copy":
        if arguments.validation_copy_command == "cleanup":
            return client.command(
                "validation_copy.cleanup",
                {
                    "job_root": arguments.job_root,
                    "session_id": _session_id(arguments.session_id),
                },
            )
        if arguments.validation_copy_command == "run":
            command = list(arguments.command_args)
            if command and command[0] == "--":
                command = command[1:]
            return client.command(
                "validation_copy.run",
                {
                    "session_id": _session_id(arguments.session_id),
                    "job_id": arguments.job_id,
                    "command": command,
                },
            )
        if arguments.validation_copy_command == "status":
            return client.command(
                "validation_copy.status",
                {
                    "session_id": _session_id(arguments.session_id),
                    "job_id": arguments.job_id,
                },
            )
        return client.command(
            f"validation_copy.{arguments.validation_copy_command}",
            {
                "session_id": _session_id(arguments.session_id),
                "paths": arguments.path,
            },
        )
    if arguments.command == "legacy":
        if arguments.legacy_command == "report":
            return client.command("legacy.report")
        apply = bool(arguments.apply and not arguments.dry_run)
        return client.command(
            f"legacy.{arguments.legacy_command}",
            {
                "apply": apply,
                "maintenance_capability": os.environ.get(
                    "ZIRCON_COORDINATOR_MAINTENANCE_TOKEN"
                )
                if apply
                else None,
            },
        )
    if arguments.command == "retention":
        if arguments.retention_command == "plan":
            return client.command("retention.plan")
        if arguments.dry_run:
            return client.command(
                "retention.show", {"plan_id": arguments.plan_id}
            )
        return client.command(
            "retention.apply",
            {
                "plan_id": arguments.plan_id,
                "maintenance_capability": os.environ.get(
                    "ZIRCON_COORDINATOR_MAINTENANCE_TOKEN"
                ),
            },
        )
    if arguments.command == "maintenance" and arguments.maintenance_command == "tick":
        return client.command(
            "maintenance.tick",
            {
                "apply_cleanup": arguments.apply_cleanup,
                "apply_retention": arguments.apply_retention,
                "apply_legacy_archive": arguments.apply_legacy_archive,
                "apply_lifecycle": arguments.apply_lifecycle,
                "maintenance_capability": os.environ.get(
                    "ZIRCON_COORDINATOR_MAINTENANCE_TOKEN"
                )
                if (
                    arguments.apply_cleanup
                    or arguments.apply_retention
                    or arguments.apply_legacy_archive
                    or arguments.apply_lifecycle
                )
                else None,
            },
        )
    if arguments.command == "audit" and arguments.audit_command == "all":
        return client.command("audit.all")
    raise CoordinatorClientError("invalid_command", f"Unsupported command {arguments.command}")


def main(argv: list[str] | None = None) -> int:
    arguments = _parser().parse_args(argv)
    try:
        result = _run(arguments)
    except (CoordinatorClientError, CoordinatorError, OSError, ValueError) as error:
        if (
            isinstance(error, CoordinatorClientError)
            and error.code == "offline"
            and error.details.get("transport") in {"descriptor_absent", "connection_refused"}
        ):
            intent = _offline_queue_intent(arguments)
            if intent is not None:
                try:
                    queued = _offline_spool(_config(arguments)).enqueue(*intent)
                except ValueError as queue_error:
                    issue = {
                        "code": "offline_queue_rejected",
                        "message": str(queue_error),
                        "details": {},
                    }
                    payload = {"status": "error", "error": issue}
                    print(json.dumps(payload, ensure_ascii=False) if arguments.json_output else issue["message"])
                    return 2
                result = {
                    "status": "queued",
                    "queueId": queued.queue_id,
                    "command": queued.command,
                }
                print(
                    json.dumps(result, ensure_ascii=False, sort_keys=True)
                    if arguments.json_output
                    else json.dumps(result, ensure_ascii=False, indent=2, sort_keys=True)
                )
                return 0
        if hasattr(error, "to_dict"):
            issue = error.to_dict()
        else:
            issue = {"code": "invalid_request", "message": str(error), "details": {}}
        payload = {"status": "offline" if issue["code"] == "offline" else "error", "error": issue}
        print(json.dumps(payload, ensure_ascii=False) if arguments.json_output else issue["message"])
        return 3 if issue["code"] == "offline" else 2
    _write_report(getattr(arguments, "report", None), result)
    print(
        json.dumps(result, ensure_ascii=False, sort_keys=True)
        if arguments.json_output
        else json.dumps(result, ensure_ascii=False, indent=2, sort_keys=True)
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
