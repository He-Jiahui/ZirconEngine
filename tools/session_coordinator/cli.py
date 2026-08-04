from __future__ import annotations

import argparse
import base64
import ctypes
import json
import os
import shlex
import signal
import sqlite3
import subprocess
import sys
import time
import uuid
import webbrowser
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from .client import CoordinatorClient, CoordinatorClientError
from .config import CoordinatorConfig
from .models import CoordinatorError
from .offline_queue import OfflineCommandSpool
from .processes import process_creation_time
from .server import (
    bootstrap_proof_bound_handoff as prepare_proof_bound_handoff,
    run_forever,
    validate_proof_bound_handoff,
)


class _CoordinatorArgumentParser(argparse.ArgumentParser):
    def error(self, message: str) -> None:
        raise CoordinatorError("cli_arguments_invalid", message)


def _reject_nonstandard_json_constant(token: str) -> None:
    raise ValueError(f"Non-standard JSON constant: {token}")


def _strict_json_loads(value: str) -> Any:
    return json.loads(value, parse_constant=_reject_nonstandard_json_constant)


def _parser() -> argparse.ArgumentParser:
    parser = _CoordinatorArgumentParser(prog="zircon-session")
    parser.add_argument("--repo-root", default=str(Path.cwd()))
    parser.add_argument("--state-root")
    parser.add_argument(
        "--port",
        type=int,
        help="Override the local listener port; use 0 only for isolated test coordinators.",
    )
    parser.add_argument("--json", action="store_true", dest="json_output")
    commands = parser.add_subparsers(dest="command", required=True)
    serve = commands.add_parser("serve")
    serve.add_argument("--automatic-start", action="store_true")
    commands.add_parser("status")
    request_status = commands.add_parser("request-status")
    request_status.add_argument("request_id")
    commands.add_parser("stop")
    bootstrap_handoff = commands.add_parser("bootstrap-handoff")
    bootstrap_handoff.add_argument("--reservation-id", required=True)
    bootstrap_handoff.add_argument("--maintenance-session-id", action="append", required=True)
    bootstrap_handoff.add_argument("--actor", default="local-bootstrap")

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
    register.add_argument("--session-role", choices=("primary", "reviewer"))
    register.add_argument("--parent-session-id")

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

    ownership = commands.add_parser("ownership")
    ownership_commands = ownership.add_subparsers(dest="ownership_command", required=True)
    ownership_matrix = ownership_commands.add_parser("matrix")
    ownership_matrix.add_argument("--prefix")
    ownership_transfer_preview = ownership_commands.add_parser("transfer-preview")
    ownership_transfer_preview.add_argument("--target-session-id", required=True)
    ownership_transfer_preview.add_argument("paths", nargs="+")
    ownership_transfer_apply = ownership_commands.add_parser("transfer-apply")
    ownership_transfer_apply.add_argument("--fingerprint", required=True)
    ownership_transfer_apply.add_argument("--confirm-fingerprint", required=True)
    ownership_transfer_apply.add_argument("--actor")
    ownership_transfer_apply.add_argument("--maintenance-capability")

    ai_effort = commands.add_parser("ai-effort")
    ai_effort_commands = ai_effort.add_subparsers(dest="ai_effort_command", required=True)
    ai_effort_commands.add_parser("report")
    ai_effort_record = ai_effort_commands.add_parser("record")
    ai_effort_record.add_argument("--ledger-id", required=True)
    ai_effort_record.add_argument("--plan-id", required=True)
    ai_effort_record.add_argument("--active-ai-hours", type=float, required=True)
    ai_effort_record.add_argument(
        "--outcome", choices=("accepted", "failed", "superseded"), required=True
    )
    ai_effort_record.add_argument(
        "--cost-class", choices=("delivery_design", "repair_validation"), required=True
    )
    ai_effort_record.add_argument("--blocked-by", action="append", default=[])
    ai_effort_record.add_argument("--source-session-id")

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
    failure_materialize = failure_commands.add_parser("materialize-local-validation")
    failure_materialize.add_argument("--session-id", required=True)
    failure_materialize.add_argument("--summary-slug", required=True)
    failure_materialize.add_argument("--source-slice", required=True)
    failure_materialize.add_argument("--reproduction", required=True)
    failure_materialize.add_argument("--lowest-known-cause", required=True)
    failure_materialize.add_argument("--acceptance-criterion", action="append", default=[])
    failure_materialize.add_argument("--related-code", action="append", default=[])
    failure_materialize.add_argument("--created-at")
    failure_return = failure_commands.add_parser("return")
    failure_return.add_argument("lifecycle_key")
    failure_return.add_argument("--session-id")
    failure_return.add_argument("--resolved-at", required=True)
    failure_return.add_argument("--root-cause", required=True)
    failure_return.add_argument("--architecture-fix", required=True)
    failure_return.add_argument("--validation", required=True)
    failure_return.add_argument("--return-summary", required=True)
    closeout_prepare = failure_commands.add_parser("closeout-prepare")
    closeout_prepare.add_argument("lifecycle_key")
    closeout_prepare.add_argument("--session-id", required=True)
    closeout_prepare.add_argument("--snapshot-id", type=int, required=True)
    closeout_prepare.add_argument("--validation-command-json", required=True)
    closeout_prepare.add_argument("--job-id", required=True)
    closeout_prepare.add_argument("--cargo-run-id", required=True)
    closeout_combined = failure_commands.add_parser("closeout-prepare-combined")
    closeout_combined.add_argument("lifecycle_keys", nargs="+")
    closeout_combined.add_argument("--delivery-record", action="append", default=[])
    closeout_combined.add_argument("--session-id", required=True)
    closeout_combined.add_argument("--snapshot-id", type=int, required=True)
    closeout_combined.add_argument("--validation-command-json", required=True)
    closeout_combined.add_argument("--job-id", required=True)
    closeout_combined.add_argument("--cargo-run-id", required=True)
    closeout_validate = failure_commands.add_parser("closeout-validate")
    closeout_validate.add_argument("closeout_id")
    closeout_validate.add_argument("--session-id", required=True)
    closeout_validate.add_argument("--job-id", required=True)
    closeout_validate.add_argument("--cargo-run-id", required=True)
    closeout_review = failure_commands.add_parser("closeout-review")
    closeout_review.add_argument("closeout_id")
    closeout_review.add_argument("--executor-session-id", required=True)
    closeout_review.add_argument("--critical-count", type=int, required=True)
    closeout_review.add_argument("--important-count", type=int, required=True)
    closeout_review.add_argument("--moderate-count", type=int, required=True)
    closeout_review.add_argument("--summary", required=True)
    closeout_commit = failure_commands.add_parser("closeout-commit")
    closeout_commit.add_argument("closeout_id")
    closeout_commit.add_argument("--session-id", required=True)
    closeout_commit.add_argument("--summary", required=True)

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
    cargo_reserve_cpu.add_argument("--dependency-lifecycle-key")
    cargo_reserve_cpu.add_argument("--dependency-fixed-sha256")
    burst_choice = cargo_reserve_cpu.add_mutually_exclusive_group()
    burst_choice.add_argument("--burst-eligible", action="store_const", const=True, dest="burst_eligible")
    burst_choice.add_argument("--no-burst", action="store_const", const=False, dest="burst_eligible")
    cargo_reserve_cpu.set_defaults(burst_eligible=None)
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
        "--template",
        choices=("coordinator-actions", "web-check", "runtime14-rust-focused"),
        required=True,
    )
    milestone_commit = milestone_commands.add_parser("commit")
    milestone_commit.add_argument("--session-id", required=True)
    milestone_commit.add_argument("--run-id", required=True)
    milestone_commit.add_argument("--milestone", required=True)
    milestone_commit.add_argument("--summary", required=True)
    milestone_defer = milestone_commands.add_parser("defer-failure")
    milestone_defer.add_argument("--session-id", required=True)
    milestone_defer.add_argument("--source-milestone", required=True)
    milestone_defer.add_argument("--target-milestone", required=True)
    milestone_defer.add_argument("--failure-lifecycle-key", required=True)
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
        copy_parser.add_argument("--external-source-json", action="append", default=[])
    copy_cargo = validation_copy_commands.add_parser("materialize-cargo")
    copy_cargo.add_argument("--session-id")
    copy_cargo.add_argument("--path", action="append", default=[])
    copy_cargo.add_argument("--external-source-json", action="append", default=[])
    copy_cargo.add_argument("command_args", nargs=argparse.REMAINDER)
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

    governance = commands.add_parser("governance")
    governance_commands = governance.add_subparsers(
        dest="governance_command", required=True
    )
    governance_preview = governance_commands.add_parser("converge-preview")
    governance_preview.add_argument("--actor", default="local-cli")
    governance_apply = governance_commands.add_parser("converge-apply")
    governance_apply.add_argument("--fingerprint", required=True)
    governance_apply.add_argument("--actor", default="local-cli")
    governance_retention_preview = governance_commands.add_parser("retention-preview")
    governance_retention_preview.add_argument("--actor", default="local-cli")
    governance_retention_apply = governance_commands.add_parser("retention-apply")
    governance_retention_apply.add_argument("--fingerprint", required=True)
    governance_retention_apply.add_argument("--actor", default="local-cli")
    governance_retention_compact = governance_commands.add_parser("retention-compact")
    governance_retention_compact.add_argument("--batch-id", required=True)
    governance_retention_compact.add_argument("--actor", default="local-cli")

    validation = commands.add_parser("validation")
    validation_commands = validation.add_subparsers(dest="validation_command", required=True)
    validation_submit = validation_commands.add_parser("submit")
    validation_submit.add_argument("--session-id", required=True)
    validation_submit.add_argument("--request-id", required=True)
    validation_manifest = validation_submit.add_mutually_exclusive_group(required=True)
    validation_manifest.add_argument("--source-manifest-json")
    validation_manifest.add_argument("--source-manifest-stdin", action="store_true")
    validation_submit.add_argument("--command-json", required=True)
    validation_submit.add_argument("--toolchain-json", required=True)
    validation_submit.add_argument("--coverage-json", required=True)
    validation_status = validation_commands.add_parser("status")
    validation_status.add_argument("--ticket-id", required=True)
    validation_result = validation_commands.add_parser("record-result")
    validation_result.add_argument("--ticket-id", required=True)
    validation_result.add_argument(
        "--status", required=True, choices=("passed", "failed", "snapshot_stale")
    )
    validation_result.add_argument("--evidence-json", default="{}")
    validation_result.add_argument("--failure-json")

    integration = commands.add_parser("integration")
    integration_commands = integration.add_subparsers(dest="integration_command", required=True)
    integration_submit = integration_commands.add_parser("submit")
    integration_submit.add_argument("--session-id", required=True)
    integration_submit.add_argument("--request-id", required=True)
    integration_submit.add_argument("--compile-ticket-id", required=True)
    integration_submit.add_argument("--path", action="append", required=True)
    integration_status = integration_commands.add_parser("status")
    integration_status.add_argument("--candidate-id", required=True)
    integration_finalize = integration_commands.add_parser("finalize")
    integration_finalize.add_argument("--candidate-id", required=True)
    integration_finalize.add_argument("--message", required=True)

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


