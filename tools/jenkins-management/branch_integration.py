"""Jenkins branch commits followed by one scheduled, serial integration queue."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import time
import argparse
import importlib.util
import sys
import tomllib
import urllib.parse
import sqlite3
import re
import subprocess
import urllib.request
import urllib.error
from contextlib import closing
import xml.etree.ElementTree as ET

JOB = "zircon-merge-queue"
CRON = "H/15 * * * *"
MODE = "branch-commit-then-scheduled-merge"
DELIVERY_SHA256 = "1ba94073123567a57563aef0d9a8f23dd3aff028c4abcd970728fbf4f8b7d4bf"
ROOT = Path(__file__).resolve().parents[2]
BEGIN = "// BEGIN ZIRCON BRANCH QUEUE"
END = "// END ZIRCON BRANCH QUEUE"


def directory(repo):
    return repo / ".jenkins/state/management/branch-integration"


def git(d, root, *arguments, check=True):
    return d.commit.git(root, "-c", "core.longpaths=true", *arguments, check=check)


def text(d, root, *arguments):
    return git(d, root, *arguments).stdout.decode("utf-8").strip()


def index_observation(d, root):
    """Resolve the real per-worktree index; a worktree's .git is a file."""
    gitdir = d.commit.physical(text(d, root, "rev-parse", "--absolute-git-dir"))
    lock = d.commit.physical(gitdir / "index.lock")
    result = {"gitDirectory": str(gitdir), "path": str(lock), "exists": lock.exists(),
              "observedAt": time.time(), "released": False}
    if lock.exists():
        stat = lock.stat()
        result.update(bytes=stat.st_size, identity=[stat.st_dev, stat.st_ino],
                      modifiedNs=stat.st_mtime_ns, sha256=hashlib.sha256(lock.read_bytes()).hexdigest(),
                      disposition="retain; original owner/native outcome must be reconciled")
    return result


def require_index(d, root):
    observation = index_observation(d, root)
    if observation["exists"]:
        error = d.commit.CommitBusy("index.lock requires original-owner reconciliation: " + observation["path"])
        error.observation = observation
        raise error
    return observation


def require_clean_main(d, repo):
    """Main is a clean synchronization checkout, never a task workspace."""
    observation = require_index(d, repo)
    if text(d, repo, "symbolic-ref", "--quiet", "--short", "HEAD") != "main":
        raise d.commit.CommitBusy("The synchronization checkout must remain on main")
    for name in ("MERGE_HEAD", "CHERRY_PICK_HEAD", "REVERT_HEAD", "rebase-merge", "rebase-apply"):
        if (Path(observation["gitDirectory"]) / name).exists():
            raise d.commit.CommitBusy("Main has an unfinished Git operation; reconcile its original owner")
    if text(d, repo, "status", "--porcelain=v1", "--untracked-files=all", "--ignore-submodules=none"):
        raise d.commit.CommitBusy("Main must be clean, including staged, working and untracked files; move task work to its managed worktree without discarding foreign edits")
    return text(d, repo, "rev-parse", "HEAD")


def github_json(d, root, repository, endpoint):
    """Read PR facts without printing credentials or enabling interactive login."""
    token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN")
    if not token:
        env = {**os.environ, "GIT_TERMINAL_PROMPT": "0", "GCM_INTERACTIVE": "never"}
        result = subprocess.run(["git", "-c", "core.fsmonitor=false", "-c", "core.hooksPath=" + os.devnull, "-c", "credential.interactive=false", "credential", "fill"],
            cwd=root, input=("protocol=https\nhost=github.com\npath=" + repository + ".git\n\n").encode(),
            stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=env, timeout=30)
        if result.returncode == 0:
            values = dict(line.split("=", 1) for line in result.stdout.decode("utf-8").splitlines() if "=" in line)
            token = values.get("password")
    headers = {"Accept": "application/vnd.github+json", "User-Agent": "Zircon-Jenkins-PR-gate"}
    if token:
        headers["Authorization"] = "Bearer " + token
    request = urllib.request.Request("https://api.github.com/repos/" + repository + "/" + endpoint, headers=headers)
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            return json.load(response)
    except urllib.error.HTTPError as error:
        raise d.commit.CommitBusy("GitHub PR observation unavailable (HTTP " + str(error.code) + "); retain the original queue entry") from None
    except (urllib.error.URLError, TimeoutError):
        raise d.commit.CommitBusy("GitHub PR observation did not complete; reconcile this original queue entry") from None


