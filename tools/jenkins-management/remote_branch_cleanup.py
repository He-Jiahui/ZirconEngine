"""Jenkins-owned periodic cleanup of merged, unoccupied remote topic branches."""
from __future__ import annotations

import argparse
from contextlib import closing
from datetime import datetime, timezone
import fnmatch
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import sqlite3
import stat
import subprocess
import sys
import time
import urllib.parse
import urllib.request
import urllib.error

ROOT = Path("E:/Git/ZirconEngine")
STATE = ROOT / ".jenkins/state/independent-service/remote-branch-cleanup"
REPORT = Path("E:/cargo-targets/zircon-local/reports/remote-merged-branch-cleanup-20261018")
JOB = "zircon-remote-branch-cleanup"
OWNER = "01a12fdb-4d74-75d1-92fa-ca578416c6ab"
REPOSITORIES = {
    "E:/Git/ZirconEngine": "He-Jiahui/ZirconEngine",
    "E:/Git/zr_vm": "He-Jiahui/zr_vm",
}
FINISHED = {"merged", "deleted", "already-on-remote-main"}
OID = re.compile(r"[0-9a-f]{40}\Z")


def physical(path):
    path = Path(os.path.abspath(path))
    for ancestor in (path, *path.parents):
        try:
            info = ancestor.lstat()
        except FileNotFoundError:
            continue
        if stat.S_ISLNK(info.st_mode) or getattr(info, "st_file_attributes", 0) & 0x400:
            raise ValueError("Physical paths required")
    return path


def read(path):
    path = physical(path)
    before = path.stat()
    value = path.read_bytes()
    after = path.stat()
    if (before.st_ino, before.st_size, before.st_mtime_ns) != (after.st_ino, after.st_size, after.st_mtime_ns):
        raise ValueError("Input changed while being read: " + str(path))
    return value


def sha(value):
    return hashlib.sha256(value).hexdigest()


