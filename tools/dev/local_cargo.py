"""Bounded independent local Cargo execution with observed-root command receipts.

No controller, installed capsule, identity file or service is consulted. The
historical native auditors remain library code; this CLI does not expose them
or claim process-tree, source-closure or compiler-artifact provenance.
"""
from __future__ import annotations

import argparse
from datetime import UTC, datetime
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import uuid

from tools.jenkins.contracts import JenkinsError
from tools.dev.agent_cargo_policy import admit_cargo, plain_path, reject_retired_options


MIN_FREE_BYTES = 35 * 1024**3
ARTIFACT_COMMANDS = {"build", "check", "test", "run", "bench", "clippy", "doc", "rustc"}


def storage_roots() -> tuple[Path, ...]:
    if os.name == "nt":
        return tuple(Path(f"{drive}:\\cargo-targets") for drive in "DEF")
    return tuple(Path(f"/mnt/{drive}/cargo-targets") for drive in "def")


def validate_output_path(value: str | Path) -> Path:
    if os.name == "nt":
        from tools.jenkins.resources.paths import build_root_containing, physical_path_under
        try:
            root = build_root_containing(value, allow_missing_root=True)
            requested = physical_path_under(root, value)
            if requested == root.path:
                raise ValueError("Output must be below, rather than equal to, a build root")
            return requested
        except JenkinsError as error:
            raise ValueError(str(error)) from error
    requested = Path(value)
    if not requested.is_absolute() or ".." in requested.parts:
        raise ValueError("Output must be an absolute physical path below D/E/F:\\cargo-targets")
    physical = requested.resolve()
    if physical != requested:
        raise ValueError(f"Output path aliases are not allowed: {requested}")
    for root in storage_roots():
        if root.resolve() == root and physical != root and physical.is_relative_to(root):
            return physical
    raise ValueError(f"Output must physically be below D/E/F drive-root cargo-targets: {requested}")



def free_bytes(path: Path) -> int:
    probe = path
    while not probe.exists() and probe != probe.parent:
        probe = probe.parent
    return shutil.disk_usage(probe).free


def prepare_command(arguments: list[str], target: Path, *, locked: bool = True) -> list[str]:
    if type(locked) is not bool:
        raise ValueError("Cargo locked control must be a boolean")
    args = list(arguments)
    separator = args.index("--") if "--" in args else len(args)
    options = args[:separator]
    if any(arg in {"--target-dir", "--config", "--artifact-dir", "--out-dir"} or arg.startswith(("--target-dir=", "--config=", "--artifact-dir=", "--out-dir=")) for arg in options):
        raise ValueError("Pass --target-dir to the local wrapper; Cargo --config output overrides are not accepted")
    command_index = 1 if options and options[0].startswith("+") else 0
    if len(options) <= command_index or options[command_index] not in ARTIFACT_COMMANDS:
        raise ValueError("Select build/check/test/run/bench/clippy/doc/rustc; use ordinary Cargo for read-only commands")
    if options[command_index] == "rustc" and separator < len(args):
        raise ValueError("Raw rustc flags are not admitted by this bounded wrapper")
    generated = ["--target-dir", str(validate_output_path(target))]
    if not locked and "--locked" in options:
        raise ValueError("Unlocked input conflicts with --locked")
    if locked and "--locked" not in options:
        generated.append("--locked")
    return ["cargo", *options, *generated, *args[separator:]]


def _git_metadata_path(value: str, base: Path) -> Path:
    """Resolve Git's relative control pointers, rejecting symlink/reparse aliases.

    Git commondir commonly contains ../..; those literal parent components are
    allowed only here, never by compiler-output admission. Check each existing
    prefix before lexical normalization so a link hidden before /.. is rejected.
    """
    if not value or value != value.strip() or any(character in value for character in "\0\r\n"):
        raise ValueError("Invalid Git control-directory pointer")
    path = Path(value)
    if not path.is_absolute():
        path = base / path
    current = Path(path.anchor)
    for component in path.parts[1:]:
        if component == "..":
            current = current.parent
        elif component != ".":
            current /= component
        if current.is_symlink() or (current.exists() and getattr(current.lstat(), "st_file_attributes", 0) & 0x400):
            raise ValueError("Git cache identity cannot use a source/control path alias")
    return plain_path(Path(os.path.abspath(path)))


def _git_pointer(path: Path) -> str:
    path = plain_path(path)
    if not path.is_file() or path.stat().st_size > 32768:
        raise ValueError("Invalid Git cache-identity control file")
    value = path.read_text(encoding="utf-8").rstrip("\r\n")
    if not value or any(character in value for character in "\0\r\n"):
        raise ValueError("Invalid Git cache-identity control file")
    return value


def repository_cache_identity(repo: Path) -> dict:
    """Read only .git and optional commondir, never Git config/credentials/state.

    Linked worktrees share the physical common directory identity. A genuinely
    non-Git source fixture uses its own root; broken Git pointers fail closed
    instead of silently allocating another per-worktree cache.
    """
    repo = plain_path(repo)
    marker = plain_path(repo / ".git")
    if not marker.exists():
        return {"kind": "source-root-without-git", "path": os.path.normcase(str(repo))}
    if marker.is_dir():
        gitdir = marker
    else:
        value = _git_pointer(marker)
        if not value.startswith("gitdir: "):
            raise ValueError("Invalid .git worktree control pointer")
        gitdir = _git_metadata_path(value[len("gitdir: "):], repo)
    if not gitdir.is_dir():
        raise ValueError("Git cache-identity directory is missing")
    pointer = plain_path(gitdir / "commondir")
    common = _git_metadata_path(_git_pointer(pointer), gitdir) if pointer.exists() else gitdir
    if not common.is_dir():
        raise ValueError("Git common directory for cache identity is missing")
    identity = common.stat()
    return {"kind": "git-common-directory", "path": os.path.normcase(str(common)),
            "device": identity.st_dev, "fileId": identity.st_ino}


