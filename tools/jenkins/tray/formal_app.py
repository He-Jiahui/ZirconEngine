"""Formal Jenkins tray UI, separate from the retired pilot profile."""
from __future__ import annotations

import os
import threading
import time
import json
import hashlib
from pathlib import Path

from . import native_windows as nw
from .formal import FormalTrayConfig, load_formal_config, start, status, stop
from .menu import COMMANDS, MenuState, menu_items, status_text

TRAY_MESSAGE = nw.WM_APP + 41


class FormalTray:
    def __init__(self, config: FormalTrayConfig):
        self.config = config; self.status = status(config); self.hwnd = 0; self.icon = None; self.closed = False
        self._lock = threading.Lock()
        self._operation_lock = threading.Lock()
        self.last_command = None

    def refresh(self):
        value = status(self.config)
        with self._lock: self.status = value
        self._telemetry()

    def _telemetry(self):
        path = self.config.paths.state / "tray" / "ui.json"; path.parent.mkdir(parents=True, exist_ok=True)
        op = self.status.get("operation") or {}
        own = None
        try: own = __import__('tools.jenkins.tray.formal_native', fromlist=['identity']).identity(os.getpid())
        except Exception: own = None
        value = {"schemaVersion": 1, "uiPid": os.getpid(), "uiBirth": (own or {}).get("birth") or (own or {}).get("creationTime"), "uiExecutable": (own or {}).get("executable", os.path.abspath(os.sys.executable)), "hwnd": self.hwnd, "state": self.status.get("state"), "url": self.config.url, "iconAdded": bool(getattr(self, "icon_added", False)), "ready": self.status.get("state") == "ready", "operation": op, "lastCommand": self.last_command}
        tmp = path.with_suffix('.tmp'); tmp.write_text(json.dumps(value, sort_keys=True), encoding='utf8'); tmp.replace(path)

    def open_logs(self):
        path = self.config.paths.logs
        path.mkdir(parents=True, exist_ok=True)
        if nw.IS_WINDOWS: os.startfile(str(path))

    def command(self, command: int):
        key = next((k for k, v in COMMANDS.items() if v == command), None)
        if key == "restart" and not MenuState(self.status).enabled("restart"): return
        if key in {"start", "stop"} and not MenuState(self.status).enabled(key): return
        self.last_command = {"commandId": command, "key": key, "at": time.time()}; self._telemetry()
        if key == "status": nw.message_box(self.hwnd, status_text(self.status), "Jenkins 状态", nw.MB_OK | nw.MB_ICONINFORMATION)
        elif key == "open": nw.open_url(self.config.url)
        elif key == "refresh": threading.Thread(target=self.refresh, daemon=True).start()
        elif key == "logs": self.open_logs()
        elif key == "start": threading.Thread(target=self._mutate, args=(start,), daemon=True).start()
        elif key == "stop":
            if self.status.get("state") in {"ready", "starting"}: threading.Thread(target=self._mutate, args=(stop,), daemon=True).start()
        elif key == "restart": threading.Thread(target=self._restart, daemon=True).start()
        elif key == "exit": self.closed = True
        elif key == "startup":
            from .formal_startup import set_current_user
            set_current_user(self.config.paths.repo, self.config.spec.path, pythonw=self.config.paths.root / "runtime" / "python" / "pythonw.exe")

    def _mutate(self, function):
        if not self._operation_lock.acquire(blocking=False): return
        try: self.status = function(self.config)
        except Exception as exc: self.status = self._operation_error(exc)
        finally: self._operation_lock.release()

    def _restart(self):
        if not self._operation_lock.acquire(blocking=False): return
        try: stop(self.config); self.status = start(self.config)
        except Exception as exc: self.status = self._operation_error(exc)
        finally: self._operation_lock.release()

    def _operation_error(self, exc):
        code = getattr(exc, "code", None)
        message = ("请重启 Windows，再启动 Jenkins 以核验恢复状态。"
                   if code == "deployment_termination_unproven" else str(exc))
        return {"state": "error", "message": message, "reasonCode": code, "url": self.config.url}

    def run(self) -> int:
        if not nw.IS_WINDOWS: return 0
        if not self._single_instance(): return 2
        def callback(hwnd, message, wparam, lparam):
            if message == TRAY_MESSAGE:
                if lparam in (nw.WM_RBUTTONUP, nw.WM_CONTEXTMENU):
                    x, y = nw.cursor_position(); selected = nw.popup_menu(hwnd, menu_items(MenuState(self.status)), x, y)
                    if selected: self.command(selected)
                elif lparam == nw.WM_LBUTTONDBLCLK: nw.open_url(self.config.url)
                return 0
            if message == nw.WM_COMMAND:
                self.command(int(wparam)); return 0
            if message == nw.WM_DESTROY:
                nw.user32.PostQuitMessage(0); return 0
            return nw.user32.DefWindowProcW(hwnd, message, wparam, lparam)
        self.hwnd, _ = nw.create_hidden_window("ZirconFormalJenkinsTray", "Jenkins", callback)
        self.icon = nw.status_icon(self.status.get("state", "stopped")); self.icon_added = bool(nw.add_icon(self.hwnd, 1, TRAY_MESSAGE, "Jenkins", self.icon)); self._telemetry()
        threading.Thread(target=self._poll, daemon=True).start()
        try:
          while not self.closed:
            with self._lock: current = dict(self.status)
            nw.update_icon(self.hwnd, 1, "Jenkins: " + current.get("state", "unknown"), self.icon)
            msg = nw.wintypes.MSG()
            if nw.user32.PeekMessageW(nw.ctypes.byref(msg), self.hwnd, 0, 0, 1):
                nw.user32.TranslateMessage(nw.ctypes.byref(msg)); nw.user32.DispatchMessageW(nw.ctypes.byref(msg))
            time.sleep(1)
        finally:
            nw.remove_icon(self.hwnd, 1); nw.user32.DestroyWindow(self.hwnd)
            if getattr(self, "_instance_handle", None):
                import ctypes
                ctypes.WinDLL("kernel32", use_last_error=True).CloseHandle(self._instance_handle)
        return 0

    def _single_instance(self) -> bool:
        if not nw.IS_WINDOWS: return True
        import ctypes
        kernel = ctypes.WinDLL("kernel32", use_last_error=True)
        kernel.CreateMutexW.argtypes = [ctypes.c_void_p, ctypes.c_bool, ctypes.c_wchar_p]; kernel.CreateMutexW.restype = ctypes.c_void_p
        kernel.CloseHandle.argtypes = [ctypes.c_void_p]; kernel.CloseHandle.restype = ctypes.c_bool
        suffix = hashlib.sha256(str(self.config.paths.repo).casefold().encode()).hexdigest()[:24]
        name = "Global\\ZirconFormalJenkinsTray-" + suffix
        handle = kernel.CreateMutexW(None, False, name)
        if not handle: return False
        if ctypes.get_last_error() == 183:
            kernel.CloseHandle(handle); return False
        self._instance_handle = handle
        return True

    def _poll(self):
        while not self.closed:
            try: self.refresh()
            except Exception: pass
            time.sleep(2)


def run_formal(config_path: str | Path) -> int:
    return FormalTray(load_formal_config(config_path)).run()
