from __future__ import annotations
import sys
from pathlib import Path

VALUE = "ZirconFormalJenkinsTray"

def command(repo: Path, spec: Path, pythonw: Path | None = None) -> str:
    executable = pythonw or Path(sys.executable).with_name("pythonw.exe")
    return f'"{executable}" "{repo / "tools" / "jenkins_tray" / "launch.py"}" --formal --config "{spec.resolve()}" tray'

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
