"""Trusted in-process native job registry.

The registry accepts identities and proofs produced by :mod:`job`; callers
cannot inject a PID or terminal boolean. Persistence is intentionally owned by
the state layer, so this is the canonical payload adapter used by it.
"""
from __future__ import annotations

from dataclasses import dataclass
from threading import RLock
from uuid import uuid4

from .job import NativeJob, TerminationProof
from tools.jenkins.state import State


@dataclass(frozen=True, slots=True)
class NativeJobRecord:
    native_job_id: str
    execution_id: str
    owner: str
    payload: dict[str, object]


class ProcessRegistry:
    def __init__(self, state: State) -> None:
        self.state = state
        self._jobs: dict[str, NativeJob] = {}
        self._lock = RLock()

    def register(self, job: NativeJob, *, execution_id: str, owner: str, role: str = "phase") -> NativeJobRecord:
        native_id = uuid4().hex
        payload = {
            "nativeJobId": native_id,
            "executionId": execution_id,
            "owner": owner,
            "pid": job.identity.pid,
            "birthToken": job.identity.creation_time,
            "executable": job.identity.executable,
            "status": "running",
            "role": role,
            "jobName": getattr(job, "job_name", None),
        }
        if role == "phase":
            launch = self.state.get("execution_launch", execution_id)
            if launch and launch["payload"].get("guardNativeJobId"):
                import os
                from .identity import ProcessIdentity, identity_matches
                ident = launch["payload"].get("launcherIdentity") or {}
                guard = self.state.get("native_job", launch["payload"]["guardNativeJobId"])
                if (ident.get("pid") != os.getpid() or not guard
                        or guard["payload"].get("role") != "execution_host_guard"
                        or guard["payload"].get("owner") != owner
                        or not identity_matches(ProcessIdentity(ident["pid"], ident["creationTime"]))):
                    raise ValueError("execution_guard_identity_mismatch")
                from . import native
                guard_handle = native.open_process_job(str(guard["payload"]["jobName"]))
                try:
                    if not native._process_is_in_job(int(getattr(job.process.process, "_handle", 0)), guard_handle):
                        raise ValueError("native_phase_not_in_execution_guard")
                finally:
                    native.close_process_job(guard_handle)
                payload["guardNativeJobId"] = guard["key"]
        record = NativeJobRecord(native_id, execution_id, owner, payload)
        with self.state.transaction() as connection:
            self.state.put("native_job", native_id, payload, connection=connection)
            self.state.event("native_job_registered", {"nativeJobId": native_id, "executionId": execution_id}, connection=connection)
        with self._lock:
            self._jobs[native_id] = job
        return record

    def record_terminal(self, job: NativeJob, proof: TerminationProof | None = None) -> NativeJobRecord:
        with self._lock:
            match = next((key for key, value in self._jobs.items() if value is job), None)
            if match is None:
                raise KeyError("unregistered_native_job")
            current_state = self.state.get("native_job", match)
            if current_state is None:
                raise KeyError("missing_native_job_state")
            current_payload = current_state["payload"]
            current = NativeJobRecord(match, str(current_payload["executionId"]), str(current_payload["owner"]), current_payload)
            observed = job.proof(requested=True)
            payload = dict(current.payload)
            payload["status"] = "terminal" if observed.complete else "termination_unproven"
            payload["terminalProof"] = observed.to_dict()
            payload["completeProof"] = observed.to_dict() if observed.complete else None
            with self.state.transaction() as connection:
                self.state.put("native_job", match, payload, expected_version=current_state["version"], connection=connection)
                self.state.event("native_job_terminal", {"nativeJobId": match, "complete": bool(observed.complete)}, connection=connection)
            updated = NativeJobRecord(current.native_job_id, current.execution_id, current.owner, payload)
            return updated

    def get(self, native_job_id: str) -> NativeJobRecord:
        with self._lock:
            row = self.state.get("native_job", native_job_id)
            if row is None:
                raise KeyError(native_job_id)
            payload = row["payload"]
            return NativeJobRecord(native_job_id, str(payload["executionId"]), str(payload["owner"]), payload)
