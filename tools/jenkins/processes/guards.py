"""Failure-only evidence from a retained outer execution Job Object."""
from __future__ import annotations

from ..contracts import JenkinsError, digest


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
    for row in state.list("native_job"):
        job = row["payload"]
        if job.get("executionId") != execution_id or row["key"] == guard["key"]:
            continue
        if job.get("guardNativeJobId") != guard["key"]:
            raise JenkinsError("execution_abort_unproven", "A native phase is not bound to the proven outer Job")
    return {"kind": "execution-host-aborted", "guardNativeJobId": guard["key"],
            "proofDigest": digest(proof), "outcome": "failed", "artifactPublicationAllowed": False}