def _compatibility_from_argument(value: str) -> dict[str, object]:
    encoded_prefix = "base64:"
    raw_value = value
    if value.startswith(encoded_prefix):
        try:
            raw_value = base64.b64decode(
                value[len(encoded_prefix) :], validate=True
            ).decode("utf-8")
        except (UnicodeDecodeError, ValueError) as error:
            raise CoordinatorError(
                "cli_arguments_invalid",
                "Cargo compatibility JSON base64 payload is invalid",
            ) from error
    try:
        compatibility = json.loads(raw_value)
    except json.JSONDecodeError as error:
        raise CoordinatorError(
            "cli_arguments_invalid",
            "Cargo compatibility JSON payload is invalid",
        ) from error
    if not isinstance(compatibility, dict):
        raise CoordinatorError(
            "cli_arguments_invalid",
            "Cargo compatibility JSON payload must be an object",
        )
    return compatibility


def _split_command(value: str) -> list[str]:
    parts = shlex.split(value, posix=False)
    return [
        part[1:-1]
        if len(part) >= 2 and part[0] == part[-1] and part[0] in {'"', "'"}
        else part
        for part in parts
    ]


def _config(arguments: argparse.Namespace) -> CoordinatorConfig:
    options: dict[str, Any] = {"state_root": arguments.state_root}
    if arguments.port is not None:
        options["port"] = arguments.port
    return CoordinatorConfig.for_repo(arguments.repo_root, **options)


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
                    "session_role": arguments.session_role,
                    "parent_session_id": arguments.parent_session_id,
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


