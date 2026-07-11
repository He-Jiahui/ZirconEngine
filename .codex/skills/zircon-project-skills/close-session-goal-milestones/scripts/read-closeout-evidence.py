from __future__ import annotations

import argparse
import ctypes
import hashlib
import importlib.util
import json
import os
import re
import sqlite3
import subprocess
import sys
from pathlib import Path
from datetime import datetime, timezone

REPO_ROOT = Path(__file__).resolve().parents[5]
sys.path.insert(0, str(REPO_ROOT))

from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.client import CoordinatorClient, CoordinatorClientError


def hash_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def process_is_alive(process_id: int) -> bool:
    if process_id <= 0:
        return False
    if os.name == "nt":
        process_query_limited_information = 0x1000
        handle = ctypes.windll.kernel32.OpenProcess(  # type: ignore[attr-defined]
            process_query_limited_information, False, process_id
        )
        if not handle:
            return False
        ctypes.windll.kernel32.CloseHandle(handle)  # type: ignore[attr-defined]
        return True
    try:
        os.kill(process_id, 0)
    except OSError:
        return False
    return True


def read_evidence(repo_root: Path, session_id: str) -> dict[str, object]:
    repo = repo_root.resolve()
    config = CoordinatorConfig.for_repo(repo)
    runtime = json.loads(config.runtime_path.read_text(encoding="utf-8"))
    runtime_repo = Path(str(runtime.get("repo_root", ""))).resolve()
    runtime_pid = int(runtime.get("pid", 0))
    if runtime_repo != repo or runtime_pid <= 0:
        raise ValueError("Coordinator runtime descriptor does not match the repository")
    if not process_is_alive(runtime_pid):
        raise ValueError("Coordinator runtime process is not alive")
    try:
        health = CoordinatorClient.from_runtime(config).health()
    except CoordinatorClientError as error:
        raise ValueError(f"Coordinator health query failed: {error.message}") from error
    branch = subprocess.run(
        ["git", "branch", "--show-current"],
        cwd=repo,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    if Path(str(health.get("repo_root", ""))).resolve() != repo:
        raise ValueError("Coordinator health response belongs to another repository")
    if int(health.get("pid", 0)) != runtime_pid:
        raise ValueError("Coordinator health PID does not match its runtime descriptor")
    database_uri = config.database_path.resolve().as_uri() + "?mode=ro"
    connection = sqlite3.connect(database_uri, uri=True)
    connection.row_factory = sqlite3.Row
    try:
        session = connection.execute(
            "SELECT session_id, status, plan_path FROM sessions WHERE session_id = ?",
            (session_id,),
        ).fetchone()
        if session is None:
            raise ValueError(f"Unknown Session {session_id}")
        epoch = connection.execute(
            "SELECT manifest_json FROM baseline_epochs ORDER BY epoch_id DESC LIMIT 1"
        ).fetchone()
        baseline_manifest = json.loads(epoch["manifest_json"]) if epoch else {}
        attributions = connection.execute(
            """
            SELECT display_path, content_hash
            FROM attributions
            WHERE session_id = ?
            ORDER BY display_path COLLATE NOCASE
            """,
            (session_id,),
        ).fetchall()
        lease_rows = connection.execute(
            """
            SELECT display_path, session_id, base_hash, expires_at
            FROM leases
            WHERE session_id = ?
            ORDER BY display_path COLLATE NOCASE
            """,
            (session_id,),
        ).fetchall()
        open_failures = connection.execute(
            """
            SELECT COUNT(*)
            FROM failure_nodes
            WHERE (fixing_plan = ? OR origin_plan = ?)
              AND kind = 'failure' AND status = 'open'
            """,
            (session["plan_path"], session["plan_path"]),
        ).fetchone()[0]
    finally:
        connection.close()

    now = datetime.now(timezone.utc)
    live_leases = {
        str(row["display_path"]).replace("\\", "/"): row
        for row in lease_rows
        if datetime.fromisoformat(str(row["expires_at"]).replace("Z", "+00:00")) > now
    }
    owned_dirty: list[str] = []
    attributed_hashes: dict[str, str | None] = {}
    staged_hashes: dict[str, str | None] = {}
    for row in attributions:
        relative = str(row["display_path"]).replace("\\", "/")
        absolute = (repo / relative).resolve()
        if not absolute.is_relative_to(repo):
            continue
        if absolute.is_file():
            current_hash = hash_file(absolute)
            if current_hash != row["content_hash"]:
                continue
            if baseline_manifest.get(relative) != current_hash:
                candidate = subprocess.run(
                    ["git", "hash-object", f"--path={relative}", "--", relative],
                    cwd=repo,
                    capture_output=True,
                    check=False,
                    text=True,
                )
                if candidate.returncode != 0:
                    continue
                owned_dirty.append(relative)
                attributed_hashes[relative] = candidate.stdout.strip()
        elif relative in baseline_manifest:
            lease = live_leases.get(relative)
            deletion_owned = (
                lease is not None
                and lease["session_id"] == session_id
                and lease["base_hash"] == baseline_manifest.get(relative)
                and row["content_hash"] in (None, baseline_manifest.get(relative))
            )
            if deletion_owned:
                owned_dirty.append(relative)
                attributed_hashes[relative] = None

        if relative in attributed_hashes:
            staged = subprocess.run(
                ["git", "rev-parse", "--verify", f":{relative}"],
                cwd=repo,
                capture_output=True,
                check=False,
                text=True,
            )
            staged_hashes[relative] = staged.stdout.strip() if staged.returncode == 0 else None

    validator_path = (
        REPO_ROOT
        / ".codex/skills/zircon-project-skills/handle-plan-failure-handoffs/scripts/validate_plan_failure_handoffs.py"
    )
    failure_diagnostics: list[str] = []
    foreign_failure_diagnostics: list[str] = []
    canonical_open_failures = 0
    if validator_path.is_file():
        specification = importlib.util.spec_from_file_location("closeout_failure_validator", validator_path)
        if specification is None or specification.loader is None:
            failure_diagnostics.append("Cannot load Failure handoff validator")
        else:
            validator = importlib.util.module_from_spec(specification)
            sys.modules[specification.name] = validator
            specification.loader.exec_module(validator)
            records, parse_errors = validator.parse_handoff_records(repo)
            validation_errors = list(validator.validate_repository(repo))
            plan_key = str(session["plan_path"]).replace("\\", "/").casefold()
            plan_name = Path(plan_key).name
            child_match = re.match(r"^(\d+)-", plan_name)
            child_prefix = (
                (Path(plan_key).parent / child_match.group(1)).as_posix().casefold() + "/"
                if child_match
                else ""
            )
            related_artifacts: set[str] = set()
            for record in records:
                origin = record.origin_plan.relative_to(repo).as_posix().casefold()
                fixing = record.fixing_plan.relative_to(repo).as_posix().casefold()
                if plan_key in {origin, fixing}:
                    related_artifacts.add(record.relative_path.casefold())
                    if record.kind == "failure":
                        canonical_open_failures += 1
            markers = {plan_key, *related_artifacts}
            if child_prefix:
                markers.add(child_prefix)
            for diagnostic in [*parse_errors, *validation_errors]:
                normalized = diagnostic.replace("\\", "/").casefold()
                if any(marker in normalized for marker in markers):
                    failure_diagnostics.append(diagnostic)
                else:
                    foreign_failure_diagnostics.append(diagnostic)
    else:
        failure_diagnostics.append("Failure handoff validator is unavailable")

    return {
        "session_id": session["session_id"],
        "branch": branch,
        "service_mode": str(health.get("mode", "")),
        "session_status": session["status"],
        "plan_path": session["plan_path"],
        "owned_dirty_paths": owned_dirty,
        "attributed_hashes": attributed_hashes,
        "staged_hashes": staged_hashes,
        "leased_paths": sorted(live_leases, key=str.casefold),
        "open_failure_count": max(int(open_failures), canonical_open_failures),
        "failure_diagnostics": sorted(set(failure_diagnostics)),
        "foreign_failure_diagnostics": sorted(set(foreign_failure_diagnostics)),
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", required=True)
    parser.add_argument("--session-id", required=True)
    arguments = parser.parse_args()
    try:
        result = read_evidence(Path(arguments.repo_root), arguments.session_id)
    except (OSError, sqlite3.Error, ValueError) as error:
        print(json.dumps({"status": "error", "message": str(error)}))
        return 1
    print(json.dumps({"status": "ok", **result}, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
