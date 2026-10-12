"""Pinned user-desktop entry point for the formal Jenkins tray.

This file is deliberately a small launcher.  It does not start the Jenkins
service, create a deployment manager, install startup registration, or copy a
tray implementation.  A release record must pin this file and every source
and runtime file it imports before the launcher is allowed to import the
existing ``FormalTray`` implementation.

The launcher is intended to be run by the release's ``pythonw.exe`` from the
interactive user desktop.  A check-only invocation is useful for a release
preflight and performs no Windows desktop calls.  A run invocation first
proves that the current token is the same user and session as the real
Explorer taskbar, then binds the current thread to ``WinSta0\\Default`` before
importing ``tools.jenkins.tray.formal_app``.
"""
from __future__ import annotations

import argparse
import ctypes
from ctypes import wintypes
import hashlib
import json
import os
import re
import stat
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any

# A pythonw launch must never create a byte-code side effect next to the
# release source.  This assignment is also made before the test module imports
# this file for its static contract checks.
sys.dont_write_bytecode = True


REPO_ROOT = Path(__file__).resolve().parents[2]
LAUNCHER_RELATIVE = Path("tools/dev/jenkins_desktop_tray.py")
FORMAL_TRAY_ROOT = Path("tools/jenkins/tray")
FORMAL_BOOTSTRAP_RELATIVE = FORMAL_TRAY_ROOT / "launch.py"
DEPLOYMENT_STATE_RELATIVE = Path(".jenkins/state/deployment")
DRIVER_STATE_RELATIVE = DEPLOYMENT_STATE_RELATIVE / "driver.json"
DEPLOYMENT_OPERATION_RELATIVE = Path(".jenkins/state/deployment-operation.json")
DEPLOYMENT_HOST_RELATIVE = Path(".jenkins/state/deployment/host.json")
KEEPER_DESCRIPTOR_RELATIVE = Path(".jenkins/state/formal-lifecycle/recovery-keeper.json")
SPEC_RELATIVE = Path(".jenkins/deployment-spec.json")
WORKFLOW_SPEC_RELATIVE = Path(".jenkins/workflow-spec.json")
CONFIG_RELATIVE = Path(".codex/config.toml")
RUNTIME_ROOT_RELATIVE = Path(".jenkins/runtime")

# Every module in this directory is part of the formal tray source closure in
# the release record.  Keeping the list explicit protects against a release
# that happens to hash formal_app.py but omits a newly imported native helper.
FORMAL_TRAY_SOURCE_RELATIVES = frozenset(
    {
        FORMAL_TRAY_ROOT / "__init__.py",
        FORMAL_TRAY_ROOT / "__main__.py",
        FORMAL_TRAY_ROOT / "app.py",
        FORMAL_TRAY_ROOT / "config.py",
        FORMAL_TRAY_ROOT / "formal.py",
        FORMAL_TRAY_ROOT / "formal_app.py",
        FORMAL_TRAY_ROOT / "formal_native.py",
        FORMAL_TRAY_ROOT / "formal_startup.py",
        FORMAL_TRAY_ROOT / "launch.py",
        FORMAL_TRAY_ROOT / "lifecycle.py",
        FORMAL_TRAY_ROOT / "menu.py",
        FORMAL_TRAY_ROOT / "native_windows.py",
        FORMAL_TRAY_ROOT / "operations.py",
        FORMAL_TRAY_ROOT / "persistence.py",
        FORMAL_TRAY_ROOT / "service.py",
        FORMAL_TRAY_ROOT / "startup.py",
        FORMAL_TRAY_ROOT / "terminal.py",
        FORMAL_TRAY_ROOT / "win32_shell.py",
        FORMAL_TRAY_ROOT / "worker_environment.py",
    }
)
_SHA256 = re.compile(r"^[0-9a-f]{64}$")


class LauncherError(RuntimeError):
    """A bounded, non-secret failure suitable for a hidden pythonw process."""

    def __init__(self, code: str, message: str):
        super().__init__(message)
        self.code = code


@dataclass(frozen=True)
class ReleasePin:
    """The immutable paths selected by one release record."""

    repo: Path
    release_path: Path
    release_sha256: str
    spec_path: Path
    pythonw_path: Path
    python_path: Path
    driver_digest: str
    driver_root: Path
    driver_launcher: Path
    driver_manifest: Path
    driver_steps: Path


def _fail(code: str, message: str) -> None:
    raise LauncherError(code, message)


def _norm(path: Path) -> str:
    return os.path.normcase(str(path))


def _inside(path: Path, root: Path) -> bool:
    """Return whether *path* is physically beneath *root* after resolution."""

    try:
        return os.path.commonpath([_norm(path), _norm(root)]) == _norm(root)
    except ValueError:
        return False


