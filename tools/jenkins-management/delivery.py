"""Jenkins-owned validation, scoped commit and local main integration.

Standing user authority replaces per-chat commit/merge grants. Validation and
Git ownership remain real checks. No push, stash, reset or source copy is used.
"""
from __future__ import annotations

import argparse
from contextlib import closing
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import sqlite3
import subprocess
import sys
import time
import tomllib
import urllib.parse
import xml.etree.ElementTree as ET

ROOT = Path("E:/Git/ZirconEngine")
OUTPUT = Path("E:/cargo-targets/zircon-local/reports/jenkins-deliveries")
BEGIN = "// BEGIN ZIRCON VALIDATED DELIVERY"
END = "// END ZIRCON VALIDATED DELIVERY"


def module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    value = importlib.util.module_from_spec(spec)
    sys.modules[name] = value
    spec.loader.exec_module(value)
    return value


commit = module("delivery_checkout_commit", Path(__file__).with_name("checkout_commit.py"))


def runtime_files(repo):
    """Verify deployed code, without gating on adapters this route never executes."""
    installed_path = repo / ".jenkins/state/management/installed.json"
    installed_raw = commit.physical(installed_path).read_bytes()
    installed = json.loads(installed_raw)
    capsule = commit.physical(installed["capsule"])
    if (installed.get("job") != "zircon-management" or capsule != repo / ".jenkins/runtime/management" / installed["capsuleSha256"]):
        raise ValueError("Installed management identity differs")
    raw = (capsule / "capsule.json").read_bytes()
    config = json.loads(raw)
    if hashlib.sha256(raw).hexdigest() != installed["capsuleSha256"] or config.get("repositoryRoot") != str(repo):
        raise ValueError("Immutable management capsule differs")
    required = {"runner.py", "source_preparation.py", "zircon-management.groovy"}
    if not required <= set(config.get("files", {})) <= required | {"checkout_guard.py", "checkout_product.py"}:
        raise ValueError("Management capsule inventory differs")
    for name, expected in config["files"].items():
        if hashlib.sha256(commit.physical(capsule / name).read_bytes()).hexdigest() != expected:
            raise ValueError("Immutable management implementation changed")
    selected = commit.read_json(repo / ".jenkins/state/deployment/driver.json")
    operation = commit.read_json(repo / ".jenkins/state/deployment-operation.json")
    for name in ("driverDigest", "runtimeOperationId"):
        if config[name] != installed[name] or selected[name] != config[name]:
            raise ValueError("Selected management/runtime identity changed")
    if operation.get("operationId") != config["runtimeOperationId"] or operation.get("state") != "running":
        raise ValueError("Selected Jenkins runtime is not running")
    driver = commit.physical(repo / ".jenkins/runtime/drivers" / config["driverDigest"])
    manifest = commit.read_json(driver / "driver-manifest.json")
    encoded = json.dumps({"schemaVersion": manifest["schemaVersion"], "files": manifest["files"]},
                         sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")
    if hashlib.sha256(encoded).hexdigest() != config["driverDigest"]:
        raise ValueError("Immutable driver manifest differs")
    for name, fact in manifest["files"].items():
        if (not name or Path(name).is_absolute() or ".." in name.split("/") or "\\" in name or ":" in name):
            raise ValueError("Driver path escaped its sealed root")
        data = commit.physical(driver / name).read_bytes()
        if len(data) != fact["bytes"] or hashlib.sha256(data).hexdigest() != fact["sha256"]:
            raise ValueError("Immutable driver implementation changed")
    if installed_path.read_bytes() != installed_raw:
        raise ValueError("Installed management changed during observation")
    return installed, config, driver


def runtime_context(repo):
    installed, config, driver = runtime_files(repo)
    for name in tuple(sys.modules):
        if name == "tools" or name.startswith("tools."):
            del sys.modules[name]
    sys.path.insert(0, str(driver))
    from tools.jenkins.deployment.driver import driver_environment, active_driver_binding
    os.environ.update(driver_environment({"root": str(driver), "driverDigest": config["driverDigest"]},
                                        repo / ".jenkins/runtime/python/python.exe", repo))
    binding = active_driver_binding(repo)
    if binding["digest"] != config["driverDigest"] or binding["runtimeOperationId"] != config["runtimeOperationId"]:
        raise ValueError("Active runtime differs from the selected immutable driver")
    from tools.jenkins.deployment.spec import load_spec
    from tools.jenkins.deployment.paths import resolve_paths
    from tools.jenkins.deployment.manager import DeploymentManager
    from tools.jenkins.deployment.drain_observations import owner_observation, quiet_observation
    spec = load_spec(repo / ".jenkins/deployment-spec.json")
    manager = DeploymentManager(spec, resolve_paths(spec))
    owner = owner_observation(manager)
    health = manager.health()
    if (owner["operationId"] != config["runtimeOperationId"] or quiet_observation(manager)
            or not all(health.get(k) is True for k in ("controller", "agent", "plugins", "ready", "acceptingRequests"))
            or health.get("quietingDown") is not False):
        raise ValueError("Actual Jenkins controller/agent is not healthy and accepting requests")
    return {"installed": installed, "config": config, "manager": manager, "owner": owner}


def policy_for(repo):
    policy = commit.load_policy(repo)
    rules = policy.get("integration", {})
    if (rules.get("enabled") is not True or rules.get("sessions") != "any"
            or rules.get("targetBranch") != "main" or rules.get("validationRequired") is not True
            or rules.get("requireCleanMain") is not True or rules.get("requirePullRequest") is not True
            or rules.get("executor") != "zircon-management"
            or any(rules.get(k) is not False for k in ("requireSeparateCommitApproval",
                "requireSeparateMergeApproval", "requireWholeWorkspaceAcceptance"))):
        raise ValueError("Standing validated main integration is not enabled")
    return policy


def text(root, *args):
    return commit.git(root, *args).stdout.decode("utf-8", "strict").strip()


def files(root, names):
    return {name: hashlib.sha256(commit.physical(root / name).read_bytes()).hexdigest()
            if (root / name).exists() else None for name in names}


def input_paths(repo, names):
    """Dependencies are readable inputs; they need no source write permission."""
    if not isinstance(names, list) or len(names) > 512:
        raise ValueError("Dependencies must be a bounded file list")
    for name in names:
        if (not isinstance(name, str) or not name or "\\" in name or ":" in name
                or any(p in {"", ".", "..", ".git"} for p in name.split("/"))
                or any(c in name for c in "\0\r\n*?[")
                or not commit.physical(repo / name).is_relative_to(repo)
                or (repo / name).exists() and not (repo / name).is_file()):
            raise ValueError("Dependencies require literal repository-relative file paths")
    return names


def validate_request(repo, payload, policy, grants):
    if payload.get("action") != "deliver-task" or payload.get("targetBranch", "main") != "main":
        raise ValueError("Only deliver-task to local main is supported")
    for field in ("sessionId", "requestId", "operationId"):
        if not isinstance(payload.get(field), str) or not commit.ID.fullmatch(payload[field]):
            raise ValueError("Invalid " + field)
    paths = commit.selected_paths(repo, payload.get("paths"), policy, grants)
    dependencies = payload.get("dependencies", [])
    input_paths(repo, dependencies)
    message = payload.get("message")
    if not isinstance(message, str) or not message.strip() or len(message) > 4096 or "\0" in message:
        raise ValueError("A bounded commit message is required")
    steps = payload.get("validation")
    if not isinstance(steps, list) or not 1 <= len(steps) <= 16:
        raise ValueError("One to sixteen scoped validation steps are required")
    for step in steps:
        if not isinstance(step, dict) or step.get("kind") not in {"pytest", "python-static", "cargo", "skills"}:
            raise ValueError("Supported validation recipes: pytest, python-static, cargo, skills")
        if step["kind"] == "cargo":
            arguments = step.get("arguments")
            if not isinstance(arguments, list) or not all(isinstance(v, str) for v in arguments):
                raise ValueError("Cargo arguments must be an array")
        else:
            selected = commit.selected_paths(repo, step.get("paths"), policy, grants)
            if step["kind"] == "skills" and any(not name.endswith("/SKILL.md") for name in selected):
                raise ValueError("Skill validation selects canonical SKILL.md files")
            if step["kind"] in {"pytest", "python-static"} and any(not name.endswith(".py") for name in selected):
                raise ValueError("Python validation requires explicit Python files")
    return paths, list(dict.fromkeys([*paths, *dependencies, *(n for s in steps for n in s.get("paths", []))]))


def source_root(repo, payload):
    root = commit.physical(payload.get("sourceRoot", repo))
    if root == repo:
        if text(repo, "symbolic-ref", "--short", "HEAD") != "main":
            raise ValueError("The shared checkout must remain on main")
        return root
    if not root.is_relative_to(commit.physical(repo / ".jenkins/worktrees")):
        raise ValueError("Use this checkout or an existing Jenkins-managed worktree")
    database = commit.physical(repo / ".jenkins/state/coordination.sqlite3")
    with closing(sqlite3.connect(database.as_uri() + "?mode=ro", uri=True, timeout=5)) as connection:
        records = [json.loads(row[0]) for row in connection.execute(
            "SELECT payload FROM records WHERE domain='management_worktree'")]
    record = next((v for v in records if commit.physical(v.get("path", v.get("root", "__missing__"))) == root
                   and v.get("status") == "ready" and v.get("sessionId") == payload["sessionId"]), None)
    if record is None:
        raise ValueError("The worktree needs its original ready owner/session identity")
    observed_branch = commit.git(root, "symbolic-ref", "--quiet", "--short", "HEAD", check=False)
    branch = observed_branch.stdout.decode("utf-8").strip() if observed_branch.returncode == 0 else None
    if branch == "main" or branch != record.get("branch"):
        raise ValueError("Managed worktree branch differs from its original identity")
    common = Path(text(root, "rev-parse", "--path-format=absolute", "--git-common-dir"))
    if commit.physical(common) != commit.physical(repo / ".git"):
        raise ValueError("Worktree Git identity differs")
    return root


def changed_paths(root, *arguments):
    return {p.decode("utf-8") for p in commit.git(root, *arguments).stdout.split(b"\0") if p}


def active_reader(repo):
    database = commit.physical(repo / ".jenkins/state/coordination.sqlite3")
    if database.exists():
        with closing(sqlite3.connect(database.as_uri() + "?mode=ro", uri=True, timeout=5)) as connection:
            for key, raw in connection.execute("SELECT key,payload FROM records WHERE domain='management_checkout_read'"):
                if json.loads(raw).get("status") in {"active", "unknown"}:
                    return key
    return None


def committed_objects(root, revision, paths):
    result = {}
    for name in paths:
        value = commit.git(root, "rev-parse", "--verify", revision + ":" + name, check=False)
        result[name] = value.stdout.decode("ascii").strip() if value.returncode == 0 else None
    return result


def feature_scope(repo, root, paths):
    if root == repo:
        return
    dirty = changed_paths(root, "diff", "--name-only", "-z", "HEAD", "--")
    dirty |= changed_paths(root, "ls-files", "--others", "--exclude-standard", "-z")
    base = text(root, "merge-base", "HEAD", "refs/heads/main")
    authored = changed_paths(root, "diff", "--name-only", "-z", base, "HEAD", "--")
    if not (dirty | authored) <= set(paths):
        raise ValueError("Feature integration includes files outside this task; retain the worktree")


def environment(output, runtime):
    names = {"PATH", "SYSTEMROOT", "WINDIR", "COMSPEC", "PATHEXT", "SYSTEMDRIVE",
             "USERPROFILE", "APPDATA", "LOCALAPPDATA", "RUSTUP_HOME", "INCLUDE", "LIB", "LIBPATH"}
    env = {k: v for k, v in os.environ.items() if k.upper() in names}
    driver = ROOT / ".jenkins/runtime/drivers" / runtime["driverDigest"]
    from tools.jenkins.deployment.driver import driver_environment
    env.update(driver_environment({"root": str(driver), "driverDigest": runtime["driverDigest"]},
                                  ROOT / ".jenkins/runtime/python/python.exe", ROOT))
    env.update(TEMP=str(output / "temp"), TMP=str(output / "temp"), TMPDIR=str(output / "temp"),
               PYTHONDONTWRITEBYTECODE="1", PYTHONNOUSERSITE="1", PYTEST_ADDOPTS="",
               PYTEST_DISABLE_PLUGIN_AUTOLOAD="1", ZR_COMMIT_TEST_ROOT=str(output / "temp"),
               ZR_COMMIT_NATIVE="1", GIT_TERMINAL_PROMPT="0",
               PYTHONPATH=os.pathsep.join((str(driver), str(ROOT / ".jenkins/runtime/test-deps"))))
    return env


def vsdev_command(command, destination):
    """Import MSVC, then preserve the native argv through Windows quoting."""
    invocation = destination / "vsdev-command.ps1"
    def literal(value):
        return "'" + value.replace("'", "''") + "'"
    invocation.write_text(
        "$ErrorActionPreference = 'Stop'\n$env:VSCMD_SKIP_SENDTELEMETRY = '1'\n"
        ". 'D:/CodexData/.codex/skills/using-vsdevcmd/scripts/Import-VsDevCmdEnvironment.ps1' -Quiet\n"
        "$taskNativeStart = New-Object System.Diagnostics.ProcessStartInfo\n"
        "$taskNativeStart.FileName = " + literal(command[0]) + "\n"
        "$taskNativeStart.Arguments = " + literal(subprocess.list2cmdline(command[1:])) + "\n"
        "$taskNativeStart.UseShellExecute = $false\n"
        "$taskNativeProcess = [System.Diagnostics.Process]::Start($taskNativeStart)\n"
        "try { $taskNativeProcess.WaitForExit(); exit $taskNativeProcess.ExitCode }\n"
        "finally { $taskNativeProcess.Dispose() }\n", encoding="utf-8-sig")
    return ["C:/Windows/System32/WindowsPowerShell/v1.0/powershell.exe", "-NoProfile", "-NonInteractive",
            "-ExecutionPolicy", "Bypass", "-File", str(invocation)]


def run_validation(root, payload, names, output, runtime):
    """Run actual commands in retained Windows Jobs and retain their EOF proof."""
    cargo = module("delivery_native_command", ROOT / "tools/dev/local_cargo.py")
    checks = module("delivery_native_checks", ROOT / "tools/jenkins-management/run_checks.py")
    output = commit.physical(output)
    output.mkdir(parents=True, exist_ok=False)
    before = files(root, names)
    head = text(root, "rev-parse", "HEAD")
    manifest = {"task_id": payload["operationId"], "owner": "zircon-management",
                "status": "running", "source_checkout": str(root), "dependencies": names,
                "temporary_artifacts": [str(output / "temp")], "outputDirectories": [str(output / "temp")],
                "retained_artifacts": [str(output), str(OUTPUT / payload["operationId"] / "cargo")],
                "retention": "Retain commands, source pins, actual native termination and failed evidence"}
    manifest_bytes = checks.encoded(manifest) + b"\n"
    (output / "artifacts.json").write_bytes(manifest_bytes)
    (output / "temp").mkdir()
    env = environment(output, runtime)
    python = str(ROOT / ".jenkins/runtime/python/python.exe")
    results = []
    with commit.verified_commit_files(root, {n: value for n, value in before.items() if value is not None}):
        for index, step in enumerate(payload["validation"]):
            destination = output / ("step-" + str(index))
            destination.mkdir()
            if step["kind"] == "pytest":
                config = destination / "pytest.ini"
                config.write_text("[pytest]\naddopts =\n", encoding="utf-8")
                command = [python, "-B", "-Xutf8", "-P", "-m", "pytest", "-c", str(config),
                           *[str(root / n) for n in step["paths"]], "-q", "--noconftest",
                           "--basetemp", str(output / "temp" / ("pytest-" + str(index))),
                           "--junitxml", str(destination / "junit.xml"), "-p", "no:cacheprovider"]
            elif step["kind"] == "python-static":
                command = [python, "-B", "-Xutf8", "-c",
                    "import ast,sys; from pathlib import Path; "
                    "[ast.parse(Path(p).read_text(encoding='utf-8-sig'), filename=p) for p in sys.argv[1:]]",
                    *[str(root / n) for n in step["paths"]]]
            elif step["kind"] == "skills":
                # The general Codex interpreter owns skill maintenance and has
                # PyYAML installed; keep Jenkins' runtime installation unchanged.
                command = ["C:/Users/HeJiahui/AppData/Local/Python/pythoncore-3.14-64/python.exe", "-B", "-Xutf8", "-P", "-c",
                    "import runpy,sys; "
                    "check=runpy.run_path('D:/CodexData/.codex/skills/.system/skill-creator/scripts/quick_validate.py')['validate_skill']; "
                    "results=[(p,*check(p)) for p in sys.argv[1:]]; "
                    "[print(p,ok,message) for p,ok,message in results]; "
                    "sys.exit(0 if all(ok for _,ok,_ in results) else 1)",
                    *[str((root / n).parent) for n in step["paths"]]]
            else:
                # Reuse the bounded public Cargo grammar and physical output checks.
                entry = module("delivery_cargo_grammar", ROOT / "tools/jenkins-management/entry.py")
                entry.parse_cargo(step["arguments"])
                from tools.jenkins.pilot.managed_cargo import toolchain_identity, execution_environment
                toolchain = toolchain_identity(root)
                generation = cargo.validate_output_path(OUTPUT / payload["operationId"] / "cargo")
                if cargo.free_bytes(Path("E:/cargo-targets")) < 35 * 1024 ** 3:
                    raise ValueError("Cargo admission requires 35 GiB free on E:")
                cargo_env = execution_environment(toolchain, generation)
                cargo_env["CARGO_HOME"] = "E:/cargo-targets/zircon-local/cache/cargo-home"
                env.update(cargo_env)
                for name in ("target", "build", "scratch", "sccache"):
                    (generation / name).mkdir(parents=True, exist_ok=True)
                command = cargo.prepare_command(step["arguments"], generation / "target")
                command[0] = toolchain["binaries"]["cargo"]["path"]
                command = vsdev_command(command, destination)
            step_env = dict(env)
            if step["kind"] == "pytest":
                # Fixture commands inherit this pytest process's retained Job;
                # explicit native-supervisor tests still launch their own Jobs.
                step_env["ZR_COMMIT_NATIVE"] = "0"
            if step["kind"] == "skills":
                step_env["PYTHONPATH"] += os.pathsep + "C:/Users/HeJiahui/AppData/Roaming/Python/Python314/site-packages"
            native = cargo._run_owned_command(command, root, step_env, destination, 1800)
            complete = checks.native_complete(native, command, root)
            commit.write_json(destination / "native-terminal.json", native)
            results.append({"recipe": step, "command": command, "cwd": str(root), "native": native,
                            "complete": complete, "status": "passed" if complete and native["rootExitCode"] == 0 else "failed"})
            if results[-1]["status"] != "passed":
                break
        after = files(root, names)
        head_after = text(root, "rev-parse", "HEAD")
        # A concurrent commit may record the very same pinned bytes. Git
        # metadata is recorded separately; it is not a source-byte change.
        relevant_commits = changed_paths(root, "diff", "--name-only", "-z", head, head_after, "--", *names)
        unchanged = before == after
    passed = len(results) == len(payload["validation"]) and all(r["status"] == "passed" for r in results) and unchanged
    result = {"status": "passed" if passed else "failed", "head": head_after, "headAtStart": head, "files": before,
              "inputsUnchanged": unchanged, "committedInputPathsDuringValidation": sorted(relevant_commits),
              "steps": results, "runtime": runtime, "formalAcceptance": False}
    commit.write_json(output / "validation.json", result)
    # Cleanup is Jenkins-owned, restricted to the registered physical temp tree.
    if all(r["complete"] for r in results):
        result["cleanup"] = checks.remove_registered_trees(output, ("temp",), manifest_bytes)
        commit.write_json(output / "validation.json", result)
    manifest.update(status="complete" if passed else "failed", validation=str(output / "validation.json"),
                    temporary_artifacts=[] if "cleanup" in result else manifest["temporary_artifacts"],
                    cleanup=result.get("cleanup"))
    commit.write_json(output / "artifacts.json", manifest)
    return result


def execute(repo, payload, *, policy=None, grants=None, validate=run_validation, runtime=None, check_runtime=lambda: None):
    """A per-operation lock protects the lifecycle; Git locks cover only mutations."""
    repo = commit.physical(repo)
    policy = policy_for(repo) if policy is None else policy
    if grants is None:
        grants = tomllib.loads((repo / ".codex/config.toml").read_text("utf-8-sig"))["permissions"]["zircon-strict"]["filesystem"]
    if policy.get("integration", {}).get("mode") == "branch-commit-then-scheduled-merge":
        helper = module("delivery_branch_integration", Path(__file__).with_name("branch_integration.py"))
        return helper.stage(sys.modules[__name__], repo, payload, policy=policy, grants=grants,
                            runtime=runtime, check_runtime=check_runtime)
    paths, names = validate_request(repo, payload, policy, grants)
    journal = repo / ".jenkins/state/management/deliveries" / (payload["operationId"] + ".json")
    previous = commit.read_json(journal) if journal.exists() else None
    if previous and previous["request"] != payload:
        raise ValueError("Operation ID already belongs to another delivery")
    if previous and previous["status"] == "complete":
        return {**previous, "reusedReceipt": True}
    if previous and previous["status"] in {"validating", "merging", "preparing-merge", "merge-recovery-required"}:
        if previous["status"] == "merging" and text(repo, "rev-parse", "HEAD") == previous.get("candidateHead"):
            previous.update(status="complete", mainHead=previous["candidateHead"], recoveredReceipt=True)
            commit.write_json(journal, previous)
            return previous
        raise ValueError("Interrupted delivery requires its existing native/merge result to be reconciled")
    root = source_root(repo, payload)
    if text(repo, "symbolic-ref", "--short", "HEAD") != "main":
        raise ValueError("Shared target checkout must remain on main")
    feature_scope(repo, root, paths)
    if root != repo and commit.git(root, "rev-parse", "--verify", "MERGE_HEAD", check=False).returncode == 0:
        raise ValueError("Retain the existing worktree merge; this request did not start it")
    attempt = (previous or {}).get("attempt", -1) + 1
    output = OUTPUT / payload["operationId"] / ("attempt-" + str(attempt))
    record = {**(previous or {}), "schemaVersion": 1, "request": payload, "status": "validating", "attempt": attempt,
              "sourceRoot": str(root), "outputRoot": str(output), "startedAt": time.time(),
              "authorization": policy["authorization"], "sessionGrantRequired": False,
              "pushPerformed": False, "notificationPerformed": False, "formalAcceptance": False}
    commit.write_json(journal, record)
    validation = validate(root, payload, names, output / "source", runtime)
    record.update(validation=validation, status="validated" if validation["status"] == "passed" else "validation-failed")
    commit.write_json(journal, record)
    if record["status"] != "validated":
        return record
    with commit.git_lock(repo / ".jenkins/state/locks/management-git.lock"):
        check_runtime()
        feature_scope(repo, root, paths)
        if (files(root, names) != validation["files"]
                or root != repo and text(root, "rev-parse", "HEAD") != validation["head"]):
            record.update(status="waiting", reason="Validated task files or HEAD changed; resume the same request")
        else:
            with commit.verified_commit_files(root, {n: v for n, v in validation["files"].items() if v is not None}):
                # Bind Git's own clean-filter/EOL representation of the pinned
                # validated files, rather than comparing raw blobs to CRLF bytes.
                expected_objects = {name: text(root, "hash-object", "--path=" + name, "--", name)
                                    if (root / name).exists() else None for name in paths}
                commit_payload = {k: payload[k] for k in ("sessionId", "requestId", "operationId", "paths", "message")}
                commit_payload["action"] = "commit-checkout"
                mapped_grants = {str(root / Path(k).relative_to(repo)): v for k, v in grants.items()
                                 if not k.startswith(":") and Path(k).is_relative_to(repo)} if root != repo else grants
                receipt = commit.execute(root, commit_payload, policy=policy, grants=mapped_grants)
                record.update(taskCommit=receipt, validatedGitObjects=expected_objects)
                actual_objects = (committed_objects(root, receipt.get("commit", receipt.get("headBefore", "HEAD")), paths)
                                  if receipt["status"] in {"complete", "no-changes"} else None)
                record["committedGitObjects"] = actual_objects
                if receipt["status"] not in {"complete", "no-changes"}:
                    record.update(status="waiting", reason=receipt.get("reasonCode"))
                elif actual_objects != expected_objects:
                    record.update(status="waiting", reason="Task commit differs from its validated Git objects; retain the original receipt")
                elif root == repo:
                    record.update(status="complete", mainHead=text(repo, "rev-parse", "HEAD"), mergeMethod="already-on-main")
                else:
                    record.update(status="committed")
        commit.write_json(journal, record)
    if record["status"] != "committed":
        return record
    with commit.git_lock(repo / ".jenkins/state/locks/management-git.lock"):
        check_runtime()
        main_head = text(repo, "rev-parse", "refs/heads/main")
        if text(root, "status", "--porcelain=v1", "--untracked-files=all"):
            raise ValueError("Feature worktree changed after its task commit")
        feature_head = text(root, "rev-parse", "HEAD")
        record.update(status="preparing-merge", featureHead=feature_head, mainHeadBefore=main_head)
        commit.write_json(journal, record)
        # Git 2.31 on this host predates merge-tree --write-tree. The clean,
        # task-owned worktree is the candidate; main is never used as scratch.
        merged = commit.git(root, "-c", "core.hooksPath=" + str(repo / ".jenkins/state/management/empty-commit-hooks"),
                            "-c", "commit.gpgSign=false", "merge", "--no-edit", "--no-gpg-sign", main_head, check=False)
        if merged.returncode:
            unmerged = commit.git(root, "ls-files", "--unmerged", "-z").stdout.decode("utf-8", "replace")
            record.update(status="merge-conflict" if unmerged else "merge-failed",
                          reason=(merged.stdout + merged.stderr).decode("utf-8", "replace")[-4000:],
                          conflictIndex=unmerged, mergeExitCode=merged.returncode)
            commit.write_json(journal, record)
            if commit.git(root, "rev-parse", "--verify", "MERGE_HEAD", check=False).returncode == 0:
                restored = commit.git(root, "merge", "--abort", check=False)
                record["candidateMergeAborted"] = bool(restored.returncode == 0
                    and text(root, "rev-parse", "HEAD") == feature_head
                    and not text(root, "status", "--porcelain=v1", "--untracked-files=all"))
                if not record["candidateMergeAborted"]:
                    record.update(status="merge-recovery-required", recoveryError=restored.stderr.decode("utf-8", "replace")[-2000:])
            commit.write_json(journal, record)
            return record
        candidate = text(root, "rev-parse", "HEAD")
    record.update(status="validating", candidateHead=candidate, mainHeadBefore=main_head)
    commit.write_json(journal, record)
    candidate_validation = validate(root, payload, names, output / "candidate", runtime)
    record.update(candidateValidation=candidate_validation,
                  status="validated" if candidate_validation["status"] == "passed" else "validation-failed")
    commit.write_json(journal, record)
    if candidate_validation["status"] != "passed":
        return record
    with commit.git_lock(repo / ".jenkins/state/locks/management-git.lock"):
        check_runtime()
        if (text(repo, "rev-parse", "HEAD") != main_head or text(root, "rev-parse", "HEAD") != candidate
                or files(root, names) != candidate_validation["files"] or active_reader(repo)
                or text(repo, "symbolic-ref", "--short", "HEAD") != "main"):
            record.update(status="waiting", reason="Main, candidate or active source reader changed; resume this request")
        else:
            record.update(status="merging")
            commit.write_json(journal, record)
            # Git refuses conflicting foreign staged/working changes; never stash/reset them.
            result = commit.git(repo, "-c", "core.hooksPath=" + str(repo / ".jenkins/state/management/empty-commit-hooks"),
                                "merge", "--ff-only", candidate, check=False)
            if result.returncode:
                record.update(status="waiting", reason=result.stderr.decode("utf-8", "replace")[-2500:])
            else:
                record.update(status="complete", mainHead=text(repo, "rev-parse", "HEAD"), mergeMethod="validated-fast-forward")
        commit.write_json(journal, record)
    return record


def install(repo, client):
    policy_for(repo)
    endpoint = "/job/zircon-management/config.xml"
    status, raw, _ = client.request(endpoint)
    if status != 200:
        raise ValueError("Management job is unavailable")
    document = ET.fromstring(raw)
    script = document.find("definition/script")
    if script is None or "--capsule" not in script.text or "zircon-windows-agent" not in script.text:
        raise ValueError("Management core routes differ")
    original = script.text
    core = original
    if BEGIN in core:
        a, b = core.index(BEGIN), core.index(END) + len(END)
        core = core[:a] + core[b:].lstrip("\r\n")
    script.text = (repo / "tools/jenkins-management/delivery.groovy").read_text("utf-8-sig").rstrip() + "\n\n" + core
    if script.text == original:
        return {"status": "installed", "unchanged": True}
    choices = document.find(".//parameterDefinitions/*[name='ACTION']/choices/a")
    if choices is not None and "deliver-task" not in [e.text for e in choices]:
        ET.SubElement(choices, "string").text = "deliver-task"
    status, _, _ = client.request(endpoint, method="POST", data=ET.tostring(document, encoding="utf-8"), content_type="application/xml")
    if status not in (200, 201, 302):
        raise ValueError("Delivery route installation failed")
    status, after, _ = client.request(endpoint)
    if status != 200 or ET.fromstring(after).findtext("definition/script") != script.text:
        raise ValueError("Delivery route lacks matching API read-back")
    return {"status": "installed", "job": "zircon-management", "action": "deliver-task"}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("install", "submit", "query", "run"))
    parser.add_argument("--input-file", type=Path)
    parser.add_argument("--output-file", type=Path)
    parser.add_argument("--operation-id")
    parser.add_argument("--resume", action="store_true")
    args = parser.parse_args(argv)
    try:
        if args.command == "query":
            if not args.operation_id or not commit.ID.fullmatch(args.operation_id):
                raise ValueError("Query requires the original operation ID")
            path = ROOT / ".jenkins/state/management/deliveries" / (args.operation_id + ".json")
            result = commit.read_json(path) if path.exists() else {"status": "not-started"}
        else:
            context = runtime_context(ROOT)
            config = context["config"]
            client = commit.transport(ROOT)
            policy_for(ROOT)
            if args.command == "install":
                result = install(ROOT, client)
            else:
                payload = commit.read_json(args.input_file)
                if args.command == "run":
                    if os.environ.get("JOB_NAME") != "zircon-management" or os.environ.get("NODE_NAME") != "zircon-windows-agent":
                        raise ValueError("Delivery requires the real Jenkins management agent")
                    for key, env in (("sessionId", "ZR_DELIVERY_SESSION"), ("requestId", "ZR_DELIVERY_REQUEST"), ("operationId", "ZR_DELIVERY_OPERATION")):
                        if payload.get(key) != os.environ.get(env):
                            raise ValueError("Delivery differs from authoritative Jenkins parameters")
                    if (config["driverDigest"] != os.environ.get("ZR_DELIVERY_DRIVER")
                            or config["runtimeOperationId"] != os.environ.get("ZR_DELIVERY_RUNTIME")):
                        raise ValueError("Delivery runtime changed")
                    os.environ["ZR_COMMIT_NATIVE"] = "1"
                    def check_runtime():
                        selected = commit.read_json(ROOT / ".jenkins/state/deployment/driver.json")
                        operation = commit.read_json(ROOT / ".jenkins/state/deployment-operation.json")
                        if (selected["driverDigest"] != config["driverDigest"]
                                or selected["runtimeOperationId"] != config["runtimeOperationId"]
                                or operation.get("operationId") != config["runtimeOperationId"]
                                or operation.get("state") != "running"):
                            raise ValueError("Selected Jenkins runtime changed during validation")
                        from tools.jenkins.deployment.drain_observations import owner_observation, quiet_observation
                        owner_observation(context["manager"], expected=context["owner"])
                        if quiet_observation(context["manager"]):
                            raise ValueError("Jenkins runtime admission is draining")
                    with commit.git_lock(ROOT / ".jenkins/state/locks" / ("delivery-" + payload["operationId"] + ".lock")):
                        result = execute(ROOT, payload, runtime={k: config[k] for k in ("driverDigest", "runtimeOperationId")},
                                         check_runtime=check_runtime)
                else:
                    grants = tomllib.loads((ROOT / ".codex/config.toml").read_text("utf-8-sig"))["permissions"]["zircon-strict"]["filesystem"]
                    validate_request(ROOT, payload, policy_for(ROOT), grants)
                    marker = ROOT / ".jenkins/state/management/delivery-submissions" / (payload["operationId"] + ".json")
                    old = commit.read_json(marker) if marker.exists() else None
                    if old and old["request"] != payload:
                        raise ValueError("Original delivery identity has different input")
                    if old and not args.resume:
                        result = {**old, "resubmitted": False}
                    else:
                        if old:
                            journal = ROOT / ".jenkins/state/management/deliveries" / (payload["operationId"] + ".json")
                            last = commit.read_json(journal) if journal.exists() else {}
                            if last.get("status") not in {"waiting", "validation-failed", "merge-conflict"}:
                                raise ValueError("Reconcile the original queue/native result before resuming")
                            queue_path = urllib.parse.urlparse(old.get("queueUrl", "")).path
                            status, raw, _ = client.request(queue_path + "api/json")
                            queue = json.loads(raw) if status == 200 else {}
                            build = queue.get("executable", {}).get("number")
                            if not build:
                                status, raw, _ = client.request('/job/zircon-management/api/json?tree=builds[number,building,queueId]')
                                queue_id = int(queue_path.rstrip('/').split('/')[-1])
                                build = next((v["number"] for v in json.loads(raw).get("builds", []) if v.get("queueId") == queue_id), None) if status == 200 else None
                            status, raw, _ = client.request('/job/zircon-management/' + str(build) + '/api/json')
                            if status != 200 or json.loads(raw).get("building") is not False:
                                raise ValueError("Original Jenkins build is not proven terminal")
                        install(ROOT, client)
                        result = {"status": "submitting", "request": payload, "submittedAt": time.time()}
                        commit.write_json(marker, result)
                        fields = {"ACTION": "deliver-task", "SESSION_ID": payload["sessionId"], "REQUEST_ID": payload["requestId"],
                                  "OPERATION_ID": payload["operationId"], "INPUT_JSON": json.dumps(payload, ensure_ascii=False),
                                  "QUERY_ONLY": "false", "EXPECTED_DRIVER_DIGEST": config["driverDigest"],
                                  "RUNTIME_OPERATION_ID": config["runtimeOperationId"]}
                        from tools.jenkins.deployment.drain_observations import owner_observation
                        owner_observation(context["manager"], expected=context["owner"])
                        status, raw, headers = client.request('/job/zircon-management/buildWithParameters', method="POST",
                            data=urllib.parse.urlencode(fields).encode(), content_type="application/x-www-form-urlencoded")
                        result.update(status="queued" if status in (201, 302) else "unknown", queueUrl=headers.get("Location"), httpStatus=status)
                        commit.write_json(marker, result)
        if args.output_file:
            commit.write_json(args.output_file, result)
        print(json.dumps(result, ensure_ascii=False))
        return 0 if result["status"] in {"installed", "queued", "complete"} else 1
    except Exception as error:
        result = {"status": "waiting" if isinstance(error, commit.CommitBusy) else "failed",
                  "message": str(error), "pushPerformed": False, "formalAcceptance": False}
        if args.command == "run" and isinstance(error, commit.CommitBusy) and 'payload' in locals():
            journal = ROOT / ".jenkins/state/management/deliveries" / (payload["operationId"] + ".json")
            if journal.exists():
                record = commit.read_json(journal)
                if record.get("status") in {"validated", "committed"}:
                    record.update(status="waiting", reason="Another Jenkins operation currently owns Git")
                    commit.write_json(journal, record)
                    result = record
        if args.output_file:
            commit.write_json(args.output_file, result)
        print(json.dumps(result, ensure_ascii=False))
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
