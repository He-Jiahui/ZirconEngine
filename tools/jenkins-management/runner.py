"""Pinned ordinary Jenkins management job; receipts never confer M8 acceptance."""
from __future__ import annotations

import argparse
from concurrent.futures import ThreadPoolExecutor
from contextlib import nullcontext
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import string
import stat
import subprocess
import sys
import time

def fixed_history_archive_input(payload):
    """Validate the sole typed historical input without writing or copying it."""
    if "fixedHistoryArchiveInput" not in payload:
        return None
    value = payload["fixedHistoryArchiveInput"]
    if (not isinstance(value, dict) or set(value) != {"schemaVersion", "kind", "access", "root"}
            or type(value.get("schemaVersion")) is not int or value["schemaVersion"] != 1
            or value.get("kind") != "zircon-fixed-docs-history" or value.get("access") != "read-only"):
        raise ValueError("Fixed historical archive requires the exact typed read-only input")
    raw = value.get("root")
    if (not isinstance(raw, str) or not raw or len(raw.encode("utf-8")) > 4096
            or any(c in raw for c in "\0\r\n") or not Path(raw).is_absolute()):
        raise ValueError("Fixed historical archive root must be an explicit absolute path")
    archive = Path(raw).resolve(strict=True)
    if not archive.is_dir():
        raise ValueError("Fixed historical archive root must be a readable directory")
    with os.scandir(archive):
        pass
    return {**value, "root": str(archive)}


ID = re.compile(r"[A-Za-z0-9][A-Za-z0-9_.-]{0,79}\Z")
WORKTREE_ID = re.compile(r"[A-Za-z0-9][A-Za-z0-9_.-]{0,159}\Z")
SHA = re.compile(r"(?:[0-9a-f]{40}|[0-9a-f]{64})\Z")
SOURCE_EVIDENCE_MAX_BYTES = 64 * 1024**2
# Windows' legacy MAX_PATH is 260, and Git for Windows 2.31 does not honor
# core.longpaths on every code path.  Measured on this host: `git add` succeeds
# at an absolute length of 258 and fails from 260 upward, while the checkout
# performed by `git worktree add` tolerates longer names.  Because the index
# write is what a later patch or commit needs, the budget is pinned to the
# stricter, measured limit.
MANAGED_PATH_BUDGET = 259
# git reports checkout progress on stderr as carriage-return updates.  Keeping a
# plain tail lets thousands of "Updating files: N%" frames push the actual
# "fatal:" line out of the window, turning a precise diagnosis into an opaque
# interruption.
GIT_PROGRESS = re.compile(r"^(?:Updating files|Checking out files|Receiving objects|"
                          r"Resolving deltas|remote:|Counting objects|Compressing objects)")
PREVIEW_ACTION = "preview-current-editor-r34"
PREVIEW_REPOSITORY = Path("E:/Git/ZirconEngine")
PREVIEW_REPORTS = Path("E:/cargo-targets/zircon-local/reports/penpot-unreal-01a0f04d")
PREVIEW_PROOF = PREVIEW_REPORTS / "ordinary-window-unreal-current/unreal-chrome-cutover-current/jenkins-fixed-editor-preview-current"
PREVIEW_SCRIPT = Path("E:/cargo-targets/zircon-local/tmp/penpot-unreal-01a0f04d/launch-current-normal-editor-preview-r34.py")
PREVIEW_SCRIPT_SHA256 = "ba8d6170222b6d2a7e722b895be8108dbc6aace45d21bc0706a1626f5f320e22"
PREVIEW_PYTHON = Path("E:/cargo-targets/zircon-local/tools/penpot-unreal-01a0f04d/python-3.14.4-embed-amd64/runtime/python.exe")
PREVIEW_PYTHON_SHA256 = "7ca24f26d6e3f463419ee4f537ddd3acd312c38fe45e678cce08572f26a8bd1a"
PREVIEW_OUTPUT = Path("E:/cargo-targets/zircon-local/tests/penu-01a0f04d/ordinary-normal-preview-r34")
PREVIEW_TIMEOUT_SECONDS = 900


ACTIONS = {"create-branch", "create-worktree", "prepare-merge", "apply-patch",
           "commit-worktree", "release-worktree", "recover-worktree", "validate", "generate",
           "prepare-flow", "submit-flow", "prepare-worktree", "inspect-worktree", "retire-worktree-reader", PREVIEW_ACTION}
GIT_ACTIONS = {"create-branch", "create-worktree", "prepare-merge", "apply-patch", "commit-worktree",
               "release-worktree", "recover-worktree", "prepare-worktree"}
AUTH = {PREVIEW_ACTION: "validation", "create-branch": "implementation", "create-worktree": "implementation",
        "prepare-merge": "implementation", "generate": "implementation",
        "apply-patch": "implementation", "commit-worktree": "commit", "release-worktree": "gc",
        "recover-worktree": "gc", "validate": "validation", "prepare-flow": "validation",
        "submit-flow": "validation", "prepare-worktree": "validation",
        "inspect-worktree": "validation", "retire-worktree-reader": "validation"}


def compact_build_environment(environment):
    """Apply before compatibility hashing and launch; never reuse a larger pool
    under a key that claims compact settings. Release/profiling symbols stay intact.
    """
    return {**environment, "CARGO_INCREMENTAL": "0", "CARGO_PROFILE_DEV_INCREMENTAL": "false",
            "CARGO_PROFILE_TEST_INCREMENTAL": "false", "CARGO_PROFILE_DEV_DEBUG": "0",
            "CARGO_PROFILE_TEST_DEBUG": "0"}


def encoded(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"),
                      ensure_ascii=False, allow_nan=False).encode("utf-8")


def sha(value):
    return hashlib.sha256(value).hexdigest()


MANAGED_WORKTREE_CAPABILITY = {
    "enabled": True, "schemaVersion": 1, "actions": sorted(ACTIONS - {PREVIEW_ACTION}),
    "sourcePreparation": "managed-worktree-source-preparation-v1",
    "compilerInputs": "managed-worktree-product-inputs-v1", "formalAcceptance": False,
}

def stderr_tail(raw, *, limit=3000):
    """Keep the diagnosable part of git stderr.

    Checkout writes carriage-return progress frames, so a plain tail can be
    entirely 'Updating files: N%' and hide the fatal line.  Progress frames are
    dropped first; the remaining real lines keep their own bounded window.
    """
    text = raw.decode("utf-8", "replace") if isinstance(raw, (bytes, bytearray)) else str(raw)
    lines = [line for line in re.split(r"[\r\n]", text) if line and not GIT_PROGRESS.match(line)]
    joined = "\n".join(lines).strip()
    if not joined:
        return text[-limit:]
    return joined[-limit:]


def managed_path_budget(root, deepest_file, deepest_dir):
    """Report whether a managed path plus its deepest entries fits MAX_PATH.

    Checkout creates directories as well as files, and the observed failure was
    a directory creation, so both measurements participate in the budget.
    """
    budget = len(str(root)) + 1 + max(deepest_file, deepest_dir)
    return budget <= MANAGED_PATH_BUDGET, budget


def deepest_tracked_paths(root):
    """Longest repository-relative file and directory Git would check out here.

    Raw bytes are used so a non-UTF-8 name cannot abort the measurement; the
    comparison only needs character counts.
    """
    command = ["git", "-c", "core.fsmonitor=false", "-c", "core.hooksPath=" + os.devnull, "-c", "core.longpaths=true", "-C", str(root), "ls-files", "-z"]
    completed = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
    if completed.returncode:
        raise ValueError(stderr_tail(completed.stderr))
    files, directories = 0, 0
    for item in completed.stdout.split(b"\0"):
        if not item:
            continue
        files = max(files, len(item))
        parts = item.split(b"/")
        for index in range(1, len(parts)):
            directories = max(directories, len(b"/".join(parts[:index])))
    return files, directories


def source_component(source):
    """Summarize every captured file and registered nested source."""
    files = source["files"]
    gitlinks = {}
    for path, value in sorted(files.items()):
        if isinstance(value, dict) and isinstance(value.get("source"), dict):
            gitlinks[path] = {"gitlinkCommit": value.get("gitlinkCommit"),
                              "source": source_component(value["source"])}
    value = {"head": source["head"], "sourceDigest": source["sourceDigest"],
             "filesDigest": sha(encoded(files)), "fileCount": len(files), "gitlinks": gitlinks}
    if source.get("sourceEvidencePolicy") == "full-file-index-v2":
        return {**value, "sourceEvidencePolicy": source["sourceEvidencePolicy"],
                "gitIndexDigest": sha(encoded(source["gitIndex"])),
                "gitIndexEntryCount": len(source["gitIndex"]["entries"]),
                "gitIndexMetadataSha256": source["gitIndex"]["metadataSha256"]}
    return {**value, "diffSha256": source["diffSha256"]}


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


class ManagedPathBudget(ValueError):
    """Refuse a worktree checkout that cannot fit the Windows path budget.

    Git's own MAX_PATH handling is unreliable on 2.31 for directory creation,
    so a long checkout half-applies and is then reported as an opaque
    interruption.  Failing before the first write keeps the operation
    diagnosable and leaves no partial worktree behind.
    """

    code = "managed_path_budget_exceeded"

    def __init__(self, message, *, details=None):
        super().__init__(message)
        self.details = details or {}
        self.retryable = False


class WorktreeConflict(ValueError):
    """Report a worktree reference whose existing state cannot be reused.

    A repeated ``create-worktree`` used to fail with a bare "worktree exists",
    which reads the same whether the caller should simply adopt the worktree it
    already owns or whether a genuinely different operation collides.  The
    reason code distinguishes the two so the caller never has to guess.
    """

    code = "managed_worktree_conflict"

    def __init__(self, message, *, reason, details=None):
        super().__init__(message)
        self.reason = reason
        self.details = details or {}
        self.retryable = False


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


class CheckoutInputIncomplete(ValueError):
    code = "checkout_input_incomplete"

    def __init__(self, message, *, input_diagnostic=None):
        super().__init__(message)
        self.input_diagnostic = None
        if input_diagnostic is not None:
            self.input_diagnostic = {
                "schemaVersion": 1,
                "kind": "installed-git-input-overlap",
                "selectedGit": {
                    key: input_diagnostic["selectedGit"][key] for key in ("path", "sha256")},
                "matchedInput": {
                    key: input_diagnostic["matchedInput"][key] for key in ("role", "root")},
            }


class CheckoutInputIdentityConflict(ValueError):
    code = "checkout_input_identity_conflict"

    def __init__(self, path, previous_path, before_sha256, after_sha256):
        self.path = path
        self.previous_path = previous_path
        self.before_sha256 = before_sha256
        self.after_sha256 = after_sha256
        super().__init__("Conflicting checkout input identity: " + path)


def _record_checkout_input_identity(files, folded, path, checksum):
    name, key = str(path), str(path).casefold()
    prior = folded.get(key)
    if prior is not None and files[prior] != checksum:
        raise CheckoutInputIdentityConflict(name, prior, files[prior], checksum)
    files[prior or name] = checksum
    folded[key] = prior or name


def checkout_expected_inputs_digest(inputs):
    """Bind internally captured native-tool facts together with declared bytes."""
    files = inputs["files"]
    trusted = inputs.get("trustedNativeInputs", {})
    if not isinstance(files, dict) or not isinstance(trusted, dict):
        raise ValueError("Actual checkout inputs require exact file and native-tool maps")
    return sha(encoded({"files": files, "trustedNativeInputs": trusted} if trusted else files))


def checkout_trusted_native_verified(inputs, guard):
    trusted = inputs.get("trustedNativeInputs", {})
    return (isinstance(trusted, dict)
            and guard.get("trustedNativeInputs", {}) == trusted
            and (not trusted or guard.get("trustedNativeInputsDigest") == sha(encoded(trusted))))


def validation_inputs_verified(result, request=None):
    """Require the proof belonging to the actual execution input policy."""
    if result.get("validationInputPolicy") == "actual-checkout":
        required = ("capturedInputVerified", "inputsUnchanged", "originalEngineUnchanged",
                    "dependencyCacheUnchanged", "externalInputsUnchanged", "checkoutInputsUnchanged",
                    "sourceNoWrites", "membershipStable")
        guard = result.get("checkoutGuard", {})
        if (any(result.get(name) is not True for name in required)
                or not isinstance(guard, dict)
                or any(guard.get(name) is not True for name in
                       ("complete", "stable", "sourceNoWrites", "membershipStable",
                        "cancellationComplete", "waitersJoined"))
                or guard.get("inputCopiesCreated") != 0
                or not isinstance(result.get("sourceRoot"), str)
                or result["sourceRoot"] != result.get("executionRoot")
                or guard.get("sourceRoot") != result["sourceRoot"]
                or guard.get("expectedInputsDigest") != result.get("expectedInputsDigest")
                or any(not isinstance(result.get(name), str) or not re.fullmatch(r"[0-9a-f]{64}", result[name])
                       for name in ("sourceDigest", "checkoutInputsDigest", "expectedInputsDigest"))):
            return False
        for name in ("checkoutReadRef", "checkoutGuardRef"):
            reference = result.get(name)
            if (not isinstance(reference, dict) or set(reference) != {"path", "sha256"}
                    or not isinstance(reference["path"], str) or not reference["path"]
                    or not Path(reference["path"]).is_absolute()
                    or not isinstance(reference["sha256"], str)
                    or not re.fullmatch(r"[0-9a-f]{64}", reference["sha256"])):
                return False
        if result["checkoutGuardRef"]["sha256"] != sha(encoded(guard) + b"\n"):
            return False
        trusted = guard.get("trustedNativeInputs", {})
        if (not isinstance(trusted, dict)
                or (trusted or "trustedNativeInputsDigest" in guard)
                and guard.get("trustedNativeInputsDigest") != sha(encoded(trusted))):
            return False
        if request is None:
            return True
        return (request.get("validationInputPolicy") == "actual-checkout"
                and request.get("payload", {}).get("sourceView") == "checkout"
                and request.get("capturedInputVerified") is True
                and request.get("source", {}).get("sourceDigest") == result["sourceDigest"]
                and all(request.get(name) == result.get(name) for name in
                        ("sourceRoot", "executionRoot", "checkoutInputsDigest",
                         "expectedInputsDigest", "checkoutReadRef"))
                and isinstance(request.get("checkoutInputs"), dict)
                and sha(encoded(request["checkoutInputs"])) == result["checkoutInputsDigest"]
                and checkout_expected_inputs_digest(request["checkoutInputs"]) == result["expectedInputsDigest"]
                and checkout_trusted_native_verified(request["checkoutInputs"], guard)
                and "preparation" not in request and "sealedInputRef" not in result)
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
    # The ordinary entry already refuses parent traversal; this side must match
    # it, because a ".." segment resolves against the caller's cwd and would
    # otherwise let a control path escape the repository it was scoped to.
    if ".." in Path(path).parts:
        raise ValueError("Physical paths cannot contain parent traversal")
    for part in (path, *path.parents):
        try:
            info = part.lstat()
        except FileNotFoundError:
            continue
        if stat.S_ISLNK(info.st_mode) or getattr(info, "st_file_attributes", 0) & 0x400:
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


def fixed_editor_preview_recipe(payload):
    """Accept identity plus two descriptor inputs; execution controls are fixed."""
    fields = {"action", "repositoryId", "sessionId", "requestId", "operationId",
              "driverDigest", "runtimeOperationId", "ownedPaths",
              "productDescriptor", "productDescriptorSha256"}
    if (not isinstance(payload, dict) or not fields <= set(payload)
            or set(payload) - fields - {"observationOnly"}
            or payload["action"] != PREVIEW_ACTION
            or type(payload.get("observationOnly", False)) is not bool):
        raise ValueError("Only the fixed Editor preview descriptor schema is supported")
    name, digest = payload["productDescriptor"], payload["productDescriptorSha256"]
    if (not isinstance(name, str) or any(c in name for c in "\0\r\n")
            or not Path(name).is_absolute() or ".." in Path(name).parts
            or not Path(name).is_relative_to(PREVIEW_REPORTS)
            or not isinstance(digest, str) or not re.fullmatch(r"[0-9a-f]{64}", digest)):
        raise ValueError("An exact task-owned absolute descriptor path and SHA256 are required")
    descriptor = {"path": str(Path(name)), "sha256": digest}
    return {"descriptor": descriptor,
            "command": [str(PREVIEW_PYTHON), "-B", "-X", "utf8", str(PREVIEW_SCRIPT),
                        "--product-descriptor", descriptor["path"],
                        "--product-descriptor-sha256", digest]}


def fixed_editor_preview_native(native, native_ref, command, repo):
    proof = native_terminal_proof(native, native_ref)
    proof["complete"] = (proof["complete"] and native["command"] == command
                         and native["cwd"] == str(repo))
    proof["naturalSuccess"] = (proof["complete"] and native["rootExitCode"] == 0
                               and native.get("cleanup", {}).get("attempted") is False)
    return proof


def fixed_editor_preview_receipt(terminal, descriptor, pixel):
    def exited(rows):
        return (isinstance(rows, list) and bool(rows)
                and all(isinstance(row, dict) and row.get("waitedForExit") is True
                        and type(row.get("exitCode")) is int and row["exitCode"] == 0 for row in rows))
    desktop = terminal.get("existingDesktop", {})
    if (terminal.get("schema") != "current-unreal-ordinary-window-preview/v1"
            or terminal.get("status") != "preview-pixel-verified"
            or terminal.get("productDescriptor") != descriptor
            or terminal.get("failure") is not None or terminal.get("pixelFrameVerified") is not True
            or terminal.get("formalAcceptance") is not False or terminal.get("wholeUiAccepted") is not False
            or terminal.get("sourceCopiesCreated") is not False
            or desktop.get("windowStation") != "WinSta0" or desktop.get("desktop") != "Default"
            or type(desktop.get("foregroundWindow")) is not int or desktop["foregroundWindow"] <= 0
            or not exited(terminal.get("cleanup")) or not exited(terminal.get("auxiliaryCleanup"))
            or pixel.get("pixelFrameVerified") is not True):
        raise ValueError("This preview lacks its actual desktop, same-frame pixels or owned process closure")


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



def read_reference(reference, *, expected_path=None, max_bytes=8 * 1024**2):
    """Read bounded immutable evidence; a hash is required before interpreting it."""
    if (not isinstance(reference, dict) or set(reference) != {"path", "sha256"}
            or not isinstance(reference["sha256"], str)
            or not re.fullmatch(r"[0-9a-f]{64}", reference["sha256"])):
        raise ValueError("An exact immutable evidence reference is required")
    path = plain(reference["path"])
    if expected_path is not None and path != plain(expected_path):
        raise ValueError("Evidence escaped its original operation/step")
    if (type(max_bytes) is not int or not 1 <= max_bytes <= SOURCE_EVIDENCE_MAX_BYTES
            or not path.is_file() or path.stat().st_size > max_bytes):
        raise ValueError("Bounded immutable JSON evidence is unavailable")
    raw = path.read_bytes()
    if sha(raw) != reference["sha256"]:
        raise ValueError("Immutable evidence bytes changed")
    def pairs(items):
        value = {}
        for key, item in items:
            if key in value:
                raise ValueError("Duplicate evidence fields")
            value[key] = item
        return value
    value = json.loads(raw, object_pairs_hook=pairs)
    if not isinstance(value, dict) or path.read_bytes() != raw:
        raise ValueError("Evidence changed during observation")
    return value


class ReconciledBatchPreparation:
    """Replay only the exact, durably unlaunched materialization intent.

    The helper's insert-only execution guard remains the default. This adapter
    consumes that one guard/insert after reconciling the existing identical row;
    every preparation journal, CAS byte and physical inventory check still runs.
    It never deletes, replaces or conceals a launched execution.
    """
    def __init__(self, state, owner):
        self.state, self.owner, self.guard = state, owner, True

    def get(self, domain, key, **kwargs):
        row = self.state.get(domain, key, **kwargs)
        if domain == "execution" and key == self.owner and self.guard:
            self.guard = False
            if (not row or row["payload"].get("status") != "preparing"
                    or row["payload"].get("kind") != "management_owned_cargo"
                    or self.state.get("execution_launch", key, **kwargs)
                    or self.state.get("execution_host", key, **kwargs)
                    or any(item["payload"].get("executionId") == key
                           for item in self.state.list("native_job", **kwargs))):
                raise ValueError("Interrupted materialization is not durably unlaunched")
            return None
        return row

    def put(self, domain, key, payload, expected_version=None, **kwargs):
        if domain == "execution" and key == self.owner and expected_version == 0:
            row = self.state.get(domain, key, **kwargs)
            if not row or row["payload"] != payload:
                raise ValueError("Original unlaunched materialization identity changed")
            return row
        return self.state.put(domain, key, payload, expected_version=expected_version, **kwargs)

    def __getattr__(self, name):
        return getattr(self.state, name)


