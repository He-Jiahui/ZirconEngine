"""Read-only proof for a same-generation metadata-only management hot update.

This operator is separately hash-pinned by the installer, never a capsule member.
It performs no POST, State construction, native launch, or lifecycle action.
"""
from __future__ import annotations

import argparse
import ast
import ctypes
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import sqlite3
import stat
import sys
import xml.etree.ElementTree as ET

SHA = re.compile(r"[0-9a-f]{64}\Z")
IDENTIFIER = re.compile(r"[A-Za-z0-9][A-Za-z0-9_.-]{0,79}\Z")
CLIENTS = {"tools/dev/local_cargo.py", "tools/dev/local-cargo.ps1",
           "tools/dev/dev-fast-build.ps1", "tools/dev/submit-jenkins-management.ps1",
           "tools/jenkins/frontend.py", "tools/jenkins-management/entry.py"}
METHODS = {"worktree_resume_release", "worktree_resume_lineage"}
ACTIONS = {"validate", "create-worktree", "inspect-worktree", "prepare-worktree"}
ADOPTION_HELPER_BEFORE = "bf6130945bc342a820e12f1597ca2c2b031b30ffa45585f9a55213dddaccc2b4"
ADOPTION_HELPER_AFTER = "8ab962e8c7e39c53e7cdd9f145e719809eb802be1d4777a12a18e741afee70be"
ADOPTION_BEFORE = b"    old = previous.get(\"ownedFiles\", {}) if previous else {}\r\n    actual = _pool_inventory(destination, old if previous else None)\r\n    if previous and previous[\"sourceDigest\"] == sealed[\"sealedInputRef\"]:\r\n"
ADOPTION_AFTER = b"    old = previous.get(\"ownedFiles\", {}) if previous else {}\r\n    adoption_rows = None\r\n    if (previous and previous[\"sourceDigest\"] != sealed[\"sealedInputRef\"] and old == desired\r\n            and recovered and recovered[\"payload\"].get(\"status\") == \"complete\"):\r\n        if (previous.get(\"preparationKey\") != key or previous.get(\"physicalPath\") != str(destination)\r\n                or previous.get(\"objectRoot\") != str(objects)\r\n                or recovered[\"payload\"].get(\"preparationKey\") != key\r\n                or recovered[\"payload\"].get(\"ownedFiles\") != old):\r\n            raise ValueError(\"Same-byte preparation lacks its completed physical binding\")\r\n        adoption_rows = [(domain, row_key, state.get(domain, row_key)) for domain, row_key in (\r\n            (\"execution\", owner), (\"pool_writer_hold\", key),\r\n            (\"resource_reservation\", reservation_key))]\r\n    actual = _pool_inventory(destination, old if previous else None)\r\n    if adoption_rows is not None and old == desired == actual:\r\n        current_execution, hold, reservation = [((row or {}).get(\"payload\", {}))\r\n                                                for _domain, _row_key, row in adoption_rows]\r\n        if (current_execution.get(\"kind\") != \"management_owned_cargo\"\r\n                or current_execution.get(\"status\") != \"preparing\"\r\n                or current_execution.get(\"preparationKey\") != key\r\n                or current_execution.get(\"reservationKey\") != reservation_key\r\n                or hold.get(\"status\") != \"active\" or hold.get(\"owner\") != owner\r\n                or hold.get(\"reservationKey\") != reservation_key\r\n                or reservation.get(\"status\") != \"active\" or reservation.get(\"owner\") != owner):\r\n            raise ValueError(\"Same-byte preparation requires its active writer and reservation\")\r\n        pins = [(\"preparation_source\", key, previous_row), (\"preparation_update\", key, recovered),\r\n                (\"sealed_input\", sealed[\"sealedInputRef\"], record), *adoption_rows]\r\n        # No source bytes change. Publish the new seal only after the complete\r\n        # inventory, without creating a per-file update journal for identical bytes.\r\n        adopted = {\"executionId\": owner, \"preparationKey\": key,\r\n                   \"sourceDigest\": sealed[\"sealedInputRef\"], \"manifest\": normalized,\r\n                   \"objectRoot\": str(objects), \"ownedFiles\": desired,\r\n                   \"physicalPath\": str(destination), \"updatedAt\": time.time()}\r\n        with state.transaction() as connection:\r\n            if any(state.get(domain, row_key, connection=connection) != pinned\r\n                   for domain, row_key, pinned in pins):\r\n                raise ValueError(\"Same-byte preparation authority changed before identity adoption\")\r\n            state.put(\"preparation_source\", key, adopted,\r\n                      expected_version=previous_row[\"version\"], connection=connection)\r\n        previous = adopted\r\n    if previous and previous[\"sourceDigest\"] == sealed[\"sealedInputRef\"]:\r\n"
ADDON_SHA256 = "59ae711879085d0f7a36f90216def9eac04538fc46d75eb1a3b97b61441277bc"
ADDON_PREFIX_SHA256 = "80d0325b36b0f4c17e1dfb05519160105a82d4e129e0edfc0d8507f43272375c"


