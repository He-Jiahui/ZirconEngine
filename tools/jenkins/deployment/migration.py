from __future__ import annotations

import hashlib
import json
import shutil
from pathlib import Path

from .spec import JenkinsError


EXCLUDED_RUNTIME = {"queue.xml", "queue.xml.bak", "secret.key.not-so-secret"}

def tree_digest(root: Path) -> str:
    h = hashlib.sha256()
    for path in sorted((p for p in root.rglob("*") if p.is_file() and p.name not in EXCLUDED_RUNTIME), key=lambda p: str(p.relative_to(root)).casefold()):
        h.update(str(path.relative_to(root)).replace("\\", "/").encode()); h.update(path.read_bytes())
    return h.hexdigest()


def migrate_home(source: Path, destination: Path, *, stop_proof: Path | None = None) -> dict[str, str]:
    if not source.is_dir(): raise JenkinsError("home_missing", "source Jenkins Home does not exist")
    if destination.exists() and any(destination.iterdir()): raise JenkinsError("home_not_empty", "destination Jenkins Home is not empty")
    source = source.resolve(); destination = destination.resolve()
    if stop_proof is None: raise JenkinsError("migration_stop_proof_required", "Home migration requires a persisted native stop proof")
    try: proof = json.loads(Path(stop_proof).read_text(encoding="utf-8"))
    except (OSError, ValueError) as exc: raise JenkinsError("migration_stop_proof_missing", "native stop proof could not be read") from exc
    runtime = proof.get("proof", {}).get("runtime", {})
    controller = runtime.get("controller", {}); agent = runtime.get("agent", {})
    drained = (proof.get("state") in {"stopped", "verified-stopped"} or proof.get("stopped") is True)
    drained = drained and proof.get("sourceHome") in {None, str(source)}
    drained = drained and proof.get("terminalVerified", True) is True
    drained = drained and proof.get("activeJobs", 0) == 0 and proof.get("queuedJobs", 0) == 0
    drained = drained and (not runtime or (controller.get("activeProcesses", 0) == 0 and agent.get("activeProcesses", 0) == 0 and controller.get("logReleased") is True and agent.get("logReleased") is True))
    if not drained:
        raise JenkinsError("migration_stop_proof_invalid", "native stop proof does not establish a drained source")
    if source == destination or destination.is_relative_to(source): raise JenkinsError("home_path_rejected", "source and destination Jenkins Homes overlap")
    for p in source.rglob("*"):
        if p.is_symlink(): raise JenkinsError("home_alias", "Jenkins Home migration refuses symlinked content", details={"path": str(p)})
    before = tree_digest(source)
    destination.mkdir(parents=True, exist_ok=True)
    shutil.copytree(source, destination, dirs_exist_ok=True, ignore=shutil.ignore_patterns(*EXCLUDED_RUNTIME))
    after = tree_digest(destination)
    if before != after: raise JenkinsError("home_copy_mismatch", "Jenkins Home copy failed digest verification")
    return {"sourceDigest": before, "destinationDigest": after}
