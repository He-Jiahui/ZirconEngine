"""Pure menu model and status projection for the Jenkins tray."""
from __future__ import annotations

from dataclasses import dataclass

COMMANDS = {
    "status": 1001, "open": 1002, "start": 1003, "stop": 1004, "restart": 1009,
    "logs": 1005, "refresh": 1006, "startup": 1007, "exit": 1008,
}


@dataclass(frozen=True)
class MenuState:
    status: dict
    operation: bool = False

    @property
    def state(self) -> str:
        return str(self.status.get("state", "degraded"))

    def enabled(self, key: str) -> bool:
        if key in {"status", "refresh", "exit"}: return True
        if self.operation and key in {"start", "stop", "restart", "startup"}: return False
        if key == "open": return bool(self.status.get("url")) and (self.status.get("controller", {}).get("ready") is True or self.state in {"ready", "busy", "starting", "stopping"})
        if key == "start": return bool(self.status.get("canStart", self.state in {"stopped", "error"}))
        if key == "stop": return bool(self.status.get("canStop", self.state in {"ready", "busy"}))
        if key == "restart": return self.state in {"ready", "busy"} and bool(self.status.get("ownerKnown")) and bool(self.status.get("canStop"))
        if key == "logs": return True
        if key == "startup": return True
        return False


def label_for(status: dict) -> str:
    state = str(status.get("state", "degraded"))
    return {"stopped": "已停止", "starting": "启动中", "ready": "就绪", "busy": "运行任务",
            "stopping": "停止中", "degraded": "异常", "error": "异常"}.get(state, state)


def menu_items(state: MenuState) -> list[tuple[int, str, bool]]:
    return [(COMMANDS["status"], f"Jenkins：{label_for(state.status)}", True),
            (COMMANDS["open"], "打开 Jenkins", state.enabled("open")),
            (COMMANDS["start"], "启动 Jenkins", state.enabled("start")),
            (COMMANDS["stop"], "停止 Jenkins", state.enabled("stop")),
            (COMMANDS["restart"], "重启 Jenkins", state.enabled("restart")),
            (COMMANDS["logs"], "打开日志", state.enabled("logs")),
            (COMMANDS["refresh"], "刷新", True),
            (COMMANDS["startup"], "登录自动启动", state.enabled("startup")),
            (COMMANDS["exit"], "退出托盘", True)]


def status_text(status: dict) -> str:
    lines = [f"状态：{label_for(status)}"]
    if status.get("message"): lines.append(str(status["message"]))
    if status.get("url"): lines.append(f"地址：{status['url']}")
    active = status.get("activeBuilds") or []
    lines.append(f"活动构建：{len(active)}；排队：{int(status.get('queuedCount') or 0)}")
    return "\n".join(lines)
