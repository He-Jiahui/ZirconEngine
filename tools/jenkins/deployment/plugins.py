from __future__ import annotations

import hashlib
import json
import os
import shutil
import time
import uuid
import re
import zipfile
from pathlib import Path
from typing import Any

from .spec import JenkinsError
from tools.jenkins.state.locks import process_lock

CORE_VERSION = "2.580.1"


def _terminated_identity(value: object) -> bool:
    """Require the recorded native identity to be absent now.

    A PID and an old proof are not sufficient: a reused PID or a still-running
    process must block offline mutation.
    """
    if not isinstance(value, dict) or not value.get("pid") or not value.get("creationTime"):
        return False
    try:
        from tools.jenkins.processes.identity import process_alive, current_identity
        alive = process_alive(int(value["pid"]))
        if alive is not False:
            return False
        # process_alive(False) establishes the PID is gone; retain the full
        # identity in the receipt so a later PID reuse cannot be mistaken for it.
        return True
    except Exception:
        return False


def read_stopped_operation(state_dir: str | Path, operation_id: str | None = None, *, expected_home: str | Path | None = None) -> dict[str, Any]:
    """Read the deployment manager's stopped operation receipt.

    The receipt is the sole authority.  Callers cannot manufacture a proof by
    passing a dictionary to the installer.
    """
    path = Path(state_dir) / "deployment-operation.json"
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError) as exc:
        raise JenkinsError("stop_proof_missing", "deployment stop receipt is unavailable") from exc
    if not isinstance(value, dict) or value.get("state") != "stopped" or not value.get("operationId"):
        raise JenkinsError("stop_proof_invalid", "deployment operation is not stopped")
    if operation_id is not None and value.get("operationId") != operation_id:
        raise JenkinsError("stop_proof_stale", "deployment stop receipt does not match the requested operation")
    if expected_home is not None:
        recorded_home = value.get("homePath")
        if not recorded_home or os.path.normcase(str(Path(recorded_home).absolute())) != os.path.normcase(str(Path(expected_home).absolute())):
            raise JenkinsError("stop_proof_home_mismatch", "stopped operation belongs to a different Jenkins Home")
    host_path = Path(state_dir) / "deployment" / "host.json"
    try:
        host = json.loads(host_path.read_text(encoding="utf-8"))
    except (OSError, ValueError) as exc:
        raise JenkinsError("stop_proof_missing", "lifecycle host stop receipt is unavailable") from exc
    if host.get("status") != "stopped" or host.get("operationId") != value.get("operationId") or \
            host.get("generation") != value.get("generation"):
        raise JenkinsError("stop_proof_stale", "lifecycle host receipt does not match this stopped generation")
    native = value.get("nativeTerminationProof")
    agent = value.get("agentTerminationProof")
    if not isinstance(native, dict) or not isinstance(agent, dict):
        raise JenkinsError("stop_proof_invalid", "controller and agent termination proofs are required")
    for label, proof in (("controller", native), ("agent", agent)):
        if proof.get("complete") is not True or proof.get("activeProcesses") != 0 or \
                proof.get("stdoutEof") is not True or proof.get("stderrEof") is not True or \
                not _terminated_identity(proof.get("identity")):
            raise JenkinsError("stop_proof_invalid", f"{label} native termination is not proven")
    host_native = host.get("nativeTerminationProof")
    host_agent = host.get("agentTerminationProof")
    if not isinstance(host_native, dict) or not isinstance(host_agent, dict) or \
            host_native.get("identity") != native.get("identity") or host_agent.get("identity") != agent.get("identity"):
        raise JenkinsError("stop_proof_stale", "deployment and native host proofs disagree")
    return value


