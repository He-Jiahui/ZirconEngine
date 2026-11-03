"""Bounded Jenkins fetch and retirement of delivered local branches/worktrees.

The client only installs a typed route. Git mutations, inventory files and
cleanup run on the fixed Jenkins agent under a retained Windows Job.
"""
from __future__ import annotations

import argparse
from functools import cache
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
import unittest
import xml.etree.ElementTree as ET

ROOT = Path("E:/Git/ZirconEngine")
REPORTS = Path("E:/cargo-targets/zircon-local/reports")
ACTION = "consolidate-worktrees"
MARKER = "// ZIRCON BOUNDED WORKTREE CONSOLIDATION"
ID = re.compile(r"[A-Za-z0-9][A-Za-z0-9_.-]{0,79}\Z")
SELECTED_PATHS_SHA256 = "e0a19a680badbb4142216f1c44c43bea5cc320effff985c04556cc88ec796ed1"
SYNC_CONTROL = ROOT / ".jenkins/state/management/reports/zircon-git-sync-20261018-r1"
MERGE_REPORTS = REPORTS / "zircon-git-sync-20261018-r1/runs"


def install(delivery, repo):
    """Adapt the retained publisher to the canonical selected-profile path guard."""
    if physical(repo) != ROOT:
        raise ValueError("Permission adapter belongs to the original repository")
    source = {"path": str(ROOT / "tools/jenkins-management/checkout_commit.py"),
              "sha256": SELECTED_PATHS_SHA256}
    qualified = load("consolidation_selected_paths", source)
    delivery.commit.selected_paths = qualified.selected_paths
    utility = sys.modules.get("periodic_merge_util")
    if utility is not None and os.environ.get("JOB_NAME") == "zircon-merge-queue":
        original_load = utility.load
        def load_registered_branch(name, *args, **kwargs):
            module = original_load(name, *args, **kwargs)
            if name == "periodic_merge_branch":
                module.register_candidate_output = register_candidate_output
            return module
        utility.load = load_registered_branch


def register_candidate_output(delivery, output, manifest):
    """Register new native outputs in this run, without recreating lost history."""
    number = os.environ.get("BUILD_NUMBER", "")
    if (os.environ.get("JOB_NAME") != "zircon-merge-queue" or not number.isdigit()
            or os.environ.get("NODE_NAME") != "zircon-windows-agent"):
        raise ValueError("Candidate inventory requires the original merge-queue agent")
    run = physical(MERGE_REPORTS / number)
    current = json.loads((run / "artifacts.json").read_bytes())
    output = physical(output)
    operation = manifest.get("task_id", "")
    if (current.get("task_id") != "zircon-git-sync-" + number or not ID.fullmatch(operation)
            or output.parent != physical(delivery.OUTPUT / operation)
            or not re.fullmatch(r"candidate-(initial-checkout|checkout|merge|abort)-[0-9]+", output.name)
            or output.exists()):
        raise ValueError("Candidate output differs from its preregistered owner")
    key = sha(str(output).casefold().encode("utf-8"))
    save(run / "candidate-inventory" / (key + ".json"),
         {**manifest, "paths": [str(output)], "releasedBytes": 0, "originalInventoryRecreated": False})


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def physical(path):
    path = Path(os.path.abspath(path))
    for ancestor in (path, *path.parents):
        try:
            info = ancestor.lstat()
        except FileNotFoundError:
            continue
        if stat.S_ISLNK(info.st_mode) or getattr(info, "st_file_attributes", 0) & 0x400:
            raise ValueError("Path aliases are protected: " + str(path))
    return path


def read_pin(pin):
    raw = physical(pin["path"]).read_bytes()
    if sha(raw) != pin["sha256"]:
        raise ValueError("Pinned input changed: " + pin["path"])
    return raw


def load(name, pin):
    raw = read_pin(pin)
    module = importlib.util.module_from_spec(importlib.util.spec_from_file_location(name, pin["path"]))
    sys.modules[name] = module
    exec(compile(raw, pin["path"], "exec"), module.__dict__)
    return module