def verified_pr_commit(d, repo, root, record, policy):
    """Only a real merged PR whose result tree was tested may advance main."""
    rules = policy.get("integration", {})
    if rules.get("requirePullRequest") is not True:
        raise ValueError("PR integration must be enabled; direct candidate publication is disabled")
    repository = rules.get("pullRequestRepository")
    if not isinstance(repository, str) or not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repository):
        raise ValueError("Configure the exact GitHub PR repository")
    remote = text(d, root, "remote", "get-url", "origin")
    if remote not in {"https://github.com/" + repository, "https://github.com/" + repository + ".git",
                      "git@github.com:" + repository, "git@github.com:" + repository + ".git"}:
        raise ValueError("Origin differs from the authorized PR repository")
    head = repository.split("/", 1)[0] + ":" + record["branch"]
    query = urllib.parse.urlencode({"state": "all", "base": "main", "head": head, "sort": "updated", "direction": "desc", "per_page": 100})
    pulls = github_json(d, root, repository, "pulls?" + query)
    matches = [p for p in pulls if p.get("head", {}).get("sha") == record["sourceCommit"]
               and p.get("head", {}).get("ref") == record["branch"]
               and p.get("head", {}).get("repo", {}).get("full_name") == repository
               and p.get("base", {}).get("ref") == "main"
               and p.get("base", {}).get("repo", {}).get("full_name") == repository]
    if len(matches) > 1:
        raise ValueError("Multiple matching PRs require original-owner reconciliation")
    if not matches:
        return {"status": "awaiting-pr", "reason": "Publish the task branch and open its PR; advisory validation does not block delivery"}
    number = matches[0].get("number")
    if not isinstance(number, int) or number < 1:
        raise ValueError("GitHub returned an invalid PR identity")
    pull = github_json(d, root, repository, "pulls/" + str(number))
    if (pull.get("head", {}).get("sha") != record["sourceCommit"]
            or pull.get("head", {}).get("ref") != record["branch"]
            or pull.get("head", {}).get("repo", {}).get("full_name") != repository
            or pull.get("base", {}).get("ref") != "main"
            or pull.get("base", {}).get("repo", {}).get("full_name") != repository):
        raise ValueError("PR source or destination changed during observation")
    receipt = {"number": number, "url": "https://github.com/" + repository + "/pull/" + str(number),
               "sourceCommit": record["sourceCommit"], "observedAt": time.time()}
    if pull.get("merged") is not True or pull.get("state") != "closed":
        return {**receipt, "status": "awaiting-pr", "reason": "The matching PR has not been merged"}
    revision = pull.get("merge_commit_sha")
    if not isinstance(revision, str) or not re.fullmatch(r"[0-9a-f]{40}", revision):
        raise ValueError("Merged PR has no immutable Git result")
    git(d, root, "fetch", "--no-tags", "origin", "refs/heads/main:refs/remotes/origin/main")
    if git(d, root, "merge-base", "--is-ancestor", revision, "refs/remotes/origin/main", check=False).returncode:
        raise d.commit.CommitBusy("PR merge is not present on origin/main")
    if git(d, root, "merge-base", "--is-ancestor", revision, record["mainHeadBefore"], check=False).returncode == 0:
        if text(d, root, "rev-parse", record["candidateHead"] + "^{tree}") == text(d, root, "rev-parse", record["mainHeadBefore"] + "^{tree}"):
            return {**receipt, "status": "merged", "mergeCommit": record["mainHeadBefore"], "prMergeCommit": revision,
                    "alreadySynchronized": True}
    if git(d, root, "merge-base", "--is-ancestor", record["mainHeadBefore"], revision, check=False).returncode:
        raise d.commit.CommitBusy("Merged PR is not a fast-forward of local main on origin/main; reconcile the local baseline")
    if (not d.advisory_validation(policy)
            and text(d, root, "rev-parse", revision + "^{tree}") != text(d, root, "rev-parse", record["candidateHead"] + "^{tree}")):
        raise d.commit.CommitBusy("The actual PR merge tree differs from the validated candidate; validate the new result before main synchronization")
    return {**receipt, "status": "merged", "mergeCommit": revision}


def remember_failure(d, record, error):
    record.update(status="waiting" if isinstance(error, d.commit.CommitBusy) else "failed",
                  reason=str(error), failedAt=time.time())
    if hasattr(error, "observation"):
        record["indexLock"] = error.observation


def owner_notice(d, repo, record):
    """Durable inbox handoff; never claim that another chat has read it."""
    owner = record["request"].get("ownerThreadId")
    if not owner:
        return
    repair = record.get("repairRequired", False)
    identity = {"ownerThreadId": owner, "operationId": record["request"]["operationId"],
                "status": "repair-required" if repair else record.get("integrationStatus", record["status"]),
                "reason": record.get("validationWarning") if repair else record.get("reason")}
    digest = hashlib.sha256(json.dumps(identity, sort_keys=True).encode()).hexdigest()
    if record.get("notification", {}).get("digest") == digest:
        return
    destination = repo / ".jenkins/records" / ("merge-queue-" + record["request"]["operationId"] + "-" + digest[:12] + ".json")
    d.commit.write_json(destination, {"kind": "jenkins-worktree-followup", **identity,
        "sourceCommit": record.get("sourceCommit"), "sourceRoot": record.get("sourceRoot"),
        "jenkinsBuildUrl": os.environ.get("BUILD_URL"), "validation": record.get("candidateValidation"),
        "request": ("Continue the original task, read the retained failing validation logs, fix the scoped source and commit the correction immediately. "
                    "Use a new correction operation linked by repairsOperationId; no user approval or passing validation is required. "
                    "Do not replay an unknown commit result."),
        "repairRequired": repair, "repairOf": record["request"]["operationId"],
        "formalAcceptance": False, "recipientRead": False})
    record["notification"] = {"digest": digest, "path": str(destination), "status": "inbox-written", "recipientRead": False}


def commit_advisory(d, repo, root, payload, record, previous, paths, path):
    """Use native Git's scoped commit; no source hash or test gate precedes it."""
    before = text(d, root, "rev-parse", "HEAD")
    trailer = "Zircon-Branch-Operation: " + payload["operationId"]
    if previous and previous.get("headBefore"):
        found = text(d, root, "log", "-100", "--format=%H", "--fixed-strings", "--grep=" + trailer)
        matches = [revision for revision in found.splitlines()
                   if text(d, root, "show", "-s", "--format=%P", revision) == previous["headBefore"]
                   and trailer in text(d, root, "show", "-s", "--format=%B", revision).splitlines()
                   and d.changed_paths(root, "diff", "--name-only", "-z", previous["headBefore"], revision, "--") <= set(paths)]
        if len(matches) > 1:
            raise ValueError("Multiple original task commits require reconciliation")
        if matches:
            record.update(status="committed", sourceCommit=matches[0], integrationStatus="pending", recoveredReceipt=True)
            return record
    tracked = {x.decode("utf-8") for x in git(d, root, "ls-files", "-z", "--", *paths).stdout.split(b"\0") if x}
    selected = [name for name in paths if name in tracked or (root / name).is_file()]
    record.update(headBefore=before, commitPaths=selected,
                  expectedAbsences=[name for name in paths if name not in selected],
                  status="committing", validationMode="advisory", sourceHashVerification=False)
    d.commit.write_json(path, record)
    if selected:
        dirty = git(d, root, "status", "--porcelain=v1", "-z", "--untracked-files=all", "--", *selected).stdout
        changed = bool(dirty) or any(name not in tracked for name in selected)
        if changed:
            for force, names in [(False, [n for n in selected if not n.startswith(".codex/skills/")]),
                                 (True, [n for n in selected if n.startswith(".codex/skills/")])]:
                if names:
                    git(d, root, "add", *(["--force"] if force else []), "--", *names)
            hooks = repo / ".jenkins/state/management/empty-commit-hooks"
            hooks.mkdir(parents=True, exist_ok=True)
            if any(hooks.iterdir()):
                raise ValueError("Managed empty commit hooks directory changed")
            git(d, root, "-c", "core.hooksPath=" + str(hooks), "-c", "commit.gpgSign=false",
                "commit", "--only", "--no-gpg-sign", "-m", payload["message"], "-m", trailer, "--", *selected)
        revision = text(d, root, "rev-parse", "HEAD")
        if changed and not d.changed_paths(root, "diff", "--name-only", "-z", before, revision, "--") <= set(paths):
            raise ValueError("Task commit includes undeclared files; retain its original receipt")
    else:
        revision = before
    record.update(status="committed", sourceCommit=revision, integrationStatus="pending", committedAt=time.time())
    return record