def require(value, message):
    if not value:
        raise ValueError("pinned-active update: " + message)


def encoded(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False,
                      allow_nan=False).encode("utf-8")

PREPARED_FLOW_HELPER_BEFORE = '8ab962e8c7e39c53e7cdd9f145e719809eb802be1d4777a12a18e741afee70be'
PREPARED_FLOW_HELPER_AFTER = 'ae9f533e1d7e5de41250f3e5791c6a773aa981b0d39f8763073b8b0b77a3a3db'
PREPARED_FLOW_RUNNER_BEFORE = '8bf13dda049fafb4c0f01bff3763815434be7fc9ce2e95e715c466ea849b1a9f'
PREPARED_FLOW_RUNNER_AFTER = 'e43350140160fc736cb2d12aee2cbdb3b693da651b2b332ee9dfa6f8781edff0'
PREPARED_FLOW_ADDITION_SHA256 = '37eed1b2040e84679da9694ca98e1ee3ee9183a219be95923c84030ae8b3e3e4'
PREPARED_FLOW_ADDITION_SIZE = 15559
PREPARED_FLOW_RUNNER_HUNKS = ((b'        registered = current_source.handle("register", {', b'        registration = {'), (b'            **prepared_fields,\n        }, self.state, self.repo)\n        if registered.get("status") != "registered":', b'            **prepared_fields,\n        }\n        # Only a completed prepared WT can retain unchanged owned absences.\n        absent_owned = prepared_ref is not None and any(\n            hashes[name] is None and name not in tracked for name in owned)\n        registered = (source_helper().register_prepared_worktree_flow(self, registration) if absent_owned\n                      else current_source.handle("register", registration, self.state, self.repo))\n        if registered.get("status") != "registered":'))


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def plain(value):
    require(isinstance(value, (str, Path)) and str(value), "physical path is missing")
    path = Path(value)
    require(path.is_absolute() and ".." not in path.parts, "path must be absolute without parent traversal")
    for item in (path, *path.parents):
        info = item.lstat()
        require(not stat.S_ISLNK(info.st_mode) and not getattr(info, "st_file_attributes", 0) & 0x400,
                "path crosses a filesystem alias")
    return path


def under(root, value):
    path = plain(value)
    require(path.is_relative_to(root), "path escaped its fixed owner")
    return path


def read(path, checksum=None):
    path = plain(path)
    with path.open("rb") as stream:
        before = os.fstat(stream.fileno())
        raw = stream.read()
        after = os.fstat(stream.fileno())
    identity = lambda s: (s.st_dev, s.st_ino, s.st_mode, s.st_size, s.st_mtime_ns, s.st_ctime_ns, s.st_nlink)
    require(identity(before) == identity(after)
            and (after.st_dev, after.st_ino) == (path.stat().st_dev, path.stat().st_ino),
            "file changed during observation")
    require(checksum is None or isinstance(checksum, str) and SHA.fullmatch(checksum) and sha(raw) == checksum,
            "referenced bytes changed")
    return raw


def document(path, checksum=None):
    value = json.loads(read(path, checksum))
    require(isinstance(value, dict), "JSON document must be an object")
    return value