class PluginMaintenance:
    """Offline plugin installer owned by the deployment maintenance job."""

    def __init__(self, home: str | Path, state_dir: str | Path):
        self.home = Path(home).absolute()
        self.state_dir = Path(state_dir).absolute()
        if self.home.name != "jenkins_home" or self.home.parent.name != ".jenkins":
            raise JenkinsError("home_path_rejected", "plugin maintenance is limited to formal Jenkins Home")

    def install(self, stage: str | Path, manifest: dict[str, Any], *, operation_id: str | None = None) -> dict[str, Any]:
        receipt = read_stopped_operation(self.state_dir, operation_id, expected_home=self.home)
        op = str(receipt["operationId"])
        if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_.-]{0,127}", op):
            raise JenkinsError("operation_id_rejected", "maintenance operation id contains unsafe characters")
        with process_lock(self.state_dir / "deployment-start.lock", reason="plugin_maintenance_busy"):
            # Re-read while holding the lifecycle lock; start/stop cannot race
            # the package replacement or rollback.
            read_stopped_operation(self.state_dir, op, expected_home=self.home)
            return self._install_unlocked(stage, manifest, operation_id=op)

    def _install_unlocked(self, stage: str | Path, manifest: dict[str, Any], *, operation_id: str | None = None) -> dict[str, Any]:
        receipt = read_stopped_operation(self.state_dir, operation_id, expected_home=self.home)
        op = str(receipt["operationId"])
        verified = verify_staged_plugins(stage, manifest)
        self.state_dir.mkdir(parents=True, exist_ok=True)
        journal_dir = self.state_dir / "plugin-maintenance"; journal_dir.mkdir(exist_ok=True)
        journal = journal_dir / f"{op}.json"
        backup = self.home / "plugin-backups" / op
        incoming = journal_dir / f"{op}.incoming"
        plugins = self.home / "plugins"
        backup.mkdir(parents=True, exist_ok=False)
        value = {"schemaVersion": 1, "operationId": op, "state": "verified",
                 "verified": verified, "backup": str(backup), "stage": str(Path(stage).absolute())}
        journal.write_text(json.dumps(value, sort_keys=True, indent=2), encoding="utf-8")
        try:
            if plugins.exists(): shutil.copytree(plugins, backup / "plugins")
            (backup / "manifest.json").write_text(json.dumps({"plugins": sorted(verified)}, sort_keys=True), encoding="utf-8")
            shutil.copytree(Path(stage), incoming)
            # Ensure only locked archives are activated, even if the staging
            # directory contains unrelated files.
            active = incoming / "active"; active.mkdir()
            for name, entry in manifest["plugins"].items():
                filename = entry.get("file", name + ".jpi")
                shutil.copy2(incoming / filename, active / filename)
            if plugins.exists(): plugins.rename(backup / "previous-plugins")
            active.rename(plugins)
            shutil.rmtree(incoming, ignore_errors=True)
            value.update({"state": "installed", "activatedAt": time.time()})
            journal.write_text(json.dumps(value, sort_keys=True, indent=2), encoding="utf-8")
            return value
        except Exception as exc:
            # Restore the previous valid set before reporting failure.
            if plugins.exists(): shutil.rmtree(plugins, ignore_errors=True)
            previous = backup / "previous-plugins"
            if previous.exists(): previous.rename(plugins)
            elif (backup / "plugins").exists(): shutil.copytree(backup / "plugins", plugins)
            value.update({"state": "rolled-back", "error": type(exc).__name__})
            journal.write_text(json.dumps(value, sort_keys=True, indent=2), encoding="utf-8")
            if isinstance(exc, JenkinsError): raise
            raise JenkinsError("plugin_install_rolled_back", "plugin activation failed and previous set was restored") from exc

    def rollback(self, backup: str | Path, *, operation_id: str | None = None) -> dict[str, Any]:
        """Restore a recorded set while the same stopped authority is held."""
        receipt = read_stopped_operation(self.state_dir, operation_id)
        op = str(receipt["operationId"])
        backup_path = Path(backup).absolute()
        if backup_path != (self.home / "plugin-backups" / op).absolute():
            raise JenkinsError("plugin_backup_home_mismatch", "plugin backup is outside the operation's Jenkins Home")
        with process_lock(self.state_dir / "deployment-start.lock", reason="plugin_rollback_busy"):
            read_stopped_operation(self.state_dir, op, expected_home=self.home)
            previous = backup_path / "previous-plugins"
            if not previous.exists(): previous = backup_path / "plugins"
            if not previous.is_dir(): raise JenkinsError("plugin_backup_missing", "plugin backup does not exist")
            plugins = self.home / "plugins"
            if plugins.exists(): shutil.rmtree(plugins)
            shutil.copytree(previous, plugins)
            return {"status": "rolled_back", "operationId": op, "backup": str(backup_path)}

def _version(value: str) -> tuple[int, ...]:
    import re
    return tuple(int(x) for x in re.findall(r"\d+", str(value)))


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""): h.update(block)
    return h.hexdigest()


def _jar_manifest(path: Path) -> dict[str, str]:
    try:
        with zipfile.ZipFile(path) as archive:
            raw = archive.read("META-INF/MANIFEST.MF").decode("utf-8", "replace")
    except (zipfile.BadZipFile, KeyError, OSError) as exc:
        raise JenkinsError("plugin_archive_invalid", "plugin package is not a valid JPI/HPI archive") from exc
    unfolded: list[str] = []
    for line in raw.replace("\r\n", "\n").split("\n"):
        if line.startswith(" ") and unfolded:
            unfolded[-1] += line[1:]
        else:
            unfolded.append(line)
    values: dict[str, str] = {}
    for line in unfolded:
        if ":" in line:
            key, value = line.split(":", 1)
            values[key.strip().lower()] = value.strip()
    return values


