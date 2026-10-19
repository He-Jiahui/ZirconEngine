"""User-authorized transition audit. Reads source/Jenkins; writes only audit metadata.

No Git mutation, build, source capture, service restart, or cleanup is performed.
Jenkins remains the lifecycle owner; temporary integration is a reviewed operator task.
"""
from __future__ import annotations

import argparse
from datetime import datetime, timedelta, timezone
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import sqlite3
import subprocess
import sys
import uuid

if __package__:
    from . import artifact_inventory as inventory
    from . import build_storage_audit
else:
    import artifact_inventory as inventory
    import build_storage_audit

TASK_ID = "jenkins-transition-monitor-01a12af6"
OWNER = "01a12af6-7511-70d1-a452-61b80678eb56"
ROOT_NAME = "jenkins-transition-monitor"
BEIJING = timezone(timedelta(hours=8))
TERMINAL = {"accepted", "complete", "completed", "cancelled", "failed", "rejected", "superseded"}


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def canonical(value) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode("utf-8")


def read_json(path: Path):
    inventory.physical_literal(path)
    return json.loads(path.read_text(encoding="utf-8-sig"))


def report_root(repo: Path, grants: dict, selected: Path | None = None) -> Path:
    root = selected or Path("E:/cargo-targets/zircon-local/reports") / ROOT_NAME
    inventory.physical_literal(root)
    # A user-supplied output path cannot turn this audit into a source/config writer.
    approved = Path("E:/cargo-targets/zircon-local/reports") / ROOT_NAME
    if root != approved or not inventory.writable(root, grants):
        raise ValueError("Audit output must be the explicitly writable transition report directory")
    return root


