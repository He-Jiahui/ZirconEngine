"""Durable recovery start intent; ambiguity never authorizes another spawn."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import time

from ..contracts import JenkinsError
from .keeper_control import _raw, _sha, _write


class RecoveryStartTransaction:
    def __init__(self, manager, capability, operation_id):
        self.manager = manager
        self.path = manager.paths.state / "recovery-lifecycle/start-transactions" / (operation_id + ".json")
        if self.path.exists():
            raise JenkinsError("recovery_start_replayed", "The original start transaction must be reconciled")
        selector = manager.paths.state / "deployment/driver.json"
        host = manager.paths.state / "deployment/host.json"
        self.value = {"schemaVersion": 1, "kind": "formal-recovery-start-transaction",
            "operationId": operation_id, "state": "prepared", "keeper": capability.actor.keeper,
            "release": capability.actor.release_ref, "request": capability.request,
            "previousOperation": manager._read(), "previousHost": json.loads(_raw(host)),
            "previousSelector": json.loads(_raw(selector)), "previousSelectorSha256": _sha(selector),
            "homePath": str(manager.paths.home.absolute()), "spawnAttempted": False,
            "automaticRetry": False, "runtimeAcceptance": False, "cargoAcceptance": False,
            "nativeTerminationProven": False, "historicalHoldsReleased": False}
        self._save()

    def _save(self):
        expected = _sha(self.path) if self.path.exists() else None
        self.value["updatedAt"] = time.time()
        _write(self.path, self.value, expected_sha=expected)

    def update(self, state, **evidence):
        self.value.update(evidence)
        self.value["state"] = state
        self._save()

    def before_spawn(self, args, selected):
        self.update("spawn-attempted", spawnAttempted=True, requestedCommand=list(args),
                    requestedCommandDigest=hashlib.sha256("\0".join(args).encode("utf-8")).hexdigest(),
                    selected=selected, selectedSha256=_sha(self.manager.paths.state / "deployment/driver.json"))

    def created(self, pid, identity=None):
        evidence = {"createdPid": pid}
        if identity is not None:
            evidence["createdIdentity"] = identity
        self.update("spawned-identity-pending" if identity is None else "spawned", **evidence)

    def unknown(self, error):
        self.update("unknown", reasonCode=getattr(error, "code", type(error).__name__), message=str(error))


__all__ = ["RecoveryStartTransaction"]