def _no_reparse_components(path: Path) -> None:
    """Reject links/reparse points in a release path and its parents."""

    current = path
    components: list[Path] = []
    while True:
        components.append(current)
        if current.parent == current:
            break
        current = current.parent
    for component in reversed(components):
        try:
            info = component.lstat()
        except FileNotFoundError:
            continue
        if stat.S_ISLNK(info.st_mode) or getattr(info, "st_file_attributes", 0) & 0x400:
            _fail("reparse_point", f"release path crosses a link or reparse point: {component}")


def _canonical_file(value: object, *, label: str, root: Path | None = None) -> Path:
    if not isinstance(value, str) or not value or any(c in value for c in '\x00\r\n"'):
        _fail("invalid_path", f"{label} is not a plain path")
    candidate = Path(value)
    if not candidate.is_absolute():
        _fail("invalid_path", f"{label} must be absolute")
    try:
        resolved = candidate.resolve(strict=True)
    except OSError as exc:
        _fail("missing_file", f"{label} is unavailable: {exc}")
    _no_reparse_components(candidate)
    if _norm(candidate) != _norm(resolved):
        _fail("path_alias", f"{label} is not a canonical path")
    if root is not None and not _inside(resolved, root):
        _fail("path_scope", f"{label} is outside the release root")
    if not resolved.is_file():
        _fail("not_file", f"{label} is not a regular file")
    return resolved


def _canonical_dir(value: object, *, label: str, root: Path | None = None) -> Path:
    if not isinstance(value, str) or not value or any(c in value for c in '\x00\r\n"'):
        _fail("invalid_path", f"{label} is not a plain path")
    candidate = Path(value)
    if not candidate.is_absolute():
        _fail("invalid_path", f"{label} must be absolute")
    try:
        resolved = candidate.resolve(strict=True)
    except OSError as exc:
        _fail("missing_directory", f"{label} is unavailable: {exc}")
    _no_reparse_components(candidate)
    if _norm(candidate) != _norm(resolved):
        _fail("path_alias", f"{label} is not a canonical path")
    if root is not None and not _inside(resolved, root):
        _fail("path_scope", f"{label} is outside the release root")
    if not resolved.is_dir():
        _fail("not_directory", f"{label} is not a directory")
    return resolved


def _sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def _read_json(path: Path, *, label: str) -> dict[str, Any]:
    try:
        value = json.loads(path.read_bytes())
    except (OSError, UnicodeError, ValueError) as exc:
        _fail("invalid_json", f"{label} cannot be read: {exc}")
    if not isinstance(value, dict):
        _fail("invalid_json", f"{label} must contain an object")
    return value


def _expected_digest(value: object, *, label: str) -> str:
    if not isinstance(value, str) or not _SHA256.fullmatch(value):
        _fail("invalid_digest", f"{label} is not a SHA-256 digest")
    return value


def _verify_entries(
    entries: object,
    *,
    label: str,
    repo: Path,
    root: Path,
) -> dict[Path, str]:
    if not isinstance(entries, dict) or not entries:
        _fail("release_closure_missing", f"{label} is empty")
    verified: dict[Path, str] = {}
    for raw_path, expected in entries.items():
        digest = _expected_digest(expected, label=f"{label} digest")
        path = _canonical_file(raw_path, label=f"{label} path", root=root)
        # The runtime and operator maps must never silently point outside this
        # checkout.  The separate roots make the intended map domain explicit.
        if not _inside(path, repo):
            _fail("release_closure_scope", f"{label} path is outside the repository: {path}")
        actual = _sha256(path)
        if actual != digest:
            _fail("release_digest_mismatch", f"{label} changed: {path}")
        verified[path] = digest
    return verified


def _effective_operator_path(raw_path: str, *, repo: Path, actor_root: Path) -> Path:
    """Resolve one absolute release key to its effective sealed-source path."""

    path = Path(raw_path)
    if _inside(path, repo / "tools" / "jenkins"):
        try:
            relative = path.relative_to(repo)
        except ValueError:
            _fail("operator_path", "sealed actor path escaped the repository")
        return actor_root / relative
    return path