def source_helper():
    # boot verifies this file as part of the immutable capsule before use.
    path = plain(Path(__file__).with_name("source_preparation.py"))
    spec = importlib.util.spec_from_file_location("jenkins_management_source_preparation", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def checkout_guard_helper():
    # boot verifies this helper in the immutable management capsule.
    path = plain(Path(__file__).with_name("checkout_guard.py"))
    spec = importlib.util.spec_from_file_location("jenkins_management_checkout_guard", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


CHECKOUT_PRODUCT_CAPABILITIES = (
    {"enabled": True, "kind": "zircon-development-editor-preview", "schemaVersion": 1, "helper": "checkout_product.py"},
    {"enabled": True, "kind": "zircon-development-editor-preview", "schemaVersion": 2, "helper": "checkout_product.py"},
)


def checkout_product_helper(config, *, required_schema=1):
    checksum = config.get("files", {}).get("checkout_product.py")
    capability = config.get("checkoutProduct")
    if (capability not in CHECKOUT_PRODUCT_CAPABILITIES
            or type(required_schema) is not int or required_schema not in {1, 2}
            or capability["schemaVersion"] < required_schema
            or not isinstance(checksum, str) or not re.fullmatch(r"[0-9a-f]{64}", checksum)):
        raise ValueError("Fixed checkout product requires its installed pinned helper")
    path = plain(Path(__file__).with_name("checkout_product.py"))
    if sha(path.read_bytes()) != checksum:
        raise ValueError("Pinned checkout product helper changed")
    spec = importlib.util.spec_from_file_location("verified_checkout_product", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def cargo_manifest_environment(root, recipe):
    """Keep workspace selection in the pool identity and admitted source tree."""
    if "manifestPath" not in recipe:
        return {}
    relative = recipe["manifestPath"]
    if not isinstance(relative, str) or relative not in {"Cargo.toml", "zircon_plugins/Cargo.toml"}:
        raise ValueError("Unsupported catalog Cargo manifest")
    manifest = plain(Path(root) / relative)
    if not manifest.is_file():
        raise ValueError("Selected Cargo manifest is absent from the admitted source root")
    return {"cargoWorkspaceManifest": relative}


def plugin_workspace_inputs(root, recipe, captured):
    """Bind the bounded plugin route to captured manifests, lock and local edges.

    This is conservative input discovery, not Cargo resolution: optional, target,
    build and dev path declarations are all covered. Cargo still resolves actual
    features under --locked. No source is copied and cwd remains the checkout.
    """
    import tomllib
    if recipe.get("manifestPath") != "zircon_plugins/Cargo.toml":
        return None
    root = plain(root)
    if captured.get("sourceView") != "checkout" or plain(captured["sourceRoot"]) != root:
        raise ValueError("The plugin workspace requires the actual canonical checkout")
    helper = source_helper()
    sources = [(root, captured["source"])] + [
        (plain(value["path"]), value["source"]) for value in captured.get("externalRoots", [])]
    evidence = {}
    for directory, snapshot in sources:
        for name, checksum in helper.source_files(snapshot).items():
            path = str(plain(directory / name))
            if path in evidence and evidence[path] != checksum:
                raise ValueError("Conflicting captured workspace inputs")
            evidence[path] = checksum
    roots = sorted({directory for directory, _ in sources}, key=lambda path: len(str(path)), reverse=True)
    files, parsed, workspaces, packages, edges = {}, {}, {}, {}, []

    def boundary(path):
        path = plain(path)
        if not any(path.is_relative_to(directory) for directory in roots):
            raise ValueError("A plugin path dependency escapes the captured source roots: " + str(path))
        return path

    def read(path, *, optional=False, configuration=False):
        path = boundary(path)
        name = str(path)
        try:
            raw = path.read_bytes()
        except FileNotFoundError:
            if not optional:
                raise ValueError("A required plugin workspace input is absent: " + name)
            raw = None
        checksum = sha(raw) if raw is not None else None
        if (not configuration and (raw is not None and name not in evidence
                                  or name in evidence and evidence[name] != checksum)):
            raise ValueError("A plugin workspace input is not bound to its captured source: " + name)
        if name in files and files[name] != checksum:
            raise ValueError("A plugin workspace input changed during discovery: " + name)
        files[name] = checksum
        return raw

    def manifest(path, *, optional=False):
        path = boundary(path)
        raw = read(path, optional=optional)
        if raw is None:
            return None
        value = tomllib.loads(raw.decode("utf-8"))
        if path in parsed and parsed[path] != value:
            raise ValueError("A plugin manifest changed during discovery")
        parsed[path] = value
        return value

    def relative_path(base, value):
        # Cargo permits ../ path dependencies. Inspect every traversed component
        # before normalization so a reparse point cannot be hidden by '..'.
        if (not isinstance(value, str) or not value or "\\" in value or ":" in value
                or value.startswith("/") or any(c in value for c in "\0\r\n")):
            raise ValueError("Noncanonical plugin dependency path")
        path = plain(base)
        for part in value.split("/"):
            if not part:
                raise ValueError("Noncanonical plugin dependency path")
            path = plain(path.parent if part == ".." else path if part == "." else path / part)
        return boundary(path)

    def workspace(path, data):
        explicit = data.get("package", {}).get("workspace")
        if explicit is not None:
            candidates = [relative_path(path.parent, explicit) / "Cargo.toml"]
        else:
            limit = next(directory for directory in roots if path.is_relative_to(directory))
            candidates, directory = [], path.parent
            while directory.is_relative_to(limit):
                candidates.append(directory / "Cargo.toml")
                if directory == limit:
                    break
                directory = directory.parent
        for candidate in candidates:
            value = manifest(candidate, optional=True)
            if value is not None and isinstance(value.get("workspace"), dict):
                # Overrides can introduce inputs outside this path graph. Do not
                # silently call an incomplete graph a guarded dependency closure.
                if value.get("patch") or value.get("replace"):
                    raise ValueError("Plugin workspace overrides require an explicit source closure")
                workspaces[str(candidate)] = files[str(candidate)]
                return candidate, value["workspace"]
        raise ValueError("A plugin path package has no captured owning workspace: " + str(path))

    selected_manifest = root / "zircon_plugins/Cargo.toml"
    selected = manifest(selected_manifest)
    members = {"zircon_plugin_material_editor_editor": "material_editor/editor",
               "zircon_plugin_prefab_tools_editor": "prefab_tools/editor",
               "zircon_plugin_ai_editor": "ai/editor",
               "zircon_plugin_gltf_importer_runtime": "gltf_importer/runtime"}
    selected_packages = recipe.get("packages")
    if (not isinstance(selected_packages, list) or len(selected_packages) != 1
            or selected_packages[0] not in members):
        raise ValueError("The plugin source route requires one catalog package")
    selected_name = selected_packages[0]
    member = members[selected_name]
    if member not in selected.get("workspace", {}).get("members", []):
        raise ValueError("The selected package is not a member of the selected plugin workspace")
    lock_path = selected_manifest.with_name("Cargo.lock")
    lock = tomllib.loads(read(lock_path).decode("utf-8"))
    lock_packages = {(value.get("name"), value.get("version"))
                     for value in lock.get("package", []) if value.get("source") is None}
    pending = [(selected_manifest.parent / member / "Cargo.toml", selected_name)]
    while pending:
        path, expected_name = pending.pop()
        path = boundary(path)
        if path in packages:
            if packages[path]["name"] != expected_name:
                raise ValueError("Plugin dependency package/manifest mismatch")
            continue
        if len(packages) >= 256 or len(edges) > 4096:
            raise ValueError("Plugin dependency input discovery exceeded its bound")
        data = manifest(path)
        package = data.get("package", {})
        if package.get("name") != expected_name:
            raise ValueError("Plugin dependency package/manifest mismatch: " + str(path))
        owner, inherited = workspace(path, data)
        version = package.get("version")
        if isinstance(version, dict) and version.get("workspace") is True:
            version = inherited.get("package", {}).get("version")
        if not isinstance(version, str):
            raise ValueError("Plugin package version is not captured")
        packages[path] = {"name": expected_name, "version": version,
                          "workspaceManifest": str(owner), "sha256": files[str(path)],
                          "presentInSelectedLock": (expected_name, version) in lock_packages}
        tables = [(kind, data.get(kind, {})) for kind in ("dependencies", "build-dependencies", "dev-dependencies")]
        for target, target_data in data.get("target", {}).items():
            tables.extend(("target." + target + "." + kind, target_data.get(kind, {}))
                          for kind in ("dependencies", "build-dependencies", "dev-dependencies"))
        for kind, declarations in tables:
            for alias, declaration in declarations.items():
                if not isinstance(declaration, dict):
                    continue
                origin, dependency = path, declaration
                if declaration.get("workspace") is True:
                    shared = inherited.get("dependencies", {}).get(alias)
                    if shared is None:
                        raise ValueError("Missing inherited plugin dependency: " + alias)
                    if not isinstance(shared, dict):
                        continue
                    dependency, origin = shared, owner
                if "path" not in dependency:
                    continue
                destination = relative_path(origin.parent, dependency["path"]) / "Cargo.toml"
                target_name = dependency.get("package", alias)
                features = sorted(set(dependency.get("features", []) + declaration.get("features", [])))
                edges.append({"from": str(path), "fromPackage": expected_name, "kind": kind, "alias": alias,
                              "to": str(destination), "package": target_name, "declarationManifest": str(origin),
                              "declaredFeatures": features, "optional": declaration.get("optional", False),
                              "defaultFeatures": dependency.get("default-features", True)
                                                 and declaration.get("default-features", True)})
                pending.append((destination, target_name))
    selected_package = packages[selected_manifest.parent / member / "Cargo.toml"]
    if not selected_package["presentInSelectedLock"]:
        raise ValueError("The selected plugin package is absent from the plugin lockfile")
    if selected_name in {"zircon_plugin_material_editor_editor", "zircon_plugin_prefab_tools_editor",
                         "zircon_plugin_ai_editor"} and not (
            any(edge["package"] == "zircon_runtime" and "native-host" in edge["declaredFeatures"] for edge in edges)
            and any(edge["fromPackage"] == "zircon_editor" and edge["package"] == "zircon_runtime"
                    and "target-editor-host" in edge["declaredFeatures"] for edge in edges)):
        raise ValueError("The selected plugin editor route lost its declared native/editor runtime feature edges")
    if selected_name == "zircon_plugin_gltf_importer_runtime" and not any(
            edge["fromPackage"] == selected_name and edge["package"] == "zircon_runtime"
            and "native-host" in edge["declaredFeatures"] for edge in edges):
        raise ValueError("The glTF route lost its declared native runtime feature edge")
    configuration = []
    for manifest_name in sorted(workspaces):
        for name in ("config", "config.toml"):
            path = Path(manifest_name).parent / ".cargo" / name
            read(path, optional=True, configuration=True)
            configuration.append({"path": str(path), "sha256": files[str(path)],
                                  "effectiveFromCanonicalCwd": path.parent.parent == root})
    value = {"schemaVersion": 1, "manifestPath": recipe["manifestPath"], "cwd": str(root),
             "selectedPackage": selected_package["name"], "selectedLockfile": str(lock_path),
             "capturedSourceRoots": sorted(str(directory) for directory in roots),
             "workspaceManifests": dict(sorted(workspaces.items())),
             "packageManifests": {str(path): packages[path] for path in sorted(packages)},
             "pathDependencyEdges": sorted(edges, key=encoded), "cargoConfiguration": configuration,
             "files": dict(sorted(files.items())), "cargoResolutionPerformed": False}
    return {**value, "inputDigest": sha(encoded(value))}


def client_path(config, repo, name, *, capsule=None):
    """Resolve one pinned client; an absent clientRoot keeps canonical semantics."""
    clients = config.get("clientSources", {})
    checksum = clients.get(name) if isinstance(clients, dict) else None
    if (not isinstance(name, str) or name not in {
            "tools/dev/local_cargo.py", "tools/dev/local-cargo.ps1", "tools/dev/dev-fast-build.ps1",
            "tools/dev/submit-jenkins-management.ps1", "tools/jenkins/frontend.py",
            "tools/jenkins-management/entry.py"}
            or not isinstance(checksum, str) or not re.fullmatch(r"[0-9a-f]{64}", checksum)):
        raise ValueError("Management client source inventory differs")
    repo = plain(repo)
    if "clientRoot" not in config:
        root = repo
    elif config["clientRoot"] == "clients":
        capsule = plain(capsule if capsule is not None else Path(__file__).parent)
        raw = plain(capsule / "capsule.json").read_bytes()
        if (capsule != repo / ".jenkins/runtime/management" / capsule.name
                or sha(raw) != capsule.name or json.loads(raw) != config
                or plain(config["repositoryRoot"]) != repo):
            raise ValueError("Management client capsule identity differs")
        root = plain(capsule / "clients")
    else:
        raise ValueError("Management client root differs")
    path = plain(root / name)
    if sha(path.read_bytes()) != checksum:
        raise ValueError("Pinned management client implementation changed")
    return path



def boot(capsule):
    capsule = plain(capsule)
    raw = (capsule / "capsule.json").read_bytes()
    config = json.loads(raw)
    if config.get("schemaVersion") != 1 or capsule.name != sha(raw):
        raise ValueError("Management capsule identity differs")
    legacy_files = {"runner.py", "source_preparation.py", "zircon-management.groovy"}
    checkout_files = legacy_files | {"checkout_guard.py"}
    product_files = checkout_files | {"checkout_product.py"}
    if set(config["files"]) not in (legacy_files, checkout_files, product_files):
        raise ValueError("Management capsule has an incomplete pinned implementation inventory")
    checkout_policy = config.get("checkoutValidation")
    if checkout_policy is not None and (set(config["files"]) not in (checkout_files, product_files) or checkout_policy != {
            "enabled": True, "policy": "actual-checkout", "guard": "checkout_guard.py"}):
        raise ValueError("Actual checkout metadata requires its complete pinned guard inventory")
    product_capability = config.get("checkoutProduct")
    if ((set(config["files"]) == product_files and product_capability not in CHECKOUT_PRODUCT_CAPABILITIES)
            or (set(config["files"]) != product_files and product_capability is not None)):
        raise ValueError("Checkout product capability differs from its exact pinned inventory")
    if set(config["files"]) == product_files and checkout_policy != {
            "enabled": True, "policy": "actual-checkout", "guard": "checkout_guard.py"}:
        raise ValueError("Checkout product requires its continuous pinned guard")
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
    for name in config["clientSources"]:
        client_path(config, repo, name, capsule=capsule)
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
    if config.get("managedWorktree") is not None:
        if (config["managedWorktree"] != MANAGED_WORKTREE_CAPABILITY
                or not {"tools/jenkins/managed_worktree.py", "tools/jenkins/managed_worktree_lifetime.py",
                        "tools/jenkins/product_inputs.py"} <= set(manifest["files"])):
            raise ValueError("Managed worktree capability lacks its exact sealed implementation")
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



def route_request_context(config, repo, manager, payload, *, state=None, query_only=False):
    """Bind managed writer/reader identities before durable request claims."""
    if payload.get("action") not in {
            "create-worktree", "prepare-merge", "prepare-worktree", "prepare-flow",
            "submit-flow", "inspect-worktree", "apply-patch", "generate"}:
        return dict(payload), None
    return source_helper().route_managed_context(
        config, repo, manager, payload, state=state, query_only=query_only)


def ensure_request_context(config, repo, manager, payload, *, state=None, record_request=True):
    """Admit an ordinary local request without manual chat registration."""
    action = payload.get("action")
    if (AUTH.get(action) not in {"implementation", "validation"}
            or action in {"prepare-worktree", "prepare-flow", "submit-flow", "retire-worktree-reader"}
            or payload.get("worktreePreparationRef") or payload.get("observationOnly") is True):
        return None
    if state is None:
        from tools.jenkins.state import State
        state = State(repo / ".jenkins/state/coordination.sqlite3")
    helper = source_helper()
    return helper.admit_ordinary_context(
        state, repo, payload, principal=helper.local_request_principal(manager),
        driver_digest=config["driverDigest"], runtime_operation_id=config["runtimeOperationId"],
        actions=[AUTH[action]], record_request=record_request)


def reconcile_prepared_external_capture(captured, original, *, legacy_sealed=False):
    """Reproduce the sealed v1 producer's path spelling from current verified bytes."""
    evidence = source_helper()
    evidence.checked_source_evidence(captured)
    evidence.checked_source_evidence(original)
    if captured == original:
        return captured, None
    legacy_keys = {"head", "files", "diffSha256", "sourceDigest"}
    if not legacy_sealed or set(captured) != legacy_keys or set(original) != legacy_keys:
        raise ValueError("External source differs from its exact original prepared evidence")
    rewrites = []

    def legacy(value, repository=""):
        if set(value) != legacy_keys:
            raise ValueError("Legacy sealed external reconciliation requires v1 at every Git root")
        files, seen = {}, set()
        for name, checksum in value["files"].items():
            folded = name.casefold()
            if folded in seen:
                raise ValueError("External source paths collide after case folding")
            seen.add(folded)
            if isinstance(checksum, dict):
                # The sealed producer preserves Gitlink spelling, including each child root.
                files[name] = {"gitlinkCommit": checksum["gitlinkCommit"],
                               "source": legacy(checksum["source"], repository + name + "/")}
            else:
                files[folded] = checksum
                if folded != name:
                    rewrites.append({"repository": repository.rstrip("/"), "from": name, "to": folded})
        current = {"head": value["head"], "files": files, "diffSha256": value["diffSha256"]}
        return {**current, "sourceDigest": sha(encoded(current))}

    reconciled = legacy(captured)
    if reconciled != original:
        changed = [field for field in ("head", "diffSha256", "files", "sourceDigest")
                   if reconciled[field] != original[field]]
        raise ValueError("External source differs from its exact original prepared evidence; changed fields: "
                         + ", ".join(changed))
    rewrites.sort(key=lambda value: (value["repository"], value["from"], value["to"]))
    return reconciled, {"kind": "sealed-legacy-v1-external-path-reconciliation",
        "comparisonPolicy": "casefold-ordinary-files-preserve-gitlinks-v1",
        "actualCaptureDigest": captured["sourceDigest"], "reconciledCaptureDigest": reconciled["sourceDigest"],
        "preparedCaptureDigest": original["sourceDigest"], "pathRewrites": rewrites,
        "exactHeadDiffFilesAndNestedGitlinks": True, "originalEvidencePreserved": True}


class _BranchCandidate:
    """Private native queue binding; never a public worktree or sourceView."""

    def __init__(self, repo, root, request, runtime, component_digest):
        self.repo, self.path = plain(repo), plain(root)
        self.request = json.loads(encoded(request))
        self.runtime = dict(runtime)
        if set(self.runtime) != {"driverDigest", "runtimeOperationId"}:
            raise ValueError("Branch preparation needs the exact current runtime binding")
        if (self.path != self.repo / ".jenkins/worktrees/integration-queue"
                or not re.fullmatch(r"[0-9a-f]{64}", component_digest)):
            raise ValueError("Branch preparation requires the one native queue candidate")
        operation = identifier(request["operationId"])
        self.record_path = self.repo / ".jenkins/state/management/branch-integration/requests" / (operation + ".json")
        self.marker_path = self.record_path.parent.parent / "candidate.json"
        record = json.loads(plain(self.record_path).read_bytes())
        self.binding = {
            "requestDigest": sha(encoded(request)), "operationId": operation,
            "sessionId": request["sessionId"], "requestId": request["requestId"],
            "ownerThreadId": request.get("ownerThreadId"), "ownedPaths": request["paths"],
            "sourceCommit": record.get("sourceCommit"), "sourceRoot": record.get("sourceRoot"),
            "branch": record.get("branch"), "candidateHead": record.get("candidateHead"),
            "mainHeadBefore": record.get("mainHeadBefore"),
            "originalRuntime": record.get("runtime"), "runtime": self.runtime,
            "componentDigest": component_digest,
        }
        self.before = None

    def verify(self, management):
        if (os.name != "nt" or os.environ.get("JOB_NAME") != "zircon-merge-queue"
                or os.environ.get("NODE_NAME") != "zircon-windows-agent"
                or os.environ.get("ZR_COMMIT_NATIVE") != "1"
                or any(os.environ.get("ZR_INTEGRATION_" + suffix) != self.runtime[field]
                       for field, suffix in (("driverDigest", "DRIVER"), ("runtimeOperationId", "RUNTIME")))):
            raise ValueError("Prepared branch input requires the real runtime-bound merge job")
        if (management.repo != self.repo or management.session != self.binding["sessionId"]
                or management.owned != self.binding["ownedPaths"]
                or any(management.config[field] != self.runtime[field] for field in self.runtime)):
            raise ValueError("Branch candidate owner, scope or current runtime changed")
        selected = json.loads(plain(self.repo / ".jenkins/state/deployment/driver.json").read_bytes())
        operation = json.loads(plain(self.repo / ".jenkins/state/deployment-operation.json").read_bytes())
        if (any(selected.get(field) != self.runtime[field] for field in self.runtime)
                or operation.get("operationId") != self.runtime["runtimeOperationId"]
                or operation.get("state") != "running"):
            raise ValueError("Branch preparation runtime is transitioning")
        record = json.loads(plain(self.record_path).read_bytes())
        marker = json.loads(plain(self.marker_path).read_bytes())
        commit_paths = record.get("commitPaths", [])
        owned_paths = self.binding["ownedPaths"]
        committed_casefold = {name.casefold() for name in commit_paths}
        owned_casefold = {name.casefold() for name in owned_paths}
        expected = {name: self.binding[name] for name in ("sourceCommit", "sourceRoot", "branch", "candidateHead", "mainHeadBefore")}
        if (record.get("request") != self.request or record.get("runtime") != self.binding["originalRuntime"]
                or any(record.get(name) != value for name, value in expected.items())
                or record.get("integrationStatus") != "validating"
                or plain(record.get("candidateRoot", "")) != self.path
                or marker.get("status") != "active" or marker.get("operationId") != self.binding["operationId"]
                or marker.get("mainHead") != self.binding["mainHeadBefore"]
                or plain(marker.get("path", "")) != self.path
                or len(committed_casefold) != len(commit_paths)
                or len(owned_casefold) != len(owned_paths)
                or not committed_casefold <= owned_casefold):
            raise ValueError("Original branch queue candidate binding changed")
        native = record.get("candidateGit", {})
        command_ref = native.get("commandRef")
        if (native.get("operationKind") != "merge" or native.get("nativeComplete") is not True
                or native.get("rootExitCode") != 0 or not isinstance(command_ref, dict)):
            raise ValueError("Candidate merge lacks its original completed native evidence")
        command = read_reference({key: command_ref[key] for key in ("path", "sha256")},
                                 expected_path=plain(native["reportRoot"]) / "command.json")
        terminal_ref = native.get("nativeTerminalRef")
        terminal = read_reference({key: terminal_ref[key] for key in ("path", "sha256")},
                                  expected_path=plain(native["reportRoot"]) / "native-terminal.json")
        if (command.get("runtime") != self.runtime or command.get("operationId") != self.binding["operationId"]
                or command.get("operationKind") != "merge" or plain(command.get("cwd", "")) != self.path
                or command.get("command", [])[-1:] != [self.binding["sourceCommit"]]
                or terminal.get("command") != command.get("command")
                or terminal.get("rootExitCode") != 0
                or native_terminal_proof(terminal, terminal_ref)["complete"] is not True):
            raise ValueError("Candidate merge proof differs from the current native queue binding")
        if (management.text(self.path, "rev-parse", "HEAD") != self.binding["candidateHead"]
                or plain(management.text(self.path, "rev-parse", "--path-format=absolute", "--git-common-dir"))
                   != plain(self.repo / ".git")
                or management.text(self.path, "status", "--porcelain=v1", "--untracked-files=all")):
            raise ValueError("Candidate HEAD, common repository or source bytes changed")
        return self.path


def validate_branch_candidate(capsule, *, component_digest, root, branch_request, cargo_recipe, ordinal, runtime):
    """Use the existing managed preparation/execution lifecycle for a queue-owned candidate."""
    recipe_fields = {"verb", "packages", "features", "profile", "offline", "noDefaultFeatures", "allFeatures",
                     "locked", "jobs", "lib", "tests", "noRun", "testThreads", "testFilter", "bins",
                     "allTests", "harnessFilters", "exact", "ignored", "nocapture"}
    if not isinstance(cargo_recipe, dict) or set(cargo_recipe) - recipe_fields:
        raise ValueError("Only the existing bounded Cargo recipe belongs to branch preparation")
    recipe_check = object.__new__(Management)
    recipe_check.payload = cargo_recipe
    recipe_check.validation_recipe()
    config, repo, manager = boot(capsule)
    from tools.jenkins.source import repository_id

    candidate = _BranchCandidate(repo, root, branch_request, runtime, component_digest)
    fixed = {"action": "validate", "repositoryId": repository_id(repo),
        "sessionId": branch_request["sessionId"], "requestId": branch_request["requestId"],
        "operationId": "branch-input-" + sha(encoded({
            "operation": branch_request["operationId"], "head": candidate.binding["candidateHead"],
            "ordinal": ordinal, "recipe": cargo_recipe, "runtime": runtime,
            "capsule": Path(capsule).name, "component": component_digest}))[:48],
        "driverDigest": config["driverDigest"], "runtimeOperationId": config["runtimeOperationId"],
        "ownedPaths": list(branch_request["paths"]), "externalRoots": [str(repo.parent / "zr_vm")],
        "buildRoot": "E:/cargo-targets", "sourceEvidencePolicy": "full-file-index-v2", **cargo_recipe}
    # Observation retains the original authorization and performs no local request registration.
    probe = Management(config, repo, manager, {**fixed, "observationOnly": True}, branch_candidate=candidate)
    candidate.verify(probe)
    if not plain(repo.parent / "zr_vm").is_dir():
        return {"status": "waiting", "reasonCode": "external_source_unavailable",
                "preparationStarted": False, "compileStarted": False, "formalAcceptance": False}
    _root, before = probe.validation_with_cache()
    environment, trusted, _policy_file = probe.validation_environment()
    for tool in ("cargo", "rustc"):
        if sha(plain(trusted[tool + "Path"]).read_bytes()) != trusted[tool + "Sha256"]:
            raise ValueError("Trusted compiler executable changed")
    identity = {"schemaVersion": 1, "backend": "managed-prepared-branch-v1",
        "capsuleSha256": Path(capsule).name, "componentDigest": component_digest,
        "candidateHead": candidate.binding["candidateHead"], "runtime": runtime,
        "capturedSourceDigest": before["sourceDigest"], "engineSourceDigest": before["source"]["sourceDigest"],
        "externalSources": [{"path": item["path"], "sourceDigest": item["source"]["sourceDigest"]}
                           for item in before["externalRoots"]],
        "dependencyCacheDigest": sha(encoded(before["dependencyCache"])),
        "toolchainDigest": sha(encoded(trusted)), "environmentDigest": sha(encoded(environment)),
        "recipeDigest": sha(encoded(cargo_recipe)), "ordinal": ordinal}
    identity_digest = sha(encoded(identity))
    payload = {**fixed, "operationId": "branch-validation-" + identity_digest[:48],
               "expectedSourceDigest": before["sourceDigest"]}
    candidate.before = before
    actor = Management(config, repo, manager, payload, branch_candidate=candidate)
    result = actor.run()
    return {**result, "candidatePreparationIdentity": identity,
            "candidatePreparationIdentityDigest": identity_digest,
            "sealedInputRef": result.get("preparation", {}).get("sealedInputRef"),
            "branchCandidateBinding": candidate.binding}


class Management:
    def __init__(self, config, repo, manager, payload, *, branch_candidate=None):
        self._branch_candidate = branch_candidate
        from tools.jenkins.state import State
        from tools.jenkins.source import repository_id
        self.config, self.repo, self.manager, self.payload = config, repo, manager, payload
        self.state = source_helper().scoped_authorization_state(State(repo / ".jenkins/state/coordination.sqlite3"))
        self.action = payload["action"]
        self.source_view = payload.get("sourceView", "worktree")
        if self.source_view not in {"worktree", "checkout"}:
            raise ValueError("Unsupported validation source view")
        if self.source_view == "checkout" and self.action != "validate":
            raise ValueError("The current checkout source view is validation-only")
        if self.action not in {"validate", "prepare-worktree", "inspect-worktree"} and "externalRoots" in payload:
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
        if ((self.action in {"prepare-worktree", "inspect-worktree", "retire-worktree-reader"} or payload.get("worktreePreparationRef"))
                and config.get("managedWorktree") != MANAGED_WORKTREE_CAPABILITY):
            raise ValueError("Managed source preparation is not installed")
        self.owned = payload.get("ownedPaths", [])
        if not isinstance(self.owned, list) or not self.owned or any(not isinstance(p, str) for p in self.owned):
            raise ValueError("ownedPaths must be an explicit path list")
        if branch_candidate is None:
            ensure_request_context(config, repo, manager, payload, state=self.state)
        elif self.action != "validate" or not isinstance(branch_candidate, _BranchCandidate):
            raise ValueError("Only the private native branch candidate retains its scoped writer")
        self.state.check_authorization(self.repo_id, self.session, AUTH[self.action], self.owned)
        if branch_candidate is not None:
            branch_candidate.verify(self)
        self.key = sha(encoded({"repositoryId": self.repo_id, "sessionId": self.session,
                               "requestId": self.request, "operationId": self.operation}))
        self.worktrees = plain(repo / ".jenkins/worktrees")
        self.query = payload.get("observationOnly") is True
        self.input_digest = sha(encoded({k: v for k, v in payload.items() if k != "observationOnly"}))

    def git_command(self, root, *args):
        binding = getattr(self, "_checkout_git_binding", None)
        program = binding["path"] if binding is not None else "git"
        checkout = ["-c", "checkout.workers=4"] if args[:2] == ("worktree", "add") else []
        return [program, "-c", "core.longpaths=true", "-c", "core.hooksPath=" + str(Path(__file__).parent / "no-hooks"),
                "-c", "core.autocrlf=false", *checkout, "-C", str(root), *args]

    def git(self, root, *args, check=True):
        from tools.jenkins.pilot.native.process_supervisor import ProcessSupervisor
        command = self.git_command(root, *args)
        digest_only = args == ("diff", "--no-ext-diff", "--no-textconv", "--binary", "--full-index", "HEAD", "--")
        stdout, stderr, checksum, size = bytearray(), bytearray(), hashlib.sha256(), [0]

        def output(_owner, channel, raw):
            if channel == "stdout":
                size[0] += len(raw)
                checksum.update(raw)
                if not digest_only:
                    stdout.extend(raw)
            else:
                stderr.extend(raw)
                del stderr[:-65536]

        supervisor = ProcessSupervisor(maximum_output_bytes=65536, output_callback=output, job_terminal_timeout=8)
        process = None
        try:
            process = supervisor.start(command, cwd=root,
                env={**os.environ, "GIT_TERMINAL_PROMPT": "0", "GIT_OPTIONAL_LOCKS": "0"},
                owner=(getattr(self, "key", "management-git"), "git", 0, 0))
            try:
                code = process.wait(timeout=3600 if args[:2] == ("worktree", "add") else 600)
            except subprocess.TimeoutExpired:
                process.terminate(reason="management_git_timeout")
                process.wait(timeout=8)
                raise
            result = subprocess.CompletedProcess(command, code, bytes(stdout), bytes(stderr))
            result.stdoutSha256, result.stdoutBytes = checksum.hexdigest(), size[0]
            result.nativeProof = {"pid": process.pid, "creationTime": str(process.creation_time),
                "commandDigest": sha(encoded(command)), "rootExitCode": code,
                "terminalScope": process.terminal_scope, "stdoutAndStderrEof": True}
            if check and code:
                raise ValueError(stderr_tail(result.stderr))
            return result
        finally:
            supervisor.close()

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
        evidence = source_helper() if "sourceEvidencePolicy" in payload else None
        metadata = evidence.full_file_index_metadata(self, root) if evidence is not None and evidence.full_file_index_requested(payload) else None
        if metadata is None:
            names = self.git(root, "ls-files", "--cached", "--others", "--exclude-standard", "-z").stdout
            names = list(dict.fromkeys(raw.decode("utf-8", "strict") for raw in names.split(b"\0") if raw))
        else:
            names = metadata["names"]
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
            if name.startswith((".git/", "target/")):
                raise ValueError("Unexpected source path: " + name)
            if input_stage is not None:
                return name, input_stage.checksum(name)
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
        stage = (source_helper().managed_input_stage(root, "capture-input")
                 if getattr(self, "action", payload.get("action")) in {"inspect-worktree", "prepare-worktree", "validate"}
                 else nullcontext())
        with stage as input_stage:
            with ThreadPoolExecutor(max_workers=4) as readers:
                for start in range(0, len(ordinary), 128):
                    for name, checksum in readers.map(capture, ordinary[start:start + 128]):
                        files[name] = checksum
                    if start and start % 4096 == 0:
                        print("[management] Input files captured: " + str(start) + "/" + str(len(ordinary)), flush=True)
        files.update(gitlinks)
        if metadata is not None:
            source = evidence.finish_full_file_index(self, root, files, metadata)
        else:
            value = {"head": self.text(root, "rev-parse", "HEAD"), "files": files,
                     "diffSha256": source_helper().source_diff_digest(self, root)}
            source = {**value, "sourceDigest": sha(encoded(value))}
        print("[management] Input capture complete: " + str(root), flush=True)
        return source

    def validation_root(self, *, payload=None):
        candidate = getattr(self, "_branch_candidate", None)
        if candidate is not None:
            return candidate.verify(self)
        payload = self.payload if payload is None else payload
        view = payload.get("sourceView", "worktree")
        if view == "worktree":
            return self.wt(payload["worktreeId"])[1]
        if view == "checkout" and not payload.get("worktreeId"):
            return plain(self.repo)
        raise ValueError("Unsupported validation source view")

    def validation_source(self, *, payload=None, observation_errors=None):
        payload = self.payload if payload is None else payload
        candidate = getattr(self, "_branch_candidate", None)
        view = "integration-candidate" if candidate is not None else payload.get("sourceView", "worktree")
        root = self.validation_root(payload=payload)
        recipes = payload.get("batch", {}).get("recipes", [payload])
        plugin_recipe = next((value for value in recipes
                              if value.get("manifestPath") == "zircon_plugins/Cargo.toml"), None)
        if plugin_recipe is not None and len(recipes) != 1:
            raise ValueError("The plugin workspace permits only one recipe per request")
        if plugin_recipe is not None and (view != "checkout" or root != plain(self.repo)):
            raise ValueError("The plugin workspace requires the actual canonical checkout")
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
        external, external_compatibility = [], []
        pinned_external = None
        if payload.get("sourceEvidencePolicy") == "full-file-index-v2" and payload.get("worktreePreparationRef"):
            helper = source_helper()
            row, _ = helper._managed_route_preparation(self.state, payload["worktreePreparationRef"],
                payload["worktreeId"], self.repo_id, self.config)
            prepared = row["payload"]
            if prepared.get("path") != str(root) or prepared.get("writerSessionId") != self.session:
                raise ValueError("Validation must retain its exact prepared worktree writer")
            pinned_external = prepared["externalRoots"]
            if [value["path"] for value in pinned_external] != requested:
                raise ValueError("Validation external roots differ from their completed preparation")
            for captured in pinned_external:
                helper.checked_source_evidence(captured["source"])
        prepared_context = None
        if pinned_external is not None and prepared.get("sourceKind") == "sealed-observed-canonical-input" and prepared.get("sourcePreparation") is True:
            engine = engine if engine is not None else self.source(root, payload=payload)
            prepared_context = helper.prepared_validation_context(self, root, engine, payload=payload)
            # These are verified original CAS tuples, never a claim about the live sibling.
            external = prepared_context["captures"]
        for item in ([] if prepared_context is not None else requested):
            try:
                dependency = plain(Path(item))
                if (not dependency.is_dir()
                        or plain(self.text(dependency, "rev-parse", "--show-toplevel")) != dependency):
                    raise ValueError("External input must be its actual physical Git root")
                if view in {"checkout", "integration-candidate"} and "externalGitlinks" not in payload:
                    from tools.jenkins import managed_worktree as formal
                    # Bind omitted Gitlinks to the complete observed external input.
                    _, captures, _ = formal._external_capture(
                        self, source_helper().checkout_external_file_view()
                        if view == "checkout" else source_helper(),
                        inspection=True, payload=payload)
                    external.extend(captures)
                else:
                    external_payload = payload
                    original_external = None
                    if pinned_external is not None:
                        original_external = next(value["source"] for value in pinned_external if value["path"] == item)
                        external_payload = helper.source_evidence_payload(payload, original_external)
                    captured = self.source(dependency, payload=external_payload)
                    if original_external is not None:
                        captured, compatibility = reconcile_prepared_external_capture(captured, original_external,
                            legacy_sealed=prepared.get("sourceKind") == "sealed-observed-canonical-input")
                        if compatibility is not None:
                            external_compatibility.append({"path": str(dependency), **compatibility})
                    external.append({"path": str(dependency), "source": captured})
            except Exception as error:
                if observation_errors is None:
                    raise
                observation_errors.append({"path": item, "message": str(error),
                                           "reasonCode": getattr(error, "code", None),
                                           "details": getattr(error, "details", None)})
        value = {"sourceView": view, "sourceRoot": str(root),
                 "source": engine if engine is not None else self.source(root, payload=payload),
                 "externalRoots": external}
        if candidate is not None:
            value["branchCandidate"] = candidate.binding
        if external_compatibility:
            value["externalCompatibility"] = external_compatibility
        if prepared_context is not None:
            value["externalInputPolicy"] = "prepared-sealed-CAS"
            value["preparedSealedInput"] = prepared_context["provenance"]
        if plugin_recipe is not None:
            value["cargoWorkspaceInputs"] = plugin_workspace_inputs(root, plugin_recipe, value)
        return root, {**value, "sourceDigest": sha(encoded(value)) if not observation_errors else None}

    def worktree_writer_binding(self):
        writer = self.state.check_authorization(self.repo_id, self.session, "implementation", self.owned)
        validation = self.state.check_authorization(self.repo_id, self.session, "validation", self.owned)
        if writer["snapshotDigest"] != validation["snapshotDigest"]:
            raise ValueError("Worktree writer authorization changed during creation")
        key = sha(encoded({"repositoryId": self.repo_id, "sessionId": self.session,
                           "requestId": self.request, "operationId": self.operation}))
        return {"repositoryId": self.repo_id, "runtimeOperationId": self.config["runtimeOperationId"],
                "creationOperationKey": key, "writerPrincipal": writer["principal"],
                "writerAuthorizationSnapshot": writer["snapshotDigest"]}

    def new_worktree(self, *, merge=False):
        name = identifier(self.payload["name"])
        reference = self.session + "-" + name
        if len(reference) > 160 or reference.endswith("."):
            raise ValueError("Managed worktree name is too long")
        path = plain(self.worktrees / reference)
        base = self.payload["baseHead"]
        if not isinstance(base, str) or not SHA.fullmatch(base):
            raise ValueError("An immutable base commit is required")
        # Resolve before the reuse branch: a recorded row stores the resolved
        # commit, so comparing it against the caller's raw spelling would report
        # a false drift whenever the two spellings differ.
        base = self.text(self.repo, "rev-parse", base + "^{commit}")
        existing = self.state.get("management_worktree", reference)
        if self.payload.get("resumeFromOperationKey") is not None and existing is None:
            raise WorktreeConflict(
                "Explicit resume requires an existing managed worktree record",
                reason="resume_record_missing", details={"reference": reference, "path": str(path)})
        if existing or path.exists():
            return self.existing_worktree(reference, path, existing, base, merge=merge)
        deepest_file, deepest_dir = deepest_tracked_paths(self.repo)
        fits, budget = managed_path_budget(path, deepest_file, deepest_dir)
        if not fits:
            raise ManagedPathBudget(
                "Managed worktree path exceeds the Windows MAX_PATH budget; "
                "shorten the session or worktree name before checkout",
                details={"worktree": str(path), "requiredBudget": budget,
                         "deepestFile": deepest_file, "deepestDir": deepest_dir,
                         "limit": MANAGED_PATH_BUDGET})
        binding = self.worktree_writer_binding()
        self.worktrees.mkdir(parents=True, exist_ok=True)
        branch = None if merge else "codex/jenkins/" + self.session + "/" + name
        args = ["worktree", "add", "--detach"] if merge else ["worktree", "add", "-b", branch]
        self.git(self.repo, *args, "--", str(path), base)
        record = {"status": "ready", "path": str(path), "branch": branch, "baseHead": base,
                  "sessionId": self.session, "requestId": self.request, "operationId": self.operation,
                  "ownedPaths": self.owned, "driverDigest": self.config["driverDigest"], **binding}
        self.state.put("management_worktree", reference, record, expected_version=0)
        return reference, path, record

    def existing_worktree(self, reference, path, existing, base, *, merge):
        """Reuse, adopt, or explicitly refuse a worktree that already exists.

        A crashed or interrupted ``create-worktree`` leaves either a state row,
        a physical directory, or both.  Rebinding must be decided from the real
        Git identity rather than from path existence alone, because a stale
        directory from an unrelated attempt has no .git and must never be
        adopted silently.
        """
        name = reference.split("-", 1)[-1] if reference.startswith(self.session + "-") else reference
        branch = None if merge else "codex/jenkins/" + self.session + "/" + name
        if existing is not None:
            if self.payload.get("resumeFromOperationKey") is not None:
                return self.resume_existing_worktree(reference, path, existing, base, merge=merge)
            value = existing["payload"]
            same_request = (value.get("sessionId") == self.session
                            and value.get("requestId") == self.request
                            and value.get("operationId") == self.operation
                            and value.get("baseHead") == base
                            and (value.get("branch") is None) == merge)
            if same_request:
                if plain(value["path"]) != path:
                    raise WorktreeConflict(
                        "Registered worktree path differs from its recorded location",
                        reason="path_drift",
                        details={"reference": reference, "recorded": str(value.get("path")),
                                 "expected": str(path)})
                return reference, path, value
            raise WorktreeConflict(
                "Worktree reference is bound to a different operation",
                reason="reference_in_use",
                details={"reference": reference, "recordedSession": value.get("sessionId"),
                         "recordedBase": value.get("baseHead"), "expectedBase": base,
                         "recordedOperation": value.get("operationId")})
        # No state row: the directory is either a genuine Git worktree of this
        # repository that a crashed operation never registered, or foreign bytes.
        if not path.is_dir():
            raise WorktreeConflict(
                "An unregistered path blocks the managed worktree location",
                reason="unregistered_path",
                details={"reference": reference, "path": str(path)})
        try:
            common = plain(self.text(path, "rev-parse", "--path-format=absolute", "--git-common-dir"))
        except ValueError as error:
            raise WorktreeConflict(
                "Existing directory is not a Git worktree of this repository",
                reason="foreign_directory",
                details={"reference": reference, "path": str(path), "detail": str(error)}) from error
        # Ownership is decided before the commit is read: a directory that
        # belongs elsewhere is never interrogated for its history.
        if common != plain(self.repo / ".git"):
            raise WorktreeConflict(
                "Existing directory belongs to a different repository",
                reason="foreign_repository",
                details={"reference": reference, "path": str(path), "commonDir": str(common)})
        try:
            head = self.text(path, "rev-parse", "HEAD")
        except ValueError as error:
            raise WorktreeConflict(
                "Existing directory has no readable HEAD",
                reason="unreadable_head",
                details={"reference": reference, "path": str(path), "detail": str(error)}) from error
        if head != base:
            raise WorktreeConflict(
                "Unregistered worktree is checked out at a different commit",
                reason="base_head_drift",
                details={"reference": reference, "path": str(path), "head": head, "expectedBase": base})
        record = {"status": "ready", "path": str(path), "branch": branch, "baseHead": base,
                  "sessionId": self.session, "requestId": self.request, "operationId": self.operation,
                  "ownedPaths": self.owned, "driverDigest": self.config["driverDigest"],
                  "adopted": True, **self.worktree_writer_binding()}
        self.state.put("management_worktree", reference, record, expected_version=0)
        return reference, path, record

    def create_branch(self):
        name = identifier(self.payload["name"])
        base = self.payload["baseHead"]
        if not isinstance(base, str) or not SHA.fullmatch(base):
            raise ValueError("An immutable base commit is required")
        branch = "codex/jenkins/" + self.session + "/" + name
        head = self.text(self.repo, "rev-parse", base + "^{commit}")
        # A repeated create-branch carries no state row, so the branch ref is
        # the only record.  Reusing it when it already points at the requested
        # commit makes the action retry-safe; pointing anywhere else is a real
        # collision the caller must resolve rather than silently re-point.
        probe = self.git(self.repo, "rev-parse", "--verify", "--quiet", "refs/heads/" + branch + "^{commit}", check=False)
        if not probe.returncode:
            current = probe.stdout.decode("utf-8", "strict").strip()
            if current == head:
                return {"status": "complete", "branch": branch, "head": head, "reused": True}
            raise WorktreeConflict(
                "Branch already exists at a different commit",
                reason="branch_diverged",
                details={"branch": branch, "current": current, "expected": head})
        self.git(self.repo, "branch", "--", branch, head)
        return {"status": "complete", "branch": branch, "head": head, "reused": False}

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


    def same_boot_worktree_resume_guard(self, original, pins, *, reserved=False):
        """Retain a reviewed read-only UNKNOWN; never reconcile an old native/write claim."""
        from tools.jenkins.managed_worktree_lifetime import active_readers, assert_no_writer
        reference = self.payload["worktreeId"]
        worktree = self.state.get("management_worktree", reference)
        if not worktree:
            raise ValueError("Same-boot resume requires its exact reviewed WT record")
        record, operations = worktree["payload"], []
        required = {("management_worktree", reference), ("management_operation", original["key"])}
        observed = set()
        for pin in pins:
            identity = (pin.get("domain"), pin.get("key"))
            row = self.state.get(*identity)
            if (identity in observed or row is None or row["version"] != pin.get("version")
                    or sha(encoded(row["payload"])) != pin.get("payloadDigest")):
                raise ValueError("Same-boot resume reviewed State pin changed")
            observed.add(identity)
            if identity[0] == "management_operation":
                value = row["payload"]
                request, result = value.get("payload", {}), value.get("result", {})
                if request.get("worktreeId") == reference or result.get("worktreeId") == reference or row["key"] == original["key"]:
                    operations.append(row)
        if not required <= observed or record.get("sessionId") != self.session:
            raise ValueError("Same-boot resume lacks its reviewed original WT and creation")
        unknown = []
        for row in operations:
            if row["key"] == original["key"]:
                if row != original:
                    raise ValueError("Same-boot resume original creation changed")
                continue
            value, request = row["payload"], row["payload"].get("payload", {})
            if (value.get("phase") != "unknown" or value.get("action") != "inspect-worktree"
                    or request.get("action") != "inspect-worktree"
                    or request.get("sessionId") != self.session
                    or request.get("runtimeOperationId") != record.get("runtimeOperationId")
                    or request.get("driverDigest") != record.get("driverDigest")
                    or sorted(request.get("ownedPaths", [])) != sorted(self.owned)
                    or value.get("inputDigest") != sha(encoded(
                        {name: item for name, item in request.items() if name != "observationOnly"}))):
                raise ValueError("Same-boot resume supports only the exact quarantined read-only UNKNOWN inspection")
            unknown.append(row["key"])
        root = plain(record["path"])
        if active_readers(self.state, reference, root):
            raise ValueError("A registered source reader freezes this worktree")
        writer = self.state.get("managed_worktree_writer", reference)
        if reserved:
            if not writer or writer["payload"] != {"worktreeId": reference, "operationKey": self.key,
                    "inputDigest": self.input_digest, "status": "active"}:
                raise ValueError("Same-boot resume writer reservation changed")
        else:
            assert_no_writer(self.state, reference)
        owners = {"management-" + row["key"] for row in operations}
        for domain in ("execution", "execution_launch", "execution_host", "management_prelaunch", "native_job"):
            for row in self.state.list(domain):
                value = row["payload"]
                if (row["key"] in owners or value.get("owner") in owners or value.get("executionId") in owners
                        or (value.get("worktreeId") == reference or value.get("sourceRoot") == str(root))
                           and value.get("owner") != "management-" + self.key):
                    raise ValueError("Same-boot read-only resume cannot reconcile an existing execution or native claim")
        return sorted(unknown)

    def worktree_resume_release(self, reference):
        """Read historical runtime authority for metadata resume, without issuing a service action."""
        from tools.jenkins.deployment import recovery_keeper as keeper
        from tools.jenkins.deployment.lost_recovery_runtime import reviewed_marker
        repo, state = self.manager.paths.repo.absolute(), self.manager.paths.state.absolute()
        paths = {"config": repo / ".codex/config.toml",
                 "deploymentSpec": self.manager.spec.path.absolute(),
                 "workflowSpec": repo / ".jenkins/workflow-spec.json"}
        config_path = plain(paths["config"])

        def identity(value):
            return (value.st_dev, value.st_ino, value.st_mode, value.st_size,
                    value.st_mtime_ns, value.st_ctime_ns, value.st_nlink)

        with config_path.open("rb") as stream:
            config_stat = os.fstat(stream.fileno())
            path_stat = config_path.stat()
            if (config_stat.st_dev, config_stat.st_ino) != (path_stat.st_dev, path_stat.st_ino):
                raise ValueError("Current metadata configuration handle differs from its physical path")
            if not stat.S_ISREG(config_stat.st_mode):
                raise ValueError("Current metadata configuration must be an ordinary physical file")
            current = {"config": {"path": str(config_path), "sha256": sha(stream.read())}}
            if identity(os.fstat(stream.fileno())) != identity(config_stat):
                raise ValueError("Current metadata configuration changed while reading")
            for field in ("deploymentSpec", "workflowSpec"):
                current[field] = {"path": str(paths[field]), "sha256": keeper._sha(paths[field])}
            document = keeper._reference(state, reference, release=True)
            for field, path in paths.items():
                value = document.get(field)
                keeper._require(isinstance(value, dict) and set(value) == {"path", "sha256"}
                                and value["path"] == str(path),
                                "Historical metadata configuration reference differs")
                keeper.require_digest(value["sha256"], field)
                if field != "config":
                    keeper._require(value == current[field], "Recovery specification changed")
            changed = document["config"] != current["config"]
            if not changed:
                document, verified = keeper._release(self.manager, reference, current_source=False)
            else:
                # This reader consumes an already issued runtime. It cannot start, stop or transfer one.
                keeper._require(document.get("schemaVersion") == 1
                                and document.get("kind") == keeper.RELEASE_KIND
                                and document.get("scope") == "execution-runtime"
                                and re.fullmatch("[0-9a-f]{32}", str(document.get("requestId", "")))
                                and isinstance(document.get("lostRecoveryRuntime"), dict)
                                and all(document.get(name) is None for name in ("ownedUpgrade", "failedOwnStart"))
                                and all(document.get(name) is False for name in
                                        ("runtimeAcceptance", "cargoAcceptance", "automaticRestart", "queueReplayRequested")),
                                "Historical configuration compatibility is metadata-only lost-runtime recovery")
                target = document.get("targetDriver")
                keeper._require(isinstance(target, dict), "Recovery target driver is missing")
                root = repo / ".jenkins/runtime/drivers" / keeper.require_digest(
                    target.get("driverDigest"), "targetDriverDigest")
                keeper._require(target.get("root") == str(root)
                                and document.get("actorCodeRoot") == str(root)
                                and Path(keeper.__file__).absolute().parents[3] == root,
                                "Metadata reader must use the exact sealed recovery actor")
                verified = keeper.verify_driver(root, expected_digest=target["driverDigest"])
                keeper._require(target == verified, "Recovery release must pin the complete sealed target snapshot")
                required_operators = (*keeper._OPERATOR_FILES, "tools/jenkins/deployment/lost_recovery_runtime.py")
                required = {repo / name for name in required_operators} | {
                    state / "migration-baseline/jenkins_desktop_holder.py"}
                operators = document.get("operatorFiles")
                keeper._require(isinstance(operators, dict) and {str(path) for path in required} <= set(operators),
                                "Recovery release must retain its complete historical operator inventory")
                for name, checksum in operators.items():
                    keeper.require_digest(checksum, "operatorSha256")
                    path = keeper._operator_path(repo, verified, name, sealed=True)
                    keeper._require(keeper._sha(path) == checksum, "Reviewed recovery operator bytes changed")
                historical = keeper._historical_regression(repo, state, document.get("regression", {}), verified,
                                                            required_operators=required_operators)
                for field, name in (("deploymentSpec", ".jenkins/deployment-spec.json"),
                                    ("workflowSpec", ".jenkins/workflow-spec.json")):
                    keeper._require(historical["sourceFiles"].get(name) == document[field]["sha256"],
                                    "Historical tested specification changed")
                keeper._require(all(historical["sourceFiles"].get(Path(name).relative_to(repo).as_posix()) == checksum
                                    for name, checksum in operators.items()
                                    if Path(name).is_relative_to(repo / "tools/jenkins")
                                    and "/tests/" not in Path(name).relative_to(repo).as_posix()),
                                "Reviewed lifecycle operator differs from its historical tested implementation")
                preparation = keeper._reference(state, document.get("preparation"))
                keeper._require(preparation.get("schemaVersion") == 1
                                and preparation.get("state") == "prepared-not-selected"
                                and preparation.get("driver") == target
                                and preparation.get("supportRegression", preparation.get("regression")) == document["regression"]
                                and preparation.get("protectedHolds") == document.get("protectedHolds"),
                                "Recovery target lacks its exact historical preparation")
                holds = document.get("protectedHolds")
                current_holds = keeper.protected_holds(state)
                # Metadata resume preserves the historical rows; unrelated current work may add holds.
                keeper._require(isinstance(holds, list) and len(holds) == 4
                                and len({(row.get("domain"), row.get("key")) for row in holds
                                         if isinstance(row, dict)}) == 4
                                and all(row in current_holds for row in holds),
                                "The four protected historical holds must remain exact")
                keeper._failed_start_marker(state, document)
                keeper._owned_upgrade_marker(state, document)
                reviewed_marker(state, document)
                owner, fence = document.get("expectedPreviousOwner"), document.get("expectedLifecycleFence")
                fence_names = {"deployment-operation.json", "deployment/host.json",
                               "deployment/driver.json", *keeper._LEGACY_FILES}
                keeper._require(isinstance(owner, dict) and isinstance(fence, dict) and set(fence) == fence_names
                                and all(fence[name] is not None for name in
                                        fence_names - {"deployment/formal-lifecycle-command.json"}),
                                "Reviewed complete predecessor binding is missing")
                for checksum in fence.values():
                    if checksum is not None:
                        keeper.require_digest(checksum, "lifecycleFenceSha256")
                previous = preparation.get("previousSelectedDriver")
                keeper._require(isinstance(previous, dict), "Prepared predecessor selector is missing")
                old_root = repo / ".jenkins/runtime/drivers" / keeper.require_digest(
                    previous.get("driverDigest"), "previousDriverDigest")
                keeper._require(previous.get("root") == str(old_root)
                                and preparation.get("previousSelectorSha256") == fence["deployment/driver.json"]
                                and previous.get("runtimeOperationId") == owner.get("operation", {}).get("operationId"),
                                "Prepared predecessor selector changed")
                old_verified = keeper.verify_driver(old_root, expected_digest=previous["driverDigest"])
                keeper._require(all(previous.get(key) == value for key, value in old_verified.items()),
                                "Prepared predecessor driver bytes changed")
            plain(config_path)
            stream.seek(0)
            keeper._require(identity(os.fstat(stream.fileno())) == identity(config_stat)
                            and identity(config_path.stat()) == identity(path_stat)
                            and sha(stream.read()) == current["config"]["sha256"]
                            and identity(os.fstat(stream.fileno())) == identity(config_stat)
                            and all(keeper._sha(paths[field]) == current[field]["sha256"]
                                    for field in ("deploymentSpec", "workflowSpec")),
                            "Current metadata configuration or specification changed during observation")
        return document, verified, {
            "kind": "historical-runtime-configuration-observation",
            "purpose": "worktree-metadata-resume", "historicalConfig": document["config"],
            "currentConfig": current["config"], "currentDeploymentSpec": current["deploymentSpec"],
            "currentWorkflowSpec": current["workflowSpec"],
            "currentConfigFileIdentity": {"device": config_stat.st_dev, "inode": config_stat.st_ino},
            "configurationChangedSinceRuntimeSpawn": changed,
            "runtimeReissued": False, "serviceLifecyclePerformed": False}

    def worktree_resume_lineage(self, previous_runtime, original, *, reserved=False):
        """Bind metadata resume to its exact reviewed predecessor and current native owner."""
        from tools.jenkins.deployment.lost_recovery_runtime import (
            SAME_BOOT_VARIANT, reviewed_marker, validate_interruption_receipt, _departure_binding)
        from tools.jenkins.deployment.owned_upgrade import _committed_command, _departure
        from tools.jenkins.deployment.recovery_keeper import (
            DESCRIPTOR_KIND, _read, _reference, _runtime_receipt)
        from tools.jenkins.processes.boot import windows_boot_filetime
        state = plain(self.repo / ".jenkins/state")
        descriptor = _read(state / "formal-lifecycle/recovery-keeper.json")
        command = _read(state / "recovery-lifecycle/command.json")
        release, target, configuration = self.worktree_resume_release(descriptor.get("release"))
        planned = reviewed_marker(state, release)
        boot = str(windows_boot_filetime())
        runtime = descriptor.get("runtime", {})
        previous_driver = original["payload"]["payload"].get("driverDigest")
        if (not isinstance(previous_driver, str) or not re.fullmatch(r"[0-9a-f]{64}", previous_driver)
                or descriptor.get("schemaVersion") != 1 or descriptor.get("kind") != DESCRIPTOR_KIND
                or descriptor.get("admission") != "idle" or descriptor.get("automaticRestart") is not False
                or any(descriptor.get(name) is not False for name in
                       ("runtimeAcceptance", "cargoAcceptance", "historicalHoldsReleased"))
                or runtime.get("state") != "running"
                or not planned or planned.get("recoveryVariant") != SAME_BOOT_VARIANT
                or planned.get("originalCommandStatus") != "committed"
                or planned.get("operationId") != previous_runtime
                or planned.get("bootFileTime") != boot or descriptor.get("bootFileTime") != boot
                or not str(planned.get("previousBootFileTime", "")).isdigit()
                or int(planned["previousBootFileTime"]) > int(boot)
                or runtime.get("operation", {}).get("operationId") != self.config["runtimeOperationId"]
                or runtime.get("selected", {}).get("runtimeOperationId") != self.config["runtimeOperationId"]
                or runtime.get("selected", {}).get("driverDigest") != self.config["driverDigest"]
                or target.get("driverDigest") != self.config["driverDigest"]):
            raise ValueError("Worktree resume requires the exact committed reviewed recovery")
        receipt = _runtime_receipt(self.manager, descriptor, release, target)
        _committed_command(state, descriptor, receipt, command)
        interruption = validate_interruption_receipt(state, descriptor, release)
        archives = interruption["originalArtifacts"]
        archived = {name: _reference(state, binding["archive"]) for name, binding in archives.items()}
        old_descriptor, old_receipt = archived["originalDescriptor"], archived["previousRuntimeReceipt"]
        old_runtime = old_descriptor.get("runtime", {})
        old_selected = old_runtime.get("selected", {})
        # The reviewed preparation verifies the sealed old driver. Its exact
        # selector and archived native issuance must also name this WT's driver.
        preparation = _reference(state, release["preparation"])
        if (old_descriptor.get("schemaVersion") != 1 or old_descriptor.get("kind") != DESCRIPTOR_KIND
                or old_descriptor.get("admission") != "idle" or old_runtime.get("state") != "running"
                or old_runtime.get("operation", {}).get("operationId") != previous_runtime
                or old_selected.get("runtimeOperationId") != previous_runtime
                or old_selected.get("driverDigest") != previous_driver
                or old_selected != preparation.get("previousSelectedDriver")
                or archives["previousSelector"]["original"]["sha256"] != preparation.get("previousSelectorSha256")
                or sha(encoded(old_selected)) != planned.get("selectedDriverDigest")
                or old_runtime.get("operation") != archived["previousOperation"]
                or old_runtime.get("host") != archived["previousHost"]
                or old_selected != archived["previousSelector"]
                or old_descriptor.get("runtimeReceipt") != planned.get("previousRuntimeReceipt")
                or old_descriptor.get("release") != planned.get("previousRecoveryRelease")
                or old_descriptor.get("bootFileTime") != planned["previousBootFileTime"]
                or old_receipt.get("kind") != "recovery-keeper-runtime"
                or old_receipt.get("runtime") != old_runtime
                or old_receipt.get("keeper") != old_descriptor.get("keeper")
                or old_receipt.get("release") != old_descriptor.get("release")
                or old_receipt.get("bootFileTime") != planned["previousBootFileTime"]
                or any(old_receipt.get(name) is not False for name in
                       ("runtimeAcceptance", "cargoAcceptance", "nativeProofSynthesized",
                        "historicalHoldsReleased", "queueReplayRequested"))
                or self.manager._read() != runtime["operation"]
                or _read(state / "deployment/host.json") != runtime["host"]
                or _read(state / "deployment/driver.json") != runtime["selected"]):
            raise ValueError("Worktree resume driver lineage differs from the archived and current native runtime")
        pin = {"domain": "management_operation", "key": original["key"],
               "version": original["version"], "payloadDigest": sha(encoded(original["payload"]))}
        quarantine = planned.get("managementQuarantine", {})
        if pin not in quarantine.get("records", []):
            raise ValueError("Worktree creation differs from the reviewed predecessor operation")
        same_boot = planned["previousBootFileTime"] == boot
        extra = {}
        if same_boot:
            if (not isinstance(quarantine.get("currentRecords"), list)
                    or any(quarantine.get(name) is not False for name in
                           ("nativeProofIssued", "artifactAcceptance", "executionHoldsReleaseAllowed"))):
                raise ValueError("Same-boot WT resume requires the exact non-accepting management quarantine")
            indexed = {(value.get("domain"), value.get("key")): value for value in quarantine["records"]}
            pins = []
            for binding in quarantine["currentRecords"]:
                selected = indexed.get((binding.get("domain"), binding.get("key")))
                if selected is None:
                    raise ValueError("Same-boot WT resume has an unpinned current record")
                pins.append(selected)
            unknown = self.same_boot_worktree_resume_guard(original, pins, reserved=reserved)
            identities = {**old_runtime.get("owner", {}), "keeper": old_descriptor.get("keeper")}
            if (set(identities) != {"supervisor", "controller", "agent", "keeper"}
                    or any(planned.get("owner", {}).get(role) != identities[role]
                           for role in ("supervisor", "controller", "agent"))
                    or planned.get("departedKeeper") != identities["keeper"]):
                raise ValueError("Same-boot WT resume departed owners differ from the exact archived identities")
            departures = {}
            for role, identity in identities.items():
                reviewed = _departure_binding(planned.get("departedIdentities", {}).get(role))
                if (reviewed != {"pid": identity["pid"], "expectedCreationTime": identity["creationTime"]}
                        or int(identity["creationTime"]) < int(boot)):
                    raise ValueError("Same-boot WT resume owner birth differs from its reviewed departure")
                departures[role] = _departure(identity)
            extra = {"reviewedStatePins": pins, "preservedUnknownInspectOperationKeys": unknown,
                "currentDepartureObservations": departures,
                "oldNativeTerminationClaimed": False, "historicalEofClaimed": False,
                "historicalHoldsReleased": False, "queueReplayRequested": False}
        return {"kind": "managed-worktree-same-boot-resume" if same_boot else "managed-worktree-later-boot-resume",
                "release": descriptor["release"],
                "configurationEvidence": configuration,
                "reviewedPlan": release["lostRecoveryRuntime"]["plan"],
                "interruption": descriptor["lostRecoveryRuntimeInterruption"],
                "predecessorArtifacts": archives, "preparation": release["preparation"],
                "previousRuntimeOperationId": previous_runtime, "previousDriverDigest": previous_driver,
                "runtimeOperationId": self.config["runtimeOperationId"], "targetDriverDigest": target["driverDigest"],
                "previousBootFileTime": planned["previousBootFileTime"], "bootFileTime": boot,
                "originalOperationPin": pin, "nativeProofIssued": False, **extra}

    def resume_existing_worktree(self, reference, path, existing, base, *, merge):
        """Rebind verified unchanged WT metadata; never rewrite its original result."""
        from tools.jenkins.managed_worktree_lifetime import writer_hold
        if "committedSuccessor" in self.payload:
            if self.config.get("managedWorktree") != MANAGED_WORKTREE_CAPABILITY:
                raise ValueError("Managed committed source preparation is not installed")
            return source_helper().resume_committed_worktree(self, reference, path, existing, base, merge=merge)
        value = existing["payload"]
        key = self.payload.get("resumeFromOperationKey")
        branch = "codex/jenkins/" + self.session + "/" + self.payload["name"]
        if (merge or self.config.get("managedWorktree") != MANAGED_WORKTREE_CAPABILITY
                or self.payload.get("worktreeId") != reference
                or not isinstance(key, str) or not re.fullmatch(r"[0-9a-f]{64}", key)
                or key != value.get("creationOperationKey") or value.get("status") != "ready"
                or value.get("repositoryId") != self.repo_id or value.get("sessionId") != self.session
                or not isinstance(value.get("driverDigest"), str)
                or not re.fullmatch(r"[0-9a-f]{64}", value["driverDigest"])
                or value.get("runtimeOperationId") == self.config["runtimeOperationId"]
                or plain(value["path"]) != path or value.get("baseHead") != base
                or value.get("branch") != branch or sorted(value.get("ownedPaths", [])) != sorted(self.owned)):
            raise ValueError("Worktree resume requires its exact ready predecessor and original writer")
        original = self.state.get("management_operation", key)
        operation = original["payload"] if original else {}
        payload, result = operation.get("payload", {}), operation.get("result", {})
        identities = {name: payload.get(name) for name in ("repositoryId", "sessionId", "requestId", "operationId")}
        if (operation.get("phase") != "complete" or operation.get("action") != "create-worktree"
                or sha(encoded(identities)) != key
                or operation.get("inputDigest") != sha(encoded({k: v for k, v in payload.items() if k != "observationOnly"}))
                or any(payload.get(n) != value.get(n) for n in
                       ("repositoryId", "sessionId", "requestId", "operationId", "driverDigest", "runtimeOperationId"))
                or result.get("status") != "complete" or result.get("operationKey") != key
                or result.get("worktreeId") != reference or result.get("baseHead") != base
                or any(result.get(n) != value.get(n) for n in
                       ("driverDigest", "runtimeOperationId", "writerPrincipal"))
                or result.get("writerAuthorizationSnapshot") != value.get("writerAuthorizationSnapshot")):
            raise ValueError("Worktree resume requires its original completed creation receipt")
        lineage = self.worktree_resume_lineage(value["runtimeOperationId"], original)
        binding = self.worktree_writer_binding()
        if (binding["writerPrincipal"] != value.get("writerPrincipal")
                or binding["writerAuthorizationSnapshot"] != value.get("writerAuthorizationSnapshot")):
            raise ValueError("Worktree resume cannot replace its original writer authority")
        with writer_hold(self):
            # Resume changes authority metadata only; a failed read leaves no partial source mutation.
            self._worktree_mutation_started = False
            if (plain(self.text(path, "rev-parse", "--path-format=absolute", "--git-common-dir")) != plain(self.repo / ".git")
                    or self.text(path, "rev-parse", "HEAD") != base
                    or self.text(path, "symbolic-ref", "--short", "HEAD") != branch
                    or self.source(path) != result.get("source")):
                raise ValueError("Worktree source or physical Git identity changed; preserve it for reconciliation")
            if lineage["kind"] == "managed-worktree-same-boot-resume":
                lineage = self.worktree_resume_lineage(value["runtimeOperationId"], original, reserved=True)
            record = {**value, **binding, "requestId": self.request, "operationId": self.operation,
                      "driverDigest": self.config["driverDigest"],
                      "resumedFrom": {"creationOperationKey": key, "runtimeOperationId": value["runtimeOperationId"],
                                      "driverDigest": value["driverDigest"],
                                      "recordDigest": sha(encoded(value)), "recovery": lineage},
                      "sourceUnchangedDuringResume": True}
            if lineage["kind"] == "managed-worktree-same-boot-resume":
                with self.state.transaction() as connection:
                    for pin in lineage["reviewedStatePins"]:
                        current = self.state.get(pin["domain"], pin["key"], connection=connection)
                        if (current is None or current["version"] != pin["version"]
                                or sha(encoded(current["payload"])) != pin["payloadDigest"]):
                            raise ValueError("Same-boot WT resume State authority changed before publication")
                    self.state.put("management_worktree", reference, record,
                                   expected_version=existing["version"], connection=connection)
            else:
                self.state.put("management_worktree", reference, record, expected_version=existing["version"])
        return reference, path, record

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

    def validation_delivery_inputs(self, original, result, captured):
        """Keep the original execution input while observing the live owner."""
        self.verify_declared_inputs(captured, original)
        if result.get("validationInputPolicy") == "sealed-source":
            root, _current, observation = self.validation_sealed_observation(captured, payload=original)
            if (observation.get("originalEngineUnchanged") is not True
                    or observation.get("dependencyCacheUnchanged") is not True):
                raise ValueError("Validated engine or dependency cache changed before delivery")
            return root, observation
        root, current = self.validation_with_cache(payload=original)
        if current != captured:
            raise ValueError("Validated candidate or dependency closure changed before delivery")
        return root, {"complete": True, "inputsUnchanged": True,
                      "originalEngineUnchanged": True, "dependencyCacheUnchanged": True}

    def validation_batch_delivery(self, key, operation, original, result, before, report, request):
        """Consume every completed step without claiming, persisting or launching."""
        from tools.jenkins.resources.paths import canonical_build_root
        owner = "management-" + key
        checkpoint = operation.get("batchCheckpoint", {})
        digest = operation.get("inputDigest")
        coverage = self.validation_batch(payload=original)["coverage"]
        count = len(original["batch"]["recipes"])
        launch_row, execution_row = self.state.get("execution_launch", owner), self.state.get("execution", owner)
        launch = launch_row["payload"] if launch_row else {}
        execution = execution_row["payload"] if execution_row else {}
        claims = launch.get("stepClaims", [])
        binding = {"schemaVersion": 1, "operationKey": key, "inputDigest": digest, "owner": owner}
        if (not isinstance(digest, str) or not re.fullmatch(r"[0-9a-f]{64}", digest)
                or digest != sha(encoded({k: v for k, v in original.items() if k != "observationOnly"}))
                or operation.get("requestId") != original.get("requestId")
                or operation.get("operationId") != original.get("operationId")
                or any(checkpoint.get(k) != v for k, v in binding.items())
                or checkpoint.get("stage") != "complete" or checkpoint.get("allRecipesExecuted") is not True
                or checkpoint.get("completedStepCount") != count or checkpoint.get("request") != result["request"]
                or checkpoint.get("capturedSourceDigest") != before["sourceDigest"]
                or checkpoint.get("sourceRoot") != before["sourceRoot"]
                or checkpoint.get("executionRoot") != request["executionRoot"]
                or checkpoint.get("coverageDigest") != sha(encoded(coverage))
                or request.get("inputDigest") != digest or request.get("coverage") != coverage
                or result.get("coverage") != coverage
                or result.get("recipeCount") != count or result.get("completedRecipeCount") != count
                or not isinstance(request.get("commands"), list) or len(request["commands"]) != count
                or request.get("command") != request["commands"][0]
                or not isinstance(request.get("stepEnvironmentDigests"), list)
                or len(request["stepEnvironmentDigests"]) != count
                or launch.get("batch") is not True or launch.get("status") != "terminal"
                or launch.get("allRecipesExecuted") is not True
                or launch.get("operationKey") != key or launch.get("inputDigest") != digest
                or launch.get("requestId") != operation.get("requestId")
                or launch.get("operationId") != operation.get("operationId")
                or launch.get("sessionId") != self.session
                or not isinstance(claims, list) or len(claims) != count
                or any(not isinstance(claim, dict) or claim.get("ordinal") != ordinal
                       or claim.get("status") != "complete" for ordinal, claim in enumerate(claims, 1))
                or execution.get("batch") is not True or execution.get("status") != "passed"
                or execution.get("sessionId") != self.session
                or execution.get("batchReceipt") != checkpoint.get("receipt")
                or execution.get("executionIdentity") != request.get("executionIdentity")
                or result.get("executionIdentity") != request.get("executionIdentity")):
            raise ValueError("Batch delivery lacks its complete original parent and ordered step coverage")
        receipt = read_reference(checkpoint.get("receipt"), expected_path=report / "receipt.json")
        envelope = {"operationKey": key, "requestId": operation["requestId"], "operationId": operation["operationId"],
                    "sessionId": self.session, "driverDigest": original["driverDigest"],
                    "runtimeOperationId": original["runtimeOperationId"]}
        if result != {**receipt, **envelope}:
            raise ValueError("Completed batch result differs from its original immutable receipt")
        sealed = read_reference(checkpoint.get("sealed"), expected_path=report / "sealed-input.json",
                                max_bytes=SOURCE_EVIDENCE_MAX_BYTES)
        prepared = read_reference(checkpoint.get("prepared"), expected_path=report / "preparation.json")
        final = read_reference(checkpoint.get("finalObservation"), expected_path=report / "input-final.json",
                               max_bytes=SOURCE_EVIDENCE_MAX_BYTES)
        prepared_input = request["preparedSource"]
        if (sealed.get("capturedInputVerified") is not True
                or sealed.get("sourceRoot") != before["sourceRoot"]
                or sealed.get("capturedSourceDigest") != before["sourceDigest"]
                or sealed.get("coverageDigest") != sha(encoded(coverage))
                or sealed.get("sealedInputRef") != sha(encoded(sealed["sourceManifest"]))
                or prepared != request["preparation"]
                or prepared.get("physicalPath") != request["executionRoot"]
                or prepared.get("sealedInputRef") != sealed["sealedInputRef"]
                or prepared.get("sourceDigest") != sealed["sealedInputRef"]
                or prepared.get("capturedSourceDigest") != before["sourceDigest"]
                or prepared_input.get("sourceDigest") != sealed["sealedInputRef"]
                or final.get("prepared") != prepared_input
                or final.get("observation", {}).get("originalEngineUnchanged") is not True
                or final.get("observation", {}).get("dependencyCacheUnchanged") is not True
                or result.get("inputObservations") != {"afterPreparation": request["inputObservation"],
                                                       "afterExecution": final["observation"]}
                or result.get("inputsUnchanged") is not (before == final.get("after"))):
            raise ValueError("Batch delivery differs from its original shared sealed input proofs")
        reuse = original["batch"].get("reuseValidation")
        if reuse is not None:
            provenance = request.get("sourceProvenance", {})
            if (provenance != checkpoint.get("sourceProvenance")
                    or provenance.get("kind") != "validated-sealed-input"
                    or any(provenance.get(name) != value for name, value in reuse.items())
                    or provenance.get("capturedSourceDigest") != before["sourceDigest"]
                    or request["inputObservation"].get("externalInputsObserved") is not False
                    or final["observation"].get("externalInputsObserved") is not False
                    or request["inputObservation"].get("inputsUnchanged") is not False
                    or final["observation"].get("inputsUnchanged") is not False):
                raise ValueError("Batch delivery lacks its exact explicit sealed donor provenance")
            donor_report = report.parent / reuse["operationKey"]
            donor_request = read_reference(provenance["requestRef"], expected_path=donor_report / "request.json",
                                           max_bytes=SOURCE_EVIDENCE_MAX_BYTES)
            donor_receipt = read_reference(reuse["receiptRef"], expected_path=donor_report / "receipt.json")
            donor_reference = provenance.get("sealedInputRef")
            donor_capture = self.state.get("management_source", reuse["operationKey"])
            donor_row, current_row = (self.state.get("sealed_input", reference)
                                      for reference in (donor_reference, sealed["sealedInputRef"]))
            donor, current = (row["payload"] if row else {} for row in (donor_row, current_row))
            donor_manifest = donor.get("sourceManifest", {})
            manifest = sealed["sourceManifest"]
            approved = canonical_build_root(self.build_root(payload=original))
            if (not donor_capture or donor_capture["payload"] != before
                    or sha(encoded({k: v for k, v in before.items() if k != "sourceDigest"})) != before["sourceDigest"]
                    or donor_request.get("source") != before or donor_receipt.get("sourceDigest") != before["sourceDigest"]
                    or donor_receipt.get("request") != provenance["requestRef"]
                    or donor_request.get("preparation") != donor_receipt.get("preparation")
                    or donor_receipt.get("preparation", {}).get("capturedSourceDigest") != before["sourceDigest"]
                    or donor_receipt.get("preparation", {}).get("sealedInputRef") != donor_reference
                    or any(donor_request.get("payload", {}).get(name) != original.get(name)
                           for name in ("repositoryId", "sessionId", "worktreeId", "driverDigest", "runtimeOperationId"))
                    or any(value.get("status") != "sealed" or value.get("repositoryId") != self.repo_id
                           or value.get("buildRoot") != str(approved.path)
                           or value.get("objectRoot") != str(approved.namespace())
                           or value.get("manifest") != value.get("sourceManifest")
                           for value in (donor, current))
                    or donor.get("repositoryRoot") != current.get("repositoryRoot")
                    or donor.get("sourceDigest") != donor_reference or sha(encoded(donor_manifest)) != donor_reference
                    or donor.get("coverageDigest") != sha(encoded(donor.get("coverage")))
                    or donor_manifest.get("validationCoverageDigest") != donor.get("coverageDigest")
                    or current.get("sourceDigest") != sealed["sealedInputRef"]
                    or current.get("sourceManifest") != manifest or current.get("coverage") != coverage
                    or current.get("coverageDigest") != sha(encoded(coverage))
                    or manifest.get("validationCoverageDigest") != current.get("coverageDigest")
                    or {k: v for k, v in donor_manifest.items() if k != "validationCoverageDigest"}
                       != {k: v for k, v in manifest.items() if k != "validationCoverageDigest"}):
                raise ValueError("Donor and batch seals lack their independent coverage and identical complete captured CAS")
            preparation = source_helper()
            preparation.verify_sealed_capture(before["source"], manifest)
            if len(manifest.get("external", [])) != len(before["externalRoots"]):
                raise ValueError("Batch seal differs from the complete captured external closure")
            for captured_external, external in zip(before["externalRoots"], manifest.get("external", [])):
                if external.get("repositoryRoot") != captured_external["path"]:
                    raise ValueError("Batch seal differs from its captured external root")
                preparation.verify_sealed_capture(captured_external["source"], external["manifest"])
        context = {"key": key, "payload": original, "inputDigest": digest, "requestId": operation["requestId"],
                   "operationId": operation["operationId"], "report": report, "checkpoint": checkpoint}
        proofs = [self.batch_step_proof(claim, context=context) for claim in claims]
        steps = [proof[0] for proof in proofs]
        native_refs, proof_ids = [proof[2] for proof in proofs], [proof[3] for proof in proofs]
        if (any(step.get("status") != "passed" for step in steps) or steps != result.get("steps")
                or proof_ids != result.get("nativeProofRefs") or native_refs[-1] != result["nativeTerminal"]
                or launch.get("nativeProofRef") != proof_ids[-1]
                or execution.get("nativeProofRef") != proof_ids[-1]
                or execution.get("nativeTerminal") != native_refs[-1]):
            raise ValueError("Every original batch Native and test result must be complete and successful")
        return {"request": result["request"], "nativeTerminal": native_refs[-1],
                "nativeProofRef": proof_ids[-1], "nativeTerminals": native_refs, "nativeProofRefs": proof_ids,
                "stepReceipts": [claim["receipt"] for claim in claims], "batchReceipt": checkpoint["receipt"],
                "sealedInputRef": sealed["sealedInputRef"], "finalObservation": checkpoint["finalObservation"],
                "coverage": coverage, "sourceDigest": before["sourceDigest"]}

    def validation_delivery(self, key, worktree_id):
        """Read a completed single or shared-batch delivery and all original proofs."""
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
                or result.get("validationInputPolicy") == "actual-checkout"
                or original.get("driverDigest") != self.config["driverDigest"]
                or original.get("runtimeOperationId") != self.config["runtimeOperationId"]
                or not captured or result.get("sourceDigest") != captured["payload"]["sourceDigest"]
                or not validation_inputs_verified(result)):
            raise ValueError("Delivery requires an actual matching successful managed validation")
        before = captured["payload"]
        from tools.jenkins.resources.paths import canonical_build_root, physical_path_under
        approved = canonical_build_root(self.build_root(payload=original))
        report = physical_path_under(approved, approved.namespace() / "management/reports" / key)
        request = read_reference(result["request"], expected_path=report / "request.json",
                                 max_bytes=SOURCE_EVIDENCE_MAX_BYTES)
        execution_root = physical_path_under(approved, request["executionRoot"], allow_missing=False)
        if (request["payload"] != original or request["source"] != before
                or result.get("executionRoot") != str(execution_root)
                or request.get("preparation") != result.get("preparation")
                or not validation_inputs_verified(result, request)
                or ("batch" in original) != (result.get("batch") is True)):
            raise ValueError("Validation differs from its original captured input and execution identity")
        if "batch" in original:
            delivery = self.validation_batch_delivery(key, operation, original, result, before, report, request)
        else:
            native = read_reference(result["nativeTerminal"], expected_path=report / "native-terminal.json")
            expected = native_terminal_proof(native, result["nativeTerminal"])
            proof_id = "management-" + key + "-native"
            native_row = self.state.get("native_job", proof_id)
            proof = native_row["payload"] if native_row else {}
            if (request["command"] != native["command"] or plain(native["cwd"]) != execution_root
                    or native.get("rootExitCode") != 0 or expected["complete"] is not True
                    or proof.get("owner") != "management-" + key or proof.get("status") != "terminal"
                    or proof.get("nativeEvidence") != native or proof.get("completeProof") != expected
                    or proof.get("terminalProof") != expected):
                raise ValueError("Validation lacks its original complete retained-Job termination proof")
            delivery = {"request": result["request"], "nativeTerminal": result["nativeTerminal"],
                        "nativeProofRef": proof_id, "sourceDigest": before["sourceDigest"]}
        root, observation = self.validation_delivery_inputs(original, result, before)
        delivery["deliveryObservation"] = observation
        return row, root, before, delivery

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

    def assert_no_current_source_readers(self, reference, root, *, connection):
        if self.config.get("managedWorktree") == MANAGED_WORKTREE_CAPABILITY:
            from tools.jenkins.managed_worktree_lifetime import active_readers
            if active_readers(self.state, reference, root, connection=connection):
                raise ValueError("Managed worktree has a retained current-source reader; release requires separate reconciliation")
            return
        for row in self.state.list("current_source_intent", connection=connection):
            intent = row["payload"]
            source = intent.get("sourceRoot")
            if (intent.get("worktreeId") == reference
                    or isinstance(source, str) and Path(source).resolve() == root.resolve()):
                raise ValueError("Managed worktree has a retained current-source reader; release requires separate reconciliation")

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
        helper_path = client_path(self.config, self.repo, "tools/dev/local_cargo.py")
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
            self.assert_no_current_source_readers(reference, root, connection=connection)
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

    def recover_worktree(self):
        """Close out a worktree whose owning operation died mid-flight.

        A crashed process leaves either ``phase=running`` on the operation or
        ``status=releasing`` on the worktree, and neither state can ever
        advance because the process that would advance it is gone.  Recovery
        is deliberately narrow: it never finishes the interrupted work, only
        re-adjudicates the record against what is actually on disk, and it
        requires proof that the original operation is no longer alive.
        """
        reference = self.payload["worktreeId"]
        row = self.state.get("management_worktree", reference)
        if not row:
            raise ValueError("Recovery requires a registered managed worktree")
        value = row["payload"]
        if value.get("sessionId") != self.session:
            raise ValueError("Worktree recovery belongs to another session")
        path = plain(value["path"])
        if path.parent != self.worktrees:
            raise ValueError("Worktree escapes the managed directory")
        operation_key = self.payload["operationKey"]
        if not isinstance(operation_key, str) or not re.fullmatch(r"[0-9a-f]{64}", operation_key):
            raise ValueError("Recovery requires the exact interrupted operation key")
        operation = self.state.get("management_operation", operation_key)
        if not operation:
            raise ValueError("Interrupted management operation is unavailable")
        operation_value = operation["payload"]
        # create-worktree names its target with "name"; the other actions use
        # "worktreeId".  Comparing both spellings keeps one check honest.
        requested = operation_value.get("payload", {})
        owned_targets = {value for value in
                         (requested.get("worktreeId"), requested.get("name"),
                          reference.split("-", 1)[-1] if reference.startswith(self.session + "-") else None)
                         if value}
        if operation_value.get("sessionId") != self.session or reference not in owned_targets:
            raise ValueError("Recovery operation does not own this worktree")
        self.scoped(value.get("ownedPaths", self.owned), "gc")
        # The interrupted operation must be provably dead before its record is
        # re-adjudicated; a live owner still holds the only correct outcome.
        from tools.jenkins.state.locks import process_lock
        lock = self.repo / ".jenkins/state/locks" / ("management-operation-" + operation_key + ".lock")
        with process_lock(lock, timeout=0):
            pass
        # An owned Git step records its execution under "management-" + key.
        owner = "management-" + operation_key
        execution = self.state.get("execution", owner)
        if execution and execution["payload"].get("status") == "running":
            natives = [item for item in self.state.list("native_job")
                       if item["payload"].get("owner") == owner]
            if not any(item["payload"].get("status") == "terminal" for item in natives):
                raise ValueError("Interrupted operation still has a running native process; stop it first")
        present = path.is_dir()
        git_identity = None
        if present:
            try:
                git_identity = plain(self.text(path, "rev-parse", "--path-format=absolute", "--git-common-dir"))
            except ValueError:
                git_identity = None
        if value.get("status") == "releasing":
            if path.exists():
                raise ValueError("Release already removed the worktree; finish the original release instead")
            resolution = "released"
        elif not present:
            resolution = "absent"
        elif git_identity == plain(self.repo / ".git"):
            resolution = "ready"
        else:
            raise ValueError("A non-Git directory occupies the managed worktree path; remove it deliberately")
        result = {"status": "recovered", "worktreeId": reference, "path": str(path),
                  "previousWorktreeStatus": value.get("status"), "previousOperationPhase": operation_value.get("phase"),
                  "resolution": resolution, "directoryPresent": present,
                  "commitPerformed": False, "pushPerformed": False, "formalAcceptance": False}
        with self.state.transaction() as connection:
            record = {**value, "status": resolution, "recoveredFrom": value.get("status"),
                      "recoveryOperationKey": self.key}
            record.pop("releaseRequest", None)
            self.state.put("management_worktree", reference, record,
                           expected_version=row["version"], connection=connection)
            finished = {**operation_value, "phase": "unknown", "recoveryResolution": resolution,
                        "finishedAt": time.time()}
            self.state.put("management_operation", operation_key, finished,
                           expected_version=operation["version"], connection=connection)
        return result

    def prepare_flow(self):
        """Register this managed worktree as the Flow chain's current source.

        Management could create a worktree and Flow could consume a registered
        current source, but nothing turned one into the other, so the two chains
        never met.  This bridges them: the worktree supplies the bytes, the
        canonical checkout supplies the identity, and the result is the payload
        ``submit-flow`` already accepts, so no Flow-side contract changes.
        """
        record, root = self.wt(self.payload["worktreeId"])
        # Import the module, not the function: a bound name cannot be replaced
        # by a caller that needs to observe the registration it makes.
        from tools.jenkins import current_source
        from tools.jenkins.source import repository_id
        identity_value = self.payload["flowIdentity"]
        if not isinstance(identity_value, dict):
            raise ValueError("A flow identity is required")
        identity = {**identity_value, "repositoryId": repository_id(self.repo)}
        if (not isinstance(identity.get("sessionId"), str) or identity["sessionId"] == self.session
                or identity.get("generation") not in (1, "1")):
            raise ValueError("Flow identity requires a separate validation reader at generation 1")
        identifier(identity["sessionId"])
        captured = self.source(root)
        # Flow refuses checkout control and compiler output as product input,
        # and this repository tracks files under .jenkins and .codex, so an
        # unfiltered closure always trips that guard.  Only the product source
        # crosses the bridge.
        control = {".git", ".codex", ".jenkins", ".opencode", "target"}
        prepared_ref = self.payload.get("worktreePreparationRef")
        prepared_fields = {}
        if prepared_ref is not None:
            from tools.jenkins.managed_worktree import preparation
            from tools.jenkins.product_inputs import product_path
            prepared = preparation(self.state, prepared_ref, self.payload["worktreeId"])["payload"]
            hashes = dict(prepared["currentHashes"])
            hashes.update({name.casefold(): checksum for name, checksum in captured["files"].items()
                           if product_path(name.casefold())})
            prepared_fields = {"worktreePreparationRef": prepared_ref,
                               "compilerControlInputs": prepared["compilerControlInputs"],
                               "patchOperationKeys": self.payload.get("patchOperationKeys", [])}
            external_inputs = prepared["externalInputs"]
        else:
            hashes = {name: checksum for name, checksum in captured["files"].items()
                      if (checksum is None or isinstance(checksum, str))
                      and name.split("/", 1)[0] not in control}
            external_inputs = self.payload.get("externalInputs", [])
        owned = [name for name in self.owned if name in hashes]
        if not owned:
            raise ValueError("Flow preparation needs at least one owned product source path; "
                             "control paths are not product input")
        if not any(checksum is not None for checksum in hashes.values()):
            raise ValueError("Flow preparation requires at least one present source file")
        tracked = set()
        for raw in self.git(root, "ls-files", "--cached", "-z").stdout.split(b"\0"):
            if raw:
                name = raw.decode("utf-8", "strict")
                tracked.add(name.casefold() if prepared_ref is not None else name)
        untracked = sorted(name for name, checksum in hashes.items()
                           if checksum is not None and name not in tracked)
        authored = self.payload.get("attemptId") or ("attempt-" + self.operation)
        coverage = self.payload.get("coverage")
        if not isinstance(coverage, dict):
            raise ValueError("Flow preparation requires an explicit product coverage template")
        registration = {
            "identity": {**identity, "attemptId": authored, "generation": 1,
                         "stageImplementationDigest": self.config["driverDigest"]},
            "baseHead": captured["head"],
            "currentHashes": hashes,
            "ownedPaths": owned,
            "untrackedPaths": untracked,
            "coverage": coverage,
            "buildRoot": self.build_root(),
            "sourceRoot": str(root), "worktreeId": self.payload["worktreeId"],
            "runtimeOperationId": self.config["runtimeOperationId"],
            "externalInputs": external_inputs,
            **prepared_fields,
        }
        # Only a completed prepared WT can retain unchanged owned absences.
        absent_owned = prepared_ref is not None and any(
            hashes[name] is None and name not in tracked for name in owned)
        registered = (source_helper().register_prepared_worktree_flow(self, registration) if absent_owned
                      else current_source.handle("register", registration, self.state, self.repo))
        if registered.get("status") != "registered":
            raise ValueError("Flow preparation did not register a current source: "
                             + str(registered.get("reasonCode")))
        reference = registered["currentSourceRef"]
        return {"status": "prepared", "worktreeId": self.payload["worktreeId"],
                "sourceRoot": str(root), "repositoryRoot": str(self.repo),
                "repositoryId": identity["repositoryId"], "currentSourceRef": reference,
                "baseHead": captured["head"], "sourceDigest": captured["sourceDigest"],
                "flowPayload": {"identity": {**identity, "attemptId": authored, "generation": 1,
                                             "stageImplementationDigest": self.config["driverDigest"]},
                                "currentSourceRef": reference,
                                "buildRoot": self.build_root(),
                                "coverage": self.payload.get("coverage"),
                                "authorization": self.payload.get("authorization"),
                                "submissionRequestId": self.payload.get("submissionRequestId")},
                "commitPerformed": False, "pushPerformed": False, "formalAcceptance": False}

    def commit_worktree(self):
        record, root = self.wt(self.payload["worktreeId"])
        expected = self.payload["expectedHead"]
        validation_key = self.payload["validationKey"]
        if not isinstance(validation_key, str) or not re.fullmatch(r"[0-9a-f]{64}", validation_key):
            raise ValueError("An exact management validation operation key is required")
        validation, current_root, before, delivery = self.validation_delivery(validation_key, self.payload["worktreeId"])
        value = validation["payload"]
        original, result = value["payload"], value["result"]
        engine = before["source"]
        if current_root != root or engine["head"] != expected:
            raise ValueError("Validated worktree or HEAD changed before commit")
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
        # "xb" made a retry fail on its own earlier write.  The message is a
        # function of this operation key, so an identical file is the same
        # intent repeated and is reused; different bytes are a real conflict.
        payload = message.encode("utf-8") + b"\n"
        if message_path.exists():
            if message_path.read_bytes() != payload:
                raise WorktreeConflict(
                    "Commit message for this operation already differs",
                    reason="commit_message_conflict",
                    details={"path": str(message_path)})
        else:
            with message_path.open("xb") as stream:
                stream.write(payload)
        main_head = self.text(self.repo, "rev-parse", "HEAD")
        self.git(root, "add", "--", *sorted(changes))
        staged_source = self.source(root)
        if staged_source["files"] != engine["files"] or staged_source["head"] != expected:
            raise ValueError("Validated worktree file bytes or HEAD changed before commit")
        _same_root, commit_observation = self.validation_delivery_inputs(original, result, before)
        if _same_root != root:
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
                "validationDelivery": delivery, "commitObservation": commit_observation,
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


    def validation_batch(self, *, payload=None):
        payload = self.payload if payload is None else payload
        if "checkoutProduct" in payload:
            product = payload.get("checkoutProduct")
            schema = product.get("schemaVersion") if isinstance(product, dict) else None
            return checkout_product_helper(self.config, required_schema=schema).validation_batch(payload, self.repo)
        batch = payload.get("batch")
        fields = {"verb", "packages", "features", "profile", "offline", "noDefaultFeatures", "lib",
                  "tests", "testFilter", "noRun", "allFeatures", "locked", "jobs", "testThreads",
                  "bins", "allTests", "harnessFilters", "exact", "ignored", "nocapture", "manifestPath"}
        if (not isinstance(batch, dict)
                or set(batch) - {"schemaVersion", "sharedSingleBatchRequired", "contractRef", "recipes", "stepEnvironments", "reuseValidation"}
                or batch.get("schemaVersion") != 1 or batch.get("sharedSingleBatchRequired") is not True
                or fields & set(payload)):
            raise ValueError("An optional batch requires one shared input and no top-level single recipe")
        reuse = batch.get("reuseValidation")
        if "reuseValidation" in batch:
            reference = reuse.get("receiptRef") if isinstance(reuse, dict) else None
            if (not isinstance(reuse, dict) or set(reuse) != {"operationKey", "capturedSourceDigest", "receiptRef"}
                    or any(not isinstance(reuse.get(name), str)
                           or not re.fullmatch(r"[0-9a-f]{64}", reuse[name])
                           for name in ("operationKey", "capturedSourceDigest"))
                    or not isinstance(reference, dict) or set(reference) != {"path", "sha256"}
                    or not isinstance(reference.get("path"), str) or not reference["path"]
                    or len(reference["path"].encode("utf-8")) > 4096
                    or not Path(reference["path"]).is_absolute()
                    or any(c in reference["path"] for c in "\0\r\n")
                    or not isinstance(reference.get("sha256"), str)
                    or not re.fullmatch(r"[0-9a-f]{64}", reference["sha256"])
                    or payload.get("sourceView", "worktree") != "worktree" or not payload.get("worktreeId")
                    or payload.get("expectedSourceDigest") != reuse["capturedSourceDigest"]):
                raise ValueError("Sealed batch reuse requires exact donor, capture, receipt and explicit source pins")
        contract_document = read_reference(batch.get("contractRef"))
        contract = contract_document.get("originalCommandContract", contract_document)
        originals = contract.get("semanticCommands")
        commands = batch.get("recipes")
        if (contract.get("sharedSingleBatchRequired") is not True
                or not isinstance(commands, list) or not 1 <= len(commands) <= 64
                or commands != originals or len(encoded(batch)) > 900 * 1024):
            raise ValueError("Batch must preserve every complete original semantic command")
        parser = getattr(self, "_batch_parser", None)
        if parser is None:
            path = client_path(self.config, self.repo, "tools/jenkins-management/entry.py")
            if sha(path.read_bytes()) != self.config["clientSources"]["tools/jenkins-management/entry.py"]:
                raise ValueError("Pinned Cargo contract parser changed")
            spec = importlib.util.spec_from_file_location("pinned_batch_cargo_contract", path)
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
            parser = module.parse_cargo
        common = {k: v for k, v in payload.items() if k != "batch"}
        recipes, names = [], set()
        for ordinal, original in enumerate(commands, 1):
            if (not isinstance(original, dict) or original.get("ordinal") != ordinal
                    or not isinstance(original.get("name"), str) or not ID.fullmatch(original["name"])
                    or original["name"] in names
                    or not isinstance(original.get("cargoArguments"), list)
                    or not original["cargoArguments"] or len(original["cargoArguments"]) > 1536
                    or any(not isinstance(v, str) or not v or any(c in v for c in "\0\r\n")
                           for v in original["cargoArguments"])
                    or type(original.get("ownedNativeTimeoutSeconds")) is not int
                    or not 1 <= original["ownedNativeTimeoutSeconds"] <= 21600):
                raise ValueError("Each original batch recipe needs its ordinal, name, complete argv and native deadline")
            names.add(original["name"])
            parsed = parser(original["cargoArguments"])
            probe = object.__new__(Management)
            probe.payload = {**common, **parsed}
            recipe = probe.validation_recipe()
            profile = original.get("profile")
            if profile is not None and ({"debug": "dev", "dev": "dev", "release": "release", "test": "test"}.get(profile)
                                        != recipe["profile"]):
                raise ValueError("Original profile differs from its Cargo argv")
            requested = original.get("targetDir")
            if requested is not None:
                if ("requestedTargetDir" in common and plain(requested) != plain(common["requestedTargetDir"])
                        or self.build_root(payload={**common, "requestedTargetDir": requested})
                           != self.build_root(payload=common)):
                    raise ValueError("Every shared recipe must retain the same explicit physical target volume")
            floor = original.get("minimumExpectedMatchedTests", 1)
            if type(floor) is not int or not 1 <= floor <= 1000000:
                raise ValueError("Executed test floors must be positive bounded counts")
            for field, value in original.items():
                if (field.startswith("requiredNamed") and field.endswith("Tests")
                        or field in {"requiredTestBinaries", "requiredProducerMarkers"}):
                    if (not isinstance(value, list) or len(value) > 4096
                            or any(not isinstance(v, str) or not v or len(v.encode("utf-8")) > 2048
                                   or "\0" in v for v in value)):
                        raise ValueError("Required test, binary and producer evidence must be bounded lists")
            recipes.append(recipe)
        environments = batch.get("stepEnvironments", [{} for _ in recipes])
        allowed_environment = {"ZR_RUNTIME03_STORE_MATRIX_OUTPUT", "ZR_RUNTIME03_STORE_MATRIX_CONTEXT_JSON",
                               "ZIRCON_RUNTIME03_CONFIG_MATRIX_DIR"}
        if (not isinstance(environments, list) or len(environments) != len(recipes)
                or any(not isinstance(v, dict) or set(v) - allowed_environment
                       or any(not isinstance(item, str) or not item or "\0" in item
                              or len(item.encode("utf-8")) > 16384 for item in v.values()) for v in environments)):
            raise ValueError("Batch context environment must preserve bounded explicit producer inputs")
        for values in environments:
            for name in ("ZR_RUNTIME03_STORE_MATRIX_OUTPUT", "ZIRCON_RUNTIME03_CONFIG_MATRIX_DIR"):
                if name in values:
                    from tools.jenkins.resources.paths import canonical_build_root, physical_path_under
                    approved = canonical_build_root(self.build_root(payload=common))
                    physical_path_under(approved, values[name])
        producer_contract = {k: v for k, v in contract.items() if k != "semanticCommands"}
        if ("originalCommands" in contract and contract["originalCommands"] != len(commands)
                or "totalProducerIdentities" in contract and
                   contract["totalProducerIdentities"] != contract.get("preservedOriginalProducers", 0)
                                                       + contract.get("previousRuntime03MatrixProducers", 0)):
            raise ValueError("Original command/producer identity counts changed")
        return {"recipes": recipes, "coverage": {"schemaVersion": 1, "sharedSingleBatchRequired": True,
                "contractRef": batch["contractRef"], "recipes": commands, "producerContract": producer_contract},
                "stepEnvironments": environments, "reuseValidation": reuse}

    def batch_reused_input(self, reuse, expected_root):
        """Read a proven donor; its tests never replace this batch's executions."""
        if reuse["operationKey"] == self.key:
            raise ValueError("A new batch cannot consume itself as a sealed donor")
        validation, root, before, delivery = self.validation_delivery(reuse["operationKey"], self.payload["worktreeId"])
        operation = validation["payload"]
        original, result = operation["payload"], operation["result"]
        from tools.jenkins.resources.paths import canonical_build_root
        if (result.get("validationInputPolicy") != "sealed-source" or root != expected_root
                or before["sourceDigest"] != reuse["capturedSourceDigest"]
                or sha(encoded({k: v for k, v in before.items() if k != "sourceDigest"})) != before["sourceDigest"]
                or canonical_build_root(self.build_root(payload=original)).path
                   != canonical_build_root(self.build_root()).path):
            raise ValueError("Sealed donor differs from the requested complete capture/worktree/output root")
        receipt = read_reference(reuse["receiptRef"], expected_path=plain(delivery["request"]["path"]).parent / "receipt.json")
        envelope = {"operationKey": reuse["operationKey"], "requestId": operation["requestId"],
            "operationId": operation["operationId"], "sessionId": self.session,
            "driverDigest": self.config["driverDigest"], "runtimeOperationId": self.config["runtimeOperationId"]}
        if result != {**receipt, **envelope}:
            raise ValueError("Donor receipt differs from its actual completed validation result")
        self.verify_declared_inputs(before, self.payload)
        provenance = {"kind": "validated-sealed-input", **reuse, "requestRef": delivery["request"],
            "sealedInputRef": result["preparation"]["sealedInputRef"],
            "nativeTerminals": delivery.get("nativeTerminals", [delivery["nativeTerminal"]]),
            "nativeProofRefs": delivery.get("nativeProofRefs", [delivery["nativeProofRef"]])}
        return before, provenance

    def batch_checkpoint(self, **changes):
        row = self.state.get("management_operation", self.key)
        if not row or row["payload"].get("inputDigest") != self.input_digest:
            raise ValueError("Batch checkpoint lacks its original parent operation")
        checkpoint = row["payload"].get("batchCheckpoint", {})
        binding = {"schemaVersion": 1, "operationKey": self.key, "inputDigest": self.input_digest,
                   "owner": "management-" + self.key}
        if checkpoint and any(checkpoint.get(k) != v for k, v in binding.items()):
            raise ValueError("Batch checkpoint identity changed")
        checkpoint = {**binding, **checkpoint, **changes}
        if changes:
            self.state.put("management_operation", self.key, {**row["payload"], "batchCheckpoint": checkpoint},
                           expected_version=row["version"])
        return checkpoint

    def batch_pool(self, recipes, preparation, trusted, environment, root, cache):
        cargo_configuration = {}
        for name in (".cargo/config", ".cargo/config.toml"):
            path = plain(root / name)
            if path.is_file():
                cargo_configuration[name] = sha(path.read_bytes())
        keys = set()
        for recipe in recipes:
            compatibility = {**environment, **cargo_manifest_environment(root, recipe),
                **({"cargoAllFeatures": True} if recipe["allFeatures"] else {}),
                "sourceCargoConfiguration": cargo_configuration}
            keys.add(preparation.preparation_key(self.repo_id, trusted, compatibility,
                recipe["packages"], recipe["features"], recipe["profile"], recipe["noDefaultFeatures"],
                cache, ["zr_vm"] if self.payload.get("externalRoots") else []))
        configurations = sorted(keys)
        pool = configurations[0] if len(configurations) == 1 else sha(encoded({"configurationSet": configurations}))
        if self.payload.get("checkoutProduct"):
            pool = sha(encoded({"configuration": pool, "checkoutProductOperation": self.key}))
        return pool, cargo_configuration

    def batch_claims(self):
        launch = self.state.get("execution_launch", "management-" + self.key)
        if not launch:
            return []
        value = launch["payload"]
        if (value.get("batch") is not True or value.get("operationKey") != self.key
                or value.get("requestId") != self.request or value.get("operationId") != self.operation
                or value.get("inputDigest") != self.input_digest):
            raise ValueError("Batch launch history differs from the original parent")
        claims = value.get("stepClaims", [])
        if (not isinstance(claims, list) or len(claims) > len(self.payload["batch"]["recipes"])
                or any(not isinstance(claim, dict) or claim.get("ordinal") != ordinal
                       or claim.get("status") not in {"claimed", "complete"}
                       for ordinal, claim in enumerate(claims, 1))):
            raise ValueError("Original batch launch prefix changed")
        return claims

    def batch_step_proof(self, claim, *, persist=False, context=None):
        """Reconcile real step proofs; an explicit delivery context is read-only."""
        if context is None:
            key, payload, digest = self.key, self.payload, self.input_digest
            request_id, operation_id = self.request, self.operation
            report_root, checkpoint = self.outputs() / "reports" / key, self.batch_checkpoint()
        else:
            if persist:
                raise ValueError("Delivery may only read original batch proofs")
            key, payload, digest = context["key"], context["payload"], context["inputDigest"]
            request_id, operation_id = context["requestId"], context["operationId"]
            report_root, checkpoint = context["report"], context["checkpoint"]
        ordinal = claim["ordinal"]
        owner = "management-" + key
        report = report_root / "steps" / ("%04d" % ordinal)
        request = read_reference(claim["request"], expected_path=report / "request.json")
        parent_request = read_reference(checkpoint["request"], expected_path=report.parent.parent / "request.json",
                                        max_bytes=SOURCE_EVIDENCE_MAX_BYTES)
        actual_checkout = parent_request.get("validationInputPolicy") == "actual-checkout"
        if actual_checkout:
            input_fields = ("checkoutInputsDigest", "expectedInputsDigest", "checkoutReadRef")
            if (checkpoint.get("validationInputPolicy") != "actual-checkout"
                    or payload.get("sourceView") != "checkout" or context is not None
                    or request.get("validationInputPolicy") != "actual-checkout"
                    or "preparation" in parent_request or "sealedInputRef" in request
                    or any(request.get(name) != parent_request.get(name) for name in input_fields)
                    or request.get("checkoutInputsDigest") != checkpoint.get("checkoutInputsDigest")
                    or request.get("checkoutReadRef") != checkpoint.get("checkoutReadRef")
                    or parent_request.get("sourceRoot") != parent_request.get("executionRoot")
                    or sha(encoded(parent_request["checkoutInputs"])) != parent_request["checkoutInputsDigest"]
                    or checkout_expected_inputs_digest(parent_request["checkoutInputs"]) != parent_request["expectedInputsDigest"]):
                raise ValueError("Actual checkout step lost its original continuous read binding")
            read_document = read_reference(request["checkoutReadRef"],
                expected_path=report_root / "checkout-read.json", max_bytes=SOURCE_EVIDENCE_MAX_BYTES)
            binding = read_document.get("binding", {})
            if (binding.get("owner") != owner or binding.get("operationKey") != key
                    or binding.get("inputDigest") != digest
                    or binding.get("sourceRoot") != parent_request["sourceRoot"]
                    or binding.get("sourceDigest") != parent_request["source"]["sourceDigest"]
                    or binding.get("reservationKey") != checkpoint.get("reservationKey")
                    or binding.get("preparationKey") != checkpoint.get("writer")
                    or any(binding.get(name) != parent_request[name] for name in input_fields[:2])
                    or read_document.get("guard", {}).get("expectedInputsDigest") != parent_request["expectedInputsDigest"]
                    or not checkout_trusted_native_verified(parent_request["checkoutInputs"],
                                                            read_document.get("guard", {}))):
                raise ValueError("Actual checkout step guard identity differs from the original reservation")
            input_binding = {"validationInputPolicy": "actual-checkout",
                             "sourceDigest": parent_request["source"]["sourceDigest"],
                             **{name: request[name] for name in input_fields}}
            result_binding = dict(input_binding)
        else:
            if (checkpoint.get("validationInputPolicy") == "actual-checkout"
                    or request.get("validationInputPolicy") == "actual-checkout"
                    or request.get("sealedInputRef") != parent_request["preparation"]["sealedInputRef"]):
                raise ValueError("A sealed checkpoint cannot be upgraded to actual checkout evidence")
            input_binding = {"sourceDigest": request["sealedInputRef"]}
            result_binding = {"sealedInputRef": request["sealedInputRef"]}
        if (parent_request.get("inputDigest") != digest
                or {k: v for k, v in parent_request["payload"].items() if k != "observationOnly"}
                   != {k: v for k, v in payload.items() if k != "observationOnly"}
                or request.get("command") != parent_request["commands"][ordinal - 1]
                or request.get("cwd") != parent_request["executionRoot"]
                or request.get("temporaryEnvironment") != parent_request["temporaryEnvironment"]
                or request.get("parentRequest") != checkpoint["request"]
                or request.get("inputDigest") != digest or request.get("ordinal") != ordinal
                or request.get("command") != claim["command"] or request.get("cwd") != claim["cwd"]
                or request.get("recipe") != payload["batch"]["recipes"][ordinal - 1]
                or request.get("recipe") != parent_request["coverage"]["recipes"][ordinal - 1]
                or request.get("timeoutSeconds") != request["recipe"]["ownedNativeTimeoutSeconds"]
                or request.get("environmentDigest") != parent_request["stepEnvironmentDigests"][ordinal - 1]
                or request.get("taskScratch") != parent_request["taskScratch"]):
            raise ValueError("Step request differs from its complete original shared input")
        native_path = plain(report / "native-terminal.json")
        raw = native_path.read_bytes()
        native_ref = {"path": str(native_path), "sha256": sha(raw)}
        native = read_reference(native_ref, expected_path=native_path)
        proof = native_terminal_proof(native, native_ref)
        if (proof["complete"] is not True or native["command"] != claim["command"]
                or native["cwd"] != claim["cwd"]):
            raise ValueError("Original step Native/Job/double-EOF proof is incomplete")
        started_path = plain(report / "started.json")
        started_raw = started_path.read_bytes()
        started_ref = {"path": str(started_path), "sha256": sha(started_raw)}
        started = read_reference(started_ref, expected_path=started_path)
        if any(started.get(k) != native[k] for k in ("command", "cwd", "pid", "creationTime")):
            raise ValueError("Original launched PID/birth differs from terminal evidence")
        proof_id = owner + "-step-%04d-native" % ordinal
        value = {"owner": owner, "executionId": owner, "role": "management_owned_cargo",
                 "batchStep": ordinal, "status": "terminal", "terminalProof": proof, "completeProof": proof,
                 "nativeEvidence": native, "pid": native["pid"], "creationTime": native["creationTime"],
                 "birthToken": native["creationTime"], "executable": proof["identity"]["executable"],
                 "commandDigest": proof["identity"]["commandDigest"], "requestId": request_id,
                 "operationId": operation_id, "taskScratch": request["taskScratch"],
                 "environmentDigest": request["environmentDigest"], "stepRequest": claim["request"],
                 **input_binding, "formalAcceptance": False}
        existing = self.state.get("native_job", proof_id)
        if existing and existing["payload"] != value:
            raise ValueError("Original per-step Native record changed")
        result = self.batch_step_evidence(request["recipe"], report, native, native_ref,
                                          target=report_root.parent.parent / "pools" / checkpoint["pool"] / "target")
        result.update(request=claim["request"], started=started_ref, nativeProofRef=proof_id,
                      inputDigest=digest, **result_binding,
                      parentRequest=checkpoint["request"], environmentDigest=request["environmentDigest"],
                      formalAcceptance=False, m8Accepted=False, performanceAccepted=False, productsAccepted=False)
        receipt_path = report / "receipt.json"
        if receipt_path.exists():
            receipt_ref = {"path": str(receipt_path), "sha256": sha(receipt_path.read_bytes())}
            if read_reference(receipt_ref, expected_path=receipt_path) != result:
                raise ValueError("Original step receipt or log evidence changed")
        elif persist:
            receipt_ref = write_json(receipt_path, result)
        else:
            receipt_ref = None
        if claim.get("status") == "complete":
            if (not existing or receipt_ref != claim.get("receipt") or native_ref != claim.get("nativeTerminal")
                    or claim.get("nativeProofRef") != proof_id):
                raise ValueError("Completed step checkpoint lacks its actual original evidence")
        if persist:
            if not existing:
                self.state.put("native_job", proof_id, value, expected_version=0)
            launch = self.state.get("execution_launch", owner)
            claims = launch["payload"]["stepClaims"]
            if len(claims) < ordinal or claims[ordinal - 1] != claim:
                raise ValueError("Original step claim changed during reconciliation")
            complete = {**claim, "status": "complete", "receipt": receipt_ref,
                        "nativeTerminal": native_ref, "nativeProofRef": proof_id}
            claims = [*claims[:ordinal - 1], complete, *claims[ordinal:]]
            if complete != claim:
                self.state.put("execution_launch", owner, {**launch["payload"], "stepClaims": claims,
                               "status": "between-steps"}, expected_version=launch["version"])
            self.batch_checkpoint(stage="running", completedStepCount=ordinal)
        return result, native, native_ref, proof_id

    def batch_recovery_ready(self):
        """Read-only admission for an explicit resume, including interrupted real steps."""
        try:
            self.validation_batch()
            checkpoint = self.batch_checkpoint()
            if self.payload.get("sourceView") == "checkout":
                read = self.state.get("management_checkout_read", "management-" + self.key)
                return (checkpoint.get("validationInputPolicy") == "actual-checkout"
                        and checkpoint.get("stage") == "complete" and bool(checkpoint.get("receipt"))
                        and bool(read) and read["payload"].get("status") == "released"
                        and read["payload"].get("pinsReleased") is True
                        and read["payload"].get("continuousReadProven") is True)
            if checkpoint.get("stage") == "complete":
                return bool(checkpoint.get("receipt"))
            for claim in self.batch_claims():
                self.batch_step_proof(claim)
            owner = "management-" + self.key
            execution = self.state.get("execution", owner)
            if (self.state.get("execution_host", owner)
                    or not self.batch_claims() and any(row["payload"].get("executionId") == owner
                        or row["payload"].get("owner") == owner for row in self.state.list("native_job"))):
                return False
            if execution and not self.batch_claims() and not checkpoint.get("prepared"):
                return (execution["payload"].get("status") == "preparing"
                        and not self.state.get("execution_host", "management-" + self.key))
            return True
        except (ValueError, OSError, KeyError, TypeError):
            return False


    def batch_step_evidence(self, original, report, native, native_ref, *, target=None):
        """Keep compiled, executed, ignored and failed evidence distinct."""
        logs, texts = {}, {}
        for name in ("stdout", "stderr"):
            path = plain(report / (name + ".bin"))
            raw = path.read_bytes()
            logs[name] = {"path": str(path), "sha256": sha(raw), "bytes": len(raw)}
            texts[name] = raw.decode("utf-8", errors="replace")
            if path.read_bytes() != raw:
                raise ValueError("Terminal step log changed during reconciliation")
        cases, pending, summaries, binaries, compiled, artifacts = [], [], [], set(), {}, []
        build_finished, fresh = False, 0
        target = plain(target if target is not None else self.outputs() / "pools" / self.batch_checkpoint()["pool"] / "target")
        for line in texts["stdout"].splitlines():
            try:
                event = json.loads(line)
            except ValueError:
                event = None
            if isinstance(event, dict):
                if event.get("reason") == "build-finished":
                    build_finished = event.get("success") is True
                if event.get("reason") == "compiler-artifact":
                    fresh += int(event.get("fresh") is True)
                    if event.get("executable"):
                        compiled[str(plain(event["executable"]))] = event.get("target", {}).get("name")
                    for name in event.get("filenames", []):
                        artifact = plain(name)
                        if not (artifact.is_relative_to(target) or artifact.is_relative_to(target.parent / "build")):
                            raise ValueError("Batch compiler artifact escaped its shared pool")
                        # Historical per-step evidence must survive later rebuilds
                        # of the same Cargo artifact. Its path is an artifact fact,
                        # not a mutable-byte input to checkpoint reconciliation.
                        artifacts.append(str(artifact))
                continue
            match = re.match(r"^test (\S+) \.\.\. (.*)$", line)
            if match:
                name, tail = match.groups()
                status = re.fullmatch(r"(ok|FAILED|ignored)(?:, .*)?", tail.strip())
                if status:
                    cases.append({"name": name, "status": status.group(1)})
                else:
                    pending.append(name)
            elif line.strip() in {"ok", "FAILED", "ignored"} and len(pending) == 1:
                cases.append({"name": pending.pop(), "status": line.strip()})
            summary = re.search(r"test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;", line)
            if summary:
                summaries.append({"status": summary.group(1), "passed": int(summary.group(2)),
                                  "failed": int(summary.group(3)), "ignored": int(summary.group(4))})
        for match in re.finditer(r"Running [^\r\n]*\(([^()\r\n]+)\)", texts["stderr"]):
            executable = str(plain(match.group(1)))
            if executable in compiled and compiled[executable]:
                binaries.add(compiled[executable])
        passed = [v["name"] for v in cases if v["status"] == "ok"]
        failed = [v["name"] for v in cases if v["status"] == "FAILED"]
        ignored = [v["name"] for v in cases if v["status"] == "ignored"]
        required = []
        for field, values in original.items():
            if field.startswith("requiredNamed") and field.endswith("Tests"):
                required += values
        missing_names = [name for name in required if not any(name in actual for actual in passed)]
        missing_binaries = [name for name in original.get("requiredTestBinaries", []) if name not in binaries]
        text = texts["stdout"] + "\n" + texts["stderr"]
        missing_markers = [marker for marker in original.get("requiredProducerMarkers", []) if marker not in text]
        arguments = original["cargoArguments"]
        execution_required = arguments[0] == "test" and "--no-run" not in arguments
        floor = original.get("minimumExpectedMatchedTests", 1)
        counted = sum(value["passed"] for value in summaries)
        if self.payload.get("checkoutProduct"):
            # Fixed rustc argv has no JSON format option. Complete Native exit is
            # step evidence; the pinned packaging helper verifies both PE bytes.
            build_finished = native["rootExitCode"] == 0
        evidence_complete = (build_finished and not failed and not pending and not missing_names
            and not missing_binaries and not missing_markers
            and (not execution_required or bool(summaries) and len(passed) >= floor and counted == len(passed)
                 and all(value["status"] == "ok" and value["failed"] == 0 for value in summaries)))
        return {"status": "passed" if native["rootExitCode"] == 0 and evidence_complete else "failed",
                "ordinal": original["ordinal"], "name": original["name"], "recipe": original,
                "originalCargoArguments": arguments, "command": native["command"],
                "timeoutSeconds": original["ownedNativeTimeoutSeconds"], "nativeTerminal": native_ref,
                "rootExitCode": native["rootExitCode"], "logs": logs, "buildFinished": build_finished,
                "executionRequired": execution_required, "compileOnly": not execution_required,
                "passedTestCount": len(passed), "failedTestCount": len(failed), "ignoredTestCount": len(ignored),
                "passedTests": passed, "failedTests": failed, "ignoredTests": ignored,
                "testSummaries": summaries, "minimumExpectedMatchedTests": floor if execution_required else None,
                "unsettledTestNames": pending, "executedTestBinaries": sorted(binaries),
                "missingNamedTests": missing_names, "missingTestBinaries": missing_binaries,
                "missingProducerMarkers": missing_markers, "testEvidenceComplete": evidence_complete,
                "artifactPaths": sorted(set(artifacts)), "cargoFreshArtifactCount": fresh}

    def claim_batch_step(self, ordinal, command, root, request_ref):
        owner = "management-" + self.key
        with self.state.transaction() as connection:
            execution = self.state.get("execution", owner, connection=connection)
            intent = self.state.get("management_prelaunch", owner, connection=connection)
            launch = self.state.get("execution_launch", owner, connection=connection)
            previous = self.batch_claims()
            reservation = self.state.get("resource_reservation", intent["payload"]["reservationKey"],
                                         connection=connection) if intent else None
            hold = self.state.get("pool_writer_hold", intent["payload"]["preparationKey"],
                                  connection=connection) if intent else None
            if (not execution or not intent or not reservation or not hold
                    or execution["payload"].get("reservationKey") != intent["payload"]["reservationKey"]
                    or reservation["payload"].get("status") != "active"
                    or hold["payload"].get("status") != "active" or hold["payload"].get("owner") != owner
                    or hold["payload"].get("reservationKey") != reservation["key"]
                    or self.state.get("execution_host", owner, connection=connection)
                    or ordinal != len(previous) + 1 or any(v.get("status") != "complete" for v in previous)
                    or not launch and intent["payload"].get("status") != "unlaunched"):
                raise ValueError("Batch step lacks its exact continuously protected parent admission")
            claim = {"ordinal": ordinal, "status": "claimed", "command": command, "cwd": str(root),
                     "request": request_ref}
            value = {**intent["payload"], "batch": True, "inputDigest": self.input_digest,
                     "status": "claimed", "ownedNativeLaunchClaimed": True,
                     "currentStep": ordinal, "stepClaims": [*previous, claim],
                     "command": command, "cwd": str(root), "request": request_ref}
            self.state.put("execution_launch", owner, value,
                           expected_version=launch["version"] if launch else 0, connection=connection)
            if not launch:
                self.state.put("management_prelaunch", owner, {**intent["payload"], "status": "claimed",
                               "ownedNativeLaunchClaimed": True, "batch": True},
                               expected_version=intent["version"], connection=connection)
            self.state.put("execution", owner, {**execution["payload"], "status": "running",
                           "launchClaimed": True, "batch": True, "command": command,
                           "currentStep": ordinal}, expected_version=execution["version"], connection=connection)
        return claim

    def batch_failure_release(self, error):
        read = self.state.get("management_checkout_read", "management-" + self.key)
        if read and read["payload"].get("status") in {"active", "unknown"}:
            return None
        admission = getattr(self, "_validation_admission", None)
        if not admission:
            return None
        resources, owner, writer, reservation = admission
        claims = self.batch_claims()
        if not claims:
            return self.validation_failure_release(error, single_only=True)
        try:
            proofs = [self.batch_step_proof(claim, persist=True) for claim in claims]
        except (ValueError, OSError, KeyError, TypeError):
            return None
        proof_id = proofs[-1][3]
        released = resources.release(reservation, writer_key=writer, native_proof_ref=proof_id)
        launch, execution = self.state.get("execution_launch", owner), self.state.get("execution", owner)
        self.state.put("execution_launch", owner, {**launch["payload"], "status": "terminal",
                       "nativeProofRef": proof_id, "batchAborted": True}, expected_version=launch["version"])
        self.state.put("execution", owner, {**execution["payload"], "status": "failed",
                       "nativeProofRef": proof_id, "batchAborted": True}, expected_version=execution["version"])
        return {"kind": "actual-batch-step-terminals", "nativeProofRefs": [value[3] for value in proofs],
                "reservationKey": reservation, "released": released}


    def validate_batch(self):
        if self.payload.get("sourceView", "worktree") == "checkout":
            return self.validate_checkout(batch=self.validation_batch())
        from tools.jenkins.resources.capacity import Capacity, ResourceManager
        from tools.jenkins.resources.paths import canonical_build_root, physical_path_under
        from tools.jenkins.workflow.planning import _environment_policy
        from tools.jenkins.contracts import JenkinsError
        batch = self.validation_batch()
        recipes, coverage = batch["recipes"], batch["coverage"]
        reuse = batch["reuseValidation"]
        observe = self.validation_sealed_observation if reuse is not None else self.validation_observation
        checkpoint = self.batch_checkpoint()
        owner = "management-" + self.key
        build_root = self.build_root()
        approved = canonical_build_root(build_root)
        output, report = self.outputs(), self.outputs() / "reports" / self.key
        root = plain(self.repo) if self.payload["sourceView"] == "checkout" else self.wt(self.payload["worktreeId"])[1]
        policy = self.state.get("resource_policy", "default")["payload"]
        resources = ResourceManager(self.state, Capacity(policy["cpuBudget"], policy["memoryBudget"], policy["diskBudget"]))
        parallelism = max(max(recipe["jobs"], recipe["testThreads"] if recipe["verb"] == "test"
                             and not recipe["noRun"] else 1) for recipe in recipes)
        amount = Capacity(parallelism, 2 * parallelism * 1024**3, 4 * 1024**3)
        environment, trusted, _policy_file = self.validation_environment()
        preparation = source_helper()
        cache_fact = self.dependency_cache()
        pool, cargo_configuration = self.batch_pool(recipes, preparation, trusted, environment, root, cache_fact)
        writer, reservation_key = "management-pool-" + pool, owner + ":" + owner
        target = physical_path_under(approved, output / "pools" / pool / "target")
        execution_root = physical_path_under(approved, target.parent / "inputs/source")
        binding = {"pool": pool, "writer": writer, "reservationKey": reservation_key,
                   "sourceRoot": str(root), "buildRoot": build_root, "executionRoot": str(execution_root),
                   "report": str(report), "coverageDigest": sha(encoded(coverage))}
        if checkpoint.get("pool") and any(checkpoint.get(k) != v for k, v in binding.items()):
            raise ValueError("Original shared batch preparation/configuration/volume changed")
        self.batch_checkpoint(**binding)
        helper_path = client_path(self.config, self.repo, "tools/dev/local_cargo.py")
        spec = importlib.util.spec_from_file_location("jenkins_managed_batch_cargo", helper_path)
        helper = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(helper)
        for tool in ("cargo", "rustc"):
            if sha(plain(trusted[tool + "Path"]).read_bytes()) != trusted[tool + "Sha256"]:
                raise ValueError("Trusted compiler executable changed")
        intent = self.state.get("management_prelaunch", owner)
        if intent:
            original_intent = intent["payload"]
            if any(original_intent.get(k) != v for k, v in {
                    "operationKey": self.key, "requestId": self.request, "operationId": self.operation,
                    "sessionId": self.session, "preparationKey": writer, "reservationKey": reservation_key,
                    "sourceRoot": str(root), "report": str(report)}.items()):
                raise ValueError("Original batch admission intent changed")
            self._validation_admission = (resources, owner, writer, reservation_key)
        else:
            self.begin_validation_intent(resources, owner, writer, root, report)
        admitted = self.state.get("resource_reservation", reservation_key)
        if checkpoint.get("stage") == "complete":
            result = read_reference(checkpoint["receipt"], expected_path=report / "receipt.json")
            proofs = [self.batch_step_proof(claim) for claim in self.batch_claims()]
            if admitted and admitted["payload"].get("status") == "active":
                resources.release(reservation_key, writer_key=writer, native_proof_ref=proofs[-1][3])
            return result
        if not admitted or admitted["payload"].get("status") != "active":
            if self.batch_claims() or self.state.get("execution", owner):
                raise ValueError("Existing batch must reconcile its original protected reservation")
            resources.queue(owner, request_id=owner, amount=amount, build_root=build_root,
                            heavy_writer=True, writer_key=writer)
            # A resource wait performs no complete source capture or CAS walk,
            # and the existing Groovy loop sleeps outside its node allocation.
            if helper.free_bytes(approved.path) < policy.get("diskReserveBytes", 35 * 1024**3) + amount.disk_bytes:
                return {"status": "waiting", "reasonCode": "resource_capacity", "buildRoot": build_root,
                        "formalAcceptance": False, "m8Accepted": False}
            _inventory, generation = resources.scan_inventory(build_root)
            try:
                admitted = resources.admit(owner, amount, inventory_generation=generation, writer_key=writer,
                    request_id=owner, build_root=build_root, heavy_writer=True)
            except JenkinsError as error:
                if error.retryable or error.code in {"pool_writer_active", "pool_cache_maintenance_active"}:
                    return {"status": "waiting", "reasonCode": error.code, "buildRoot": build_root,
                            "formalAcceptance": False, "m8Accepted": False}
                raise
            if admitted is None:
                return {"status": "waiting", "reasonCode": "resource_capacity", "buildRoot": build_root,
                        "formalAcceptance": False, "m8Accepted": False}
        hold = self.state.get("pool_writer_hold", writer)
        if (admitted["payload"].get("owner") != owner or admitted["payload"].get("buildRoot") != build_root
                or not hold or hold["payload"].get("owner") != owner or hold["payload"].get("status") != "active"
                or hold["payload"].get("reservationKey") != admitted["key"]):
            raise ValueError("Shared batch lost its exact physical reservation/writer")
        captured = self.state.get("management_source", self.key)
        if reuse is not None:
            before, provenance = self.batch_reused_input(reuse, root)
            if (captured and captured["payload"] != before
                    or checkpoint.get("sourceProvenance") not in (None, provenance)):
                raise ValueError("Original explicitly reused input or its donor provenance changed")
            if not captured:
                self.state.put("management_source", self.key, before, expected_version=0)
            self.batch_checkpoint(sourceProvenance=provenance)
        elif captured:
            before = captured["payload"]
            if (before["sourceRoot"] != str(root)
                    or sha(encoded({k: v for k, v in before.items() if k != "sourceDigest"})) != before["sourceDigest"]):
                raise ValueError("Original complete batch source capture changed")
            self.verify_declared_inputs(before, self.payload)
        else:
            root, before = self.validation_with_cache()
            self.state.put("management_source", self.key, before, expected_version=0)
        captured_configuration = {path: value for path, value in before["source"]["files"].items()
                                  if path.casefold() in {".cargo/config", ".cargo/config.toml"} and value is not None}
        if before["dependencyCache"] != cache_fact or captured_configuration != cargo_configuration:
            raise ValueError("Captured Cargo/cache configuration differs from its shared pool")
        self.batch_checkpoint(capturedSourceDigest=before["sourceDigest"])
        cache, scratch = plain(before["dependencyCache"]["home"]), plain(report / "tmp")
        environment.setdefault("SYSTEMROOT", os.environ.get("SYSTEMROOT", r"C:\Windows"))
        environment.update({"CARGO_TARGET_DIR": str(target), "CARGO_BUILD_BUILD_DIR": str(target.parent / "build"),
            "CARGO_HOME": str(cache), "TEMP": str(scratch), "TMP": str(scratch), "TMPDIR": str(scratch),
            "RUSTC": trusted["rustcPath"], "RUSTC_WRAPPER": "", "RUSTC_WORKSPACE_WRAPPER": "",
            "PYTHONDONTWRITEBYTECODE": "1", "CARGO_ENCODED_RUSTFLAGS": "", "RUSTFLAGS": "", "RUSTDOCFLAGS": ""})
        commands, environments = [], []
        for ordinal, recipe in enumerate(recipes):
            raw = list(self.payload["batch"]["recipes"][ordinal]["cargoArguments"])
            separator = raw.index("--") if "--" in raw else len(raw)
            controls = ["--message-format=json"]
            if not any(v == "--jobs" or v.startswith("--jobs=") or v.startswith("-j") for v in raw[:separator]):
                controls += ["--jobs", str(recipe["jobs"])]
            command = helper.prepare_command([*raw[:separator], *controls, *raw[separator:]], target,
                                              locked=recipe["locked"])
            command[0] = trusted["cargoPath"]
            commands.append(command)
            environments.append({**environment, "CARGO_NET_OFFLINE": "true" if recipe["offline"] else "false",
                                 **batch["stepEnvironments"][ordinal]})
        environment_digests = [sha(encoded(value)) for value in environments]
        execution_identity = sha(encoded({"sourceDigest": before["sourceDigest"], "commands": commands,
            "cwd": str(execution_root), "environmentDigests": environment_digests, "coverage": coverage,
            "driverDigest": self.config["driverDigest"], "runtimeOperationId": self.config["runtimeOperationId"]}))
        outputs = {"build": target.parent, "cache": cache, "reports": report, "tmp": scratch,
                   "source": execution_root, "sourceObjects": approved.namespace() / "inputs"}
        self.artifact_manifest(root, outputs)
        for path in (target, target.parent / "build", cache):
            path.mkdir(parents=True, exist_ok=True)
        scratch_record = self.state.get("management_task_scratch", self.key)
        if scratch_record:
            scratch_path = report / "tmp-generation.json"
            task_scratch = {"path": str(scratch_path), "sha256": sha(scratch_path.read_bytes())}
            if (read_reference(task_scratch, expected_path=scratch_path) != scratch_record["payload"]
                    or scratch_record["payload"].get("environmentDigest") != environment_digests[0]
                    or scratch_record["payload"].get("status") != "owned"):
                raise ValueError("Original shared TEMP generation requires reconciliation")
        else:
            task_scratch = self.task_tmp(environment=environments[0])
        self.artifact_manifest(root, outputs, task_scratch=task_scratch)
        checkpoint = self.batch_checkpoint()
        sealed_path = report / "sealed-input.json"
        if sealed_path.exists():
            sealed_ref = {"path": str(sealed_path), "sha256": sha(sealed_path.read_bytes())}
            sealed = read_reference(sealed_ref, expected_path=sealed_path, max_bytes=SOURCE_EVIDENCE_MAX_BYTES)
        else:
            sealed = preparation.seal_source(self.state, source_root=root, snapshot=before,
                repository_root=self.repo, repository_id=self.repo_id, session_id=self.session,
                owner=owner, approved=approved, coverage=coverage,
                management=self if before.get("externalInputPolicy") == "prepared-sealed-CAS" else None)
            sealed_ref = write_json(sealed_path, sealed)
        if (checkpoint.get("sealed") not in (None, sealed_ref) or sealed.get("capturedInputVerified") is not True
                or sealed["capturedSourceDigest"] != before["sourceDigest"]
                or sealed["coverageDigest"] != sha(encoded(coverage))
                or sealed["sealedInputRef"] != sha(encoded(sealed["sourceManifest"]))):
            raise ValueError("Original aggregate shared seal identity changed")
        self.batch_checkpoint(sealed=sealed_ref)
        prepared_path = report / "preparation.json"
        if prepared_path.exists():
            prepared_ref = {"path": str(prepared_path), "sha256": sha(prepared_path.read_bytes())}
            prepared = read_reference(prepared_ref, expected_path=prepared_path)
        else:
            current_execution = self.state.get("execution", owner)
            materialization_state = (ReconciledBatchPreparation(self.state, owner) if current_execution else self.state)
            prepared = preparation.materialize_source(materialization_state, approved=approved, owner=owner,
                preparation_key_value=writer, source_path=execution_root, reservation_key=admitted["key"],
                sealed=sealed, identity={"repositoryId": self.repo_id, "sessionId": self.session,
                    "requestId": self.request, "operationId": self.operation, "driverDigest": self.config["driverDigest"],
                    "runtimeOperationId": self.config["runtimeOperationId"]},
                command=commands[0], environment_digest=environment_digests[0])
            prepared_ref = write_json(prepared_path, prepared)
        if (checkpoint.get("prepared") not in (None, prepared_ref)
                or prepared["sealedInputRef"] != sealed["sealedInputRef"]
                or prepared["sourceDigest"] != sealed["sealedInputRef"]
                or prepared["capturedSourceDigest"] != before["sourceDigest"]
                or prepared["physicalPath"] != str(execution_root)):
            raise ValueError("Original materialized shared input identity changed")
        self.batch_checkpoint(prepared=prepared_ref)
        checkpoint = self.batch_checkpoint()
        if checkpoint.get("finalObservation"):
            final_observation = read_reference(checkpoint["finalObservation"], expected_path=report / "input-final.json",
                                               max_bytes=SOURCE_EVIDENCE_MAX_BYTES)
            request_ref = checkpoint["request"]
            request_document = read_reference(request_ref, expected_path=report / "request.json",
                                              max_bytes=SOURCE_EVIDENCE_MAX_BYTES)
            prepared_before = request_document["preparedSource"]
            preparation_observation = request_document["inputObservation"]
        else:
            prepared_before = preparation.verify_source(sealed["sourceManifest"], execution_root, approved)
            if prepared_before["sourceDigest"] != sealed["sealedInputRef"]:
                raise ValueError("Complete execution input differs from its shared seal")
            observed_root, _observed, preparation_observation = observe(before)
            if (observed_root != root or preparation_observation["originalEngineUnchanged"] is not True
                    or preparation_observation["dependencyCacheUnchanged"] is not True):
                raise ValueError("Original engine/cache changed before shared batch execution")
            parent_request = {"payload": self.payload, "inputDigest": self.input_digest, "source": before,
                "sourceRoot": str(root), "executionRoot": str(execution_root), "command": commands[0],
                "commands": commands, "target": str(target), "resource": amount.__dict__, "coverage": coverage,
                "preparation": prepared, "sourceSealing": sealed["metrics"], "preparedSource": prepared_before,
                "validationInputPolicy": "sealed-source", "capturedInputVerified": True,
                "inputObservation": preparation_observation, "executionIdentity": execution_identity,
                "taskScratch": task_scratch, "environmentDigest": environment_digests[0],
                "stepEnvironmentDigests": environment_digests,
                "temporaryEnvironment": {k: environment[k] for k in ("TEMP", "TMP", "TMPDIR")}}
            if reuse is not None:
                parent_request["sourceProvenance"] = provenance
            if checkpoint.get("request"):
                request_ref = checkpoint["request"]
                original_request = read_reference(request_ref, expected_path=report / "request.json",
                                              max_bytes=SOURCE_EVIDENCE_MAX_BYTES)
                # A resume rechecks current engine/cache and prepared bytes, but
                # preserves the original (possibly changed foreign) observation.
                for key in ("payload", "inputDigest", "commands", "preparation", "preparedSource",
                            "executionIdentity", "stepEnvironmentDigests", "taskScratch", "coverage"):
                    if original_request[key] != parent_request[key]:
                        raise ValueError("Original complete batch request changed during resume")
                if original_request.get("sourceProvenance") != parent_request.get("sourceProvenance"):
                    raise ValueError("Original sealed donor provenance changed during resume")
                preparation_observation = original_request["inputObservation"]
            else:
                request_ref = write_json(report / "request.json", parent_request)
            self.batch_checkpoint(request=request_ref, stage="prepared")
        completed = []
        for ordinal, original in enumerate(self.payload["batch"]["recipes"], 1):
            claims = self.batch_claims()
            if len(claims) >= ordinal:
                result, _native, _native_ref, _proof_id = self.batch_step_proof(claims[ordinal - 1], persist=True)
            else:
                if checkpoint.get("finalObservation"):
                    raise ValueError("Final input proof cannot precede an unexecuted original recipe")
                step_report = report / "steps" / ("%04d" % ordinal)
                step_request = {"parentRequest": request_ref, "inputDigest": self.input_digest,
                    "ordinal": ordinal, "recipe": original, "command": commands[ordinal - 1], "cwd": str(execution_root),
                    "timeoutSeconds": original["ownedNativeTimeoutSeconds"], "taskScratch": task_scratch,
                    "sealedInputRef": sealed["sealedInputRef"], "environmentDigest": environment_digests[ordinal - 1],
                    "temporaryEnvironment": {k: environment[k] for k in ("TEMP", "TMP", "TMPDIR")}}
                step_ref = write_json(step_report / "request.json", step_request)
                claim = self.claim_batch_step(ordinal, commands[ordinal - 1], execution_root, step_ref)
                native = helper._run_owned_command(commands[ordinal - 1], execution_root,
                    environments[ordinal - 1], step_report, original["ownedNativeTimeoutSeconds"])
                write_json(step_report / "native-terminal.json", native)
                result, _native, _native_ref, _proof_id = self.batch_step_proof(claim, persist=True)
            completed.append(result)
        # Full source and prepared-byte checks occur once before and once after
        # the complete contiguous sequence, never once for every recipe.
        if not checkpoint.get("finalObservation"):
            input_error, prepared_error = None, None
            try:
                observed_root, after, completion_observation = observe(before)
                if observed_root != root:
                    raise ValueError("Shared batch source root changed")
            except Exception as error:
                after, input_error = None, str(error)
                completion_observation = {"originalEngineUnchanged": False, "dependencyCacheUnchanged": False,
                                          "inputsUnchanged": False, "captureError": str(error)}
            try:
                prepared_after = preparation.verify_source(sealed["sourceManifest"], execution_root, approved)
            except Exception as error:
                prepared_after, prepared_error = None, str(error)
            final_observation = {"after": after, "observation": completion_observation,
                                 "prepared": prepared_after, "inputError": input_error,
                                 "preparedError": prepared_error}
            final_ref = write_json(report / "input-final.json", final_observation)
            self.batch_checkpoint(finalObservation=final_ref, stage="closed", allRecipesExecuted=True)
        proof = {"validationInputPolicy": "sealed-source", "capturedInputVerified": True,
            "originalEngineUnchanged": preparation_observation["originalEngineUnchanged"] is True
                                      and final_observation["observation"]["originalEngineUnchanged"] is True,
            "dependencyCacheUnchanged": preparation_observation["dependencyCacheUnchanged"] is True
                                       and final_observation["observation"]["dependencyCacheUnchanged"] is True,
            "inputsUnchanged": before == final_observation["after"],
            "preparedInputsUnchanged": prepared_before == final_observation["prepared"]}
        last = self.batch_step_proof(self.batch_claims()[-1])
        task_cache = self.task_tmp(native=last[1], native_ref=last[2], proof_id=last[3])
        status = "passed" if all(step["status"] == "passed" for step in completed) and validation_inputs_verified(proof) else "failed"
        result = {"status": status, "batch": True, "request": request_ref, "steps": completed,
            "recipeCount": len(recipes), "completedRecipeCount": len(completed), "coverage": coverage,
            "nativeProofRefs": [value["nativeProofRef"] for value in completed], "nativeTerminal": last[2],
            "sourceDigest": before["sourceDigest"], "sourceRoot": str(root), "executionRoot": str(execution_root),
            "preparation": prepared, "sourceSealing": sealed["metrics"], "executionIdentity": execution_identity,
            "inputObservations": {"afterPreparation": preparation_observation,
                                  "afterExecution": final_observation["observation"]}, **proof,
            "inputError": final_observation["inputError"], "preparedInputError": final_observation["preparedError"],
            "target": str(target), "requestedTargetDir": self.payload.get("requestedTargetDir"),
            "actualTargetDir": str(target), "targetPolicy": "managed-compatibility-pool",
            "taskScratch": task_scratch, "postTaskCache": task_cache, "formalAcceptance": False,
            "m8Accepted": False, "performanceAccepted": False, "productsAccepted": False}
        result["artifactManifest"] = self.artifact_manifest(root, outputs, native=last[1],
            task_scratch=task_scratch, task_cache=task_cache)
        receipt_ref = write_json(report / "receipt.json", result)
        execution, launch = self.state.get("execution", owner), self.state.get("execution_launch", owner)
        self.state.put("execution", owner, {**execution["payload"], "status": status, **proof,
            "batch": True, "nativeProofRef": last[3], "nativeTerminal": last[2],
            "batchReceipt": receipt_ref, "executionIdentity": execution_identity}, expected_version=execution["version"])
        self.state.put("execution_launch", owner, {**launch["payload"], "status": "terminal",
            "nativeProofRef": last[3], "allRecipesExecuted": True}, expected_version=launch["version"])
        self.batch_checkpoint(stage="complete", receipt=receipt_ref, allRecipesExecuted=True)
        resources.release(admitted["key"], writer_key=writer, native_proof_ref=last[3])
        return result


    def validation_environment(self):
        """Bind typed read-only archive input into the trusted native identity."""
        from tools.jenkins.workflow.planning import _environment_policy
        environment, trusted, policy_file = _environment_policy(self.state, self.repo)
        environment = compact_build_environment(environment)
        environment = {key: value for key, value in environment.items()
                       if key.casefold() != "zircon_docs_history_archive_root"}
        archive_input = fixed_history_archive_input(self.payload)
        if archive_input is not None:
            environment["ZIRCON_DOCS_HISTORY_ARCHIVE_ROOT"] = archive_input["root"]
        return environment, trusted, policy_file

    def validation_recipe(self):
        allowed = {"action", "repositoryId", "sessionId", "requestId", "operationId", "driverDigest",
                   "runtimeOperationId", "ownedPaths", "observationOnly", "sourceView", "worktreeId",
                   "externalRoots", "externalGitlinks", "buildRoot", "packages", "features", "profile",
                   "verb", "offline", "noDefaultFeatures", "lib", "tests", "testFilter", "noRun",
                   "allFeatures", "locked", "jobs", "testThreads", "requestedTargetDir",
                   "expectedSourceDigest", "expectedEngineSourceDigest", "externalCapturePolicy",
                   "sourceHashes", "dependencyHashes", "bins", "allTests", "harnessFilters",
                   "exact", "ignored", "nocapture", "worktreePreparationRef", "sourceEvidencePolicy",
                   "fixedHistoryArchiveInput", "manifestPath"}
        if set(self.payload) - allowed:
            raise ValueError("Unsupported Cargo fields: " + ", ".join(sorted(set(self.payload) - allowed)))
        if ("sourceEvidencePolicy" in self.payload
                and self.payload["sourceEvidencePolicy"] != "full-file-index-v2"):
            raise ValueError("Unsupported source evidence policy")
        if ("worktreePreparationRef" in self.payload
                and (not isinstance(self.payload["worktreePreparationRef"], str)
                     or not re.fullmatch(r"[0-9a-f]{64}", self.payload["worktreePreparationRef"]))):
            raise ValueError("Worktree preparation requires an exact operation reference")
        if {"externalCapturePolicy", "expectedEngineSourceDigest"} & set(self.payload):
            engine_digest = self.payload.get("expectedEngineSourceDigest")
            if (self.payload.get("externalCapturePolicy") != "jenkins-admission"
                    or not isinstance(engine_digest, str)
                    or not re.fullmatch(r"[0-9a-f]{64}", engine_digest)):
                raise ValueError("Jenkins dependency admission requires an explicit complete engine source digest")
        archive_input = fixed_history_archive_input(self.payload)
        packages, features = self.payload.get("packages"), self.payload.get("features", [])
        tests = self.payload.get("tests", [])
        if (not isinstance(packages, list) or not 1 <= len(packages) <= 8
                or any(not isinstance(p, str) or not re.fullmatch(r"[A-Za-z0-9_-]{1,80}", p) for p in packages)
                or not isinstance(features, list) or len(features) > 64
                or any(not isinstance(f, str) or not re.fullmatch(r"[A-Za-z0-9_:/.-]{1,120}", f) for f in features)
                or not isinstance(tests, list) or len(tests) > 16
                or any(not isinstance(t, str) or not re.fullmatch(r"[A-Za-z0-9_.-]{1,80}", t) for t in tests)):
            raise ValueError("Explicit bounded Cargo packages/features/test targets required")
        manifest_fields = {}
        if "manifestPath" in self.payload:
            manifest = self.payload["manifestPath"]
            if (not isinstance(manifest, str) or manifest not in {"Cargo.toml", "zircon_plugins/Cargo.toml"}
                    or manifest == "zircon_plugins/Cargo.toml" and len(packages) != 1):
                raise ValueError("A catalog Cargo manifest and one explicit plugin package are required")
            manifest_fields["manifestPath"] = manifest
        verb, profile = self.payload.get("verb", "test"), self.payload.get("profile", "dev")
        if (verb not in {"check", "build", "test"} or profile not in {"dev", "release", "test"}
                or profile == "test" and verb != "test"):
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
        extra = {name: self.payload[name] for name in ("bins", "allTests", "exact", "ignored", "nocapture")
                 if name in self.payload}
        filters = self.payload.get("harnessFilters", [])
        if (any(type(value) is not bool for value in extra.values())
                or not isinstance(filters, list) or len(filters) > 512
                or any(not isinstance(value, str) or not value or value.startswith("-")
                       or len(value.encode("utf-8")) > 256 or any(c in value for c in "\0\r\n")
                       for value in filters)
                or verb != "test" and (filters or any(extra.get(k) for k in ("exact", "ignored", "nocapture")))
                or options["noRun"] and (filters or any(extra.get(k) for k in ("exact", "ignored", "nocapture")))):
            raise ValueError("Unsupported Cargo targets or harness selectors")
        if "harnessFilters" in self.payload:
            extra["harnessFilters"] = filters
        if not options["noDefaultFeatures"] or options["allFeatures"] or extra.get("allTests"):
            raise ValueError("build_scope_policy: require explicit features with noDefaultFeatures=true; broad feature/test selection prohibited")
        if verb == "test" and not ((options["lib"] and not tests) or (not options["lib"] and len(tests) == 1)):
            raise ValueError("build_scope_policy: select lib or exactly one test target")
        if manifest_fields.get("manifestPath") == "zircon_plugins/Cargo.toml":
            editor_filters = {"zircon_plugin_material_editor_editor": {"material_graph_"},
                              "zircon_plugin_prefab_tools_editor": {"prefab_"},
                              "zircon_plugin_ai_editor": {"authoring_tests::", "gameplay_debugger_tests::",
                                                          "tests::pie_mirror_", "tests::perception_overlay_"}}
            plugin_lib = (len(packages) == 1 and packages[0] in editor_filters and not features
                          and options["lib"] and not tests and test_filter in editor_filters[packages[0]])
            gltf = (packages == ["zircon_plugin_gltf_importer_runtime"]
                    and sorted(features) == sorted([
                        "runtime", "zircon_runtime/native-host", "zircon_runtime/animation"])
                    and ((not options["lib"] and tests == ["character_skinning_sources"] and test_filter is None)
                         or (options["lib"] and not tests and test_filter
                             == "source_snapshot_tests::importer_rejects_required_extensions_outside_its_stable_policy")))
            if not ((plugin_lib or gltf) and verb == "test" and profile == "test" and options["locked"]
                    and not options["noRun"] and jobs == threads == 1
                    and not filters and not any(extra.values())):
                raise ValueError("build_scope_policy: plugin manifest permits only the bounded material, prefab, AI lib recipes or the two glTF source test recipes")
        command = [verb, "--jobs", str(jobs), "--message-format=json"]
        if manifest_fields:
            command[1:1] = ["--manifest-path", manifest_fields["manifestPath"]]
        for name, flag in (("bins", "--bins"), ("allTests", "--tests")):
            if extra.get(name):
                command.append(flag)
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
        elif profile == "test":
            command += ["--profile", "test"]
        if verb == "test" and not options["noRun"]:
            if test_filter is not None:
                command.append(test_filter)
            command += ["--", "--test-threads", str(threads)]
            for name, flag in (("exact", "--exact"), ("ignored", "--ignored"), ("nocapture", "--nocapture")):
                if extra.get(name):
                    command.append(flag)
            command += filters
        return {"verb": verb, "profile": profile, "packages": packages, "features": features,
                "tests": tests, "testFilter": test_filter, "jobs": jobs, "testThreads": threads,
                **options, **extra, **manifest_fields, "command": command,
                **({"fixedHistoryArchiveInput": archive_input} if archive_input is not None else {})}

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
            maximum = 128 if field == "sourceHashes" else 4096
            if not isinstance(requested, dict) or not 1 <= len(requested) <= maximum:
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

    def validation_failure_release(self, error, *, single_only=False):
        read = self.state.get("management_checkout_read", "management-" + self.key)
        context = getattr(self, "_checkout_context", None)
        if (read and read["payload"].get("status") in {"active", "unknown"}
                or context is not None and context.get("readReleased") is not True):
            return None
        if "batch" in self.payload and not single_only:
            return self.batch_failure_release(error)
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
            return self.validate_batch() if "batch" in self.payload else self.validate_prepared()
        except Exception as error:
            if ("batch" in self.payload and not self.batch_claims()
                    and (getattr(error, "errno", None) == 28 or getattr(error, "winerror", None) in {39, 112}
                         or getattr(error, "code", None) in {"source_storage_budget", "source_quota_exceeded"})):
                return {"status": "waiting", "reasonCode": "source_storage_budget", "message": str(error),
                        "buildRoot": self.build_root(), "originalReservationRetained": True,
                        "formalAcceptance": False, "m8Accepted": False}
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
        if "batch" in self.payload:
            claims = self.batch_claims()
            value["batch"] = {"contractRef": self.payload["batch"]["contractRef"],
                              "expectedRecipeCount": len(self.payload["batch"]["recipes"]),
                              "steps": claims, "sharedWriter": self.batch_checkpoint().get("writer")}
            value["processes"] = []
            for claim in claims:
                proof = self.state.get("native_job", claim.get("nativeProofRef", ""))
                if proof:
                    actual = proof["payload"]
                    value["processes"].append({"pid": actual["pid"], "creationTime": actual["creationTime"],
                        "command": actual["nativeEvidence"]["command"], "ordinal": claim["ordinal"],
                        "nativeProofRef": claim["nativeProofRef"],
                        "native_terminal": actual.get("completeProof", {}).get("complete") is True})
                else:
                    value["processes"].append({"ordinal": claim["ordinal"], "command": claim["command"],
                                               "native_terminal": False, "status": "unproven"})
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
            expected_native_path = path.parent / "native-terminal.json"
            expected_proof_id = "management-" + self.key + "-native"
            expected_environment_digest = record["environmentDigest"]
            if "batch" in self.payload:
                checkpoint = self.batch_checkpoint()
                claims = self.batch_claims()
                if (checkpoint.get("allRecipesExecuted") is not True
                        or checkpoint.get("stage") not in {"closed", "complete"}
                        or len(claims) != len(self.payload["batch"]["recipes"])
                        or any(value.get("status") != "complete" for value in claims)):
                    raise ValueError("Shared TEMP remains protected until every batch phase has real terminal evidence")
                phases = [self.batch_step_proof(value) for value in claims]
                last = phases[-1]
                if (last[1] != native or last[2] != native_ref or last[3] != proof_id):
                    raise ValueError("Shared TEMP release differs from the actual last completed step")
                expected_native_path, expected_proof_id = plain(last[2]["path"]), last[3]
                step_request = read_reference(claims[-1]["request"])
                if (step_request["taskScratch"] != scratch_ref
                        or any(step_request["temporaryEnvironment"].get(k) != str(path)
                               for k in ("TEMP", "TMP", "TMPDIR"))):
                    raise ValueError("Batch step TEMP differs from the exclusive shared generation")
                expected_environment_digest = step_request["environmentDigest"]
            if plain(native_ref["path"]) != expected_native_path:
                raise ValueError("Native evidence is outside the original task report")
            evidence = plain(native_ref["path"]).read_bytes()
            proof = self.state.get("native_job", proof_id)["payload"]
            if sha(evidence) != native_ref["sha256"] or json.loads(evidence) != native:
                raise ValueError("Original native evidence bytes changed")
            expected_proof = native_terminal_proof(json.loads(evidence), native_ref)
            tree = native["processTree"]
            if (proof_id != expected_proof_id or proof["nativeEvidence"] != native
                    or proof.get("owner") != "management-" + self.key or proof.get("role") != "management_owned_cargo"
                    or proof.get("requestId") != self.request or proof.get("operationId") != self.operation
                    or proof.get("pid") != native["pid"] or proof.get("creationTime") != native["creationTime"]
                    or proof.get("birthToken") != native["creationTime"]
                    or proof.get("executable") != expected_proof["identity"]["executable"]
                    or proof.get("commandDigest") != expected_proof["identity"]["commandDigest"]
                    or proof.get("taskScratch") != scratch_ref or proof.get("environmentDigest") != expected_environment_digest
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

    def checkout_read_inputs(self, before, compiler, preparation, helper):
        """Capture original checkout bytes, configuration and directory membership."""
        import stat
        import tomllib
        files, folded, trees, git_roots, configuration = {}, {}, {}, [], []
        monitors = {str(plain(before["sourceRoot"])): None}

        def monitor_file(path):
            if any(values is None and path.is_relative_to(Path(name))
                   for name, values in monitors.items()):
                return
            parent = path.parent
            while not parent.is_dir():
                if parent == parent.parent:
                    raise ValueError("An input has no existing physical monitor ancestor")
                parent = parent.parent
            parent = plain(parent)
            key = str(parent)
            if key not in monitors:
                monitors[key] = []
            if monitors[key] is not None:
                relative = path.relative_to(parent).as_posix()
                if relative not in monitors[key]:
                    monitors[key].append(relative)

        def add(path, checksum):
            path = plain(path)
            _record_checkout_input_identity(files, folded, path, checksum)
            # Full roots are added separately; narrow ancestors also observe
            # creation of a missing configuration directory along this path.
            monitor_file(path)

        def file(path):
            path = plain(path)
            try:
                checksum = helper._file_fact(path)["sha256"]
            except FileNotFoundError:
                checksum = None
            add(path, checksum)
            return checksum

        def tree(path):
            path, name = plain(path), str(plain(path))
            if name in trees:
                return
            try:
                observed = path.lstat()
            except FileNotFoundError:
                trees[name] = {"present": False, "files": {}, "directories": []}
                return
            if not stat.S_ISDIR(observed.st_mode):
                raise ValueError("Input tree is not a physical directory: " + name)
            manifest, directories, pending = {}, [], [path]
            while pending:
                directory = pending.pop()
                directories.append(directory.relative_to(path).as_posix())
                with os.scandir(directory) as entries:
                    for entry in entries:
                        child = plain(Path(entry.path))
                        if entry.is_dir(follow_symlinks=False):
                            pending.append(child)
                        elif entry.is_file(follow_symlinks=False):
                            checksum = file(child)
                            if checksum is None:
                                raise ValueError("Input disappeared during inventory: " + str(child))
                            manifest[child.relative_to(path).as_posix()] = checksum
                        else:
                            raise ValueError("Unsupported input tree entry: " + str(child))
            trees[name] = {"present": True, "files": dict(sorted(manifest.items())),
                           "directories": sorted(directories)}

        def config_path(value, relative_to):
            if not isinstance(value, str) or not value or "\0" in value:
                raise ValueError("Configuration input must name an exact file")
            path = Path(os.path.expanduser(value))
            if not path.is_absolute():
                path = relative_to / path
            return plain(path)

        def rust_flags(value):
            tokens = value.split() if isinstance(value, str) else value
            if not isinstance(tokens, list) or any(not isinstance(item, str) or not item for item in tokens):
                raise CheckoutInputIncomplete("Rust flags must have an exact supported argument list")
            pure_codegen = {"opt-level", "debuginfo", "debug-assertions", "overflow-checks", "lto",
                "panic", "codegen-units", "embed-bitcode", "force-frame-pointers", "force-unwind-tables",
                "prefer-dynamic", "relocation-model", "code-model", "target-cpu", "target-feature",
                "symbol-mangling-version", "split-debuginfo", "strip", "link-dead-code"}
            offset = 0
            while offset < len(tokens):
                flag = tokens[offset]
                offset += 1
                if flag in {"-O", "-g", "--document-private-items", "--document-hidden-items", "--show-coverage"}:
                    continue
                if flag == "-C" or flag.startswith("-C") and len(flag) > 2:
                    option = flag[2:]
                    if flag == "-C":
                        if offset == len(tokens):
                            raise CheckoutInputIncomplete("Rust codegen option has no value")
                        option = tokens[offset]
                        offset += 1
                    name, separator, argument = option.partition("=")
                    if name not in pure_codegen or (separator and not re.fullmatch(r"[-A-Za-z0-9_+.,=]+", argument)):
                        raise CheckoutInputIncomplete("Rust codegen option requires additional native/path inputs: " + name)
                    continue
                if flag in {"--cfg", "--check-cfg", "--cap-lints", "--edition", "-A", "-W", "-D", "-F"}:
                    if offset == len(tokens):
                        raise CheckoutInputIncomplete("Rust option has no value")
                    option = tokens[offset]
                    offset += 1
                elif flag.startswith(("--cfg=", "--check-cfg=", "--cap-lints=", "--edition=")):
                    flag, option = flag.split("=", 1)
                elif flag.startswith(("-A", "-W", "-D", "-F")) and len(flag) > 2:
                    flag, option = flag[:2], flag[2:]
                else:
                    raise CheckoutInputIncomplete("Rust flag requires an explicitly supported input closure: " + flag)
                patterns = {"--cfg": r'[A-Za-z_][A-Za-z0-9_]*(?:="[A-Za-z0-9_. -]*")?',
                            "--check-cfg": r'[A-Za-z0-9_"(), .-]+',
                            "--cap-lints": r"(?:allow|warn|deny|forbid)",
                            "--edition": r"(?:2015|2018|2021|2024)"}
                pattern = patterns.get(flag, r"[A-Za-z_][A-Za-z0-9_:]*")
                if not re.fullmatch(pattern, option):
                    raise CheckoutInputIncomplete("Rust option requires additional path/native inputs: " + flag)

        def cargo_config(path, active=None):
            path = plain(path)
            name = str(path)
            active = set() if active is None else active
            if name in active:
                raise ValueError("Cargo configuration include cycle")
            checksum = file(path)
            if checksum is None:
                return
            raw = path.read_bytes()
            if sha(raw) != checksum:
                raise ValueError("Cargo configuration changed during discovery")
            parsed = tomllib.loads(raw.decode("utf-8"))
            configuration.append({"kind": "cargo", "path": name, "sha256": checksum})
            includes = parsed.get("include", [])
            if not isinstance(includes, list):
                raise ValueError("Unsupported Cargo include configuration")
            for item in includes:
                value = item.get("path") if isinstance(item, dict) else item
                included = config_path(value, path.parent)
                cargo_config(included, active | {name})
            # Overrides can introduce an otherwise uncaptured compiler, source
            # or runner. This slice refuses them rather than certifying a gap.
            if parsed.get("paths"):
                raise ValueError("Cargo path overrides require explicit checkout inputs")
            if parsed.get("env"):
                raise CheckoutInputIncomplete("Cargo environment overrides require their actual native input closure")
            for source_value in parsed.get("source", {}).values():
                if isinstance(source_value, dict) and any(
                        source_value.get(key) for key in ("directory", "local-registry", "git")):
                    raise ValueError("Cargo source replacement requires explicit checkout inputs")
            build = parsed.get("build", {})
            if any(build.get(key) for key in ("rustc", "rustdoc", "rustc-wrapper", "rustc-workspace-wrapper")):
                raise CheckoutInputIncomplete("Cargo compiler overrides require an explicit toolchain closure")
            if build.get("target") is not None and build["target"] != compiler.get("host"):
                raise CheckoutInputIncomplete("Cross-target Cargo configuration requires its actual sysroot closure")
            for key in ("rustflags", "rustdocflags"):
                if key in build:
                    rust_flags(build[key])
            for target in parsed.get("target", {}).values():
                if isinstance(target, dict):
                    if any(target.get(key) for key in ("linker", "runner")):
                        raise CheckoutInputIncomplete("Cargo linker/runner overrides require an explicit toolchain closure")
                    for key in ("rustflags", "rustdocflags"):
                        if key in target:
                            rust_flags(target[key])

        def git_configuration(path):
            raw = self.git(path, "config", "--null", "--show-origin", "--includes", "--list").stdout
            items = raw.split(b"\0")
            if items[-1] != b"" or (len(items) - 1) % 2:
                raise ValueError("Git configuration origin format is incomplete")
            for offset in range(0, len(items) - 1, 2):
                origin = items[offset].decode("utf-8", "strict")
                if origin.startswith("file:"):
                    file(config_path(origin[5:], path))
                elif origin not in {"command line:", "standard input:"}:
                    raise ValueError("Unsupported effective Git configuration origin")
                key, _separator, value = items[offset + 1].partition(b"\n")
                key = key.decode("utf-8", "strict").casefold()
                if key in {"core.excludesfile", "core.attributesfile", "include.path"} or (
                        key.startswith("includeif.") and key.endswith(".path")):
                    # Active include files also appear among origin records.
                    # Relative include names use the including file's directory.
                    if origin.startswith("file:"):
                        including = config_path(origin[5:], path).parent
                    else:
                        including = path
                    file(config_path(value.decode("utf-8", "strict"), including))
            configuration.append({"kind": "git", "sourceRoot": str(path), "effectiveDigest": sha(raw)})

        def repository(path, snapshot):
            path = plain(path)
            monitors[str(path)] = None
            flattened = preparation.source_files(snapshot)
            for relative, checksum in flattened.items():
                add(path / relative, checksum)
            git_dir = plain(Path(self.text(path, "rev-parse", "--absolute-git-dir")))
            common = plain(Path(self.text(path, "rev-parse", "--path-format=absolute", "--git-common-dir")))
            git_roots.append({"sourceRoot": str(path), "gitDir": str(git_dir), "commonDir": str(common)})
            monitors[str(git_dir)] = monitors[str(common)] = None
            marker = path / ".git"
            if stat.S_ISREG(marker.lstat().st_mode):
                file(marker)
            for base in dict.fromkeys((git_dir, common)):
                for name in ("HEAD", "index", "config", "config.worktree", "packed-refs", "commondir",
                             "gitdir", "shallow", "info/exclude", "info/attributes", "info/grafts",
                             "objects/info/alternates"):
                    file(base / name)
                tree(base / "refs")
                tree(base / "reftable")
                for shared in sorted(base.glob("sharedindex.*")):
                    file(shared)
            # Ignored attribute/ignore controls still influence source discovery.
            directories = {path}
            for relative in flattened:
                directories.update(parent for parent in (path / relative).parents
                                   if parent == path or parent.is_relative_to(path))
            for directory in sorted(directories):
                file(directory / ".gitignore")
                file(directory / ".gitattributes")
            git_configuration(path)
            for relative, value in snapshot["files"].items():
                if isinstance(value, dict):
                    repository(path / relative, value["source"])

        root = plain(before["sourceRoot"])
        repository(root, before["source"])
        for external in before["externalRoots"]:
            repository(plain(external["path"]), external["source"])
        for relative in ("docs", "templates", ".codex/skills", ".codex/audits"):
            tree(root / relative)
        for relative in ("rust-toolchain", "rust-toolchain.toml"):
            file(root / relative)
        cache = plain(before["dependencyCache"]["home"])
        if not cache.is_dir():
            raise ValueError("Actual checkout requires an existing physical Cargo home")
        monitors[str(cache)] = None
        for name, checksum in before["dependencyCache"]["configSha256"].items():
            add(cache / name, checksum)
        for directory in (root, *root.parents, cache):
            for relative in (("config", "config.toml") if directory == cache else
                             (".cargo/config", ".cargo/config.toml")):
                cargo_config(directory / relative)
        for relative in ("registry/src", "git/checkouts", "registry/index", "registry/cache", "git/db"):
            tree(cache / relative)
        bound_git = getattr(self, "_checkout_git_binding", None)
        if not isinstance(bound_git, dict):
            raise CheckoutInputIncomplete("Actual checkout Git has no internal installed-tool binding")
        git_fact = checkout_guard_helper().capture_trusted_git_input()
        if git_fact != bound_git:
            raise CheckoutInputIncomplete("The originally bound installed Git identity changed")
        git_executable = plain(git_fact["path"])
        input_roots = (
            ("checkout", root), ("dependency-cache", cache),
            *(("external", plain(item["path"])) for item in before["externalRoots"]),
        )
        for role, source in input_roots:
            if git_executable == source or git_executable.is_relative_to(source):
                raise CheckoutInputIncomplete(
                    "Installed Git cannot be a checkout or dependency-cache input",
                    input_diagnostic={
                        "selectedGit": {"path": str(git_executable), "sha256": git_fact["sha256"]},
                        "matchedInput": {"role": role, "root": str(source)},
                    })
        add(git_executable, git_fact["sha256"])
        trusted_native_inputs = {str(git_executable): git_fact}
        # Explicit default absences can become effective later; monitoring also
        # covers parent creation. Existing effective origins are captured above.
        user = plain(Path.home())
        xdg = plain(Path(os.environ.get("XDG_CONFIG_HOME", str(user / ".config"))))
        for path in (user / ".gitconfig", xdg / "git/config", xdg / "git/ignore",
                     xdg / "git/attributes", git_executable.parent.parent / "etc/gitconfig",
                     git_executable.parent.parent / "etc/gitattributes"):
            file(path)
        for key in ("GIT_CONFIG_GLOBAL", "GIT_CONFIG_SYSTEM"):
            if os.environ.get(key):
                file(config_path(os.environ[key], root))
        for fact in compiler["plainFileFacts"].values():
            add(Path(fact["path"]), fact["sha256"])
            monitors[str(plain(Path(fact["path"]).parent))] = None
        native_environment = getattr(self, "_checkout_native_environment", None)
        if not isinstance(native_environment, dict) or not native_environment:
            raise CheckoutInputIncomplete("The actual trusted native environment is unavailable")

        def native_directories(variable, *, must_exist):
            raw = native_environment.get(variable)
            if not isinstance(raw, str) or not raw or any(char in raw for char in "\0\r\n"):
                raise CheckoutInputIncomplete("Native input directory list is missing: " + variable)
            entries = raw.split(";")
            while entries and not entries[-1].strip():
                entries.pop()
            result = []
            for value in entries:
                value = value.strip()
                if value.startswith('"') and value.endswith('"'):
                    value = value[1:-1]
                if (not value or '"' in value or not re.match(r"^[A-Za-z]:[\\/]", value)
                        or not Path(value).is_absolute()):
                    raise CheckoutInputIncomplete("Native inputs require literal absolute directories: " + variable)
                try:
                    directory = plain(value)
                    if directory == directory.parent:
                        raise ValueError("A native input cannot use an entire volume")
                    if directory.exists() and not directory.is_dir():
                        raise ValueError("Native input is not a directory")
                    if must_exist and not directory.is_dir():
                        raise ValueError("Required native input directory does not exist")
                except (ValueError, OSError) as error:
                    raise CheckoutInputIncomplete(variable + ": " + str(error)) from error
                if directory not in result:
                    result.append(directory)
            if not result:
                raise CheckoutInputIncomplete("Native input directory list is empty: " + variable)
            return result

        def native_tree(directory):
            try:
                tree(directory)
            except (ValueError, OSError) as error:
                raise CheckoutInputIncomplete("Cannot capture native input directory: " + str(directory)) from error

        native_inputs = {"environmentDigest": sha(encoded(native_environment)),
                         "headerLibraryDirectories": {}, "tools": {}, "toolDirectories": []}
        for variable in ("INCLUDE", "LIB", "LIBPATH"):
            directories = native_directories(variable, must_exist=True)
            native_inputs["headerLibraryDirectories"][variable] = [str(path) for path in directories]
            for directory in directories:
                monitors[str(directory)] = None
                native_tree(directory)
        search_directories = native_directories("PATH", must_exist=False)
        native_inputs["pathDirectories"] = [str(path) for path in search_directories]

        def native_tool(name, *, required):
            candidates, selected = [], None
            for directory in search_directories:
                path = directory / name
                try:
                    checksum = file(path)
                except (ValueError, OSError) as error:
                    raise CheckoutInputIncomplete("Cannot capture native executable: " + str(path)) from error
                candidates.append({"path": str(path), "sha256": checksum})
                if checksum is not None:
                    selected = path
                    break
            native_inputs["tools"][name] = {"search": candidates,
                                           "selected": str(selected) if selected is not None else None}
            if selected is None:
                if required:
                    raise CheckoutInputIncomplete("Trusted PATH has no required native tool: " + name)
                return
            directory = selected.parent
            if directory == directory.parent:
                raise CheckoutInputIncomplete("Native tool directory cannot be an entire volume")
            monitors[str(directory)] = None
            # Pin the selected installation directory, including adjacent DLLs,
            # localized resources and helper executables the tool may invoke.
            # Its SDK/VS installation ancestors are never scanned.
            native_tree(directory)
            if str(directory) not in native_inputs["toolDirectories"]:
                native_inputs["toolDirectories"].append(str(directory))

        for name in ("cl.exe", "link.exe", "lib.exe", "rc.exe", "mt.exe"):
            native_tool(name, required=True)
        for name in ("cmake.exe", "ninja.exe", "clang.exe", "clang-cl.exe",
                     "lld-link.exe", "llvm-lib.exe", "ml64.exe", "cvtres.exe"):
            native_tool(name, required=False)
        for variable, value in native_environment.items():
            if (variable in {"CC", "CXX", "AR", "RC", "CMAKE", "RUSTC_LINKER"}
                    or variable.startswith(("CC_", "CXX_", "AR_"))
                    or variable.startswith("CARGO_TARGET_") and variable.endswith("_LINKER")) and value:
                # Native override strings may contain an executable plus argv,
                # so the PATH-only closure cannot certify them implicitly.
                raise CheckoutInputIncomplete("Explicit native compiler override is not captured: " + variable)
        native_inputs["toolDirectories"].sort()
        policy_file = getattr(self, "_checkout_policy_file", None)
        if policy_file is not None:
            file(policy_file)
        workspace_inputs = before.get("cargoWorkspaceInputs")
        if workspace_inputs is not None:
            for path, checksum in workspace_inputs["files"].items():
                if file(path) != checksum:
                    raise ValueError("A plugin workspace input changed before native guard acquisition")
        if "batch" in self.payload:
            contract = self.payload["batch"]["contractRef"]
            if file(contract["path"]) != contract["sha256"]:
                raise ValueError("Original batch contract bytes changed")
        # Full source/Git/cache/toolchain roots already cover their children.
        # Keep only the narrow ancestor monitors needed for explicit absences.
        monitors = {name: values for name, values in monitors.items()
                    if values is None or not any(other != name and coverage is None
                        and Path(name).is_relative_to(Path(other))
                        for other, coverage in monitors.items())}
        roots = sorted(monitors)
        interests = {name: sorted(values) for name, values in monitors.items() if values is not None}
        value = {"files": dict(sorted(files.items())), "treeFacts": dict(sorted(trees.items())),
                 "gitRoots": sorted(git_roots, key=lambda item: item["sourceRoot"].casefold()),
                 "configuration": sorted(configuration, key=lambda item: encoded(item)),
                 "compiler": compiler, "nativeToolchain": native_inputs,
                 "trustedNativeInputs": trusted_native_inputs,
                 "watchRoots": roots, "watchInterests": interests}
        if workspace_inputs is not None:
            value["cargoWorkspaceInputs"] = workspace_inputs
        return value, roots

    @staticmethod
    def checkout_read_ignored(before, checkout_inputs, *, outputs=None):
        """Propagate exact noninput bookkeeping/output paths to every watcher."""
        root = plain(before["sourceRoot"])
        cache = plain(before["dependencyCache"]["home"])
        candidates = [root / relative for relative in (
            ".jenkins/state", ".jenkins/agent", ".jenkins/runtime", ".jenkins/logs",
            ".jenkins/cache", ".jenkins/tmp", ".jenkins/builds", ".jenkins/records",
            ".jenkins/jenkins_home", ".jenkins/worktrees",
            # These narrow paths remain safe when state/runtime contains an input.
            ".jenkins/state/coordination.sqlite3", ".jenkins/state/coordination.sqlite3-wal",
            ".jenkins/state/coordination.sqlite3-shm", ".jenkins/state/coordination.sqlite3-journal",
            ".jenkins/state/locks")]
        candidates.extend(cache / relative for relative in (
            ".package-cache", ".package-cache-mutate", ".global-cache",
            ".global-cache-wal", ".global-cache-shm", ".global-cache-journal", "registry/CACHEDIR.TAG"))
        candidates.extend(plain(value) for name, value in (outputs or {}).items()
                          if name in {"build", "reports", "tmp"})
        inputs = [plain(name) for name in checkout_inputs["files"]]
        safe = [plain(path) for path in candidates
                if not any(value == path or value.is_relative_to(path) for value in inputs)]
        result = {}
        for name in checkout_inputs["watchRoots"]:
            watch_root = plain(name)
            result[name] = sorted({path.relative_to(watch_root).as_posix() for path in safe
                                   if path != watch_root and path.is_relative_to(watch_root)})
        return result

    def checkout_git_read_conflict(self, *, connection=None, unknown_only=False):
        """Durable unknown readers survive the death of their live guard."""
        for row in self.state.list("management_checkout_read", connection=connection):
            value = row["payload"]
            if (value.get("repositoryId") == self.repo_id
                    and value.get("status") in ({"unknown"} if unknown_only else {"active", "unknown"})):
                return row
        return None

    def acquire_checkout_guard(self, before, checkout_inputs):
        module = checkout_guard_helper()
        return module.CheckoutReadGuard(
            plain(before["sourceRoot"]), files=checkout_inputs["files"],
            watch_roots=checkout_inputs["watchRoots"],
            watch_interests=checkout_inputs["watchInterests"],
            trusted_native_inputs=checkout_inputs["trustedNativeInputs"],
            ignored_activity=self.checkout_read_ignored(
                before, checkout_inputs, outputs=getattr(self, "_checkout_outputs", None))).acquire()

    def checkout_unknown(self, reason, *, message=None):
        """Retain the original read, writer and resource; never replay a claim."""
        context = self._checkout_context
        owner = context["owner"]
        read = self.state.get("management_checkout_read", owner)
        if read:
            self.state.put("management_checkout_read", owner, {
                **read["payload"], "status": "unknown", "continuousReadProven": False,
                "reasonCode": reason, "message": message, "unknownAt": time.time()},
                expected_version=read["version"])
        for domain in ("execution", "execution_launch"):
            row = self.state.get(domain, owner)
            if row:
                self.state.put(domain, owner, {**row["payload"], "status": "unknown",
                    "reasonCode": reason}, expected_version=row["version"])
        # No finish/close: an unproven native tree may still read these inputs.
        # The durable unknown row also blocks managed Git after this process dies.
        return {"status": "unknown", "reasonCode": reason, "message": message,
                "validationInputPolicy": "actual-checkout", "sourceRoot": str(context["root"]),
                "executionRoot": str(context["root"]), "sourceDigest": context["before"]["sourceDigest"],
                "checkoutInputsDigest": context["inputsDigest"], "checkoutReadRef": context.get("readRef"),
                "originalReservationRetained": True, "originalWriterRetained": True,
                "continuousReadProven": False, "formalAcceptance": False, "m8Accepted": False}

    def checkout_final_inputs(self):
        """Observe every original input before ending the continuous read interval."""
        context = self._checkout_context
        root, before, guard = context["root"], context["before"], context["guard"]
        after, inputs_after, capture_error = None, None, None
        observation = {"complete": False, "inputsUnchanged": False, "originalEngineUnchanged": False,
                       "dependencyCacheUnchanged": False, "externalInputsUnchanged": False}
        try:
            observed_root, after, observation = self.validation_observation(before)
            if observed_root != root:
                raise ValueError("Actual checkout source root changed")
            compiler = context["preparation"].trusted_toolchain_facts(
                context["trustedToolchain"], context["helper"]._file_fact,
                environment=context["toolchainEnvironment"])
            inputs_after, _roots = self.checkout_read_inputs(
                after, compiler, context["preparation"], context["helper"])
        except Exception as error:
            capture_error = str(error)
        stability_error = None
        try:
            guard.assert_stable()
        except Exception as error:
            stability_error = str(error)
        finished = guard.finish()
        guard_ref = write_json(context["report"] / "checkout-guard.json", finished)
        exact = before == after and context["inputs"] == inputs_after
        proof = {"validationInputPolicy": "actual-checkout", "capturedInputVerified": True,
                 "inputsUnchanged": before == after, "checkoutInputsUnchanged": context["inputs"] == inputs_after,
                 "originalEngineUnchanged": observation.get("originalEngineUnchanged") is True,
                 "externalInputsUnchanged": observation.get("externalInputsUnchanged") is True,
                 # This includes actual dependency bytes, beyond legacy config facts.
                 "dependencyCacheUnchanged": observation.get("dependencyCacheUnchanged") is True and exact,
                 "sourceNoWrites": finished.get("sourceNoWrites") is True,
                 "membershipStable": finished.get("membershipStable") is True,
                 "sourceRoot": str(root), "executionRoot": str(root), "sourceDigest": before["sourceDigest"],
                 "checkoutInputsDigest": context["inputsDigest"],
                 "expectedInputsDigest": context["guardFacts"]["expectedInputsDigest"],
                 "checkoutReadRef": context["readRef"], "checkoutGuardRef": guard_ref,
                 "checkoutGuard": finished}
        final = {"after": after, "checkoutInputsAfter": inputs_after, "observation": observation,
                 "inputError": capture_error, "guardError": stability_error, "proof": proof}
        final_ref = write_json(context["report"] / "input-final.json", final)
        context.update(final=final, finalRef=final_ref, proof=proof)
        if (not validation_inputs_verified(proof, context["requestDocument"])
                or capture_error or stability_error):
            return False
        read = self.state.get("management_checkout_read", context["owner"])
        if (not read or read["payload"].get("checkoutReadRef") != context["readRef"]
                or read["payload"].get("status") != "active"):
            raise ValueError("Original checkout read reservation changed")
        guard.close()
        self.state.put("management_checkout_read", context["owner"], {
            **read["payload"], "status": "released", "pinsReleased": True,
            "continuousReadProven": True, "checkoutGuardRef": guard_ref, "finalObservation": final_ref,
            "nativeProofRefs": list(context["nativeProofRefs"]), "releasedAt": time.time()},
            expected_version=read["version"])
        context["readReleased"] = True
        return True

    def verify_checkout_product_delivery(self, result):
        product = self.payload.get("checkoutProduct")
        schema = product.get("schemaVersion") if isinstance(product, dict) else None
        helper = checkout_product_helper(self.config, required_schema=schema)
        report = plain(self.outputs() / "reports" / self.key)
        checkpoint = self.batch_checkpoint()
        saved = read_reference(checkpoint["receipt"], expected_path=report / "receipt.json",
                               max_bytes=SOURCE_EVIDENCE_MAX_BYTES)
        envelope = {"operationKey", "requestId", "operationId", "sessionId", "driverDigest", "runtimeOperationId"}
        if saved != {k: v for k, v in result.items() if k not in envelope} or saved.get("status") != "passed":
            raise ValueError("Original editor delivery receipt changed")
        request = read_reference(saved["request"], expected_path=report / "request.json",
                                 max_bytes=SOURCE_EVIDENCE_MAX_BYTES)
        helper.validate_admission_capture(request.get("payload", {}), request.get("source"), self.repo)
        guard = read_reference(saved["checkoutGuardRef"], expected_path=report / "checkout-guard.json",
                               max_bytes=SOURCE_EVIDENCE_MAX_BYTES)
        if (saved.get("checkoutGuard") != guard or not validation_inputs_verified(saved, request)
                or {k: v for k, v in request.get("payload", {}).items() if k != "observationOnly"}
                   != {k: v for k, v in self.payload.items() if k != "observationOnly"}):
            raise ValueError("Original editor delivery source guard changed")
        owner = "management-" + self.key
        reader = self.state.get("management_checkout_read", owner)
        if (not reader or reader["payload"].get("status") != "released"
                or reader["payload"].get("pinsReleased") is not True
                or reader["payload"].get("continuousReadProven") is not True
                or reader["payload"].get("checkoutGuardRef") != saved["checkoutGuardRef"]):
            raise ValueError("Original editor checkout read closure changed")
        claims = self.batch_claims()
        if len(claims) != 2 or any(item["status"] != "complete" for item in claims):
            raise ValueError("Original editor delivery lacks both native steps")
        proofs = [self.batch_step_proof(item) for item in claims]
        if [item[3] for item in proofs] != saved["nativeProofRefs"]:
            raise ValueError("Original editor Cargo native evidence changed")
        packaging = saved.get("packagingNativeProofs")
        if not isinstance(packaging, list) or len(packaging) != 3:
            raise ValueError("Original editor delivery lacks owned CRT verification")
        for ordinal, item in enumerate(packaging, 1):
            native = read_reference(item["nativeTerminal"],
                expected_path=report / "packaging" / ("%04d" % ordinal) / "native-terminal.json")
            proof = native_terminal_proof(native, item["nativeTerminal"])
            key = owner + "-package-%04d-native" % ordinal
            row = self.state.get("native_job", key)
            recorded = row["payload"] if row else {}
            if (item.get("nativeProofRef") != key or item.get("command") != native["command"]
                    or item.get("terminalProof") != proof or proof["complete"] is not True
                    or proof["processExitCode"] != 0 or native["cwd"] != str(self.repo)
                    or recorded.get("owner") != owner or recorded.get("status") != "terminal"
                    or recorded.get("role") != "management_editor_package"
                    or recorded.get("requestId") != self.request or recorded.get("operationId") != self.operation
                    or recorded.get("environmentDigest") != request["stepEnvironmentDigests"][-1]
                    or recorded.get("nativeEvidence") != native or recorded.get("completeProof") != proof
                    or recorded.get("terminalProof") != proof):
                raise ValueError("Original editor packaging native evidence changed")
        helper.verify_published_product(self.repo, saved["checkoutProduct"],
                                        self.outputs() / "products" / self.key / "bundle")

    def checkout_batch_reconcile(self, checkpoint, resources, report):
        """Only already closed original batches may reconcile without a live guard."""
        owner = "management-" + self.key
        read = self.state.get("management_checkout_read", owner)
        if (checkpoint.get("validationInputPolicy") != "actual-checkout" or checkpoint.get("stage") != "complete"
                or not read or read["payload"].get("status") != "released"
                or read["payload"].get("pinsReleased") is not True
                or read["payload"].get("continuousReadProven") is not True):
            return {"status": "unknown", "reasonCode": "checkout_guard_continuity_unproven",
                    "validationInputPolicy": "actual-checkout", "originalReservationRetained": True,
                    "formalAcceptance": False}
        result = read_reference(checkpoint["receipt"], expected_path=report / "receipt.json",
                                max_bytes=SOURCE_EVIDENCE_MAX_BYTES)
        request = read_reference(checkpoint["request"], expected_path=report / "request.json",
                                 max_bytes=SOURCE_EVIDENCE_MAX_BYTES)
        guard = read_reference(result["checkoutGuardRef"], expected_path=report / "checkout-guard.json",
                               max_bytes=SOURCE_EVIDENCE_MAX_BYTES)
        if (result.get("checkoutGuard") != guard or not validation_inputs_verified(result, request)
                or read["payload"].get("checkoutGuardRef") != result["checkoutGuardRef"]
                or read["payload"].get("checkoutInputsDigest") != result["checkoutInputsDigest"]):
            raise ValueError("Original completed checkout batch proof changed")
        claims = self.batch_claims()
        expected_count = len(self.payload["batch"]["recipes"])
        if self.payload.get("checkoutProduct") and result.get("status") == "failed":
            expected_count = result.get("completedRecipeCount")
            if type(expected_count) is not int or not 1 <= expected_count <= 2:
                raise ValueError("Original failed product step count changed")
        if (len(claims) != expected_count
                or any(claim["status"] != "complete" for claim in claims)):
            raise ValueError("Original completed checkout batch has a missing native step")
        proofs = [self.batch_step_proof(claim) for claim in claims]
        if [item[3] for item in proofs] != result["nativeProofRefs"]:
            raise ValueError("Original batch Native proof bindings changed")
        if self.payload.get("checkoutProduct") and result.get("status") == "passed":
            self.verify_checkout_product_delivery(result)
        reservation = self.state.get("resource_reservation", checkpoint["reservationKey"])
        if reservation and reservation["payload"].get("status") == "active":
            resources.release(checkpoint["reservationKey"], writer_key=checkpoint["writer"],
                              native_proof_ref=proofs[-1][3])
        return result


    def checkout_input_conflict_result(self, error, *, owner, writer, guard, admitted):
        if (admitted is not None or getattr(self, "_checkout_context", None) is not None
                or getattr(self, "_validation_admission", None) is not None):
            return None
        try:
            with self.state.connect() as connection:
                claims = [
                    self.state.get("management_checkout_read", owner, connection=connection),
                    self.state.get("execution", owner, connection=connection),
                    self.state.get("execution_launch", owner, connection=connection),
                    self.state.get("execution_host", owner, connection=connection),
                    self.state.get("management_prelaunch", owner, connection=connection),
                    self.state.get("resource_reservation", owner + ":" + owner, connection=connection),
                    self.state.get("pool_writer_hold", writer, connection=connection),
                ]
                native_claims = any(
                    row["payload"].get("owner") == owner
                    or row["payload"].get("executionId") == owner
                    or row["key"] == owner + "-native"
                    or row["key"].startswith(owner + "-package-")
                    for row in self.state.list("native_job", connection=connection)
                )
        except Exception:
            return None
        if any(claims) or native_claims:
            return None
        if guard is not None:
            guard.close()
        return {
            "status": "conflict",
            "reasonCode": error.code,
            "message": str(error),
            "conflictingInput": {
                "path": error.path,
                "previousPath": error.previous_path,
                "beforeSha256": error.before_sha256,
                "afterSha256": error.after_sha256,
            },
            "compileStarted": False,
            "formalAcceptance": False,
        }

    def validate_checkout(self, *, batch=None):
        """Run real checkout Cargo with no sealed input or materialized source."""
        from tools.jenkins.resources.capacity import Capacity, ResourceManager
        from tools.jenkins.resources.paths import canonical_build_root, physical_path_under
        from tools.jenkins.workflow.planning import _environment_policy
        from tools.jenkins.workflow import pools
        from tools.jenkins.state.locks import process_lock
        from tools.jenkins.contracts import JenkinsError
        if self.payload.get("sourceView") != "checkout" or self.payload.get("worktreeId"):
            raise ValueError("Actual checkout execution requires its canonical validation-only root")
        guard_checksum = self.config.get("files", {}).get("checkout_guard.py")
        if (not isinstance(guard_checksum, str) or not re.fullmatch(r"[0-9a-f]{64}", guard_checksum)
                or sha(plain(Path(__file__).with_name("checkout_guard.py")).read_bytes()) != guard_checksum):
            raise ValueError("Actual checkout requires its capsule's hash-verified continuous read guard")
        product = self.payload.get("checkoutProduct")
        product_schema = product.get("schemaVersion") if isinstance(product, dict) else None
        product_helper = (checkout_product_helper(self.config, required_schema=product_schema)
                          if product is not None else None)
        root, owner = self.validation_root(), "management-" + self.key
        if root != plain(self.repo):
            raise ValueError("Actual checkout differs from the canonical repository")
        recipes = batch["recipes"] if batch is not None else [self.validation_recipe()]
        if batch is not None and batch["reuseValidation"] is not None:
            raise ValueError("Sealed worktree donor cannot become an actual checkout input")
        coverage = batch["coverage"] if batch is not None else {
            name: value for name, value in recipes[0].items() if name != "command"}
        build_root, output = self.build_root(), self.outputs()
        approved = canonical_build_root(build_root)
        if (any(recipe.get("manifestPath") == "zircon_plugins/Cargo.toml" for recipe in recipes)
                and approved.path != plain(Path("E:/cargo-targets"))):
            raise ValueError("The plugin recipe requires the physical E:/cargo-targets build root")
        report = plain(output / "reports" / self.key)
        policy = self.state.get("resource_policy", "default")["payload"]
        resources = ResourceManager(self.state, Capacity(
            policy["cpuBudget"], policy["memoryBudget"], policy["diskBudget"]))
        checkpoint = self.batch_checkpoint() if batch is not None else {}
        if checkpoint.get("stage") == "complete":
            return self.checkout_batch_reconcile(checkpoint, resources, report)
        old_read = self.state.get("management_checkout_read", owner)
        if (old_read or self.state.get("execution", owner) or self.state.get("execution_launch", owner)
                or report.exists() or checkpoint.get("sealed") or checkpoint.get("prepared")
                or checkpoint.get("request")):
            return {"status": "unknown", "reasonCode": "checkout_guard_continuity_unproven",
                    "validationInputPolicy": "actual-checkout", "originalReservationRetained": True,
                    "originalWriterRetained": True, "formalAcceptance": False}
        environment, trusted, policy_file = self.validation_environment()
        self._checkout_policy_file = policy_file
        self._checkout_native_environment = dict(environment)
        preparation = source_helper()
        helper_path = client_path(self.config, self.repo, "tools/dev/local_cargo.py")
        spec = importlib.util.spec_from_file_location("jenkins_managed_checkout_cargo", helper_path)
        helper = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(helper)
        parallelism = max(max(recipe["jobs"], recipe["testThreads"] if recipe["verb"] == "test"
                              and not recipe["noRun"] else 1) for recipe in recipes)
        amount = Capacity(parallelism, 2 * parallelism * 1024**3, 4 * 1024**3)
        guard, admitted = None, None
        try:
            # This same mutex serializes admission with every managed Git write.
            with process_lock(self.repo / ".jenkins/state/locks/management-git.lock",
                              reason="management_git_busy", timeout=0):
                conflict = self.checkout_git_read_conflict(unknown_only=True)
                if conflict:
                    return {"status": "waiting", "reasonCode": "checkout_source_busy",
                            "blockingReadOwner": conflict["key"], "formalAcceptance": False}
                self._checkout_git_binding = checkout_guard_helper().capture_trusted_git_input()
                try:
                    root, before = self.validation_with_cache()
                    if product_helper is not None:
                        product_helper.validate_admission_capture(self.payload, before, self.repo)
                except SourceInputMismatch as error:
                    return {"status": "conflict", "reasonCode": "preparation-conflict", "message": str(error),
                            "preparationStarted": False, "compileStarted": False, "formalAcceptance": False,
                            **error.diagnostics}
                captured = self.state.get("management_source", self.key)
                if captured and captured["payload"] != before:
                    raise ValueError("Original queued checkout source changed")
                toolchain_binding = dict(trusted)
                toolchain_environment = dict(environment)
                compiler = preparation.trusted_toolchain_facts(
                    toolchain_binding, helper._file_fact, environment=toolchain_environment)
                for tool in ("cargo", "rustc"):
                    if (compiler["binaries"][tool]["path"] != trusted[tool + "Path"]
                            or compiler["binaries"][tool]["sha256"] != trusted[tool + "Sha256"]):
                        raise ValueError("Actual compiler differs from the trusted installed toolchain")
                cache = plain(before["dependencyCache"]["home"])
                if batch is not None:
                    pool, _configuration = self.batch_pool(
                        recipes, preparation, trusted, environment, root, before["dependencyCache"])
                else:
                    recipe = recipes[0]
                    compatibility = {**environment, **cargo_manifest_environment(root, recipe),
                        **({"cargoAllFeatures": True} if recipe["allFeatures"] else {}),
                        "sourceCargoConfiguration": {
                            name: value for name, value in before["source"]["files"].items()
                            if name.casefold() in {".cargo/config", ".cargo/config.toml"}}}
                    pool = preparation.preparation_key(self.repo_id, trusted, compatibility,
                        recipe["packages"], recipe["features"], recipe["profile"],
                        recipe["noDefaultFeatures"], before["dependencyCache"],
                        ["zr_vm"] if before["externalRoots"] else [])
                writer = "management-pool-" + pool
                target = physical_path_under(approved, output / "pools" / pool / "target")
                scratch = plain(report / "tmp")
                self._checkout_outputs = {"build": target.parent, "cache": cache, "reports": report, "tmp": scratch}
                if product_helper is not None:
                    self._checkout_outputs["product"] = physical_path_under(approved, output / "products" / self.key)
                inputs, _watch_roots = self.checkout_read_inputs(before, compiler, preparation, helper)
                inputs_digest = sha(encoded(inputs))
                guard = self.acquire_checkout_guard(before, inputs)
                # Capture names/HEAD/index and every byte again after all native
                # pins and monitors have been acquired, before any process claim.
                current_root, current, initial_observation = self.validation_observation(before)
                current_compiler = preparation.trusted_toolchain_facts(
                    toolchain_binding, helper._file_fact, environment=toolchain_environment)
                current_inputs, _ = self.checkout_read_inputs(
                    current, current_compiler, preparation, helper)
                guard.assert_stable()
                if current_root != root or current != before or current_inputs != inputs:
                    raise ValueError("Checkout inputs changed during native read admission")
                guard_facts = guard.facts()
                self.begin_validation_intent(resources, owner, writer, root, report)
                resources.queue(owner, request_id=owner, amount=amount, build_root=build_root,
                                heavy_writer=True, writer_key=writer)
                if helper.free_bytes(approved.path) < policy.get("diskReserveBytes", 35 * 1024**3) + amount.disk_bytes:
                    guard.close()
                    return {"status": "waiting", "reasonCode": "resource_capacity", "sourceDigest": before["sourceDigest"],
                            "formalAcceptance": False}
                _inventory, generation = resources.scan_inventory(build_root)
                try:
                    admitted = resources.admit(owner, amount, inventory_generation=generation, writer_key=writer,
                                              request_id=owner, build_root=build_root, heavy_writer=True)
                except JenkinsError as error:
                    if error.retryable or error.code in {"pool_writer_active", "pool_cache_maintenance_active"}:
                        guard.close()
                        return {"status": "waiting", "reasonCode": error.code,
                                "sourceDigest": before["sourceDigest"], "formalAcceptance": False}
                    raise
                if admitted is None:
                    guard.close()
                    return {"status": "waiting", "reasonCode": "resource_capacity",
                            "sourceDigest": before["sourceDigest"], "formalAcceptance": False}
                self._checkout_context = {"owner": owner, "writer": writer, "resources": resources,
                    "reservationKey": admitted["key"], "root": root, "report": report, "target": target,
                    "before": before, "inputs": inputs, "inputsDigest": inputs_digest, "guard": guard,
                    "guardFacts": guard_facts, "helper": helper, "preparation": preparation,
                    "trustedToolchain": toolchain_binding,
                    "toolchainEnvironment": toolchain_environment,
                    "nativeProofRefs": [], "launched": False, "readReleased": False}
                if admitted["key"] != owner + ":" + owner:
                    raise ValueError("Original checkout reservation identity changed")
                # Register artifact ownership before creating compiler/TEMP outputs.
                self.artifact_manifest(root, self._checkout_outputs)
                if not captured:
                    self.state.put("management_source", self.key, before, expected_version=0)
                binding = {"schemaVersion": 1, "validationInputPolicy": "actual-checkout",
                    "owner": owner, "operationKey": self.key, "repositoryId": self.repo_id,
                    "sessionId": self.session, "requestId": self.request, "operationId": self.operation,
                    "driverDigest": self.config["driverDigest"], "runtimeOperationId": self.config["runtimeOperationId"],
                    "inputDigest": self.input_digest, "sourceRoot": str(root), "sourceDigest": before["sourceDigest"],
                    "checkoutInputsDigest": inputs_digest, "expectedInputsDigest": guard_facts["expectedInputsDigest"],
                    "fileIdentityDigest": guard_facts["fileIdentityDigest"],
                    "reservationKey": admitted["key"], "preparationKey": writer, "watchRoots": inputs["watchRoots"],
                    "watchInterests": inputs["watchInterests"], "inputCopiesCreated": 0}
                read_ref = write_json(report / "checkout-read.json", {"binding": binding, "guard": guard_facts})
                self._checkout_context["readRef"] = read_ref
                self.state.put("management_checkout_read", owner, {
                    **binding, "status": "active", "pinsReleased": False, "continuousReadProven": False,
                    "checkoutReadRef": read_ref, "guardFacts": guard_facts, "acquiredAt": time.time()}, expected_version=0)
                if batch is not None:
                    self.batch_checkpoint(pool=pool, writer=writer, reservationKey=admitted["key"],
                        validationInputPolicy="actual-checkout", sourceRoot=str(root), executionRoot=str(root),
                        report=str(report), buildRoot=build_root, capturedSourceDigest=before["sourceDigest"],
                        checkoutInputsDigest=inputs_digest, checkoutReadRef=read_ref,
                        coverageDigest=sha(encoded(coverage)), stage="guarded")
            environment.setdefault("SYSTEMROOT", os.environ.get("SYSTEMROOT", r"C:\Windows"))
            environment.update({"CARGO_TARGET_DIR": str(target), "CARGO_BUILD_BUILD_DIR": str(target.parent / "build"),
                "CARGO_HOME": str(cache), "TEMP": str(scratch), "TMP": str(scratch), "TMPDIR": str(scratch),
                "RUSTC": trusted["rustcPath"], "RUSTDOC": compiler["binaries"]["rustdoc"]["path"],
                "RUSTC_WRAPPER": "", "RUSTC_WORKSPACE_WRAPPER": "", "PYTHONDONTWRITEBYTECODE": "1",
                "CARGO_ENCODED_RUSTFLAGS": "", "RUSTFLAGS": "", "RUSTDOCFLAGS": ""})
            commands, environments = [], []
            for ordinal, recipe in enumerate(recipes):
                if batch is None:
                    raw = list(recipe["command"])
                elif product_helper is not None:
                    raw = ["cargo", *self.payload["batch"]["recipes"][ordinal]["cargoArguments"]]
                else:
                    raw = list(self.payload["batch"]["recipes"][ordinal]["cargoArguments"])
                    separator = raw.index("--") if "--" in raw else len(raw)
                    controls = ["--message-format=json"]
                    if not any(value == "--jobs" or value.startswith("--jobs=") or value.startswith("-j")
                               for value in raw[:separator]):
                        controls += ["--jobs", str(recipe["jobs"])]
                    raw = [*raw[:separator], *controls, *raw[separator:]]
                if product_helper is not None:
                    command = [trusted["cargoPath"], *raw[1:]]
                else:
                    command = helper.prepare_command(raw, target, locked=recipe["locked"])
                    command[0] = trusted["cargoPath"]
                commands.append(command)
                environments.append({**environment, "CARGO_NET_OFFLINE": "true" if recipe["offline"] else "false",
                                     **(batch["stepEnvironments"][ordinal] if batch is not None else {})})
            environment_digests = [sha(encoded(value)) for value in environments]
            for path in (target, target.parent / "build"):
                path.mkdir(parents=True, exist_ok=True)
            task_scratch = self.task_tmp(environment=environments[0])
            execution_identity = sha(encoded({"validationInputPolicy": "actual-checkout",
                "sourceDigest": before["sourceDigest"], "checkoutInputsDigest": inputs_digest,
                "commands": commands, "cwd": str(root), "environmentDigests": environment_digests,
                "coverage": coverage, "driverDigest": self.config["driverDigest"],
                "runtimeOperationId": self.config["runtimeOperationId"]}))
            execution = {**binding, "executionId": owner, "kind": "management_owned_cargo", "status": "preparing",
                "command": commands[0], "environmentDigest": environment_digests[0], "sourceRoot": str(root),
                "executionRoot": str(root), "checkoutReadRef": read_ref, "batch": batch is not None,
                "executionIdentity": execution_identity, "formalAcceptance": False}
            self.state.put("execution", owner, execution, expected_version=0)
            pools._hold(self.state, owner, writer)
            self.artifact_manifest(root, self._checkout_outputs, task_scratch=task_scratch)
            request_document = {"payload": self.payload, "inputDigest": self.input_digest, "source": before,
                "sourceRoot": str(root), "executionRoot": str(root), "command": commands[0], "commands": commands,
                "target": str(target), "resource": amount.__dict__, "coverage": coverage,
                "validationInputPolicy": "actual-checkout", "capturedInputVerified": True,
                "checkoutInputs": inputs, "checkoutInputsDigest": inputs_digest,
                "expectedInputsDigest": guard_facts["expectedInputsDigest"], "checkoutReadRef": read_ref,
                "inputObservation": initial_observation, "executionIdentity": execution_identity,
                "taskScratch": task_scratch, "environmentDigest": environment_digests[0],
                "stepEnvironmentDigests": environment_digests,
                "temporaryEnvironment": {key: environment[key] for key in ("TEMP", "TMP", "TMPDIR")}}
            request_ref = write_json(report / "request.json", request_document)
            context = self._checkout_context
            context.update(requestDocument=request_document, requestRef=request_ref)
            if batch is not None:
                self.batch_checkpoint(request=request_ref, stage="prepared")
            completed, native, native_ref, proof_id = [], None, None, None
            product, product_error, product_native = None, None, []
            packaging_proofs = []
            def packaging_run(command, **options):
                shell = Path(environments[-1].get("SYSTEMROOT") or environments[-1].get("SystemRoot") or r"C:\Windows") / "System32/WindowsPowerShell/v1.0/powershell.exe"
                if (not isinstance(command, list) or len(command) != 5
                        or plain(command[0]) != plain(shell)
                        or command[1:4] != ["-NoProfile", "-NonInteractive", "-Command"]
                        or not command[4].startswith("$s = Get-AuthenticodeSignature -LiteralPath ")
                        or options.get("env") != environments[-1]
                        or options.get("capture_output") is not True or options.get("check") is not False):
                    raise ValueError("Packaging may only execute the pinned CRT signature check")
                guard.assert_stable()
                pools._hold(self.state, owner, writer)
                evidence = report / "packaging" / ("%04d" % (len(packaging_proofs) + 1))
                evidence.mkdir(parents=True, exist_ok=False)
                package_native = helper._run_owned_command(command, root, environments[-1], evidence, 120)
                package_ref = write_json(evidence / "native-terminal.json", package_native)
                proof = native_terminal_proof(package_native, package_ref)
                if (proof["complete"] is not True or package_native["command"] != command
                        or package_native["cwd"] != str(root)):
                    raise RuntimeError("checkout_packaging_termination_unproven")
                package_id = owner + "-package-%04d-native" % (len(packaging_proofs) + 1)
                self.state.put("native_job", package_id, {
                    "owner": owner, "executionId": owner, "role": "management_editor_package",
                    "status": "terminal", "terminalProof": proof, "completeProof": proof,
                    "nativeEvidence": package_native, "pid": package_native["pid"],
                    "creationTime": package_native["creationTime"], "birthToken": package_native["creationTime"],
                    "executable": proof["identity"]["executable"], "commandDigest": proof["identity"]["commandDigest"],
                    "requestId": self.request, "operationId": self.operation, "taskScratch": task_scratch,
                    "environmentDigest": environment_digests[-1], "formalAcceptance": False}, expected_version=0)
                packaging_proofs.append({"nativeProofRef": package_id, "nativeTerminal": package_ref,
                                         "command": command, "terminalProof": proof})
                guard.assert_stable()
                return subprocess.CompletedProcess(command, package_native["rootExitCode"],
                    (evidence / "stdout.bin").read_bytes(), (evidence / "stderr.bin").read_bytes())
            for ordinal, command in enumerate(commands, 1):
                guard.assert_stable()
                pools._hold(self.state, owner, writer)
                read = self.state.get("management_checkout_read", owner)
                if not read or read["payload"].get("status") != "active" or read["payload"].get("checkoutReadRef") != read_ref:
                    raise ValueError("Actual native launch lost its original read reservation")
                if batch is not None:
                    original = self.payload["batch"]["recipes"][ordinal - 1]
                    step_report = report / "steps" / ("%04d" % ordinal)
                    step_request = {"parentRequest": request_ref, "inputDigest": self.input_digest,
                        "ordinal": ordinal, "recipe": original, "command": command, "cwd": str(root),
                        "timeoutSeconds": original["ownedNativeTimeoutSeconds"], "taskScratch": task_scratch,
                        "validationInputPolicy": "actual-checkout", "checkoutInputsDigest": inputs_digest,
                        "expectedInputsDigest": guard_facts["expectedInputsDigest"], "checkoutReadRef": read_ref,
                        "environmentDigest": environment_digests[ordinal - 1],
                        "temporaryEnvironment": request_document["temporaryEnvironment"]}
                    step_ref = write_json(step_report / "request.json", step_request)
                    claim = self.claim_batch_step(ordinal, command, root, step_ref)
                    timeout = original["ownedNativeTimeoutSeconds"]
                else:
                    step_report, timeout = report, 21600
                    self.claim_validation_launch(owner, command, root, request_ref)
                context["launched"] = True
                native = helper._run_owned_command(command, root, environments[ordinal - 1], step_report, timeout)
                native_ref = write_json(step_report / "native-terminal.json", native)
                terminal_proof = native_terminal_proof(native, native_ref)
                if native["command"] != command or native["cwd"] != str(root) or terminal_proof["complete"] is not True:
                    return self.checkout_unknown("checkout_native_termination_unproven")
                if batch is not None:
                    step, native, native_ref, proof_id = self.batch_step_proof(claim, persist=True)
                    completed.append(step)
                else:
                    proof_id = owner + "-native"
                    self.state.put("native_job", proof_id, {
                        "owner": owner, "executionId": owner, "role": "management_owned_cargo", "status": "terminal",
                        "terminalProof": terminal_proof, "completeProof": terminal_proof,
                        "nativeEvidence": native, "pid": native["pid"], "creationTime": native["creationTime"],
                        "birthToken": native["creationTime"], "executable": terminal_proof["identity"]["executable"],
                        "commandDigest": terminal_proof["identity"]["commandDigest"], "requestId": self.request,
                        "operationId": self.operation, "taskScratch": task_scratch,
                        "environmentDigest": environment_digests[0], "validationInputPolicy": "actual-checkout",
                        "sourceDigest": before["sourceDigest"], "checkoutInputsDigest": inputs_digest,
                        "checkoutReadRef": read_ref, "formalAcceptance": False}, expected_version=0)
                context["nativeProofRefs"].append(proof_id)
                guard.assert_stable()
                if product_helper is not None:
                    product_native.append({**terminal_proof, "command": command, "cwd": str(root)})
                    if step["status"] != "passed":
                        break
            all_recipes_executed = len(completed) == len(recipes) if batch is not None else True
            if product_helper is not None and all_recipes_executed and all(x["status"] == "passed" for x in completed):
                try:
                    product = product_helper.stage_and_publish(root, target, output, self.key,
                        product_helper.source_inventory_from_capture(before), product_native, guard, environments[-1], packaging_run)
                except (ValueError, OSError) as error:
                    product_error = str(error)
            if not self.checkout_final_inputs():
                return self.checkout_unknown("checkout_input_stability_unproven",
                    message=context["final"].get("inputError") or context["final"].get("guardError"))
            proof = context["proof"]
            if batch is not None:
                self.batch_checkpoint(finalObservation=context["finalRef"], checkoutGuardRef=proof["checkoutGuardRef"],
                                      stage="closed", allRecipesExecuted=all_recipes_executed)
            task_cache = self.task_tmp(native=native, native_ref=native_ref, proof_id=proof_id)
            status = ("passed" if (all(step["status"] == "passed" for step in completed)
                        if batch is not None else native["rootExitCode"] == 0) else "failed")
            if product_helper is not None and (product is None or product_error):
                status = "failed"
            result = {"status": status, "request": request_ref, "nativeTerminal": native_ref,
                "nativeProofRef": proof_id, **proof, "executionIdentity": execution_identity,
                "inputObservations": {"afterPreparation": initial_observation,
                                     "afterExecution": context["final"]["observation"]},
                "inputError": context["final"]["inputError"], "target": str(target),
                "requestedTargetDir": self.payload.get("requestedTargetDir"), "actualTargetDir": str(target),
                "targetPolicy": "managed-compatibility-pool", "taskScratch": task_scratch, "postTaskCache": task_cache,
                "sourceCopiesCreated": 0, "formalAcceptance": False, "m8Accepted": False}
            if batch is not None:
                result.update(batch=True, steps=completed, recipeCount=len(recipes), completedRecipeCount=len(completed),
                              coverage=coverage, nativeProofRefs=list(context["nativeProofRefs"]),
                              performanceAccepted=False, productsAccepted=False)
                if product_helper is not None:
                    result.update(checkoutProduct=product, productError=product_error, packagingNativeProofs=packaging_proofs,
                                  allRecipesExecuted=all_recipes_executed)
            else:
                artifacts, fresh = [], 0
                for line in (report / "stdout.bin").read_text(encoding="utf-8", errors="replace").splitlines():
                    try:
                        event = json.loads(line)
                    except ValueError:
                        continue
                    if event.get("reason") == "compiler-artifact":
                        fresh += int(event.get("fresh") is True)
                        for name in event.get("filenames", []):
                            path = plain(name)
                            if not (path.is_relative_to(target) or path.is_relative_to(target.parent / "build")):
                                raise ValueError("Compiler artifact escaped the approved pool")
                            if path.is_file():
                                artifacts.append({"path": str(path), "sha256": sha(path.read_bytes())})
                result.update(recipe=coverage, artifactFacts=artifacts, cargoFreshArtifactCount=fresh)
            result["artifactManifest"] = self.artifact_manifest(
                root, self._checkout_outputs, native=native, task_scratch=task_scratch, task_cache=task_cache)
            receipt_ref = write_json(report / "receipt.json", result)
            execution, launch = self.state.get("execution", owner), self.state.get("execution_launch", owner)
            self.state.put("execution", owner, {**execution["payload"], "status": status, **proof,
                "nativeProofRef": proof_id, "nativeTerminal": native_ref, "receipt": receipt_ref,
                **({"batchReceipt": receipt_ref} if batch is not None else {})}, expected_version=execution["version"])
            self.state.put("execution_launch", owner, {**launch["payload"], "status": "terminal",
                "nativeProofRef": proof_id, **({"allRecipesExecuted": all_recipes_executed} if batch is not None else {})},
                expected_version=launch["version"])
            if batch is not None:
                self.batch_checkpoint(stage="complete", receipt=receipt_ref, allRecipesExecuted=all_recipes_executed)
            resources.release(admitted["key"], writer_key=writer, native_proof_ref=proof_id)
            return result
        except Exception as error:
            if getattr(error, "code", None) == "checkout_input_identity_conflict":
                conflict = self.checkout_input_conflict_result(
                    error, owner=owner, writer=writer, guard=guard, admitted=admitted)
                if conflict is not None:
                    return conflict
            if guard is not None and admitted is None:
                guard.close()
            if getattr(error, "code", None) == "checkout_input_incomplete" and admitted is None:
                result = {"status": "waiting", "reasonCode": "checkout_input_incomplete", "message": str(error),
                          "inputsComplete": False, "compileStarted": False,
                          "formalAcceptance": False, "m8Accepted": False}
                if isinstance(error, CheckoutInputIncomplete) and error.input_diagnostic is not None:
                    result["inputDiagnostic"] = error.input_diagnostic
                return result
            if getattr(error, "code", None) in {"checkout_source_busy", "management_git_busy"} and admitted is None:
                return {"status": "waiting", "reasonCode": getattr(error, "code"), "message": str(error),
                        "formalAcceptance": False}
            context = getattr(self, "_checkout_context", None)
            if context is not None:
                if context["launched"] or self.state.get("execution_launch", owner):
                    return self.checkout_unknown("checkout_execution_interrupted", message=str(error))
                # No native claim has occurred. Join monitoring and close pins
                # before using the existing durable-unlaunched failure release.
                guard.close()
                read = self.state.get("management_checkout_read", owner)
                if read:
                    self.state.put("management_checkout_read", owner, {
                        **read["payload"], "status": "released", "pinsReleased": True,
                        "durableUnlaunched": True, "releasedAt": time.time()},
                        expected_version=read["version"])
                context["readReleased"] = True
            raise

    def validation_with_cache(self, *, payload=None):
        candidate = getattr(self, "_branch_candidate", None)
        if candidate is not None and candidate.before is not None:
            root, before = candidate.verify(self), candidate.before
            self.verify_declared_inputs(before, self.payload if payload is None else payload)
            return root, before
        root, source = self.validation_source(payload=payload)
        value = {**{k: v for k, v in source.items() if k != "sourceDigest"},
                 "dependencyCache": self.dependency_cache(payload=payload)}
        before = {**value, "sourceDigest": sha(encoded(value))}
        self.verify_declared_inputs(before, self.payload if payload is None else payload)
        return root, before

    def validation_sealed_observation(self, before, *, payload=None):
        """Recheck owned engine/cache without observing mutable foreign sources."""
        payload = self.payload if payload is None else payload
        root = self.validation_root(payload=payload)
        engine, cache = self.source(root, payload=payload), self.dependency_cache(payload=payload)
        observation = {"complete": False, "observationPolicy": "sealed-engine-cache",
            "engineAndCacheObserved": True, "sourceDigest": None, "inputsUnchanged": False,
            "originalEngineUnchanged": (str(root) == before["sourceRoot"]
                and payload.get("sourceView", "worktree") == before["sourceView"] and engine == before["source"]),
            "dependencyCacheUnchanged": cache == before["dependencyCache"],
            "engineSourceDigest": engine["sourceDigest"], "dependencyCacheDigest": sha(encoded(cache)),
            "externalInputsObserved": False, "externalInputsUnchanged": None,
            "externalInputErrors": [], "inputChanges": None, "sourceComponents": None}
        return root, None, observation

    def validation_observation(self, before, *, payload=None):
        """Observe live inputs without making the foreign checkout an execution dependency."""
        if before.get("externalInputPolicy") == "prepared-sealed-CAS":
            return self.validation_sealed_observation(before, payload=payload)
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
        if self.payload.get("sourceView", "worktree") == "checkout":
            return self.validate_checkout()
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
        environment, trusted, _policy_file = self.validation_environment()
        preparation = source_helper()
        compatibility_environment = {**environment, **cargo_manifest_environment(root, recipe),
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
        helper_path = client_path(self.config, self.repo, "tools/dev/local_cargo.py")
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
                                                           if name != "verb"},
                    management=self if before.get("externalInputPolicy") == "prepared-sealed-CAS" else None)
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
        if self.config.get("managedWorktree") == MANAGED_WORKTREE_CAPABILITY:
            from tools.jenkins.managed_worktree import MUTATIONS, writer_hold
            if self.action == "apply-patch":
                self.patch_input(self.payload["patchReference"], self.payload["beforeHashes"])
            if (self.action == "prepare-worktree" and "resumePreparationFromOperationKey" in self.payload):
                with source_helper().materialized_preparation_recovery_hold(self):
                    return self._dispatch()
            if self.action in MUTATIONS and self.payload.get("worktreeId"):
                with writer_hold(self):
                    return self._dispatch()
        return self._dispatch()

    def preview_current_editor(self):
        recipe = fixed_editor_preview_recipe(self.payload)
        if self.repo != plain(PREVIEW_REPOSITORY):
            raise ValueError("The fixed preview belongs to its original physical checkout")
        descriptor = {**recipe["descriptor"], "path": str(plain(recipe["descriptor"]["path"]))}
        read_reference(descriptor)
        command = [*recipe["command"]]
        command[6] = descriptor["path"]
        pins = ((PREVIEW_SCRIPT, PREVIEW_SCRIPT_SHA256), (PREVIEW_PYTHON, PREVIEW_PYTHON_SHA256))
        def check_inputs():
            for path, digest in pins:
                if sha(plain(path).read_bytes()) != digest:
                    raise ValueError("Reviewed fixed preview script or interpreter changed")
            read_reference(descriptor)
        check_inputs()
        registration = json.loads(plain(PREVIEW_REPORTS / "artifacts.json").read_bytes())["jenkins_fixed_editor_preview_preparation"]
        if plain(registration["proofDirectory"]) != plain(PREVIEW_PROOF):
            raise ValueError("Fixed preview evidence is not registered to its task")
        if plain(PREVIEW_OUTPUT).exists():
            raise ValueError("Existing preview evidence is retained; reconcile its original operation")
        report = plain(PREVIEW_PROOF / "native" / self.key)
        report.mkdir(parents=True, exist_ok=False)
        request_ref = write_json(report / "request.json", {
            "schema": "fixed-current-editor-preview-request/v1", "owner": self.session,
            "payload": self.payload, "inputDigest": self.input_digest, "command": command,
            "cwd": str(self.repo), "productDescriptor": descriptor,
            "scriptSha256": PREVIEW_SCRIPT_SHA256, "interpreterSha256": PREVIEW_PYTHON_SHA256,
            "output": str(PREVIEW_OUTPUT), "timeoutSeconds": PREVIEW_TIMEOUT_SECONDS,
            "formalAcceptance": False})
        helper_path = client_path(self.config, self.repo, "tools/dev/local_cargo.py")
        if sha(helper_path.read_bytes()) != self.config["clientSources"]["tools/dev/local_cargo.py"]:
            raise ValueError("Pinned owned native command implementation changed")
        spec = importlib.util.spec_from_file_location("jenkins_fixed_editor_preview_native", helper_path)
        helper = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(helper)
        environment = {**os.environ, "PYTHONDONTWRITEBYTECODE": "1", "PYTHONIOENCODING": "utf-8"}
        native = helper._run_owned_command(command, self.repo, environment, report, PREVIEW_TIMEOUT_SECONDS)
        native_ref = write_json(report / "native-terminal.json", native)
        proof = fixed_editor_preview_native(native, native_ref, command, self.repo)
        result = {"request": request_ref, "nativeEvidence": native_ref, "nativeTerminal": proof,
                  "productDescriptor": descriptor, "output": str(PREVIEW_OUTPUT),
                  "formalAcceptance": False, "m8Accepted": False, "wholeUiAccepted": False}
        if not proof["complete"]:
            return {**result, "status": "unknown", "reasonCode": "preview_native_termination_unproven"}
        if not proof["naturalSuccess"]:
            return {**result, "status": "failed", "reasonCode": "preview_native_command_failed"}
        check_inputs()
        terminal_path = plain(PREVIEW_OUTPUT / "preview-terminal.json")
        terminal_ref = {"path": str(terminal_path), "sha256": sha(terminal_path.read_bytes())}
        terminal = read_reference(terminal_ref, expected_path=terminal_path)
        pixel_ref = terminal.get("pixelReceipt")
        if (not isinstance(pixel_ref, dict)
                or not plain(pixel_ref.get("path", "")).is_relative_to(plain(PREVIEW_OUTPUT / "pixels"))):
            raise ValueError("Pixel receipt escaped the fixed preview output")
        pixel = read_reference(pixel_ref)
        read_reference(terminal["seed"])
        fixed_editor_preview_receipt(terminal, descriptor, pixel)
        return {**result, "status": "passed", "previewReceipt": terminal_ref, "pixelReceipt": pixel_ref,
                "existingDesktop": terminal["existingDesktop"]}

    def _dispatch(self):
        if self.action == PREVIEW_ACTION:
            return self.preview_current_editor()
        if self.action == "prepare-worktree":
            helper = source_helper()
            return helper.managed_prepare_worktree(self, preparation_helper=helper)
        if self.action == "inspect-worktree":
            helper = source_helper()
            return helper.managed_inspect_worktree(self, preparation_helper=helper)
        if self.action == "retire-worktree-reader":
            if "unlaunchedRecovery" in self.payload:
                return source_helper().retire_unlaunched_flow_reader(self)
            from tools.jenkins.managed_worktree_lifetime import retire_reader
            return retire_reader(self)
        if self.action == "create-branch":
            return self.create_branch()
        if self.action == "create-worktree":
            ref, _path, value = self.new_worktree()
            source = getattr(self, "_committed_successor_source", None)
            return {**value, "status": "complete", "worktreeId": ref,
                    "source": self.source(_path) if source is None else source}
        if self.action == "prepare-merge":
            return self.prepare_merge()
        if self.action == "apply-patch":
            return self.apply_patch()
        if self.action == "release-worktree":
            return self.release_worktree()
        if self.action == "recover-worktree":
            return self.recover_worktree()
        if self.action == "prepare-flow":
            return self.prepare_flow()
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
            if "resumeExecution" in self.payload["flowPayload"]:
                return source_helper().resume_prepared_flow_execution(self, self.payload["flowPayload"])
            delivery = handle("submit", self.payload["flowPayload"], self.state, self.repo)
            return {"status": "submitted" if delivery.get("delivery") == "submitted" else "unknown",
                    "submission": delivery, "formalAcceptance": False}
        raise ValueError("Unknown management action")

    def existing_result(self, row):
        value = row["payload"]
        if value["inputDigest"] != self.input_digest:
            raise ValueError("Original operation has different input")
        result = value.get("result", {"status": "unknown", "reasonCode": "original_operation_running",
                                     "operation": value, "formalAcceptance": False})
        if self.payload.get("checkoutProduct") and result.get("status") == "passed":
            self.verify_checkout_product_delivery(result)
        return result

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
                if (row["payload"]["phase"] != "waiting"
                        and not ("batch" in self.payload and self.batch_recovery_ready())
                        and not (self.action == "retire-worktree-reader"
                                 and isinstance(self.payload.get("unlaunchedRecovery"), dict)
                                 and self.payload["unlaunchedRecovery"].get("schemaVersion") == 1
                                 and self.payload["unlaunchedRecovery"].get("kind") == "cancelled-before-native-launch-v1")):
                    return {"status": "unknown", "reasonCode": "original_operation_requires_reconciliation",
                            "operation": row["payload"], "formalAcceptance": False}
            record = {**(row["payload"] if row else {}), "requestId": self.request, "operationId": self.operation,
                      "sessionId": self.session, "action": self.action, "inputDigest": self.input_digest,
                      "phase": "running", "startedAt": time.time(), "payload": self.payload}
            self.state.put("management_operation", self.key, record, expected_version=row["version"] if row else 0)
            try:
                if self.action in GIT_ACTIONS:
                    with process_lock(self.repo / ".jenkins/state/locks/management-git.lock", reason="management_git_busy", timeout=0):
                        read = self.checkout_git_read_conflict()
                        if read:
                            result = {"status": "waiting", "reasonCode": "checkout_source_busy",
                                      "blockingReadOwner": read["key"], "formalAcceptance": False}
                        else:
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
            finished = {**record, **current["payload"], "phase": "waiting" if result["status"] == "waiting" else
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
        payload, routing = route_request_context(
            config, repo, manager, payload, query_only=payload.get("observationOnly") is True)
        result = Management(config, repo, manager, payload).run()
        if routing is not None:
            result = {**result, "managedContext": routing}
    except Exception as error:
        result = {"status": "failed", "message": str(error), "formalAcceptance": False}
    # Control responses may be refreshed; immutable evidence lives in State/reports.
    if "destination" not in locals() or destination.parent != permitted:
        print(encoded(result).decode("utf-8"))
        return 2
    destination.write_bytes(encoded(result) + b"\n")
    print(encoded(result).decode("utf-8"))
    return 0 if result.get("status") in {"complete", "prepared", "passed", "waiting", "merge-prepared", "submitted", "released"} else 2


if __name__ == "__main__":
    raise SystemExit(main())
