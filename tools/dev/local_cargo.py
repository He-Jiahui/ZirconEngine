"""Run local Cargo with physical output checks and no coordinator dependency."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import uuid

from tools.jenkins.contracts import JenkinsError


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


def prepare_command(arguments: list[str], target: Path) -> list[str]:
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
    if "--locked" not in options:
        generated.append("--locked")
    return ["cargo", *options, *generated, *args[separator:]]


def _default_target(repo: Path, args: list[str], *, dry_run: bool) -> Path:
    # This namespace never reuses the retired coordinator's pools or job scratch.
    root = storage_roots()[0] if dry_run else max(
        (root for root in storage_roots() if root.parent.exists()), key=free_bytes
    )
    toolchain = repo / "rust-toolchain.toml"
    identity = json.dumps({"repo": str(repo).casefold(), "platform": sys.platform,
                           "toolchain": toolchain.read_text(encoding="utf-8") if toolchain.exists() else "default",
                           "cargoArgs": args}, sort_keys=True).encode("utf-8")
    key = hashlib.sha256(identity).hexdigest()[:20]
    return root / "zircon-local" / sys.platform / key / "target"


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
        terminate_process_job, wait_for_process_job_terminal,
    )
    from tools.jenkins.pilot.process_evidence import JobEvidence
    if os.name != "nt" or not 0 < timeout <= 86400:
        raise ValueError("Owned local evidence requires Windows and a bounded positive timeout")
    deadline = time.monotonic() + timeout
    process, job, query, waiter, birth = None, None, None, None, None
    native_done = threading.Event()
    native_result = {"terminal": False}
    pipes = {name: {"eof": False, "closed": False} for name in ("stdout", "stderr")}
    logs, errors, root_exit = {}, [], None
    def fail(stage, error):
        errors.append({"stage": stage, "type": type(error).__name__, "message": str(error)})
    def wait_native():
        try:
            wait_for_process_job_terminal(job, timeout_seconds=max(0.001, deadline - time.monotonic()))
            native_result["terminal"] = True
        except BaseException as error:
            fail("owned_job_wait", error)
        finally:
            native_done.set()
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
        api = ctypes.WinDLL("kernel32", use_last_error=True)
        api.PeekNamedPipe.argtypes = [wintypes.HANDLE, ctypes.c_void_p, wintypes.DWORD,
                                     ctypes.c_void_p, ctypes.POINTER(wintypes.DWORD), ctypes.c_void_p]
        api.PeekNamedPipe.restype = wintypes.BOOL
        raw = {name: getattr(getattr(process, name).buffer, "raw", getattr(process, name).buffer) for name in pipes}
        handles = {name: msvcrt.get_osfhandle(stream.fileno()) for name, stream in raw.items()}
        waiter = threading.Thread(target=wait_native, name="local-cargo-owned-job-wait", daemon=True)
        waiter.start()
        resume_popen_process(process)
        # This owner alone reads raw pipe bytes. No reader takes a Buffered lock.
        while True:
            for name in pipes:
                if pipes[name]["eof"]:
                    continue
                available = wintypes.DWORD()
                if not api.PeekNamedPipe(handles[name], None, 0, None, ctypes.byref(available), None):
                    code = ctypes.get_last_error()
                    if code not in (109, 232):  # broken/no-data pipe: verify an actual empty read
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
            root_exit = process.poll()
            if errors:
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
            try:
                # Terminate only this retained Job; keep its handle until waiter settlement.
                terminate_process_job(job)
                if waiter is not None:
                    waiter.join(timeout=10)
                else:
                    wait_for_process_job_terminal(job, timeout_seconds=10)
                if process is not None:
                    root_exit = process.poll()
            except BaseException as cleanup_error:
                fail("owned_job_cleanup", cleanup_error)
    finally:
        if process is not None:
            # All pipe reads run in this owner. Close cannot contend with a buffered reader.
            for name in pipes:
                try:
                    getattr(process, name).close()
                    pipes[name]["closed"] = True
                except BaseException as error:
                    fail("pipe_close", error)
        for log in logs.values():
            try:
                log.close()
            except BaseException as error:
                fail("log_close", error)
        waiter_joined = waiter is None or not waiter.is_alive()
        if job and waiter_joined:
            try:
                close_process_job(job)
                if query is not None:
                    query.capture("afterClose")
            except BaseException as error:
                fail("owned_job_close", error)
        # A still-running native waiter retains the Job until this failed CLI exits.
        # Windows closes that exact handle at process exit. This is NOT terminal proof.
        if query is not None:
            query.close()
        if process is not None:
            try:
                process.close(close_streams=False)
            except BaseException as error:
                fail("root_handle_close", error)
    job_evidence = query.to_dict() if query is not None else {"records": {}, "errors": []}
    after_close = job_evidence["records"].get("afterClose", {})
    complete = (root_exit is not None and waiter_joined and native_result["terminal"]
                and all(p["eof"] for p in pipes.values()) and after_close.get("activeProcesses") == 0
                and not errors and not job_evidence["errors"])
    return {"schemaVersion": 1, "command": command, "cwd": str(repo),
            "pid": process.pid if process is not None else None,
            "creationTime": birth if process is not None else None, "rootExitCode": root_exit,
            "processTree": {"scope": "retained-windows-job", "terminal": complete,
                            "nativeWaiterJoined": waiter_joined, "readersFinished": True,
                            "pipes": pipes, "pipeEOF": all(p["eof"] for p in pipes.values()),
                            "jobHandleRetainedUntilCliExit": bool(job and not waiter_joined)},
            "nativeJobEvidence": job_evidence, "terminalErrors": errors, "formalAcceptance": False}


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


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", default=str(Path(__file__).resolve().parents[2]))
    parser.add_argument("--target-dir")
    parser.add_argument("--evidence-directory")
    parser.add_argument("--evidence-input-manifest")
    parser.add_argument("--evidence-timeout-seconds", type=float)
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--jenkins-identity-file")
    parser.add_argument("--independent-preview", action="store_true")
    parser.add_argument("cargo_args", nargs=argparse.REMAINDER)
    options = parser.parse_args(argv)
    args = options.cargo_args[1:] if options.cargo_args[:1] == ["--"] else options.cargo_args
    try:
        repo = Path(options.repo_root).resolve()
        evidence_requested = options.evidence_directory is not None
        if evidence_requested != (options.evidence_input_manifest is not None):
            raise ValueError("Evidence directory and selected-input manifest must be supplied together")
        evidence_timeout = 21600 if options.evidence_timeout_seconds is None else options.evidence_timeout_seconds
        if options.evidence_timeout_seconds is not None and not evidence_requested:
            raise ValueError("Evidence timeout requires evidence mode")
        if evidence_requested:
            if not options.independent_preview or not options.target_dir or not 0 < evidence_timeout <= 86400:
                raise ValueError("Evidence mode requires independent preview, explicit fresh target and bounded timeout")
            if _evidence_output(options.evidence_directory).exists():
                raise ValueError("Evidence directory must be exclusively new; existing paths are preserved")
        if options.jenkins_identity_file:
            os.environ["ZIRCON_JENKINS_IDENTITY_FILE"] = str(Path(options.jenkins_identity_file).resolve())
        # M8 owns all known build/test entry points once activated.  The
        # identity file is supplied by the caller/session; without it we fail
        # before Cargo can start.  Prior to M8 this returns None and preserves
        # the independent local evidence path below.
        from tools.jenkins.frontend import entry_from_environment, sole_entry_enforced
        gate_repo = Path(__file__).resolve().parents[2]
        if options.independent_preview and sole_entry_enforced(gate_repo):
            raise ValueError("Jenkins sole-entry gate active; independent preview refuses service dispatch")
        forwarded = None if options.independent_preview else entry_from_environment(
            gate_repo, job="zircon-flow", parameters={"cargoVerb": args[0] if args else ""})
        if forwarded is not None:
            print(json.dumps(forwarded, ensure_ascii=False, sort_keys=True))
            return 0
        target = validate_output_path(options.target_dir or _default_target(repo, args, dry_run=options.dry_run))
        command = prepare_command(args, target)
        if evidence_requested and target == target.parent / "build":
            raise ValueError("Evidence target and build directories must be distinct")
        if evidence_requested and (target.exists() or validate_output_path(target.parent / "build").exists()):
            raise ValueError("Evidence requires a new target/build namespace; existing outputs are preserved")
        if options.dry_run:
            plan = {"status": "dry_run", "command": command, "formalAcceptance": False}
            if evidence_requested:
                plan["evidencePlan"] = {"directory": options.evidence_directory,
                                        "inputManifest": options.evidence_input_manifest,
                                        "timeoutSeconds": evidence_timeout}
            print(json.dumps(plan, ensure_ascii=False))
            return 0
        if free_bytes(target) < MIN_FREE_BYTES:
            raise ValueError("The target drive must have at least 35 GiB free before Cargo starts")
        root = next(root for root in storage_roots() if target.is_relative_to(root))
        cache = validate_output_path(root / "zircon-local/cache")
        scratch = validate_output_path(root / "zircon-local/scratch" / uuid.uuid4().hex)
        directories = [target, cache / "cargo-home", cache / "sccache", scratch / "temporary", target.parent / "build"]
        for directory in directories:
            exclusive = evidence_requested and directory in {target, target.parent / "build"}
            validate_output_path(directory).mkdir(parents=True, exist_ok=not exclusive)
            validate_output_path(directory)
        environment = dict(os.environ)
        environment.update({
            "CARGO_TARGET_DIR": str(target), "CARGO_HOME": str(cache / "cargo-home"),
            "CARGO_BUILD_BUILD_DIR": str(target.parent / "build"),
            "SCCACHE_DIR": str(cache / "sccache"),
            "RUSTC_WRAPPER": "", "RUSTC_WORKSPACE_WRAPPER": "",
            "TEMP": str(scratch / "temporary"), "TMP": str(scratch / "temporary"),
            "TMPDIR": str(scratch / "temporary"),
        })
        # Cargo retains its native target locks; no shared index or old pool is changed.
        if evidence_requested:
            return _run_with_evidence(command, repo, target, environment, options.evidence_directory,
                                      options.evidence_input_manifest, evidence_timeout)
        return subprocess.run(command, cwd=repo, env=environment, check=False).returncode
    except (OSError, ValueError, JenkinsError) as error:
        print(f"Local Cargo preflight failed: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