def _verify_operator_entries(entries: object, *, repo: Path, actor_root: Path) -> dict[str, tuple[Path, str]]:
    """Verify the successor's mixed operator map.

    The release inventory keeps absolute canonical paths for every operator.
    Entries beneath ``repo/tools/jenkins`` are resolved beneath the sealed
    actor root; every other entry (including this user-desktop wrapper) remains
    in the canonical checkout.  This prevents a repo checkout from silently
    replacing the actor's selected implementation.
    """

    if not isinstance(entries, dict) or not entries:
        _fail("release_closure_missing", "operator closure is empty")
    verified: dict[str, tuple[Path, str]] = {}
    for raw_path, expected in entries.items():
        digest = _expected_digest(expected, label="operator closure digest")
        if not isinstance(raw_path, str) or not raw_path:
            _fail("operator_path", "operator path is invalid")
        path = _canonical_file(raw_path, label="operator closure path", root=repo)
        if raw_path != str(path):
            _fail("operator_path", "operator path is not the canonical absolute release key")
        effective = _effective_operator_path(str(path), repo=repo, actor_root=actor_root)
        if _norm(effective) != _norm(path):
            effective = _canonical_file(str(effective), label="sealed actor operator", root=actor_root)
        actual = _sha256(effective)
        if actual != digest:
            _fail("release_digest_mismatch", f"operator closure changed: {raw_path}")
        verified[str(path)] = (effective, digest)
    return verified


def required_tray_source_relatives() -> frozenset[Path]:
    """Expose the closure contract for static tests and release tooling."""

    return frozenset(FORMAL_TRAY_SOURCE_RELATIVES)


def _verify_driver_manifest(pin: ReleasePin) -> None:
    manifest = _read_json(pin.driver_manifest, label="driver manifest")
    if manifest.get("schemaVersion") != 1 or manifest.get("driverDigest") != pin.driver_digest:
        _fail("driver_manifest_mismatch", "selected driver manifest does not match its digest")
    files = manifest.get("files")
    if not isinstance(files, dict) or not files:
        _fail("driver_manifest_missing", "selected driver manifest has no file closure")
    for relative, metadata in files.items():
        if not isinstance(relative, str) or not relative or not isinstance(metadata, dict):
            _fail("driver_manifest_invalid", "selected driver manifest contains an invalid file")
        rel = Path(relative.replace("/", os.sep))
        if rel.is_absolute() or any(part in {"", ".", ".."} for part in rel.parts):
            _fail("driver_manifest_path", f"driver manifest path is not relative: {relative}")
        target = _canonical_file(str(pin.driver_root / rel), label="driver manifest file", root=pin.driver_root)
        expected = _expected_digest(metadata.get("sha256"), label="driver manifest digest")
        if metadata.get("bytes") != target.stat().st_size or _sha256(target) != expected:
            _fail("driver_manifest_mismatch", f"selected driver file changed: {relative}")


