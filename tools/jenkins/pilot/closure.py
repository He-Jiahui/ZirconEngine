"""Seal the fixed Cargo closure with a pilot-owned native metadata transport.

The planner and its public transport are owned by this independent package.
"""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import subprocess
import threading
import time
from typing import Callable
from types import SimpleNamespace

from . import inputs
from .source_manifest import file_hash, manifest_digest
from .native.process_identity import popen_process_creation_time
from .snapshot import _git
from .native.windows_job_process import (
    _close_handle as close_owned_handle,
    create_atomic_kill_on_close_process, resume_popen_process,
    terminate_and_close_process_job,
)
from .contracts import PilotError, canonical_json, digest
from .process_evidence import JobEvidence, error_record
from .governance import require_live_storage_owner
from .lifetime import _kernel, _open_owner
from .managed_cargo import executor_manifest, toolchain_identity
from .snapshot import capture
from .storage import ManagedStorage
from .trusted_tools import git_identity


DECLARED_COMMANDS = (("cargo", "check", "--locked", "-p", "zircon_reflect_derive"),)
MAXIMUM_METADATA_BYTES = 256 * 1024 * 1024
PIPE_EOF_TIMEOUT_SECONDS = 15.0


class NativeMetadataTransport:
    """Fully capture two pipes and terminate only the retained native Job."""

    def __init__(self, *, timeout_seconds: float | None = None,
                 maximum_output_bytes: int = MAXIMUM_METADATA_BYTES,
                 lease_owner: tuple[int, str] | None = None,
                 record: Callable[[dict], None] | None = None):
        self.timeout_seconds = (inputs.METADATA_TIMEOUT_SECONDS
                                if timeout_seconds is None else timeout_seconds)
        if self.timeout_seconds <= 0 or maximum_output_bytes <= 0:
            raise ValueError("metadata timeout and output limit must be positive")
        self.maximum_output_bytes = maximum_output_bytes
        self.lease_owner = lease_owner
        self.record = record
        self.receipts: list[dict] = []

    def __call__(self, arguments, source, environment):
        argv = tuple(arguments)
        started = time.monotonic()
        receipt = {"schemaVersion": 1, "argv": list(argv), "cwd": str(source),
                   "startedUTC": datetime.now(timezone.utc).isoformat(),
                   "timeoutSeconds": self.timeout_seconds, "outcome": "failed",
                   "pid": None, "creationTime": None, "exitCode": None,
                   "processTree": {"scope": "windows_job", "terminal": False}}
        process, job = None, None
        job_evidence = None
        native_observation = None
        owner_kernel, owner_handle = None, None
        if self.lease_owner is not None:
            receipt["leaseOwner"] = {"pid": self.lease_owner[0], "creationTime": self.lease_owner[1]}
        streams: dict[str, list[bytes]] = {"stdout": [], "stderr": []}
        readers: list[threading.Thread] = []
        eof: set[str] = set()
        errors: list[BaseException] = []
        output_bytes = 0
        native_exit_code = None
        lock = threading.Lock()
        outcome = "failed"

        def check_owner():
            # The retained handle identifies the verified process even if its
            # numeric PID is later reused. This poll needs no process snapshot.
            if owner_handle is not None and owner_kernel.WaitForSingleObject(owner_handle, 0) != 258:
                receipt["reason"] = "lease_owner_exited"
                raise OSError("the exact fixture lease owner exited during metadata")

        def drain(name):
            nonlocal output_bytes
            try:
                stream = getattr(process, name).buffer
                while data := stream.read1(65536):
                    with lock:
                        output_bytes += len(data)
                        if output_bytes > self.maximum_output_bytes:
                            errors.append(OSError("metadata output exceeded its complete-capture limit"))
                            return
                        streams[name].append(data)
                with lock:
                    eof.add(name)
            except BaseException as error:
                with lock:
                    errors.append(error)

        def collected():
            return tuple(b"".join(streams[name]).decode("utf-8", errors="replace")
                         for name in ("stdout", "stderr"))

        try:
            if self.lease_owner is not None:
                try:
                    owner_kernel = _kernel()
                    owner_handle = _open_owner(owner_kernel, *self.lease_owner)
                except OSError:
                    receipt["reason"] = "lease_owner_unavailable"
                    raise
            check_owner()
            process, job = create_atomic_kill_on_close_process(argv, cwd=Path(source), env=dict(environment))
            receipt.update(pid=process.pid, creationTime=popen_process_creation_time(process))
            job_evidence = JobEvidence(SimpleNamespace(job_handle=job))
            for name in streams:
                reader = threading.Thread(target=drain, args=(name,), daemon=True,
                                          name=f"pilot-metadata-{name}-{process.pid}")
                reader.start()
                readers.append(reader)
            check_owner()
            resume_popen_process(process)
            deadline = time.monotonic() + self.timeout_seconds
            while process.poll() is None:
                check_owner()
                with lock:
                    if errors:
                        raise OSError("metadata pipe capture failed") from errors[0]
                if time.monotonic() >= deadline:
                    outcome = "timed_out"
                    break
                time.sleep(0.02)
            receipt["exitCode"] = process.poll()
            check_owner()
            if outcome != "timed_out":
                outcome = "completed" if receipt["exitCode"] == 0 else "failed"
        finally:
            # Native termination also converges descendants after a successful
            # root exit. No PID ancestry discovery or legacy fallback is used.
            cleanup_error = None
            if job_evidence is not None:
                job_evidence.capture("beforeWait")
            if job is not None:
                owned_job, job = job, None
                try:
                    terminate_and_close_process_job(owned_job)
                    receipt["jobAccountingZero"] = True
                except BaseException as error:
                    cleanup_error = error
            if job_evidence is not None:
                job_evidence.capture("afterClose")
                # A query duplicate also retains the Job. Always close it
                # before pipe waits so failed termination still gets native
                # KILL_ON_CLOSE rather than a surviving query-only holder.
                job_evidence.close()
                if job_evidence.errors:
                    cleanup_error = cleanup_error or OSError("native metadata Job observation failed")
            eof_deadline = time.monotonic() + PIPE_EOF_TIMEOUT_SECONDS
            for reader in readers:
                reader.join(timeout=max(0, eof_deadline - time.monotonic()))
            if any(reader.is_alive() for reader in readers):
                cleanup_error = cleanup_error or OSError("metadata pipes did not reach EOF after native Job termination")
            if errors:
                cleanup_error = cleanup_error or OSError("metadata pipe capture failed")
            try:
                check_owner()
            except OSError as error:
                cleanup_error = cleanup_error or error
            finally:
                if owner_handle is not None:
                    close_owned_handle(owner_handle)
                    owner_handle = None
            if cleanup_error is not None:
                outcome = "failed"
            if process is not None:
                try:
                    native_exit_code = process.poll()
                    receipt["exitCode"] = native_exit_code
                except BaseException as error:
                    cleanup_error = cleanup_error or error
                    outcome = "failed"
                # Closing a pipe while its reader is blocked can itself block;
                # preserve the failed receipt instead of claiming EOF.
                if not any(reader.is_alive() for reader in readers):
                    process.close()
                else:
                    # Do not let TextIOWrapper.close block past the EOF bound.
                    # Close the exact retained root handle; the daemon reader
                    # retains its pipe until EOF or isolated CLI teardown.
                    if process._handle:
                        close_owned_handle(process._handle)
                        process._handle = 0
                    process._closed = True
            receipt.update(outcome=outcome, outputBytes=output_bytes,
                           pipeEOF=eof == {"stdout", "stderr"},
                           elapsedSeconds=time.monotonic() - started,
                           stdoutSha256=hashlib.sha256(b"".join(streams["stdout"])).hexdigest(),
                           stderrSha256=hashlib.sha256(b"".join(streams["stderr"])).hexdigest())
            receipt["processTree"]["terminal"] = (
                outcome in {"completed", "timed_out"}
                and receipt.get("jobAccountingZero") is True and receipt["pipeEOF"]
                and cleanup_error is None
                and (outcome == "timed_out" or receipt["exitCode"] == 0))
            actual_terminal = (receipt.get("jobAccountingZero") is True and receipt["pipeEOF"]
                               and cleanup_error is None and job_evidence is not None
                               and not job_evidence.errors)
            native_observation = {
                "command": list(argv), "commandHash": digest(list(argv)), "exitCode": native_exit_code,
                "processTree": {"scope": "windows_job", "terminal": actual_terminal,
                    "pid": receipt["pid"], "creationTime": receipt["creationTime"],
                    "readersFinished": not any(reader.is_alive() for reader in readers),
                    "pipeEOF": receipt["pipeEOF"],
                    "pipes": {name: {"eof": name in eof} for name in streams}},
                "nativeJobEvidence": job_evidence.to_dict() if job_evidence is not None else None,
                "terminalErrors": [error_record("metadata_cleanup", cleanup_error)] if cleanup_error else []}
            receipt["nativeObservation"] = native_observation
            self.receipts.append(json.loads(canonical_json(receipt)))
            if self.record is not None:
                # An observer receives a detached value and cannot rewrite the
                # retained observation or the actual native return code.
                self.record(json.loads(canonical_json(receipt)))
            if cleanup_error is not None:
                raise OSError("metadata native lifetime or complete capture could not be verified") from cleanup_error
        stdout, stderr = collected()
        if outcome == "timed_out":
            failure = subprocess.TimeoutExpired(argv, self.timeout_seconds, output=stdout, stderr=stderr)
            failure.native_observation = native_observation
            raise failure
        result = subprocess.CompletedProcess(argv, native_exit_code, stdout, stderr)
        result.native_observation = native_observation
        return result


