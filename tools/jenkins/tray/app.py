"""Win32 UI owns presentation; background workers own all blocking operations."""
from __future__ import annotations

import os
import queue
import threading
import time

from . import native_windows as nw
from .config import TrayError, validate_runtime
from .menu import COMMANDS, MenuState, menu_items, status_text, label_for
from .persistence import safe_error, write_json
from .win32_shell import SingleInstance, identity_key

TRAY_MESSAGE = nw.WM_APP + 73
REFRESH_MESSAGE = nw.WM_APP + 74


class TrayApp:
    def __init__(self, config):
        self.config = config
        self.status = {"state": "starting", "message": "正在查询 Jenkins", "canStart": False, "canStop": False}
        self.operation = False
        self.closed = False
        self.hwnd = 0
        self.instance = SingleInstance(identity_key(config))
        self.events = queue.Queue()
        self.poll_lock = threading.Lock()
        self.finished = threading.Event()
        self.icon = None
        self.callback = None
        self.taskbar_created = 0
        from tools.jenkins.pilot.native.process_identity import process_creation_time
        self.telemetry = {"schemaVersion": 1, "pid": os.getpid(), "birth": process_creation_time(os.getpid()),
                          "hwnd": 0, "iconAddCount": 0, "menuCommands": {}, "browserOpened": 0,
                          "status": "starting", "closed": False, "lastObservationAt": None}

    def _telemetry(self):
        self.telemetry.update(status=self.status.get("state", "error"), closed=self.closed,
                              menu=[{"command": c, "label": label, "enabled": enabled}
                                    for c, label, enabled in menu_items(MenuState(self.status, self.operation))])
        write_json(self.config.state_dir / "ui-runtime.json", self.telemetry)

    def run(self) -> int:
        validate_runtime(self.config)
        if not self.instance.acquire():
            return 0
        try:
            self.taskbar_created = nw.user32.RegisterWindowMessageW("TaskbarCreated")
            self.hwnd, self.callback = nw.create_hidden_window("ZirconJenkinsTray", "Jenkins 托盘", self._window_proc)
            self.icon = nw.status_icon("starting")
            if not nw.add_icon(self.hwnd, 1, TRAY_MESSAGE, "Jenkins：查询中", self.icon):
                raise TrayError("Windows 未能注册 Jenkins 托盘图标")
            self.telemetry.update(hwnd=int(self.hwnd), iconAddCount=1)
            self._telemetry()
            threading.Thread(target=self._poll_loop, daemon=True, name="jenkins-tray-poll").start()
            nw.run_message_loop()
            return 0
        finally:
            self.closed = True
            self.finished.set()
            if self.hwnd:
                nw.remove_icon(self.hwnd, 1)
            if self.icon:
                nw.user32.DestroyIcon(self.icon)
            self.instance.close()
            self._telemetry()

    def _window_proc(self, hwnd, message, wparam, lparam):
        try:
            if message == self.taskbar_created and self.taskbar_created:
                nw.remove_icon(hwnd, 1)
                if nw.add_icon(hwnd, 1, TRAY_MESSAGE, "Jenkins：" + label_for(self.status), self.icon):
                    self.telemetry["iconAddCount"] += 1
                    self._telemetry()
                return 0
            if message == nw.WM_DESTROY:
                nw.remove_icon(hwnd, 1)
                nw.user32.PostQuitMessage(0)
                return 0
            if message == nw.WM_COMMAND:
                self._command(int(wparam) & 0xffff)
                return 0
            if message == TRAY_MESSAGE:
                event = int(lparam) & 0xffff
                if event == nw.WM_LBUTTONDBLCLK:
                    self._open()
                elif event in (nw.WM_RBUTTONUP, nw.WM_CONTEXTMENU):
                    x, y = nw.cursor_position()
                    selected = nw.popup_menu(hwnd, menu_items(MenuState(self.status, self.operation)), x, y)
                    if selected:
                        self._command(selected)
                return 0
            if message == REFRESH_MESSAGE:
                self._consume_events()
                return 0
        except Exception as error:
            self.status = {"state": "error", "message": safe_error(error), "canStart": False, "canStop": False}
            self._paint()
            return 0
        return nw.user32.DefWindowProcW(hwnd, message, wparam, lparam)

    def _post(self, kind, value):
        self.events.put((kind, value))
        if self.hwnd and not self.closed:
            nw.user32.PostMessageW(self.hwnd, REFRESH_MESSAGE, 0, 0)

    def _work(self, kind, function):
        def worker():
            try:
                self._post(kind, function())
            except Exception as error:
                self._post("error", {"kind": kind, "message": safe_error(error)})
        threading.Thread(target=worker, daemon=True, name="jenkins-tray-" + kind).start()

    def _poll_loop(self):
        while not self.finished.is_set():
            tick = time.monotonic()
            self._refresh()
            self.finished.wait(max(0.1, 5 - (time.monotonic() - tick)))

    def _refresh(self):
        if not self.poll_lock.acquire(blocking=False):
            return
        try:
            from .operations import run_worker, read_operation
            pending = read_operation(self.config)
            if pending and pending.get("state") in {"pending", "reserved", "active"} and not self.operation:
                from tools.jenkins.pilot.native.process_identity import process_matches_creation_time
                if not process_matches_creation_time(pending["workerPid"], pending["workerCreationTime"]):
                    result = run_worker(self.config, "reconcile")
                    if isinstance(result.get("status"), dict):
                        self._post("status", result["status"])
            self._post("status", run_worker(self.config, "status"))
        except Exception as error:
            self._post("poll-error", safe_error(error))
        finally:
            self.poll_lock.release()

    def _consume_events(self):
        while True:
            try:
                kind, value = self.events.get_nowait()
            except queue.Empty:
                break
            if kind == "status":
                if not self.operation:
                    self.status = value
                self.telemetry["lastObservationAt"] = time.time()
            elif kind == "stop-preview":
                self.operation = False
                self._confirm_stop(value)
            elif kind == "mutation":
                self.operation = False
                if value.get("confirmationRequired"):
                    self._confirm_stop(value["preview"])
                elif isinstance(value.get("status"), dict):
                    self.status = value["status"]
                self._work("refresh-complete", self._refresh)
            elif kind == "startup-query":
                self.operation = False
                removing = value.get("installed") is True
                text = "关闭 Jenkins 托盘的登录自动启动？" if removing else "登录 Windows 时自动打开托盘？Jenkins 仍由你手动启动。"
                if nw.message_box(self.hwnd, text, "登录自动启动", nw.MB_YESNO | nw.MB_ICONINFORMATION) == nw.IDYES:
                    from . import startup
                    self.operation = True
                    self._work("startup-update", lambda: startup.remove(self.config) if removing else startup.install(self.config))
            elif kind == "startup-update":
                self.operation = False
                nw.message_box(self.hwnd, "登录启动项已更新。", "Jenkins 托盘", nw.MB_OK | nw.MB_ICONINFORMATION)
            elif kind in ("error", "poll-error"):
                if kind == "error":
                    self.operation = False
                    value = value["message"]
                self.status = {"state": "error", "message": value, "canStart": False, "canStop": False}
        self._paint()

    def _paint(self):
        icon = nw.status_icon(self.status.get("state", "error"))
        nw.update_icon(self.hwnd, 1, "Jenkins：" + label_for(self.status), icon)
        previous, self.icon = self.icon, icon
        if previous:
            nw.user32.DestroyIcon(previous)
        self._telemetry()

    def _command(self, command: int):
        key = next((key for key, number in COMMANDS.items() if number == command), None)
        if key is None or not MenuState(self.status, self.operation).enabled(key):
            return
        counts = self.telemetry["menuCommands"]
        counts[key] = counts.get(key, 0) + 1
        self.telemetry["lastCommand"] = key
        if key == "status":
            nw.message_box(self.hwnd, status_text(self.status), "Jenkins 状态", nw.MB_OK | nw.MB_ICONINFORMATION)
        elif key == "open":
            self._open()
        elif key == "start":
            self._mutate("start")
        elif key == "stop":
            from .operations import run_worker
            self.operation = True
            self._work("stop-preview", lambda: run_worker(self.config, "stop-preview"))
        elif key == "logs":
            opened = nw.shell32.ShellExecuteW(self.hwnd, "open", str(self.config.pilot_root / "logs"), None, None, nw.SW_SHOWNORMAL)
            if int(opened or 0) <= 32:
                raise TrayError("Windows 未能打开 Jenkins 日志目录")
        elif key == "refresh":
            self._work("refresh-complete", self._refresh)
        elif key == "startup":
            from . import startup
            self.operation = True
            self._work("startup-query", lambda: startup.query(self.config))
        elif key == "exit":
            self.closed = True
            self.finished.set()
            nw.user32.DestroyWindow(self.hwnd)
        self._telemetry()

    def _open(self):
        if nw.open_url(str(self.status.get("url") or "")):
            self.telemetry["browserOpened"] += 1
            self._telemetry()

    def _confirm_stop(self, preview):
        builds = preview.get("activeBuilds", [])
        if builds:
            listing = "\n".join(f"{b['job']} #{b['number']}（{b.get('requestId', '')}）" for b in builds)
            if nw.message_box(self.hwnd, "停止将中止以下任务：\n" + listing + "\n\n继续停止 Jenkins？排队任务会保留。",
                              "确认停止 Jenkins", nw.MB_YESNO | nw.MB_ICONWARNING) != nw.IDYES:
                return
        self._mutate("stop", confirmation=preview)

    def _mutate(self, action, confirmation=None):
        if self.operation:
            return
        from .operations import run_worker
        self.operation = True
        self.status = {**self.status, "state": "starting" if action == "start" else "stopping",
                       "canStart": False, "canStop": False}
        self._paint()
        self._work("mutation", lambda: run_worker(self.config, action, confirmation=confirmation))


def run(config) -> int:
    return TrayApp(config).run()
