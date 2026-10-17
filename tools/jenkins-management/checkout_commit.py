"""Ordinary checkout commits without whole-tree hashes or per-chat grants.

The current user has authorized scoped local commits. This route deliberately
does not issue product acceptance, push changes, or replay old failed workflows.
"""
from __future__ import annotations

import argparse
import base64
from contextlib import contextmanager
import importlib.util
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import sqlite3
import stat
import subprocess
import sys
import threading
import time
import tomllib
import urllib.parse
import urllib.error
import urllib.request
import uuid
import xml.etree.ElementTree as ET

ROOT = Path("E:/Git/ZirconEngine")
ID = re.compile(r"[A-Za-z0-9][A-Za-z0-9_.-]{0,79}\Z")
BEGIN = "// BEGIN ZIRCON SCOPED CHECKOUT COMMIT"
END = "// END ZIRCON SCOPED CHECKOUT COMMIT"


class CommitBusy(ValueError):
    code = "management_git_busy"


@contextmanager
def git_lock(path):
    """Use the same native byte lock as the other management Git routes."""
    path = physical(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a+b") as handle:
        if path.stat().st_size == 0:
            handle.write(b"\0")
            handle.flush()
        handle.seek(0)
        try:
            if os.name == "nt":
                import msvcrt
                msvcrt.locking(handle.fileno(), msvcrt.LK_NBLCK, 1)
            else:
                import fcntl
                fcntl.flock(handle.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
        except OSError as error:
            raise CommitBusy("Another management operation owns Git; continue the same request") from error
        try:
            yield
        finally:
            handle.seek(0)
            if os.name == "nt":
                msvcrt.locking(handle.fileno(), msvcrt.LK_UNLCK, 1)
            else:
                fcntl.flock(handle.fileno(), fcntl.LOCK_UN)


def physical(path):
    path = Path(os.path.abspath(path))
    for parent in (path, *path.parents):
        try:
            info = parent.lstat()
        except FileNotFoundError:
            continue
        if stat.S_ISLNK(info.st_mode) or getattr(info, "st_file_attributes", 0) & 0x400:
            raise ValueError("Commit paths must be physical paths")
    return path


def read_json(path):
    return json.loads(physical(path).read_text(encoding="utf-8-sig"))


def write_json(path, value):
    path = physical(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = physical(path.with_name(path.name + ".tmp"))
    with temporary.open("w", encoding="utf-8", newline="\n") as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write("\n")
        stream.flush()
        os.fsync(stream.fileno())
    os.replace(temporary, path)


def load_policy(repo):
    policy = read_json(repo / "tools/jenkins-management/local-commit-policy.json")
    if (policy.get("schemaVersion") != 1 or policy.get("enabled") is not True
            or physical(policy["repositoryRoot"]) != repo
            or policy.get("authorization", {}).get("source") != "user"
            or not policy["authorization"].get("instruction")
            or policy.get("pushAllowed") is not False
            or any(policy.get(field) is not False for field in
                   ("requireSessionGrant", "requireSourceDigest", "requireFormalAcceptance"))):
        raise ValueError("Scoped local commit policy is not enabled for this checkout")
    return policy


def selected_paths(repo, paths, policy, grants):
    if not isinstance(paths, list) or not paths or len(paths) > 256:
        raise ValueError("Select between 1 and 256 individual task files")
    result, seen = [], set()
    for name in paths:
        if (not isinstance(name, str) or not name or "\\" in name or ":" in name
                or name.startswith("/") or any(c in name for c in "\0\r\n*?[")
                or any(p in {"", ".", ".."} for p in name.split("/"))
                or any(p.rstrip(" .") != p for p in name.split("/"))
                or any(p.casefold() == ".git" for p in PurePosixPath(name).parts)
                or name.casefold().startswith(".jenkins/")
                or name.casefold() in seen):
            raise ValueError("Commit requires unique literal repository-relative file paths")
        target = physical(repo / name)
        if target.exists() and not target.is_file():
            raise ValueError("Select individual files, not directories")
        if not target.is_relative_to(repo):
            raise ValueError("Commit path escaped this checkout")
        matching = [(len(str(Path(root))), mode) for root, mode in grants.items()
                    if not root.startswith(":") and target.is_relative_to(Path(root))]
        allowed = max(matching, default=(0, "read"))[1] == "write"
        if not allowed and name not in policy.get("controlFiles", []):
            raise ValueError("Selected file has no project write grant: " + name)
        seen.add(name.casefold())
        result.append(name)
    return result


def native_processes():
    """Reuse the deployed runtime's atomic Windows Job launcher."""
    selected = read_json(ROOT / ".jenkins/state/deployment/driver.json")
    driver = physical(ROOT / ".jenkins/runtime/drivers" / selected["driverDigest"])
    path = physical(driver / "tools/jenkins/processes/native.py")
    spec = importlib.util.spec_from_file_location("checkout_commit_native", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def run_native_command(command, repo, environment, timeout=120):
    """Wait for the complete owned tree and close it on timeout or interruption."""
    native = native_processes()
    process, handle = native.create_atomic_kill_on_close_process(tuple(command), cwd=repo, env=environment)
    output = ["", ""]
    failures = []

    def drain(index, stream):
        try:
            output[index] = stream.read()
        except Exception as error:
            failures.append(str(error))

    readers = [threading.Thread(target=drain, args=(i, stream), daemon=True)
               for i, stream in enumerate((process.stdout, process.stderr))]
    try:
        for reader in readers:
            reader.start()
        native.resume_popen_process(process)
        try:
            code = process.wait(timeout=timeout)
            native.wait_for_process_job_terminal(handle, timeout_seconds=10)
        except BaseException:
            native.terminate_process_job(handle, timeout_seconds=10)
            process.wait(timeout=10)
            raise
        for reader in readers:
            reader.join(timeout=10)
        if any(reader.is_alive() for reader in readers) or failures:
            raise ValueError("Native Git streams did not terminate cleanly")
        return subprocess.CompletedProcess(command, code, output[0].encode("utf-8"), output[1].encode("utf-8"))
    finally:
        # Closing this retained handle kills only this command's owned Job.
        native.close_process_job(handle)
        for reader in readers:
            if reader.ident:
                reader.join(timeout=10)
        process.close()


def git(repo, *args, check=True):
    environment = {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}
    environment.update(GIT_LITERAL_PATHSPECS="1", GIT_TERMINAL_PROMPT="0",
                       GIT_OPTIONAL_LOCKS="0")
    executable = Path(shutil.which("git") or "git")
    if os.name == "nt" and executable.parent.name.casefold() == "cmd":
        actual = executable.parent.parent / "mingw64/bin/git.exe"
        if actual.is_file():
            executable = physical(actual)
    trace = os.environ.get("ZR_COMMIT_TRACE")
    if trace:
        trace_path = physical(trace)
        if not trace_path.is_relative_to(physical(Path("E:/cargo-targets/zircon-local/reports/jenkins-commit-audit-20261017"))):
            raise ValueError("Git trace output escaped its task report root")
        environment["GIT_TRACE2_EVENT"] = trace_path.as_posix()
    command = [str(executable), "-c", "core.quotepath=false", *args]
    if os.name == "nt" and os.environ.get("ZR_COMMIT_NATIVE") == "1":
        result = run_native_command(command, repo, environment)
    else:
        result = subprocess.run(command, cwd=repo, env=environment, stdin=subprocess.DEVNULL,
                                capture_output=True, timeout=120)
    if check and result.returncode:
        raise ValueError(result.stderr.decode("utf-8", "replace")[-2500:].strip()
                         or "Git command failed")
    return result


def checkout_reader(repo):
    database = physical(repo / ".jenkins/state/coordination.sqlite3")
    if not database.exists():
        return None
    with sqlite3.connect(database.as_uri() + "?mode=ro", uri=True, timeout=5) as connection:
        for key, raw in connection.execute(
                "SELECT key,payload FROM records WHERE domain='management_checkout_read'"):
            value = json.loads(raw)
            if value.get("status") in {"active", "unknown"}:
                return key
    return None


def recover_stale_index_lock(repo, operation_id):
    """Quarantine an abandoned empty lock, while holding its exclusive handle."""
    lock = physical(repo / ".git/index.lock")
    if not lock.exists():
        return None
    before = lock.stat()
    if before.st_size or time.time() - before.st_mtime < 600 or os.name != "nt":
        raise CommitBusy("Git index lock is recent or contains pending index data")
    import ctypes
    from ctypes import wintypes
    import msvcrt
    api = ctypes.WinDLL("kernel32", use_last_error=True)
    api.CreateFileW.argtypes = [wintypes.LPCWSTR, wintypes.DWORD, wintypes.DWORD,
                               ctypes.c_void_p, wintypes.DWORD, wintypes.DWORD, ctypes.c_void_p]
    api.CreateFileW.restype = wintypes.HANDLE
    api.CloseHandle.argtypes = [wintypes.HANDLE]
    api.CloseHandle.restype = wintypes.BOOL
    api.SetFileInformationByHandle.argtypes = [wintypes.HANDLE, ctypes.c_int, ctypes.c_void_p, wintypes.DWORD]
    api.SetFileInformationByHandle.restype = wintypes.BOOL
    # Share mode zero proves that no process still has this lock open for I/O.
    handle = api.CreateFileW(str(lock), 0x80000000 | 0x10000, 0, None, 3, 0x02200000, None)
    if handle == wintypes.HANDLE(-1).value:
        raise CommitBusy("Git index lock still has a native owner")
    descriptor = None
    try:
        descriptor = msvcrt.open_osfhandle(handle, os.O_RDONLY | os.O_BINARY)
        observed = os.fstat(descriptor)
        if ((observed.st_dev, observed.st_ino, observed.st_size, observed.st_mtime_ns)
                != (before.st_dev, before.st_ino, 0, before.st_mtime_ns)):
            raise CommitBusy("Git index lock changed during recovery")
        directory = physical(repo / ".jenkins/state/management/recovered-index-locks")
        directory.mkdir(parents=True, exist_ok=True)
        base = directory / (operation_id + "-" + str(time.time_ns()))
        destination = str(base) + ".lock"
        class RenameInfo(ctypes.Structure):
            _fields_ = [("replace", wintypes.BOOL), ("root", wintypes.HANDLE),
                        ("length", wintypes.DWORD), ("name", wintypes.WCHAR * (len(destination) + 1))]
        rename = RenameInfo(False, None, len(destination.encode("utf-16-le")), destination)
        evidence = {"status": "recovery-intent", "originalPath": str(lock), "retainedPath": destination,
                    "identity": [before.st_dev, before.st_ino], "bytes": 0,
                    "lastModified": before.st_mtime, "exclusiveNativeHandle": True,
                    "reason": "abandoned empty Git index lock older than ten minutes"}
        write_json(Path(str(base) + ".json"), evidence)
        if not api.SetFileInformationByHandle(handle, 3, ctypes.byref(rename), ctypes.sizeof(rename)):
            raise ctypes.WinError(ctypes.get_last_error())
        evidence.update(status="quarantined", recoveredAt=time.time())
        write_json(Path(str(base) + ".json"), evidence)
        return evidence
    finally:
        if descriptor is not None:
            os.close(descriptor)
        else:
            api.CloseHandle(handle)


def expected_file_hashes(payload, paths):
    """Optional validation pins apply only to the explicitly selected files."""
    if "expectedFileHashes" not in payload:
        return None
    expected = payload["expectedFileHashes"]
    if (not isinstance(expected, dict) or set(expected) != set(paths)
            or any(not isinstance(value, str) or not re.fullmatch(r"[0-9a-f]{64}", value)
                   for value in expected.values())):
        raise ValueError("expectedFileHashes must contain exactly the selected paths and SHA-256 values")
    return dict(expected)


@contextmanager
def verified_commit_files(repo, expected):
    """Reuse native read pins through commit; ordinary unbound requests stay unchanged."""
    if expected is None:
        yield
        return
    if os.name != "nt":
        raise ValueError("Validated checkout commits require Windows native read pins")
    path = ROOT / "tools/jenkins-management/checkout_guard.py"
    spec = importlib.util.spec_from_file_location("checkout_commit_read_guard", path)
    guard = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(guard)
    added_root = str(ROOT) not in sys.path
    if added_root:
        sys.path.insert(0, str(ROOT))
    hierarchy = None
    try:
        hierarchy = guard._PinnedHierarchy()
        for name, checksum in sorted(expected.items()):
            try:
                hierarchy.file(guard._absolute(physical(repo / name)), checksum)
            except guard.CheckoutGuardBusy as error:
                raise CommitBusy("A selected validated file has an active writer; continue the same request") from error
        hierarchy.verify()
        yield
        hierarchy.verify()
    finally:
        try:
            if hierarchy is not None:
                hierarchy.close()
        finally:
            if added_root:
                sys.path.remove(str(ROOT))


def commit_request(repo, payload, *, policy=None, grants=None):
    """Validate explicit task scope without reading or changing the Git index."""
    repo = physical(repo)
    policy = load_policy(repo) if policy is None else policy
    if grants is None:
        config = tomllib.loads((repo / ".codex/config.toml").read_text(encoding="utf-8-sig"))
        grants = config["permissions"]["zircon-strict"]["filesystem"]
    if payload.get("action") != "commit-checkout":
        raise ValueError("Unsupported commit action")
    for field in ("sessionId", "requestId", "operationId"):
        if not isinstance(payload.get(field), str) or not ID.fullmatch(payload[field]):
            raise ValueError("Invalid " + field)
    paths = selected_paths(repo, payload.get("paths"), policy, grants)
    expected = expected_file_hashes(payload, paths)
    message = payload.get("message")
    if not isinstance(message, str) or not message.strip() or len(message) > 4096 or "\0" in message:
        raise ValueError("A bounded commit message is required")
    if type(payload.get("observationOnly", False)) is not bool:
        raise ValueError("observationOnly must be boolean")
    request = {k: payload[k] for k in ("sessionId", "requestId", "operationId", "action", "paths", "message")}
    if expected is not None:
        request["expectedFileHashes"] = expected
    return request, policy


def execute(repo, payload, *, policy=None, grants=None):
    """Called under the shared management Git lock by the Jenkins worker."""
    repo = physical(repo)
    request, policy = commit_request(repo, payload, policy=policy, grants=grants)
    paths, message = request["paths"], request["message"]
    expected = request.get("expectedFileHashes")
    directory = physical(repo / ".jenkins/state/management/checkout-commits")
    record_path = physical(directory / (payload["operationId"] + ".json"))
    previous = read_json(record_path) if record_path.exists() else None
    if previous and previous.get("request") != request:
        raise ValueError("This operation ID already belongs to another commit request")
    if previous and previous["status"] in {"complete", "no-changes"}:
        return {**previous, "reusedReceipt": True}
    if payload.get("observationOnly") is True:
        return previous or {"status": "not-started", "request": request, "formalAcceptance": False}
    reader = checkout_reader(repo)
    if reader:
        return {"status": "waiting", "reasonCode": "checkout_source_busy", "blockingReader": reader,
                "request": request, "formalAcceptance": False}
    lock_recovery = recover_stale_index_lock(repo, payload["operationId"])
    head = git(repo, "rev-parse", "HEAD").stdout.decode().strip()
    trailer = "Zircon-Commit-Operation: " + payload["operationId"]
    if previous:
        commits = git(repo, "log", "-100", "--format=%H%x00%P%x00%B%x00",
                      "--fixed-strings", "--grep=" + trailer).stdout.decode("utf-8", "replace").split("\0")
        for i in range(0, len(commits) - 2, 3):
            commit, parents, body = commits[i:i + 3]
            if trailer in body.splitlines() and parents == previous.get("headBefore"):
                result = {**previous, "status": "complete", "commit": commit.strip(), "recoveredReceipt": True}
                write_json(record_path, result)
                return result
    with verified_commit_files(repo, expected):
        tracked = set(git(repo, "ls-files", "-z", "--", *paths).stdout.decode("utf-8").split("\0"))
        for name in paths:
            if not (repo / name).exists() and name not in tracked:
                raise ValueError("Selected file does not exist in Git or the checkout: " + name)
            mode = git(repo, "ls-files", "--stage", "--", name).stdout
            if mode.startswith(b"160000 "):
                raise ValueError("Submodule commits require their separate integration workflow")
        dirty = git(repo, "status", "--porcelain=v1", "-z", "--untracked-files=all", "--", *paths).stdout
        result = {"schemaVersion": 1, "status": "committing" if dirty else "no-changes",
                  "request": request, "headBefore": head, "paths": paths,
                  "authorization": policy["authorization"], "sourceDigestRequired": False,
                  "sessionGrantRequired": False, "formalAcceptance": False,
                  "pushPerformed": False, "notificationPerformed": False, "startedAt": time.time()}
        if previous:
            result["previousAttempts"] = [*previous.get("previousAttempts", []),
                {"headBefore": previous.get("headBefore"), "status": previous["status"],
                 "startedAt": previous.get("startedAt")}]
        if expected is not None:
            result["verifiedFileHashes"] = expected
        if lock_recovery:
            result["indexLockRecovery"] = lock_recovery
        write_json(record_path, result)
        if not dirty:
            return result
        # --only commits exactly these working files and retains other staged entries.
        # Add makes explicitly selected untracked files available to --only.
        git(repo, "add", "--", *paths)
        hooks = physical(repo / ".jenkins/state/management/empty-commit-hooks")
        hooks.mkdir(parents=True, exist_ok=True)
        if any(hooks.iterdir()):
            raise ValueError("The managed empty hooks directory must remain empty")
        git(repo, "-c", "core.hooksPath=" + str(hooks), "-c", "commit.gpgSign=false",
            "commit", "--only", "--no-gpg-sign", "-m", message.strip(), "-m", trailer, "--", *paths)
        commit = git(repo, "rev-parse", "HEAD").stdout.decode().strip()
        result.update(status="complete", commit=commit, finishedAt=time.time())
        write_json(record_path, result)
        return result


TERMINAL = {"complete", "no-changes"}
QUEUE_PENDING = {"pending", "dispatching", "submitted", "delivery-unknown", "waiting"}


def queue_record_path(repo, operation_id):
    if not isinstance(operation_id, str) or not ID.fullmatch(operation_id):
        raise ValueError("Invalid operationId")
    return physical(repo / ".jenkins/state/management/checkout-queue" / (operation_id + ".json"))


def queue_observation(repo, request):
    """Queries never enqueue builds or change a Git/request file."""
    journal = None
    for directory in ("checkout-commits", "checkout-queue"):
        path = physical(repo / ".jenkins/state/management" / directory / (request["operationId"] + ".json"))
        if path.exists():
            value = read_json(path)
            if value.get("request") != request:
                raise ValueError("This operation ID already belongs to another commit request")
            if directory == "checkout-commits" and value.get("status") not in TERMINAL:
                journal = value
                continue
            return value
    return journal or {"status": "not-started", "request": request, "formalAcceptance": False}


def queue_intake(repo, request, *, claim_delivery=False):
    """Record intent while allowing only one client to dispatch its initial build."""
    path = queue_record_path(repo, request["operationId"])
    with git_lock(repo / ".jenkins/state/locks/checkout-queue.lock"):
        previous = read_json(path) if path.exists() else None
        if previous and previous.get("request") != request:
            raise ValueError("This operation ID already belongs to another commit request")
        if previous and (previous["status"] in TERMINAL or (not claim_delivery and previous["status"] != "failed")
                         or (previous.get("deliveryAttemptedAt") and previous["status"] != "failed")):
            return previous, False
        value = {**(previous or {"schemaVersion": 1, "createdAt": time.time()}), "request": request,
                 "status": "dispatching" if claim_delivery else "pending", "updatedAt": time.time(),
                 "formalAcceptance": False, "pushPerformed": False, "notificationPerformed": False}
        if claim_delivery:
            value["deliveryAttemptedAt"] = time.time()
        elif previous and previous["status"] == "failed":
            value.pop("deliveryAttemptedAt", None)
        write_json(path, value)
        return value, True


def queue_update(repo, request, result, *, create=False):
    path = queue_record_path(repo, request["operationId"])
    with git_lock(repo / ".jenkins/state/locks/checkout-queue.lock"):
        previous = read_json(path) if path.exists() else None
        if previous and previous.get("request") != request:
            raise ValueError("This operation ID already belongs to another commit request")
        if previous and previous.get("status") in TERMINAL:
            return previous
        if previous is None and not create:
            return None
        value = {**(previous or {"schemaVersion": 1, "createdAt": time.time()}),
                 **result, "request": request, "updatedAt": time.time(), "formalAcceptance": False,
                 "pushPerformed": False, "notificationPerformed": False}
        write_json(path, value)
        return value


def submit_queued(repo, payload, client, *, query_only=False):
    request, _ = commit_request(repo, payload)
    observed = queue_observation(repo, request)
    if query_only or observed["status"] in TERMINAL:
        return observed
    # Persist the exact file list BEFORE any external delivery. A lost HTTP
    # response is recovered by the Jenkins followup under this same identity.
    value, claimed = queue_intake(repo, request, claim_delivery=True)
    if not claimed:
        return {**value, "status": "queued" if value["status"] not in TERMINAL else value["status"],
                "automaticFollowup": True}
    selected = read_json(repo / ".jenkins/state/deployment/driver.json")
    fields = {"ACTION": "commit-checkout", "SESSION_ID": request["sessionId"],
              "REQUEST_ID": request["requestId"], "OPERATION_ID": request["operationId"],
              "INPUT_JSON": json.dumps(request, ensure_ascii=False), "QUERY_ONLY": "false",
              "EXPECTED_DRIVER_DIGEST": selected["driverDigest"],
              "RUNTIME_OPERATION_ID": selected["runtimeOperationId"]}
    try:
        status, body, headers = client.request("/job/zircon-management/buildWithParameters", method="POST",
            data=urllib.parse.urlencode(fields).encode(), content_type="application/x-www-form-urlencoded")
        if status not in (201, 302):
            raise ValueError("Jenkins commit submission returned HTTP " + str(status))
        value = queue_update(repo, request, {"status": "submitted", "queueUrl": headers.get("Location")})
        return {**value, "status": "queued" if value["status"] not in TERMINAL else value["status"],
                "automaticFollowup": True}
    except Exception as error:
        value = queue_update(repo, request, {"status": "delivery-unknown", "lastDeliveryError": str(error)})
        return {**value, "status": "pending" if value["status"] not in TERMINAL else value["status"],
                "automaticFollowup": True}


def process_pending(repo, *, limit=12, budget_seconds=120):
    """Only explicit submitted commit requests are eligible; never sweep sources."""
    directory = physical(repo / ".jenkins/state/management/checkout-queue")
    results = []
    start = time.monotonic()
    if directory.exists():
        for path in sorted(directory.glob("*.json"), key=lambda entry: entry.stat().st_mtime_ns):
            if len(results) >= limit or time.monotonic() - start >= budget_seconds:
                break
            value = read_json(path)
            if value.get("status") not in QUEUE_PENDING:
                continue
            request = value["request"]
            if path != queue_record_path(repo, request["operationId"]):
                raise ValueError("Queued commit filename differs from its operation ID")
            try:
                with git_lock(repo / ".jenkins/state/locks/management-git.lock"):
                    result = execute(repo, request)
            except Exception as error:
                result = {"status": "waiting" if isinstance(error, CommitBusy) else "failed",
                          "reasonCode": getattr(error, "code", "scoped_commit_failed"), "message": str(error)}
            updated = queue_update(repo, request, result)
            results.append({"operationId": request["operationId"], "status": updated["status"],
                            "commit": updated.get("commit"), "reasonCode": updated.get("reasonCode")})
    failures = []
    if directory.exists():
        for path in directory.glob("*.json"):
            if read_json(path).get("status") == "failed":
                failures.append(path.stem)
    completed = sum(value["status"] in TERMINAL for value in results)
    waiting = sum(value["status"] == "waiting" for value in results)
    return {"status": "complete", "requests": results, "failedRequests": failures,
            "description": f"Git commits: {completed} completed; {waiting} waiting; {len(failures)} failed",
            "formalAcceptance": False,
            "pushPerformed": False, "notificationPerformed": False}


def transport(repo):
    # Keep the ordinary entry independent of uncommitted coordinator modules.
    credentials = read_json(repo / ".jenkins/state/credentials.json")
    token = credentials.get("apiToken") or credentials.get("password")
    if credentials.get("authenticationMode") != "api-token" or not token or not credentials.get("username"):
        raise ValueError("Configured local Jenkins API token is required")
    authorization = "Basic " + base64.b64encode((credentials["username"] + ":" + token).encode()).decode()

    class NoRedirect(urllib.request.HTTPRedirectHandler):
        def redirect_request(self, *args, **kwargs):
            return None

    class LocalTransport:
        def request(self, path, *, method="GET", data=None, content_type=None):
            if path.startswith(("http:", "https:", "//")):
                raise ValueError("Only the configured local Jenkins endpoint is allowed")
            headers = {"Authorization": authorization, "Accept": "application/json"}
            if content_type:
                headers["Content-Type"] = content_type
            request = urllib.request.Request("http://127.0.0.1:18080/" + path.lstrip("/"),
                data=data, method=method, headers=headers)
            try:
                with urllib.request.build_opener(NoRedirect()).open(request, timeout=30) as response:
                    return response.status, response.read(), dict(response.headers.items())
            except urllib.error.HTTPError as error:
                return error.code, error.read(), dict(error.headers.items())
    return LocalTransport()


def install(repo, client):
    """Extend the existing management job while retaining its other routes."""
    load_policy(repo)
    endpoint = "/job/zircon-management/config.xml"
    status, raw, _ = client.request(endpoint)
    if status != 200:
        raise ValueError("Existing Jenkins management job is unavailable")
    document = ET.fromstring(raw)
    script = document.find("definition/script")
    if script is None or "zircon-windows-agent" not in script.text or "--capsule" not in script.text:
        raise ValueError("The existing management job does not have its expected core routes")
    addon = (repo / "tools/jenkins-management/checkout-commit.groovy").read_text(encoding="utf-8-sig")
    original_script = script.text
    old_script = original_script
    if BEGIN in old_script:
        start, end = old_script.index(BEGIN), old_script.index(END) + len(END)
        old_script = old_script[:start] + old_script[end:].lstrip("\r\n")
    script.text = addon.rstrip() + "\n\n" + old_script
    action = document.find(".//parameterDefinitions/*[name='ACTION']")
    if action is not None and action.tag.endswith("ChoiceParameterDefinition"):
        choices = action.find("choices/a")
        if choices is None:
            raise ValueError("Unrecognized management action choice list")
        if "commit-checkout" not in [x.text for x in choices]:
            ET.SubElement(choices, "string").text = "commit-checkout"
    if script.text == original_script:
        return {"status": "installed", "job": "zircon-management", "action": "commit-checkout", "unchanged": True}
    updated = ET.tostring(document, encoding="utf-8", xml_declaration=True)
    # Existing running Pipeline builds retain their CPS program and capsule.
    status, body, _ = client.request(endpoint, method="POST", data=updated, content_type="application/xml")
    if status not in (200, 201, 302):
        raise ValueError("Management action installation failed: HTTP " + str(status))
    check_status, after, _ = client.request(endpoint)
    if check_status != 200 or ET.fromstring(after).findtext("definition/script") != script.text:
        raise ValueError("Management action installation has no matching read-back")
    return {"status": "installed", "job": "zircon-management", "action": "commit-checkout"}


def install_tests(repo, client):
    endpoint = "/job/zircon-management-tests/config.xml"
    status, raw, _ = client.request(endpoint)
    if status != 200:
        raise ValueError("Existing management regression job is unavailable")
    document = ET.fromstring(raw)
    script = document.find("definition/script")
    if script is None or "zircon-windows-agent" not in script.text:
        raise ValueError("Existing management regression job differs")
    begin, end = "// BEGIN ZIRCON SCOPED COMMIT TESTS", "// END ZIRCON SCOPED COMMIT TESTS"
    original = script.text
    previous = original
    if begin in previous:
        a, b = previous.index(begin), previous.index(end) + len(end)
        previous = previous[:a] + previous[b:].lstrip("\r\n")
    addon = (repo / "tools/jenkins-management/checkout-commit-tests.groovy").read_text(encoding="utf-8-sig")
    script.text = addon.rstrip() + "\n\n" + previous
    if script.text != original:
        status, _, _ = client.request(endpoint, method="POST",
            data=ET.tostring(document, encoding="utf-8", xml_declaration=True), content_type="application/xml")
        if status not in (200, 201, 302):
            raise ValueError("Commit regression route installation failed")
        status, after, _ = client.request(endpoint)
        if status != 200 or ET.fromstring(after).findtext("definition/script") != script.text:
            raise ValueError("Commit regression route read-back differs")
    return {"job": "zircon-management-tests", "status": "installed"}


def install_followup(repo, client):
    """Keep a bounded Jenkins timer for durable, explicitly scoped requests."""
    name = "zircon-git-followup"
    endpoint = "/job/" + name + "/config.xml"
    status, raw, _ = client.request(endpoint)
    if status == 404:
        document = ET.fromstring('''<flow-definition><description>Resume explicit local Git commit requests</description>
          <properties><org.jenkinsci.plugins.workflow.job.properties.DisableConcurrentBuildsJobProperty/>
          <jenkins.model.BuildDiscarderProperty><strategy class="hudson.tasks.LogRotator"><numToKeep>30</numToKeep>
          <daysToKeep>-1</daysToKeep><artifactDaysToKeep>-1</artifactDaysToKeep><artifactNumToKeep>-1</artifactNumToKeep>
          </strategy></jenkins.model.BuildDiscarderProperty>
          <org.jenkinsci.plugins.workflow.job.properties.PipelineTriggersJobProperty><triggers>
          <hudson.triggers.TimerTrigger><spec>H/5 * * * *</spec></hudson.triggers.TimerTrigger>
          </triggers></org.jenkinsci.plugins.workflow.job.properties.PipelineTriggersJobProperty></properties>
          <definition class="org.jenkinsci.plugins.workflow.cps.CpsFlowDefinition"><script/><sandbox>true</sandbox></definition>
          <disabled>false</disabled></flow-definition>''')
        post = "/createItem?name=" + name
    elif status == 200:
        document = ET.fromstring(raw)
        if "// ZIRCON DURABLE CHECKOUT COMMIT FOLLOWUP" not in document.findtext("definition/script", ""):
            raise ValueError("Existing Git followup job has an unknown owner")
        post = endpoint
    else:
        raise ValueError("Git followup job inspection failed: HTTP " + str(status))
    script = document.find("definition/script")
    source = (repo / "tools/jenkins-management/checkout-commit-followup.groovy").read_text(encoding="utf-8-sig")
    if (status == 200 and script.text == source and document.findtext("disabled") == "false"
            and document.findtext(".//hudson.triggers.TimerTrigger/spec") == "H/5 * * * *"):
        return {"status": "installed", "job": name, "unchanged": True}
    if status == 200:
        if document.findtext(".//hudson.triggers.TimerTrigger/spec") != "H/5 * * * *":
            raise ValueError("Git followup timer changed; repair its schedule before submitting")
        document.find("disabled").text = "false"
    script.text = source
    code, _, _ = client.request(post, method="POST", data=ET.tostring(document, encoding="utf-8", xml_declaration=True),
                                content_type="application/xml")
    if code not in (200, 201, 302):
        raise ValueError("Git followup job installation failed: HTTP " + str(code))
    code, raw, _ = client.request(endpoint)
    after = ET.fromstring(raw) if code == 200 else None
    if after is None or after.findtext("definition/script") != source or after.findtext("disabled") != "false":
        raise ValueError("Git followup job read-back differs or is disabled")
    return {"status": "installed", "job": name, "schedule": "H/5 * * * *"}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("install", "submit", "run", "followup"))
    parser.add_argument("--repo-root", type=Path, default=ROOT)
    parser.add_argument("--input-file", type=Path)
    parser.add_argument("--output-file", type=Path)
    parser.add_argument("--path", action="append", dest="paths")
    parser.add_argument("--message")
    parser.add_argument("--session-id")
    parser.add_argument("--request-id")
    parser.add_argument("--operation-id")
    parser.add_argument("--query-only", action="store_true")
    parser.add_argument("--defer", action="store_true", help="Leave the durable request for the next Jenkins followup")
    args = parser.parse_args(argv)
    repo = physical(args.repo_root)
    payload = None
    try:
        if repo != physical(ROOT):
            raise ValueError("The public entry is limited to the authorized checkout")
        if args.defer and (args.command != "submit" or args.query_only):
            raise ValueError("Deferred delivery requires a new submit, without --query-only")
        if args.command == "followup":
            if os.environ.get("JOB_NAME") != "zircon-git-followup" or os.environ.get("NODE_NAME") != "zircon-windows-agent":
                raise ValueError("Git followup requires its Jenkins job and fixed agent")
            os.environ["ZR_COMMIT_NATIVE"] = "1"
            result = process_pending(repo)
        elif args.command == "run":
            if os.environ.get("JOB_NAME") != "zircon-management" or os.environ.get("NODE_NAME") != "zircon-windows-agent":
                raise ValueError("Git mutations require the Jenkins management job and fixed agent")
            payload = read_json(args.input_file)
            for field, variable in (("sessionId", "ZR_COMMIT_SESSION"), ("requestId", "ZR_COMMIT_REQUEST"),
                                    ("operationId", "ZR_COMMIT_OPERATION")):
                if payload.get(field) != os.environ.get(variable):
                    raise ValueError("Commit request differs from authoritative Jenkins parameters")
            if payload.get("observationOnly", False) != (os.environ.get("ZR_COMMIT_QUERY", "false").lower() == "true"):
                raise ValueError("Commit query flag differs from Jenkins parameters")
            os.environ["ZR_COMMIT_NATIVE"] = "1"
            os.environ["ZR_COMMIT_TRACE"] = str(Path("E:/cargo-targets/zircon-local/reports/jenkins-commit-audit-20261017") /
                ("native-git-" + payload["operationId"] + "." + os.environ.get("BUILD_NUMBER", "unknown") + ".jsonl"))
            with git_lock(repo / ".jenkins/state/locks/management-git.lock"):
                result = execute(repo, payload)
            if payload.get("observationOnly") is not True:
                request, _ = commit_request(repo, payload)
                queue_update(repo, request, result)
        else:
            load_policy(repo)
            client = transport(repo)
            if args.command == "install":
                result = install(repo, client)
                result["regression"] = install_tests(repo, client)
                result["followup"] = install_followup(repo, client)
            else:
                payload = read_json(args.input_file) if args.input_file else {
                    "action": "commit-checkout", "paths": args.paths, "message": args.message,
                    "sessionId": args.session_id or "local-checkout", "requestId": args.request_id or "commit-" + uuid.uuid4().hex[:16],
                    "operationId": args.operation_id or "commit-" + uuid.uuid4().hex[:16]}
                payload["observationOnly"] = args.query_only
                for field, value in (("sessionId", args.session_id), ("requestId", args.request_id),
                                     ("operationId", args.operation_id)):
                    if value is not None and payload.get(field) != value:
                        raise ValueError("Commit input differs from explicit " + field)
                request, _ = commit_request(repo, payload)
                observed = queue_observation(repo, request)
                if args.query_only or observed["status"] in TERMINAL:
                    result = observed
                    if args.output_file:
                        write_json(args.output_file, result)
                    print(json.dumps(result, ensure_ascii=False))
                    return 0
                queue_intake(repo, request)
                # Repair only this extension if a normal management reinstall removed it.
                status, job, _ = client.request("/job/zircon-management/config.xml")
                if status != 200:
                    raise ValueError("Jenkins management job is unavailable")
                if BEGIN not in ET.fromstring(job).findtext("definition/script", ""):
                    install(repo, client)
                install_followup(repo, client)
                if args.defer:
                    result = queue_observation(repo, request)
                    if result["status"] not in TERMINAL:
                        result = {**result, "status": "queued", "automaticFollowup": True, "deferred": True}
                else:
                    result = submit_queued(repo, payload, client)
        if args.output_file:
            write_json(args.output_file, result)
            if args.command == "followup":
                description = physical(args.output_file.with_suffix(".description.txt"))
                description.write_text(result["description"] + "\n", encoding="utf-8")
        print(json.dumps(result, ensure_ascii=False))
        if args.command == "followup" and result.get("failedRequests"):
            return 3
        return 0 if result["status"] not in {"failed", "unknown", "waiting"} else 1
    except Exception as error:
        result = {"status": "waiting" if getattr(error, "code", None) == "management_git_busy" else "failed",
                  "reasonCode": getattr(error, "code", "scoped_commit_failed"),
                  "message": str(error), "formalAcceptance": False, "pushPerformed": False}
        if args.command == "run" and payload and payload.get("observationOnly") is not True:
            try:
                request, _ = commit_request(repo, payload)
                queue_update(repo, request, result)
            except Exception:
                # Retain the original failure. The durable input remains pending
                # if another short-lived queue writer owns its native lock.
                pass
        elif args.command == "submit" and payload and not args.query_only:
            try:
                request, _ = commit_request(repo, payload)
                pending = queue_observation(repo, request)
                if pending["status"] in QUEUE_PENDING:
                    result = {**pending, "automaticFollowup": True, "lastDeliveryError": str(error)}
            except Exception:
                pass
        if args.output_file:
            write_json(args.output_file, result)
        print(json.dumps(result, ensure_ascii=False))
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