def verify_captured_inputs(sealed, fingerprints: dict[str, str | None],
                           external: dict[Path, dict[str, str | None]] | None = None) -> None:
    observed = {entry["path"]: entry["sha256"] for entry in sealed.manifest["entries"]}
    if observed != fingerprints:
        raise PilotError("captured bytes differ from the planned Cargo closure")
    observed_external = {
        sibling["mount"]: {entry["path"]: entry["sha256"] for entry in sibling["entries"]}
        for sibling in sealed.manifest.get("external", [])}
    if observed_external != {root.name: entries for root, entries in (external or {}).items()}:
        raise PilotError("captured external bytes differ from the planned Cargo closure")


def _executor_identity(repo_root: Path) -> dict[str, str]:
    result = executor_manifest(repo_root)
    for name in ("closure", "snapshot", "external_snapshot", "inputs", "source_manifest", "storage", "governance", "contracts", "lifetime"):
        path = repo_root / f"tools/jenkins_pilot/{name}.py"
        result[path.relative_to(repo_root).as_posix()] = file_hash(path)
    return result


def seal_closure(*, root: Path, repo_root: Path, output: str = "cargo-closure.json") -> dict:
    lease = require_live_storage_owner(root, repo_root)
    storage = ManagedStorage(root)
    storage.path(output)  # Admit the output before starting metadata.
    scratch = storage.temporary_directory(prefix="closure-")
    # Each metadata attempt owns a fresh output generation. A failed lifetime
    # observation can never make its locks/cache eligible for a later attempt.
    metadata_target = storage.path(f"{scratch.relative_to(root).as_posix()}/metadata-generation")
    original_environment = {name: os.environ.get(name) for name in ("TMP", "TEMP")}
    transport = NativeMetadataTransport(
        lease_owner=(lease["owner_pid"], lease["owner_process_creation_time"]),
        record=lambda receipt: storage.atomic_write(
        f"{scratch.relative_to(root).as_posix()}/metadata-{len(transport.receipts):04d}.json", canonical_json(receipt)))
    executors = _executor_identity(repo_root)
    git_tool = git_identity()
    metadata_toolchain = toolchain_identity(repo_root)
    topology_paths = inputs.live_git_paths(repo_root)
    base = _git(repo_root, "rev-parse", "HEAD").decode().strip()
    try:
        os.environ.update(TMP=str(scratch), TEMP=str(scratch))
        metadata_cache = {}
        fingerprints, external, topology = inputs.plan_inputs(
                repo_root, metadata_target, DECLARED_COMMANDS,
                metadata_cache=metadata_cache, transport=transport)
        external_topology = {sibling: inputs.live_git_paths(sibling) for sibling in external}
        external_bases = {sibling: _git(sibling, "rev-parse", "HEAD").decode().strip()
            for sibling in external}
        if require_live_storage_owner(root, repo_root) != lease:
            raise PilotError("storage lease changed before closure capture")
        paths = sorted(fingerprints)
        # These are exact planner-selected files, never a broad directory or '*'.
        sealed = capture(repo_root, storage, paths=paths, untracked_allowlist=paths, external=external)
        verify_captured_inputs(sealed, fingerprints, external)
        # Plan again against fresh Git inventories. Cached immutable metadata
        # avoids a duplicate Cargo launch but cannot hide newly selected files.
        repeated = inputs.plan_inputs(
                repo_root, metadata_target, DECLARED_COMMANDS,
                metadata_cache=metadata_cache, transport=transport)
        if repeated != (fingerprints, external, topology):
            raise PilotError("Cargo closure selection changed during capture")
        sealed_bases = {sibling["mount"]: sibling["baseCommit"]
                        for sibling in sealed.manifest.get("external", [])}
        for sibling, selected in external.items():
            current_external_base = _git(sibling, "rev-parse", "HEAD").decode().strip()
            if (current_external_base != external_bases[sibling]
                    or sealed_bases.get(sibling.name) != external_bases[sibling]
                    or inputs.live_git_paths(sibling) != external_topology[sibling]):
                raise PilotError("external Git base or source topology changed during closure capture")
            for relative, expected in selected.items():
                if inputs.input_fingerprint(sibling, relative) != expected:
                    raise PilotError("external Cargo closure changed during capture")
        for relative, expected in {**topology, **fingerprints}.items():
            if inputs.input_fingerprint(repo_root, relative) != expected:
                raise PilotError("Cargo closure changed after planning or during capture")
        if inputs.live_git_paths(repo_root) != topology_paths:
            raise PilotError("Cargo source topology changed during closure capture")
        current_base = _git(repo_root, "rev-parse", "HEAD").decode().strip()
        if (sealed.manifest["baseCommit"] != base or current_base != base
                or _executor_identity(repo_root) != executors or git_identity() != git_tool):
            raise PilotError("source base or trusted metadata executor changed during capture")
        if toolchain_identity(repo_root) != metadata_toolchain:
            raise PilotError("metadata toolchain changed during closure capture")
        if require_live_storage_owner(root, repo_root) != lease:
            raise PilotError("storage lease changed during closure capture")
        complete_manifest = {"source/" + name: identity for name, identity in fingerprints.items() if identity is not None}
        for sibling, selected in external.items():
            complete_manifest.update({sibling.name + "/" + name: identity for name, identity in selected.items() if identity is not None})
        result = {"schemaVersion": sealed.manifest.get("schemaVersion", 1),
                  "acceptance": "pilot-evidence", "formalAcceptance": False,
                  "commands": DECLARED_COMMANDS, "paths": paths, "allowNew": paths,
                  "fingerprints": fingerprints, "topology": topology,
                  "externalFingerprints": {str(sibling): selected for sibling, selected in external.items()},
                  "externalBases": {str(sibling): base for sibling, base in external_bases.items()},
                  "externalTopology": {str(sibling): sorted(paths) for sibling, paths in external_topology.items()},
                  "sourceManifestDigest": manifest_digest(complete_manifest),
                  "bundle": str(sealed.bundle), "bundleHash": sealed.bundle_hash,
                  "inputHash": sealed.input_hash, "baseCommit": base,
                  "executorManifest": executors, "gitIdentity": git_tool,
                  "metadataToolchainIdentity": metadata_toolchain, "metadataReceipts": transport.receipts}
        storage.atomic_write(output, canonical_json(result))
        return result
    finally:
        for name, value in original_environment.items():
            if value is None:
                os.environ.pop(name, None)
            else:
                os.environ[name] = value


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--repo-root", type=Path, required=True)
    parser.add_argument("--output", default="cargo-closure.json")
    args = parser.parse_args()
    result = seal_closure(root=args.root, repo_root=args.repo_root.resolve(strict=True), output=args.output)
    print(canonical_json({key: result[key] for key in ("bundle", "bundleHash", "inputHash", "baseCommit")}).decode())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