def verify_release(release_path: Path | str, release_sha256: str, *, repo: Path = REPO_ROOT) -> ReleasePin:
    """Verify every immutable pin needed before importing the formal tray."""

    repo = _canonical_dir(str(repo), label="repository root")
    release = _canonical_file(os.fspath(release_path), label="release record", root=repo / DEPLOYMENT_STATE_RELATIVE)
    expected_release_digest = _expected_digest(release_sha256, label="release digest")
    if _sha256(release) != expected_release_digest:
        _fail("release_digest_mismatch", "release record bytes do not match the supplied pin")
    payload = _read_json(release, label="release record")
    kind = payload.get("kind")
    lost_marker = payload.get("lostRecoveryRuntime")
    owned_release = kind == "formal-owned-recovery-upgrade-release" and lost_marker is None
    lost_release = kind == "formal-lost-owner-recovery-release" and isinstance(lost_marker, dict)
    if payload.get("schemaVersion") != 1 or not (owned_release or lost_release):
        _fail("release_kind", "release record is not a reviewed sealed desktop recovery release")
    if lost_release:
        if (set(lost_marker) != {"plan", "originalCommand", "originalDescriptor"}
                or payload.get("failedOwnStart") is not None or payload.get("ownedUpgrade") is not None):
            _fail("release_kind", "lost recovery runtime requires its distinct reviewed marker")
        plan_ref = lost_marker["plan"]
        if not isinstance(plan_ref, dict) or set(plan_ref) != {"path", "sha256"}:
            _fail("release_kind", "lost recovery runtime plan reference is incomplete")
        plan_path = _canonical_file(plan_ref["path"], label="lost recovery runtime plan",
                                    root=repo / ".jenkins/state/migration-baseline")
        if _sha256(plan_path) != _expected_digest(plan_ref["sha256"], label="lost recovery runtime plan digest"):
            _fail("release_digest_mismatch", "lost recovery runtime plan changed")
        plan = _read_json(plan_path, label="lost recovery runtime plan")
        if (plan.get("recoveryVariant") not in {"lost-recovery-runtime", "departed-committed-recovery-runtime"}
                or plan.get("originalCommandStatus") !=
                    ("committed" if plan.get("recoveryVariant") == "departed-committed-recovery-runtime" else "unknown")
                or plan.get("requestId") != payload.get("requestId")
                or plan.get("scope") != payload.get("scope")
                or plan.get("owner") != payload.get("expectedPreviousOwner")
                or plan.get("lifecycleFence") != payload.get("expectedLifecycleFence")):
            _fail("release_kind", "lost recovery runtime differs from its reviewed plan")
        for name, relative in (("originalCommand", ".jenkins/state/recovery-lifecycle/command.json"),
                               ("originalDescriptor", ".jenkins/state/formal-lifecycle/recovery-keeper.json")):
            reference = lost_marker[name]
            if (not isinstance(reference, dict) or set(reference) != {"path", "sha256"}
                    or reference != plan.get(name) or reference["path"] != str(repo / relative)):
                _fail("release_kind", "lost recovery runtime original reference changed")
            _expected_digest(reference["sha256"], label="lost recovery runtime original digest")
    if payload.get("runtimeAcceptance") is not False or payload.get("cargoAcceptance") is not False:
        _fail("release_acceptance", "release must remain pending runtime and cargo acceptance")

    config_record = payload.get("config")
    spec_record = payload.get("deploymentSpec")
    workflow_record = payload.get("workflowSpec")
    if not isinstance(config_record, dict) or not isinstance(spec_record, dict) or not isinstance(workflow_record, dict):
        _fail("release_configuration", "release does not pin config and specifications")
    config_path = _canonical_file(config_record.get("path"), label="release config", root=repo)
    spec_path = _canonical_file(spec_record.get("path"), label="deployment spec", root=repo)
    workflow_path = _canonical_file(workflow_record.get("path"), label="workflow spec", root=repo)
    if (_norm(config_path) != _norm(repo / CONFIG_RELATIVE)
            or _norm(spec_path) != _norm(repo / SPEC_RELATIVE)
            or _norm(workflow_path) != _norm(repo / WORKFLOW_SPEC_RELATIVE)):
        _fail("release_configuration", "release config/spec path is not canonical")
    if _sha256(config_path) != _expected_digest(config_record.get("sha256"), label="release config digest"):
        _fail("release_digest_mismatch", "config.toml changed")
    if _sha256(spec_path) != _expected_digest(spec_record.get("sha256"), label="deployment spec digest"):
        _fail("release_digest_mismatch", "deployment-spec.json changed")
    if _sha256(workflow_path) != _expected_digest(workflow_record.get("sha256"), label="workflow spec digest"):
        _fail("release_digest_mismatch", "workflow-spec.json changed")
    spec = _read_json(spec_path, label="deployment spec")
    if _norm(Path(str(spec.get("repositoryRoot", "")).strip())) != _norm(repo):
        _fail("deployment_spec_scope", "deployment spec points at a different repository")

    target = payload.get("targetDriver")
    if not isinstance(target, dict):
        _fail("driver_missing", "release does not select a driver")
    digest = target.get("driverDigest")
    if not isinstance(digest, str) or not re.fullmatch(r"[0-9a-f]{64}", digest):
        _fail("driver_missing", "selected driver digest is invalid")
    driver_root = _canonical_dir(target.get("root"), label="selected driver root", root=repo / RUNTIME_ROOT_RELATIVE)
    expected_root = (repo / RUNTIME_ROOT_RELATIVE / "drivers" / digest).resolve(strict=True)
    if target.get("root") != str(expected_root) or _norm(driver_root) != _norm(expected_root):
        _fail("driver_scope", "selected driver root is not content addressed")
    actor_code_root = target.get("root")
    if payload.get("actorCodeRoot") != actor_code_root:
        _fail("actor_scope", "actorCodeRoot must equal the selected target root")

    operator_entries = payload.get("operatorFiles")
    runtime_entries = payload.get("runtimeFiles")
    operator = _verify_operator_entries(operator_entries, repo=repo, actor_root=driver_root)
    runtime = _verify_entries(runtime_entries, label="runtime closure", repo=repo, root=repo / RUNTIME_ROOT_RELATIVE)
    expected_tray = {Path(*[part.casefold() for part in value.parts]) for value in FORMAL_TRAY_SOURCE_RELATIVES}
    actor_tray = {
        Path(*[part.casefold() for part in Path(raw).relative_to(repo).parts])
        for raw in operator
        if _inside(Path(raw), repo / "tools" / "jenkins" / "tray")
    }
    missing_tray = sorted(expected_tray - actor_tray)
    if missing_tray:
        _fail("tray_closure_missing", f"release omits formal tray source: {missing_tray[0]}")
    launcher_key = str((repo / LAUNCHER_RELATIVE).resolve(strict=True))
    launcher_key = next((raw for raw in operator if _norm(Path(raw)) == _norm(Path(launcher_key))), None)
    if launcher_key is None:
        _fail("launcher_unpinned", "release does not pin tools/dev/jenkins_desktop_tray.py")

    expected_pythonw = (repo / RUNTIME_ROOT_RELATIVE / "python" / "pythonw.exe").resolve(strict=True)
    expected_python = (repo / RUNTIME_ROOT_RELATIVE / "python" / "python.exe").resolve(strict=True)
    if _norm(expected_pythonw) not in {_norm(path) for path in runtime}:
        _fail("pythonw_unpinned", "release runtime closure omits pythonw.exe")
    if _norm(expected_python) not in {_norm(path) for path in runtime}:
        _fail("python_unpinned", "release runtime closure omits python.exe")
    pythonw_path = next(path for path in runtime if _norm(path) == _norm(expected_pythonw))
    python_path = next(path for path in runtime if _norm(path) == _norm(expected_python))
    driver_launcher = _canonical_file(target.get("launcher"), label="selected driver launcher", root=driver_root)
    driver_manifest = _canonical_file(target.get("manifest"), label="selected driver manifest", root=driver_root)
    driver_steps = _canonical_file(target.get("steps"), label="selected driver steps", root=driver_root)
    if driver_launcher.name.casefold() != "launcher.py" or driver_manifest.name.casefold() != "driver-manifest.json":
        _fail("driver_scope", "selected driver entry names are invalid")
    if driver_steps.relative_to(driver_root).as_posix().casefold() != "pipeline/steps.groovy":
        _fail("driver_scope", "selected driver steps are not pipeline/steps.groovy")
    pin = ReleasePin(repo, release, expected_release_digest, spec_path, pythonw_path, python_path,
                     digest, driver_root, driver_launcher, driver_manifest, driver_steps)
    _verify_driver_manifest(pin)

    driver_state_path = _canonical_file(str(repo / DRIVER_STATE_RELATIVE), label="selected driver state", root=repo)
    driver_state = _read_json(driver_state_path, label="selected driver state")
    if driver_state.get("runtimeAcceptance") is not False or driver_state.get("cargoAcceptance") is not False:
        _fail("driver_selection_mismatch", "selected driver has an unexpected acceptance flag")
    if driver_state.get("driverDigest") != digest or _norm(Path(str(driver_state.get("root", "")))) != _norm(driver_root):
        _fail("driver_selection_mismatch", "driver.json does not select the release driver")
    if (_norm(Path(str(driver_state.get("launcher", "")))) != _norm(driver_launcher)
            or _norm(Path(str(driver_state.get("manifest", "")))) != _norm(driver_manifest)
            or _norm(Path(str(driver_state.get("steps", "")))) != _norm(driver_steps)):
        _fail("driver_selection_mismatch", "driver.json entry points differ from the release")
    recovery = driver_state.get("recoveryRelease")
    if not isinstance(recovery, dict) or _norm(Path(str(recovery.get("path", "")))) != _norm(release) or recovery.get("sha256") != expected_release_digest:
        _fail("driver_selection_mismatch", "driver.json recovery release pin differs")
    runtime_id = driver_state.get("runtimeOperationId")
    if not isinstance(runtime_id, str) or not runtime_id:
        _fail("driver_selection_mismatch", "selected driver has no runtime operation id")
    operation_path = _canonical_file(str(repo / DEPLOYMENT_OPERATION_RELATIVE), label="deployment operation", root=repo)
    host_path = _canonical_file(str(repo / DEPLOYMENT_HOST_RELATIVE), label="deployment host", root=repo)
    operation = _read_json(operation_path, label="deployment operation")
    host = _read_json(host_path, label="deployment host")
    if not (operation.get("operationId") == operation.get("generation") == runtime_id):
        _fail("runtime_selection_mismatch", "deployment-operation does not match selected runtime")
    if not (host.get("operationId") == host.get("generation") == runtime_id):
        _fail("runtime_selection_mismatch", "deployment host does not match selected runtime")
    keeper_path = _canonical_file(str(repo / KEEPER_DESCRIPTOR_RELATIVE), label="formal keeper descriptor", root=repo)
    keeper = _read_json(keeper_path, label="recovery keeper descriptor")
    runtime = keeper.get("runtime")
    if not isinstance(runtime, dict):
        _fail("runtime_selection_mismatch", "recovery keeper descriptor has no runtime")
    selected = runtime.get("selected")
    if not isinstance(selected, dict):
        _fail("runtime_selection_mismatch", "recovery keeper descriptor has no selected target")
    for key in ("driverDigest", "root", "launcher", "manifest", "steps", "runtimeOperationId"):
        if selected.get(key) != driver_state.get(key):
            _fail("runtime_selection_mismatch", f"recovery keeper selected {key} differs")
    descriptor_release = keeper.get("release")
    if descriptor_release != recovery:
        _fail("runtime_selection_mismatch", "recovery keeper release differs from selected driver")
    descriptor_operation = runtime.get("operation")
    descriptor_host = runtime.get("host")
    if descriptor_operation != operation or descriptor_host != host:
        _fail("runtime_selection_mismatch", "recovery keeper runtime differs from deployment records")
    keeper_identity = keeper.get("keeper")
    if not isinstance(keeper_identity, dict) or not all(keeper_identity.get(key) for key in ("pid", "creationTime", "executable", "commandDigest")):
        _fail("runtime_selection_mismatch", "recovery keeper identity is incomplete")
    return pin


