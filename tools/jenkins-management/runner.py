"""Pinned ordinary Jenkins management job; receipts never confer M8 acceptance."""
from __future__ import annotations

import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import string
import subprocess
import sys
import time

ID = re.compile(r"[A-Za-z0-9][A-Za-z0-9_.-]{0,79}\Z")
WORKTREE_ID = re.compile(r"[A-Za-z0-9][A-Za-z0-9_.-]{0,159}\Z")
SHA = re.compile(r"(?:[0-9a-f]{40}|[0-9a-f]{64})\Z")
ACTIONS = {"create-branch", "create-worktree", "prepare-merge", "apply-patch",
           "commit-worktree", "release-worktree", "validate", "generate", "submit-flow"}
GIT_ACTIONS = {"create-branch", "create-worktree", "prepare-merge", "apply-patch", "commit-worktree", "release-worktree"}
AUTH = {"create-branch": "implementation", "create-worktree": "implementation",
        "prepare-merge": "implementation", "generate": "implementation",
        "apply-patch": "implementation", "commit-worktree": "commit", "release-worktree": "gc",
        "validate": "validation", "submit-flow": "validation"}


def encoded(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"),
                      ensure_ascii=False, allow_nan=False).encode("utf-8")


def sha(value):
    return hashlib.sha256(value).hexdigest()


def source_component(source):
    """Summarize every captured file and registered nested source."""
    files = source["files"]
    gitlinks = {}
    for path, value in sorted(files.items()):
        if isinstance(value, dict) and isinstance(value.get("source"), dict):
            gitlinks[path] = {"gitlinkCommit": value.get("gitlinkCommit"),
                              "source": source_component(value["source"])}
    return {"head": source["head"], "sourceDigest": source["sourceDigest"],
            "diffSha256": source["diffSha256"], "filesDigest": sha(encoded(files)),
            "fileCount": len(files), "gitlinks": gitlinks}


def source_components(snapshot):
    """Summarize complete captured inputs without retaining source bytes."""
    return {"sourceView": snapshot["sourceView"], "sourceRoot": snapshot["sourceRoot"],
            "sourceDigest": snapshot["sourceDigest"], "source": source_component(snapshot["source"]),
            "externalRoots": [{"path": value["path"], "source": source_component(value["source"])}
                              for value in snapshot["externalRoots"]],
            "dependencyCache": snapshot.get("dependencyCache")}


class SourceInputMismatch(ValueError):
    def __init__(self, expected, actual, components):
        super().__init__("The complete declared source/dependency/cache input changed before preparation")
        self.expected, self.actual, self.sourceComponents = expected, actual, components
        self.diagnostics = {"expectedSourceDigest": expected, "actualSourceDigest": actual,
                            "sourceComponents": components}


class EngineSourceInputMismatch(SourceInputMismatch):
    def __init__(self, expected, actual, component):
        ValueError.__init__(self, "The declared engine source changed before Jenkins dependency admission")
        self.expected, self.actual = expected, actual
        self.sourceComponents = {"source": component}
        self.diagnostics = {"expectedEngineSourceDigest": expected, "actualEngineSourceDigest": actual,
                            "sourceComponents": self.sourceComponents}


def input_changes(before, after, limit=128):
    """Bounded values identifying a real input mismatch without copying sources."""
    changes = []
    count = 0
    def summary(value):
        if value is None or isinstance(value, (str, bool, int, float)):
            return value
        return {"sha256": sha(encoded(value))}
    def visit(left, right, path):
        nonlocal count
        if left == right:
            return
        if isinstance(left, dict) and isinstance(right, dict):
            for name in sorted(set(left) | set(right)):
                if name == "sourceDigest":
                    continue
                visit(left.get(name), right.get(name), path + [str(name)])
        elif isinstance(left, list) and isinstance(right, list):
            for index in range(max(len(left), len(right))):
                visit(left[index] if index < len(left) else None,
                      right[index] if index < len(right) else None, path + [str(index)])
        else:
            count += 1
            if len(changes) < limit:
                changes.append({"path": path, "before": summary(left), "after": summary(right)})
    visit(before, after, [])
    return {"changedValueCount": count, "values": changes, "truncated": count > len(changes)}


def validation_inputs_verified(result, request=None):
    """Successful sealed execution does not certify a mutable foreign checkout."""
    if result.get("validationInputPolicy") is None:
        return result.get("inputsUnchanged") is True and result.get("preparedInputsUnchanged") is True
    if result.get("validationInputPolicy") != "sealed-source" or any(
            result.get(name) is not True for name in ("capturedInputVerified", "originalEngineUnchanged",
                                                     "dependencyCacheUnchanged", "preparedInputsUnchanged")):
        return False
    if request is None:
        return True
    prepared = result.get("preparation", {})
    sealed_ref = prepared.get("sealedInputRef")
    if not isinstance(sealed_ref, str) or not re.fullmatch(r"[0-9a-f]{64}", sealed_ref):
        return False
    return (request.get("validationInputPolicy") == "sealed-source"
            and request.get("capturedInputVerified") is True
            and request.get("preparation") == prepared
            and prepared.get("sealedInputRef") == prepared.get("sourceDigest")
            and prepared.get("sealedInputRef") == request.get("preparedSource", {}).get("sourceDigest")
            and prepared.get("capturedSourceDigest") == result.get("sourceDigest")
            and prepared.get("capturedSourceDigest") == request.get("source", {}).get("sourceDigest")
            and request.get("inputObservation", {}).get("originalEngineUnchanged") is True
            and request.get("inputObservation", {}).get("dependencyCacheUnchanged") is True)


def plain(path):
    path = Path(path).absolute()
    for part in (path, *path.parents):
        if part.exists() or part.is_symlink():
            if part.is_symlink() or getattr(part.lstat(), "st_file_attributes", 0) & 0x400:
                raise ValueError("Physical paths only; alias/reparse point rejected")
    return path


def identifier(value):
    if not isinstance(value, str) or not ID.fullmatch(value) or value.endswith("."):
        raise ValueError("Invalid bounded identifier")
    return value