def capsule(repo, checksum):
    require(isinstance(checksum, str) and SHA.fullmatch(checksum), "capsule digest is invalid")
    root = plain(repo / ".jenkins/runtime/management" / checksum)
    value = document(root / "capsule.json", checksum)
    require(value.get("schemaVersion") == 1 and value.get("repositoryRoot") == str(repo)
            and value.get("agentLabel") == "zircon-windows-agent", "capsule owner differs")
    files = value.get("files", {})
    require(isinstance(files, dict) and set(files) in (
        {"runner.py", "source_preparation.py", "zircon-management.groovy"},
        {"runner.py", "source_preparation.py", "zircon-management.groovy", "checkout_guard.py"},
        {"runner.py", "source_preparation.py", "zircon-management.groovy", "checkout_guard.py", "checkout_product.py"}),
        "capsule inventory differs")
    for name, checksum in files.items():
        read(root / name, checksum)
    require(set(value.get("clientSources", {})) == CLIENTS, "six-client inventory differs")
    runner = None
    if "clientRoot" in value:
        spec = importlib.util.spec_from_file_location("pinned_install_client_selector", root / "runner.py")
        runner = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(runner)
    for name, checksum in value["clientSources"].items():
        path = runner.client_path(value, repo, name, capsule=root) if runner is not None else repo / name
        read(path, checksum)
    return root, value


def runner_interface(raw):
    tree = ast.parse(raw.decode("utf-8-sig"))
    classes = [node for node in tree.body if isinstance(node, ast.ClassDef) and node.name == "Management"]
    require(len(classes) == 1, "runner Management owner is ambiguous")
    owner = classes[0]
    for name in METHODS:
        require(sum(isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)) and node.name == name
                    for node in owner.body) <= 1, "metadata method is duplicated")
    require(any(isinstance(node, ast.FunctionDef) and node.name == "worktree_resume_lineage"
                for node in owner.body), "runner lacks the existing metadata lineage method")
    owner.body = [node for node in owner.body
                  if not isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)) or node.name not in METHODS]
    return ast.dump(tree, include_attributes=False)

def prepared_flow_compatibility(repo, old_root, old, new_root, new):
    """Accept only the reviewed prepared-WT absent-owned release bytes."""
    require(old["files"]["runner.py"] == PREPARED_FLOW_RUNNER_BEFORE
            and new["files"]["runner.py"] == PREPARED_FLOW_RUNNER_AFTER
            and old["files"]["source_preparation.py"] == PREPARED_FLOW_HELPER_BEFORE
            and new["files"]["source_preparation.py"] == PREPARED_FLOW_HELPER_AFTER,
            "parallel Flow adapter differs from the exact reviewed release")
    contracts = []
    for metadata in (old, new):
        contract = dict(metadata)
        contract["files"] = {name: checksum for name, checksum in metadata["files"].items()
                             if name not in {"runner.py", "source_preparation.py"}}
        contracts.append(contract)
    require(contracts[0] == contracts[1], "Flow adapter changed another capsule member or client")
    before = read(old_root / "source_preparation.py", PREPARED_FLOW_HELPER_BEFORE)
    after = read(new_root / "source_preparation.py", PREPARED_FLOW_HELPER_AFTER)
    require(after.startswith(before) and len(after) - len(before) == PREPARED_FLOW_ADDITION_SIZE
            and sha(after[len(before):]) == PREPARED_FLOW_ADDITION_SHA256,
            "Flow adapter changed existing preparation implementation")
    old_tree, new_tree = ast.parse(before.decode("utf-8-sig")), ast.parse(after.decode("utf-8-sig"))
    additions = new_tree.body[len(old_tree.body):]
    require(len(additions) == 2
            and all(isinstance(node, ast.FunctionDef) for node in additions)
            and [node.name for node in additions] == [
                "_verify_prepared_worktree_flow", "register_prepared_worktree_flow"]
            and ast.dump(old_tree, include_attributes=False)
            == ast.dump(ast.Module(body=new_tree.body[:-2], type_ignores=new_tree.type_ignores),
                        include_attributes=False), "Flow adapter has unreviewed module effects")
    before_runner = read(old_root / "runner.py", PREPARED_FLOW_RUNNER_BEFORE)
    after_runner = read(new_root / "runner.py", PREPARED_FLOW_RUNNER_AFTER)
    candidate = before_runner
    for old_hunk, new_hunk in PREPARED_FLOW_RUNNER_HUNKS:
        require(candidate.count(old_hunk) == 1, "Flow adapter runner hunk is ambiguous")
        candidate = candidate.replace(old_hunk, new_hunk, 1)
    require(candidate == after_runner, "Flow adapter changed another runner execution path")
    for old_hunk, new_hunk in reversed(PREPARED_FLOW_RUNNER_HUNKS):
        require(candidate.count(new_hunk) == 1, "Flow adapter inverse hunk is ambiguous")
        candidate = candidate.replace(new_hunk, old_hunk, 1)
    require(candidate == before_runner, "Flow adapter runner hunk is not reversible")
    return sha(runner_interface(before_runner).encode("utf-8"))