def _manifest_dependencies(raw: str) -> list[tuple[str, str, bool]]:
    parsed = []
    for item in filter(None, (part.strip() for part in raw.split(","))):
        head, *attrs = item.split(";")
        name, sep, version = head.partition(":")
        if not sep or not name or not version:
            raise JenkinsError("plugin_dependency_metadata_invalid", "plugin archive dependency metadata is malformed")
        optional = any(attr.strip().lower() == "resolution:=optional" for attr in attrs)
        parsed.append((name, version, optional))
    return sorted(parsed)


def load_checksum_manifest(path: str | Path) -> dict[str, Any]:
    try: value = json.loads(Path(path).read_text(encoding="utf-8"))
    except (OSError, ValueError) as exc: raise JenkinsError("invalid_plugin_manifest", "cannot read plugin checksum manifest") from exc
    if not isinstance(value, dict) or value.get("schemaVersion") != 1 or not isinstance(value.get("plugins"), dict):
        raise JenkinsError("invalid_plugin_manifest", "plugin checksum manifest has an unsupported schema")
    return value


def verify_staged_plugins(stage: str | Path, manifest: dict[str, Any]) -> dict[str, str]:
    stage = Path(stage)
    if not stage.is_dir(): raise JenkinsError("plugin_stage_missing", "plugin stage does not exist")
    if manifest.get("jenkinsVersion") and _version(manifest["jenkinsVersion"]) != _version(CORE_VERSION):
        raise JenkinsError("plugin_core_mismatch", "plugin manifest targets a different Jenkins core")
    names = set(manifest["plugins"])
    for plugin, entry in manifest["plugins"].items():
        if _version(entry.get("requiredCore", "0")) > _version(CORE_VERSION):
            raise JenkinsError("plugin_core_too_old", "plugin requires a newer Jenkins core", details={"plugin": plugin})
        for dependency in entry.get("dependencies", []):
            dep = dependency.get("name") if isinstance(dependency, dict) else dependency
            optional = bool(dependency.get("optional")) if isinstance(dependency, dict) else False
            if not optional and dep not in names:
                raise JenkinsError("plugin_dependency_missing", "required plugin dependency is absent", details={"plugin": plugin, "dependency": dep})
    verified: dict[str, str] = {}
    for plugin, entry in manifest["plugins"].items():
        filename = entry.get("file", plugin + ".jpi")
        path = stage / filename
        if Path(filename).name != filename or Path(filename).suffix.lower() not in {".jpi", ".hpi"}:
            raise JenkinsError("plugin_path_rejected", "plugin archive filename is unsafe", details={"plugin": plugin})
        if not path.is_file(): raise JenkinsError("plugin_missing", "staged plugin is missing", details={"plugin": plugin})
        actual = sha256(path)
        if actual.lower() != str(entry.get("sha256", "")).lower():
            raise JenkinsError("plugin_checksum_mismatch", "staged plugin checksum does not match lock", details={"plugin": plugin})
        if not entry.get("version") and not entry.get("dependencies"):
            verified[plugin] = actual
            continue
        meta = _jar_manifest(path)
        embedded_id = meta.get("short-name") or meta.get("plugin-id")
        embedded_version = meta.get("plugin-version") or meta.get("implementation-version")
        if embedded_id != plugin:
            raise JenkinsError("plugin_id_mismatch", "plugin archive id does not match lock", details={"plugin": plugin})
        if not entry.get("version") or str(entry["version"]) != embedded_version:
            raise JenkinsError("plugin_version_mismatch", "plugin archive version does not match lock", details={"plugin": plugin})
        expected_deps = sorted((str(dep["name"]), str(dep.get("version", "")), bool(dep.get("optional", False)))
                               for dep in entry.get("dependencies", []))
        actual_deps = _manifest_dependencies(meta.get("plugin-dependencies", ""))
        if expected_deps != actual_deps:
            raise JenkinsError("plugin_dependency_mismatch", "plugin archive dependencies do not match lock", details={"plugin": plugin})
        verified[plugin] = actual
    return verified


def stage_operation(state_dir: Path, operation_id: str, manifest: dict[str, Any]) -> Path:
    stage = state_dir / "plugin-staging" / operation_id
    stage.mkdir(parents=True, exist_ok=False)
    (stage / "manifest.json").write_text(json.dumps(manifest, sort_keys=True, indent=2), encoding="utf-8")
    return stage
