from __future__ import annotations
import hashlib
import json
import sys
from pathlib import Path

VALUE = "ZirconFormalJenkinsTray"

_DEPLOYMENT_STATE = Path(".jenkins") / "state" / "deployment"
_DRIVER_STATE = _DEPLOYMENT_STATE / "driver.json"
_DESKTOP_LAUNCHER = Path("tools") / "dev" / "jenkins_desktop_tray.py"
_SUCCESSOR_KINDS = {"formal-owned-recovery-upgrade-release", "formal-lost-owner-recovery-release"}


def _selected_successor(repo: Path) -> tuple[Path, str] | None:
    """Return the exact selected owned-release reference, if one is live.

    Startup registration is explicit, but it must not silently launch a
    mutable checkout when the selected runtime has an owned successor.  A
    missing driver record means this repository is still on the compatibility
    path; a malformed or mismatched selected record is rejected instead of
    falling back to an unpinned command.
    """
    repo = repo.resolve()
    driver_path = repo / _DRIVER_STATE
    if not driver_path.is_file():
        return None
    try:
        driver = json.loads(driver_path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, ValueError) as exc:
        raise RuntimeError("selected driver state is unreadable") from exc
    reference = driver.get("recoveryRelease") if isinstance(driver, dict) else None
    if reference is None:
        return None
    if not isinstance(reference, dict) or not isinstance(reference.get("path"), str) or not isinstance(reference.get("sha256"), str):
        raise RuntimeError("selected successor release reference is invalid")
    release = Path(reference["path"])
    if not release.is_absolute():
        raise RuntimeError("selected successor release path must be absolute")
    release = release.resolve(strict=True)
    deployment_root = (repo / _DEPLOYMENT_STATE).resolve()
    if release != Path(reference["path"]) or deployment_root not in release.parents:
        raise RuntimeError("selected successor release path is outside deployment state")
    release_bytes = release.read_bytes()
    digest = hashlib.sha256(release_bytes).hexdigest()
    if digest != reference["sha256"]:
        raise RuntimeError("selected successor release digest changed")
    try:
        payload = json.loads(release_bytes)
    except (UnicodeError, ValueError) as exc:
        raise RuntimeError("selected successor release is invalid") from exc
    if not isinstance(payload, dict) or payload.get("kind") not in _SUCCESSOR_KINDS:
        raise RuntimeError("selected release is not a reviewed recovery successor")
    target = payload.get("targetDriver")
    if not isinstance(target, dict) or driver.get("driverDigest") != target.get("driverDigest"):
        raise RuntimeError("selected driver and successor release differ")
    return release, digest

def command(repo: Path, spec: Path, pythonw: Path | None = None) -> str:
    repo = repo.resolve()
    executable = (pythonw or Path(sys.executable).with_name("pythonw.exe")).resolve()
    successor = _selected_successor(repo)
    if successor is not None:
        release, digest = successor
        launcher = repo / _DESKTOP_LAUNCHER
        return f'"{executable}" -B "{launcher}" --release "{release}" --release-sha256 {digest} --run'
    # Compatibility is retained only when no successor has been selected;
    # launch.py is the canonical formal tray bootstrap (the old
    # tools/jenkins_tray path is retired).
    launcher = repo / "tools" / "jenkins" / "tray" / "launch.py"
    return f'"{executable}" -B "{launcher}" --formal --config "{spec.resolve()}" tray'

def set_current_user(repo: Path, spec: Path, *, pythonw: Path | None = None) -> dict:
    import winreg
    value = command(repo.resolve(), spec.resolve(), pythonw)
    key = winreg.CreateKey(winreg.HKEY_CURRENT_USER, r"Software\Microsoft\Windows\CurrentVersion\Run")
    with key: winreg.SetValueEx(key, VALUE, 0, winreg.REG_SZ, value)
    return {"installed": True, "valueName": VALUE, "command": value}

def remove_current_user() -> dict:
    import winreg
    try:
        key = winreg.OpenKey(winreg.HKEY_CURRENT_USER, r"Software\Microsoft\Windows\CurrentVersion\Run", 0, winreg.KEY_SET_VALUE)
        with key: winreg.DeleteValue(key, VALUE)
    except FileNotFoundError: pass
    return {"installed": False, "valueName": VALUE}

def query_current_user() -> dict:
    import winreg
    try:
        key = winreg.OpenKey(winreg.HKEY_CURRENT_USER, r"Software\Microsoft\Windows\CurrentVersion\Run")
        with key: value, _ = winreg.QueryValueEx(key, VALUE)
        return {"installed": True, "valueName": VALUE, "command": value}
    except (FileNotFoundError, OSError):
        return {"installed": False, "valueName": VALUE}
