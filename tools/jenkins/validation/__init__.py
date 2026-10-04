"""Authoritative Jenkins validation receipt API."""
from .receipts import (
    ACCEPTANCE_KIND, PRODUCER_KIND, closure_manifest,
    issue_acceptance_receipt, validate_acceptance_receipt,
    validate_producer_receipt,
    TrustedReceiptAuthority,
)

__all__ = [
    "ACCEPTANCE_KIND", "PRODUCER_KIND", "closure_manifest",
    "issue_acceptance_receipt", "validate_acceptance_receipt",
    "validate_producer_receipt",
    "TrustedReceiptAuthority",
]


def handle(action, payload, state, repo_root=None, *, domain="validation"):
    """Owner entrypoint for pipeline and execution adapters."""
    authority = TrustedReceiptAuthority(state, repo_root=repo_root)
    if action in {"register", "record"}:
        raise ValueError("register and record are internal trusted-collector operations")
    if action == "get":
        return authority.get(payload["executionId"])
    if action == "accept":
        return authority.accept(payload["executionId"], payload.get("expected", {}))
    if action == "accept-flow":
        return authority.accept_flow(payload)
    if action == "reconcile":
        return authority.get(payload["executionId"])
    raise ValueError(f"unknown validation action: {action}")