def _default_target(repo: Path, args: list[str], *, dry_run: bool, build_root: Path | None = None) -> Path:
    # Stable across linked trees and preview/execution. Cargo owns profile/target
    # subdirectories, feature fingerprints and the native writer lock; these do
    # not create another entire Cargo home/target pool per package or worktree.
    # The versioned namespace never adopts retired coordinator pools/job scratch.
    roots = storage_roots()
    root = build_root or next((candidate for candidate in roots if candidate.parent.exists()), roots[0])
    toolchain = plain_path(repo / "rust-toolchain.toml")
    legacy_toolchain = plain_path(repo / "rust-toolchain")
    toolchain_input = (toolchain.read_text(encoding="utf-8") if toolchain.is_file() else
                       legacy_toolchain.read_text(encoding="utf-8") if legacy_toolchain.is_file() else "default-installed")
    host_arch = os.environ.get("PROCESSOR_ARCHITEW6432") or os.environ.get("PROCESSOR_ARCHITECTURE")
    if not host_arch and hasattr(os, "uname"):
        host_arch = os.uname().machine
    identity = json.dumps({"schema": "agent-common-git-pool-v1", "repository": repository_cache_identity(repo),
                           "platform": sys.platform, "hostArchitecture": host_arch, "toolchain": toolchain_input},
                          sort_keys=True).encode("utf-8")
    # Leave room for libgit2's MAX_PATH-bound checkout of the pinned zr_vm dependency.
    # This short, separate namespace must not adopt any historical build pool.
    key = hashlib.sha256(identity).hexdigest()[:16]
    return root / "zircon-local" / "a" / sys.platform / key / "target"


def _evidence_output(value: str | Path) -> Path:
    output = validate_output_path(value)
    root = next(root for root in storage_roots() if output.is_relative_to(root))
    namespace = root / "zircon-local"
    if output == namespace or not output.is_relative_to(namespace):
        raise ValueError("Local evidence must physically be below D/E/F:\\cargo-targets\\zircon-local")
    return output


def _file_fact(path: Path) -> dict:
    from tools.jenkins.pilot.source_manifest import file_hash
    before = path.stat()
    identity = file_hash(path)
    after = path.stat()
    fields = ("st_dev", "st_ino", "st_size", "st_mtime_ns", "st_ctime_ns")
    if any(getattr(before, field) != getattr(after, field) for field in fields):
        raise ValueError(f"Evidence file changed while fingerprinting: {path}")
    return {"path": str(path), "sha256": identity, "byteLength": after.st_size}


def _compiled_file_fact(path: Path, generation: Path, *, target_directory: str) -> dict:
    """Hash an executable inside the caller's exact compiler target/build roots."""
    if os.name != "nt":
        raise ValueError("Compiled artifact identity proof requires Windows")
    import ntpath
    from tools.jenkins.pilot.native.paths import (
        _verify_local_drive_mapping, _verify_pinned_local_storage_mapping,
    )
    from tools.jenkins.pilot.native.windows_storage import WindowsStorage

    drive, _tail = ntpath.splitdrive(str(generation))
    expected_device = _verify_local_drive_mapping(drive)
    storage = WindowsStorage(generation, expected_device=expected_device)
    try:
        _verify_pinned_local_storage_mapping(storage, generation, expected_device)
        relative = path.relative_to(generation).as_posix()
        digest, size = storage.compiled_file_fact(
            relative, allowed_directories=(target_directory, "build")
        )
        _verify_pinned_local_storage_mapping(storage, generation, expected_device)
        return {"path": str(path), "sha256": digest, "byteLength": size}
    finally:
        storage.close()


def _write_fact(path: Path, value: object) -> dict:
    path = _evidence_output(path)
    with path.open("x", encoding="utf-8", newline="\n") as stream:
        json.dump(value, stream, ensure_ascii=True, sort_keys=True, allow_nan=False)
        stream.write("\n")
        stream.flush()
        os.fsync(stream.fileno())
    return _file_fact(_evidence_output(path))


def _input_request(path: Path, repo: Path) -> tuple[dict, dict]:
    fact = _file_fact(path)
    value = json.loads(path.read_text(encoding="utf-8"))
    if _file_fact(path) != fact:
        raise ValueError("Original selected-input request changed during read")
    if not isinstance(value, dict) or set(value) != {"schemaVersion", "repositoryRoot", "inputs"} or type(value["schemaVersion"]) is not int or value["schemaVersion"] != 1:
        raise ValueError("Unsupported local selected-input request")
    if value["repositoryRoot"] != str(repo) or not isinstance(value["inputs"], dict) or not value["inputs"]:
        raise ValueError("Local selected-input request must name this exact repository and nonempty input map")
    return value, fact


def _source_facts(repo: Path, inputs: dict) -> dict:
    import re
    from tools.jenkins.pilot.source_manifest import file_hash
    from tools.jenkins.pilot.snapshot import portable_path
    observed, folded = {}, set()
    for name, requested in inputs.items():
        if not isinstance(name, str) or not re.fullmatch(r"[A-Za-z0-9._/-]+", name) or portable_path(name) != name:
            raise ValueError("Selected input path must be canonical portable ASCII")
        if name.casefold() in folded or ".git" in name.casefold().split("/"):
            raise ValueError("Selected inputs contain a case alias or Git control path")
        folded.add(name.casefold())
        if requested is not None and (not isinstance(requested, str) or not re.fullmatch(r"[0-9a-f]{64}", requested)):
            raise ValueError("Selected input identities must be lowercase SHA-256 or null")
        if name.startswith("external-zr_vm/"):
            root, relative = repo.parent / "zr_vm", name[len("external-zr_vm/"):]
        else:
            root, relative = repo, name
        if not relative:
            raise ValueError("Selected input must name a file")
        try:
            observed[name] = file_hash(root / relative)
        except FileNotFoundError:
            observed[name] = None
    # Version 1 declares ASCII ordinal keys, compact JSON, explicit null absence.
    encoded = json.dumps(observed, sort_keys=True, separators=(",", ":"), ensure_ascii=True, allow_nan=False).encode("utf-8")
    return {"protocol": "zircon-local-selected-source-map-v1", "scope": "requested-selected-files",
            "immutableInputSeal": False, "manifest": observed,
            "manifestSha256": hashlib.sha256(encoded).hexdigest()}