def stage(d, repo, payload, *, policy, grants, runtime=None, check_runtime=lambda: None):
    """Commit only the task paths. Validation belongs to the later merge job."""
    paths, _ = d.validate_request(repo, payload, policy, grants)
    path = directory(repo) / "requests" / (payload["operationId"] + ".json")
    previous = d.commit.read_json(path) if path.exists() else None
    if previous and previous["request"] != payload:
        raise ValueError("Original branch delivery identity has different input")
    if previous and previous["status"] in {"committed", "merged"}:
        return {**previous, "reusedReceipt": True}
    record = previous or {"schemaVersion": 1, "request": payload, "status": "preflight",
        "startedAt": time.time(), "runtime": runtime, "pushPerformed": False,
        "formalAcceptance": False, "validationPerformed": False, "mainAdvanced": False}
    d.commit.write_json(path, record)
    try:
        root = d.source_root(repo, payload)
        record["indexLock"] = index_observation(d, root)
        if root == repo:
            raise ValueError("Commit to the task's Jenkins-managed worktree branch first; main is published only by the scheduled merge queue")
        with closing(sqlite3.connect((repo / ".jenkins/state/coordination.sqlite3").as_uri() + "?mode=ro", uri=True, timeout=5)) as connection:
            owners = [json.loads(row[0]) for row in connection.execute("SELECT payload FROM records WHERE domain='management_worktree'")]
        owner = next((value for value in owners if d.commit.physical(value.get("path", value.get("root", "__missing__"))) == root
                      and value.get("status") == "ready" and value.get("sessionId") == payload["sessionId"]), None)
        if owner is None:
            raise ValueError("No original ready worktree ownership matches this task; retain the source candidate")
        if runtime and not d.advisory_validation(policy) and owner.get("driverDigest") != runtime.get("driverDigest"):
            raise ValueError("Historical worktree owner belongs to another driver; reconcile its original operation before current-runtime adoption")
        if not all(any(name == scope or name.startswith(scope.rstrip("/") + "/") for scope in owner.get("ownedPaths", [])) for name in paths):
            raise ValueError("Task commit exceeds its original worktree ownership; reconcile the owner scope")
        record.update(sourceRoot=str(root), branch=text(d, root, "symbolic-ref", "--short", "HEAD"))
        with d.commit.git_lock(repo / ".jenkins/state/locks/management-git.lock"):
            check_runtime()
            if policy.get("integration", {}).get("blockCommitOnDirtyMain", True):
                require_clean_main(d, repo)
            record["indexLock"] = require_index(d, root)
            if git(d, root, "rev-parse", "--verify", "MERGE_HEAD", check=False).returncode == 0:
                raise ValueError("Retain the existing unfinished merge; this task did not start it")
            if d.advisory_validation(policy):
                record = commit_advisory(d, repo, root, payload, record, previous, paths, path)
                d.commit.write_json(path, record)
                return record
            before = text(d, root, "rev-parse", "HEAD")
            trailer = "Zircon-Branch-Operation: " + payload["operationId"]
            if previous and previous.get("headBefore"):
                found = text(d, root, "log", "-100", "--format=%H", "--fixed-strings", "--grep=" + trailer)
                matches = []
                for revision in found.splitlines():
                    if (text(d, root, "show", "-s", "--format=%P", revision) == previous["headBefore"]
                            and trailer in text(d, root, "show", "-s", "--format=%B", revision).splitlines()):
                        matches.append(revision)
                if len(matches) > 1:
                    raise ValueError("Multiple original task commits require reconciliation")
                if matches:
                    if (not previous.get("expectedGitObjects")
                            or d.committed_objects(root, matches[0], previous["commitPaths"]) != previous["expectedGitObjects"]
                            or not d.changed_paths(root, "diff", "--name-only", "-z", previous["headBefore"], matches[0], "--") <= set(previous["commitPaths"])):
                        raise ValueError("Original interrupted commit differs from its recorded task objects; retain it for reconciliation")
                    record.update(status="committed", sourceCommit=matches[0], integrationStatus="pending", recoveredReceipt=True)
                    d.commit.write_json(path, record)
                    return record
                if before != previous["headBefore"]:
                    raise ValueError("The interrupted branch commit has an unexpected HEAD; retain the original request")
            hashes = d.files(root, paths)
            supplied = payload.get("expectedFileHashes")
            if supplied is not None and supplied != hashes:
                raise ValueError("Selected task bytes changed from the submitted hashes")
            tracked = {x.decode("utf-8") for x in git(d, root, "ls-files", "-z", "--", *paths).stdout.split(b"\0") if x}
            selected = [name for name in paths if name in tracked or hashes[name] is not None]
            record.update(headBefore=before, files=hashes, commitPaths=selected,
                          expectedAbsences=[name for name in paths if name not in selected], status="committing")
            d.commit.write_json(path, record)
            if selected:
                with d.commit.verified_commit_files(root, {n: h for n, h in hashes.items() if h is not None}):
                    expected = {n: text(d, root, "hash-object", "--path=" + n, "--", n) if hashes[n] is not None else None for n in selected}
                    record["expectedGitObjects"] = expected
                    d.commit.write_json(path, record)
                    dirty = git(d, root, "status", "--porcelain=v1", "-z", "--untracked-files=all", "--", *selected).stdout
                    changed = bool(dirty) or any(n not in tracked and hashes[n] is not None for n in selected)
                    if changed:
                        ordinary = [n for n in selected if not n.startswith(".codex/skills/")]
                        skills = [n for n in selected if n.startswith(".codex/skills/")]
                        if ordinary:
                            git(d, root, "add", "--", *ordinary)
                        if skills:
                            git(d, root, "add", "--force", "--", *skills)
                        hooks = repo / ".jenkins/state/management/empty-commit-hooks"
                        hooks.mkdir(parents=True, exist_ok=True)
                        if any(hooks.iterdir()):
                            raise ValueError("Managed empty commit hooks directory changed")
                        git(d, root, "-c", "core.hooksPath=" + str(hooks), "-c", "commit.gpgSign=false",
                            "commit", "--only", "--no-gpg-sign", "-m", payload["message"], "-m", trailer, "--", *selected)
                    revision = text(d, root, "rev-parse", "HEAD")
                    if d.committed_objects(root, revision, selected) != expected:
                        raise ValueError("Task commit objects differ from the pinned task files")
                    if changed:
                        actual = d.changed_paths(root, "diff", "--name-only", "-z", before, revision, "--")
                        if not actual <= set(selected):
                            raise ValueError("Task commit includes undeclared files; retain its original receipt")
            else:
                revision = before
            record.update(status="committed", sourceCommit=revision, integrationStatus="pending", committedAt=time.time())
            if d.files(root, paths) != hashes:
                raise ValueError("A task path changed during its branch commit; retain the original commit")
    except Exception as error:
        remember_failure(d, record, error)
        owner_notice(d, repo, record)
    d.commit.write_json(path, record)
    return record