def _runtime_descriptor(config: CoordinatorConfig) -> dict[str, object]:
    try:
        payload = json.loads(config.runtime_path.read_text(encoding="utf-8"))
        instance_id = payload["instance_id"]
        pid = payload["pid"]
        creation_time = payload["process_creation_time"]
        executable = payload["executable"]
        command_line = payload["command_line"]
    except (OSError, KeyError, TypeError, ValueError, json.JSONDecodeError) as error:
        raise CoordinatorError(
            "bootstrap_runtime_descriptor_invalid",
            "Proof-bound bootstrap requires the current runtime descriptor",
        ) from error
    if (
        not isinstance(instance_id, str)
        or not instance_id
        or not isinstance(pid, int)
        or pid <= 0
        or not isinstance(creation_time, str)
        or not creation_time
        or not isinstance(executable, str)
        or not executable
        or not isinstance(command_line, list)
        or not command_line
        or any(not isinstance(part, str) or not part for part in command_line)
    ):
        raise CoordinatorError(
            "bootstrap_runtime_descriptor_invalid",
            "Proof-bound bootstrap runtime descriptor has an invalid process identity",
        )
    return payload


@dataclass(frozen=True)
class _PredecessorHandle:
    runtime: dict[str, object]
    kernel32: Any | None = None
    native_handle: int | None = None


