"""Owned native process supervision for the Jenkins support layer.

The module keeps process identity and termination evidence together.  Windows
launches use a kill-on-close Job Object so descendants cannot escape the
execution boundary; non-Windows callers get a small subprocess fallback for
unit tests and development tooling.
"""

from .identity import ProcessIdentity, current_identity, identity_matches, process_alive
from .job import NativeJob, ManagedProcess, TerminationProof
from .registry import NativeJobRecord, ProcessRegistry

__all__ = [
    "ManagedProcess",
    "NativeJob",
    "ProcessIdentity",
    "TerminationProof",
    "NativeJobRecord",
    "ProcessRegistry",
    "current_identity",
    "identity_matches",
    "process_alive",
    "ExecutionHost",
    "HostIdentity",
    "request_cancel",
]

def __getattr__(name):
    """Load the owner host lazily to keep primitive process imports acyclic."""
    if name in {"ExecutionHost", "HostIdentity", "request_cancel"}:
        from .execution_host import ExecutionHost, HostIdentity, request_cancel
        return {"ExecutionHost": ExecutionHost, "HostIdentity": HostIdentity,
                "request_cancel": request_cancel}[name]
    raise AttributeError(name)