def clean_candidate(d, repo, main, record):
    root = repo / ".jenkins/worktrees/integration-queue"
    marker = directory(repo) / "candidate.json"
    if root.exists():
        known = d.commit.read_json(marker) if marker.exists() else {}
        if known.get("path") != str(root) or known.get("status") != "idle":
            raise d.commit.CommitBusy("Retain the existing candidate; its original native/merge result needs reconciliation")
        common = d.commit.physical(text(d, root, "rev-parse", "--path-format=absolute", "--git-common-dir"))
        if common != d.commit.physical(repo / ".git"):
            raise ValueError("Candidate Git identity changed")
        require_index(d, root)
        if text(d, root, "status", "--porcelain=v1", "--untracked-files=all"):
            raise ValueError("Retain unexpected candidate changes; no automatic reset or clean")
        if not (record.get("integrationStatus") == "awaiting-pr" and record.get("mainHeadBefore") == main
                and text(d, root, "rev-parse", "HEAD") == record.get("candidateHead")):
            git(d, root, "checkout", "--detach", main)
    else:
        d.commit.write_json(marker, {"path": str(root), "status": "creating", "operationId": record["request"]["operationId"]})
        git(d, repo, "worktree", "add", "--detach", str(root), main)
    d.commit.write_json(marker, {"path": str(root), "status": "active", "operationId": record["request"]["operationId"], "mainHead": main})
    return root, marker