# Windows identity and desktop checks are kept behind this boundary so a
# check-only release audit remains usable on non-Windows hosts.
def _win32_error(operation: str) -> LauncherError:
    return LauncherError("win32_error", f"{operation} failed with Win32 error {ctypes.get_last_error()}")


def _token_identity(token: Any) -> tuple[str, bool]:
    if sys.platform != "win32":
        _fail("windows_only", "the user-desktop launcher requires Windows")
    advapi = ctypes.WinDLL("advapi32", use_last_error=True)
    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    TOKEN_QUERY = 0x0008
    TokenUser = 1
    advapi.GetTokenInformation.argtypes = [wintypes.HANDLE, wintypes.DWORD, wintypes.LPVOID, wintypes.DWORD, ctypes.POINTER(wintypes.DWORD)]
    advapi.GetTokenInformation.restype = wintypes.BOOL
    advapi.ConvertSidToStringSidW.argtypes = [wintypes.LPVOID, ctypes.POINTER(wintypes.LPWSTR)]
    advapi.ConvertSidToStringSidW.restype = wintypes.BOOL
    advapi.IsTokenRestricted.argtypes = [wintypes.HANDLE]
    advapi.IsTokenRestricted.restype = wintypes.BOOL
    kernel.LocalFree.argtypes = [wintypes.HLOCAL]
    kernel.LocalFree.restype = wintypes.HLOCAL
    size = wintypes.DWORD()
    advapi.GetTokenInformation(token, TokenUser, None, 0, ctypes.byref(size))
    if not size.value:
        raise _win32_error("GetTokenInformation(TokenUser)")
    buffer = ctypes.create_string_buffer(size.value)
    if not advapi.GetTokenInformation(token, TokenUser, buffer, size, ctypes.byref(size)):
        raise _win32_error("GetTokenInformation(TokenUser)")

    class _SidAndAttributes(ctypes.Structure):
        _fields_ = [("sid", ctypes.c_void_p), ("attributes", wintypes.DWORD)]

    sid = ctypes.cast(buffer, ctypes.POINTER(_SidAndAttributes)).contents.sid
    text = ctypes.c_wchar_p()
    if not advapi.ConvertSidToStringSidW(sid, ctypes.byref(text)):
        raise _win32_error("ConvertSidToStringSidW")
    try:
        sid_text = text.value or ""
    finally:
        kernel.LocalFree(ctypes.cast(text, wintypes.HLOCAL))
    # TokenHasRestrictions reports that a token has ever been filtered,
    # including a normal UAC desktop token. IsTokenRestricted checks the
    # restricting SID list that distinguishes the private sandbox token.
    ctypes.set_last_error(0)
    restricted = bool(advapi.IsTokenRestricted(token))
    if not restricted and ctypes.get_last_error():
        raise _win32_error("IsTokenRestricted")
    return sid_text, restricted