def encoded(value):
    return (json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n").encode("utf-8")


def save(path, value, *, replace=False):
    path = physical(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists() and path.stat().st_nlink != 1:
        raise ValueError("State outputs must not be hard links")
    if not replace:
        with path.open("xb") as stream:
            stream.write(encoded(value)); stream.flush(); os.fsync(stream.fileno())
        return
    old = read(path) if path.exists() else None
    if old is None:
        save(path, value)
    else:
        with path.open("r+b") as stream:
            if stream.read() != old:
                raise ValueError("Concurrent state update")
            stream.seek(0); stream.write(encoded(value)); stream.truncate()
            stream.flush(); os.fsync(stream.fileno())


def load_module(name, path, expected=None):
    data = read(path)
    if expected and sha(data) != expected:
        raise ValueError("Helper changed: " + str(path))
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    exec(compile(data, str(path), "exec"), module.__dict__)
    return module


def protected_reason(name, default, protected, refs, oids, oid, open_heads, patterns):
    if (not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._/-]{0,240}", name)
            or ".." in name or "//" in name or name.endswith(("/", ".lock"))):
        return "unrecognized-ref-name"
    if name == default or any(fnmatch.fnmatchcase(name.casefold(), p.casefold()) for p in patterns):
        return "default-or-preserved-branch"
    if protected is not False:
        return "protected-or-protection-unknown"
    if name in refs or oid in oids:
        return "worktree-or-unfinished-task"
    if name in open_heads:
        return "open-pull-request"
    if not OID.fullmatch(oid):
        return "invalid-tip"
    return None


def reference_holds(value):
    refs, oids = set(), set()
    def visit(item):
        if isinstance(item, dict):
            for key, child in item.items():
                if isinstance(child, str):
                    if key.casefold() in {"branch", "sourcebranch", "headbranch", "basebranch", "targetbranch",
                                           "headref", "sourceref", "candidateref", "integrationbranch", "ref"}:
                        refs.add(child.removeprefix("refs/heads/"))
                    if key.casefold() in {"sourcecommit", "candidatecommit", "commit", "commitid", "commitsha",
                                           "revision", "head", "sourcehead"} and OID.fullmatch(child):
                        oids.add(child)
                visit(child)
        elif isinstance(item, list):
            for child in item:
                visit(child)
    if not isinstance(value, dict) or str(value.get("status", "")).casefold() not in FINISHED:
        visit(value)
    return refs, oids


def reconcile_pending(entry, current_oid):
    if entry.get("status") != "unknown":
        return None
    if current_oid is None:
        return "original-deletion-confirmed-absent"
    if current_oid != entry["oid"]:
        return "original-ref-changed-preserved"
    return "original-deletion-outcome-needs-review"


class Commands:
    def __init__(self, output, deadline, github_reader, git_exe):
        self.output, self.deadline, self.counter = output, deadline, 0
        self.github_reader = github_reader
        self.environment = {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}
        self.environment.update(GIT_TERMINAL_PROMPT="0", GCM_INTERACTIVE="Never", GIT_OPTIONAL_LOCKS="0", GH_PROMPT_DISABLED="1")
        self.git_exe = str(physical(git_exe))

    def call(self, args, repo, *, check=True):
        self.counter += 1
        timeout = min(75, self.deadline - time.monotonic())
        if timeout <= 0:
            raise TimeoutError("Remote cleanup command budget exhausted")
        result = subprocess.run(args, cwd=repo, env=self.environment, stdin=subprocess.DEVNULL,
                                capture_output=True, timeout=timeout)
        save(self.output / f"command-{self.counter:04d}.json", {
            "argv": args, "cwd": str(repo), "exitCode": result.returncode,
            "stdoutSha256": sha(result.stdout), "stderrSha256": sha(result.stderr),
            "stdout": result.stdout.decode("utf-8", "replace"), "stderr": result.stderr.decode("utf-8", "replace")})
        if check and result.returncode:
            raise ValueError("Native command failed; see command-" + f"{self.counter:04d}.json")
        return result

    def git(self, repo, *args, check=True):
        return self.call([self.git_exe, "-c", "core.fsmonitor=false", "-c", "core.hooksPath=" + os.devnull,
                          "-c", "core.quotepath=false", *args], repo, check=check)

    def api(self, repository, suffix):
        if time.monotonic() >= self.deadline:
            raise TimeoutError("Remote cleanup API budget exhausted")
        # Reuse the established PR gate and its noninteractive Git credentials.
        # Credentials and the credential helper's output are never recorded.
        result = (self.github_reader(repository, suffix.lstrip("/")) if self.github_reader
                  else self.github_get(repository, suffix))
        self.counter += 1
        save(self.output / f"command-{self.counter:04d}.json", {
            "kind": "github-read", "method": "GET", "repository": repository,
            "endpoint": suffix, "responseSha256": sha(encoded(result)), "response": result})
        return result

    def github_get(self, repository, suffix):
        # The same noninteractive credential-fill protocol used by the PR gate.
        # Never pass this secret-bearing response to the command log writer.
        token = self.environment.get("GH_TOKEN") or self.environment.get("GITHUB_TOKEN")
        if not token:
            credential = subprocess.run([self.git_exe, "-c", "credential.interactive=false", "credential", "fill"],
                cwd=ROOT, env=self.environment,
                input=("protocol=https\nhost=github.com\npath=" + repository + ".git\n\n").encode(),
                stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=30)
            if credential.returncode:
                raise ValueError("Existing GitHub credentials unavailable")
            values = dict(line.split("=", 1) for line in credential.stdout.decode("utf-8").splitlines() if "=" in line)
            token = values.get("password")
        if not token:
            raise ValueError("Authenticated branch observations required")
        headers = {"Accept": "application/vnd.github+json", "User-Agent": "Zircon-Jenkins-Remote-Cleanup",
                   "Authorization": "Bearer " + token, "X-GitHub-Api-Version": "2022-11-28"}
        request = urllib.request.Request("https://api.github.com/repos/" + repository + suffix, headers=headers)
        try:
            with urllib.request.urlopen(request, timeout=30) as response:
                return json.load(response)
        except urllib.error.HTTPError as error:
            code = error.code
            error.close()
            raise ValueError("GitHub branch observation unavailable (HTTP " + str(code) + ")") from None
        except (urllib.error.URLError, TimeoutError):
            raise ValueError("GitHub branch observation did not complete") from None

    def pages(self, repository, suffix, limit=1000):
        values = []
        for page in range(1, limit // 100 + 2):
            separator = "&" if "?" in suffix else "?"
            items = self.api(repository, suffix + separator + f"per_page=100&page={page}")
            if not isinstance(items, list):
                raise ValueError("Expected a complete GitHub list")
            values.extend(items)
            if len(values) > limit:
                raise ValueError("Inventory exceeds configured budget; retain all branches")
            if len(items) < 100:
                return values
        raise ValueError("Remote pagination incomplete")


class StateReferenceCache:
    """Cache reference sets, never source data; refresh changed versioned rows."""
    def __init__(self):
        self.rows = {}

    def _refresh(self, connection):
        updated = {}
        for domain, key, version, modified in connection.execute("SELECT domain,key,version,updated_at FROM records"):
            identity, marker = (domain, key), (version, modified)
            old = self.rows.get(identity)
            if old is not None and old[0] == marker:
                updated[identity] = old
            else:
                (payload,) = connection.execute("SELECT payload FROM records WHERE domain=? AND key=?", identity).fetchone()
                updated[identity] = (marker, *reference_holds(json.loads(payload)))
        self.rows = updated
        refs, oids = set(), set()
        for _, r, o in updated.values():
            refs.update(r); oids.update(o)
        return refs, oids

    def refresh(self):
        database = ROOT / ".jenkins/state/coordination.sqlite3"
        with closing(sqlite3.connect(physical(database).as_uri() + "?mode=ro", uri=True, timeout=3)) as connection:
            connection.execute("BEGIN")
            return self._refresh(connection)


def local_holds(commands, repo):
    refs, oids = set(), set()
    raw = commands.git(repo, "worktree", "list", "--porcelain").stdout.decode("utf-8")
    for part in raw.splitlines():
        if part.startswith("branch refs/heads/"):
            refs.add(part[len("branch refs/heads/"):])
        elif part.startswith("HEAD ") and OID.fullmatch(part[5:]):
            oids.add(part[5:])
    r, o = commands.state_refs.refresh(); refs.update(r); oids.update(o)
    operations = ROOT / ".jenkins/state/independent-service/zircon-git-sync/operations"
    for path in sorted(operations.glob("*.json")):
        r, o = reference_holds(json.loads(read(path))); refs.update(r); oids.update(o)
    return refs, oids


def open_heads(commands, repository):
    # Preserve both sides of stacked PRs, including a deleted source repo.
    refs = set()
    for pull in commands.pages(repository, "/pulls?state=open"):
        for role in ("head", "base"):
            ref = (pull.get(role) or {}).get("ref")
            if isinstance(ref, str):
                refs.add(ref)
    return refs


def verify_plan(expected):
    raw = read(STATE / "plan.json")
    if sha(raw) != expected:
        raise ValueError("Deployment input changed")
    plan = json.loads(raw)
    executed = {str(ROOT / relative) for relative in (
        "tools/jenkins-management/remote_branch_cleanup.py",
        "tools/jenkins-management/tests/test_remote_branch_cleanup.py",
        "tools/jenkins-management/checkout_commit.py",
        "tools/jenkins-management/run_checks.py",
        "tools/dev/local_cargo.py",
    )} | {plan["gitExecutable"]}
    if set(plan["sourcePins"]) != executed:
        raise ValueError("Cleanup pins must cover its exact executed dependencies")
    for path, wanted in plan["sourcePins"].items():
        if sha(read(path)) != wanted:
            raise ValueError("Pinned cleanup dependency changed: " + path)
    selected = json.loads(read(ROOT / ".jenkins/state/deployment/driver.json"))
    if (selected["driverDigest"], selected["runtimeOperationId"]) != (plan["driverDigest"], plan["runtimeOperationId"]):
        raise ValueError("Selected Jenkins runtime changed; keep cleanup pending")
    return plan


def verify_runtime(checks, plan):
    # The selected installed capsule may advance independently of this job.
    # Verify its seal and runtime through the existing verifier; only code this
    # job actually executes must match the cleanup deployment's source pins.
    installed = json.loads(read(ROOT / ".jenkins/state/management/installed.json"))
    digest = installed.get("capsuleSha256", "")
    if not isinstance(digest, str) or not re.fullmatch(r"[0-9a-f]{64}", digest):
        raise ValueError("Invalid installed management capsule")
    capsule = physical(ROOT / ".jenkins/runtime/management" / digest)
    if physical(installed.get("capsule", "")) != capsule:
        raise ValueError("Installed management capsule path changed")
    raw = read(capsule / "capsule.json")
    if sha(raw) != digest:
        raise ValueError("Installed management capsule seal changed")
    clients = json.loads(raw)["clientSources"]
    checks.verify_pins(ROOT, plan["driverDigest"], plan["runtimeOperationId"], clients)
    observer = "tools/dev/local_cargo.py"
    if clients[observer] != plan["sourcePins"][str(ROOT / observer)]:
        raise ValueError("Installed native observer differs from the cleanup deployment")
    return digest


def record_preflight_failure(output, args, error):
    receipt = {"status": "preflight-failed", "reason": type(error).__name__ + ": " + str(error),
        "nativeStarted": False, "remoteCommandsStarted": False, "remoteBranchesDeleted": 0,
        "formalAcceptance": False, "sourceCopies": 0, "filesDeleted": 0, "releasedBytes": 0}
    save(output / "receipt.json", receipt)
    summary = dict(receipt, job=JOB, build=int(os.environ["BUILD_NUMBER"]), mode=args.mode,
        nativeComplete=False, nativeExitCode=None, completedAtUtc=datetime.now(timezone.utc).isoformat())
    save(output / "summary.json", summary)
    if args.mode != "self-test":
        save(STATE / "state.json", summary, replace=True)
        save(ROOT / ".jenkins/records" / f"remote-branch-cleanup-{os.environ['BUILD_NUMBER']}.response.json", {
            "isTriggerTask": False, "triggerPermitted": False, "deleteAuthorized": False,
            "ownerThreadId": OWNER, "delivery": "durable-inbox-only", "status": receipt["status"],
            "summary": str(output / "summary.json"), "summarySha256": sha(read(output / "summary.json")),
            "remoteBranchesDeleted": 0, "formalAcceptance": False})
    print(json.dumps(receipt))
    return 1


def index_available(commands, repo):
    git_dir = physical(commands.git(repo, "rev-parse", "--absolute-git-dir").stdout.decode().strip())
    if (git_dir / "index.lock").exists():
        raise ValueError("Existing Git index owner retained")


def remote_oid(commands, repo, name):
    lines = commands.git(repo, "ls-remote", "--heads", "origin", "refs/heads/" + name).stdout.decode().splitlines()
    if not lines:
        return None
    if len(lines) != 1 or lines[0].split()[1] != "refs/heads/" + name:
        raise ValueError("Unexpected remote ref advertisement")
    value = lines[0].split()[0]
    if not OID.fullmatch(value):
        raise ValueError("Invalid remote object ID")
    return value


def cleanup_repository(commands, repo, repository, policy, policy_sha, mode, output, journal,
                       entry, verify_sources):
    expected = "https://github.com/" + repository + ".git"
    if commands.git(repo, "remote", "get-url", "origin").stdout.decode().strip() != expected:
        raise ValueError("Origin identity changed")
    pushurl = commands.git(repo, "remote", "get-url", "--push", "origin").stdout.decode().strip()
    if pushurl != expected:
        raise ValueError("Push destination differs from the registered origin")
    metadata = commands.api(repository, "")
    if metadata.get("archived") or metadata.get("disabled") or metadata.get("full_name", "").casefold() != repository.casefold():
        raise ValueError("Repository identity or state changed")
    default = metadata["default_branch"]
    branches = commands.pages(repository, "/branches", policy["maxBranchesPerRepository"])
    heads = open_heads(commands, repository)
    refs, oids = local_holds(commands, repo)
    index_available(commands, repo)
    commands.git(repo, "fetch", "--no-tags", "origin")
    default_oid = remote_oid(commands, repo, default)
    if default_oid is None:
        raise ValueError("Remote default branch unavailable")
    results = []
    entry.update(defaultBranch=default, defaultOid=default_oid, branches=results)
    # Reconcile vanished refs as well as refs still present in the branch list.
    for key, pending in journal.items():
        if pending.get("repository") == repository and pending.get("status") == "unknown":
            disposition = reconcile_pending(pending, remote_oid(commands, repo, pending["branch"]))
            if disposition != "original-deletion-outcome-needs-review":
                pending["status"] = disposition
                save(STATE / "journal.json", journal, replace=True)
    for branch in sorted(branches, key=lambda b: b["name"]):
        name, oid = branch["name"], branch["commit"]["sha"]
        item = {"branch": name, "oid": oid, "defaultBranch": default, "defaultOid": default_oid, "status": "retained"}
        results.append(item)
        key = repository + ":refs/heads/" + name
        pending = journal.get(key, {})
        if pending.get("status") == "unknown":
            item["reason"] = reconcile_pending(pending, remote_oid(commands, repo, name))
            if item["reason"] != "original-deletion-outcome-needs-review":
                pending["status"] = item["reason"]; save(STATE / "journal.json", journal, replace=True)
            continue
        reason = protected_reason(name, default, branch.get("protected"), refs, oids, oid, heads, policy["preservePatterns"])
        if reason:
            item["reason"] = reason; continue
        merged = commands.git(repo, "merge-base", "--is-ancestor", oid, default_oid, check=False)
        if merged.returncode != 0:
            item["reason"] = "not-proven-merged" if merged.returncode == 1 else "merge-proof-unavailable"
            continue
        item["mergeProof"] = "tip-is-ancestor-of-current-remote-default"
        if mode == "audit":
            item["status"] = "eligible"; continue
        if sum(i["status"] == "deleted" for i in results) >= policy["maxDeletesPerRepository"]:
            item["reason"] = "cycle-deletion-budget"; continue
        # Refresh policy, live worktree/task holds, open PRs and the exact remote tip.
        if sha(read(STATE / "policy.json")) != policy_sha:
            raise ValueError("Policy changed before remote deletion")
        verify_sources()
        current = commands.api(repository, "/branches/" + urllib.parse.quote(name, safe=""))
        fresh_refs, fresh_oids = local_holds(commands, repo)
        reason = protected_reason(name, default, current.get("protected"), fresh_refs, fresh_oids,
                                  current["commit"]["sha"], open_heads(commands, repository), policy["preservePatterns"])
        if reason or current["commit"]["sha"] != oid or remote_oid(commands, repo, default) != default_oid:
            item["reason"] = reason or "remote-ref-or-default-advanced"; continue
        index_available(commands, repo)
        # An exact expected old OID makes deletion fail if another writer advances it.
        intent = {"repository": repository, "branch": name, "oid": oid, "defaultOid": default_oid,
                  "status": "unknown", "run": str(output), "policySha256": policy_sha}
        journal[key] = intent
        save(output / ("delete-intent-" + sha(key.encode())[:16] + ".json"), intent)
        save(STATE / "journal.json", journal, replace=True)
        result = commands.git(repo, "push", "--porcelain", "--force-with-lease=refs/heads/" + name + ":" + oid,
                              "origin", ":refs/heads/" + name, check=False)
        after = remote_oid(commands, repo, name)
        if result.returncode != 0 or after is not None:
            item["status"] = "unknown"
            item["reason"] = "remote-deletion-needs-reconciliation"
            raise ValueError("Remote deletion outcome requires its original journal entry")
        item["status"] = "deleted"
        item["absenceVerified"] = True
        journal[key] = dict(intent, status="deleted", remoteRefAbsent=True)
        save(STATE / "journal.json", journal, replace=True)
    return entry


def worker(args):
    output = physical(args.output)
    if output != physical(REPORT / "runs" / os.environ["BUILD_NUMBER"]):
        raise ValueError("Worker output differs from its original build")
    plan = verify_plan(args.plan_sha)
    policy_raw = read(STATE / "policy.json"); policy = json.loads(policy_raw)
    if policy.get("schemaVersion") != 1 or policy.get("authorization", {}).get("source") != "user":
        raise ValueError("Missing user cleanup authorization")
    if policy.get("repositories") != list(REPOSITORIES) or not 1 <= policy["maxDeletesPerRepository"] <= 32:
        raise ValueError("Invalid bounded repository or deletion scope")
    if not 1 <= policy["maxBranchesPerRepository"] <= 2000 or not isinstance(policy.get("preservePatterns"), list):
        raise ValueError("Invalid branch policy")
    if args.mode == "apply" and policy.get("enabled") is not True:
        raise ValueError("Remote cleanup policy is disabled")
    helper = load_module("remote_cleanup_git_lock", ROOT / "tools/jenkins-management/checkout_commit.py", args.lock_helper_sha)
    native_git = plan["gitExecutable"]
    commands = Commands(output, time.monotonic() + 540, None, native_git)
    commands.state_refs = StateReferenceCache()
    result = {"schemaVersion": 1, "mode": args.mode, "status": "running", "repositories": [], "remoteBranchesDeleted": 0,
              "sourceCopies": 0, "localBranchesDeleted": 0, "worktreesDeleted": 0, "filesDeleted": 0, "releasedBytes": 0,
              "policySha256": sha(policy_raw), "formalAcceptance": False}
    try:
        # Historical payloads total several GiB. Parse them once outside the Git
        # critical section, then recheck row versions and fresh live worktrees
        # under the shared lock before selection and each exact deletion.
        commands.state_refs.refresh()
        with helper.git_lock(ROOT / ".jenkins/state/locks/management-git.lock"):
            journal = json.loads(read(STATE / "journal.json")) if (STATE / "journal.json").exists() else {}
            for root, repository in REPOSITORIES.items():
                entry = {"repository": repository, "root": root, "branches": []}
                result["repositories"].append(entry)
                cleanup_repository(commands, physical(root), repository, policy, sha(policy_raw), args.mode, output,
                                   journal, entry, lambda: verify_plan(args.plan_sha))
            result["status"] = "complete"
    except helper.CommitBusy:
        result["status"] = "waiting-native-git"
        result["reason"] = "Existing management Git owner retained; next scheduled cycle will reconcile"
    except Exception as error:
        result.update(status="failed", reason=type(error).__name__ + ": " + str(error))
    for entry in result["repositories"]:
        entry["deletedCount"] = sum(i["status"] == "deleted" for i in entry["branches"])
        entry["eligibleCount"] = sum(i["status"] == "eligible" for i in entry["branches"])
        result["remoteBranchesDeleted"] += entry["deletedCount"]
    save(output / "receipt.json", result)
    return 1 if result["status"] == "failed" else 0


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mode", choices=("self-test", "audit", "apply"), required=True)
    parser.add_argument("--self-sha", required=True)
    parser.add_argument("--plan-sha", required=True)
    parser.add_argument("--worker", action="store_true")
    parser.add_argument("--output", type=Path)
    parser.add_argument("--lock-helper-sha")
    args = parser.parse_args(argv)
    if sha(read(__file__)) != args.self_sha:
        raise ValueError("Cleanup source changed; revalidate before deployment")
    if os.environ.get("JOB_NAME") != JOB or os.environ.get("NODE_NAME") != "zircon-windows-agent":
        raise ValueError("Cleanup must run on its independent Jenkins job and fixed agent")
    if args.worker:
        return worker(args)
    number = os.environ["BUILD_NUMBER"]
    if not number.isdigit():
        raise ValueError("Invalid Jenkins build identity")
    output = physical(REPORT / "runs" / number); output.mkdir(parents=True)
    manifest = {"taskId": "remote-merged-branch-cleanup-20261018", "ownerThreadId": OWNER,
        "physicalRoot": str(output), "dependencies": {"planSha256": args.plan_sha, "sourceSha256": args.self_sha},
        "retention": "Exact branch selection, deletion intents, native proofs and failed outcomes; no source copies",
        "releasedBytes": 0, "filesDeleted": 0, "sourceCopies": 0, "createdBeforeProducts": True}
    save(output / "artifacts.json", manifest)
    try:
        plan = verify_plan(args.plan_sha)
        manifest["dependencies"] = plan
        sys.path.insert(0, str(ROOT / ".jenkins/runtime/drivers" / plan["driverDigest"]))
        checks = load_module("remote_cleanup_native_checks", ROOT / "tools/jenkins-management/run_checks.py", plan["sourcePins"][str(ROOT / "tools/jenkins-management/run_checks.py")])
        manifest["runtimeCapsuleSha256"] = verify_runtime(checks, plan)
        cargo = load_module("remote_cleanup_native_cargo", ROOT / "tools/dev/local_cargo.py", plan["sourcePins"][str(ROOT / "tools/dev/local_cargo.py")])
        save(output / "artifacts.json", manifest, replace=True)
    except Exception as error:
        return record_preflight_failure(output, args, error)
    environment = dict(os.environ, PYTHONDONTWRITEBYTECODE="1", PYTHONNOUSERSITE="1")
    environment["PATH"] = str(Path(plan["gitExecutable"]).parent) + os.pathsep + environment.get("PATH", "")
    temporary = output / "tmp"; temporary.mkdir()
    environment.update(TEMP=str(temporary), TMP=str(temporary))
    if args.mode == "self-test":
        command = [sys.executable, "-B", "-X", "utf8", "-m", "unittest", "discover", "-s", "tools/jenkins-management/tests", "-p", "test_remote_branch_cleanup.py", "-v"]
    else:
        command = [sys.executable, "-B", "-X", "utf8", str(Path(__file__).absolute()), "--worker", "--mode", args.mode,
                   "--self-sha", args.self_sha, "--plan-sha", args.plan_sha, "--output", str(output),
                   "--lock-helper-sha", plan["sourcePins"][str(ROOT / "tools/jenkins-management/checkout_commit.py")]]
    native = cargo._run_owned_command(command, ROOT, environment, output, 570)
    save(output / "native-terminal.json", native)
    complete = checks.native_complete(native, command, ROOT) and all(native["processTree"]["pipes"][s]["closed"] for s in ("stdout", "stderr"))
    unchanged = all(sha(read(p)) == v for p, v in plan["sourcePins"].items())
    receipt = json.loads(read(output / "receipt.json")) if (output / "receipt.json").exists() else {"status": "self-test" if args.mode == "self-test" else "native-outcome-unknown"}
    summary = {"job": JOB, "build": int(number), "mode": args.mode, "status": receipt["status"], "receipt": receipt,
        "nativeComplete": complete, "nativeExitCode": native["rootExitCode"], "pid": native["pid"], "creationTime": native["creationTime"],
        "sourcePinsUnchanged": unchanged, "completedAtUtc": datetime.now(timezone.utc).isoformat(), "sourceCopies": 0, "filesDeleted": 0, "releasedBytes": 0}
    save(output / "summary.json", summary)
    if args.mode != "self-test":
        save(STATE / "state.json", summary, replace=True)
        if receipt.get("remoteBranchesDeleted", 0) or receipt["status"] in {"failed", "native-outcome-unknown"}:
            save(ROOT / ".jenkins/records" / f"remote-branch-cleanup-{number}.response.json", {
                "isTriggerTask": False, "triggerPermitted": False, "deleteAuthorized": False, "ownerThreadId": OWNER,
                "delivery": "durable-inbox-only", "summary": str(output / "summary.json"), "summarySha256": sha(read(output / "summary.json")),
                "status": receipt["status"], "remoteBranchesDeleted": receipt.get("remoteBranchesDeleted", 0), "formalAcceptance": False})
    print(json.dumps({"status": receipt["status"], "nativeComplete": complete, "remoteBranchesDeleted": receipt.get("remoteBranchesDeleted", 0)}))
    return 0 if complete and unchanged and native["rootExitCode"] == 0 else 1


if __name__ == "__main__":
    raise SystemExit(main())