def write_json(path: Path, value, *, replace=False):
    inventory.physical_literal(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    data = canonical(value) + b"\n"
    if not replace:
        with path.open("xb") as stream:
            stream.write(data)
        return
    # This disposable file is owned by this invocation; it is always reconciled.
    pending = path.with_name(path.name + "." + uuid.uuid4().hex + ".pending")
    try:
        with pending.open("xb") as stream:
            stream.write(data)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(pending, path)
    finally:
        if pending.exists():
            pending.unlink()


def git(repo: Path, *args) -> bytes:
    environment = {**os.environ, "GIT_OPTIONAL_LOCKS": "0"}
    return subprocess.run(["git", "--no-optional-locks", "-c", "core.fsmonitor=false", "-C", str(repo), *args],
                          env=environment, capture_output=True, timeout=120, check=True).stdout


def git_observation(repo: Path) -> dict:
    before = git(repo, "rev-parse", "HEAD").decode().strip()
    rows = git(repo, "status", "--porcelain=v1", "-z", "--untracked-files=all").split(b"\0")
    changes, index = [], 0
    while index < len(rows):
        raw = rows[index]
        index += 1
        if not raw:
            continue
        status, name = raw[:2].decode("ascii"), raw[3:].decode("utf-8", "surrogateescape")
        row = {"path": name, "indexStatus": status[0], "worktreeStatus": status[1]}
        if "R" in status or "C" in status:
            row["previousPath"] = rows[index].decode("utf-8", "surrogateescape")
            index += 1
        path = repo / name
        try:
            inventory.physical_literal(path)
            if path.is_file():
                first = path.stat()
                row["sha256"] = sha(path.read_bytes())
                last = path.stat()
                row["stableDuringHash"] = (first.st_mtime_ns, first.st_size) == (last.st_mtime_ns, last.st_size)
        except (OSError, ValueError):
            row["hashObservation"] = "unavailable-or-aliased"
        changes.append(row)
    lock = repo / ".git/index.lock"
    try:
        inventory.physical_literal(lock)
        st = lock.stat()
        lock_fact = {"exists": True, "bytes": st.st_size, "modifiedNs": st.st_mtime_ns,
                     "sha256": sha(lock.read_bytes()), "disposition": "preserve-owner-investigation-required"}
    except FileNotFoundError:
        lock_fact = {"exists": False}
    except (OSError, ValueError):
        lock_fact = {"exists": True, "observation": "unavailable", "disposition": "preserve"}
    worktrees = git(repo, "worktree", "list", "--porcelain").decode("utf-8", "replace")
    branches = git(repo, "for-each-ref", "--format=%(refname:short) %(objectname)", "refs/heads/").decode().splitlines()
    after = git(repo, "rev-parse", "HEAD").decode().strip()
    return {"head": before, "headAfter": after, "stableHead": before == after,
            "branch": git(repo, "branch", "--show-current").decode().strip(),
            "changes": changes, "changedCount": len(changes),
            "stagedCount": sum(x["indexStatus"] not in {" ", "?"} for x in changes),
            "unmerged": git(repo, "diff", "--name-only", "--diff-filter=U", "-z").decode("utf-8", "replace").split("\0")[:-1],
            "indexLock": lock_fact, "worktreesPorcelain": worktrees,
            "prunableWorktreeCount": sum(x.startswith("prunable") for x in worktrees.splitlines()),
            "localBranches": branches,
            "limitation": "Live observation; dirty files and old branch names are not approved merge candidates."}


def verify_selected_driver(repo: Path) -> tuple[dict, Path, dict]:
    selector = read_json(repo / ".jenkins/state/deployment/driver.json")
    checksum = selector.get("driverDigest", "")
    if not isinstance(checksum, str) or not re.fullmatch("[0-9a-f]{64}", checksum):
        raise ValueError("Selected driver digest is invalid")
    root = repo / ".jenkins/runtime/drivers" / checksum
    inventory.physical_literal(root)
    if Path(selector.get("root", "")) != root:
        raise ValueError("Selected driver root differs")
    manifest = read_json(root / "driver-manifest.json")
    files = manifest["files"]
    if (manifest.get("schemaVersion") != 1 or not files or manifest.get("driverDigest") != checksum
            or sha(canonical({"schemaVersion": 1, "files": files})) != checksum):
        raise ValueError("Selected driver manifest differs")
    for name, entry in files.items():
        relative = PurePosixPath(name)
        if relative.is_absolute() or "\\" in name or ":" in name or any(p in {".", ".."} for p in relative.parts):
            raise ValueError("Selected driver manifest path is invalid")
        path = root.joinpath(*relative.parts)
        inventory.physical_literal(path)
        raw = path.read_bytes()
        if len(raw) != entry["bytes"] or sha(raw) != entry["sha256"]:
            raise ValueError("Selected driver file differs")
    observed = set()
    for path in root.rglob("*"):
        inventory.physical_literal(path)
        if path.is_file():
            observed.add(path.relative_to(root).as_posix())
    if observed != set(files) | {"driver-manifest.json"}:
        raise ValueError("Selected driver has undeclared files")
    return selector, root, files


HEALTH_PROBE = r'''
import json,sys
from pathlib import Path
sys.path.insert(0,sys.argv[1])
from tools.jenkins.deployment.manager import DeploymentManager
from tools.jenkins.deployment.paths import resolve_paths
from tools.jenkins.deployment.spec import load_spec
from tools.jenkins.deployment.live_observations import observe_live_identity,observe_endpoint_owner
repo=Path(sys.argv[2]); spec=load_spec(repo/'.jenkins/deployment-spec.json')
if spec.controller['listenAddress']!='127.0.0.1': raise ValueError('Non-loopback endpoint')
manager=DeploymentManager(spec,resolve_paths(spec))
host=json.loads((repo/'.jenkins/state/deployment/host.json').read_text())
operation=json.loads((repo/'.jenkins/state/deployment-operation.json').read_text())
owners={}
for name in ['controllerIdentity','agentIdentity']:
 try: owners[name]={'verified':True,'identity':observe_live_identity(host[name])}
 except Exception as error: owners[name]={'verified':False,'errorType':type(error).__name__}
try:
 endpoint={'verified':True,'observation':observe_endpoint_owner('127.0.0.1',int(spec.controller['httpPort']),host['controllerIdentity']['pid'])}
except Exception as error: endpoint={'verified':False,'errorType':type(error).__name__}
health=manager.health() if endpoint.get('verified') and owners['controllerIdentity'].get('verified') else {'ready':False,'reason':'native_endpoint_owner_unverified'}
print(json.dumps({'health':health,'owners':owners,'endpoint':endpoint,
 'operationState':operation.get('state'),'operationId':operation.get('operationId'),'generation':operation.get('generation'),
 'hostStatus':host.get('status'),'hostOperationId':host.get('operationId')}))
'''


def jenkins_observation(repo: Path) -> dict:
    try:
        selector, driver, files = verify_selected_driver(repo)
        result = subprocess.run([sys.executable, "-I", "-B", "-c", HEALTH_PROBE, str(driver), str(repo)],
                                cwd=repo, env={**os.environ, "NO_PROXY": "127.0.0.1,localhost"},
                                capture_output=True, encoding="utf-8", timeout=45, check=True)
        observed = json.loads(result.stdout)
        binding = (selector.get("runtimeOperationId") == observed.get("operationId")
                   == observed.get("generation") == observed.get("hostOperationId"))
        owned = all(x.get("verified") is True for x in observed["owners"].values()) and observed["endpoint"]["verified"]
        observed.update({"driverIntegrityVerified": True, "driverDigest": selector["driverDigest"],
                         "runtimeBindingVerified": binding, "nativeOwnersVerified": bool(owned),
                         "healthy": bool(binding and owned and observed["health"].get("ready")
                                         and observed["operationState"] == "running"),
                         "deployedPostTaskCacheCode": "tools/jenkins/workflow/post_task_cache.py" in files,
                         "formalMergeAcceptance": "not-established-by-health", "rawCredentialsLogged": False})
        return observed
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        return {"healthy": False, "observationComplete": False, "errorType": type(error).__name__,
                "rawCredentialsLogged": False, "formalMergeAcceptance": "unknown"}


def state_observation(repo: Path) -> dict:
    path = repo / ".jenkins/state/coordination.sqlite3"
    try:
        inventory.physical_literal(path)
        with sqlite3.connect(path.as_uri() + "?mode=ro", uri=True, timeout=5) as connection:
            connection.execute("PRAGMA query_only=ON")
            connection.execute("PRAGMA temp_store=MEMORY")
            domains = dict(connection.execute("SELECT domain,count(*) FROM records GROUP BY domain"))
            pending = []
            for key, raw, version, updated in connection.execute(
                    "SELECT key,payload,version,updated_at FROM records WHERE domain='workflow'"):
                value = json.loads(raw)
                if value.get("status") not in TERMINAL or value.get("status") in {"failed", "rejected"}:
                    pending.append({"key": key, "version": version, "updatedAt": updated,
                                    "status": value.get("status"), "template": value.get("template"),
                                    "sourceDigest": value.get("sourceDigest"), "driverDigest": value.get("driverDigest"),
                                    "reasonCode": value.get("reasonCode")})
            policy_row = connection.execute("SELECT payload FROM records WHERE domain='resource_policy' AND key='default'").fetchone()
            policy = json.loads(policy_row[0]) if policy_row else {}
        return {"complete": True, "domainCounts": domains, "pendingWorkflows": pending,
                "scheduledMaintenance": policy.get("scheduledMaintenance"),
                "postTaskCachePolicy": policy.get("postTaskCache"), "retentionPolicy": policy.get("retention"),
                "diskReserveBytes": policy.get("diskReserveBytes"),
                "limitation": "Fixture acceptance and domain counts do not accept current source or merge lifecycle."}
    except (OSError, ValueError, sqlite3.Error) as error:
        return {"complete": False, "errorType": type(error).__name__}


def findings(report: dict) -> list[dict]:
    issues = []
    def add(code, severity, subject, message):
        issues.append({"id": code + ":" + subject, "code": code, "severity": severity, "subject": subject, "message": message})
    if not report["jenkins"].get("healthy"):
        add("jenkins_unhealthy", "blocking", "runtime", "Jenkins runtime is stopped, unreachable, unbound, or unverified")
    observation = report["git"]
    if observation.get("observationComplete") is False:
        add("git_observation_incomplete", "blocking", "checkout", "Git observation failed; preserve the checkout and retry read-only")
    if observation.get("indexLock", {}).get("exists"):
        add("index_lock", "blocking", ".git/index.lock", "Preserve the lock pending native occupancy and owner evidence")
    if observation.get("unmerged"):
        add("unmerged", "blocking", "checkout", "An unfinished merge has unmerged paths")
    if not observation.get("stableHead", False) or any(x.get("stableDuringHash") is False for x in observation.get("changes", [])):
        add("source_moving", "review", "checkout", "Source changed during observation; refresh the candidate before integration")
    for disk in report.get("disks", []):
        if disk.get("belowBuildReserve"):
            add("disk_reserve", "blocking", disk["drive"], "Free bytes are below the 35 GiB compilation reserve; no deletion is authorized")
    for value in report.get("state", {}).get("pendingWorkflows", []):
        if value.get("status") in {"blocked", "failed", "waiting", "unknown", "delivery_unknown", "held"}:
            add("workflow_attention", "review", value["key"], str(value.get("status")))
    if not report["state"].get("complete"):
        add("state_unavailable", "blocking", "jenkins-state", "Jenkins State cannot be observed read-only")
    if not report["state"].get("scheduledMaintenance"):
        add("scheduled_maintenance_unverified", "review", "jenkins", "An authoritative scheduled-maintenance policy is unverified")
    deep = report.get("inventory")
    if deep:
        if not deep.get("processObservationComplete"):
            add("process_observation_incomplete", "review", "inventory", "Process observation is incomplete; active output ownership is unresolved")
        for row in deep.get("outputItems", []):
            if row.get("category") == "legacy-task-output-unclaimed" and not row.get("manifestPaths"):
                add("owner_manifest_missing", "review", row["path"], "Legacy output needs an owner review; preserve it")
            if row.get("sourceTreeMarkers"):
                add("source_copy_candidate", "review", row["path"], "Source-tree markers need Jenkins ownership review; preserve them")
    issues.extend(report.get("buildStorage", {}).get("findings", []))
    return sorted({x["id"]: x for x in issues}.values(), key=lambda x: x["id"])


def issue_delta(previous: list[dict], current: list[dict]) -> dict:
    before = {x["id"]: x for x in previous}
    after = {x["id"]: x for x in current}
    changed = [after[k] for k in after if k not in before or after[k] != before[k]]
    resolved = [before[k] for k in before if k not in after]
    return {"notify": bool(changed or resolved), "newOrChanged": changed, "resolved": resolved}


def reconcile_manifest(existing: dict | None, fresh: dict) -> dict:
    """A new audit cannot erase pending ownership or scheduler handoff metadata."""
    if existing is None:
        return fresh
    if any(existing.get(key) != fresh[key] for key in ("task_id", "owner", "source_root", "output_roots")):
        raise ValueError("Existing report directory has a different task or owner")
    return {**existing, "status": "active"}


def run(repo: Path, output: Path | None = None, *, deep=False) -> dict:
    repo = inventory.physical_literal(repo)
    _, grants, config_hash = inventory.load_grants(repo)
    root = report_root(repo, grants, output)
    previous = read_json(root / "latest.json") if (root / "latest.json").exists() else {}
    now = datetime.now(timezone.utc)
    run_id = now.strftime("%Y%m%dT%H%M%S") + "-" + uuid.uuid4().hex[:8]
    folder = root / now.astimezone(BEIJING).strftime("%Y-%m-%d") / run_id
    manifest = {"schema_version": 1, "task_id": TASK_ID, "owner": OWNER, "status": "active",
                "source_root": str(repo), "output_roots": [str(root)], "processes": [],
                "retained": [{"path": str(root), "kind": "audit-metadata-directory",
                              "reason": "User-authorized daily audit and temporary merge health history; Jenkins owns retention",
                              "depends_on": [str(repo / ".codex/config.toml")],
                              "review_after": (now + timedelta(days=30)).isoformat()}],
                "unknown": [], "cleanup": {"deleted_files": 0, "deleted_bytes": 0}}
    manifest_path = root / "artifacts.json"
    manifest = reconcile_manifest(read_json(manifest_path) if manifest_path.exists() else None, manifest)
    manifest["last_audit_started_at_utc"] = now.isoformat()
    write_json(manifest_path, manifest, replace=True)
    report = {"schemaVersion": 1, "taskId": TASK_ID, "runId": run_id, "owner": OWNER,
              "observedAtUtc": now.isoformat(), "observedAtBeijing": now.astimezone(BEIJING).isoformat(),
              "repository": str(repo), "configSha256": config_hash, "mode": "deep" if deep else "fast",
              "runtimePermissionEnforcement": "not-proven-by-this-audit",
              "jenkins": jenkins_observation(repo), "state": state_observation(repo),
              "integrationAuthority": "temporary-user-authorized-operator-review",
              "automaticGitWrites": False, "automaticCleanup": False}
    report["auditImplementation"] = {str(path): sha(path.read_bytes()) for path in
                                     (Path(__file__).absolute(), Path(inventory.__file__).absolute(),
                                      Path(build_storage_audit.__file__).absolute())}
    try:
        report["git"] = git_observation(repo)
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        report["git"] = {"observationComplete": False, "errorType": type(error).__name__}
    report["disks"] = []
    for drive in "DEF":
        try:
            usage = inventory.shutil.disk_usage(f"{drive}:/")
            report["disks"].append({"drive": drive, "freeBytes": usage.free, "belowBuildReserve": usage.free < 35 * 1024**3})
        except OSError:
            report["disks"].append({"drive": drive, "observation": "unavailable"})
    if deep:
        try:
            report["inventory"] = inventory.scan(repo)
        except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
            report["inventory"] = {"complete": False, "outputItems": [], "errorType": type(error).__name__,
                                   "disposition": "retain previous findings; scan did not complete"}
    report["buildStorage"] = build_storage_audit.audit(repo, report.get("inventory"), previous.get("buildStorage"))
    report["findings"] = findings(report)
    # Keep ownership findings from the latest deep scan across quick health checks.
    if not deep:
        existing = {x["id"] for x in report["findings"]}
        report["findings"].extend(x for x in previous.get("findings", [])
                                  if x["code"] in {"owner_manifest_missing", "source_copy_candidate", "process_observation_incomplete"}
                                  and x["id"] not in existing)
        report["lastDeepReport"] = previous.get("lastDeepReport")
    else:
        report["lastDeepReport"] = str(folder / "report.json")
        if not report["buildStorage"].get("complete"):
            existing = {x["id"] for x in report["findings"]}
            report["findings"].extend(x for x in previous.get("findings", [])
                if x["code"] in {"owner_manifest_missing", "source_copy_candidate", "process_observation_incomplete"}
                and x["id"] not in existing)
    report["attention"] = issue_delta(previous.get("findings", []), report["findings"])
    report["reportPath"] = str(folder / "report.json")
    report["completedAtUtc"] = datetime.now(timezone.utc).isoformat()
    write_json(folder / "report.json", report)
    latest = {k: v for k, v in report.items() if k != "inventory"}
    write_json(root / "latest.json", latest, replace=True)
    return latest


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, default=Path(__file__).absolute().parents[2])
    parser.add_argument("--deep", action="store_true", help="Include the complete permitted-directory inventory")
    parser.add_argument("--preflight", action="store_true", help="Validate the physical report write grant without creating files")
    args = parser.parse_args(argv)
    try:
        if args.preflight:
            repo = inventory.physical_literal(args.repo)
            _, grants, _ = inventory.load_grants(repo)
            print(json.dumps({"reportRoot": str(report_root(repo, grants)), "valid": True}))
            return 0
        value = run(args.repo, deep=args.deep)
        print(json.dumps({"reportPath": value["reportPath"], "findings": len(value["findings"]),
                          "jenkinsHealthy": value["jenkins"].get("healthy"), "attention": value["attention"]["notify"]}))
        return 0
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        print(json.dumps({"status": "audit_failed", "errorType": type(error).__name__}), file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
