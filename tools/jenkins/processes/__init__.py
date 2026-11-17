"""Owned native process supervision for the Jenkins support layer.

The module keeps process identity and termination evidence together.  Windows
launches use a kill-on-close Job Object so descendants cannot escape the
execution boundary; non-Windows callers get a small subprocess fallback for
unit tests and development tooling.
"""

# Public exports stay lazy so module CLIs can retire before loading native owners.

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
    """Preserve native library exports without loading them for CLI startup."""
    from importlib import import_module
    if name in {"ProcessIdentity", "current_identity", "identity_matches", "process_alive"}:
        module = import_module(".identity", __name__)
    elif name in {"NativeJob", "ManagedProcess", "TerminationProof"}:
        module = import_module(".job", __name__)
    elif name in {"NativeJobRecord", "ProcessRegistry"}:
        module = import_module(".registry", __name__)
    elif name in {"ExecutionHost", "HostIdentity", "request_cancel"}:
        module = import_module(".execution_host", __name__)
    else:
        raise AttributeError(name)
    value = getattr(module, name)
    globals()[name] = value
    return value