def _windows_kernel32():
    return ctypes.WinDLL("kernel32", use_last_error=True)


def _capture_predecessor_handle(runtime: dict[str, object]) -> _PredecessorHandle:
    """Capture the exact predecessor before proof/hold commits.

    Retaining the Windows process handle prevents PID reuse between the
    identity check, shutdown request, and exit wait.
    """
    pid = int(runtime["pid"])
    expected_creation = str(runtime["process_creation_time"])
    if os.name == "nt":
        synchronize = 0x00100000
        query_limited_information = 0x1000
        kernel32 = _windows_kernel32()
        kernel32.OpenProcess.argtypes = [ctypes.c_uint32, ctypes.c_bool, ctypes.c_uint32]
        kernel32.OpenProcess.restype = ctypes.c_void_p
        kernel32.CloseHandle.argtypes = [ctypes.c_void_p]
        kernel32.CloseHandle.restype = ctypes.c_bool
        native_handle = kernel32.OpenProcess(
            synchronize | query_limited_information, False, pid
        )
        if not native_handle:
            raise CoordinatorError(
                "bootstrap_predecessor_identity_unavailable",
                "Cannot capture the predecessor process before proof-bound shutdown",
                details={"pid": pid, "win32Error": ctypes.get_last_error()},
            )
        handle = _PredecessorHandle(runtime, kernel32, int(native_handle))
    else:
        try:
            os.kill(pid, 0)
        except ProcessLookupError as error:
            raise CoordinatorError(
                "bootstrap_predecessor_exited_before_hold",
                "The predecessor exited before proof-bound shutdown",
                details={"pid": pid},
            ) from error
        except PermissionError as error:
            raise CoordinatorError(
                "bootstrap_predecessor_identity_unavailable",
                "Cannot capture the predecessor process before proof-bound shutdown",
                details={"pid": pid},
            ) from error
        handle = _PredecessorHandle(runtime)
    try:
        actual_creation = process_creation_time(pid)
    except OSError as error:
        _close_predecessor_handle(handle)
        raise CoordinatorError(
            "bootstrap_predecessor_identity_unavailable",
            "Cannot verify the captured predecessor process identity",
        ) from error
    if actual_creation != expected_creation:
        _close_predecessor_handle(handle)
        raise CoordinatorError(
            "bootstrap_predecessor_changed",
            "The runtime descriptor PID changed before proof-bound shutdown",
        )
    return handle


def _close_predecessor_handle(handle: _PredecessorHandle) -> None:
    if handle.kernel32 is not None and handle.native_handle is not None:
        handle.kernel32.CloseHandle(handle.native_handle)


def _predecessor_handle_exited(handle: _PredecessorHandle) -> bool:
    if handle.kernel32 is None or handle.native_handle is None:
        pid = int(handle.runtime["pid"])
        try:
            os.kill(pid, 0)
        except ProcessLookupError:
            return True
        except PermissionError as error:
            raise CoordinatorError(
                "bootstrap_predecessor_identity_unavailable",
                "Cannot prove whether the predecessor process exited",
                details={"pid": pid},
            ) from error
        return False
    wait_object_0 = 0
    wait_timeout = 258
    handle.kernel32.WaitForSingleObject.argtypes = [ctypes.c_void_p, ctypes.c_uint32]
    handle.kernel32.WaitForSingleObject.restype = ctypes.c_uint32
    wait_result = int(handle.kernel32.WaitForSingleObject(handle.native_handle, 0))
    if wait_result == wait_object_0:
        return True
    if wait_result == wait_timeout:
        return False
    raise CoordinatorError(
        "bootstrap_predecessor_wait_failed",
        "Cannot wait for the captured predecessor process to exit",
        details={"pid": handle.runtime["pid"], "waitResult": wait_result},
    )


def _shutdown_predecessor(handle: _PredecessorHandle) -> None:
    if _predecessor_handle_exited(handle):
        raise CoordinatorError(
            "bootstrap_predecessor_exited_before_shutdown",
            "The predecessor exited before its controlled shutdown request",
            details={"pid": handle.runtime["pid"]},
        )
    pid = int(handle.runtime["pid"])
    # Do not terminate descendants: a surprise real Cargo tree must survive for
    # audited reconciliation and will make the post-handoff audit not-ready.
    if os.name == "nt":
        subprocess.run(
            ["taskkill.exe", "/PID", str(pid), "/F"],
            check=True,
            capture_output=True,
            text=True,
            timeout=15,
        )
    else:
        os.kill(pid, signal.SIGTERM)