def _toolchain_facts(repo: Path) -> dict:
    from tools.jenkins.pilot.managed_cargo import toolchain_identity
    # Pure installed-file discovery: no rustup/native command or authority issuance.
    result = toolchain_identity(repo)
    installed = Path(result["binaries"]["cargo"]["path"]).parent.parent
    files = {}
    for value in result["binaries"].values():
        fact = _file_fact(Path(value["path"]))
        if fact["sha256"] != value["sha256"]:
            raise ValueError("Installed compiler changed during discovery")
        files[fact["path"]] = fact
    for name, identity in {**result["libraries"], **result["sysrootInputs"]}.items():
        fact = _file_fact(installed / name)
        if fact["sha256"] != identity:
            raise ValueError("Installed compiler libraries changed during discovery")
        files[fact["path"]] = fact
    python = _file_fact(Path(result["python"]["path"]))
    settings = _file_fact(Path(os.environ.get("RUSTUP_HOME", str(Path.home() / ".rustup"))) / "settings.toml")
    if python["sha256"] != result["python"]["sha256"] or settings["sha256"] != result["settingsSha256"]:
        raise ValueError("Python or installed toolchain settings changed during discovery")
    files[python["path"]], files[settings["path"]] = python, settings
    result["plainFileFacts"] = files
    return result


def _cargo_artifacts(stdout: Path, target: Path, build: Path) -> dict:
    target, build = (validate_output_path(root) for root in (target, build))
    if build != target.parent / "build":
        raise ValueError("Cargo build output must be the exact owned target sibling")
    if target == build:
        raise ValueError("Evidence target and build directories must be distinct")
    artifacts, finished = [], []
    generation = target.parent
    with stdout.open(encoding="utf-8") as stream:
        for line in stream:
            message = json.loads(line)
            if not isinstance(message, dict):
                raise ValueError("Cargo JSON output contains a non-object message")
            if message.get("reason") == "build-finished":
                finished.append(message)
            if message.get("reason") != "compiler-artifact" or message.get("executable") is None:
                continue
            executable = validate_output_path(message["executable"])
            if not any(executable.is_relative_to(root) for root in (target, build)):
                raise ValueError("Cargo executable artifact escaped the exact target/build directories")
            artifacts.append({"packageId": message["package_id"], "manifestPath": message["manifest_path"],
                              "target": message["target"], "profile": message["profile"],
                              "fresh": message["fresh"],
                              "binary": _compiled_file_fact(executable, generation,
                                                            target_directory=target.name)})
    return {"outputDirectories": [str(target), str(build)],
            "buildFinished": finished, "executableArtifacts": artifacts}