def _process_identity(pid: int) -> tuple[str, int, bool, str]:
    if sys.platform != "win32":
        _fail("windows_only", "the user-desktop launcher requires Windows")
    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    advapi = ctypes.WinDLL("advapi32", use_last_error=True)
    PROCESS_QUERY_LIMITED_INFORMATION = 0x1000
    TOKEN_QUERY = 0x0008
    kernel.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
    kernel.OpenProcess.restype = wintypes.HANDLE
    kernel.CloseHandle.argtypes = [wintypes.HANDLE]
    kernel.CloseHandle.restype = wintypes.BOOL
    kernel.ProcessIdToSessionId.argtypes = [wintypes.DWORD, ctypes.POINTER(wintypes.DWORD)]
    kernel.ProcessIdToSessionId.restype = wintypes.BOOL
    kernel.QueryFullProcessImageNameW.argtypes = [wintypes.HANDLE, wintypes.DWORD, wintypes.LPWSTR, ctypes.POINTER(wintypes.DWORD)]
    kernel.QueryFullProcessImageNameW.restype = wintypes.BOOL
    advapi.OpenProcessToken.argtypes = [wintypes.HANDLE, wintypes.DWORD, ctypes.POINTER(wintypes.HANDLE)]
    advapi.OpenProcessToken.restype = wintypes.BOOL
    process = kernel.OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, False, int(pid))
    if not process:
        raise _win32_error("OpenProcess")
    token = wintypes.HANDLE()
    try:
        if not advapi.OpenProcessToken(process, TOKEN_QUERY, ctypes.byref(token)):
            raise _win32_error("OpenProcessToken")
        try:
            sid, restricted = _token_identity(token)
        finally:
            kernel.CloseHandle(token)
        session = wintypes.DWORD()
        if not kernel.ProcessIdToSessionId(int(pid), ctypes.byref(session)):
            raise _win32_error("ProcessIdToSessionId")
        size = wintypes.DWORD(32768)
        path_buffer = ctypes.create_unicode_buffer(size.value)
        if not kernel.QueryFullProcessImageNameW(process, 0, path_buffer, ctypes.byref(size)):
            raise _win32_error("QueryFullProcessImageNameW")
        return sid, int(session.value), restricted, path_buffer.value
    finally:
        kernel.CloseHandle(process)


