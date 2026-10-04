"""Explicit real Windows acceptance; messages alone never count as a passing check."""
from __future__ import annotations

import argparse
import ctypes
import json
import os
import subprocess
import time
from ctypes import wintypes
from pathlib import Path

from tools.jenkins.tray.config import load_config, validate_runtime
from tools.jenkins.tray.persistence import read_json, write_json
from tools.jenkins.tray import native_windows as nw
from tools.jenkins.pilot.native.process_identity import process_matches_creation_time
from tools.jenkins.tray.app import TRAY_MESSAGE
from tools.jenkins.tray import service

ENUMPROC = ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.HWND, wintypes.LPARAM)
nw.user32.EnumWindows.argtypes = [ENUMPROC, wintypes.LPARAM]
nw.user32.EnumChildWindows.argtypes = [wintypes.HWND, ENUMPROC, wintypes.LPARAM]
nw.user32.GetWindowThreadProcessId.argtypes = [wintypes.HWND, ctypes.POINTER(wintypes.DWORD)]
nw.user32.GetClassNameW.argtypes = [wintypes.HWND, wintypes.LPWSTR, ctypes.c_int]
nw.user32.GetWindowTextW.argtypes = [wintypes.HWND, wintypes.LPWSTR, ctypes.c_int]
nw.user32.GetDlgItem.argtypes = [wintypes.HWND, ctypes.c_int]
nw.user32.GetDlgItem.restype = wintypes.HWND
nw.user32.IsWindow.argtypes = [wintypes.HWND]
nw.user32.IsWindow.restype = wintypes.BOOL


def wait_until(observe, predicate, timeout=30):
    deadline = time.monotonic() + timeout
    while True:
        value = observe()
        if predicate(value):
            return value
        if time.monotonic() >= deadline:
            raise AssertionError("bounded native UI observation did not converge")
        time.sleep(0.1)


def class_name(hwnd):
    value = ctypes.create_unicode_buffer(256)
    nw.user32.GetClassNameW(hwnd, value, len(value))
    return value.value


def text(hwnd):
    value = ctypes.create_unicode_buffer(32768)
    nw.user32.GetWindowTextW(hwnd, value, len(value))
    return value.value


def owned_windows(pid, kind):
    result = []
    @ENUMPROC
    def visit(hwnd, _):
        owner = wintypes.DWORD()
        nw.user32.GetWindowThreadProcessId(hwnd, ctypes.byref(owner))
        if owner.value == pid and class_name(hwnd) == kind:
            result.append(int(hwnd))
        return True
    nw.user32.EnumWindows(visit, 0)
    return result


def dialog(pid, title, button=1, *, required_text=None, timeout=45):
    found = wait_until(lambda: [h for h in owned_windows(pid, "#32770") if text(h) == title], bool, timeout)
    hwnd = found[0]
    def body_text():
        contents = []
        @ENUMPROC
        def visit(child, _):
            if class_name(child) == "Static" and text(child):
                contents.append(text(child))
            return True
        nw.user32.EnumChildWindows(hwnd, visit, 0)
        return "\n".join(contents)
    # EnumWindows may observe the native dialog while Windows is still
    # constructing its children. Wait for actual text, not only its title.
    body = wait_until(body_text, lambda value: bool(value) and
                     (required_text is None or required_text in value), timeout)
    def target_button():
        direct = nw.user32.GetDlgItem(hwnd, button)
        if direct or button != 1:
            return direct
        # A native single-button informational box can expose its sole
        # closing button as IDCANCEL on this desktop. Never use this fallback
        # for the explicit Yes/No buttons of a mutation confirmation.
        buttons = []
        @ENUMPROC
        def visit(child, _):
            if class_name(child) == "Button":
                buttons.append(int(child))
            return True
        nw.user32.EnumChildWindows(hwnd, visit, 0)
        return buttons[0] if len(buttons) == 1 else None
    control = wait_until(target_button, bool, timeout)
    if not nw.user32.PostMessageW(control, 0x00F5, 0, 0):  # BM_CLICK
        raise ctypes.WinError(ctypes.get_last_error())
    wait_until(lambda: bool(nw.user32.IsWindow(hwnd)), lambda alive: not alive)
    return body


def post(hwnd, message, wparam=0, lparam=0):
    if not nw.user32.PostMessageW(hwnd, message, wparam, lparam):
        raise ctypes.WinError(ctypes.get_last_error())