def integrate(d, repo, path, *, runtime, check_runtime, validate=None, pull_request=None):
    record = d.commit.read_json(path)
    payload, revision = record["request"], record["sourceCommit"]
    validate = d.run_validation if validate is None else validate
    pull_request = verified_pr_commit if pull_request is None else pull_request
    candidate_marker = None
    was_awaiting_pr = record.get("integrationStatus") == "awaiting-pr"
    try:
        policy = d.policy_for(repo)
        advisory = d.advisory_validation(policy)
        check_runtime()
        with d.commit.git_lock(repo / ".jenkins/state/locks/management-git.lock"):
            record["mainIndexLock"] = index_observation(d, repo)
            if text(d, repo, "symbolic-ref", "--quiet", "--short", "HEAD") != "main":
                raise d.commit.CommitBusy("The synchronization checkout must remain on main")
            main = text(d, repo, "rev-parse", "--verify", "HEAD^{commit}")
            base = text(d, repo, "merge-base", main, revision)
            authored = d.changed_paths(repo, "diff", "--name-only", "-z", base, revision, "--")
            if not authored <= set(payload["paths"]):
                raise ValueError("Recorded feature history includes another task's files; owner must provide a scoped branch")
            root, candidate_marker = clean_candidate(d, repo, main, record)
            record.update(integrationStatus="merging-candidate", mainHeadBefore=main, candidateRoot=str(root))
            d.commit.write_json(path, record)
            hooks = repo / ".jenkins/state/management/empty-commit-hooks"
            hooks.mkdir(parents=True, exist_ok=True)
            if any(hooks.iterdir()):
                raise ValueError("Managed empty commit hooks directory changed")
            result = git(d, root, "-c", "core.hooksPath=" + str(hooks), "-c", "commit.gpgSign=false",
                         "merge", "--no-edit", "--no-gpg-sign", revision, check=False)
            if result.returncode:
                record.update(integrationStatus="merge-conflict", reason=(result.stdout + result.stderr).decode("utf-8", "replace")[-4000:])
                if git(d, root, "rev-parse", "--verify", "MERGE_HEAD", check=False).returncode == 0:
                    # Only the clean queue-owned merge begun above is aborted.
                    aborted = git(d, root, "merge", "--abort", check=False)
                    record["ownedMergeAborted"] = aborted.returncode == 0
                if not text(d, root, "status", "--porcelain=v1", "--untracked-files=all"):
                    d.commit.write_json(candidate_marker, {"path": str(root), "status": "idle"})
                owner_notice(d, repo, record)
                d.commit.write_json(path, record)
                return record
            candidate = text(d, root, "rev-parse", "HEAD")
        attempt = record.get("integrationAttempt", -1) + 1
        record.update(integrationAttempt=attempt, candidateHead=candidate, integrationStatus="validating")
        d.commit.write_json(path, record)
        _, names = d.validate_request(repo, payload, policy,
            tomllib.loads((repo / ".codex/config.toml").read_text("utf-8-sig"))["permissions"]["zircon-strict"]["filesystem"])
        cached = record.get("candidateValidation", {})
        validation_key = {"tree": text(d, root, "rev-parse", "HEAD^{tree}"),
                          "runtime": runtime, "recipes": payload.get("validation", []), "files": d.files(root, names)}
        if advisory and not payload.get("validation"):
            validation = {"status": "skipped", "head": candidate, "files": d.files(root, names),
                          "steps": [], "runtime": runtime, "formalAcceptance": False,
                          "reason": "Validation is optional; the session supplied no checks"}
            record["validationReused"] = False
        elif (was_awaiting_pr and cached.get("status") in ({"passed", "failed"} if advisory else {"passed"})
                and record.get("candidateValidationKey") == validation_key
                and cached.get("runtime") == runtime and cached.get("files") == validation_key["files"]
                and (advisory or len(cached.get("steps", [])) == len(payload.get("validation", [])))
                and all(s.get("complete") is True and (advisory or s.get("status") == "passed") for s in cached["steps"])):
            validation = cached
            record["validationReused"] = True
        else:
            validation = validate(root, payload, names, d.OUTPUT / payload["operationId"] / ("integration-" + str(attempt)), runtime)
            record["validationReused"] = False
        record["candidateValidationKey"] = validation_key
        record["candidateValidation"] = validation
        record["validationMode"] = "advisory" if advisory else "required"
        if advisory and validation["status"] not in {"passed", "skipped"}:
            record.update(repairRequired=True,
                          validationWarning="Optional validation failed; keep the commit, continue PR integration and let the original session commit a correction")
            owner_notice(d, repo, record)
        if advisory and any(step.get("complete") is not True for step in validation.get("steps", [])):
            raise d.commit.CommitBusy("Validation still has an active or unknown process; retain its candidate until the process exits")
        if ((not advisory and validation["status"] != "passed")
                or any((root / n).exists() for n in record.get("expectedAbsences", []))):
            record.update(integrationStatus="validation-failed", reason="Actual merged candidate validation failed; retain the branch commit")
        else:
            publication = pull_request(d, repo, root, record, policy)
            record["pullRequest"] = publication
            if publication["status"] != "merged":
                record.update(integrationStatus="awaiting-pr", reason=publication.get("reason"), mainAdvanced=False)
                if not text(d, root, "status", "--porcelain=v1", "--untracked-files=all"):
                    d.commit.write_json(candidate_marker, {"path": str(root), "status": "idle"})
                d.commit.write_json(path, record)
                return record
            with d.commit.git_lock(repo / ".jenkins/state/locks/management-git.lock"):
                check_runtime()
                require_index(d, repo)
                require_index(d, root)
                if (require_clean_main(d, repo) != main
                        or text(d, root, "rev-parse", "HEAD") != candidate
                        or d.files(root, names) != validation["files"] or d.active_reader(repo)
                        or text(d, root, "status", "--porcelain=v1", "--untracked-files=all")):
                    raise d.commit.CommitBusy("Main/candidate or an active source reader changed; retry this original queue entry")
                result = git(d, repo, "-c", "core.hooksPath=" + str(hooks), "merge", "--ff-only", publication["mergeCommit"], check=False)
                if result.returncode:
                    raise d.commit.CommitBusy(result.stderr.decode("utf-8", "replace")[-2500:])
                record.update(status="merged", integrationStatus="complete", mainAdvanced=True,
                              mainHead=text(d, repo, "rev-parse", "HEAD"), mergedAt=time.time())
                require_clean_main(d, repo)
        if not text(d, root, "status", "--porcelain=v1", "--untracked-files=all"):
            d.commit.write_json(candidate_marker, {"path": str(root), "status": "idle"})
    except Exception as error:
        record.update(integrationStatus="waiting" if isinstance(error, d.commit.CommitBusy) else "failed", reason=str(error))
        if hasattr(error, "observation"):
            record["mainIndexLock"] = error.observation
        validation = record.get("candidateValidation", {})
        if (candidate_marker is not None and validation.get("status") in {"passed", "failed", "skipped"}
                and all(step.get("complete") is True for step in validation.get("steps", []))
                and git(d, root, "rev-parse", "--verify", "MERGE_HEAD", check=False).returncode != 0
                and not text(d, root, "status", "--porcelain=v1", "--untracked-files=all")):
            d.commit.write_json(candidate_marker, {"path": str(root), "status": "idle"})
    if record.get("integrationStatus") != "complete":
        owner_notice(d, repo, record)
    d.commit.write_json(path, record)
    return record


# Strict profiles retain their broad origin recovery checks. Ordinary task
# delivery uses the user's optional validation policy below.
ORIGIN_MAIN_VALIDATION = [
    {"kind": "cargo", "arguments": [command, *manifest, "--workspace", *targets, "--locked", *lint]}
    for manifest in ([], ["--manifest-path", "zircon_plugins/Cargo.toml"])
    for command, targets, lint in (("check", ["--all-targets"], []), ("test", [], []),
                                   ("clippy", ["--all-targets"], ["--", "-D", "warnings"]))
]


