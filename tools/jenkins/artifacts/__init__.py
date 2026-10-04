"""Content addressed artifact publication and reference accounting."""
from .store import (ArtifactStore, ArtifactManifest, ArtifactRef, ArtifactRegistry,
                    publish_execution_outputs, attach_execution_outputs,
                    release_execution_outputs, inspect_execution_outputs,
                    find_reusable_outputs)
def handle(action, payload, state, repo_root, *, domain="artifacts"):
    if action == "gc":
        from ..maintenance import handle as maintenance_handle
        return maintenance_handle("gc", payload, state, repo_root)
    from ..resources.paths import canonical_build_root, build_namespace
    root=canonical_build_root(payload.get("buildRoot"))
    namespace = build_namespace(root, "artifacts")
    from ..contracts import response
    result = ArtifactRegistry(state, ArtifactStore(namespace)).handle(action, payload)
    return response("accepted", operation_id=payload.get("operationId"), result=result)
__all__ = ["ArtifactStore", "ArtifactManifest", "ArtifactRef", "ArtifactRegistry",
           "publish_execution_outputs", "attach_execution_outputs",
           "release_execution_outputs", "inspect_execution_outputs",
           "find_reusable_outputs"]