def run(config_path: Path, *, exit_after=False):
    if os.name != "nt":
        raise RuntimeError("Windows-only acceptance")
    config = load_config(config_path)
    validate_runtime(config)
    telemetry = lambda: read_json(config.state_dir / "ui-runtime.json") or {}
    before = wait_until(telemetry, lambda v: v.get("status") in {"ready", "busy"} and not v.get("closed"), 120)
    pid, birth, hwnd = before["pid"], before["birth"], before["hwnd"]
    if not process_matches_creation_time(pid, birth):
        raise AssertionError("tray identity is not live")
    owner = wintypes.DWORD()
    nw.user32.GetWindowThreadProcessId(hwnd, ctypes.byref(owner))
    if owner.value != pid:
        raise AssertionError("window ownership differs from exact tray process")
    result = {"schemaVersion": 1, "pid": pid, "birth": birth, "hwnd": hwnd, "checks": {"identity": True}}
    duplicate = subprocess.run([str(config.pythonw_executable), "-B", str(config.launch_path), "--config",
                                str(config.config_path), "tray"], cwd=config.repo_root,
                               stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                               timeout=30, creationflags=subprocess.CREATE_NO_WINDOW)
    if duplicate.returncode != 0 or telemetry()["pid"] != pid or telemetry()["iconAddCount"] != before["iconAddCount"]:
        raise AssertionError("duplicate tray did not exit successfully without another icon")
    result["checks"]["singleInstance"] = True
    message = nw.user32.RegisterWindowMessageW("TaskbarCreated")
    post(hwnd, message)
    recreated = wait_until(telemetry, lambda v: v.get("iconAddCount") == before["iconAddCount"] + 1)
    result["checks"]["taskbarCreated"] = {"message": message, "iconAddCount": recreated["iconAddCount"]}
    post(hwnd, nw.WM_COMMAND, 1001)
    result["statusDialog"] = dialog(pid, "Jenkins 状态", required_text="活动构建")
    result["checks"]["statusMenu"] = True
    post(hwnd, TRAY_MESSAGE, 0, (1 << 16) | nw.WM_CONTEXTMENU)
    popup = wait_until(lambda: owned_windows(pid, "#32768"), bool)[0]
    result["checks"]["contextMenu"] = {"hwnd": popup, "class": "#32768"}
    post(hwnd, 0x001F)  # WM_CANCELMODE closes only this tray's native popup.
    wait_until(lambda: owned_windows(pid, "#32768"), lambda menus: not menus)
    count = telemetry().get("browserOpened", 0)
    post(hwnd, nw.WM_COMMAND, 1002)
    wait_until(telemetry, lambda v: v.get("browserOpened", 0) == count + 1)
    post(hwnd, TRAY_MESSAGE, 0, (1 << 16) | nw.WM_LBUTTONDBLCLK)
    wait_until(telemetry, lambda v: v.get("browserOpened", 0) == count + 2)
    result["checks"]["browserMenuAndDoubleClick"] = True
    for key, command in (("logs", 1005), ("refresh", 1006)):
        count = telemetry().get("menuCommands", {}).get(key, 0)
        post(hwnd, nw.WM_COMMAND, command)
        wait_until(telemetry, lambda v: v.get("menuCommands", {}).get(key, 0) == count + 1)
        result["checks"][key] = True
    if exit_after:
        runtime = service._runtime(config)[0]
        post(hwnd, nw.WM_COMMAND, 1008)
        wait_until(lambda: process_matches_creation_time(pid, birth), lambda alive: not alive)
        if not telemetry().get("closed"):
            raise AssertionError("tray did not persist closed state")
        for key in ("controller", "agent"):
            record = runtime[key]
            if not process_matches_creation_time(record["pid"], record["creationTime"]):
                raise AssertionError("exiting tray stopped an owned Jenkins runtime")
        result["checks"]["exitKeepsService"] = True
    result.update(completedAt=time.time(), rootIdentity=dict(config.root_identity))
    destination = config.pilot_root / "evidence" / f"windows-ui-{pid}-{int(time.time())}.json"
    write_json(destination, result)
    return {"evidence": str(destination), **result}


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--config", type=Path, required=True)
    parser.add_argument("--exit", action="store_true")
    arguments = parser.parse_args()
    print(json.dumps(run(arguments.config, exit_after=arguments.exit), ensure_ascii=True))