def sync_origin_main(d, repo, *, runtime, check_runtime, validate=None):
    """Validate the exact fetched main tree before a clean local fast-forward."""
    validate = d.run_validation if validate is None else validate
    policy = d.policy_for(repo)
    advisory = d.advisory_validation(policy)
    recipes = [] if advisory else ORIGIN_MAIN_VALIDATION
    repository = policy.get("integration", {}).get("pullRequestRepository")
    if not isinstance(repository, str) or not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repository):
        raise ValueError("Origin synchronization requires the authorized GitHub repository")
    if text(d, repo, "remote", "get-url", "origin") not in {
        "https://github.com/" + repository, "https://github.com/" + repository + ".git",
        "git@github.com:" + repository, "git@github.com:" + repository + ".git",
    }:
        raise ValueError("Origin differs from the authorized PR repository")
    with d.commit.git_lock(repo / ".jenkins/state/locks/management-git.lock"):
        check_runtime()
        require_index(d, repo)
        before = text(d, repo, "rev-parse", "HEAD")
        git(d, repo, "fetch", "--no-tags", "origin", "refs/heads/main:refs/remotes/origin/main")
        revision = text(d, repo, "rev-parse", "refs/remotes/origin/main")
        if revision == before:
            return {"status": "current", "head": before}
        require_clean_main(d, repo)
        if git(d, repo, "merge-base", "--is-ancestor", before, revision, check=False).returncode:
            raise d.commit.CommitBusy("Local main diverges from origin/main; retain both histories")
        operation = "origin-main-" + revision
        receipt = directory(repo) / "origin-sync" / (revision + ".json")
        previous = d.commit.read_json(receipt) if receipt.exists() else {}
        if previous.get("status") in {"validating", "unknown"}:
            raise d.commit.CommitBusy("Reconcile the original origin/main validation before retrying")
        attempt = previous.get("attempt", -1) + 1
        record = {"request": {"operationId": operation}, "status": "validating", "attempt": attempt,
                  "before": before, "revision": revision, "runtime": runtime,
                  "previousAttempts": [*previous.get("previousAttempts", []),
                      *([ {k: v for k, v in previous.items() if k != "previousAttempts"} ] if previous else [])]}
        root, marker = clean_candidate(d, repo, revision, record)
        names = sorted(d.changed_paths(repo, "diff", "--name-only", "-z", before, revision, "--"))
        record["tree"] = text(d, root, "rev-parse", "HEAD^{tree}")
        d.commit.write_json(receipt, record)
    payload = {"operationId": operation, "validation": recipes}
    validation = validate(root, payload, names, d.OUTPUT / operation / ("attempt-" + str(attempt)), runtime)
    record["validation"] = validation
    complete = (len(validation.get("steps", [])) == len(recipes)
                and all(step.get("complete") is True and step.get("status") == "passed" for step in validation["steps"]))
    accepted = validation.get("status") in ({"passed", "skipped"} if advisory else {"passed"})
    record["validationMode"] = "advisory" if advisory else "required"
    record["status"] = "validated" if accepted and complete else "validation-failed"
    d.commit.write_json(receipt, record)
    if record["status"] != "validated":
        # Incomplete native evidence must be reconciled, never overwritten.
        if any(step.get("complete") is not True for step in validation.get("steps", [])):
            record["status"] = "unknown"
        elif not text(d, root, "status", "--porcelain=v1", "--untracked-files=all"):
            d.commit.write_json(marker, {"path": str(root), "status": "idle"})
        d.commit.write_json(receipt, record)
        raise d.commit.CommitBusy("Fetched origin/main failed its full validation; local main retained")
    with d.commit.git_lock(repo / ".jenkins/state/locks/management-git.lock"):
        check_runtime()
        require_index(d, root)
        if (require_clean_main(d, repo) != before or text(d, root, "rev-parse", "HEAD") != revision
                or (not advisory and d.files(root, names) != validation["files"]) or d.active_reader(repo)
                or text(d, root, "status", "--porcelain=v1", "--untracked-files=all")):
            raise d.commit.CommitBusy("Main/candidate changed during origin validation; retain original receipt")
        git(d, repo, "merge", "--ff-only", revision)
        require_clean_main(d, repo)
        record.update(status="synchronized", synchronizedAt=time.time())
        d.commit.write_json(receipt, record)
        d.commit.write_json(marker, {"path": str(root), "status": "idle"})
    return record


def drain(d, repo, *, runtime, check_runtime):
    results = []
    with d.commit.git_lock(directory(repo) / "drain.lock"):
        try:
            synchronization = sync_origin_main(d, repo, runtime=runtime, check_runtime=check_runtime)
        except d.commit.CommitBusy as error:
            if not d.advisory_validation(d.policy_for(repo)):
                raise
            synchronization = {"status": "waiting", "reason": str(error), "foreignChangesPreserved": True}
        for path in sorted((directory(repo) / "requests").glob("*.json"), key=lambda p: p.stat().st_mtime):
            record = d.commit.read_json(path)
            states = {"pending", "waiting", "awaiting-pr"}
            if d.advisory_validation(d.policy_for(repo)):
                states.add("validation-failed")
            if record.get("status") != "committed" or record.get("integrationStatus") not in states:
                continue
            record = integrate(d, repo, path, runtime=runtime, check_runtime=check_runtime)
            results.append({k: record.get(k) for k in ("sourceCommit", "status", "integrationStatus", "mainHead", "reason", "notification")})
            if len(results) >= 8:
                break
    return {"status": "complete" if all(v["integrationStatus"] in {"complete", "awaiting-pr"} for v in results) else "pending",
            "originMain": synchronization, "job": JOB, "cron": CRON, "results": results, "observedAt": time.time(), "formalAcceptance": False}