def _object_name(handle: Any) -> str:
    user32 = ctypes.WinDLL("user32", use_last_error=True)
    UOI_NAME = 2
    user32.GetUserObjectInformationW.argtypes = [wintypes.HANDLE, wintypes.DWORD, wintypes.LPVOID, wintypes.DWORD, ctypes.POINTER(wintypes.DWORD)]
    user32.GetUserObjectInformationW.restype = wintypes.BOOL
    size = wintypes.DWORD()
    user32.GetUserObjectInformationW(handle, UOI_NAME, None, 0, ctypes.byref(size))
    if not size.value:
        raise _win32_error("GetUserObjectInformationW(size)")
    buffer = ctypes.create_unicode_buffer(size.value // ctypes.sizeof(ctypes.c_wchar) + 1)
    if not user32.GetUserObjectInformationW(handle, UOI_NAME, buffer, ctypes.sizeof(buffer), ctypes.byref(size)):
        raise _win32_error("GetUserObjectInformationW")
    return buffer.value


def _verify_user_desktop() -> None:
    """Prove the current process can create UI beside the real Explorer tray."""

    if sys.platform != "win32":
        _fail("windows_only", "the user-desktop launcher requires Windows")
    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    user32 = ctypes.WinDLL("user32", use_last_error=True)
    user32.GetWindowThreadProcessId.argtypes = [wintypes.HWND, ctypes.POINTER(wintypes.DWORD)]
    user32.GetWindowThreadProcessId.restype = wintypes.DWORD
    user32.GetProcessWindowStation.argtypes = []
    user32.GetProcessWindowStation.restype = wintypes.HWINSTA
    pid = os.getpid()
    current_sid, current_session, restricted, _ = _process_identity(pid)
    if restricted:
        _fail("restricted_token", "the tray launcher cannot run from a restricted token")

    user32.FindWindowW.argtypes = [ctypes.c_wchar_p, ctypes.c_wchar_p]
    user32.FindWindowW.restype = wintypes.HWND
    shell = user32.FindWindowW("Shell_TrayWnd", None)
    if not shell:
        raise _win32_error("FindWindowW(Shell_TrayWnd)")
    explorer_pid = wintypes.DWORD()
    thread_id = user32.GetWindowThreadProcessId(shell, ctypes.byref(explorer_pid))
    if not thread_id or not explorer_pid.value:
        raise _win32_error("GetWindowThreadProcessId(Shell_TrayWnd)")
    explorer_sid, explorer_session, explorer_restricted, explorer_path = _process_identity(int(explorer_pid.value))
    if Path(explorer_path).name.casefold() != "explorer.exe":
        _fail("explorer_identity", "Shell_TrayWnd is not owned by Explorer.exe")
    if explorer_session != current_session:
        _fail("session_mismatch", "launcher and Explorer are in different sessions")
    if explorer_sid.casefold() != current_sid.casefold():
        _fail("sid_mismatch", "launcher token SID differs from the Explorer SID")
    if explorer_restricted:
        _fail("explorer_token", "Explorer is running with an unexpected restricted token")

    user32.GetThreadDesktop.argtypes = [wintypes.DWORD]
    user32.GetThreadDesktop.restype = wintypes.HANDLE
    shell_desktop = user32.GetThreadDesktop(thread_id)
    if not shell_desktop or _object_name(shell_desktop).casefold() != "default":
        _fail("explorer_desktop", "Explorer taskbar is not on the Default desktop")
    process_station = user32.GetProcessWindowStation()
    if not process_station or _object_name(process_station).casefold() != "winsta0":
        _fail("window_station", "launcher is not in WinSta0")

    DESKTOP_READOBJECTS = 0x0001
    DESKTOP_CREATEWINDOW = 0x0002
    DESKTOP_CREATEMENU = 0x0004
    DESKTOP_ENUMERATE = 0x0040
    DESKTOP_WRITEOBJECTS = 0x0080
    access = (DESKTOP_READOBJECTS | DESKTOP_CREATEWINDOW | DESKTOP_CREATEMENU |
              DESKTOP_ENUMERATE | DESKTOP_WRITEOBJECTS)
    user32.OpenInputDesktop.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
    user32.OpenInputDesktop.restype = wintypes.HANDLE
    user32.CloseDesktop.argtypes = [wintypes.HANDLE]
    user32.CloseDesktop.restype = wintypes.BOOL
    input_desktop = user32.OpenInputDesktop(0, False, DESKTOP_READOBJECTS)
    if not input_desktop:
        raise _win32_error("OpenInputDesktop")
    try:
        if _object_name(input_desktop).casefold() != "default":
            _fail("input_desktop", "the interactive input desktop is not Default")
    finally:
        user32.CloseDesktop(input_desktop)

    user32.OpenDesktopW.argtypes = [ctypes.c_wchar_p, wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
    user32.OpenDesktopW.restype = wintypes.HANDLE
    user32.SetThreadDesktop.argtypes = [wintypes.HANDLE]
    user32.SetThreadDesktop.restype = wintypes.BOOL
    desktop = user32.OpenDesktopW("Default", 0, False, access)
    if not desktop:
        raise _win32_error("OpenDesktopW(Default)")
    # Keep this handle live for the entire FormalTray process.  It also avoids
    # accidentally returning to the private sandbox desktop before run_formal
    # creates its hidden window.
    if not user32.SetThreadDesktop(desktop):
        user32.CloseDesktop(desktop)
        raise _win32_error("SetThreadDesktop(Default)")
    current_desktop = user32.GetThreadDesktop(kernel.GetCurrentThreadId())
    if not current_desktop or _object_name(current_desktop).casefold() != "default":
        user32.CloseDesktop(desktop)
        _fail("desktop_binding", "launcher thread did not bind to Default")
    # Deliberately retain the handle in a module global.  FormalTray owns no
    # desktop handle of its own, and this process exits when the UI exits.
    globals()["_DESKTOP_HANDLE"] = desktop


def _current_pythonw(pin: ReleasePin) -> None:
    try:
        executable = Path(sys.executable).resolve(strict=True)
    except OSError as exc:
        _fail("pythonw_missing", f"current interpreter is unavailable: {exc}")
    if executable.name.casefold() != "pythonw.exe" or _norm(executable) != _norm(pin.pythonw_path):
        _fail("pythonw_mismatch", "launcher must run from the release-pinned pythonw.exe")


def _parse_args(argv: list[str] | None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--release", type=Path, required=True, help="immutable formal release record")
    parser.add_argument("--release-sha256", required=True, help="SHA-256 pin for --release")
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--run", action="store_true", help="bind WinSta0\\Default and run the pinned FormalTray")
    mode.add_argument("--check-only", action="store_true", help="verify release pins without starting UI")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    try:
        args = _parse_args(argv)
        pin = verify_release(args.release, args.release_sha256)
        if not args.run:
            return 0
        _current_pythonw(pin)
        _verify_user_desktop()
        # Import only after every immutable and interactive-desktop preflight
        # has passed.  FormalTray's start/stop/restart menu actions dispatch
        # through the public keeper-control path in the pinned implementation.
        # The tray classes must come from the selected actor's sealed source
        # root.  Importing from the checkout would mix a reviewed release with
        # mutable source files and could bypass the actorCodeRoot binding.
        sys.path.insert(0, str(pin.driver_root))
        from tools.jenkins.tray.formal_app import run_formal

        return int(run_formal(pin.spec_path))
    except LauncherError as exc:
        stream = getattr(sys, "stderr", None)
        if stream is not None:
            print(f"jenkins_desktop_tray: {exc.code}: {exc}", file=stream)
        return 2
    except Exception as exc:  # pragma: no cover - final hidden-entry guard
        stream = getattr(sys, "stderr", None)
        if stream is not None:
            print(f"jenkins_desktop_tray: launch_failed: {exc}", file=stream)
        return 3


if __name__ == "__main__":
    raise SystemExit(main())