def _run_owned_command(command: list[str], repo: Path, environment: dict[str, str], output: Path, timeout: float) -> dict:
    import ctypes
    import msvcrt
    import threading
    import time
    from ctypes import wintypes
    from types import SimpleNamespace
    from tools.jenkins.pilot.native.process_identity import popen_process_creation_time
    from tools.jenkins.pilot.native.windows_job_process import (
        close_process_job, create_atomic_kill_on_close_process, resume_popen_process,
        terminate_and_close_process_job, wait_for_process_job_terminal,
    )
    from tools.jenkins.pilot.process_evidence import JobEvidence
    if os.name != "nt" or not 0 < timeout <= 86400:
        raise ValueError("Owned local evidence requires Windows and a bounded positive timeout")
    cleanup_seconds = 30.0
    deadline = time.monotonic() + timeout
    observer_deadline = deadline + cleanup_seconds + 1.0
    process, job, query, waiter, birth = None, None, None, None, None
    waiter_started = False
    native_done = threading.Event()
    native_result = {"terminal": False}
    pipes = {name: {"eof": False, "closed": False} for name in ("stdout", "stderr")}
    logs, errors, wait_errors, root_exit = {}, [], [], None
    error_lock = threading.Lock()
    raw, handles = {}, {}
    cleanup = {"attempted": False, "budgetSeconds": cleanup_seconds, "complete": False,
               "terminationRequested": False, "jobTerminationProven": False,
               "rootSignaled": False, "pipeEOF": False, "nativeWaiterJoined": False,
               "originalWaiterProven": False, "errors": []}
    api = ctypes.WinDLL("kernel32", use_last_error=True)
    api.PeekNamedPipe.argtypes = [wintypes.HANDLE, ctypes.c_void_p, wintypes.DWORD,
                                 ctypes.c_void_p, ctypes.POINTER(wintypes.DWORD), ctypes.c_void_p]
    api.PeekNamedPipe.restype = wintypes.BOOL
    api.GetCurrentProcess.restype = wintypes.HANDLE
    api.DuplicateHandle.argtypes = [wintypes.HANDLE, wintypes.HANDLE, wintypes.HANDLE,
                                   ctypes.POINTER(wintypes.HANDLE), wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
    api.DuplicateHandle.restype = wintypes.BOOL
    api.CloseHandle.argtypes = [wintypes.HANDLE]
    api.CloseHandle.restype = wintypes.BOOL

    def fail(stage, error, *, observer=False, during_cleanup=False):
        item = {"stage": stage, "type": type(error).__name__, "message": str(error)}
        with error_lock:
            errors.append(item)
            if observer:
                wait_errors.append(item)
            if during_cleanup:
                cleanup["errors"].append(item)

    def observed_wait_errors():
        with error_lock:
            return bool(wait_errors)

    def wait_native():
        try:
            wait_for_process_job_terminal(job, timeout_seconds=max(0.001, observer_deadline - time.monotonic()))
            native_result["terminal"] = True
        except BaseException as error:
            fail("owned_job_wait", error, observer=True)
        finally:
            native_done.set()

    def prepare_pipe_handles():
        for name in pipes:
            if name not in raw:
                buffer = getattr(process, name).buffer
                raw[name] = getattr(buffer, "raw", buffer)
                handles[name] = msvcrt.get_osfhandle(raw[name].fileno())

    def drain_once():
        for name in pipes:
            if pipes[name]["eof"]:
                continue
            available = wintypes.DWORD()
            if not api.PeekNamedPipe(handles[name], None, 0, None, ctypes.byref(available), None):
                code = ctypes.get_last_error()
                if code not in (109, 232):
                    raise ctypes.WinError(code)
                data = raw[name].read(1)
            elif available.value:
                data = raw[name].read(min(65536, available.value))
            else:
                continue
            if data == b"":
                pipes[name]["eof"] = True
            elif data is None:
                raise OSError("Raw pipe read made no progress")
            else:
                remaining = memoryview(data)
                while remaining:
                    written = logs[name].write(remaining)
                    if not written:
                        raise OSError("Evidence output write made no progress")
                    remaining = remaining[written:]

    try:
        for name in pipes:
            path = _evidence_output(output / (name + ".bin"))
            logs[name] = path.open("xb", buffering=0)
        process, job = create_atomic_kill_on_close_process(tuple(command), cwd=repo, env=environment)
        if not job:
            raise OSError("Retained Windows Job handle is missing")
        birth = popen_process_creation_time(process)
        if not birth or not birth.isdigit():
            raise OSError("Retained root creation time is unavailable")
        query = JobEvidence(SimpleNamespace(job_handle=job))
        query.capture("beforeResume", members=True)
        if query.errors:
            raise OSError("Owned Job query evidence is unavailable")
        _write_fact(output / "started.json", {"schemaVersion": 1, "command": command, "cwd": str(repo),
                    "pid": process.pid, "creationTime": birth, "scope": "retained-windows-job",
                    "formalAcceptance": False})
        prepare_pipe_handles()
        waiter = threading.Thread(target=wait_native, name="local-cargo-owned-job-wait", daemon=True)
        waiter.start()
        waiter_started = True
        resume_popen_process(process)
        while True:
            drain_once()
            root_exit = process.poll()
            if observed_wait_errors():
                raise OSError("Owned native waiter failed")
            if root_exit is not None and native_done.is_set() and all(p["eof"] for p in pipes.values()):
                break
            if time.monotonic() >= deadline:
                raise TimeoutError("Local Cargo owned Job or pipe EOF exceeded deadline")
            time.sleep(0.02)
        waiter.join(timeout=2)
        if waiter.is_alive() or not native_result["terminal"]:
            raise OSError("Owned Job waiter did not finish")
        root_exit = process.wait(timeout=max(0.001, deadline - time.monotonic()))
        if query.capture("afterTerminal")["activeProcesses"] != 0 or query.errors:
            raise OSError("Owned Job terminal accounting is incomplete")
    except BaseException as error:
        fail("command_or_pipe", error)
        if job:
            cleanup.update(attempted=True, reason="timeout" if isinstance(error, TimeoutError)
                           or time.monotonic() >= deadline else "command-or-observer-error",
                           identity={"pid": process.pid if process is not None else None,
                                     "creationTime": birth},
                           method="freeze-owned-members-and-wait-retained-process-handles")
            cleanup_deadline = time.monotonic() + cleanup_seconds
            if query is not None:
                query.capture("beforeCleanup", members=True)
            duplicate = wintypes.HANDLE()
            try:
                local = api.GetCurrentProcess()
                if not api.DuplicateHandle(local, job, local, ctypes.byref(duplicate), 0, False, 2):
                    raise ctypes.WinError(ctypes.get_last_error())
                # This helper consumes only the duplicate. The waiter and query
                # still own the original exact Job until native/EOF settlement.
                termination_handle = duplicate.value
                duplicate = wintypes.HANDLE()
                cleanup["terminationRequested"] = True
                terminate_and_close_process_job(
                    termination_handle, timeout_seconds=max(0.001, cleanup_deadline - time.monotonic()))
                cleanup["jobTerminationProven"] = True
            except BaseException as cleanup_error:
                fail("owned_job_cleanup", cleanup_error, during_cleanup=True)
            finally:
                if duplicate.value and not api.CloseHandle(duplicate):
                    fail("cleanup_duplicate_close", ctypes.WinError(ctypes.get_last_error()), during_cleanup=True)
            if process is not None:
                try:
                    prepare_pipe_handles()
                    while time.monotonic() < cleanup_deadline:
                        drain_once()
                        root_exit = process.poll()
                        if root_exit is not None and (not waiter_started or native_done.is_set()) and all(
                                p["eof"] for p in pipes.values()):
                            break
                        time.sleep(0.02)
                    if not all(p["eof"] for p in pipes.values()):
                        raise TimeoutError("Cleanup did not observe both actual pipe EOFs")
                except BaseException as cleanup_error:
                    fail("cleanup_pipe", cleanup_error, during_cleanup=True)
                # Pipe failure must not discard the held root's independent
                # signal/exit proof or prevent bounded native waiter settlement.
                try:
                    root_exit = process.wait(timeout=max(0.001, cleanup_deadline - time.monotonic()))
                    cleanup["rootSignaled"] = True
                except BaseException as cleanup_error:
                    fail("cleanup_root_wait", cleanup_error, during_cleanup=True)
            if waiter_started:
                waiter.join(timeout=max(0.001, cleanup_deadline - time.monotonic()))
                if waiter.is_alive():
                    fail("cleanup_waiter_join", TimeoutError("Owned native waiter did not join within cleanup budget"),
                         during_cleanup=True)
            if query is not None:
                query.capture("afterCleanup")
    finally:
        if process is not None:
            for name in pipes:
                try:
                    getattr(process, name).close()
                    pipes[name]["closed"] = True
                except BaseException as error:
                    fail("pipe_close", error, during_cleanup=cleanup["attempted"])
        for log in logs.values():
            try:
                log.close()
            except BaseException as error:
                fail("log_close", error, during_cleanup=cleanup["attempted"])
        waiter_joined = not waiter_started or not waiter.is_alive()
        if job and waiter_joined:
            try:
                close_process_job(job)
                if query is not None:
                    query.capture("afterClose")
            except BaseException as error:
                fail("owned_job_close", error, during_cleanup=cleanup["attempted"])
        # An unjoined observer keeps its original Job alive until this failed
        # CLI exits; no EOF, termination or cleanup success is inferred.
        if query is not None:
            try:
                query.close()
            except BaseException as error:
                fail("query_handle_close", error, during_cleanup=cleanup["attempted"])
        if process is not None:
            try:
                process.close(close_streams=False)
            except BaseException as error:
                fail("root_handle_close", error, during_cleanup=cleanup["attempted"])
    job_evidence = query.to_dict() if query is not None else {"records": {}, "errors": []}
    after_close = job_evidence["records"].get("afterClose", {})
    after_cleanup = job_evidence["records"].get("afterCleanup", {})
    pipe_eof = all(p["eof"] for p in pipes.values())
    original_wait_proven = (not waiter_started or
                            (waiter_joined and native_result["terminal"] and not observed_wait_errors()))
    cleanup.update(pipeEOF=pipe_eof, nativeWaiterJoined=waiter_joined, originalWaiterProven=original_wait_proven)
    cleanup["complete"] = bool(cleanup["attempted"] and cleanup["jobTerminationProven"] and cleanup["rootSignaled"]
                               and original_wait_proven and pipe_eof and after_cleanup.get("activeProcesses") == 0
                               and after_close.get("activeProcesses") == 0
                               and not cleanup["errors"] and not job_evidence["errors"])
    with error_lock:
        archived_errors = [dict(item) for item in errors]
    complete = (root_exit is not None and waiter_joined and native_result["terminal"]
                and pipe_eof and after_close.get("activeProcesses") == 0
                and not archived_errors and not job_evidence["errors"])
    return {"schemaVersion": 1, "command": command, "cwd": str(repo),
            "pid": process.pid if process is not None else None,
            "creationTime": birth, "rootExitCode": root_exit,
            "processTree": {"scope": "retained-windows-job", "terminal": complete,
                            "nativeWaiterJoined": waiter_joined, "readersFinished": True,
                            "pipes": pipes, "pipeEOF": pipe_eof,
                            "jobHandleRetainedUntilCliExit": bool(job and not waiter_joined)},
            "nativeJobEvidence": job_evidence, "terminalErrors": archived_errors,
            "cleanup": cleanup, "formalAcceptance": False}


def _run_with_evidence(command: list[str], repo: Path, target: Path, environment: dict[str, str],
                       output_value: str, input_manifest: str, timeout: float) -> int:
    if command[1].startswith("+") or command[1] not in {"build", "test"}:
        raise ValueError("Local artifact evidence admits build or test --no-run without a rustup override")
    if command[1] == "test" and "--no-run" not in command:
        raise ValueError("Local test artifact evidence requires --no-run")
    if "--" in command or command.count("--message-format=json") != 1:
        raise ValueError("Local artifact evidence requires one literal --message-format=json and no binary arguments")
    target = validate_output_path(target)
    build = validate_output_path(environment.get("CARGO_BUILD_BUILD_DIR", ""))
    if validate_output_path(environment.get("CARGO_TARGET_DIR", "")) != target or build != target.parent / "build":
        raise ValueError("Cargo environment must bind the exact owned target and build sibling")
    if target == build:
        raise ValueError("Evidence target and build directories must be distinct")
    output = _evidence_output(output_value)
    validate_output_path(output.parent).mkdir(parents=True, exist_ok=True)
    _evidence_output(output).mkdir(exist_ok=False)
    _evidence_output(output)
    selected, original_input = _input_request(Path(input_manifest), repo)
    before = _source_facts(repo, selected["inputs"])
    before_ref = _write_fact(output / "inputs-before.json", before)
    if before["manifest"] != selected["inputs"]:
        raise ValueError("Original selected-input request drifted before native start")
    compiler = _toolchain_facts(repo)
    actual = [compiler["binaries"]["cargo"]["path"], *command[1:]]
    environment = dict(environment, RUSTC=compiler["binaries"]["rustc"]["path"],
                       RUSTDOC=compiler["binaries"]["rustdoc"]["path"])
    driver_names = [
        "tools/dev/local_cargo.py", "tools/dev/local-cargo.ps1",
        "tools/dev/agent_cargo_policy.py", "tools/dev/agent-cargo-common.ps1",
        "tools/jenkins/pilot/native/windows_job_process.py",
        "tools/jenkins/pilot/native/process_identity.py",
        "tools/jenkins/pilot/process_evidence.py", "tools/jenkins/pilot/source_manifest.py",
        "tools/jenkins/pilot/managed_cargo.py", "tools/jenkins/resources/paths.py",
        "tools/jenkins/pilot/native/paths.py",
        "tools/jenkins/pilot/native/windows_storage.py",
        "tools/jenkins/pilot/native/compiled_products.py",
    ]
    drivers = {name: _file_fact(repo / name) for name in driver_names}
    environment_names = {"PATH", "INCLUDE", "LIB", "LIBPATH", "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS",
                         "RUSTC", "RUSTDOC", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER",
                         "CARGO_HOME", "CARGO_TARGET_DIR", "CARGO_BUILD_BUILD_DIR", "TEMP", "TMP", "TMPDIR"}
    environment_binding = {key: value for key, value in environment.items()
                           if key.upper() in environment_names or key.startswith(("CC_", "CXX_", "AR_", "CARGO_TARGET_", "CARGO_PROFILE_"))}
    request = {"schemaVersion": 1, "origin": "independent-local-command", "cwd": str(repo),
               "literalPreparedCommand": command, "command": actual, "targetDirectory": str(target),
               "buildDirectory": str(build), "artifactOutputDirectories": [str(target), str(build)],
               "selectedInputRequest": original_input, "inputsBefore": before_ref,
               "sourceScope": before["scope"], "sourceManifestProtocol": before["protocol"],
               "compiler": compiler, "driverFiles": drivers, "buildEnvironment": environment_binding,
               "immutableInputSeal": False, "formalAcceptance": False}
    request_ref = _write_fact(output / "request.json", request)
    native = _run_owned_command(actual, repo, environment, output, timeout)
    native_ref = _write_fact(output / "native-terminal.json", native)
    receipt = {"schema": "zircon_local_cargo_command_evidence_v1", "origin": request["origin"],
               "status": "incomplete", "request": request_ref, "nativeTerminal": native_ref,
               "rootExitCode": native["rootExitCode"], "selectedInputsUnchanged": False,
               "sourceManifestProtocol": before["protocol"], "sourceManifestSha256": before["manifestSha256"],
               "sourceScope": before["scope"], "immutableInputSeal": False,
               "compilerInputClosureVerified": False, "captureAdmission": False,
               "JenkinsAcceptance": False, "milestoneAcceptance": False, "formalAcceptance": False}
    try:
        if not native["processTree"]["terminal"]:
            raise ValueError("Retained Job/pipe/native accounting proof is incomplete")
        after = _source_facts(repo, selected["inputs"])
        receipt["inputsAfter"] = _write_fact(output / "inputs-after.json", after)
        receipt["selectedInputsUnchanged"] = after == before
        receipt["compilerUnchanged"] = _toolchain_facts(repo) == compiler
        receipt["driversUnchanged"] = {name: _file_fact(repo / name) for name in driver_names} == drivers
        if not receipt["selectedInputsUnchanged"] or not receipt["compilerUnchanged"] or not receipt["driversUnchanged"]:
            raise ValueError("Selected source, installed compiler or driver changed during actual command")
        artifacts = _cargo_artifacts(output / "stdout.bin", target, build)
        receipt["driversUnchanged"] = {name: _file_fact(repo / name) for name in driver_names} == drivers
        if not receipt["driversUnchanged"]:
            raise ValueError("Evidence driver changed during native artifact auditing")
        receipt["cargoArtifacts"] = _write_fact(output / "cargo-artifacts.json", artifacts)
        if native["rootExitCode"] == 0 and (len(artifacts["buildFinished"]) != 1 or artifacts["buildFinished"][0].get("success") is not True):
            raise ValueError("Successful root lacks its actual unique Cargo build-finished message")
        receipt["stdout"] = _file_fact(output / "stdout.bin")
        receipt["stderr"] = _file_fact(output / "stderr.bin")
        receipt["status"] = "completed-selected-input-command" if native["rootExitCode"] == 0 else "command-failed"
    except (OSError, ValueError, KeyError, TypeError) as error:
        receipt["evidenceError"] = {"type": type(error).__name__, "message": str(error)}
    receipt_ref = _write_fact(output / "receipt.json", receipt)
    print(json.dumps({"status": receipt["status"], "receipt": receipt_ref,
                      "captureAdmission": False, "formalAcceptance": False}, ensure_ascii=True))
    if receipt["status"] == "incomplete":
        return native["rootExitCode"] if native["rootExitCode"] not in (None, 0) else 2
    return native["rootExitCode"]


def _build_root(value: str | Path) -> Path:
    if os.name == "nt":
        from tools.jenkins.resources.paths import canonical_build_root
        try:
            return canonical_build_root(value, allow_missing=True).path
        except JenkinsError as error:
            raise ValueError(str(error)) from error
    root = Path(value)
    if root not in storage_roots() or root.resolve() != root:
        raise ValueError("Build root must be the physical D/E/F drive-root cargo-targets")
    return root


def local_environment(target: Path) -> tuple[dict[str, str], list[Path]]:
    """Prepare values only; reuse the selected pool's sibling Cargo home in place.

    A cold home is not an offline dependency cache. Callers may use standard
    Cargo online resolution or explicitly select an already compatible pool;
    this wrapper never copies caches, edits TLS/config, or restores Jenkins.
    MSVC must already be prepared by the caller. Unlisted toolchain/product
    environment variables are deliberately not inherited.
    """
    target = validate_output_path(target)
    generation = target.parent
    directories = {name: validate_output_path(generation / name)
                   for name in ("build", "cargo-home", "sccache", "temporary")}
    if any(target == directory or target.is_relative_to(directory) or directory.is_relative_to(target)
           for directory in directories.values()):
        raise ValueError("Cargo target/build/cache/temporary directories must be distinct siblings")
    allowed = {"SYSTEMROOT", "WINDIR", "COMSPEC", "PATH", "PATHEXT", "USERPROFILE", "HOME", "APPDATA", "LOCALAPPDATA",
               "INCLUDE", "LIB", "LIBPATH", "VCTOOLSINSTALLDIR", "WINDOWSSDKDIR", "WINDOWSSDKVERSION", "RUSTUP_HOME"}
    environment = {name: value for name, value in os.environ.items() if name.upper() in allowed}
    environment.update({"CARGO_TARGET_DIR": str(target), "CARGO_BUILD_BUILD_DIR": str(directories["build"]),
                        "CARGO_HOME": str(directories["cargo-home"]), "SCCACHE_DIR": str(directories["sccache"]),
                        "TEMP": str(directories["temporary"]), "TMP": str(directories["temporary"]),
                        "TMPDIR": str(directories["temporary"]), "RUSTC_WRAPPER": "", "RUSTC_WORKSPACE_WRAPPER": "",
                        "CARGO_INCREMENTAL": "0", "CARGO_PROFILE_DEV_DEBUG": "0", "CARGO_PROFILE_TEST_DEBUG": "0",
                        "CARGO_PROFILE_DEV_INCREMENTAL": "false", "CARGO_PROFILE_TEST_INCREMENTAL": "false",
                        "PYTHONDONTWRITEBYTECODE": "1"})
    return environment, [target, *directories.values()]


def _configuration_preflight(repo: Path, environment: dict[str, str]) -> None:
    # Reject, rather than interpret, configuration that can override build-dir,
    # rustc, runners, flags or environment. No config/credential contents are read.
    for directory in (repo, *repo.parents):
        if any((directory / ".cargo" / name).exists() for name in ("config", "config.toml")):
            raise ValueError("Ambient repository/ancestor Cargo configuration is not admitted by this bounded entry")
    home = Path(environment["CARGO_HOME"])
    if any((home / name).exists() for name in ("config", "config.toml")):
        raise ValueError("The selected Cargo cache contains an unreviewed Cargo configuration")


def _validate_artifact_evidence(command: list[str], timeout: float) -> None:
    if command[1] not in {"build", "test"} or (command[1] == "test" and "--no-run" not in command):
        raise ValueError("Native artifact evidence admits only build or test --no-run")
    if "--" in command or command.count("--message-format=json") != 1:
        raise ValueError("Native artifact evidence requires one literal --message-format=json and no harness arguments")
    if not 0 < timeout <= 86400:
        raise ValueError("Evidence timeout must be positive and at most 86400 seconds")


def _ordinary_receipt(command: list[str], repo: Path, target: Path, environment: dict[str, str],
                      output: Path, selection: dict) -> int:
    """Observe an actual root exit and regular-file logs; not process-tree acceptance."""
    output = _evidence_output(output)
    validate_output_path(output.parent).mkdir(parents=True, exist_ok=True)
    output.mkdir(exist_ok=False)
    _evidence_output(output)
    receipt = {"schema": "zircon_agent_local_cargo_command_v1", "origin": "agent-local-command",
               "executionOwner": "Agent", "command": command, "cwd": str(repo), "owner": selection,
               "targetDirectory": str(target), "buildDirectory": environment["CARGO_BUILD_BUILD_DIR"],
               "status": "command-not-started", "commandExecuted": False, "exitCode": None,
               "startedAt": datetime.now(UTC).isoformat(), "processTreeVerified": False,
               "logsFinal": False, "compilerArtifactEvidence": False, "immutableInputSeal": False,
               "sourceClosureVerified": False, "formalAcceptance": False, "milestoneAcceptance": False,
               "JenkinsAcceptance": False}
    stdout, stderr = output / "stdout.bin", output / "stderr.bin"
    result = 2
    try:
        with stdout.open("xb") as out, stderr.open("xb") as err:
            # Files, not captured pipes: Cargo retains its own native writer lock.
            # We do not claim descendant/EOF proof from the observed root exit.
            completed = subprocess.run(command, cwd=repo, env=environment, stdout=out, stderr=err, check=False)
        receipt.update(commandExecuted=True, exitCode=completed.returncode,
                       status="command-completed" if completed.returncode == 0 else "command-failed")
        result = completed.returncode
    except (OSError, KeyboardInterrupt) as error:
        receipt["error"] = {"type": type(error).__name__, "message": str(error)}
        if isinstance(error, KeyboardInterrupt):
            receipt.update(status="command-interrupted", commandExecuted=None)
            result = 130  # Wrapper interruption, not an invented Cargo root exit.
    receipt["finishedAt"] = datetime.now(UTC).isoformat()
    try:
        for name, path in (("stdout", stdout), ("stderr", stderr)):
            if path.exists():
                receipt[name] = _file_fact(path)
    except Exception as error:
        receipt.update(status="receipt-incomplete", evidenceError={"type": type(error).__name__, "message": str(error)})
        result = result or 2
    reference = None
    try:
        reference = _write_fact(output / "receipt.json", receipt)
    except Exception as error:
        receipt.update(status="receipt-write-failed", evidenceError={"type": type(error).__name__, "message": str(error)})
        result = result or 2
    # Preserve Cargo JSON output for existing product consumers. Binary logs are
    # authoritative; bounded-memory console replay is text convenience only.
    try:
        for path, stream in ((stdout, sys.stdout), (stderr, sys.stderr)):
            if path.exists():
                last = ""
                with path.open(encoding="utf-8", errors="replace") as log:
                    while chunk := log.read(65536):
                        stream.write(chunk)
                        last = chunk[-1]
                if last and last != "\n":
                    stream.write("\n")
    except (OSError, BrokenPipeError):
        # A closed console does not change the actual command/log/receipt facts.
        pass
    print(json.dumps({"status": receipt["status"], "executionOwner": "Agent", "commandExecuted": receipt["commandExecuted"],
                      "exitCode": receipt["exitCode"], "receipt": reference, "evidenceError": receipt.get("evidenceError"),
                      "formalAcceptance": False}, ensure_ascii=True))
    return result


class _LocalParser(argparse.ArgumentParser):
    def error(self, message):
        raise ValueError(message)


def main(argv: list[str] | None = None) -> int:
    execution_attempted = False
    try:
        supplied = list(sys.argv[1:] if argv is None else argv)
        reject_retired_options(supplied)
        parser = _LocalParser(description="Execute one scoped Cargo package directly under Agent ownership", allow_abbrev=False)
        parser.add_argument("--repo-root", default=str(Path(__file__).resolve().parents[2]))
        parser.add_argument("--target-dir")
        parser.add_argument("--build-root")
        parser.add_argument("--receipt-directory")
        parser.add_argument("--evidence-directory")
        parser.add_argument("--evidence-input-manifest")
        parser.add_argument("--evidence-timeout-seconds", type=float)
        parser.add_argument("--dry-run", action="store_true")
        parser.add_argument("--query-only", action="store_true", help="Read-only local command plan; never executes or queries Jenkins")
        parser.add_argument("--unlocked", action="store_true")
        parser.add_argument("--independent-preview", action="store_true", help="Deprecated read-only plan alias; omit to execute directly")
        parser.add_argument("cargo_args", nargs=argparse.REMAINDER)
        options = parser.parse_args(supplied)
        args = options.cargo_args[1:] if options.cargo_args[:1] == ["--"] else options.cargo_args
        native_options = (options.evidence_directory, options.evidence_input_manifest, options.evidence_timeout_seconds)
        if any(value is not None for value in native_options):
            if any(value is None or value == "" for value in native_options) or options.receipt_directory is not None:
                raise ValueError("Native evidence needs directory, input manifest and timeout together; ordinary receipt directory cannot be mixed")
            # Do not expose the historical artifact runner as ordinary execution.
            # A shared Cargo pool's writer lock ends before post-command auditing,
            # and a foreign --repo-root cannot identify the actual loaded drivers.
            raise ValueError("Native artifact evidence execution is not supported by this local entry; use ordinary command receipts without artifact-provenance claims")
        repo = plain_path(options.repo_root)
        selection = admit_cargo(args, repo)
        root = _build_root(options.build_root) if options.build_root is not None else None
        readonly = options.dry_run or options.query_only or options.independent_preview
        target = validate_output_path(options.target_dir) if options.target_dir is not None else validate_output_path(
            _default_target(repo, args, dry_run=readonly, build_root=root))
        actual_root = next(candidate for candidate in storage_roots() if target.is_relative_to(candidate))
        if root is not None and actual_root != root:
            raise ValueError("Explicit target and build root must use the same physical drive-root")
        environment, directories = local_environment(target)
        _configuration_preflight(repo, environment)
        command = prepare_command(selection["cargoArguments"], target, locked=not options.unlocked)
        output = _evidence_output(options.receipt_directory) if options.receipt_directory is not None else (
            actual_root / "zircon-local" / "agent-cargo-receipts" / uuid.uuid4().hex)
        output = _evidence_output(output)
        if any(output == directory or directory.is_relative_to(output) or output.is_relative_to(directory) for directory in directories):
            raise ValueError("Receipt/evidence output must be separate from compiler/cache/temporary directories")
        plan = {"status": "planned", "executionOwner": "Agent", "package": selection["package"],
                "packageManifest": selection["packageManifest"], "workspaceManifest": selection["workspaceManifest"],
                "selectionScope": selection["selectionScope"], "profile": selection["profile"], "features": selection["features"],
                "noDefaultFeatures": True, "command": command, "cargoArguments": command[1:], "cwd": str(repo),
                "targetDirectory": str(target), "buildDirectory": environment["CARGO_BUILD_BUILD_DIR"],
                "buildRoot": str(actual_root), "outputEnvironment": {key: environment[key] for key in (
                    "CARGO_HOME", "CARGO_TARGET_DIR", "CARGO_BUILD_BUILD_DIR", "SCCACHE_DIR", "TEMP", "TMP", "TMPDIR",
                    "CARGO_INCREMENTAL", "CARGO_PROFILE_DEV_DEBUG", "CARGO_PROFILE_TEST_DEBUG")},
                "evidenceMode": "observed-root-command",
                "commandExecuted": False, "formalAcceptance": False, "milestoneAcceptance": False,
                "queryOnly": options.query_only, "nativeWriterLock": "Cargo",
                "dependencyCachePolicy": "reuse-compatible-pool-cargo-home-in-place; never copy or import configuration",
                "offlineRequiresExistingDependencies": True, "compilerEnvironmentPolicy": "caller-prepared MSVC; allowlisted environment"}
        if options.independent_preview:
            plan["deprecatedOptions"] = ["IndependentPreview now means a read-only plan; omit it to execute directly"]
        if readonly:
            print(json.dumps(plan, ensure_ascii=True, sort_keys=True))
            return 0
        if output.exists():
            raise ValueError("Receipt/evidence directory must be fresh; existing history is preserved")
        for directory in [*directories, output.parent]:
            if free_bytes(directory) < MIN_FREE_BYTES:
                raise ValueError("Every output/cache/evidence volume requires at least 35 GiB free")
        executable = shutil.which("cargo", path=environment.get("PATH", ""))
        if not executable:
            raise ValueError("An installed Cargo executable is required; this wrapper never installs a toolchain")
        for directory in directories:
            validate_output_path(directory).mkdir(parents=True, exist_ok=True)
            validate_output_path(directory)
        execution_attempted = True
        return _ordinary_receipt([str(Path(executable).absolute()), *command[1:]], repo, target, environment, output, selection)
    except Exception as error:
        print(json.dumps({"status": "pending", "reasonCode": "agent_local_cargo_input", "executionOwner": "Agent",
                          "message": str(error), "commandExecuted": None if execution_attempted else False,
                          "executionAttempted": execution_attempted, "formalAcceptance": False, "milestoneAcceptance": False}))
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