def install(d, repo, client, runtime):
    script = (repo / "tools/jenkins-management/zircon-merge-queue.groovy").read_text("utf-8-sig")
    for name, value in runtime.items():
        script = script.replace("@@" + name + "@@", value)
    status, raw, _ = client.request("/job/" + JOB + "/config.xml")
    if status == 404:
        doc = ET.Element("flow-definition", {"plugin": "workflow-job"})
        ET.SubElement(doc, "description").text = "Commit task branches first; one scheduled queue validates and integrates immutable commits into local main."
        ET.SubElement(doc, "keepDependencies").text = "false"
        props = ET.SubElement(doc, "properties")
        ET.SubElement(props, "org.jenkinsci.plugins.workflow.job.properties.DisableConcurrentBuildsJobProperty")
        triggers = ET.SubElement(ET.SubElement(props, "org.jenkinsci.plugins.workflow.job.properties.PipelineTriggersJobProperty"), "triggers")
        ET.SubElement(ET.SubElement(triggers, "hudson.triggers.TimerTrigger"), "spec").text = CRON
        definition = ET.SubElement(doc, "definition", {"class": "org.jenkinsci.plugins.workflow.cps.CpsFlowDefinition", "plugin": "workflow-cps"})
        ET.SubElement(definition, "script").text = script
        ET.SubElement(definition, "sandbox").text = "true"
        ET.SubElement(doc, "disabled").text = "false"
        endpoint = "/createItem?name=" + JOB
    elif status == 200:
        doc = ET.fromstring(raw)
        if not (doc.findtext("definition/script") or "").startswith("// ZIRCON SCHEDULED BRANCH INTEGRATION"):
            raise ValueError("Existing merge job has another owner; preserve its configuration")
        doc.find("definition/script").text = script
        if doc.findtext(".//hudson.triggers.TimerTrigger/spec") != CRON:
            raise ValueError("Existing integration schedule differs; preserve its owner's schedule")
        endpoint = "/job/" + JOB + "/config.xml"
    else:
        raise ValueError("Scheduled merge job observation unavailable")
    status, _, _ = client.request(endpoint, method="POST", data=ET.tostring(doc, encoding="utf-8"), content_type="application/xml")
    if status not in (200, 201, 302):
        raise ValueError("Scheduled integration installation failed")
    status, raw, _ = client.request("/job/" + JOB + "/config.xml")
    observed = ET.fromstring(raw) if status == 200 else None
    if (observed is None or observed.findtext("definition/script") != script
            or observed.findtext(".//hudson.triggers.TimerTrigger/spec") != CRON
            or observed.findtext("disabled") != "false"):
        raise ValueError("Scheduled integration lacks exact API read-back")
    result = {"status": "installed", "job": JOB, "cron": CRON, "runtime": runtime,
              "configSha256": hashlib.sha256(raw).hexdigest(), "actualMergeAccepted": False}
    d.commit.write_json(directory(repo) / "installed.json", result)
    return result


def install_entry(d, repo, client, *, reuse_existing=False):
    endpoint = "/job/zircon-management/config.xml"
    status, raw, _ = client.request(endpoint)
    if status != 200:
        raise ValueError("Management job unavailable")
    doc = ET.fromstring(raw)
    script = doc.find("definition/script")
    if script is None or "--capsule" not in script.text or "zircon-windows-agent" not in script.text:
        raise ValueError("Management core identity changed")
    old = script.text
    if BEGIN in old or END in old:
        if old.count(BEGIN) != 1 or old.count(END) != 1 or old.index(BEGIN) >= old.index(END):
            raise ValueError("Branch entry markers are incomplete or ambiguous")
        if reuse_existing:
            return {"status": "installed", "job": "zircon-management",
                    "actions": ["commit-branch", "check-branch-queue"], "reused": True}
        start, end = old.index(BEGIN), old.index(END) + len(END)
        old = old[:start] + old[end:].lstrip("\r\n")
    script.text = (repo / "tools/jenkins-management/branch-integration.groovy").read_text("utf-8-sig").rstrip() + "\n\n" + old
    choices = doc.find(".//parameterDefinitions/*[name='ACTION']/choices/a")
    if choices is not None:
        for action in ("commit-branch", "check-branch-queue"):
            if action not in [e.text for e in choices]:
                ET.SubElement(choices, "string").text = action
    status, _, _ = client.request(endpoint, method="POST", data=ET.tostring(doc, encoding="utf-8"), content_type="application/xml")
    if status not in (200, 201, 302):
        raise ValueError("Branch entry installation failed")
    status, after, _ = client.request(endpoint)
    if status != 200 or ET.fromstring(after).findtext("definition/script") != script.text:
        raise ValueError("Branch entry lacks exact API read-back")
    return {"status": "installed", "job": "zircon-management", "actions": ["commit-branch", "check-branch-queue"]}


