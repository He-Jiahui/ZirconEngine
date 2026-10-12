"""Failure-only evidence from a retained outer execution Job Object."""
from __future__ import annotations

from ..contracts import JenkinsError, digest


def guard_phases_complete(state, guard, *, connection=None):
    """An outer Job's accounting cannot replace each nested phase's proof."""
    expected = guard["payload"]
    for row in state.list("native_job", connection=connection):
        job = row["payload"]
        if row["key"] == guard["key"]:
            continue
        if (job.get("executionId") != expected.get("executionId")
                and job.get("owner") != expected.get("owner")
                and job.get("guardNativeJobId") != guard["key"]):
            continue
        proof = job.get("completeProof")
        identity = proof.get("identity") if isinstance(proof, dict) else None
        if (job.get("role") != "phase" or job.get("owner") != expected.get("owner")
                or job.get("executionId") != expected.get("executionId")
                or job.get("guardNativeJobId") != guard["key"] or job.get("status") != "terminal"
                or not isinstance(proof, dict) or proof.get("complete") is not True
                or job.get("terminalProof") != proof or not isinstance(identity, dict)
                or type(job.get("pid")) is not int or job["pid"] <= 0
                or not isinstance(job.get("birthToken"), str) or not job["birthToken"].isdecimal()
                or int(job["birthToken"]) <= 0 or not isinstance(job.get("executable"), str)
                or not job["executable"] or type(identity.get("pid")) is not int
                or identity.get("pid") != job.get("pid")
                or identity.get("creationTime") != job.get("birthToken")
                or identity.get("executable") != job.get("executable")
                or type(proof.get("processExitCode")) is not int
                or type(proof.get("activeProcesses")) is not int or proof["activeProcesses"] != 0
                or any(proof.get(field) is not True for field in
                       ("childrenGone", "stdoutEof", "stderrEof"))):
            return False
    return True


def abort_evidence(state, execution_id: str):
    record = state.get("execution_abort", execution_id)
    if not record:
        from .boot import boot_abort_evidence
        return boot_abort_evidence(state, execution_id)
    abort = record["payload"]
    launch = state.get("execution_launch", execution_id)
    guard = state.get("native_job", abort.get("guardNativeJobId", ""))
    proof = guard["payload"].get("completeProof") if guard else None
    if (not launch or launch["payload"].get("guardNativeJobId") != abort.get("guardNativeJobId")
            or not guard or guard["payload"].get("role") != "execution_host_guard"
            or guard["payload"].get("owner") != execution_id
            or guard["payload"].get("executionId") != execution_id
            or guard["payload"].get("status") != "terminal"
            or not isinstance(proof, dict) or proof.get("complete") is not True
            or digest(proof) != abort.get("proofDigest")):
        raise JenkinsError("execution_abort_unproven", "Host abort lacks its enclosing native Job proof")
    if not guard_phases_complete(state, guard):
        raise JenkinsError("execution_abort_unproven", "A nested native phase lacks its own complete termination proof")
    return {"kind": "execution-host-aborted", "guardNativeJobId": guard["key"],
            "proofDigest": digest(proof), "outcome": "failed", "artifactPublicationAllowed": False}