def write_json(path, value):
    path = plain(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    data = encoded(value) + b"\n"
    if path.exists():
        if path.read_bytes() != data:
            raise ValueError("Immutable receipt already has different content")
    else:
        with path.open("xb") as stream:
            stream.write(data)
    return {"path": str(path), "sha256": sha(data)}


def native_terminal_proof(native, native_ref):
    """Adapt actual retained-Job observations; executable is the launch argument."""
    command, tree = native["command"], native["processTree"]
    if (not isinstance(command, list) or not command
            or any(not isinstance(value, str) or "\0" in value for value in command)):
        raise ValueError("Native command is not an exact argv list")
    identity = {"pid": native["pid"], "creationTime": native["creationTime"],
                "executable": command[0], "commandDigest": sha("\0".join(command).encode("utf-8"))}
    records = native["nativeJobEvidence"]["records"]
    terminal_active = records.get("afterTerminal", {}).get("activeProcesses")
    active = records.get("afterClose", {}).get("activeProcesses")
    children_gone = (tree.get("scope") == "retained-windows-job" and tree.get("terminal") is True
                     and tree.get("nativeWaiterJoined") is True
                     and type(terminal_active) is int and terminal_active == 0
                     and type(active) is int and active == 0
                     and not native["nativeJobEvidence"]["errors"])
    stdout_eof = tree["pipes"]["stdout"].get("eof") is True
    stderr_eof = tree["pipes"]["stderr"].get("eof") is True
    birth = identity["creationTime"]
    valid_identity = (type(identity["pid"]) is int and identity["pid"] > 0
                      and isinstance(birth, str) and birth.isascii() and birth.isdigit()
                      and int(birth) > 0 and Path(command[0]).is_absolute())
    complete = (valid_identity and children_gone and stdout_eof and stderr_eof
                and tree.get("pipeEOF") is True and tree.get("readersFinished") is True
                and type(native["rootExitCode"]) is int and not native["terminalErrors"])
    return {"identity": identity, "requested": False, "processExitCode": native["rootExitCode"],
            "activeProcesses": active, "stdoutEof": stdout_eof, "stderrEof": stderr_eof,
            "childrenGone": children_gone, "complete": complete, "nativeEvidence": native_ref,
            "executableEvidence": "trusted-launch-argument", "nativeImageQueryClaimed": False}


def source_helper():
    # boot verifies this file as part of the immutable capsule before use.
    path = plain(Path(__file__).with_name("source_preparation.py"))
    spec = importlib.util.spec_from_file_location("jenkins_management_source_preparation", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def boot(capsule):
    capsule = plain(capsule)
    raw = (capsule / "capsule.json").read_bytes()
    config = json.loads(raw)
    if config.get("schemaVersion") != 1 or capsule.name != sha(raw):
        raise ValueError("Management capsule identity differs")
    if set(config["files"]) != {"runner.py", "source_preparation.py", "zircon-management.groovy"}:
        raise ValueError("Management capsule lacks its pinned source preparation helper")
    for name, checksum in config["files"].items():
        if sha(plain(capsule / name).read_bytes()) != checksum:
            raise ValueError("Pinned management code changed")
    repo = plain(config["repositoryRoot"])
    ordinary_clients = {"tools/dev/local_cargo.py", "tools/dev/local-cargo.ps1", "tools/dev/dev-fast-build.ps1",
                        "tools/dev/submit-jenkins-management.ps1", "tools/jenkins/frontend.py",
                        "tools/jenkins-management/entry.py"}
    client_names = set(config["clientSources"])
    if client_names not in ({"tools/dev/local_cargo.py"}, ordinary_clients):
        raise ValueError("Management client source inventory differs")
    entry = config.get("ordinaryEntry")
    if entry is not None and (client_names != ordinary_clients or entry != {
            "enabled": True, "actions": ["validate"], "formalAcceptance": False, "m8Accepted": False}):
        raise ValueError("Ordinary entry requires its complete pinned six-file client inventory")
    for name, checksum in config["clientSources"].items():
        if sha(plain(repo / name).read_bytes()) != checksum:
            raise ValueError("Pinned management client implementation changed")
    driver = plain(repo / ".jenkins/runtime/drivers" / config["driverDigest"])
    manifest = json.loads((driver / "driver-manifest.json").read_bytes())
    if (sha(encoded({"schemaVersion": manifest["schemaVersion"], "files": manifest["files"]}))
            != config["driverDigest"]):
        raise ValueError("Driver manifest differs")
    for name, fact in manifest["files"].items():
        relative = Path(name)
        if relative.is_absolute() or ".." in relative.parts or "\\" in name or ":" in name:
            raise ValueError("Invalid driver path")
        data = plain(driver / relative).read_bytes()
        if len(data) != fact["bytes"] or sha(data) != fact["sha256"]:
            raise ValueError("Pinned driver source changed")
    sys.path.insert(0, str(driver))
    from tools.jenkins.deployment.driver import active_driver_binding, driver_environment
    os.environ.update(driver_environment({"root": str(driver), "driverDigest": config["driverDigest"]},
                                        repo / ".jenkins/runtime/python/python.exe", repo))
    binding = active_driver_binding(repo)
    if (binding["digest"] != config["driverDigest"]
            or binding["runtimeOperationId"] != config["runtimeOperationId"]):
        raise ValueError("Selected runtime changed; install a new management capsule")
    # A stale job may neither start new work during drain nor accept a stopped owner.
    operation = json.loads((repo / ".jenkins/state/deployment-operation.json").read_bytes())
    if operation.get("operationId") != config["runtimeOperationId"] or operation.get("state") != "running":
        raise ValueError("The pinned formal runtime is not running")
    from tools.jenkins.deployment.spec import load_spec
    from tools.jenkins.deployment.paths import resolve_paths
    from tools.jenkins.deployment.manager import DeploymentManager
    deployment = load_spec(repo / ".jenkins/deployment-spec.json")
    manager = DeploymentManager(deployment, resolve_paths(deployment))
    from tools.jenkins.deployment.drain_observations import owner_observation, quiet_observation
    owner = owner_observation(manager)
    if owner["operationId"] != config["runtimeOperationId"] or quiet_observation(manager):
        raise ValueError("Pinned owner changed or deployment admission is draining")
    return config, repo, manager


class Management:
    def __init__(self, config, repo, manager, payload):
        from tools.jenkins.state import State
        from tools.jenkins.source import repository_id
        self.config, self.repo, self.manager, self.payload = config, repo, manager, payload
        self.state = State(repo / ".jenkins/state/coordination.sqlite3")
        self.action = payload["action"]
        self.source_view = payload.get("sourceView", "worktree")
        if self.source_view not in {"worktree", "checkout"}:
            raise ValueError("Unsupported validation source view")
        if self.source_view == "checkout" and self.action != "validate":
            raise ValueError("The current checkout source view is validation-only")
        if self.action != "validate" and "externalRoots" in payload:
            raise ValueError("External roots are validation-only")
        self.session = identifier(payload["sessionId"])
        self.request = identifier(payload["requestId"])
        self.operation = identifier(payload["operationId"])
        self.repo_id = payload.get("repositoryId")
        names = {repository_id(repo), sha(str(repo).casefold().encode()), sha(encoded(str(repo).casefold()))}
        if self.repo_id not in names:
            raise ValueError("Repository ID is not bound to this physical checkout")
        if (self.action not in ACTIONS or payload.get("repositoryId") != self.repo_id
                or payload.get("driverDigest") != config["driverDigest"]
                or payload.get("runtimeOperationId") != config["runtimeOperationId"]):
            raise ValueError("Management identity or action differs")
        self.owned = payload.get("ownedPaths", [])
        if not isinstance(self.owned, list) or not self.owned or any(not isinstance(p, str) for p in self.owned):
            raise ValueError("ownedPaths must be an explicit path list")
        self.state.check_authorization(self.repo_id, self.session, AUTH[self.action], self.owned)
        self.key = sha(encoded({"repositoryId": self.repo_id, "sessionId": self.session,
                               "requestId": self.request, "operationId": self.operation}))
        self.worktrees = plain(repo / ".jenkins/worktrees")
        self.query = payload.get("observationOnly") is True
        self.input_digest = sha(encoded({k: v for k, v in payload.items() if k != "observationOnly"}))

    def git_command(self, root, *args):
        return ["git", "-c", "core.longpaths=true", "-c", "core.hooksPath=" + str(Path(__file__).parent / "no-hooks"),
                "-c", "core.autocrlf=false", "-C", str(root), *args]

    def git(self, root, *args, check=True):
        command = self.git_command(root, *args)
        result = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                timeout=1800 if args[:2] == ('worktree', 'add') else 600, check=False, env={**os.environ, "GIT_TERMINAL_PROMPT": "0"})
        if check and result.returncode:
            raise ValueError(result.stderr.decode("utf-8", "replace")[-3000:])
        return result

    def text(self, root, *args):
        return self.git(root, *args).stdout.decode("utf-8", "strict").strip()

    def wt(self, reference, *, own=True):
        if not isinstance(reference, str) or not WORKTREE_ID.fullmatch(reference):
            raise ValueError("Invalid managed worktree ID")
        row = self.state.get("management_worktree", reference)
        if not row or row["payload"].get("status") not in {"ready", "merge-prepared", "conflict"}:
            raise ValueError("Managed worktree is not registered")
        value = row["payload"]
        path = plain(value["path"])
        if path.parent != self.worktrees or not path.is_dir():
            raise ValueError("Worktree escapes the managed directory")
        if own and value["sessionId"] != self.session:
            raise ValueError("Worktree belongs to another session")
        self.state.check_authorization(self.repo_id, value["sessionId"], "validation", value["ownedPaths"])
        if plain(self.text(path, "rev-parse", "--path-format=absolute", "--git-common-dir")) != plain(self.repo / ".git"):
            raise ValueError("Managed worktree no longer belongs to this repository")
        return value, path

    def scoped(self, paths, action="implementation"):
        self.state.check_authorization(self.repo_id, self.session, action, list(paths))

    def source(self, root, *, payload=None):
        payload = self.payload if payload is None else payload
        names = self.git(root, "ls-files", "--cached", "--others", "--exclude-standard", "-z").stdout
        names = list(dict.fromkeys(raw.decode("utf-8", "strict") for raw in names.split(b"\0") if raw))
        files, gitlinks = {}, {}
        if str(root).casefold() == r"E:\Git\zr_vm".casefold():
            stage = self.git(root, "ls-files", "--stage", "-z").stdout
            indexed = {}
            for row in stage.split(b"\0"):
                if row.startswith(b"160000 "):
                    fields, relative = row.split(b"\t", 1)
                    indexed[relative.decode("utf-8")] = fields.decode("ascii").split()[1]
            requested = payload.get("externalGitlinks")
            if (not isinstance(requested, dict) or set(requested) != set(indexed)
                    or any(not isinstance(v, dict) or v.get("indexCommit") != indexed[k]
                           or not isinstance(v.get("head"), str) or not SHA.fullmatch(v["head"])
                           for k, v in requested.items())):
                raise ValueError("Existing external submodules require explicit index and HEAD registration")
            for name, requested_input in requested.items():
                child = plain(root / name)
                if not child.is_relative_to(root) or not child.is_dir():
                    raise ValueError("Registered external submodule is unavailable: " + name)
                if plain(self.text(child, "rev-parse", "--show-toplevel")) != child:
                    raise ValueError("External submodule physical Git root differs: " + name)
                child_source = self.source(child, payload=payload)
                if child_source["head"] != requested_input["head"]:
                    raise ValueError("Registered external submodule HEAD changed: " + name)
                gitlinks[name] = {"gitlinkCommit": requested_input["indexCommit"], "source": child_source}
        def capture(name):
            path = plain(root / name)
            if not path.is_relative_to(root) or name.startswith((".git/", "target/")):
                raise ValueError("Unexpected source path: " + name)
            if path.is_file():
                digest = hashlib.sha256()
                with path.open("rb") as stream:
                    for chunk in iter(lambda: stream.read(1024 * 1024), b""):
                        digest.update(chunk)
                return name, digest.hexdigest()
            if path.exists():
                raise ValueError("Source submodule/directory requires a registered flow input: " + name)
            return name, None
        ordinary = [name for name in names if name not in gitlinks]
        print("[management] Capturing input " + str(root) + ": " + str(len(ordinary)) + " files", flush=True)
        # Bounded readers preserve every physical path and file hash check.
        with ThreadPoolExecutor(max_workers=4) as readers:
            for start in range(0, len(ordinary), 128):
                for name, checksum in readers.map(capture, ordinary[start:start + 128]):
                    files[name] = checksum
                if start and start % 4096 == 0:
                    print("[management] Input files captured: " + str(start) + "/" + str(len(ordinary)), flush=True)
        files.update(gitlinks)
        value = {"head": self.text(root, "rev-parse", "HEAD"), "files": files,
                 "diffSha256": sha(self.git(root, "diff", "--no-ext-diff", "--no-textconv", "--binary", "--full-index", "HEAD", "--").stdout)}
        print("[management] Input capture complete: " + str(root), flush=True)
        return {**value, "sourceDigest": sha(encoded(value))}

    def validation_source(self, *, payload=None, observation_errors=None):
        payload = self.payload if payload is None else payload
        view = payload.get("sourceView", "worktree")
        if view == "worktree":
            _record, root = self.wt(payload["worktreeId"])
        elif view == "checkout" and not payload.get("worktreeId"):
            root = plain(self.repo)
        else:
            raise ValueError("Unsupported validation source view")
        requested = payload.get("externalRoots", [])
        canonical = plain(self.repo.parent / "zr_vm")
        if (not isinstance(requested, list) or len(requested) > 1
                or any(not isinstance(item, str) or plain(Path(item)) != canonical for item in requested)
                or view == "checkout" and requested != [str(canonical)]):
            raise ValueError("Validation requires an explicit canonical zr_vm dependency")
        # Observations capture the owned engine independently, even if the
        # foreign checkout becomes unreadable or changes its registered HEAD.
        admission = payload.get("externalCapturePolicy") == "jenkins-admission"
        engine = self.source(root, payload=payload) if observation_errors is not None or admission else None
        if admission and observation_errors is None:
            # Verify the entire owned engine before reading the mutable foreign
            # closure or admitting preparation. Observations remain independent.
            self.verify_declared_engine(engine, payload)
        external = []
        for item in requested:
            try:
                dependency = plain(Path(item))
                if (not dependency.is_dir()
                        or plain(self.text(dependency, "rev-parse", "--show-toplevel")) != dependency):
                    raise ValueError("External input must be its actual physical Git root")
                external.append({"path": str(dependency), "source": self.source(dependency, payload=payload)})
            except Exception as error:
                if observation_errors is None:
                    raise
                observation_errors.append({"path": item, "message": str(error),
                                           "reasonCode": getattr(error, "code", None),
                                           "details": getattr(error, "details", None)})
        value = {"sourceView": view, "sourceRoot": str(root),
                 "source": engine if engine is not None else self.source(root, payload=payload),
                 "externalRoots": external}
        return root, {**value, "sourceDigest": sha(encoded(value)) if not observation_errors else None}

    def new_worktree(self, *, merge=False):
        name = identifier(self.payload["name"])
        reference = self.session + "-" + name
        if len(reference) > 160 or reference.endswith("."):
            raise ValueError("Managed worktree name is too long")
        path = plain(self.worktrees / reference)
        if path.exists() or self.state.get("management_worktree", reference):
            raise ValueError("Worktree exists; observe the original operation")
        base = self.payload["baseHead"]
        if not isinstance(base, str) or not SHA.fullmatch(base):
            raise ValueError("An immutable base commit is required")
        base = self.text(self.repo, "rev-parse", base + "^{commit}")
        self.worktrees.mkdir(parents=True, exist_ok=True)
        branch = None if merge else "codex/jenkins/" + self.session + "/" + name
        args = ["worktree", "add", "--detach"] if merge else ["worktree", "add", "-b", branch]
        self.git(self.repo, *args, "--", str(path), base)
        record = {"status": "ready", "path": str(path), "branch": branch, "baseHead": base,
                  "sessionId": self.session, "requestId": self.request, "operationId": self.operation,
                  "ownedPaths": self.owned, "driverDigest": self.config["driverDigest"]}
        self.state.put("management_worktree", reference, record, expected_version=0)
        return reference, path, record

    def create_branch(self):
        name = identifier(self.payload["name"])
        base = self.payload["baseHead"]
        if not isinstance(base, str) or not SHA.fullmatch(base):
            raise ValueError("An immutable base commit is required")
        branch = "codex/jenkins/" + self.session + "/" + name
        head = self.text(self.repo, "rev-parse", base + "^{commit}")
        self.git(self.repo, "branch", "--", branch, head)
        return {"status": "complete", "branch": branch, "head": head}

    def prepare_merge(self):
        values = self.payload.get("inputs")
        if not isinstance(values, list) or not 1 <= len(values) <= 8:
            raise ValueError("Merge requires one through eight registered worktrees")
        if any("patchOperationKeys" in item for item in values):
            return self.prepare_dirty_merge(values)
        heads, inputs, paths = [], [], set()
        base = self.payload["baseHead"]
        if not isinstance(base, str) or not SHA.fullmatch(base):
            raise ValueError("An immutable base commit is required")
        for item in values:
            record, root = self.wt(item["worktreeId"], own=False)
            head = self.text(root, "rev-parse", "HEAD")
            if head != item.get("head") or self.text(root, "status", "--porcelain"):
                raise ValueError("Merge source must retain its requested clean HEAD")
            heads.append(head)
            inputs.append({"worktreeId": item["worktreeId"], "head": head, "sessionId": record["sessionId"]})
            paths.update(p.decode("utf-8") for p in self.git(self.repo, "diff", "--name-only", "-z",
                                                         base, head, "--").stdout.split(b"\0") if p)
        self.scoped(sorted(paths))
        original_head = self.text(self.repo, "rev-parse", "HEAD")
        ref, root, record = self.new_worktree(merge=True)
        result = self.git(root, "merge", "--no-commit", "--no-ff", "--no-edit", "--", *heads, check=False)
        conflicts = self.text(root, "diff", "--name-only", "--diff-filter=U").splitlines()
        status = ("conflict" if conflicts else "failed") if result.returncode else "merge-prepared"
        updated = {**record, "status": status, "inputs": inputs, "conflicts": conflicts}
        row = self.state.get("management_worktree", ref)
        self.state.put("management_worktree", ref, updated, expected_version=row["version"])
        if self.text(self.repo, "rev-parse", "HEAD") != original_head:
            raise ValueError("Shared checkout HEAD changed during merge preparation")
        return {"status": status, "worktreeId": ref, "path": str(root), "inputs": inputs,
                "conflicts": conflicts, "resultTree": self.text(root, "write-tree") if not conflicts else None,
                "mainHeadBefore": original_head, "mainHeadAfter": original_head,
                "commitPerformed": False, "pushPerformed": False,
                "source": self.source(root) if not conflicts else None}

    def patch_input(self, reference, hashes):
        from tools.jenkins.source.unified_patch import parse_unified_patch
        patch_path = plain(reference["path"])
        admitted = tuple(self.repo / ".jenkins/state" / name for name in ("patches", "controls", "migration-baseline"))
        if not any(patch_path.is_relative_to(base) for base in admitted) or patch_path.stat().st_size > 2 * 1024**2:
            raise ValueError("Patch must be an immutable bounded State input")
        raw = patch_path.read_bytes()
        if sha(raw) != reference["sha256"]:
            raise ValueError("Patch input digest changed")
        patch = parse_unified_patch(raw.decode("utf-8", "strict"))
        if not isinstance(hashes, dict) or not 1 <= len(hashes) <= 128:
            raise ValueError("Explicit bounded beforeHashes required")
        paths = list(patch.paths)
        if any(path.startswith((".git/", ".codex/", "target/")) or path == ".git" for path in paths):
            raise ValueError("Patch may not change managed control or compiler directories")
        if any(not any(path.casefold() == scope.casefold() or path.casefold().startswith(scope.casefold() + "/")
                       for scope in self.owned) for path in paths):
            raise ValueError("Patch differs from this task's ownedPaths")
        self.scoped(paths)
        return patch

    def historical_patch_continuation(self, key, row):
        """Consume one pinned completed patch from a reviewed same-boot predecessor."""
        operation = row["payload"]
        original, result = operation["payload"], operation["result"]
        previous = original["runtimeOperationId"]
        previous_driver = original.get("driverDigest")
        if (result.get("runtimeOperationId") != previous or result.get("driverDigest") != previous_driver
                or not isinstance(previous_driver, str) or not re.fullmatch(r"[0-9a-f]{64}", previous_driver)):
            raise ValueError("Completed patch runtime/driver identities disagree")
        if previous == self.config["runtimeOperationId"]:
            if previous_driver != self.config["driverDigest"]:
                raise ValueError("Current-runtime patch must retain the current driver")
            return None
        from tools.jenkins.deployment.lost_recovery_runtime import (
            SAME_BOOT_VARIANT, reviewed_marker, validate_interruption_receipt)
        from tools.jenkins.deployment.owned_upgrade import _committed_command
        from tools.jenkins.deployment.recovery_keeper import (
            DESCRIPTOR_KIND, _read, _reference, _release, _runtime_receipt)
        from tools.jenkins.processes.boot import windows_boot_filetime
        state = plain(self.repo / ".jenkins/state")
        descriptor_path = state / "formal-lifecycle/recovery-keeper.json"
        command_path = state / "recovery-lifecycle/command.json"
        lineage = getattr(self, "_patch_recovery_lineage", None)
        if lineage is None:
            descriptor_raw = plain(descriptor_path).read_bytes()
            descriptor = _read(descriptor_path)
            command_raw = plain(command_path).read_bytes()
            command = _read(command_path)
            runtime = descriptor.get("runtime", {})
            if (descriptor.get("schemaVersion") != 1 or descriptor.get("kind") != DESCRIPTOR_KIND
                    or descriptor.get("admission") != "idle" or descriptor.get("automaticRestart") is not False
                    or any(descriptor.get(name) is not False for name in
                           ("runtimeAcceptance", "cargoAcceptance", "historicalHoldsReleased"))
                    or runtime.get("state") != "running"
                    or runtime.get("operation", {}).get("operationId") != self.config["runtimeOperationId"]
                    or runtime.get("selected", {}).get("driverDigest") != self.config["driverDigest"]
                    or runtime.get("selected", {}).get("runtimeOperationId") != self.config["runtimeOperationId"]
                    or descriptor.get("bootFileTime") != str(windows_boot_filetime())):
                raise ValueError("Historical patches require the exact live recovered runtime descriptor")
            release, target = _release(self.manager, descriptor.get("release"), current_source=False)
            planned = reviewed_marker(state, release)
            if (not planned or planned.get("recoveryVariant") != SAME_BOOT_VARIANT
                    or planned.get("originalCommandStatus") != "committed"
                    or planned.get("operationId") != previous
                    or planned.get("previousBootFileTime") != planned.get("bootFileTime")
                    or target.get("driverDigest") != self.config["driverDigest"]):
                raise ValueError("Completed patch is outside the single reviewed same-boot recovery")
            receipt = _runtime_receipt(self.manager, descriptor, release, target)
            _committed_command(state, descriptor, receipt, command)
            interruption = validate_interruption_receipt(state, descriptor, release)
            archives = interruption["originalArtifacts"]
            archived = {name: _reference(state, binding["archive"]) for name, binding in archives.items()}
            old_descriptor, old_receipt = archived["originalDescriptor"], archived["previousRuntimeReceipt"]
            old_runtime = old_descriptor.get("runtime", {})
            old_selected = old_runtime.get("selected", {})
            # _release verified the complete immutable predecessor snapshot.
            # Bind that exact prepared selector to the reviewed original archives.
            preparation = _reference(state, release["preparation"])
            if (old_descriptor.get("kind") != DESCRIPTOR_KIND or old_descriptor.get("admission") != "idle"
                    or old_runtime.get("state") != "running"
                    or old_runtime.get("operation", {}).get("operationId") != previous
                    or old_selected.get("runtimeOperationId") != previous
                    or old_selected.get("driverDigest") != previous_driver
                    or old_selected != preparation.get("previousSelectedDriver")
                    or archives["previousSelector"]["original"]["sha256"] != preparation.get("previousSelectorSha256")
                    or sha(encoded(old_selected)) != planned.get("selectedDriverDigest")
                    or old_runtime.get("operation") != archived["previousOperation"]
                    or old_runtime.get("host") != archived["previousHost"]
                    or old_selected != archived["previousSelector"]
                    or old_descriptor.get("runtimeReceipt") != planned.get("previousRuntimeReceipt")
                    or old_descriptor.get("release") != planned.get("previousRecoveryRelease")
                    or old_descriptor.get("bootFileTime") != descriptor["bootFileTime"]
                    or old_receipt.get("kind") != "recovery-keeper-runtime"
                    or old_receipt.get("runtime") != old_runtime
                    or old_receipt.get("keeper") != old_descriptor.get("keeper")
                    or old_receipt.get("release") != old_descriptor.get("release")
                    or old_receipt.get("bootFileTime") != descriptor["bootFileTime"]
                    or any(old_receipt.get(name) is not False for name in
                           ("runtimeAcceptance", "cargoAcceptance", "nativeProofSynthesized",
                            "historicalHoldsReleased", "queueReplayRequested"))
                    or self.manager._read() != runtime["operation"]
                    or _read(state / "deployment/host.json") != runtime["host"]
                    or _read(state / "deployment/driver.json") != runtime["selected"]):
                raise ValueError("Historical patch lineage differs from the exact archived and current runtime")
            quarantine = planned.get("managementQuarantine", {})
            if (not isinstance(quarantine.get("records"), list)
                    or not isinstance(quarantine.get("currentRecords"), list)
                    or any(quarantine.get(name) is not False for name in
                           ("nativeProofIssued", "artifactAcceptance", "executionHoldsReleaseAllowed"))):
                raise ValueError("Historical patch lacks its reviewed management quarantine")
            lineage = {"previousRuntime": previous, "previousDriver": previous_driver, "quarantine": quarantine,
                       "descriptorRaw": descriptor_raw, "commandRaw": command_raw,
                       "evidence": {
                           "descriptor": {"path": str(descriptor_path), "sha256": sha(descriptor_raw)},
                           "committedCommand": {"path": str(command_path), "sha256": sha(command_raw)},
                           "runtimeReceipt": descriptor["runtimeReceipt"], "release": descriptor["release"],
                           "reviewedPlan": release["lostRecoveryRuntime"]["plan"],
                           "interruption": descriptor["lostRecoveryRuntimeInterruption"],
                           "predecessorArtifacts": archives, "preparation": release["preparation"],
                           "predecessorDriver": {"driverDigest": previous_driver, "root": old_selected["root"],
                                                 "manifest": old_selected["manifest"]},
                           "reviewedManagementRecordsDigest": quarantine["recordsDigest"]}}
            self._patch_recovery_lineage = lineage
        if (previous != lineage["previousRuntime"] or previous_driver != lineage["previousDriver"]
                or plain(descriptor_path).read_bytes() != lineage["descriptorRaw"]
                or plain(command_path).read_bytes() != lineage["commandRaw"]):
            raise ValueError("Historical patch recovery lineage changed")
        identities = {name: original.get(name) for name in
                      ("repositoryId", "sessionId", "requestId", "operationId")}
        if (identities["repositoryId"] != self.repo_id
                or identities["sessionId"] != operation.get("sessionId")
                or original.get("action") != operation.get("action")
                or any(operation.get(name) != identities[name] or result.get(name) != identities[name]
                       for name in ("requestId", "operationId", "sessionId"))
                or key != sha(encoded(identities))
                or operation.get("inputDigest") != sha(encoded(
                    {name: value for name, value in original.items() if name != "observationOnly"}))):
            raise ValueError("Historical patch's original operation identity is inconsistent")
        pin = {"domain": "management_operation", "key": key, "version": row["version"],
               "payloadDigest": sha(encoded(operation))}
        matching = [value for value in lineage["quarantine"]["records"]
                    if value.get("domain") == pin["domain"] and value.get("key") == key]
        if (matching != [pin] or {"domain": pin["domain"], "key": key}
                not in lineage["quarantine"]["currentRecords"]):
            raise ValueError("Completed patch no longer matches its exact reviewed quarantine pin")
        return {"kind": "completed-patch-same-boot-continuation", "operationPin": pin,
                "previousRuntimeOperationId": previous, "previousDriverDigest": previous_driver,
                "runtimeOperationId": self.config["runtimeOperationId"],
                "driverDigest": self.config["driverDigest"], "sessionId": identities["sessionId"],
                "sourceDigest": result["source"]["sourceDigest"], "lineage": lineage["evidence"],
                "nativeProofIssued": False, "artifactAcceptance": False}

    def prepare_dirty_merge(self, values):
        """Replay completed author-scoped patch operations without Git commits."""
        from tools.jenkins.source import _git_base_tree
        from tools.jenkins.source.unified_patch import render_patch_bytes
        base = self.payload["baseHead"]
        if not isinstance(base, str) or not SHA.fullmatch(base):
            raise ValueError("An immutable base commit is required")
        base = self.text(self.repo, "rev-parse", base + "^{commit}")
        tree = _git_base_tree(self.repo, base)
        def file_hashes(source):
            hashes = {}
            for path, checksum in source["files"].items():
                canonical = path.casefold()
                if canonical in hashes:
                    raise ValueError("Source contains case-colliding file paths")
                hashes[canonical] = checksum
            return hashes
        plans, inputs, all_paths, observations = [], [], set(), []
        continuations = []
        def verify_continuations():
            for evidence in continuations:
                key = evidence["operationPin"]["key"]
                row = self.state.get("management_operation", key)
                if not row or self.historical_patch_continuation(key, row) != evidence:
                    raise ValueError("Historical completed patch changed during merge preparation")
        for item in values:
            record, root = self.wt(item["worktreeId"], own=False)
            captured = self.source(root)
            keys = item.get("patchOperationKeys")
            if (captured["head"] != base or item.get("head") != base
                    or item.get("sourceDigest") != captured["sourceDigest"]
                    or not isinstance(keys, list) or not 1 <= len(keys) <= 32
                    or len(keys) != len(set(keys))
                    or any(not isinstance(key, str) or not re.fullmatch(r"[0-9a-f]{64}", key) for key in keys)):
                raise ValueError("Dirty merge requires the exact base, source and completed patch operation sequence")
            images, operations, previous_files = {}, [], None
            for key in keys:
                row = self.state.get("management_operation", key)
                operation = row["payload"] if row else {}
                original, result = operation.get("payload", {}), operation.get("result", {})
                if (operation.get("phase") != "complete" or operation.get("action") != "apply-patch"
                        or operation.get("sessionId") != record["sessionId"] or result.get("status") != "complete"
                        or original.get("repositoryId") != self.repo_id or original.get("worktreeId") != item["worktreeId"]
                        or original.get("expectedHead") != base
                        or result.get("operationKey") != key or result.get("sessionId") != record["sessionId"]
                        or result.get("driverDigest") != original.get("driverDigest")
                        or result.get("runtimeOperationId") != original.get("runtimeOperationId")):
                    raise ValueError("Dirty merge lacks the original author's completed patch operation")
                continuation = self.historical_patch_continuation(key, row)
                if continuation is not None:
                    continuations.append(continuation)
                proof_source = result.get("source", {})
                if (proof_source.get("head") != base or proof_source.get("sourceDigest") !=
                        sha(encoded({name: value for name, value in proof_source.items() if name != "sourceDigest"}))):
                    raise ValueError("Completed patch source identity is inconsistent")
                patch = self.patch_input(original["patchReference"], original["beforeHashes"])
                self.state.check_authorization(self.repo_id, record["sessionId"], "implementation", list(patch.paths))
                preimages = {}
                for path in patch.paths:
                    if path not in images:
                        entry = tree.get(path.casefold())
                        images[path] = self.git(self.repo, "cat-file", "blob", entry[1]).stdout if entry else None
                    preimages[path] = images[path]
                rendered = render_patch_bytes(patch, preimages, original["beforeHashes"])
                after = {path: None if data is None else sha(data) for path, data in rendered.items()}
                proof_files = file_hashes(proof_source)
                if result.get("afterHashes") != after or any(proof_files.get(path) != checksum
                                                          for path, checksum in after.items()):
                    raise ValueError("Completed patch postimages differ from its operation evidence")
                if previous_files is not None:
                    expected_files = dict(previous_files)
                    for path, checksum in after.items():
                        if checksum is None and path not in proof_files:
                            expected_files.pop(path, None)
                        else:
                            expected_files[path] = checksum
                    if proof_files != expected_files:
                        raise ValueError("Completed patch sequence changes unregistered source bytes")
                previous_files = proof_files
                images.update(rendered)
                operations.append(original)
            if result["source"] != captured:
                raise ValueError("Worktree differs from its last completed patch source")
            changed = set(p.decode("utf-8", "strict") for p in
                          self.git(root, "diff", "--name-only", "-z", "HEAD", "--").stdout.split(b"\0") if p)
            changed.update(p.decode("utf-8", "strict") for p in
                           self.git(root, "ls-files", "--others", "--exclude-standard", "-z").stdout.split(b"\0") if p)
            if (not changed or not {path.casefold() for path in changed} <= {path.casefold() for path in images}
                    or any(file_hashes(captured).get(path) != (None if data is None else sha(data))
                           for path, data in images.items())):
                raise ValueError("Dirty worktree contains changes outside its exact patch sequence")
            conflicts = {path.casefold() for path in all_paths} & {path.casefold() for path in images}
            if conflicts:
                return {"status": "conflict", "conflicts": sorted(conflicts), "inputs": inputs,
                        "historicalPatchContinuations": continuations,
                        "commitPerformed": False, "pushPerformed": False}
            all_paths.update(images)
            if len(all_paths) > 128:
                raise ValueError("Merged patch scope exceeds 128 paths")
            plans.extend(operations)
            observations.append((root, captured))
            inputs.append({"worktreeId": item["worktreeId"], "head": base, "sessionId": record["sessionId"],
                           "sourceDigest": captured["sourceDigest"], "patchOperationKeys": keys})
        self.scoped(sorted(all_paths))
        verify_continuations()
        if any(self.source(root) != captured for root, captured in observations):
            raise ValueError("Merge input changed before candidate preparation")
        original_head = self.text(self.repo, "rev-parse", "HEAD")
        ref, root, record = self.new_worktree(merge=True)
        for operation in plans:
            self.patch_worktree(ref, base, operation["patchReference"], operation["beforeHashes"], capture=False)
        self.git(root, "add", "--", *sorted(all_paths))
        source = self.source(root)
        if (self.text(self.repo, "rev-parse", "HEAD") != original_head
                or any(self.source(path) != captured for path, captured in observations)):
            raise ValueError("An original merge input changed during candidate preparation")
        verify_continuations()
        row = self.state.get("management_worktree", ref)
        self.state.put("management_worktree", ref,
                       {**record, "status": "merge-prepared", "inputs": inputs, "conflicts": [],
                        "historicalPatchContinuations": continuations},
                       expected_version=row["version"])
        return {"status": "merge-prepared", "worktreeId": ref, "path": str(root), "inputs": inputs,
                "conflicts": [], "resultTree": self.text(root, "write-tree"), "source": source,
                "mainHeadBefore": original_head, "mainHeadAfter": original_head,
                "commitPerformed": False, "pushPerformed": False, "integrationKind": "completed-patch-replay",
                "historicalPatchContinuations": continuations}

    def apply_patch(self):
        return self.patch_worktree(self.payload["worktreeId"], self.payload["expectedHead"],
                                   self.payload["patchReference"], self.payload["beforeHashes"])

    def patch_worktree(self, worktree_id, expected_head, reference, hashes, *, capture=True):
        from tools.jenkins.source.unified_patch import render_patch_bytes
        _record, root = self.wt(worktree_id)
        if self.text(root, "rev-parse", "HEAD") != expected_head:
            raise ValueError("Managed worktree HEAD changed")
        patch = self.patch_input(reference, hashes)
        paths = list(patch.paths)
        targets, before = {}, {}
        for path in paths:
            target = plain(root / path)
            if not target.is_relative_to(root) or (target.exists() and not target.is_file()):
                raise ValueError("Patch target is not a plain managed worktree file")
            ancestor = target.parent
            while ancestor != root:
                if ancestor.exists() and not ancestor.is_dir():
                    raise ValueError("Patch parent is not a directory")
                ancestor = ancestor.parent
            targets[path] = target
            before[path] = target.read_bytes() if target.is_file() else None
        postimages = render_patch_bytes(patch, before, hashes)
        # All files have passed scope, path, preimage and hunk checks before writes.
        _current, current_root = self.wt(worktree_id)
        if current_root != root or self.text(root, "rev-parse", "HEAD") != expected_head:
            raise ValueError("Managed worktree changed before patch writes")
        for path in paths:
            target, data = plain(targets[path]), postimages[path]
            if data is None:
                if not target.is_file() or target.read_bytes() != before[path]:
                    raise ValueError("Patch deletion preimage changed")
                target.unlink()
            elif before[path] is None:
                if target.exists():
                    raise ValueError("Patch creation target appeared")
                target.parent.mkdir(parents=True, exist_ok=True)
                plain(target)
                with target.open("xb") as stream:
                    stream.write(data)
                    stream.flush()
                    os.fsync(stream.fileno())
            else:
                if not target.is_file():
                    raise ValueError("Patch replacement preimage disappeared")
                with target.open("r+b") as stream:
                    if stream.read() != before[path]:
                        raise ValueError("Patch replacement preimage changed")
                    stream.seek(0)
                    stream.write(data)
                    stream.truncate()
                    stream.flush()
                    os.fsync(stream.fileno())
        after = {}
        for path in paths:
            target = plain(targets[path])
            actual = target.read_bytes() if target.is_file() else None
            if actual != postimages[path] or (actual is None and target.exists()):
                raise ValueError("Patch postimage differs from actual worktree bytes")
            after[path] = None if actual is None else sha(actual)
        return {"status": "complete", "worktreeId": worktree_id,
                "patchReference": reference, "afterHashes": after, "source": self.source(root) if capture else None}

    def validation_delivery(self, key, worktree_id):
        """Read one completed managed Cargo delivery and its original native proof."""
        if not isinstance(key, str) or not re.fullmatch(r"[0-9a-f]{64}", key):
            raise ValueError("An exact completed validation operation key is required")
        row = self.state.get("management_operation", key)
        operation = row["payload"] if row else {}
        original, result = operation.get("payload", {}), operation.get("result", {})
        captured = self.state.get("management_source", key)
        if (operation.get("phase") != "complete" or operation.get("action") != "validate"
                or operation.get("sessionId") != self.session or result.get("status") != "passed"
                or original.get("repositoryId") != self.repo_id or original.get("worktreeId") != worktree_id
                or original.get("sourceView", "worktree") != "worktree"
                or original.get("driverDigest") != self.config["driverDigest"]
                or original.get("runtimeOperationId") != self.config["runtimeOperationId"]
                or not captured or result.get("sourceDigest") != captured["payload"]["sourceDigest"]
                or not validation_inputs_verified(result)):
            raise ValueError("Worktree release requires an actual matching successful managed validation")
        root, before = self.validation_with_cache(payload=original)
        if before != captured["payload"]:
            raise ValueError("Validated candidate or dependency closure changed before worktree release")
        from tools.jenkins.resources.paths import canonical_build_root, physical_path_under
        approved = canonical_build_root(self.build_root(payload=original))
        report = physical_path_under(approved, approved.namespace() / "management/reports" / key)
        documents = {}
        for name, filename in (("request", "request.json"), ("nativeTerminal", "native-terminal.json")):
            reference = result[name]
            path = plain(reference["path"])
            if path != report / filename:
                raise ValueError("Validation delivery is outside its original managed report")
            raw = path.read_bytes()
            if sha(raw) != reference["sha256"]:
                raise ValueError("Original validation delivery changed")
            documents[name] = json.loads(raw)
        request, native = documents["request"], documents["nativeTerminal"]
        execution_root = physical_path_under(approved, request["executionRoot"], allow_missing=False)
        expected = native_terminal_proof(native, result["nativeTerminal"])
        proof_id = "management-" + key + "-native"
        native_row = self.state.get("native_job", proof_id)
        proof = native_row["payload"] if native_row else {}
        if (request["payload"] != original or request["source"] != before
                or request["command"] != native["command"] or plain(native["cwd"]) != execution_root
                or result.get("executionRoot") != str(execution_root)
                or request.get("preparation") != result.get("preparation")
                or not validation_inputs_verified(result, request)
                or native.get("rootExitCode") != 0 or expected["complete"] is not True
                or proof.get("owner") != "management-" + key or proof.get("status") != "terminal"
                or proof.get("nativeEvidence") != native or proof.get("completeProof") != expected
                or proof.get("terminalProof") != expected):
            raise ValueError("Validation lacks its original complete retained-Job termination proof")
        return row, root, before, {"request": result["request"], "nativeTerminal": result["nativeTerminal"],
                                  "nativeProofRef": proof_id, "sourceDigest": before["sourceDigest"]}

    def worktree_release_inventory(self, root, source):
        """Admit every actual file, preserving ignored or otherwise unattributed bytes."""
        expected = {name.casefold(): checksum for name, checksum in source["files"].items()
                    if checksum is not None}
        files, directories = [], []
        pending = [root]
        while pending:
            directory = pending.pop()
            directories.append(str(directory.relative_to(root)))
            with os.scandir(directory) as entries:
                for entry in entries:
                    path = plain(Path(entry.path))
                    relative = path.relative_to(root).as_posix()
                    if entry.is_dir(follow_symlinks=False):
                        pending.append(path)
                    elif entry.is_file(follow_symlinks=False):
                        before = path.stat()
                        checksum = sha(path.read_bytes())
                        after = path.stat()
                        if (before.st_size != after.st_size or before.st_mtime_ns != after.st_mtime_ns
                                or relative.casefold() != ".git" and expected.get(relative.casefold()) != checksum):
                            raise ValueError("Worktree contains changing or unattributed file bytes: " + relative)
                        files.append({"path": relative, "sha256": checksum, "bytes": after.st_size,
                                      "mtimeNs": after.st_mtime_ns})
                    else:
                        raise ValueError("Worktree contains an unowned physical filesystem entry")
        actual = {value["path"].casefold() for value in files if value["path"].casefold() != ".git"}
        if actual != set(expected) or not any(value["path"].casefold() == ".git" for value in files):
            raise ValueError("The complete physical worktree inventory differs from its source hashes")
        return {"files": sorted(files, key=lambda value: value["path"].casefold()),
                "directories": sorted(directories), "fileCount": len(files),
                "logicalBytes": sum(value["bytes"] for value in files)}

    def release_worktree(self):
        """Remove one validated input worktree through Jenkins' retained native Git owner."""
        from tools.jenkins.workflow.planning import _environment_policy
        import shutil
        reference = self.payload["worktreeId"]
        record, root = self.wt(reference)
        before = self.source(root)
        keys = self.payload.get("patchOperationKeys")
        if (record["sessionId"] != self.session or before["head"] != record.get("baseHead")
                or before["head"] != self.payload.get("expectedHead")
                or before["sourceDigest"] != self.payload.get("sourceDigest")
                or not isinstance(keys, list) or not 1 <= len(keys) <= 32
                or len(keys) != len(set(keys))
                or any(not isinstance(key, str) or not re.fullmatch(r"[0-9a-f]{64}", key) for key in keys)):
            raise ValueError("Release needs this session's exact full source and completed patch keys")
        self.scoped(record["ownedPaths"], "gc")
        branch_before = self.text(self.repo, "rev-parse", record["branch"]) if record.get("branch") else None
        if self.text(self.repo, "rev-parse", record["baseHead"] + "^{commit}") != record["baseHead"]:
            raise ValueError("Original worktree base Git object is unavailable")
        merge_key, validation_key = self.payload.get("mergeOperationKey"), self.payload.get("validationKey")
        if not isinstance(merge_key, str) or not re.fullmatch(r"[0-9a-f]{64}", merge_key):
            raise ValueError("Release requires the exact completed combined candidate operation")
        merge_row = self.state.get("management_operation", merge_key)
        merge = merge_row["payload"] if merge_row else {}
        merged = merge.get("result", {})
        if (merge.get("phase") != "complete" or merge.get("action") != "prepare-merge"
                or merge.get("sessionId") != self.session or merged.get("status") != "merge-prepared"
                or merged.get("driverDigest") != self.config["driverDigest"]
                or merged.get("runtimeOperationId") != self.config["runtimeOperationId"]
                or merged.get("integrationKind") != "completed-patch-replay"
                or merged.get("commitPerformed") is not False or merged.get("pushPerformed") is not False):
            raise ValueError("Release lacks its real combined candidate without product commits")
        inputs = [item for item in merged.get("inputs", []) if item.get("worktreeId") == reference]
        if (len(inputs) != 1 or inputs[0].get("sessionId") != self.session
                or inputs[0].get("head") != before["head"] or inputs[0].get("sourceDigest") != before["sourceDigest"]
                or inputs[0].get("patchOperationKeys") != keys):
            raise ValueError("Combined candidate does not preserve this exact completed input")
        candidate_id = merged["worktreeId"]
        candidate_record, candidate = self.wt(candidate_id)
        validation_row, validated_root, validation_source, delivery = self.validation_delivery(validation_key, candidate_id)
        if (candidate == root or validated_root != candidate or candidate_record.get("inputs") != merged["inputs"]
                or merged.get("source") != validation_source["source"]):
            raise ValueError("Candidate delivery differs from its original combined source")
        pins, continuations, retained_patches = [], [], []
        for key in keys:
            row = self.state.get("management_operation", key)
            operation = row["payload"] if row else {}
            original, result = operation.get("payload", {}), operation.get("result", {})
            if (operation.get("phase") != "complete" or operation.get("action") != "apply-patch"
                    or operation.get("sessionId") != self.session or result.get("status") != "complete"
                    or original.get("repositoryId") != self.repo_id or original.get("worktreeId") != reference
                    or original.get("expectedHead") != before["head"]
                    or result.get("driverDigest") != original.get("driverDigest")
                    or result.get("operationKey") != key or result.get("sessionId") != self.session):
                raise ValueError("Worktree has a foreign or unfinished patch operation")
            patch = self.patch_input(original["patchReference"], original["beforeHashes"])
            self.state.check_authorization(self.repo_id, self.session, "gc", list(patch.paths))
            source = result["source"]
            if source["sourceDigest"] != sha(encoded({name: value for name, value in source.items() if name != "sourceDigest"})):
                raise ValueError("Completed patch source hashes are inconsistent")
            continuation = self.historical_patch_continuation(key, row)
            if continuation is not None:
                continuations.append(continuation)
            pins.append({"domain": "management_operation", "key": key, "version": row["version"],
                         "payloadDigest": sha(encoded(operation))})
            retained_patches.append(original["patchReference"])
        if result["source"] != before:
            raise ValueError("Input no longer matches its final completed patch's full source")
        inventory = self.worktree_release_inventory(root, before)
        environment, _trusted, _policy = _environment_policy(self.state, self.repo)
        git_path = shutil.which("git", path=environment.get("PATH"))
        if not git_path:
            raise ValueError("Trusted Jenkins environment has no native Git executable")
        git_path = plain(Path(git_path))
        command = self.git_command(self.repo, "worktree", "remove", "--force", "--", str(root))
        command[0] = str(git_path)
        git_digest = sha(git_path.read_bytes())
        owner = "management-" + self.key
        report = plain(self.outputs() / "reports" / self.key)
        helper_path = plain(self.repo / "tools/dev/local_cargo.py")
        spec = importlib.util.spec_from_file_location("jenkins_release_native", helper_path)
        helper = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(helper)
        if report.exists():
            raise ValueError("Release report exists; reconcile the original operation")
        scratch = plain(report / "tmp")
        retained = {"candidate": candidate, "reports": report, "tmp": scratch,
                    "gitObjects": self.repo / ".git/objects"}
        self.artifact_manifest(root, retained)
        request_ref = write_json(report / "request.json", {
            "payload": self.payload, "source": before, "inventory": inventory, "command": command,
            "cwd": str(self.repo), "gitExecutableSha256": git_digest,
            "patchPins": pins, "retainedPatches": retained_patches, "mergeOperationKey": merge_key,
            "validationKey": validation_key, "validationDelivery": delivery,
            "historicalPatchContinuations": continuations, "branchPreserved": record.get("branch"),
            "baseHeadPreserved": record.get("baseHead"), "candidatePreserved": candidate_id})
        # Serialise with managed Git mutations; CAS closes admission to new validation.
        with self.state.transaction() as connection:
            current = self.state.get("management_worktree", reference, connection=connection)
            if not current or current["payload"] != record:
                raise ValueError("Input worktree changed before release admission")
            patch_rows = []
            related_keys = set(keys)
            for row in self.state.list("management_operation", connection=connection):
                payload = row["payload"]
                original = payload.get("payload", {})
                related = (original.get("worktreeId") == reference
                           or any(item.get("worktreeId") == reference for item in original.get("inputs", [])
                                  if isinstance(item, dict)))
                if row["key"] == self.key or not related:
                    continue
                related_keys.add(row["key"])
                if payload.get("phase") != "complete":
                    raise ValueError("Worktree has pending or unknown managed work")
                if payload.get("action") == "apply-patch":
                    if payload.get("sessionId") != self.session:
                        raise ValueError("Worktree patch ownership differs")
                    patch_rows.append(row["key"])
            if set(patch_rows) != set(keys):
                raise ValueError("Release omitted an original worktree patch operation")
            owners = {"management-" + key for key in related_keys}
            for execution in self.state.list("execution", connection=connection):
                value = execution["payload"]
                if execution["key"] in owners or value.get("sourceRoot") == str(root):
                    if value.get("status") not in {"passed", "failed", "cancelled"}:
                        raise ValueError("Worktree validation is still active or unknown")
                    if not value.get("durableUnlaunched"):
                        native = self.state.get("native_job", value.get("nativeProofRef", ""), connection=connection)
                        if not native or native["payload"].get("completeProof", {}).get("complete") is not True:
                            raise ValueError("Previous worktree native termination is unproven")
            for native in self.state.list("native_job", connection=connection):
                value = native["payload"]
                cwd = value.get("nativeEvidence", {}).get("cwd")
                if value.get("owner") in owners or value.get("executionId") in owners or cwd == str(root):
                    if value.get("status") != "terminal" or value.get("completeProof", {}).get("complete") is not True:
                        raise ValueError("A retained worktree process still lacks complete EOF/native proof")
            for pin in pins:
                row = self.state.get(pin["domain"], pin["key"], connection=connection)
                if not row or row["version"] != pin["version"] or sha(encoded(row["payload"])) != pin["payloadDigest"]:
                    raise ValueError("Completed worktree patch evidence changed")
            for key, saved in ((merge_key, merge_row), (validation_key, validation_row)):
                if self.state.get("management_operation", key, connection=connection) != saved:
                    raise ValueError("Candidate acceptance evidence changed")
            self.state.put("management_worktree", reference, {**record, "status": "releasing",
                           "releaseOperationKey": self.key, "releaseRequest": request_ref},
                           expected_version=current["version"], connection=connection)
            self.state.put("execution", owner, {
                           "kind": "management_owned_git", "status": "running", "sourceRoot": str(root),
                           "repositoryId": self.repo_id, "sessionId": self.session, "requestId": self.request,
                           "operationId": self.operation, "driverDigest": self.config["driverDigest"],
                           "runtimeOperationId": self.config["runtimeOperationId"]},
                           expected_version=0, connection=connection)
            self.state.put("execution_launch", owner, {"status": "claimed", "kind": "management_owned_git",
                           "executionId": owner, "worktreeId": reference, "request": request_ref,
                           "command": command, "cwd": str(self.repo)}, expected_version=0, connection=connection)
        # An interrupted or failed removal leaves 'releasing' and preserves evidence.
        if self.source(root) != before or self.worktree_release_inventory(root, before) != inventory:
            raise ValueError("Worktree physical bytes changed after release admission")
        if sha(git_path.read_bytes()) != git_digest:
            raise ValueError("Native Git executable changed before release")
        scratch.mkdir(parents=True, exist_ok=False)
        environment.update(GIT_TERMINAL_PROMPT="0", PYTHONDONTWRITEBYTECODE="1",
                           TEMP=str(scratch), TMP=str(scratch), TMPDIR=str(scratch))
        volume_before = helper.free_bytes(root.parent)
        native = helper._run_owned_command(command, self.repo, environment, report, 1800)
        native_ref = write_json(report / "native-terminal.json", native)
        proof = native_terminal_proof(native, native_ref)
        if native.get("command") != command or native.get("cwd") != str(self.repo) or not proof["complete"]:
            raise ValueError("Worktree removal lacks exact complete native termination")
        proof_id = owner + "-native"
        self.state.put("native_job", proof_id, {"owner": owner, "executionId": owner, "role": "management_owned_git",
                       "status": "terminal", "terminalProof": proof, "completeProof": proof,
                       "nativeEvidence": native, "requestId": self.request, "operationId": self.operation,
                       "worktreeId": reference, "pid": native["pid"], "creationTime": native["creationTime"],
                       "birthToken": native["creationTime"], "executable": proof["identity"]["executable"],
                       "commandDigest": proof["identity"]["commandDigest"],
                       "formalAcceptance": False}, expected_version=0)
        execution = self.state.get("execution", owner)
        self.state.put("execution", owner, {**execution["payload"],
                       "status": "passed" if native["rootExitCode"] == 0 else "failed",
                       "nativeProofRef": proof_id}, expected_version=execution["version"])
        launch = self.state.get("execution_launch", owner)
        self.state.put("execution_launch", owner, {**launch["payload"], "status": "terminal",
                       "nativeProofRef": proof_id}, expected_version=launch["version"])
        if native["rootExitCode"] != 0 or root.exists():
            raise ValueError("Native Git did not remove the exact managed worktree")
        registered = self.git(self.repo, "worktree", "list", "--porcelain", "-z").stdout
        for entry in registered.split(b"\0"):
            if entry.startswith(b"worktree ") and plain(Path(entry[9:].decode("utf-8", "strict"))) == root:
                raise ValueError("Removed worktree remains registered")
        if (branch_before is not None and self.text(self.repo, "rev-parse", record["branch"]) != branch_before
                or self.text(self.repo, "rev-parse", record["baseHead"] + "^{commit}") != record["baseHead"]):
            raise ValueError("Original branch or base Git object changed during native worktree removal")
        volume_after = helper.free_bytes(root.parent)
        result = {"status": "released", "worktreeId": reference, "path": str(root),
                  "request": request_ref, "nativeTerminal": native_ref, "nativeProofRef": proof_id,
                  "deletedFileCount": inventory["fileCount"], "logicalDeletedBytes": inventory["logicalBytes"],
                  "volumeFreeBytesBefore": volume_before, "volumeFreeBytesAfter": volume_after,
                  "actualVolumeFreeBytesDelta": volume_after - volume_before,
                  "actualVolumeReleasedBytes": max(0, volume_after - volume_before),
                  "exclusiveVolumeClaimed": False, "measurementScope": "volume-free-space-observation",
                  "branchPreserved": record.get("branch"),
                  "baseHeadPreserved": record.get("baseHead"), "candidatePreserved": candidate_id,
                  "retainedPatches": retained_patches, "validationDelivery": delivery,
                  "commitPerformed": False, "pushPerformed": False, "formalAcceptance": False}
        result_ref = write_json(report / "receipt.json", result)
        current = self.state.get("management_worktree", reference)
        if current["payload"].get("releaseOperationKey") != self.key or current["payload"].get("status") != "releasing":
            raise ValueError("Worktree release ownership changed after native removal")
        self.state.put("management_worktree", reference, {**current["payload"], "status": "released",
                       "releaseReceipt": result_ref, "logicalDeletedBytes": inventory["logicalBytes"],
                       "actualVolumeFreeBytesDelta": volume_after - volume_before},
                       expected_version=current["version"])
        result["receipt"] = result_ref
        manifest_ref = self.artifact_manifest(root, retained, native=native, finished=True)
        manifest_path = plain(manifest_ref["path"])
        manifest = json.loads(manifest_path.read_bytes())
        manifest["cleanup"] = {"status": "released-input-worktree", "receipt": result_ref,
                               "deleted_files": inventory["fileCount"], "deleted_bytes": inventory["logicalBytes"],
                               "actualVolumeFreeBytesDelta": volume_after - volume_before,
                               "actualVolumeReleasedBytes": result["actualVolumeReleasedBytes"],
                               "exclusiveVolumeClaimed": False}
        manifest["released"] = [{"path": str(root), "kind": "managed-input-worktree",
                                "nativeProofRef": proof_id, "request": request_ref}]
        manifest["retained"] += [{"path": value["path"], "kind": "original-patch",
                                 "sha256": value["sha256"], "reason": "Original completed patch evidence retained",
                                 "depends_on": [merge_key, validation_key], "review_after": None}
                                for value in retained_patches]
        manifest_path.write_bytes(encoded(manifest) + b"\n")
        result["artifactManifest"] = {"path": str(manifest_path), "sha256": sha(manifest_path.read_bytes())}
        return result

    def commit_worktree(self):
        record, root = self.wt(self.payload["worktreeId"])
        expected = self.payload["expectedHead"]
        validation_key = self.payload["validationKey"]
        if not isinstance(validation_key, str) or not re.fullmatch(r"[0-9a-f]{64}", validation_key):
            raise ValueError("An exact management validation operation key is required")
        validation = self.state.get("management_operation", validation_key)
        captured = self.state.get("management_source", validation_key)
        value = validation["payload"] if validation else {}
        result = value.get("result", {})
        original = value.get("payload", {})
        if (value.get("phase") != "complete" or value.get("action") != "validate" or result.get("status") != "passed"
                or value.get("sessionId") != self.session or original.get("repositoryId") != self.repo_id
                or original.get("worktreeId") != self.payload["worktreeId"]
                or original.get("sourceView", "worktree") != "worktree"
                or original.get("driverDigest") != self.config["driverDigest"]
                or original.get("runtimeOperationId") != self.config["runtimeOperationId"]):
            raise ValueError("Commit lacks matching actual source/native validation")
        # Compare exactly the validation snapshot, including its original cache
        # and external dependency identities; a bare source() snapshot differs.
        current_root, before = self.validation_with_cache(payload=original)
        engine = before["source"]
        if (current_root != root or engine["head"] != expected or not captured or captured["payload"] != before
                or result.get("sourceDigest") != before["sourceDigest"] or not validation_inputs_verified(result)):
            raise ValueError("Validated worktree or dependency inputs changed before commit")
        from tools.jenkins.resources.paths import canonical_build_root, physical_path_under
        approved = canonical_build_root(self.build_root(payload=original))
        report = physical_path_under(approved, approved.namespace() / "management" / "reports" / validation_key)
        facts = {}
        for name, filename in (("nativeTerminal", "native-terminal.json"), ("request", "request.json")):
            reference = result[name]
            if plain(reference["path"]) != plain(report / filename):
                raise ValueError("Validation evidence path differs")
            raw = plain(reference["path"]).read_bytes()
            if sha(raw) != reference["sha256"]:
                raise ValueError("Validation evidence digest changed")
            facts[name] = json.loads(raw)
        native, request = facts["nativeTerminal"], facts["request"]
        execution_root = physical_path_under(approved, request.get("executionRoot", str(root)), allow_missing=False)
        if (request["source"] != before or request["payload"] != original
                or native["command"] != request["command"] or plain(native["cwd"]) != execution_root
                or result.get("executionRoot", str(root)) != str(execution_root)
                or result.get("preparedInputsUnchanged") is not True
                or request.get("preparation") != result.get("preparation")
                or not validation_inputs_verified(result, request)
                or native["rootExitCode"] != 0
                or native_terminal_proof(native, result["nativeTerminal"])["complete"] is not True):
            raise ValueError("Original native validation is not complete and successful")
        changes = set(p.decode("utf-8", "strict") for p in
                      self.git(root, "diff", "--name-only", "-z", "HEAD", "--").stdout.split(b"\0") if p)
        changes.update(p.decode("utf-8", "strict") for p in
                       self.git(root, "ls-files", "--others", "--exclude-standard", "-z").stdout.split(b"\0") if p)
        if not changes or len(changes) > 128 or any(
                not any(path.casefold() == scope.casefold() or path.casefold().startswith(scope.casefold() + "/")
                        for scope in self.owned) for path in changes):
            raise ValueError("Commit must contain only bounded task-owned changes")
        self.scoped(sorted(changes), "commit")
        message = self.payload.get("message")
        if not isinstance(message, str) or not message.strip() or len(message.encode("utf-8")) > 4096 or "\0" in message:
            raise ValueError("A bounded commit message is required")
        message_path = plain(self.repo / ".jenkins/state/controls" / (self.key + ".commit-message.txt"))
        with message_path.open("xb") as stream:
            stream.write(message.encode("utf-8") + b"\n")
        main_head = self.text(self.repo, "rev-parse", "HEAD")
        self.git(root, "add", "--", *sorted(changes))
        staged_source = self.source(root)
        if staged_source["files"] != engine["files"] or staged_source["head"] != expected:
            raise ValueError("Validated worktree file bytes or HEAD changed before commit")
        _same_root, current = self.validation_with_cache(payload=original)
        if _same_root != root or current != before:
            raise ValueError("Validated source or dependencies changed during commit preparation")
        self.git(root, "-c", "commit.gpgSign=false", "commit", "--no-verify", "--no-gpg-sign", "--file", str(message_path))
        head = self.text(root, "rev-parse", "HEAD")
        if self.text(self.repo, "rev-parse", "HEAD") != main_head:
            raise ValueError("Main checkout HEAD changed during worktree commit")
        row = self.state.get("management_worktree", self.payload["worktreeId"])
        self.state.put("management_worktree", self.payload["worktreeId"],
                       {**record, "status": "ready", "head": head, "validationKey": validation_key},
                       expected_version=row["version"])
        return {"status": "complete", "worktreeId": self.payload["worktreeId"], "head": head,
                "tree": self.text(root, "rev-parse", "HEAD^{tree}"), "validationKey": validation_key,
                "validatedSourceDigest": before["sourceDigest"], "nativeTerminal": result["nativeTerminal"],
                "mainHeadBefore": main_head, "mainHeadAfter": main_head, "pushPerformed": False}

    def generate(self):
        _record, root = self.wt(self.payload["worktreeId"])
        before = self.source(root)
        destination = self.outputs() / "generated" / self.key
        self.artifact_manifest(root, {"generated": destination})
        destination.mkdir(parents=True, exist_ok=False)
        recipe = self.payload.get("recipe")
        if recipe == "source-index":
            path = destination / "source-index.json"
            fact = write_json(path, before)
        elif recipe == "render-template":
            relative = self.payload["template"]
            source = plain(root / relative)
            if not source.is_relative_to(root) or source.suffix != ".in":
                raise ValueError("Only a worktree .in template may be rendered")
            self.scoped([relative], "validation")
            variables = self.payload.get("variables", {})
            if (not isinstance(variables, dict) or any(not isinstance(k, str) or not isinstance(v, str)
                                                       for k, v in variables.items())):
                raise ValueError("Template variables must be strings")
            text = source.read_text(encoding="utf-8")
            if len(text.encode("utf-8")) > 1024 * 1024:
                raise ValueError("Template exceeds one MiB")
            data = string.Template(text).substitute(variables).encode("utf-8")
            if len(data) > 4 * 1024 * 1024:
                raise ValueError("Generated file exceeds four MiB")
            name = identifier(self.payload.get("outputName", source.stem))
            if name.split(".")[0].upper() in {"CON", "PRN", "AUX", "NUL", *("COM" + str(i) for i in range(1, 10)), *("LPT" + str(i) for i in range(1, 10))}:
                raise ValueError("Reserved Windows output name")
            path = plain(destination / name)
            with path.open("xb") as stream:
                stream.write(data)
            fact = {"path": str(path), "sha256": sha(data), "bytes": len(data),
                    "templateSha256": sha(source.read_bytes()), "variablesDigest": sha(encoded(variables))}
        else:
            raise ValueError("Supported generation recipes: source-index, render-template")
        if self.source(root) != before:
            raise ValueError("Worktree changed during generation")
        manifest = self.artifact_manifest(root, {"generated": destination}, finished=True)
        return {"status": "complete", "recipe": recipe, "artifact": fact,
                "artifactManifest": manifest, "sourceDigest": before["sourceDigest"]}

    def build_root(self, *, payload=None):
        from tools.jenkins.resources.paths import canonical_build_root, physical_path_under
        payload = self.payload if payload is None else payload
        requested = payload.get("requestedTargetDir")
        explicit = payload.get("buildRoot")
        if requested is not None:
            if not isinstance(requested, str) or not Path(requested).is_absolute():
                raise ValueError("requestedTargetDir must be an absolute physical D/E/F cargo-targets path")
            target = plain(Path(requested))
            requested_root = Path(target.anchor) / "cargo-targets"
            approved = canonical_build_root(str(requested_root))
            physical_path_under(approved, target)
            if explicit is not None and canonical_build_root(explicit).path != approved.path:
                raise ValueError("Requested target and buildRoot choose different physical volumes")
            return str(approved.path)
        configured = getattr(getattr(self, "manager", None), "spec", None)
        storage = getattr(configured, "storage", {})
        return str(canonical_build_root(explicit or storage.get("buildRoot", r"D:\cargo-targets")).path)

    def validation_recipe(self):
        allowed = {"action", "repositoryId", "sessionId", "requestId", "operationId", "driverDigest",
                   "runtimeOperationId", "ownedPaths", "observationOnly", "sourceView", "worktreeId",
                   "externalRoots", "externalGitlinks", "buildRoot", "packages", "features", "profile",
                   "verb", "offline", "noDefaultFeatures", "lib", "tests", "testFilter", "noRun",
                   "allFeatures", "locked", "jobs", "testThreads", "requestedTargetDir",
                   "expectedSourceDigest", "expectedEngineSourceDigest", "externalCapturePolicy",
                   "sourceHashes", "dependencyHashes"}
        if set(self.payload) - allowed:
            raise ValueError("Unsupported Cargo fields: " + ", ".join(sorted(set(self.payload) - allowed)))
        if {"externalCapturePolicy", "expectedEngineSourceDigest"} & set(self.payload):
            engine_digest = self.payload.get("expectedEngineSourceDigest")
            if (self.payload.get("externalCapturePolicy") != "jenkins-admission"
                    or not isinstance(engine_digest, str)
                    or not re.fullmatch(r"[0-9a-f]{64}", engine_digest)):
                raise ValueError("Jenkins dependency admission requires an explicit complete engine source digest")
        packages, features = self.payload.get("packages"), self.payload.get("features", [])
        tests = self.payload.get("tests", [])
        if (not isinstance(packages, list) or not 1 <= len(packages) <= 8
                or any(not isinstance(p, str) or not re.fullmatch(r"[A-Za-z0-9_-]{1,80}", p) for p in packages)
                or not isinstance(features, list) or len(features) > 64
                or any(not isinstance(f, str) or not re.fullmatch(r"[A-Za-z0-9_:/.-]{1,120}", f) for f in features)
                or not isinstance(tests, list) or len(tests) > 16
                or any(not isinstance(t, str) or not re.fullmatch(r"[A-Za-z0-9_.-]{1,80}", t) for t in tests)):
            raise ValueError("Explicit bounded Cargo packages/features/test targets required")
        verb, profile = self.payload.get("verb", "test"), self.payload.get("profile", "dev")
        if verb not in {"check", "build", "test"} or profile not in {"dev", "release"}:
            raise ValueError("Unsupported Cargo recipe")
        options = {name: self.payload.get(name, name == "locked")
                   for name in ("offline", "noDefaultFeatures", "lib", "noRun", "allFeatures", "locked")}
        if any(type(value) is not bool for value in options.values()):
            raise ValueError("Cargo recipe switches must be booleans")
        jobs, threads = self.payload.get("jobs", 1), self.payload.get("testThreads", 1)
        test_filter = self.payload.get("testFilter")
        if (type(jobs) is not int or not 1 <= jobs <= 8 or type(threads) is not int or not 1 <= threads <= 8
                or test_filter is not None and (not isinstance(test_filter, str) or not test_filter
                    or len(test_filter.encode("utf-8")) > 256 or "\0" in test_filter or test_filter.startswith("-"))
                or verb != "test" and (options["noRun"] or test_filter is not None or "testThreads" in self.payload)
                or options["noRun"] and test_filter is not None):
            raise ValueError("Unsupported Cargo jobs/test execution controls")
        command = [verb, "--jobs", str(jobs), "--message-format=json"]
        for name, flag in (("locked", "--locked"), ("offline", "--offline"),
                           ("noDefaultFeatures", "--no-default-features"), ("lib", "--lib"),
                           ("allFeatures", "--all-features"), ("noRun", "--no-run")):
            if options[name]:
                command.append(flag)
        for package in packages:
            command += ["--package", package]
        for test in tests:
            command += ["--test", test]
        if features:
            command += ["--features", ",".join(features)]
        if profile == "release":
            command += ["--release"]
        if verb == "test" and not options["noRun"]:
            if test_filter is not None:
                command.append(test_filter)
            command += ["--", "--test-threads", str(threads)]
        return {"verb": verb, "profile": profile, "packages": packages, "features": features,
                "tests": tests, "testFilter": test_filter, "jobs": jobs, "testThreads": threads,
                **options, "command": command}

    def verify_declared_engine(self, engine, payload):
        expected = payload.get("expectedEngineSourceDigest")
        if expected is not None and (not isinstance(expected, str)
                or not re.fullmatch(r"[0-9a-f]{64}", expected) or engine["sourceDigest"] != expected):
            raise EngineSourceInputMismatch(expected, engine["sourceDigest"], source_component(engine))

    def verify_declared_inputs(self, before, payload):
        self.verify_declared_engine(before["source"], payload)
        expected = payload.get("expectedSourceDigest")
        if expected is not None and (not isinstance(expected, str) or not re.fullmatch(r"[0-9a-f]{64}", expected)
                                     or before["sourceDigest"] != expected):
            raise SourceInputMismatch(expected, before["sourceDigest"], source_components(before))
        def inventory(source, prefix=""):
            result = {}
            for name, value in source["files"].items():
                if isinstance(value, dict) and isinstance(value.get("source"), dict):
                    result.update(inventory(value["source"], prefix + name.casefold() + "/"))
                else:
                    result[prefix + name.casefold()] = value
            return result
        engine = inventory(before["source"])
        dependency = dict(engine)
        for external in before["externalRoots"]:
            dependency.update(inventory(external["source"], "external-zr_vm/"))
        for field, values in (("sourceHashes", engine), ("dependencyHashes", dependency)):
            requested = payload.get(field)
            if requested is None:
                continue
            if not isinstance(requested, dict) or not 1 <= len(requested) <= 128:
                raise ValueError("Input hash declarations must be explicit bounded maps")
            for path, checksum in requested.items():
                if (not isinstance(path, str) or not path or "\\" in path or ":" in path
                        or Path(path).is_absolute() or any(part in {"", ".", ".."} for part in path.split("/"))
                        or checksum is not None and (not isinstance(checksum, str)
                            or not re.fullmatch(r"[0-9a-f]{64}", checksum))):
                    raise ValueError("Input hash declaration has an invalid path or digest")
                if field == "sourceHashes":
                    scopes = payload["ownedPaths"]
                    if not any(path.casefold() == scope.casefold()
                               or path.casefold().startswith(scope.casefold() + "/") for scope in scopes):
                        raise ValueError("Declared source hash is outside the original author's scope")
                if values.get(path.casefold()) != checksum:
                    raise ValueError("Declared input hash changed: " + path)

    def begin_validation_intent(self, resources, owner, writer, root, report):
        intent = {"kind": "management-validation-prelaunch", "status": "unlaunched",
                  "executionId": owner, "operationKey": self.key, "repositoryId": self.repo_id,
                  "sessionId": self.session, "requestId": self.request, "operationId": self.operation,
                  "driverDigest": self.config["driverDigest"],
                  "runtimeOperationId": self.config["runtimeOperationId"],
                  "preparationKey": writer, "reservationKey": owner + ":" + owner,
                  "sourceRoot": str(root), "report": str(report), "ownedNativeLaunchClaimed": False}
        old = self.state.get("management_prelaunch", owner)
        if old and old["payload"] != intent:
            raise ValueError("Original preparation intent changed; reconcile its existing reservation")
        if not old:
            self.state.put("management_prelaunch", owner, intent, expected_version=0)
        self._validation_admission = (resources, owner, writer, intent["reservationKey"])

    def claim_validation_launch(self, owner, command, root, request_ref):
        with self.state.transaction() as connection:
            intent = self.state.get("management_prelaunch", owner, connection=connection)
            execution = self.state.get("execution", owner, connection=connection)
            if (not intent or intent["payload"].get("status") != "unlaunched" or not execution
                    or execution["payload"].get("kind") != "management_owned_cargo"
                    or execution["payload"].get("reservationKey") != intent["payload"]["reservationKey"]
                    or self.state.get("execution_launch", owner, connection=connection)
                    or self.state.get("execution_host", owner, connection=connection)):
                raise ValueError("Native launch lacks its exact unlaunched preparation intent")
            launch = {**intent["payload"], "status": "claimed", "ownedNativeLaunchClaimed": True,
                      "command": command, "cwd": str(root), "request": request_ref, "claimedAt": time.time()}
            self.state.put("management_prelaunch", owner, launch,
                           expected_version=intent["version"], connection=connection)
            self.state.put("execution_launch", owner, launch, expected_version=0, connection=connection)
            self.state.put("execution", owner, {**execution["payload"], "status": "running",
                           "launchClaimed": True}, expected_version=execution["version"], connection=connection)

    def validation_failure_release(self, error):
        admission = getattr(self, "_validation_admission", None)
        if not admission:
            return None
        resources, owner, writer, reservation_key = admission
        proof_id = owner + "-native"
        proof = self.state.get("native_job", proof_id)
        if (proof and proof["payload"].get("owner") == owner
                and proof["payload"].get("status") == "terminal"
                and proof["payload"].get("completeProof", {}).get("complete") is True):
            released = resources.release(reservation_key, writer_key=writer, native_proof_ref=proof_id)
            evidence = {"kind": "actual-native-terminal", "nativeProofRef": proof_id,
                        "reservationKey": reservation_key, "released": released}
        else:
            with self.state.transaction() as connection:
                intent = self.state.get("management_prelaunch", owner, connection=connection)
                native = [row for row in self.state.list("native_job", connection=connection)
                          if row["payload"].get("executionId") == owner or row["payload"].get("owner") == owner]
                if (not intent or intent["payload"].get("status") != "unlaunched"
                        or intent["payload"].get("ownedNativeLaunchClaimed") is not False
                        or self.state.get("execution_launch", owner, connection=connection)
                        or self.state.get("execution_host", owner, connection=connection) or native
                        or plain(Path(intent["payload"]["report"]) / "started.json").exists()):
                    return None
                execution = self.state.get("execution", owner, connection=connection)
                value = execution["payload"] if execution else {
                    **intent["payload"], "kind": "management_owned_cargo"}
                if (value.get("reservationKey") != reservation_key
                        or value.get("preparationKey") != writer
                        or value.get("sessionId") != self.session):
                    return None
                self.state.put("execution", owner, {**value, "status": "failed", "failedBeforeLaunch": True,
                               "durableUnlaunched": True, "reasonCode": getattr(error, "code", "preparation_failed")},
                               expected_version=execution["version"] if execution else 0, connection=connection)
                self.state.put("management_prelaunch", owner, {**intent["payload"], "status": "failed",
                               "failedBeforeLaunch": True, "durableUnlaunched": True},
                               expected_version=intent["version"], connection=connection)
            released = resources.release_unlaunched(reservation_key, writer_key=writer, execution_id=owner)
            evidence = {"kind": "durable-unlaunched", "executionId": owner,
                        "reservationKey": reservation_key, "released": released}
        execution = self.state.get("execution", owner)
        if evidence["kind"] == "actual-native-terminal" and execution:
            self.state.put("execution", owner, {**execution["payload"], "status": "failed",
                           "nativeProofRef": proof_id, "failureRelease": evidence},
                           expected_version=execution["version"])
        if evidence["kind"] == "durable-unlaunched":
            queued = self.state.get("resource_queue", reservation_key)
            if queued and queued["payload"].get("status") in {"waiting", "admitted"}:
                self.state.put("resource_queue", reservation_key, {**queued["payload"], "status": "cancelled",
                               "releaseEvidence": "durable_unlaunched"}, expected_version=queued["version"])
        self.state.put("management_failure_release", self.key, evidence, expected_version=0)
        return evidence

    def validate(self):
        try:
            return self.validate_prepared()
        except Exception as error:
            evidence = self.validation_failure_release(error)
            if evidence is None:
                raise
            from tools.jenkins.contracts import JenkinsError
            return {"status": "failed", "reasonCode": getattr(error, "code", "validation_preparation_failed"),
                    "message": str(error), "reservationRelease": evidence, "formalAcceptance": False,
                    **({"details": error.details} if isinstance(error, JenkinsError) else {})}

    def outputs(self):
        from tools.jenkins.resources.paths import canonical_build_root, physical_path_under
        approved = canonical_build_root(self.build_root())
        return physical_path_under(approved, approved.namespace() / "management")

    def artifact_manifest(self, root, paths, *, native=None, finished=False, task_scratch=None, task_cache=None):
        path = self.outputs() / "reports" / self.key / "artifacts.json"
        value = {"schema_version": 1, "task_id": "jenkins-management-" + self.key,
                 "owner": self.session, "status": "needs-review" if native is not None or finished else "active",
                 "source_root": str(root), "request_id": self.request, "operation_id": self.operation,
                 "output_roots": {name: str(path) for name, path in paths.items()},
                 "processes": [] if native is None else [{"pid": native["pid"], "creationTime": native["creationTime"],
                     "command": native["command"], "root_exit_code": native["rootExitCode"],
                     "native_terminal": native["processTree"]["terminal"]}],
                 "retained": [{"path": str(path), "kind": name, "reason": ("Shared management Cargo cache; outside task-exclusive cleanup"
                            if name == "cache" else "Managed output/evidence retained for reuse or review"),
                     "depends_on": [], "review_after": None} for name, path in paths.items()],
                 "unknown": ([{"path": str(paths.get("reports", root)), "reason": "Native termination remains unproven"}]
                             if native is not None and not native["processTree"]["terminal"] else []),
                 "cleanup": {"receipt": None, "deleted_files": 0, "deleted_bytes": 0}}
        if task_scratch is not None:
            value["taskScratch"] = task_scratch
        if task_cache is not None:
            value["cleanup"] = {"receipt": task_cache.get("receipt"), "status": task_cache["status"],
                                "deleted_files": task_cache["removedFileCount"],
                                "deleted_bytes": task_cache["reclaimedBytes"]}
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(encoded(value) + b"\n")
        return {"path": str(path), "sha256": sha(path.read_bytes())}

    def task_tmp(self, *, environment=None, native=None, native_ref=None, proof_id=None):
        from tools.jenkins.resources.paths import canonical_build_root, physical_path_under
        from tools.jenkins.pilot.native.windows_storage import WindowsStorage
        from tools.jenkins.workflow.post_task_cache import _directory_fact, _bounded_files, _file_fact, _references
        approved = canonical_build_root(self.build_root())
        path = physical_path_under(approved, approved.namespace() / "management" / "reports" / self.key / "tmp")
        binding = {"operationKey": self.key, "repositoryId": self.repo_id, "sessionId": self.session,
                   "requestId": self.request, "operationId": self.operation, "path": str(path),
                   "buildRoot": str(approved.path), "volumeIdentity": approved.volume_identity,
                   "driverDigest": self.config["driverDigest"], "runtimeOperationId": self.config["runtimeOperationId"]}
        if environment is not None:
            if any(environment.get(k) != str(path) for k in ("TEMP", "TMP", "TMPDIR")):
                raise ValueError("Native TEMP must equal this operation's exclusive directory")
            value = {**binding, "environmentDigest": sha(encoded(environment)), "status": "creating"}
            self.state.put("management_task_scratch", self.key, value, expected_version=0)
            granted = approved.path / "zircon-local"
            with WindowsStorage(granted, expected_device=approved.volume_identity) as storage:
                identity = storage.create_directory(str(path.relative_to(granted)))
            value.update(directoryIdentity=_directory_fact(identity), status="owned")
            value["generation"] = sha(encoded(value))
            self.state.put("management_task_scratch", self.key, value, expected_version=1)
            return write_json(path.parent / "tmp-generation.json", value)
        previous = self.state.get("management_task_cache", self.key)
        if previous:
            return previous["payload"]
        result = {**binding, "status": "protected", "removedFileCount": 0, "reclaimedBytes": 0,
                  "maxFiles": 64, "maxBytes": 256 * 1024**2, "maxSeconds": 10, "nativeProofRef": proof_id,
                  "nativeEvidence": native_ref, "warmAndHistoricalPreserved": True, "removed": []}
        try:
            record = self.state.get("management_task_scratch", self.key)["payload"]
            if any(record.get(k) != v for k, v in binding.items()) or record["status"] != "owned":
                raise ValueError("Exclusive TEMP generation is unproven")
            if record["generation"] != sha(encoded({k: v for k, v in record.items() if k != "generation"})):
                raise ValueError("TEMP generation changed")
            result.update(generation=record["generation"], directoryIdentity=record["directoryIdentity"])
            scratch_bytes = plain(path.parent / "tmp-generation.json").read_bytes()
            scratch_ref = {"path": str(path.parent / "tmp-generation.json"), "sha256": sha(scratch_bytes)}
            if json.loads(scratch_bytes) != record:
                raise ValueError("Prelaunch TEMP manifest changed")
            if plain(native_ref["path"]) != path.parent / "native-terminal.json":
                raise ValueError("Native evidence is outside the original task report")
            evidence = plain(native_ref["path"]).read_bytes()
            proof = self.state.get("native_job", proof_id)["payload"]
            if sha(evidence) != native_ref["sha256"] or json.loads(evidence) != native:
                raise ValueError("Original native evidence bytes changed")
            expected_proof = native_terminal_proof(json.loads(evidence), native_ref)
            tree = native["processTree"]
            if (proof_id != "management-" + self.key + "-native" or proof["nativeEvidence"] != native
                    or proof.get("owner") != "management-" + self.key or proof.get("role") != "management_owned_cargo"
                    or proof.get("requestId") != self.request or proof.get("operationId") != self.operation
                    or proof.get("pid") != native["pid"] or proof.get("creationTime") != native["creationTime"]
                    or proof.get("birthToken") != native["creationTime"]
                    or proof.get("executable") != expected_proof["identity"]["executable"]
                    or proof.get("commandDigest") != expected_proof["identity"]["commandDigest"]
                    or proof.get("taskScratch") != scratch_ref or proof.get("environmentDigest") != record["environmentDigest"]
                    or proof.get("status") != "terminal" or expected_proof["complete"] is not True
                    or proof.get("completeProof") != expected_proof or proof.get("terminalProof") != expected_proof
                    or not all(tree.get(k) is True for k in ("terminal", "pipeEOF", "nativeWaiterJoined"))
                    or type(native["rootExitCode"]) is not int or native["terminalErrors"]
                    or native["nativeJobEvidence"]["records"]["afterClose"]["activeProcesses"] != 0):
                raise ValueError("Original native terminal/EOF/waiter proof is incomplete")
            self.state.check_authorization(self.repo_id, self.session, "gc", self.owned)
            deadline, files, total = time.monotonic() + 10, [], 0
            with WindowsStorage(path, expected_device=approved.volume_identity) as storage:
                if _directory_fact(storage.root_identity) != record["directoryIdentity"]:
                    raise ValueError("TEMP physical generation changed")
                for relative in _bounded_files(storage, "", result, deadline):
                    if len(files) >= 64:
                        raise ValueError("TEMP exceeds the file bound")
                    fact, _identity = _file_fact(storage, relative, 256 * 1024**2 - total, deadline)
                    files.append(fact)
                    total += fact["byteLength"]
                result["files"] = files
                self.state.put("management_task_cache", self.key, {**result, "status": "inventory"}, expected_version=0)
                for fact in files:
                    current, identity = _file_fact(storage, fact["path"], fact["byteLength"], deadline)
                    with self.state.transaction():
                        protected, references = _references(self.state, str(path))
                        if references or fact["sha256"] in protected:
                            result["status"] = "protected"
                            continue
                        if current != fact or time.monotonic() > deadline:
                            raise ValueError("TEMP file changed or cleanup deadline expired")
                        storage.delete(fact["path"], expected_identity=identity)
                        if storage.exists(fact["path"]):
                            raise ValueError("TEMP deletion acknowledgement is missing")
                        result["removed"].append({**fact, "physicalNameAbsent": True})
                        result["removedFileCount"] += 1
                        result["reclaimedBytes"] += fact["byteLength"]
                if result["removedFileCount"] == len(files):
                    result["status"] = "complete"
            result["generation"] = record["generation"]
        except Exception as error:
            result.update(status="partial" if result["removedFileCount"] else "protected", reason=str(error))
        receipt = write_json(path.parent / "task-cache.json", result)
        result["receipt"] = receipt
        self.state.put("management_task_cache", self.key, result)
        return result

    def dependency_cache(self, *, payload=None):
        payload = self.payload if payload is None else payload
        from tools.jenkins.resources.paths import canonical_build_root, physical_path_under
        approved = canonical_build_root(self.build_root(payload=payload))
        cache = physical_path_under(approved, approved.path / "zircon-local" / "cache" / "cargo-home")
        facts = {}
        for name in ("config", "config.toml"):
            path = plain(cache / name)
            facts[name] = sha(path.read_bytes()) if path.is_file() else None
        return {"home": str(cache), "configSha256": facts}

    def validation_with_cache(self, *, payload=None):
        root, source = self.validation_source(payload=payload)
        value = {**{k: v for k, v in source.items() if k != "sourceDigest"},
                 "dependencyCache": self.dependency_cache(payload=payload)}
        before = {**value, "sourceDigest": sha(encoded(value))}
        self.verify_declared_inputs(before, self.payload if payload is None else payload)
        return root, before

    def validation_observation(self, before, *, payload=None):
        """Observe live inputs without making the foreign checkout an execution dependency."""
        errors = []
        root, source = self.validation_source(payload=payload, observation_errors=errors)
        cache = self.dependency_cache(payload=payload)
        value = {**{key: item for key, item in source.items() if key != "sourceDigest"},
                 "dependencyCache": cache}
        after = {**value, "sourceDigest": sha(encoded(value))} if not errors else None
        observation = {
            "complete": not errors,
            "sourceDigest": after["sourceDigest"] if after is not None else None,
            "inputsUnchanged": before == after,
            "originalEngineUnchanged": (str(root) == before["sourceRoot"]
                and source["sourceView"] == before["sourceView"] and source["source"] == before["source"]),
            "dependencyCacheUnchanged": cache == before["dependencyCache"],
            "externalInputsUnchanged": before["externalRoots"] == source["externalRoots"] if not errors else None,
            "inputChanges": input_changes(before, after) if after is not None else None,
            "sourceComponents": source_components(after) if after is not None else None,
            "externalInputErrors": errors,
        }
        return root, after, observation

    def validate_prepared(self):
        from tools.jenkins.resources.capacity import Capacity, ResourceManager
        from tools.jenkins.resources.paths import canonical_build_root, physical_path_under
        from tools.jenkins.workflow.planning import _environment_policy
        recipe = self.validation_recipe()
        try:
            root, before = self.validation_with_cache()
        except SourceInputMismatch as error:
            if (not self.state.get("management_source", self.key)
                    and not self.state.get("management_prelaunch", "management-" + self.key)):
                return {"status": "conflict", "reasonCode": "preparation-conflict",
                        "message": str(error), "preparationStarted": False, "compileStarted": False,
                        "formalAcceptance": False, **error.diagnostics}
            raise
        old = self.state.get("management_source", self.key)
        if old and old["payload"] != before:
            raise ValueError("Original queued validation source changed")
        if not old:
            self.state.put("management_source", self.key, before, expected_version=0)
        packages, features = recipe["packages"], recipe["features"]
        verb, profile = recipe["verb"], recipe["profile"]
        offline, no_defaults = recipe["offline"], recipe["noDefaultFeatures"]
        policy = self.state.get("resource_policy", "default")["payload"]
        resources = ResourceManager(self.state, Capacity(policy["cpuBudget"], policy["memoryBudget"], policy["diskBudget"]))
        parallelism = max(recipe["jobs"], recipe["testThreads"] if verb == "test" and not recipe["noRun"] else 1)
        amount = Capacity(parallelism, 2 * parallelism * 1024**3, 4 * 1024**3)
        environment, trusted, _policy_file = _environment_policy(self.state, self.repo)
        preparation = source_helper()
        compatibility_environment = {**environment,
                                     **({"cargoAllFeatures": True} if recipe["allFeatures"] else {}),
                                     "sourceCargoConfiguration": {
            path: checksum for path, checksum in before["source"]["files"].items()
            if path.casefold() in {".cargo/config", ".cargo/config.toml"}}}
        pool = preparation.preparation_key(self.repo_id, trusted, compatibility_environment, packages, features, profile,
                                           no_defaults, before["dependencyCache"],
                                           ["zr_vm"] if before["externalRoots"] else [])
        owner, writer = "management-" + self.key, "management-pool-" + pool
        build_root = self.build_root()
        approved = canonical_build_root(build_root)
        output = self.outputs()
        target = plain(output / "pools" / pool / "target")
        execution_root = physical_path_under(approved, target.parent / "inputs" / "source")
        report = plain(output / "reports" / self.key)
        if report.exists():
            raise ValueError("Validation report exists; reconcile the original operation")
        helper_path = plain(self.repo / "tools/dev/local_cargo.py")
        spec = importlib.util.spec_from_file_location("jenkins_managed_local_cargo", helper_path)
        helper = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(helper)
        if helper.free_bytes(target) < 35 * 1024**3:
            raise ValueError("Target volume must retain 35 GiB free")
        command = list(recipe["command"])
        command = helper.prepare_command(command, target, locked=recipe["locked"])
        command[0] = trusted["cargoPath"]
        for tool in ("cargo", "rustc"):
            if sha(plain(trusted[tool + "Path"]).read_bytes()) != trusted[tool + "Sha256"]:
                raise ValueError("Trusted compiler executable changed")
        cache = plain(before["dependencyCache"]["home"])
        scratch = plain(report / "tmp")
        environment.setdefault("SYSTEMROOT", os.environ.get("SYSTEMROOT", r"C:\Windows"))
        environment.update({"CARGO_TARGET_DIR": str(target), "CARGO_BUILD_BUILD_DIR": str(target.parent / "build"),
                            "CARGO_HOME": str(cache), "TEMP": str(scratch), "TMP": str(scratch),
                            "TMPDIR": str(scratch), "RUSTC": trusted["rustcPath"],
                            "RUSTC_WRAPPER": "", "RUSTC_WORKSPACE_WRAPPER": "", "PYTHONDONTWRITEBYTECODE": "1",
                            "CARGO_NET_OFFLINE": "true" if offline else "false",
                            "CARGO_ENCODED_RUSTFLAGS": "", "RUSTFLAGS": "", "RUSTDOCFLAGS": ""})
        self.begin_validation_intent(resources, owner, writer, root, report)
        resources.queue(owner, request_id=owner, amount=amount, build_root=build_root,
                        heavy_writer=True, writer_key=writer)
        _inventory, generation = resources.scan_inventory(build_root)
        from tools.jenkins.contracts import JenkinsError
        try:
            admitted = resources.admit(owner, amount, inventory_generation=generation, writer_key=writer,
                                      request_id=owner, build_root=build_root, heavy_writer=True)
        except JenkinsError as error:
            if error.code in {"pool_writer_active", "pool_cache_maintenance_active"} or (error.code == "storage_snapshot_stale" and error.retryable):
                return {"status": "waiting", "sourceDigest": before["sourceDigest"],
                        "reasonCode": error.code, "formalAcceptance": False}
            raise
        if admitted is None:
            return {"status": "waiting", "sourceDigest": before["sourceDigest"],
                    "reasonCode": "resource_capacity", "formalAcceptance": False}
        outputs = {"build": target.parent, "cache": cache, "reports": report, "tmp": scratch,
                   "source": execution_root, "sourceObjects": approved.namespace() / "inputs"}
        self.artifact_manifest(root, outputs)
        for path in (target, target.parent / "build", cache):
            path.mkdir(parents=True, exist_ok=True)
        task_scratch = self.task_tmp(environment=environment)
        self.artifact_manifest(root, outputs, task_scratch=task_scratch)
        coverage = {name: value for name, value in recipe.items() if name != "command"}
        sealed = preparation.seal_source(self.state, source_root=root, snapshot=before,
                    repository_root=self.repo, repository_id=self.repo_id, session_id=self.session,
                    owner=owner, approved=approved, coverage={name: value for name, value in coverage.items()
                                                           if name != "verb"})
        # No source copies or candidate workspaces are created by the agent.
        # The admitted Jenkins writer owns this stable formal preparation pool.
        prepared = preparation.materialize_source(self.state, approved=approved, owner=owner,
                    preparation_key_value=writer, source_path=execution_root, reservation_key=admitted["key"],
                    sealed=sealed, identity={"repositoryId": self.repo_id, "sessionId": self.session,
                        "requestId": self.request, "operationId": self.operation,
                        "driverDigest": self.config["driverDigest"], "runtimeOperationId": self.config["runtimeOperationId"]},
                    command=command, environment_digest=sha(encoded(environment)))
        prepared_before = preparation.verify_source(sealed["sourceManifest"], execution_root, approved)
        captured_input_verified = (sealed.get("capturedInputVerified") is True
            and sealed["capturedSourceDigest"] == before["sourceDigest"]
            and prepared["capturedSourceDigest"] == before["sourceDigest"]
            and prepared["sealedInputRef"] == sealed["sealedInputRef"] == prepared_before["sourceDigest"])
        if not captured_input_verified:
            raise ValueError("Prepared execution input is not the exact complete declared sealed capture")
        unchanged_root, _unchanged, preparation_observation = self.validation_observation(before)
        if (unchanged_root != root or preparation_observation["originalEngineUnchanged"] is not True
                or preparation_observation["dependencyCacheUnchanged"] is not True):
            raise ValueError("Original engine or dependency-cache configuration changed during source preparation")
        execution_identity = sha(encoded({"sourceDigest": sealed["sealedInputRef"], "command": command,
                    "cwd": str(execution_root), "environmentDigest": sha(encoded(environment)),
                    "coverage": coverage, "driverDigest": self.config["driverDigest"]}))
        request_ref = write_json(report / "request.json", {"payload": self.payload, "source": before,
                                 "sourceRoot": str(root), "executionRoot": str(execution_root),
                                 "command": command, "target": str(target), "resource": amount.__dict__,
                                 "preparation": prepared, "sourceSealing": sealed["metrics"],
                                 "preparedSource": prepared_before,
                                 "validationInputPolicy": "sealed-source",
                                 "capturedInputVerified": captured_input_verified,
                                 "inputObservation": preparation_observation,
                                 "executionIdentity": execution_identity,
                                 "taskScratch": task_scratch, "environmentDigest": sha(encoded(environment)),
                                 "temporaryEnvironment": {k: environment[k] for k in ("TEMP", "TMP", "TMPDIR")}})
        # This existing function retains a Windows Job, waits descendants and pipe EOF.
        self.claim_validation_launch(owner, command, execution_root, request_ref)
        native = helper._run_owned_command(command, execution_root, environment, report, 21600)
        native_ref = write_json(report / "native-terminal.json", native)
        if native["command"] != command or native["cwd"] != str(execution_root):
            raise ValueError("Native evidence differs from the admitted command or prepared source")
        terminal_proof = native_terminal_proof(native, native_ref)
        terminal = terminal_proof["complete"] is True
        if terminal:
            proof_id = owner + "-native"
            self.state.put("native_job", proof_id, {"owner": owner, "executionId": owner, "role": "management_owned_cargo",
                           "status": "terminal", "terminalProof": terminal_proof, "completeProof": terminal_proof,
                           "nativeEvidence": native, "pid": native["pid"], "creationTime": native["creationTime"],
                           "birthToken": native["creationTime"], "executable": terminal_proof["identity"]["executable"],
                           "commandDigest": terminal_proof["identity"]["commandDigest"],
                           "requestId": self.request, "operationId": self.operation,
                           "taskScratch": task_scratch, "environmentDigest": sha(encoded(environment)),
                           "formalAcceptance": False}, expected_version=0)
        task_cache = self.task_tmp(native=native, native_ref=native_ref,
                                  proof_id=owner + "-native" if terminal else None)
        artifacts, fresh = [], 0
        for line in (report / "stdout.bin").read_text(encoding="utf-8", errors="replace").splitlines():
            try:
                event = json.loads(line)
            except ValueError:
                continue
            if event.get("reason") != "compiler-artifact":
                continue
            fresh += int(event.get("fresh") is True)
            for name in event.get("filenames", []):
                path = plain(name)
                if not (path.is_relative_to(target) or path.is_relative_to(target.parent / "build")):
                    raise ValueError("Compiler artifact escaped the admitted target/build directories")
                if path.is_file():
                    artifacts.append({"path": str(path), "sha256": sha(path.read_bytes())})
        input_error, prepared_after, prepared_error = None, None, None
        try:
            after_root, after, completion_observation = self.validation_observation(before)
            if after_root != root:
                raise ValueError("Validation source root changed")
        except Exception as error:
            after, input_error = None, str(error)
            completion_observation = {"complete": False, "sourceDigest": None, "inputsUnchanged": False,
                "originalEngineUnchanged": False, "dependencyCacheUnchanged": False,
                "externalInputsUnchanged": None, "inputChanges": None, "sourceComponents": None,
                "externalInputErrors": [], "captureError": str(error)}
        try:
            prepared_after = preparation.verify_source(sealed["sourceManifest"], execution_root, approved)
        except Exception as error:
            prepared_error = str(error)
        prepared_unchanged = prepared_before == prepared_after
        input_proof = {"validationInputPolicy": "sealed-source", "capturedInputVerified": captured_input_verified,
            "originalEngineUnchanged": (preparation_observation["originalEngineUnchanged"] is True
                                       and completion_observation["originalEngineUnchanged"] is True),
            "dependencyCacheUnchanged": (preparation_observation["dependencyCacheUnchanged"] is True
                                        and completion_observation["dependencyCacheUnchanged"] is True),
            "inputsUnchanged": before == after, "preparedInputsUnchanged": prepared_unchanged}
        if terminal:
            resources.release(admitted["key"], writer_key=writer, native_proof_ref=proof_id)
        status = ("passed" if terminal and native["rootExitCode"] == 0 and validation_inputs_verified(input_proof)
                  else "failed" if terminal else "unknown")
        execution = self.state.get("execution", owner)
        self.state.put("execution", owner, {**execution["payload"], "status": status,
                       "nativeProofRef": owner + "-native" if terminal else None,
                       "nativeTerminal": native_ref, **input_proof, "executionIdentity": execution_identity},
                       expected_version=execution["version"])
        launch = self.state.get("execution_launch", owner)
        self.state.put("execution_launch", owner, {**launch["payload"], "status": "terminal" if terminal else "unknown",
                       "nativeProofRef": owner + "-native" if terminal else None},
                       expected_version=launch["version"])
        result = {"status": status, "request": request_ref, "nativeTerminal": native_ref, **input_proof,
                  "sourceDigest": before["sourceDigest"], "inputError": input_error,
                  "inputObservations": {"afterPreparation": preparation_observation,
                                        "afterExecution": completion_observation},
                  "inputChanges": input_changes(before, after) if after is not None else None,
                  "sourceRoot": str(root), "executionRoot": str(execution_root), "preparation": prepared,
                  "sourceSealing": sealed["metrics"],
                  "preparedInputsUnchanged": prepared_unchanged, "preparedInputError": prepared_error,
                  "executionIdentity": execution_identity,
                  "target": str(target), "requestedTargetDir": self.payload.get("requestedTargetDir"),
                  "actualTargetDir": str(target), "targetPolicy": "managed-compatibility-pool", "recipe": coverage,
                  "artifactFacts": artifacts, "cargoFreshArtifactCount": fresh,
                  "taskScratch": task_scratch, "postTaskCache": task_cache,
                  "formalAcceptance": False, "m8Accepted": False}
        result["artifactManifest"] = self.artifact_manifest(root, outputs, native=native,
                                                          task_scratch=task_scratch, task_cache=task_cache)
        write_json(report / "receipt.json", result)
        return result


    def dispatch(self):
        if self.action == "create-branch":
            return self.create_branch()
        if self.action == "create-worktree":
            ref, _path, value = self.new_worktree()
            return {**value, "status": "complete", "worktreeId": ref}
        if self.action == "prepare-merge":
            return self.prepare_merge()
        if self.action == "apply-patch":
            return self.apply_patch()
        if self.action == "release-worktree":
            return self.release_worktree()
        if self.action == "commit-worktree":
            return self.commit_worktree()
        if self.action == "generate":
            return self.generate()
        if self.action == "validate":
            return self.validate()
        if self.action == "submit-flow":
            from tools.jenkins.current_source import handle
            identity = self.payload["flowPayload"].get("identity", {})
            if identity.get("sessionId") != self.session or identity.get("repositoryId") != self.repo_id:
                raise ValueError("Flow must retain this original author and repository")
            delivery = handle("submit", self.payload["flowPayload"], self.state, self.repo)
            return {"status": "submitted" if delivery.get("delivery") == "submitted" else "unknown",
                    "submission": delivery, "formalAcceptance": False}
        raise ValueError("Unknown management action")

    def existing_result(self, row):
        value = row["payload"]
        if value["inputDigest"] != self.input_digest:
            raise ValueError("Original operation has different input")
        return value.get("result", {"status": "unknown", "reasonCode": "original_operation_running",
                                   "operation": value, "formalAcceptance": False})

    def run(self):
        from tools.jenkins.state.locks import process_lock
        if self.query:
            row = self.state.get("management_operation", self.key)
            return self.existing_result(row) if row else {
                "status": "failed", "reasonCode": "original_operation_missing", "formalAcceptance": False}
        with process_lock(self.repo / ".jenkins/state/locks" / ("management-operation-" + self.key + ".lock"), timeout=0):
            row = self.state.get("management_operation", self.key)
            if row:
                previous = self.existing_result(row)
                if row["payload"]["phase"] == "complete":
                    return previous
                if row["payload"]["phase"] != "waiting":
                    return {"status": "unknown", "reasonCode": "original_operation_requires_reconciliation",
                            "operation": row["payload"], "formalAcceptance": False}
            record = {"requestId": self.request, "operationId": self.operation,
                      "sessionId": self.session, "action": self.action, "inputDigest": self.input_digest,
                      "phase": "running", "startedAt": time.time(), "payload": self.payload}
            self.state.put("management_operation", self.key, record, expected_version=row["version"] if row else 0)
            try:
                if self.action in GIT_ACTIONS:
                    with process_lock(self.repo / ".jenkins/state/locks/management-git.lock", reason="management_git_busy", timeout=0):
                        result = self.dispatch()
                else:
                    result = self.dispatch()
            except BaseException as error:
                if getattr(error, "code", None) == "management_git_busy":
                    result = {"status": "waiting", "reasonCode": "management_git_busy", "formalAcceptance": False}
                else:
                    result = {"status": "unknown", "reasonCode": "management_operation_interrupted",
                              "message": str(error), "formalAcceptance": False}
                if isinstance(error, SourceInputMismatch):
                    result.update(error.diagnostics)
            result = {**result, "operationKey": self.key, "requestId": self.request,
                      "operationId": self.operation, "sessionId": self.session,
                      "driverDigest": self.config["driverDigest"], "runtimeOperationId": self.config["runtimeOperationId"]}
            current = self.state.get("management_operation", self.key)
            finished = {**record, "phase": "waiting" if result["status"] == "waiting" else
                        "unknown" if result["status"] == "unknown" else "complete", "result": result,
                        "formalAcceptance": False, "finishedAt": time.time()}
            self.state.put("management_operation", self.key, finished, expected_version=current["version"])
            return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--capsule", required=True, type=Path)
    parser.add_argument("--input-file", required=True, type=Path)
    parser.add_argument("--output-file", required=True, type=Path)
    args = parser.parse_args()
    try:
        config, repo, manager = boot(args.capsule)
        permitted = plain(repo / ".jenkins/state/controls")
        source, destination = plain(args.input_file), plain(args.output_file)
        if source.parent != permitted or destination.parent != permitted or source.stat().st_size > 1024 * 1024:
            raise ValueError("Management controls must stay in the fixed State directory")
        payload = json.loads(source.read_bytes())
        result = Management(config, repo, manager, payload).run()
    except Exception as error:
        result = {"status": "failed", "message": str(error), "formalAcceptance": False}
    # Control responses may be refreshed; immutable evidence lives in State/reports.
    if "destination" not in locals() or destination.parent != permitted:
        print(encoded(result).decode("utf-8"))
        return 2
    destination.write_bytes(encoded(result) + b"\n")
    print(encoded(result).decode("utf-8"))
    return 0 if result.get("status") in {"complete", "passed", "waiting", "merge-prepared", "submitted", "released"} else 2


if __name__ == "__main__":
    raise SystemExit(main())