def save(path, value):
    path = physical(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("x", encoding="utf-8", newline="\n") as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write("\n")


def validate_plan(plan):
    if (not ID.fullmatch(plan.get("taskId", "")) or not ID.fullmatch(plan.get("operationId", ""))
            or plan.get("mode") not in {"qualify", "fetch-audit", "apply", "mvp-tooling",
                                        "qualify-integration", "repair-integration"}
            or plan.get("repository") != str(ROOT)):
        raise ValueError("Invalid bounded consolidation identity")
    for selection in (plan.get("branches", []), plan.get("worktrees", [])):
        if not isinstance(selection, list) or len(selection) > 128:
            raise ValueError("A bounded explicit selection is required")
    refs = []
    for item in plan.get("branches", []):
        ref = item.get("ref", "")
        if (not re.fullmatch(r"refs/heads/codex/[A-Za-z0-9][A-Za-z0-9._/-]*", ref)
                or ".." in ref or "//" in ref or ref.endswith(("/", ".lock"))
                or not re.fullmatch(r"[0-9a-f]{40}", item.get("oid", ""))):
            raise ValueError("Only exact local codex refs can be retired")
        refs.append(ref)
    if len(refs) != len(set(refs)):
        raise ValueError("Duplicate branch selection")
    roots = []
    for item in plan.get("worktrees", []):
        root = physical(item["path"])
        if root.parent != ROOT / ".jenkins/worktrees" or root.name == "integration-queue":
            raise ValueError("Only immediate managed task worktrees can be retired")
        if not re.fullmatch(r"[0-9a-f]{40}", item.get("head", "")):
            raise ValueError("An exact worktree HEAD is required")
        roots.append(str(root).casefold())
    if len(roots) != len(set(roots)):
        raise ValueError("Duplicate worktree selection")


def disposition(*, merged, dirty, locked=False, holds=(), ignored=False, process=False, registered=True):
    for condition, reason in ((not registered, "unregistered-owner"), (not merged, "unique-commits"),
                              (dirty, "uncommitted-source"), (locked, "native-git-operation"),
                              (bool(holds), "retained-reader-or-writer"), (ignored, "untracked-or-ignored-files"),
                              (process, "active-process-reference")):
        if condition:
            return reason
    return "eligible"


def runtime(plan):
    selected = json.loads((ROOT / ".jenkins/state/deployment/driver.json").read_bytes())
    if any(selected.get(key) != value for key, value in plan["runtime"].items()):
        raise ValueError("Selected runtime changed; retain the original request")
    return load("consolidation_runtime", plan["runtimeHelper"]).observe_binding(ROOT, selected["driverDigest"])


def verify_helpers(plan):
    expected = {"runtimeHelper": ROOT / "tools/dev/jenkins_cleanup_runtime.py",
                "transportHelper": ROOT / "tools/dev/jenkins_governance.py"}
    for field, path in expected.items():
        if physical(plan[field]["path"]) != path:
            raise ValueError("Unsupported management helper")
    parent = ROOT / ".jenkins/runtime/branch-integration"
    for field, name in (("gitHelper", "checkout_commit.py"), ("nativeHelper", "local_cargo.py"),
                        ("checksHelper", "run_checks.py")):
        path = physical(plan[field]["path"])
        if (path.parent.parent != parent or not re.fullmatch(r"[0-9a-f]{64}", path.parent.name)
                or path.name != name):
            raise ValueError("Only installed immutable Git/native helpers are supported")
        raw = (path.parent / "component.json").read_bytes()
        if sha(raw) != path.parent.name or json.loads(raw)["files"][name]["sha256"] != plan[field]["sha256"]:
            raise ValueError("Immutable component inventory changed")
    if physical(plan["testFile"]) != physical(Path(__file__).parent / "tests/test_worktree_consolidation.py"):
        raise ValueError("Only this route's scoped regression suite is supported")
    required = [*plan["sourcePins"], *(plan[field] for field in expected),
                *(plan[field] for field in ("gitHelper", "nativeHelper", "checksHelper"))]
    for pin in required:
        read_pin(pin)


def state_holds():
    """Read only the small lifecycle domains; never deserialize historical operations."""
    result, owners = {}, {}
    database = ROOT / ".jenkins/state/coordination.sqlite3"
    with sqlite3.connect(database.as_uri() + "?mode=ro", uri=True) as connection:
        for key, raw in connection.execute("SELECT key,payload FROM records WHERE domain='management_worktree'"):
            owners[key] = json.loads(raw)
        for key, raw in connection.execute("SELECT key,payload FROM records WHERE domain='managed_worktree_writer'"):
            value = json.loads(raw)
            if value.get("status") != "released":
                result.setdefault(value.get("worktreeId", key), []).append("writer:" + key)
        for key, raw in connection.execute("SELECT key,payload FROM records WHERE domain='current_source_intent'"):
            value = json.loads(raw)
            # Conservatively retain historical reader references, including retired ones.
            if value.get("worktreeId"):
                result.setdefault(value["worktreeId"], []).append("reader:" + key)
    return result, owners


def process_references(roots):
    """Keep command lines in memory; report only matching process identities."""
    command = ["powershell.exe", "-NoLogo", "-NoProfile", "-NonInteractive", "-Command",
               "$ErrorActionPreference='Stop'; [Console]::OutputEncoding=[Text.UTF8Encoding]::new($false); "
               "@(Get-CimInstance Win32_Process | "
               "Select-Object ProcessId,CreationDate,CommandLine) | ConvertTo-Json -Compress"]
    result = subprocess.run(command, capture_output=True, timeout=45, check=True)
    rows = json.loads(result.stdout.decode("utf-8-sig", errors="strict"))
    if not isinstance(rows, list) or not rows:
        raise ValueError("Native process inventory unavailable")
    matched = {str(root): [] for root in roots}
    for row in rows:
        line = str(row.get("CommandLine") or "").replace("\\", "/").casefold()
        for root in roots:
            if root.as_posix().casefold() in line:
                matched[str(root)].append({"pid": row["ProcessId"], "creationTime": row["CreationDate"]})
    return matched


def porcelain_fields(block):
    if not block.strip():
        return {}
    return dict((line.split(" ", 1) + [""])[:2] for line in block.splitlines())


def legacy_owner(root, fields, item):
    """Keep attribution for a pre-State Git worktree explicitly selected by the user task."""
    owner = item.get("legacyOwner")
    if not owner:
        return False
    evidence = physical(owner["evidence"]["path"])
    if (not re.fullmatch(r"[0-9a-f-]{36}", owner.get("ownerThreadId", ""))
            or not evidence.is_relative_to(Path("D:/CodexData/.codex/sessions"))
            or owner["ownerThreadId"] not in evidence.name
            or physical(owner["gitFile"]["path"]) != root / ".git"
            or fields["HEAD"] != item["head"] or fields.get("branch") != item["branch"]):
        raise ValueError("Original legacy worktree attribution changed")
    read_pin(owner["gitFile"])
    if root.name.encode("utf-8") not in read_pin(owner["evidence"]):
        raise ValueError("Owning chat does not identify this exact worktree")
    return True


def inventory(helper, selected_worktrees=None, include_branches=True, selections=None):
    def text(root, *args):
        return helper.git(root, *args).stdout.decode("utf-8").strip()
    remote = text(ROOT, "rev-parse", "origin/main")
    holds, owners = state_holds()
    common = physical(text(ROOT, "rev-parse", "--path-format=absolute", "--git-common-dir"))
    trees = []
    for block in text(ROOT, "worktree", "list", "--porcelain").split("\n\n"):
        fields = porcelain_fields(block)
        if not fields:
            continue
        root = physical(fields["worktree"])
        if selected_worktrees is not None and str(root) not in selected_worktrees:
            trees.append({"path": str(root), "head": fields["HEAD"], "branch": fields.get("branch"),
                          "reason": "outside-operation-scope"})
            continue
        gd = physical(text(root, "rev-parse", "--absolute-git-dir"))
        statuses = helper.git(root, "status", "--porcelain=v1", "-z", "--untracked-files=all").stdout
        ignored = helper.git(root, "ls-files", "--others", "--ignored", "--exclude-standard", "-z").stdout
        same_repo = physical(text(root, "rev-parse", "--path-format=absolute", "--git-common-dir")) == common
        merged = helper.git(ROOT, "merge-base", "--is-ancestor", fields["HEAD"], remote, check=False).returncode == 0
        protected = root == ROOT or root.name == "integration-queue"
        locked = ("locked" in fields or any((gd / name).exists() for name in
                  ("index.lock", "MERGE_HEAD", "CHERRY_PICK_HEAD", "REVERT_HEAD", "rebase-merge", "rebase-apply")))
        owner = owners.get(root.name, {})
        registered = (same_repo and (physical(owner.get("path", ROOT)) == root and owner.get("status") == "ready"
                      or not owner and legacy_owner(root, fields, (selections or {}).get(str(root), {}))))
        reason = "protected-checkout" if protected else disposition(
            merged=merged, dirty=bool(statuses), locked=locked, holds=holds.get(root.name, []),
            ignored=bool(ignored), registered=registered)
        trees.append({"path": str(root), "head": fields["HEAD"], "branch": fields.get("branch"),
                      "tree": text(root, "rev-parse", "HEAD^{tree}"), "reason": reason,
                      "dirtyEntries": statuses.count(b"\0"), "statusSha256": sha(statuses),
                      "ignoredEntries": ignored.count(b"\0"), "holds": holds.get(root.name, [])})
    checked_out = {tree["branch"] for tree in trees}
    branches = []
    lines = text(ROOT, "for-each-ref", "--format=%(refname)%09%(objectname)", "refs/heads").splitlines() if include_branches else []
    held_branches = {"refs/heads/codex/jenkins/" + item.rsplit("-", 1)[0] + "/" + item.rsplit("-", 1)[1]
                     for item in holds if "-" in item}
    for line in lines:
        ref, oid = line.split("\t")
        merged = helper.git(ROOT, "merge-base", "--is-ancestor", oid, remote, check=False).returncode == 0
        reason = ("checked-out" if ref in checked_out else "retained-reader-or-writer" if ref in held_branches
                  else "protected-ref" if not ref.startswith("refs/heads/codex/")
                  else "eligible" if merged else "unique-commits")
        branches.append({"ref": ref, "oid": oid, "reason": reason})
    return {"remoteMain": remote, "worktrees": trees, "branches": branches}


@cache
def allocation_api():
    import ctypes
    from ctypes import wintypes
    class StandardInfo(ctypes.Structure):
        _fields_ = [("allocation", ctypes.c_longlong), ("size", ctypes.c_longlong),
                    ("links", wintypes.DWORD), ("deletePending", ctypes.c_ubyte), ("directory", ctypes.c_ubyte)]
    query = ctypes.WinDLL("kernel32", use_last_error=True).GetFileInformationByHandleEx
    query.argtypes = [wintypes.HANDLE, ctypes.c_int, ctypes.c_void_p, wintypes.DWORD]
    query.restype = wintypes.BOOL
    return query, StandardInfo


def file_facts(path):
    import ctypes
    import msvcrt
    query, info_type = allocation_api()
    with path.open("rb") as stream:
        info = info_type()
        if not query(msvcrt.get_osfhandle(stream.fileno()), 1, ctypes.byref(info), ctypes.sizeof(info)):
            raise ctypes.WinError(ctypes.get_last_error())
        observed = os.fstat(stream.fileno())
        return {"path": str(path), "bytes": observed.st_size, "sha256": sha(stream.read()),
                "mtimeNs": observed.st_mtime_ns, "hardLinks": info.links,
                "allocatedBytes": info.allocation if info.links == 1 else 0}


def retire(plan, before, helper, output):
    by_path = {row["path"]: row for row in before["worktrees"]}
    roots = [Path(row["path"]) for row in plan.get("worktrees", [])]
    processes = process_references(roots) if roots else {}
    deleted, files, released, allocated = [], 0, 0, 0
    selections = {str(physical(item["path"])): item for item in plan.get("worktrees", [])}
    def journal(event):
        with (output / "cleanup-journal.jsonl").open("a", encoding="utf-8") as stream:
            stream.write(json.dumps(event, ensure_ascii=False) + "\n")
            stream.flush()
            os.fsync(stream.fileno())
    for expected in plan.get("worktrees", []):
        root = physical(expected["path"])
        row = by_path[str(root)]
        if row["head"] != expected["head"] or row["reason"] != "eligible" or processes[str(root)]:
            raise ValueError("Selected worktree changed or is occupied: " + str(root))
        facts = []
        for parent, dirs, names in os.walk(root, followlinks=False):
            for name in dirs + names:
                physical(Path(parent) / name)
            for name in names:
                path = Path(parent) / name
                facts.append(file_facts(path))
        save(output / (root.name + "-deletion-plan.json"), {"head": row["head"], "tree": row["tree"], "files": facts})
        runtime(plan)
        current = next(item for item in inventory(helper, {str(root)}, False, selections)["worktrees"] if item["path"] == str(root))
        if current != row or process_references([root])[str(root)]:
            raise ValueError("Selected worktree changed before native removal")
        # Git's own clean-worktree check is retained. Never use --force.
        journal({"status": "intent", "kind": "worktree", "path": str(root), "head": row["head"]})
        helper.git(ROOT, "worktree", "remove", "--", str(root))
        if root.exists():
            raise ValueError("Native worktree removal needs reconciliation")
        deleted.append(str(root)); files += len(facts); released += sum(item["bytes"] for item in facts)
        allocated += sum(item["allocatedBytes"] for item in facts)
        journal({"status": "removed", "kind": "worktree", "path": str(root),
                 "files": len(facts), "logicalBytes": sum(item["bytes"] for item in facts),
                 "releasedFileAllocationBytes": sum(item["allocatedBytes"] for item in facts)})
    current = inventory(helper, set())
    refs = {row["ref"]: row for row in current["branches"]}
    for expected in plan.get("branches", []):
        row = refs.get(expected["ref"])
        if not row or row["oid"] != expected["oid"] or row["reason"] != "eligible":
            raise ValueError("Selected branch changed or is occupied: " + expected["ref"])
    for expected in plan.get("branches", []):
        runtime(plan)
        tip = helper.git(ROOT, "rev-parse", "--verify", expected["ref"]).stdout.decode().strip()
        occupied = helper.git(ROOT, "for-each-ref", "--format=%(worktreepath)", expected["ref"]).stdout.strip()
        contained = helper.git(ROOT, "merge-base", "--is-ancestor", tip, "origin/main", check=False).returncode == 0
        if tip != expected["oid"] or occupied or not contained:
            raise ValueError("Branch acquired a new owner or source before deletion")
        gd = Path(helper.git(ROOT, "rev-parse", "--absolute-git-dir").stdout.decode().strip())
        if (gd / "index.lock").exists():
            raise ValueError("Existing Git index owner retained")
        journal({"status": "intent", "kind": "branch", **expected})
        helper.git(ROOT, "update-ref", "-d", expected["ref"], expected["oid"])
        deleted.append(expected["ref"])
        journal({"status": "removed", "kind": "branch", **expected})
    return {"deleted": deleted, "deletedFiles": files, "releasedLogicalBytes": released,
            "releasedFileAllocationBytes": allocated,
            "deletedBranches": len(plan.get("branches", [])), "deletedWorktrees": len(plan.get("worktrees", []))}


def execute(plan, output):
    proof = runtime(plan)
    save(output / "runtime.json", proof)
    if plan["mode"] in {"mvp-tooling", "qualify-integration"}:
        return qualify_tooling(plan, output)
    if plan["mode"] == "repair-integration":
        return repair_integration(plan, output)
    helper = load("consolidation_git", plan["gitHelper"])
    os.environ["ZR_COMMIT_NATIVE"] = "1"
    with helper.git_lock(ROOT / ".jenkins/state/locks/management-git.lock"):
        if (Path(helper.git(ROOT, "rev-parse", "--absolute-git-dir").stdout.decode().strip()) / "index.lock").exists():
            raise ValueError("Existing shared Git index owner retained")
        if plan["mode"] == "fetch-audit":
            helper.git(ROOT, "fetch", "--no-tags", "origin")
        selected = {str(physical(item["path"])) for item in plan.get("worktrees", [])} if plan["mode"] == "apply" else None
        selections = {str(physical(item["path"])): item for item in plan.get("worktrees", [])}
        before = inventory(helper, selected, selections=selections)
        if plan.get("expectedRemoteMain") and before["remoteMain"] != plan["expectedRemoteMain"]:
            raise ValueError("Remote main changed since explicit selection")
        save(output / "inventory.json", before)
        result = {"status": "complete", "mode": plan["mode"], "remoteMain": before["remoteMain"],
                  "deletedBranches": 0, "deletedWorktrees": 0, "deletedFiles": 0, "releasedLogicalBytes": 0}
        if plan["mode"] == "apply":
            result.update(retire(plan, before, helper, output))
            save(output / "inventory-after.json", inventory(helper, set()))
        save(output / "receipt.json", result)
    return result


def qualify_tooling(plan, output):
    # Test fixtures are children of this operation's registered output, never a
    # source snapshot. No test in these fixed suites invokes Cargo.
    fixtures = physical(output / "fixtures")
    fixtures.mkdir()
    os.environ.update(ZR_COMMIT_TEST_ROOT=str(fixtures),
                      ZIRCON_LOCAL_CARGO_TEST_ROOT=str(fixtures / "local-cargo"),
                      TEMP=str(fixtures), TMP=str(fixtures), TMPDIR=str(fixtures))
    for name in tuple(sys.modules):
        if name == "tools" or name.startswith("tools."):
            del sys.modules[name]
    sys.path.insert(0, str(ROOT))
    if plan["mode"] == "mvp-tooling":
        # The deployed driver is a regular tools package; remove it from this
        # test process so it cannot shadow the current checkout's namespace.
        drivers = str(ROOT / ".jenkins/runtime/drivers").replace("\\", "/").casefold() + "/"
        sys.path[:] = [entry for entry in sys.path if not entry.replace("\\", "/").casefold().startswith(drivers)]
    if plan["mode"] == "mvp-tooling":
        paths = [ROOT / "tools/tests/test_local_cargo.py",
                 ROOT / "tools/jenkins-management/tests/test_build_storage_policy.py"]
    else:
        paths = [Path(plan["testFile"]),
                 ROOT / "tools/jenkins-management/tests/test_merge_queue_recovery.py",
                 ROOT / "tools/jenkins-management/tests/test_branch_integration.py",
                 ROOT / "tools/jenkins-management/tests/test_checkout_commit.py"]
    suites = []
    for index, path in enumerate(paths):
        module = load("consolidation_regression_" + str(index),
                      {"path": str(path), "sha256": sha(path.read_bytes())})
        suites.append(unittest.defaultTestLoader.loadTestsFromModule(module))
    if plan["mode"] == "qualify-integration":
        original = json.loads(read_pin(plan["originalPlan"]))
        for index, (pin, case) in enumerate(((original["worker"], "ScopeTests"),
                                           (original["publisherSource"], "PrRecoveryContractTests"))):
            module = load("consolidation_publication_" + str(index), pin)
            suites.append(unittest.defaultTestLoader.loadTestsFromTestCase(getattr(module, case)))
    result = unittest.TextTestRunner(verbosity=2).run(unittest.TestSuite(suites))
    for pin in plan["sourcePins"]:
        read_pin(pin)
    retained = [str(path) for path in fixtures.iterdir()]
    summary = {"status": "passed" if result.wasSuccessful() else "failed", "tests": result.testsRun,
               "failures": [{"test": str(test), "traceback": trace} for test, trace in result.failures],
               "errors": [{"test": str(test), "traceback": trace} for test, trace in result.errors],
               "skipped": len(result.skipped), "retainedFixtures": retained,
               "compilationPerformed": False, "formalAcceptance": False}
    save(output / "test-result.json", summary)
    return summary


def reconcile_prelaunch(plan, output):
    proof = plan.get("prelaunchProof")
    if not proof:
        return
    failed = json.loads(read_pin(proof["summary"]))
    native = json.loads(read_pin(proof["native"]))
    checks = load("consolidation_prelaunch_checks", plan["checksHelper"])
    audit = json.loads(read_pin(proof["audit"]))
    root = ROOT / ".jenkins/worktrees/integration-queue"
    prior = next(row for row in audit["worktrees"] if Path(row["path"]) == root)
    operation = proof["operationId"]
    phases = [row for row in failed.get("receipt", {}).get("operations", []) if row.get("operationId") == operation]
    if (not ID.fullmatch(operation) or failed.get("build") != proof["build"]
            or failed.get("nativeComplete") is not True or failed.get("nativeExitCode") != 1
            or native.get("rootExitCode") != 1 or native.get("cleanup", {}).get("attempted") is not False
            or not checks.native_complete(native, native["command"], ROOT)
            or len(phases) != 1 or phases[0].get("status") != "reconciliation-required"
            or "jenkins-one-worktree-flow-20261015" not in phases[0].get("reason", "")):
        raise ValueError("Original prelaunch failure has no exact native proof")
    helper = load("consolidation_prelaunch_git", plan["gitHelper"])
    os.environ["ZR_COMMIT_NATIVE"] = "1"
    marker_path = ROOT / ".jenkins/state/management/branch-integration/candidate.json"
    record_path = marker_path.parent / "requests" / (operation + ".json")
    with helper.git_lock(ROOT / ".jenkins/state/locks/management-git.lock"):
        marker = json.loads(marker_path.read_bytes())
        record = json.loads(record_path.read_bytes())
        gd = Path(helper.git(root, "rev-parse", "--absolute-git-dir").stdout.decode().strip())
        head = helper.git(root, "rev-parse", "HEAD").stdout.decode().strip()
        dirty = helper.git(root, "status", "--porcelain=v1", "--untracked-files=all").stdout
        common = Path(helper.git(root, "rev-parse", "--path-format=absolute", "--git-common-dir").stdout.decode().strip())
        if (marker.get("status") != "active" or marker.get("operationId") != operation
                or Path(marker.get("path", "")) != root or head != prior["head"] or dirty
                or common != ROOT / ".git" or record["request"].get("ownerThreadId") != plan["ownerThreadId"]
                or any(key in record for key in ("candidateGit", "candidateGitAttempt", "candidateGitHistory", "candidateHead"))
                or any((gd / name).exists() for name in ("index.lock", "MERGE_HEAD", "CHERRY_PICK_HEAD", "REVERT_HEAD", "rebase-merge", "rebase-apply"))):
            raise ValueError("Original candidate acquired native work or changed source")
        recovered = {"status": "prelaunch-failure-reconciled", "operationId": operation,
                     "headUnchanged": head, "originalMarker": marker, "proof": proof,
                     "nativeGitStarted": False, "nativeTerminationClaimed": False,
                     "originalRequestUnchanged": True}
        save(output / "prelaunch-reconciliation.json", recovered)
        record["candidateRecovery"] = recovered
        helper.write_json(record_path, record)
        helper.write_json(marker_path, {"path": str(root), "status": "idle", "prelaunchRecovery": recovered})


def repair_integration(plan, output):
    qualified = json.loads(read_pin(plan["qualification"]))
    if qualified.get("status") != "passed" or qualified.get("nativeComplete") is not True:
        raise ValueError("The affected regression run has no successful native terminal")
    original_raw = read_pin(plan["originalPlan"])
    original = json.loads(original_raw)
    if physical(plan["originalPlan"]["path"]).parent != SYNC_CONTROL / "runtime-plans":
        raise ValueError("Retain the original periodic integration owner")
    missing = {str(Path(original[field]["path"])) for field in
               ("queueRecoverySource", "selectedPermissionsBridge")}
    missing.add(str(Path("E:/cargo-targets/zircon-local/reports/jenkins-transition-monitor/operator/user-repair-20261021/deployed-r2/worker-r6-selected-permissions.py")))
    if any(Path(path).exists() for path in missing):
        raise ValueError("Original missing dependencies changed; reconcile before repair")
    updated = dict(original)
    installed_plan = plan.get("installedPlan", plan["originalPlan"])
    installed = json.loads(read_pin(installed_plan))
    updated.update(runtime=plan["runtime"], revision=installed["revision"] + 1,
                   previousPlan=installed_plan,
                   queueRecoverySource={"path": str(ROOT / "tools/jenkins-management/merge_queue_recovery.py"),
                                        "sha256": sha((ROOT / "tools/jenkins-management/merge_queue_recovery.py").read_bytes())},
                   selectedPermissionsBridge={"path": str(Path(__file__).absolute()),
                                              "sha256": sha(Path(__file__).read_bytes())})
    updated["sourcePins"] = {path: pin for path, pin in original["sourcePins"].items()
                             if str(Path(path)) not in missing}
    for pin in (updated["queueRecoverySource"], updated["selectedPermissionsBridge"],
                {"path": str(ROOT / "tools/jenkins-management/checkout_commit.py"), "sha256": SELECTED_PATHS_SHA256}):
        read_pin(pin)
        updated["sourcePins"][pin["path"]] = pin
    new_path = SYNC_CONTROL / "runtime-plans" / (plan["taskId"] + "-" + plan["operationId"] + ".json")
    updated["repair"] = {"ownerThreadId": plan["ownerThreadId"], "qualification": plan["qualification"],
                         "missingHistoricalInputs": sorted(missing), "originalRequestsChanged": False}
    save(new_path, updated)
    new_sha = sha(new_path.read_bytes())
    reconcile_prelaunch(plan, output)
    transport = load("consolidation_transport", plan["transportHelper"]).Transport(ROOT)
    endpoint = "job/zircon-merge-queue/config.xml"
    status, raw, _ = transport.request(endpoint)
    if status != 200 or sha(raw) != plan["expectedMergeConfigSha256"]:
        raise ValueError("Periodic integration configuration changed")
    document = ET.fromstring(raw)
    script = document.find("definition/script")
    if (script is None or document.findtext(".//hudson.triggers.TimerTrigger/spec") != "H/15 * * * *"
            or document.findtext("disabled") != "false"):
        raise ValueError("Original periodic integration schedule or owner changed")
    old_path = str(Path(installed_plan["path"])).replace("\\", "/")
    if script.text.count(installed_plan["sha256"]) != 1 or script.text.count(old_path) != 1:
        raise ValueError("Original periodic integration plan binding changed")
    script.text = script.text.replace(installed_plan["sha256"], new_sha).replace(old_path, new_path.as_posix())
    _, latest, _ = transport.request(endpoint)
    if latest != raw:
        raise ValueError("Concurrent periodic integration update retained")
    status, _, _ = transport.request(endpoint, method="POST", data=ET.tostring(document, encoding="utf-8"), content_type="application/xml")
    observed_status, observed, _ = transport.request(endpoint)
    if status not in (200, 201, 302) or observed_status != 200 or ET.fromstring(observed).findtext("definition/script") != script.text:
        raise ValueError("Periodic integration installation needs original-outcome reconciliation")
    # Use the canonical public installer to retire the missing branch-entry
    # wrapper. It preserves unrelated management routes and their identities.
    branch_source = ROOT / "tools/jenkins-management/branch_integration.py"
    branch = load("consolidation_branch_installer", {"path": str(branch_source), "sha256": sha(branch_source.read_bytes())})
    installed = branch.install_entry(None, ROOT, transport)
    if read_pin(plan["originalPlan"]) != original_raw:
        raise ValueError("Historical plan changed during repair")
    receipt = {"status": "complete", "originalPlan": plan["originalPlan"],
               "plan": {"path": str(new_path), "sha256": new_sha},
               "previousConfigSha256": sha(raw), "configSha256": sha(observed),
               "branchEntry": installed, "schedule": "H/15 * * * *", "sourceCopies": 0,
               "originalRequestsChanged": False, "actualMergeAccepted": False}
    save(output / "receipt.json", receipt)
    return receipt


def controller(plan, input_path):
    validate_plan(plan)
    if os.environ.get("JOB_NAME") != "zircon-management" or os.environ.get("NODE_NAME") != "zircon-windows-agent":
        raise ValueError("Consolidation requires its real Jenkins management agent")
    verify_helpers(plan)
    output = physical(REPORTS / plan["taskId"] / plan["operationId"])
    output.mkdir(parents=True, exist_ok=False)
    save(output / "artifacts.json", {"taskId": plan["taskId"], "ownerThreadId": plan["ownerThreadId"],
         "operationId": plan["operationId"], "sourcePins": plan["sourcePins"], "runtime": plan["runtime"],
         "output": str(output), "retention": "Inventory, explicit selections, exact native outcomes and deletion evidence"})
    runtime(plan)
    native = load("consolidation_native", plan["nativeHelper"])
    checks = load("consolidation_checks", plan["checksHelper"])
    if plan["mode"] == "qualify":
        command = [sys.executable, "-B", "-X", "utf8", plan["testFile"]]
    else:
        command = [sys.executable, "-B", "-X", "utf8", str(Path(__file__).absolute()),
                   "worker", "--input-file", str(input_path), "--output", str(output)]
    environment = {**os.environ, "TEMP": str(output), "TMP": str(output), "TMPDIR": str(output),
                   "PYTHONDONTWRITEBYTECODE": "1", "ZR_COMMIT_NATIVE": "0"}
    terminal = native._run_owned_command(command, ROOT, environment, output, 900)
    save(output / "native-terminal.json", terminal)
    complete = checks.native_complete(terminal, command, ROOT)
    result = {"status": "passed" if complete and terminal["rootExitCode"] == 0 else "failed",
              "nativeComplete": complete, "exitCode": terminal["rootExitCode"], "output": str(output)}
    save(output / "summary.json", result)
    print(json.dumps(result))
    return 0 if result["status"] == "passed" else 1


def install_route(plan):
    validate_plan(plan)
    verify_helpers(plan)
    transport = load("consolidation_transport", plan["transportHelper"]).Transport(ROOT)
    endpoint = "job/zircon-management/config.xml"
    status, raw, _ = transport.request(endpoint)
    if status != 200:
        raise ValueError("Management job unavailable")
    document = ET.fromstring(raw)
    script = document.find("definition/script")
    if script is None or "--capsule" not in script.text or MARKER in script.text:
        raise ValueError("Unrecognized or already installed management route; reconcile first")
    template = Path(__file__).with_suffix(".groovy").read_text(encoding="utf-8")
    template = template.replace("@@SOURCE@@", str(Path(__file__).absolute()).replace("\\", "/"))
    template = template.replace("@@SHA@@", sha(Path(__file__).read_bytes()))
    script.text = template + "\n" + script.text
    choices = document.find(".//parameterDefinitions/*[name='ACTION']/choices/a")
    if choices is not None:
        ET.SubElement(choices, "string").text = ACTION
    status, latest, _ = transport.request(endpoint)
    if latest != raw:
        raise ValueError("Concurrent management configuration update retained")
    updated = ET.tostring(document, encoding="utf-8")
    status, _, _ = transport.request(endpoint, method="POST", data=updated, content_type="application/xml")
    observed_status, observed, _ = transport.request(endpoint)
    if observed_status != 200 or ET.fromstring(observed).findtext("definition/script") != script.text:
        raise ValueError("Installation outcome needs reconciliation; do not resubmit")
    print(json.dumps({"status": "installed", "previousConfigSha256": sha(raw), "configSha256": sha(observed)}))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("install", "controller", "worker"))
    parser.add_argument("--input-file", type=Path, required=True)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    plan = json.loads(args.input_file.read_bytes())
    validate_plan(plan)
    if args.command == "install":
        install_route(plan)
        return 0
    if args.command == "controller":
        return controller(plan, args.input_file)
    if os.environ.get("JOB_NAME") != "zircon-management" or os.environ.get("NODE_NAME") != "zircon-windows-agent":
        raise ValueError("Worker requires the original Jenkins agent")
    verify_helpers(plan)
    if physical(args.output) != physical(REPORTS / plan["taskId"] / plan["operationId"]):
        raise ValueError("Worker output differs from its registered operation")
    result = execute(plan, args.output)
    print(json.dumps(result))
    return 1 if result["status"] == "failed" else 0


if __name__ == "__main__":
    raise SystemExit(main())
