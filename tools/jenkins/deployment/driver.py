"""Seal and verify the Python support code and Scripted Pipeline implementation."""
from __future__ import annotations

import ast
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import shutil
import uuid

from tools.jenkins.contracts import JenkinsError, canonical_json, digest


_LAUNCHER = '''"""Verify the complete snapshot before importing its control entrypoint."""
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import runpy
import sys

root = Path(__file__).absolute().parent
manifest = json.loads((root / "driver-manifest.json").read_text(encoding="utf-8"))
unsigned = {"schemaVersion": manifest["schemaVersion"], "files": manifest["files"]}
encoded = json.dumps(unsigned, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")
if hashlib.sha256(encoded).hexdigest() != root.name or manifest.get("driverDigest") != root.name:
    raise SystemExit("sealed driver identity mismatch")
for name, entry in manifest["files"].items():
    relative = PurePosixPath(name)
    if relative.is_absolute() or any(p in {".", ".."} for p in relative.parts) or "\\\\" in name or ":" in name:
        raise SystemExit("sealed driver path rejected")
    path = root.joinpath(*relative.parts)
    for part in [path, *path.parents]:
        if part == root.parent:
            break
        if part.is_symlink() or getattr(part.lstat(), "st_file_attributes", 0) & 0x400:
            raise SystemExit("sealed driver alias rejected")
    data = path.read_bytes()
    if len(data) != entry["bytes"] or hashlib.sha256(data).hexdigest() != entry["sha256"]:
        raise SystemExit("sealed driver bytes changed")
observed = set()
for path in root.rglob("*"):
    if path.is_symlink() or getattr(path.lstat(), "st_file_attributes", 0) & 0x400:
        raise SystemExit("sealed driver alias rejected")
    if path.is_file():
        observed.add(path.relative_to(root).as_posix())
if observed != set(manifest["files"]) | {"driver-manifest.json"}:
    raise SystemExit("sealed driver contains undeclared files")
sys.path.insert(0, str(root))
os.environ["PYTHONDONTWRITEBYTECODE"] = "1"
if sys.argv[1:2] == ["__lifecycle-host"]:
    sys.argv.pop(1)
    runpy.run_module("tools.jenkins.deployment.host", run_name="__main__")
elif sys.argv[1:2] == ["__execution-host"]:
    sys.argv.pop(1)
    runpy.run_module("tools.jenkins.processes.execution_host", run_name="__main__")
else:
    runpy.run_module("tools.jenkins", run_name="__main__")
'''


def _plain(path: Path) -> None:
    """Reject every existing alias in a path before resolving it."""
    for part in (path, *path.parents):
        if part.exists() or part.is_symlink():
            attrs = part.lstat()
            if part.is_symlink() or getattr(attrs, "st_file_attributes", 0) & 0x400:
                raise JenkinsError("driver_alias_rejected", "Driver paths must have a physical identity")


def _summary(root: Path, driver_digest: str) -> dict:
    return {"driverDigest": driver_digest, "root": str(root),
            "launcher": str(root / "launcher.py"),
            "steps": str(root / "pipeline/steps.groovy"),
            "manifest": str(root / "driver-manifest.json")}


def verify_driver(root: str | Path, *, expected_digest: str | None = None) -> dict:
    root = Path(root).absolute()
    _plain(root)
    try:
        manifest = json.loads((root / "driver-manifest.json").read_text(encoding="utf-8"))
        files = manifest["files"]
        actual = digest({"schemaVersion": manifest["schemaVersion"], "files": files})
        if manifest["schemaVersion"] != 1 or not files or actual != manifest["driverDigest"]:
            raise ValueError("manifest identity")
        if expected_digest and actual != expected_digest:
            raise ValueError("expected identity")
        if root.name != actual:
            raise ValueError("snapshot directory identity")
        for name, entry in files.items():
            relative = PurePosixPath(name)
            if (relative.is_absolute() or any(p in {".", ".."} for p in relative.parts)
                    or "\\" in name or ":" in name):
                raise ValueError("manifest path")
            path = root.joinpath(*relative.parts)
            _plain(path)
            data = path.read_bytes()
            if len(data) != entry["bytes"] or hashlib.sha256(data).hexdigest() != entry["sha256"]:
                raise ValueError("snapshot content")
        observed = set()
        for path in root.rglob("*"):
            _plain(path)
            if path.is_file():
                observed.add(path.relative_to(root).as_posix())
        if observed != set(files) | {"driver-manifest.json"}:
            raise ValueError("undeclared snapshot file")
    except (OSError, ValueError, KeyError, TypeError) as exc:
        raise JenkinsError("driver_integrity_failed", "Sealed driver verification failed") from exc
    return _summary(root, actual)