def load_delivery():
    delivery_path = Path(__file__).with_name("delivery.py")
    delivery_bytes = delivery_path.read_bytes()
    policy = json.loads((ROOT / "tools/jenkins-management/local-commit-policy.json").read_text("utf-8-sig"))
    advisory = policy.get("integration", {}).get("validationMode") == "advisory"
    if not advisory and hashlib.sha256(delivery_bytes).hexdigest() != DELIVERY_SHA256:
        raise ValueError("Branch delivery implementation differs from its reviewed digest")
    spec = importlib.util.spec_from_file_location("branch_queue_delivery", delivery_path)
    d = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = d
    exec(compile(delivery_bytes, str(delivery_path), "exec"), d.__dict__)
    return d


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("submit", "query", "install", "install-entry", "run", "drain"))
    parser.add_argument("--input-file", type=Path)
    parser.add_argument("--output-file", type=Path)
    parser.add_argument("--operation-id")
    parser.add_argument("--validate-only", action="store_true")
    parser.add_argument("--resume", action="store_true")
    args = parser.parse_args(argv)
    d = load_delivery()
    result = None
    try:
        if args.command == "query":
            if not args.operation_id or not d.commit.ID.fullmatch(args.operation_id):
                raise ValueError("Query requires the original operation ID")
            path = directory(ROOT) / "requests" / (args.operation_id + ".json")
            if not path.exists():
                path = ROOT / ".jenkins/state/management/deliveries" / (args.operation_id + ".json")
            result = d.commit.read_json(path) if path.exists() else {"status": "not-started"}
        else:
            context = d.runtime_context(ROOT)
            config, client = context["config"], d.commit.transport(ROOT)
            runtime = {k: config[k] for k in ("driverDigest", "runtimeOperationId")}
            policy = d.policy_for(ROOT)
            grants = tomllib.loads((ROOT / ".codex/config.toml").read_text("utf-8-sig"))["permissions"]["zircon-strict"]["filesystem"]
            def current_runtime():
                selected = d.commit.read_json(ROOT / ".jenkins/state/deployment/driver.json")
                operation = d.commit.read_json(ROOT / ".jenkins/state/deployment-operation.json")
                if (any(selected[k] != config[k] for k in runtime) or operation.get("operationId") != config["runtimeOperationId"]
                        or operation.get("state") != "running"):
                    raise ValueError("Selected runtime changed during branch integration")
                if not context.get("advisory"):
                    from tools.jenkins.deployment.drain_observations import owner_observation, quiet_observation
                    owner_observation(context["manager"], expected=context["owner"])
                    if quiet_observation(context["manager"]):
                        raise ValueError("Selected runtime is draining")
            if args.command in {"install", "install-entry"}:
                result = install_entry(d, ROOT, client)
                if args.command == "install":
                    result["integration"] = install(d, ROOT, client, runtime)
            elif args.command == "drain":
                if (os.environ.get("JOB_NAME") != JOB or os.environ.get("NODE_NAME") != "zircon-windows-agent"
                        or os.environ.get("ZR_INTEGRATION_DRIVER") != runtime["driverDigest"]
                        or os.environ.get("ZR_INTEGRATION_RUNTIME") != runtime["runtimeOperationId"]):
                    raise ValueError("Only the real runtime-bound scheduled Jenkins job may publish main")
                os.environ["ZR_COMMIT_NATIVE"] = "1"
                result = drain(d, ROOT, runtime=runtime, check_runtime=current_runtime)
            else:
                payload = d.commit.read_json(args.input_file)
                _, names = d.validate_request(ROOT, payload, policy, grants)
                if args.command == "run":
                    if os.environ.get("JOB_NAME") != "zircon-management" or os.environ.get("NODE_NAME") != "zircon-windows-agent":
                        raise ValueError("Branch commits/checks require the actual Jenkins management agent")
                    for field in ("sessionId", "requestId", "operationId"):
                        if payload[field] != os.environ.get("ZR_BRANCH_" + field.upper()):
                            raise ValueError("Branch request differs from Jenkins parameters")
                    if (os.environ.get("ZR_BRANCH_DRIVER") != runtime["driverDigest"]
                            or os.environ.get("ZR_BRANCH_RUNTIME") != runtime["runtimeOperationId"]):
                        raise ValueError("Branch runtime differs from Jenkins parameters")
                    os.environ["ZR_COMMIT_NATIVE"] = "1"
                    with d.commit.git_lock(directory(ROOT) / ("operation-" + payload["operationId"] + ".lock")):
                        if os.environ.get("ZR_BRANCH_ACTION") == "check-branch-queue":
                            validation = d.run_validation(ROOT, payload, names, d.OUTPUT / payload["operationId"] / "check-only", runtime)
                            result = {"status": "complete" if validation["status"] in {"passed", "skipped"} else "validation-failed",
                                      "validation": validation, "commitPerformed": False, "mainAdvanced": False}
                        else:
                            result = stage(d, ROOT, payload, policy=policy, grants=grants, runtime=runtime, check_runtime=current_runtime)
                else:
                    action = "check-branch-queue" if args.validate_only else "commit-branch"
                    marker = directory(ROOT) / "submissions" / (payload["operationId"] + (".check" if args.validate_only else "") + ".json")
                    with d.commit.git_lock(directory(ROOT) / ("submit-" + payload["operationId"] + ".lock")):
                        old = d.commit.read_json(marker) if marker.exists() else None
                        if old and old["request"] != payload:
                            raise ValueError("Original branch submission has different input")
                        if old and not args.resume:
                            result = {**old, "resubmitted": False}
                        else:
                            if old:
                                path = directory(ROOT) / "requests" / (payload["operationId"] + ".json")
                                last = d.commit.read_json(path) if path.exists() else {}
                                if last.get("status") not in {"waiting", "failed"}:
                                    raise ValueError("Reconcile the original queue/commit result before resuming")
                                queue = urllib.parse.urlparse(old.get("queueUrl", "")).path
                                status, raw, _ = client.request(queue + "api/json?tree=executable[number,url],cancelled")
                                build = json.loads(raw).get("executable", {}).get("number") if status == 200 else None
                                if not build:
                                    raise ValueError("Original Jenkins delivery must be reconciled before retry")
                                status, raw, _ = client.request("/job/zircon-management/" + str(build) + "/api/json?tree=building")
                                if status != 200 or json.loads(raw).get("building") is not False:
                                    raise ValueError("Original branch job is not proven terminal")
                            install_entry(d, ROOT, client, reuse_existing=True)
                            result = {"status": "submitting", "request": payload, "submittedAt": time.time()}
                            d.commit.write_json(marker, result)
                            fields = {"ACTION": action, "SESSION_ID": payload["sessionId"], "REQUEST_ID": payload["requestId"],
                                      "OPERATION_ID": payload["operationId"], "INPUT_JSON": json.dumps(payload, ensure_ascii=False),
                                      "QUERY_ONLY": "false", "EXPECTED_DRIVER_DIGEST": runtime["driverDigest"],
                                      "RUNTIME_OPERATION_ID": runtime["runtimeOperationId"]}
                            current_runtime()
                            status, _, headers = client.request("/job/zircon-management/buildWithParameters", method="POST",
                                data=urllib.parse.urlencode(fields).encode(), content_type="application/x-www-form-urlencoded")
                            result.update(status="queued" if status in (201, 302) else "delivery-unknown", queueUrl=headers.get("Location"), httpStatus=status)
                            d.commit.write_json(marker, result)
        if args.output_file:
            d.commit.write_json(args.output_file, result)
        print(json.dumps(result, ensure_ascii=False))
        return 0 if result["status"] in {"installed", "queued", "complete", "committed", "merged"} else 1
    except Exception as error:
        result = {"status": "waiting" if isinstance(error, d.commit.CommitBusy) else "failed", "reason": str(error),
                  "formalAcceptance": False, "pushPerformed": False}
        if args.output_file:
            d.commit.write_json(args.output_file, result)
        print(json.dumps(result, ensure_ascii=False))
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