def _wait_for_predecessor_exit(
    handle: _PredecessorHandle, *, timeout_seconds: float = 15.0
) -> None:
    deadline = time.monotonic() + timeout_seconds
    while time.monotonic() < deadline:
        if _predecessor_handle_exited(handle):
            return
        time.sleep(0.05)
    raise CoordinatorError(
        "bootstrap_predecessor_exit_timeout",
        "The predecessor did not exit after the proof-bound shutdown request",
    )


def _start_successor(runtime: dict[str, object]) -> int:
    creationflags = getattr(subprocess, "CREATE_NO_WINDOW", 0) if os.name == "nt" else 0
    process = subprocess.Popen(
        [str(runtime["executable"]), *(str(part) for part in runtime["command_line"])],
        stdin=subprocess.DEVNULL,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        creationflags=creationflags,
    )
    return int(process.pid)


def _wait_for_successor(
    config: CoordinatorConfig,
    *,
    predecessor_instance_id: str,
    predecessor_pid: int,
    timeout_seconds: float = 30.0,
) -> dict[str, object]:
    deadline = time.monotonic() + timeout_seconds
    while time.monotonic() < deadline:
        try:
            runtime = _runtime_descriptor(config)
        except CoordinatorError:
            time.sleep(0.05)
            continue
        if runtime["instance_id"] == predecessor_instance_id or int(runtime["pid"]) == predecessor_pid:
            time.sleep(0.05)
            continue
        try:
            health = CoordinatorClient.from_runtime(config).health()
        except CoordinatorClientError:
            time.sleep(0.05)
            continue
        supervision = health.get("supervision")
        if isinstance(supervision, dict) and bool(supervision.get("maintenanceHold")):
            return {"runtime": runtime, "health": health}
        time.sleep(0.05)
    raise CoordinatorError(
        "bootstrap_successor_timeout",
        "The successor did not publish a proof-bound maintenance hold",
    )


def _bootstrap_operational_error_code(error: BaseException) -> str:
    if isinstance(error, CoordinatorError):
        return error.code
    if isinstance(error, subprocess.TimeoutExpired):
        return "bootstrap_process_timeout"
    if isinstance(error, subprocess.CalledProcessError):
        return "bootstrap_process_command_failed"
    if isinstance(error, sqlite3.DatabaseError):
        return "bootstrap_database_error"
    return "bootstrap_process_os_error"


def bootstrap_proof_bound_handoff(
    config: CoordinatorConfig,
    *,
    reservation_id: str,
    maintenance_session_ids: tuple[str, ...],
    actor: str,
) -> dict[str, object]:
    """Execute the no-gap predecessor-stop-to-successor-proof protocol."""
    predecessor = _runtime_descriptor(config)
    predecessor_handle = _capture_predecessor_handle(predecessor)
    prepared: dict[str, object] | None = None
    try:
        prepared = prepare_proof_bound_handoff(
            config,
            reservation_id=reservation_id,
            maintenance_session_ids=maintenance_session_ids,
            actor=actor,
            expected_daemon_instance_id=str(predecessor["instance_id"]),
            expected_process_id=int(predecessor["pid"]),
            expected_process_creation_time=str(predecessor["process_creation_time"]),
        )
        _shutdown_predecessor(predecessor_handle)
        _wait_for_predecessor_exit(predecessor_handle)
        _start_successor(predecessor)
        successor = _wait_for_successor(
            config,
            predecessor_instance_id=str(predecessor["instance_id"]),
            predecessor_pid=int(predecessor["pid"]),
        )
        audit = validate_proof_bound_handoff(
            config,
            action_id=str(prepared["actionId"]),
            reservation_id=reservation_id,
        )
    except (
        CoordinatorError,
        subprocess.SubprocessError,
        sqlite3.DatabaseError,
        OSError,
    ) as error:
        if prepared is None:
            raise
        return {
            **prepared,
            "ready": False,
            "blockers": [
                {
                    "kind": "predecessor_handoff",
                    "code": _bootstrap_operational_error_code(error),
                }
            ],
        }
    finally:
        _close_predecessor_handle(predecessor_handle)
    return {
        **prepared,
        **audit,
        "successorInstanceId": successor["runtime"]["instance_id"],
        "successorSchemaVersion": successor["runtime"].get("schema_version"),
    }