def compatibility(repo, old_root, old, new_root, new, *, allow_materialize_adoption=False,
                  allow_prepared_worktree_flow=False):
    if (allow_prepared_worktree_flow
            and old["files"]["source_preparation.py"] == PREPARED_FLOW_HELPER_BEFORE
            and new["files"]["source_preparation.py"] == PREPARED_FLOW_HELPER_AFTER):
        return prepared_flow_compatibility(repo, old_root, old, new_root, new)
    old_contract, new_contract = dict(old), dict(new)
    helper_changed = old["files"]["source_preparation.py"] != new["files"]["source_preparation.py"]
    excluded = {"runner.py"}
    if helper_changed and allow_materialize_adoption:
        excluded.add("source_preparation.py")
    old_contract["files"] = {k: v for k, v in old["files"].items() if k not in excluded}
    new_contract["files"] = {k: v for k, v in new["files"].items() if k not in excluded}
    require(old_contract == new_contract, "parallel update may change only runner metadata methods")
    before_raw = read(old_root / "runner.py", old["files"]["runner.py"])
    after_raw = read(new_root / "runner.py", new["files"]["runner.py"])
    before, after = runner_interface(before_raw), runner_interface(after_raw)
    require(before == after, "runner public or execution implementation changed")
    if helper_changed:
        require(allow_materialize_adoption and before_raw == after_raw
                and old["files"]["source_preparation.py"] == ADOPTION_HELPER_BEFORE
                and new["files"]["source_preparation.py"] == ADOPTION_HELPER_AFTER,
                "parallel preparation update differs from the exact identity-adoption release")
        old_helper = read(old_root / "source_preparation.py", ADOPTION_HELPER_BEFORE)
        new_helper = read(new_root / "source_preparation.py", ADOPTION_HELPER_AFTER)
        require(old_helper.count(ADOPTION_BEFORE) == new_helper.count(ADOPTION_AFTER) == 1
                and old_helper.replace(ADOPTION_BEFORE, ADOPTION_AFTER, 1) == new_helper
                and new_helper.replace(ADOPTION_AFTER, ADOPTION_BEFORE, 1) == old_helper,
                "preparation update differs outside the exact identity-adoption hunk")
    return sha(before.encode("utf-8"))

def pipeline(repo, root, metadata, codex):
    tokens = {"AGENT": "zircon-windows-agent", "REPOSITORY": str(repo),
              "PYTHON": str(repo / ".jenkins/runtime/python/python.exe"), "CODEX": codex,
              "CAPSULE": str(root), "DRIVER": metadata["driverDigest"], "RUNTIME": metadata["runtimeOperationId"]}
    text = read(root / "zircon-management.groovy", metadata["files"]["zircon-management.groovy"]).decode("utf-8-sig")
    text = text.replace("\r\n", "\n")
    for key, value in tokens.items():
        require(isinstance(value, str) and not any(c in value for c in '\r\n"'), "pipeline token is invalid")
        text = text.replace("@@" + key + "@@", value.replace("\\", "/").replace("'", "\\'"))
    require("@@" not in text, "pipeline has an unresolved token")
    return text


def observed_pipeline(repo, root, metadata, codex, script):
    require(isinstance(script, str), "running CPS script is missing")
    core = pipeline(repo, root, metadata, codex)
    if script != core:
        try:
            raw = read(repo / "tools/jenkins-management/checkout-commit.groovy", ADDON_SHA256)
        except OSError as error:
            raise ValueError("pinned-active update: running CPS addon is unavailable") from error
        prefix = raw.decode("utf-8-sig").replace("\r\n", "\n").rstrip("\n") + "\n\n"
        require(len(prefix.encode("utf-8")) == 2352
                and sha(prefix.encode("utf-8")) == ADDON_PREFIX_SHA256
                and script == prefix + core, "running CPS addon or capsule differs")
    return sha(script.encode("utf-8"))