def seal_driver(repository_root: str | Path) -> dict:
    """Capture each source once; existing snapshots are verified and never overwritten."""
    repo = Path(repository_root).absolute()
    _plain(repo)
    sources: dict[str, bytes] = {
        "tools/__init__.py": b'"""This package belongs to one verified Jenkins snapshot."""\n',
        "launcher.py": _LAUNCHER.encode("utf-8"),
    }
    for path in sorted((repo / "tools/jenkins").rglob("*.py")):
        relative = path.relative_to(repo).as_posix()
        if "/tests/" in relative or "__pycache__" in path.parts:
            continue
        _plain(path)
        sources[relative] = path.read_bytes()
    for path in sorted((repo / ".jenkins/pipeline").glob("*.groovy")):
        _plain(path)
        sources["pipeline/" + path.name] = path.read_bytes()
    for name, data in sources.items():
        if name.endswith(".py"):
            try:
                ast.parse(data, filename=name)
            except (SyntaxError, UnicodeError, ValueError) as exc:
                raise JenkinsError("driver_syntax_invalid", "Driver source failed syntax validation",
                                   details={"path": name}) from exc
    if "tools/jenkins/__main__.py" not in sources or "pipeline/steps.groovy" not in sources:
        raise JenkinsError("driver_inputs_missing", "Driver control entrypoint or Pipeline steps are missing")
    files = {name: {"sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data)}
             for name, data in sorted(sources.items())}
    driver_digest = digest({"schemaVersion": 1, "files": files})
    manifest = {"schemaVersion": 1, "driverDigest": driver_digest, "files": files}
    snapshots = repo / ".jenkins/runtime/drivers"
    _plain(snapshots)
    snapshots.mkdir(parents=True, exist_ok=True)
    destination = snapshots / driver_digest
    if destination.exists():
        return verify_driver(destination, expected_digest=driver_digest)
    staging = snapshots / (".incoming-" + uuid.uuid4().hex)
    staging.mkdir()
    try:
        for name, data in sources.items():
            target = staging / name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
        (staging / "driver-manifest.json").write_bytes(canonical_json(manifest) + b"\n")
        try:
            os.rename(staging, destination)
        except FileExistsError:
            verify_driver(destination, expected_digest=driver_digest)
        return verify_driver(destination, expected_digest=driver_digest)
    finally:
        if staging.exists():
            _plain(staging)
            if staging.resolve().parent != snapshots.resolve() or not staging.name.startswith(".incoming-"):
                raise JenkinsError("driver_cleanup_rejected", "Driver staging cleanup escaped its owner")
            shutil.rmtree(staging)


def driver_environment(snapshot: dict, python: str | Path, repository_root: str | Path) -> dict:
    verified = verify_driver(snapshot["root"], expected_digest=snapshot["driverDigest"])
    repo = Path(repository_root).absolute()
    executable = Path(python).absolute()
    _plain(repo)
    _plain(executable)
    if executable != repo / ".jenkins/runtime/python/python.exe" or not executable.is_file():
        raise JenkinsError("driver_python_rejected", "Driver requires the dedicated repository runtime")
    return {"ZIRCON_SEALED_DRIVER": verified["steps"], "ZIRCON_DRIVER_LAUNCHER": verified["launcher"],
            "ZIRCON_DRIVER_DIGEST": verified["driverDigest"], "JENKINS_PYTHON": str(executable),
            "ZIRCON_REPO_ROOT": str(repo), "ZIRCON_REPOSITORY_ID": digest(str(repo).casefold()),
            "PYTHONDONTWRITEBYTECODE": "1"}


def active_driver_binding(repository_root: str | Path) -> dict:
    """Bind a control request to the verified driver selected by deployment."""
    repo = Path(repository_root).absolute()
    record_path = repo / ".jenkins/state/deployment/driver.json"
    _plain(record_path)
    try:
        record = json.loads(record_path.read_text(encoding="utf-8"))
        driver_digest = record["driverDigest"]
        expected_root = repo / ".jenkins/runtime/drivers" / driver_digest
        if Path(record["root"]).absolute() != expected_root:
            raise ValueError("driver root")
        if not isinstance(record.get("runtimeOperationId"), str) or not record["runtimeOperationId"]:
            raise ValueError("driver generation")
        verified = verify_driver(expected_root, expected_digest=driver_digest)
    except (OSError, ValueError, KeyError, TypeError) as exc:
        raise JenkinsError("active_driver_unavailable", "The active driver is unavailable") from exc
    if (Path(__file__).absolute() != expected_root / "tools/jenkins/deployment/driver.py"
            or os.environ.get("ZIRCON_DRIVER_DIGEST") != driver_digest
            or Path(os.environ.get("ZIRCON_DRIVER_LAUNCHER", "")).absolute() != Path(verified["launcher"])
            or Path(os.environ.get("JENKINS_PYTHON", "")).absolute() != repo / ".jenkins/runtime/python/python.exe"):
        raise JenkinsError("driver_context_mismatch", "Control must use the selected sealed driver")
    operation_id = str(record["runtimeOperationId"])
    return {"digest": driver_digest, "generation": operation_id,
            "runtimeOperationId": operation_id, "root": str(expected_root), "launcher": verified["launcher"]}


def runtime_agent_name(base_name: str, runtime_operation_id: str) -> str:
    """Derive an isolated Jenkins node name for one runtime operation."""
    if not isinstance(base_name, str) or not base_name.strip() or any(ch not in "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789._-" for ch in base_name):
        raise JenkinsError("agent_name_invalid", "agent base name is required")
    if not isinstance(runtime_operation_id, str) or not runtime_operation_id.strip():
        raise JenkinsError("runtime_operation_invalid", "runtime operation id is required")
    if any(ch not in "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789._-" for ch in runtime_operation_id):
        raise JenkinsError("runtime_operation_invalid", "runtime operation id contains unsupported characters")
    suffix = hashlib.sha256(runtime_operation_id.encode("utf-8")).hexdigest()[:16]
    return f"{base_name.strip()}-{suffix}"


def runtime_agent_label(base_name: str, runtime_operation_id: str) -> str:
    """Pure label fence matching the operation-specific agent name."""
    return runtime_agent_name(base_name, runtime_operation_id)