def _run(arguments: argparse.Namespace) -> dict[str, Any]:
    config = _config(arguments)
    if arguments.command == "serve":
        run_forever(config, automatic_start=arguments.automatic_start)
        return {"status": "stopped"}
    if arguments.command == "bootstrap-handoff":
        return bootstrap_proof_bound_handoff(
            config,
            reservation_id=arguments.reservation_id,
            maintenance_session_ids=tuple(arguments.maintenance_session_id),
            actor=arguments.actor,
        )
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
    if arguments.command == "request-status":
        return client.command_request_status(arguments.request_id)
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
                "session_role": arguments.session_role,
                "parent_session_id": arguments.parent_session_id,
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
    if arguments.command == "ownership":
        if arguments.ownership_command == "matrix":
            return client.command("ownership.matrix", {"prefix": arguments.prefix})
        if arguments.ownership_command == "transfer-preview":
            return client.command(
                "ownership.transfer.preview",
                {
                    "target_session_id": _session_id(arguments.target_session_id),
                    "paths": arguments.paths,
                },
            )
        return client.command(
            "ownership.transfer.apply",
            {
                "fingerprint": arguments.fingerprint,
                "confirm_fingerprint": arguments.confirm_fingerprint,
                "actor": arguments.actor,
                "maintenance_capability": arguments.maintenance_capability,
            },
        )
    if arguments.command == "ai-effort":
        if arguments.ai_effort_command == "report":
            return client.command("ai_effort.report", {})
        return client.command(
            "ai_effort.record",
            {
                "ledger_id": arguments.ledger_id,
                "plan_id": arguments.plan_id,
                "active_ai_hours": arguments.active_ai_hours,
                "outcome": arguments.outcome,
                "cost_class": arguments.cost_class,
                "blocked_by": arguments.blocked_by,
                "source_session_id": arguments.source_session_id,
            },
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
        if arguments.failure_command == "materialize-local-validation":
            return client.command(
                "failure.materialize_local_validation",
                {
                    "session_id": arguments.session_id,
                    "summary_slug": arguments.summary_slug,
                    "source_slice": arguments.source_slice,
                    "reproduction": arguments.reproduction,
                    "lowest_known_cause": arguments.lowest_known_cause,
                    "acceptance_criteria": arguments.acceptance_criterion,
                    "related_code": arguments.related_code,
                    "created_at": arguments.created_at,
                },
            )
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
        if arguments.failure_command in {"closeout-prepare", "closeout-prepare-combined"}:
            command = json.loads(arguments.validation_command_json)
            if not isinstance(command, list) or not all(
                isinstance(item, str) for item in command
            ):
                raise CoordinatorError(
                    "failure_closeout_validation_command_invalid",
                    "Failure closeout --validation-command-json must encode a string array",
                )
            executor_thread_id = os.environ.get("CODEX_THREAD_ID")
            if not executor_thread_id:
                raise CoordinatorError(
                    "failure_closeout_executor_thread_missing",
                    "Failure closeout prepare must run from a discoverable Codex task",
                )
            combined = arguments.failure_command == "closeout-prepare-combined"
            payload = {
                    "session_id": arguments.session_id,
                    "snapshot_id": arguments.snapshot_id,
                    "validation_command": command,
                    "validation_job_id": arguments.job_id,
                    "validation_run_id": arguments.cargo_run_id,
                    "executor_thread_id": executor_thread_id,
                    "actor": arguments.session_id,
                }
            if combined:
                payload.update(
                    {
                        "lifecycle_keys": arguments.lifecycle_keys,
                        "delivery_records": arguments.delivery_record,
                    }
                )
            else:
                payload["lifecycle_key"] = arguments.lifecycle_key
            return client.command(
                (
                    "failure.closeout_prepare_combined"
                    if combined
                    else "failure.closeout_prepare"
                ),
                payload,
            )
        if arguments.failure_command == "closeout-validate":
            return client.command(
                "failure.closeout_validate",
                {
                    "session_id": arguments.session_id,
                    "closeout_id": arguments.closeout_id,
                    "job_id": arguments.job_id,
                    "cargo_run_id": arguments.cargo_run_id,
                    "actor": arguments.session_id,
                },
            )
        if arguments.failure_command == "closeout-review":
            reviewer_thread_id = os.environ.get("CODEX_THREAD_ID")
            if not reviewer_thread_id:
                raise CoordinatorError(
                    "failure_closeout_reviewer_thread_missing",
                    "Failure closeout review must run from a discoverable Codex task",
                )
            return client.command(
                "failure.closeout_review",
                {
                    "session_id": reviewer_thread_id,
                    "reviewer_thread_id": reviewer_thread_id,
                    "executor_session_id": arguments.executor_session_id,
                    "closeout_id": arguments.closeout_id,
                    "critical_count": arguments.critical_count,
                    "important_count": arguments.important_count,
                    "moderate_count": arguments.moderate_count,
                    "summary": arguments.summary,
                },
            )
        if arguments.failure_command == "closeout-commit":
            return client.command(
                "failure.closeout_commit",
                {
                    "session_id": arguments.session_id,
                    "closeout_id": arguments.closeout_id,
                    "summary": arguments.summary,
                    "actor": arguments.session_id,
                },
            )
    if arguments.command == "cargo":
        if arguments.cargo_command == "reserve-cpu":
            command = list(arguments.command_args)
            if command and command[0] == "--":
                command = command[1:]
            payload: dict[str, object] = {
                "session_id": _session_id(arguments.session_id),
                "compatibility": _compatibility_from_argument(arguments.compatibility_json),
                "target_dir": arguments.target_dir,
                "ttl_seconds": arguments.ttl_seconds,
                "command": command,
            }
            if arguments.burst_eligible is not None:
                payload["burst_eligible"] = arguments.burst_eligible
            if arguments.dependency_lifecycle_key is not None:
                payload["dependency_lifecycle_key"] = arguments.dependency_lifecycle_key
            if arguments.dependency_fixed_sha256 is not None:
                payload["dependency_fixed_sha256"] = arguments.dependency_fixed_sha256
            return client.command(
                "cargo.reserve_cpu",
                payload,
            )
        if arguments.cargo_command == "reserve-gpu":
            command = list(arguments.command_args)
            if command and command[0] == "--":
                command = command[1:]
            return client.command(
                "cargo.reserve_gpu",
                {
                    "session_id": _session_id(arguments.session_id),
                    "compatibility": _compatibility_from_argument(arguments.compatibility_json),
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
                        _compatibility_from_argument(arguments.compatibility_json)
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
                {
                    "sessionId": arguments.session_id,
                    "milestoneId": arguments.milestone.strip().upper(),
                },
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
        elif arguments.milestone_command == "defer-failure":
            return client.command(
                "milestone.defer_failure",
                {
                    "session_id": arguments.session_id,
                    "source_milestone_key": arguments.source_milestone.strip().upper(),
                    "target_milestone_key": arguments.target_milestone.strip().upper(),
                    "failure_lifecycle_key": arguments.failure_lifecycle_key,
                    "actor": arguments.session_id,
                },
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
        external_sources = [
            json.loads(payload) for payload in arguments.external_source_json
        ]
        if arguments.validation_copy_command == "materialize-cargo":
            command = list(arguments.command_args)
            if command and command[0] == "--":
                command = command[1:]
            return client.command(
                "validation_copy.materialize_cargo",
                {
                    "session_id": _session_id(arguments.session_id),
                    "paths": arguments.path,
                    "external_sources": external_sources,
                    "command": command,
                },
            )
        return client.command(
            f"validation_copy.{arguments.validation_copy_command}",
            {
                "session_id": _session_id(arguments.session_id),
                "paths": arguments.path,
                "external_sources": external_sources,
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
    if arguments.command == "governance":
        if arguments.governance_command == "converge-preview":
            return client.command(
                "governance.converge.preview", {"actor": arguments.actor}
            )
        if arguments.governance_command == "converge-apply":
            return client.command(
                "governance.converge.apply",
                {
                    "fingerprint": arguments.fingerprint,
                    "actor": arguments.actor,
                    "maintenance_capability": os.environ.get(
                        "ZIRCON_COORDINATOR_MAINTENANCE_TOKEN"
                    ),
                },
            )
        if arguments.governance_command == "retention-preview":
            return client.command(
                "governance.retention.preview", {"actor": arguments.actor}
            )
        if arguments.governance_command == "retention-apply":
            return client.command(
                "governance.retention.apply",
                {
                    "fingerprint": arguments.fingerprint,
                    "actor": arguments.actor,
                    "maintenance_capability": os.environ.get(
                        "ZIRCON_COORDINATOR_MAINTENANCE_TOKEN"
                    ),
                },
            )
        return client.command(
            "governance.retention.compact",
            {
                "batch_id": arguments.batch_id,
                "actor": arguments.actor,
                "maintenance_capability": os.environ.get(
                    "ZIRCON_COORDINATOR_MAINTENANCE_TOKEN"
                ),
            },
        )
    if arguments.command == "integration":
        if arguments.integration_command == "status":
            return client.command("integration.status", {"candidate_id": arguments.candidate_id})
        if arguments.integration_command == "finalize":
            return client.command(
                "integration.finalize",
                {"candidate_id": arguments.candidate_id, "message": arguments.message},
            )
        return client.command(
            "integration.submit",
            {
                "session_id": arguments.session_id,
                "request_id": arguments.request_id,
                "compile_ticket_id": arguments.compile_ticket_id,
                "paths": arguments.path,
            },
        )
    if arguments.command == "validation":
        if arguments.validation_command == "status":
            return client.command("validation.status", {"ticket_id": arguments.ticket_id})
        if arguments.validation_command == "record-result":
            try:
                evidence = _strict_json_loads(arguments.evidence_json)
                failure = (
                    _strict_json_loads(arguments.failure_json)
                    if arguments.failure_json is not None
                    else None
                )
            except ValueError as error:
                raise CoordinatorError(
                    "validation_ticket_json_invalid",
                    "Validation result JSON arguments must be valid JSON",
                ) from error
            if not isinstance(evidence, dict) or (
                failure is not None and not isinstance(failure, dict)
            ):
                raise CoordinatorError(
                    "validation_ticket_json_invalid",
                    "Validation result evidence and failure context must be JSON objects",
                )
            if arguments.status == "failed" and failure is None:
                raise CoordinatorError(
                    "validation_ticket_failure_context_missing",
                    "A failed validation result requires --failure-json for the forward repair",
                )
            return client.command(
                "validation.record_result",
                {
                    "ticket_id": arguments.ticket_id,
                    "status": arguments.status,
                    "evidence": evidence,
                    "failure": failure,
                },
            )
        manifest_json = arguments.source_manifest_json
        if arguments.source_manifest_stdin:
            manifest_json = sys.stdin.read().removeprefix("\ufeff")
        try:
            source_manifest = _strict_json_loads(manifest_json)
            command = _strict_json_loads(arguments.command_json)
            toolchain = _strict_json_loads(arguments.toolchain_json)
            coverage = _strict_json_loads(arguments.coverage_json)
        except ValueError as error:
            raise CoordinatorError(
                "validation_ticket_json_invalid",
                "Validation submit JSON arguments must be valid JSON",
            ) from error
        if not isinstance(source_manifest, dict) or not isinstance(command, list):
            raise CoordinatorError(
                "validation_ticket_json_invalid",
                "Validation submit requires an object manifest and string-array command",
            )
        if not isinstance(toolchain, dict) or not isinstance(coverage, dict):
            raise CoordinatorError(
                "validation_ticket_json_invalid",
                "Validation submit toolchain and coverage must be JSON objects",
            )
        return client.command(
            "validation.submit",
            {
                "session_id": arguments.session_id,
                "request_id": arguments.request_id,
                "source_manifest": source_manifest,
                "command": command,
                "toolchain": toolchain,
                "coverage": coverage,
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
    raw_arguments = list(sys.argv[1:] if argv is None else argv)
    json_output = "--json" in raw_arguments
    arguments: argparse.Namespace | None = None
    try:
        arguments = _parser().parse_args(raw_arguments)
        result = _run(arguments)
    except (
        CoordinatorClientError,
        CoordinatorError,
        sqlite3.DatabaseError,
        OSError,
        ValueError,
    ) as error:
        if (
            isinstance(error, CoordinatorClientError)
            and error.code == "offline"
            and error.details.get("transport") in {"descriptor_absent", "connection_refused"}
        ):
            intent = _offline_queue_intent(arguments) if arguments is not None else None
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
                    print(json.dumps(payload, ensure_ascii=False) if json_output else issue["message"])
                    return 2
                result = {
                    "status": "queued",
                    "queueId": queued.queue_id,
                    "command": queued.command,
                }
                print(
                    json.dumps(result, ensure_ascii=False, sort_keys=True)
                    if json_output
                    else json.dumps(result, ensure_ascii=False, indent=2, sort_keys=True)
                )
                return 0
        if isinstance(error, sqlite3.DatabaseError):
            issue = {
                "code": "coordinator_database_error",
                "message": "Coordinator database operation failed",
                "details": {},
            }
        elif hasattr(error, "to_dict"):
            issue = error.to_dict()
        else:
            issue = {"code": "invalid_request", "message": str(error), "details": {}}
        payload = {"status": "offline" if issue["code"] == "offline" else "error", "error": issue}
        preserve_recovery_identity = isinstance(error, CoordinatorClientError) and issue[
            "code"
        ].startswith("command_post_")
        print(
            json.dumps(payload, ensure_ascii=False)
            if json_output or preserve_recovery_identity
            else issue["message"]
        )
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