def observe_native(pid):
    """Query one retained handle; return safe identity and parsed runner paths only."""
    require(os.name == "nt" and type(pid) is int and pid > 0, "Windows native PID is required")
    from ctypes import wintypes
    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
    kernel.OpenProcess.restype = wintypes.HANDLE
    kernel.CloseHandle.argtypes = [wintypes.HANDLE]
    kernel.GetProcessTimes.argtypes = [wintypes.HANDLE] + [ctypes.POINTER(wintypes.FILETIME)] * 4
    kernel.QueryFullProcessImageNameW.argtypes = [wintypes.HANDLE, wintypes.DWORD, wintypes.LPWSTR, ctypes.POINTER(wintypes.DWORD)]
    kernel.GetExitCodeProcess.argtypes = [wintypes.HANDLE, ctypes.POINTER(wintypes.DWORD)]
    handle = kernel.OpenProcess(0x1000, False, pid)
    require(bool(handle), "native process identity is unavailable")
    try:
        times = [wintypes.FILETIME() for _ in range(4)]
        require(kernel.GetProcessTimes(handle, *(ctypes.byref(x) for x in times)), "native birth is unavailable")
        birth = str((times[0].dwHighDateTime << 32) | times[0].dwLowDateTime)
        length = wintypes.DWORD(32768)
        image = ctypes.create_unicode_buffer(length.value)
        require(kernel.QueryFullProcessImageNameW(handle, 0, image, ctypes.byref(length)), "native executable is unavailable")
        class UnicodeString(ctypes.Structure):
            _fields_ = [("Length", wintypes.USHORT), ("MaximumLength", wintypes.USHORT), ("Buffer", ctypes.c_void_p)]
        nt = ctypes.WinDLL("ntdll")
        nt.NtQueryInformationProcess.argtypes = [wintypes.HANDLE, wintypes.ULONG, ctypes.c_void_p,
                                                wintypes.ULONG, ctypes.POINTER(wintypes.ULONG)]
        nt.NtQueryInformationProcess.restype = ctypes.c_long
        needed = wintypes.ULONG()
        nt.NtQueryInformationProcess(handle, 60, None, 0, ctypes.byref(needed))
        require(ctypes.sizeof(UnicodeString) <= needed.value <= 2 * 1024**2, "native command size is invalid")
        buffer = ctypes.create_string_buffer(needed.value)
        require(nt.NtQueryInformationProcess(handle, 60, buffer, needed.value, ctypes.byref(needed)) == 0,
                "native command is unavailable")
        string = ctypes.cast(buffer, ctypes.POINTER(UnicodeString)).contents
        require(string.Length % 2 == 0 and ctypes.addressof(buffer) <= string.Buffer
                and string.Buffer + string.Length <= ctypes.addressof(buffer) + len(buffer),
                "native command buffer is invalid")
        command = ctypes.wstring_at(string.Buffer, string.Length // 2)
        shell = ctypes.WinDLL("shell32", use_last_error=True)
        shell.CommandLineToArgvW.argtypes = [wintypes.LPCWSTR, ctypes.POINTER(ctypes.c_int)]
        shell.CommandLineToArgvW.restype = ctypes.POINTER(wintypes.LPWSTR)
        count = ctypes.c_int()
        argv = shell.CommandLineToArgvW(command, ctypes.byref(count))
        require(bool(argv), "native command cannot be parsed")
        try:
            args = [argv[i] for i in range(count.value)]
        finally:
            kernel.LocalFree.argtypes = [ctypes.c_void_p]
            kernel.LocalFree(ctypes.cast(argv, ctypes.c_void_p))
        require(len(args) == 11 and args[1:4] == ["-B", "-X", "utf8"]
                and args[5] == "--capsule" and args[7] == "--input-file" and args[9] == "--output-file",
                "native process is not the pinned management runner")
        exit_code = wintypes.DWORD()
        require(kernel.GetExitCodeProcess(handle, ctypes.byref(exit_code)) and exit_code.value == 259,
                "native process is no longer active")
        return {"pid": pid, "creationTime": birth, "executable": image.value,
                "commandDigest": sha(command.encode("utf-8")), "runnerPath": args[4],
                "capsulePath": args[6], "inputFile": args[8], "outputFile": args[10]}
    finally:
        kernel.CloseHandle(handle)


def live_reader(root, repo, metadata):
    spec = importlib.util.spec_from_file_location("verified_parallel_install_runner", root / "runner.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    for name in tuple(sys.modules):
        if name == "tools" or name == "tools.jenkins" or name.startswith("tools.jenkins."):
            del sys.modules[name]
    verified, bound_repo, manager = module.boot(root)
    require(verified == metadata and bound_repo == repo, "immutable bootstrap changed its binding")
    def get_json(path):
        require(path.startswith(("job/zircon-management/", "queue/api/json")), "unexpected API read")
        status, raw = manager._request(path, method="GET", timeout=15)
        require(status == 200, "Jenkins observation is unavailable")
        value = json.loads(raw)
        require(isinstance(value, dict), "Jenkins observation is not an object")
        return value
    return get_json


def parameters(value):
    result = {}
    for action in value.get("actions", []):
        for item in action.get("parameters", []):
            name = item.get("name")
            require(name not in result, "build parameter is duplicated")
            result[name] = item.get("value")
    names = {"SESSION_ID", "REQUEST_ID", "OPERATION_ID", "ACTION", "INPUT_JSON",
             "EXPECTED_DRIVER_DIGEST", "RUNTIME_OPERATION_ID", "QUERY_ONLY"}
    require(set(result) == names, "build parameter inventory differs")
    return result


def verify_update(request, *, get_json=None, observe_native=None):
    """Verify an optional concurrent install; injected callbacks replace observations only."""
    require(isinstance(request, dict), "request must be an object")
    repo = plain(request.get("repositoryRoot", ""))
    require(request.get("phase") in {"before-post", "after-post"}, "installation phase differs")
    require(isinstance(request.get("proofSha256"), str) and SHA.fullmatch(request["proofSha256"]),
            "proof SHA is required")
    proof_path = under(repo / ".jenkins/state/management/reports", request.get("proofPath", ""))
    proof = document(proof_path, request.get("proofSha256"))
    require(proof.get("schemaVersion") == 1 and proof.get("kind") == "management-pinned-active-update-v1"
            and proof.get("repositoryRoot") == str(repo), "proof owner or schema differs")
    for proof_key, request_key in (
        ("previousInstalledSha256", "expectedInstalledSha256"),
        ("candidateCapsuleSha256", "expectedCandidateCapsuleSha256"),
        ("previousJobConfigSha256", "expectedPreviousJobConfigSha256"),
        ("driverDigest", "expectedDriverDigest")):
        require(SHA.fullmatch(str(request.get(request_key, "")))
                and proof.get(proof_key) == request[request_key], "proof digest binding differs")
    require(proof.get("runtimeOperationId") == request.get("expectedRuntimeOperationId")
            and isinstance(proof["runtimeOperationId"], str) and proof["runtimeOperationId"],
            "runtime binding differs")
    installed_path = repo / ".jenkins/state/management/installed.json"
    original = read(installed_path, proof["previousInstalledSha256"])
    installed = json.loads(original)
    old_root, old = capsule(repo, proof.get("previousCapsuleSha256"))
    new_root, new = capsule(repo, proof["candidateCapsuleSha256"])
    require(installed.get("schemaVersion") == 1 and installed.get("job") == "zircon-management"
            and installed.get("capsuleSha256") == proof["previousCapsuleSha256"]
            and installed.get("capsule") == str(old_root) and installed.get("formalAcceptance") is False,
            "previous installation differs")
    for name in ("driverDigest", "runtimeOperationId"):
        require(old.get(name) == new.get(name) == installed.get(name) == proof[name], "generation differs")
    require(installed.get("ordinaryEntry") == old.get("ordinaryEntry")
            and installed.get("managedWorktree") == old.get("managedWorktree"), "installed capability differs")
    selected = document(repo / ".jenkins/state/deployment/driver.json")
    runtime = document(repo / ".jenkins/state/deployment-operation.json")
    require(selected.get("driverDigest") == proof["driverDigest"]
            and selected.get("runtimeOperationId") == proof["runtimeOperationId"]
            and runtime.get("operationId") == proof["runtimeOperationId"] and runtime.get("state") == "running",
            "selected runtime changed")
    prepared_flow_release = (old["files"]["source_preparation.py"] == PREPARED_FLOW_HELPER_BEFORE
                             and new["files"]["source_preparation.py"] == PREPARED_FLOW_HELPER_AFTER)
    interface_digest = compatibility(repo, old_root, old, new_root, new, allow_materialize_adoption=True,
                                    allow_prepared_worktree_flow=True)
    codex = str(plain(request.get("codexExe", "")))
    require(str(plain(request.get("previousCodexExe", ""))) == codex, "parallel Codex host changes are unsupported")
    builds = proof.get("builds")
    require(isinstance(builds, list) and 1 <= len(builds) <= 16, "proof requires bounded original builds")
    indexed = {}
    for binding in builds:
        require(isinstance(binding, dict) and type(binding.get("number")) is int and binding["number"] > 0
                and binding["number"] not in indexed and type(binding.get("queueId")) is int
                and binding["queueId"] > 0, "proof build identity is invalid")
        indexed[binding["number"]] = binding
    reader = get_json or live_reader(old_root, repo, old)
    native_reader = observe_native or globals()["observe_native"]
    runs = reader("job/zircon-management/api/json?tree=builds[number,queueId,building,result]")
    queue = reader("queue/api/json?tree=items[task[name]]")
    require(isinstance(runs.get("builds"), list) and isinstance(queue.get("items"), list)
            and all(isinstance(item, dict) and type(item.get("number")) is int
                    and type(item.get("building")) is bool for item in runs["builds"])
            and all(isinstance(item, dict) and isinstance(item.get("task"), dict)
                    and isinstance(item["task"].get("name"), str) for item in queue["items"]),
            "build/queue observation differs")
    require(not any(item.get("task", {}).get("name") == "zircon-management" for item in queue["items"]),
            "queued management work requires reconciliation")
    active = [item for item in runs["builds"] if item.get("building") is True]
    require(len({item.get("number") for item in active}) == len(active)
            and all(item.get("number") in indexed for item in active), "unproven management build is active")
    database = plain(repo / ".jenkins/state/coordination.sqlite3")
    observed, observed_capsules = [], {}
    with sqlite3.connect(database.as_uri() + "?mode=ro", uri=True) as connection:
        connection.execute("PRAGMA query_only=ON")
        for item in active:
            binding = indexed[item["number"]]
            active_root, active_metadata = capsule(repo, binding.get("capsuleSha256"))
            compatibility(repo, active_root, active_metadata, old_root, old,
                          allow_materialize_adoption=prepared_flow_release)
            require(item.get("queueId") == binding["queueId"], "active queue identity differs")
            value = reader(f"job/zircon-management/{item['number']}/api/json?tree=number,queueId,building,result,actions[parameters[name,value]]")
            require(value.get("number") == item["number"] and value.get("queueId") == binding["queueId"],
                    "active build identity changed")
            if value.get("building") is False:
                require(value.get("result") is not None, "build termination is inconclusive")
                continue
            require(value.get("building") is True and value.get("result") is None, "build status is inconclusive")
            params = parameters(value)
            require(params["ACTION"] in ACTIONS and params["QUERY_ONLY"] is False
                    and params["EXPECTED_DRIVER_DIGEST"] == proof["driverDigest"]
                    and params["RUNTIME_OPERATION_ID"] == proof["runtimeOperationId"], "active request generation differs")
            payload = json.loads(params["INPUT_JSON"])
            require(isinstance(payload, dict), "active input must be an object")
            authority = {"sessionId": params["SESSION_ID"], "requestId": params["REQUEST_ID"],
                         "operationId": params["OPERATION_ID"], "action": params["ACTION"],
                         "driverDigest": proof["driverDigest"], "runtimeOperationId": proof["runtimeOperationId"],
                         "observationOnly": False}
            require(all(name not in payload or payload[name] == value for name, value in authority.items()),
                    "active input replaces its build identity")
            payload.update(authority)
            identity = {name: payload.get(name) for name in ("repositoryId", "sessionId", "requestId", "operationId", "action")}
            require(binding.get("identity") == identity
                    and all(isinstance(identity[name], str) and IDENTIFIER.fullmatch(identity[name])
                            for name in ("sessionId", "requestId", "operationId")), "proof request identity differs")
            key = sha(encoded({name: identity[name] for name in ("repositoryId", "sessionId", "requestId", "operationId")}))
            input_digest = sha(encoded({name: value for name, value in payload.items() if name != "observationOnly"}))
            require(binding.get("operationKey") == key and binding.get("inputDigest") == input_digest,
                    "active input digest differs")
            row = connection.execute("SELECT payload FROM records WHERE domain='management_operation' AND key=?", (key,)).fetchone()
            require(row is not None, "original management operation is missing")
            operation = json.loads(row[0])
            require(operation.get("action") == identity["action"] and operation.get("inputDigest") == input_digest
                    and operation.get("payload") == payload, "original management operation differs")
            home = plain(repo / ".jenkins/jenkins_home/jobs/zircon-management/builds" / str(item["number"]) / "build.xml")
            run = ET.fromstring(read(home))
            scripts = run.findall("./execution/script")
            require(run.tag == "flow-build" and run.findtext("queueId") == str(binding["queueId"])
                    and len(scripts) == 1
                    and binding.get("pipelineSha256") == observed_pipeline(
                        repo, active_root, active_metadata, codex, scripts[0].text),
                    "running CPS script is not the original pinned capsule")
            native = binding.get("native", {})
            require(isinstance(native, dict) and type(native.get("pid")) is int and native["pid"] > 0
                    and isinstance(native.get("creationTime"), str) and native["creationTime"].isdigit()
                    and isinstance(native.get("commandDigest"), str) and SHA.fullmatch(native["commandDigest"]),
                    "native proof identity is incomplete")
            actual = native_reader(native["pid"])
            require(isinstance(actual, dict), "native observation is invalid")
            require(all(actual.get(name) == native.get(name) for name in ("pid", "creationTime", "commandDigest"))
                    and plain(actual.get("executable", "")) == plain(native.get("executable", ""))
                    == plain(repo / ".jenkins/runtime/python/python.exe"), "native PID or birth binding changed")
            require(plain(actual.get("runnerPath", "")) == active_root / "runner.py"
                    and plain(actual.get("capsulePath", "")) == active_root, "active process uses another capsule")
            input_ref = binding.get("inputRef", {})
            require(isinstance(input_ref, dict) and isinstance(input_ref.get("sha256"), str)
                    and SHA.fullmatch(input_ref["sha256"]), "original control reference is incomplete")
            control = under(repo / ".jenkins/state/controls", input_ref.get("path", ""))
            output = Path(actual.get("outputFile", ""))
            require(output.is_absolute() and ".." not in output.parts
                    and plain(output.parent) == control.parent, "native output escaped its delivery owner")
            if output.exists():
                plain(output)
            require(control.parent == repo / ".jenkins/state/controls"
                    and re.fullmatch(re.escape("management-" + identity["operationId"] + "." + str(item["number"]))
                                     + r"\.\d+\.json", control.name)
                    and plain(actual.get("inputFile", "")) == control
                    and output == control.with_name(control.stem + ".response.json"),
                    "native control belongs to another delivery")
            require(document(control, input_ref.get("sha256")) == payload, "original control bytes or payload differ")
            observed.append(item["number"])
            observed_capsules[str(item["number"])] = active_root.name
    read(installed_path, proof["previousInstalledSha256"])
    read(proof_path, request["proofSha256"])
    for checksum in {old_root.name, new_root.name, *observed_capsules.values()}:
        capsule(repo, checksum)
    return {"status": "pinned-active-compatible", "phase": request["phase"],
            "proofSha256": request["proofSha256"], "previousCapsuleSha256": proof["previousCapsuleSha256"],
            "candidateCapsuleSha256": proof["candidateCapsuleSha256"], "observedActiveBuildNumbers": observed,
            "runnerInterfaceSha256": interface_digest, "commandDigestPolicy": "windows-command-line-utf8-sha256-v1",
            "observedActiveCapsules": observed_capsules,
            "sourcePreparationPolicy": ("prepared-worktree-absent-owned-v1" if prepared_flow_release else
                "same-byte-identity-adoption-v1"
                if old["files"]["source_preparation.py"] != new["files"]["source_preparation.py"] else "unchanged"),
            "runtimeChanged": False, "oldCapsuleRetained": True}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--observe-native", type=int)
    args = parser.parse_args(argv)
    try:
        result = observe_native(args.observe_native) if args.observe_native is not None else verify_update(json.load(sys.stdin))
        print(json.dumps(result, sort_keys=True, separators=(",", ":"), ensure_ascii=False, allow_nan=False))
        return 0
    except Exception as exc:
        print(json.dumps({"status": "rejected", "message": str(exc)}, separators=(",", ":"), ensure_ascii=False))
        return 1


if __name__ == "__main__":
    raise SystemExit(main())

