"""Formal Jenkins deployment support.

This package owns the repository-local controller lifecycle.  It deliberately
does not import the retired pilot runtime.
"""
from pathlib import Path

from .spec import DeploymentSpec, load_spec
from .paths import DeploymentPaths, resolve_paths
from .spec import JenkinsError

def handle(action, payload, state, repo_root, *, domain="deployment"):
    spec = load_spec(payload.get("spec", repo_root / ".jenkins" / "deployment-spec.json"))
    paths = resolve_paths(spec, payload.get("buildRoot"))
    if action == "validate": return {"status": "ready", "repositoryRoot": str(spec.repository_root), "home": str(paths.home), "buildRoot": str(paths.build_root)}
    if action == "prepare-runtime":
        for p in (paths.root, paths.home, paths.runtime, paths.logs, paths.state, paths.cache, paths.tmp, paths.agent): p.mkdir(parents=True, exist_ok=True)
        return {"status": "prepared", "runtimeRoot": str(paths.runtime), "home": str(paths.home), "state": str(paths.state)}
    if action == "prepare-jobs":
        from .manager import DeploymentManager
        java = Path(payload.get("java") or spec.controller["java"]["executable"])
        war = Path(payload.get("war") or spec.controller["warPath"])
        manager = DeploymentManager(spec, paths, java, war)
        return {"status": "prepared", **manager.register_jobs()}
    from .manager import DeploymentManager
    java = Path(payload.get("java") or spec.controller["java"]["executable"])
    war = Path(payload.get("war") or spec.controller["warPath"])
    manager = DeploymentManager(spec, paths, java, war)
    if action == "recover-activation":
        return manager.recover_activation(transition_digest=payload.get("transitionDigest"))
    if action in {"health", "start", "stop", "reconcile"}: return getattr(manager, action)()
    raise JenkinsError("operation_unknown", f"unknown deployment action: {action}")

__all__ = ["DeploymentSpec", "DeploymentPaths", "load_spec", "resolve_paths"]
