"""Fixed Jenkins templates with independent Windows Job supervision."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import signal
import subprocess
import sys
import threading
import time
from dataclasses import asdict
from contextlib import ExitStack
from pathlib import Path
from typing import Callable

from .native.process_supervisor import ProcessSupervisor

from .client import JenkinsClient, JenkinsTransportError
from .contracts import PilotError, RequestIdentity, canonical_json, digest, identifier
from .snapshot import load_snapshot, materialize, verify_materialized
from .storage import ManagedStorage, require_managed_root
from .process_evidence import JobEvidence, error_record, observed_atomic_factory


def run_directory(root: Path, identity: RequestIdentity) -> Path:
    return ManagedStorage(root).path(
        f"runs/{identity.session_id}/{identity.request_id}/{identity.attempt_id}-g{identity.generation}"
    )


def template_command(identity: RequestIdentity, source: Path, run_dir: Path, *, repo_root: Path, probe_seconds: float = 60) -> list[str]:
    if identity.template == "python-static":
        return [sys.executable, "-B", str(Path(__file__).with_name("static_validation.py")), str(source)]
    if identity.template == "fault-probe":
        return [sys.executable, "-B", "-m", "tools.jenkins.pilot.fault_probe", str(run_dir / "probe.json"), str(probe_seconds)]
    raise PilotError("Cargo requires retained prepare_execution ownership")


def _artifact(backend, relative: str) -> dict[str, object]:
    data = backend.read_bytes(relative)
    return {"path": relative, "sha256": hashlib.sha256(data).hexdigest(), "size": len(data)}


def run(
    *, root: Path, repo_root: Path, bundle: Path, identity: RequestIdentity,
    job: str, build_number: int, cancellation: threading.Event | None = None,
    health: Callable[[], bool] | None = None, probe_seconds: float = 60,
    maximum_seconds: float = 900,
    registry_seed: Path | None = None,
) -> dict[str, object]:
    started = time.monotonic()
    identifier(job, "job")
    if type(build_number) is not int or build_number < 1:
        raise PilotError("runner requires an exact Jenkins build number")
    if registry_seed is not None and identity.template != "managed-cargo-check-v2":
        raise PilotError("registry seeding requires the retained Cargo execution contract")
    storage = ManagedStorage(root)
    directory = run_directory(root, identity)
    relative = directory.relative_to(root).as_posix()
    with ExitStack() as resources:
        parent = resources.enter_context(storage.backend())
        if parent.exists(relative):
            raise PilotError("this immutable attempt already exists; reconcile its original receipt")
        created = parent.create_directory(relative)
        backend = resources.enter_context(ManagedStorage(directory).backend())
        pinned = backend.root_identity
        if (created.native_identity is None or pinned.native_identity is None
                or not created.native_identity.same_object(pinned.native_identity)):
            raise PilotError("execution directory changed before its identity was retained")
        return _run_in_directory(root=root, repo_root=repo_root, bundle=bundle, identity=identity,
            job=job, build_number=build_number, cancellation=cancellation, health=health,
            probe_seconds=probe_seconds, maximum_seconds=maximum_seconds,
            registry_seed=registry_seed,
            run_backend=backend, resources=resources, started=started)


def _run_in_directory(
    *, root: Path, repo_root: Path, bundle: Path, identity: RequestIdentity,
    job: str, build_number: int, cancellation: threading.Event | None = None,
    health: Callable[[], bool] | None = None, probe_seconds: float = 60,
    maximum_seconds: float = 900, registry_seed: Path | None = None,
    run_backend, resources: ExitStack, started: float,
) -> dict[str, object]:
    identifier(job, "job")
    if type(build_number) is not int or build_number < 1:
        raise PilotError("runner requires an exact Jenkins build number")
    driver_hash = None
    if (root / "driver-proof.json").exists():
        from .deployment import verify_driver
        driver_hash = verify_driver(root)
    storage = ManagedStorage(root)
    run_dir = run_directory(root, identity)
    source = run_dir / "sealed" / "source"
    sealed = materialize(bundle, source, repo_root=repo_root, expected_input_hash=identity.input_hash)
    before = verify_materialized(sealed, source)
    driver_root = Path(__file__).resolve().parents[2]
    cargo_execution = None
    if identity.template == "managed-cargo-check-v2":
        from .managed_cargo import prepare_execution
        cargo_execution = prepare_execution(repo_root=driver_root, source_root=source, run_root=run_dir,
                                            identity=identity, managed_root=root)
        resources.callback(cargo_execution.close)
        if registry_seed is not None:
            cargo_execution.seed_registry(registry_seed)
        if run_backend.exists("managed-request.json"):
            request = json.loads(run_backend.read_bytes("managed-request.json"))
            seed_hash = request["generation"].get("registrySeedHash")
            if seed_hash:
                seed = cargo_execution.generation.backend.read_bytes("registry-seed.json", expected_sha256=seed_hash)
                if digest(json.loads(seed)) != seed_hash:
                    raise PilotError("registry provenance no longer matches the retained generation")
                run_backend.write_bytes("registry-seed.json", seed, expected_sha256=seed_hash)
        command = cargo_execution.command
    else:
        command = template_command(identity, source, run_dir, repo_root=repo_root, probe_seconds=probe_seconds)
    environment = dict(os.environ)
    scratch = run_dir / "scratch"
    with storage.backend() as backend:
        backend.ensure_directory(scratch.relative_to(storage.root).as_posix())
    environment.update({"TEMP": str(scratch), "TMP": str(scratch), "PYTHONDONTWRITEBYTECODE": "1", "PYTHONUTF8": "1"})
    # Only the trusted driver may supply executable Python modules.
    environment["PYTHONPATH"] = str(driver_root)
    environment["CODEX_THREAD_ID"] = f"jenkins-{identity.session_id}-{identity.request_id}"
    environment["PATH"] = str(Path(sys.executable).parent) + os.pathsep + environment.get("PATH", "")
    if cargo_execution is not None:
        environment = cargo_execution.environment
    execution_cwd = source if cargo_execution is not None else driver_root
    owner = (identity.request_id, identity.attempt_id, identity.generation, identity.generation)
    prepare_seconds = time.monotonic() - started
    execution_started = time.monotonic()
    output_path = run_dir / "output.log"
    output_lock = threading.Lock()
    with run_backend.open_lock_file("output.log") as output:
        if output.seek(0, 2) != 0:
            raise PilotError("execution log already contains data; preserve the immutable attempt")
        def emit(_owner, channel, data):
            with output_lock:
                output.write(data)
                output.flush()

        pipe_observations = {}
        supervisor = ProcessSupervisor(output_callback=emit, atomic_popen=observed_atomic_factory(pipe_observations))
        reason, terminal = None, False
        process, evidence = None, None
        exit_code, process_metadata = None, {}
        execution_errors = []
        readers_finished = False
        try:
            process = supervisor.start(command, cwd=execution_cwd, env=environment, owner=owner)
            evidence = JobEvidence(process)
            metadata = {"identity": identity.to_dict(), "job": job, "buildNumber": build_number,
                        "pid": process.pid, "creationTime": process.creation_time, "commandHash": digest(command)}
            storage.atomic_write(f"{run_dir.relative_to(storage.root).as_posix()}/process.json", canonical_json(metadata))
            while process.process.poll() is None:
                if cancellation is not None and cancellation.is_set():
                    reason = "cancelled"
                elif time.monotonic() - execution_started > maximum_seconds:
                    reason = "timeout"
                elif health is not None and not health():
                    reason = "controller_or_agent_disconnected"
                if reason:
                    # terminate() returns only after existing native job accounting
                    # proves zero active processes. Killing a root PID alone is insufficient.
                    evidence.capture("beforeWait", members=True)
                    process.terminate(reason=reason)
                    terminal = True
                    break
                time.sleep(0.2)
            if "beforeWait" not in evidence.records:
                evidence.capture("beforeWait", members=True)
            try:
                exit_code = process.wait(timeout=15)
            except Exception as error:
                execution_errors.append(error_record("wait", error))
                reason = reason or "owned_job_wait_failed"
            finally:
                # Root exit is distinct from descendant convergence and survives
                # a wait() which terminates its Job then raises.
                exit_code = process.process.poll()
                evidence.capture("afterWait")
        except Exception as error:
            execution_errors.append(error_record("execution", error))
            reason = reason or "owned_execution_failed"
        finally:
            # On cancellation terminate() may already have consumed the native
            # handle. Capture before terminate above instead of guessing proof.
            if evidence is not None and execution_errors:
                before_close = evidence.capture("beforeClose")
                if before_close["activeProcesses"] != 0:
                    # A query handle also retains the Job. After wait() closes
                    # its original handle on a query error, retaining our copy
                    # would prevent KILL_ON_CLOSE and leave pipe.close blocked
                    # behind a reader held by a surviving owned descendant.
                    # Release only our copy before any potentially blocking
                    # stream close; further accounting stays unverified.
                    evidence.close()
            try:
                supervisor.close()
            except Exception as error:
                execution_errors.append(error_record("close", error))
                reason = reason or "owned_cleanup_failed"
            if process is not None:
                exit_code = process.process.poll()
                process_metadata = asdict(process.snapshot())
                readers_finished = all(not reader.is_alive() for reader in process._readers)
            if evidence is not None:
                final = evidence.capture("afterClose")
                terminal = final["activeProcesses"] == 0
                evidence.close()
                if evidence.errors:
                    execution_errors.extend(evidence.errors)
                    terminal = False
                    reason = reason or "owned_job_evidence_unverified"
    run_seconds = time.monotonic() - execution_started
    # A failed close can leave a reader finishing asynchronously. Archive an
    # immutable observation, not dictionaries that a late reader can mutate.
    pipe_observations = json.loads(canonical_json(pipe_observations))
    pipe_eof = (len(pipe_observations) == 2 and all(
        value.get("eof") is True and "error" not in value for value in pipe_observations.values()))
    if not readers_finished or not pipe_eof:
        execution_errors.append({"stage": "output_capture", "type": "UnverifiedPipeEOF",
                                 "message": "Both output readers must finish after observing actual EOF"})
        reason = reason or "output_capture_unverified"
    after = None
    source_error = None
    try:
        after = verify_materialized(sealed, source)
        if driver_hash is not None and verify_driver(root) != driver_hash:
            raise PilotError("trusted driver changed during execution")
    except PilotError as error:
        source_error = str(error)
    outcome = "failed" if execution_errors else "cancelled" if reason else "passed" if exit_code == 0 and terminal and before == after and readers_finished and pipe_eof else "failed"
    receipt = {
        "schemaVersion": 2, "acceptance": "jenkins-command-evidence-v2", "identity": identity.to_dict(),
        "job": job, "buildNumber": build_number, "bundleHash": sealed.bundle_hash,
        "command": command, "commandHash": digest(command), "sourceBefore": before, "sourceAfter": after,
        "exitCode": exit_code, "outcome": outcome, "reason": reason or source_error,
        "toolchain": {"python": platform.python_version(), "platform": platform.platform()},
        "processTree": {"terminal": terminal, "scope": "windows_job" if terminal else "unverified",
                        "pid": process.pid if process is not None else None,
                        "creationTime": process.creation_time if process is not None else None,
                        "readersFinished": readers_finished, "pipeEOF": pipe_eof,
                        "pipes": pipe_observations},
        "timings": {"prepareSeconds": prepare_seconds, "runSeconds": run_seconds},
        "artifacts": [_artifact(run_backend, "output.log")], "processDetails": process_metadata,
        "terminalErrors": execution_errors,
        "nativeJobEvidence": evidence.to_dict() if evidence is not None else None,
    }
    if driver_hash:
        receipt["toolchain"]["driverInputHash"] = driver_hash
    if cargo_execution is not None:
        from .managed_cargo import collect_evidence
        try:
            observation = json.loads(canonical_json({key: receipt[key] for key in (
                "command", "commandHash", "exitCode", "processTree", "nativeJobEvidence", "terminalErrors")}))
            cargo_execution.finalize(observation)
            receipt["managedCargo"] = collect_evidence(repo_root=driver_root, run_root=run_dir,
                                                       exit_code=exit_code, observation=observation)
        except Exception as error:
            receipt["managedCargo"] = {"accepted": False, "formalAcceptance": False,
                "error": error_record("managed_cargo_collector", error)}
            receipt["terminalErrors"].append(receipt["managedCargo"]["error"])
            receipt["reason"] = receipt["reason"] or "managed_cargo_collector_failed"
        if not receipt["managedCargo"].get("accepted"):
            if receipt["outcome"] != "cancelled":
                receipt["outcome"] = "failed"
        for artifact_name in ("managed-request.json", "managed-result.json", "managed-evidence.json", "registry-seed.json"):
            artifact_path = run_dir / artifact_name
            if artifact_path.is_file():
                receipt["artifacts"].append(_artifact(run_backend, artifact_name))
    storage.atomic_write(f"{run_dir.relative_to(storage.root).as_posix()}/receipt.json", canonical_json(receipt))
    return receipt


def controller_health(root: Path, job: str, build_number: int) -> Callable[[], bool]:
    from .jenkins_config import read_credentials, read_manifest
    manifest, credentials = read_manifest(root), read_credentials(root)
    client = JenkinsClient(str(manifest["controllerUrl"]), **credentials, timeout=2)
    last_ok = time.monotonic()
    next_poll = 0.0
    status = True

    def healthy() -> bool:
        nonlocal last_ok, next_poll, status
        if time.monotonic() < next_poll:
            return status
        next_poll = time.monotonic() + 1
        try:
            node = client.api(f"/computer/{manifest.get('agentName', 'zircon-windows-agent')}/api/json?tree=offline")
            build = client.api(f"/job/{job}/{build_number}/api/json?tree=building,result")
            status = node.get("offline") is False and build.get("building") is True
            if status:
                last_ok = time.monotonic()
        except JenkinsTransportError:
            status = time.monotonic() - last_ok < 5
        return status

    return healthy


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--repo-root", type=Path, required=True)
    parser.add_argument("--bundle", type=Path, required=True)
    parser.add_argument("--request-file", type=Path)
    parser.add_argument("--build-number", type=int, required=True)
    parser.add_argument("--job", default="zircon-pilot")
    parser.add_argument("--probe-seconds", type=float, default=60)
    parser.add_argument("--monitor-jenkins", action="store_true")
    parser.add_argument("--execution-only", action="store_true")
    args = parser.parse_args()
    from .governance import require_live_storage_owner
    require_live_storage_owner(args.root, args.repo_root)
    if args.request_file:
        identity = RequestIdentity.from_dict(json.loads(args.request_file.read_bytes()))
    else:
        identity = RequestIdentity(os.environ["SESSION_ID"], os.environ["REQUEST_ID"], os.environ["ATTEMPT_ID"],
                                   int(os.environ["GENERATION"]), os.environ["INPUT_HASH"], os.environ["TEMPLATE"])
    expected_archive = os.environ.get("BUNDLE_HASH")
    if expected_archive and load_snapshot(args.bundle, expected_input_hash=identity.input_hash).bundle_hash != expected_archive:
        raise PilotError("uploaded source archive differs from its request binding")
    cancellation = threading.Event()
    for name in ("SIGTERM", "SIGINT", "SIGBREAK"):
        if hasattr(signal, name):
            signal.signal(getattr(signal, name), lambda *_args: cancellation.set())
    health = controller_health(args.root, args.job, args.build_number) if args.monitor_jenkins else None
    if args.execution_only:
        from .execution_dispatch import require_execution_worker
        require_execution_worker(root=args.root, repo_root=args.repo_root, identity=identity,
                                 job=args.job, build_number=args.build_number)
        receipt = run(root=args.root, repo_root=args.repo_root, bundle=args.bundle, identity=identity,
                      job=args.job, build_number=args.build_number, cancellation=cancellation, health=health,
                      probe_seconds=args.probe_seconds)
    else:
        from .execution_dispatch import enqueue_and_wait
        receipt = enqueue_and_wait(root=args.root, repo_root=args.repo_root, bundle=args.bundle, identity=identity,
                                   job=args.job, build_number=args.build_number)
    print(json.dumps({"outcome": receipt["outcome"], "runDirectory": str(run_directory(args.root, identity))}))
    workspace = os.environ.get("WORKSPACE")
    if workspace and not args.execution_only:
        destination = require_managed_root(Path(workspace), allow_root=False)
        if not destination.is_relative_to(args.root / "work/agent"):
            raise PilotError("Jenkins archive workspace belongs to a different private root")
        archive = ManagedStorage(destination)
        archive.atomic_write("receipt.json", canonical_json(receipt))
        archive.atomic_write("output.log", (run_directory(args.root, identity) / "output.log").read_bytes())
        for artifact in receipt["artifacts"]:
            if artifact["path"] != "output.log":
                archive.atomic_write(artifact["path"], (run_directory(args.root, identity) / artifact["path"]).read_bytes())
    return 0 if receipt["outcome"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
