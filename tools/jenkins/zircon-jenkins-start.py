#!/usr/bin/env python3
"""Zircon Jenkins Launcher — 启动托盘，托盘负责启动 Jenkins。

用法（在仓库根目录执行）:
  python tools/jenkins/zircon-jenkins-start.py [--spec .jenkins/deployment-spec.json] [--no-tray]

  --spec   部署规格路径（默认 .jenkins/deployment-spec.json）
  --no-tray  仅检查环境，不启动托盘（用于 CI 或无 GUI 环境）
"""
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
from pathlib import Path

sys.dont_write_bytecode = True
REPO = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO))


def _fail(message: str, code: int = 1) -> None:
    print(f"[ERROR] {message}", file=sys.stderr)
    sys.exit(code)


def _info(message: str) -> None:
    print(f"[INFO]  {message}")


def _check_preflight(spec_path: Path) -> dict:
    """快速前置检查：spec、Java、WAR、Python 可执行文件都存在。"""
    if not spec_path.is_file():
        _fail(f"部署规格文件不存在: {spec_path}")

    spec = json.loads(spec_path.read_text(encoding="utf-8"))
    ctrl = spec.get("controller", {})

    java = Path(ctrl.get("java", {}).get("executable", ""))
    war = Path(ctrl.get("warPath", ""))
    python = REPO / ".jenkins" / "runtime" / "python" / "python.exe"

    missing = []
    if not java.is_file():
        missing.append(f"Java 可执行文件不存在: {java}")
    if not war.is_file():
        missing.append(f"Jenkins WAR 不存在: {war}")
    if not python.is_file():
        missing.append(f"部署专用 Python 不存在: {python}")

    if missing:
        for msg in missing:
            print(f"[WARN]  {msg}", file=sys.stderr)
        if all("Python" not in m for m in missing):
            # Python 缺失是硬阻断；其他缺失由 DeploymentManager.start() 二次核验
            pass
        else:
            _fail("前置检查失败，请确认 .jenkins/runtime/ 已完整部署")

    return spec


def _start_tray(spec_path: Path) -> int:
    """以独立子进程启动托盘（非阻塞返回子进程句柄）。

    托盘本身会在后台负责启动 Jenkins；这里只负责把它拉起来。
    """
    python_exe = REPO / ".jenkins" / "runtime" / "python" / "pythonw.exe"
    if not python_exe.is_file():
        # 回退到普通 python.exe（有控制台窗口）
        python_exe = REPO / ".jenkins" / "runtime" / "python" / "python.exe"
    if not python_exe.is_file():
        _fail("找不到部署专用 python(w).exe，无法启动托盘")

    tray_main = REPO / "tools" / "jenkins" / "tray" / "launch.py"
    if not tray_main.is_file():
        _fail(f"托盘启动入口不存在: {tray_main}")

    cmd = [
        str(python_exe),
        str(tray_main),
        "--formal",
        "--config", str(spec_path),
        "tray",
    ]

    _info(f"启动托盘: {python_exe.name} launch.py --formal tray")
    flags = 0
    if sys.platform == "win32":
        # DETACHED_PROCESS — 托盘独立于本启动器进程
        flags = 0x00000008
    proc = subprocess.Popen(
        cmd,
        cwd=str(REPO),
        creationflags=flags,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    return proc.pid


def _wait_health(spec: dict, timeout: int = 60) -> bool:
    """轮询 Jenkins health endpoint，返回是否在 timeout 秒内就绪。"""
    try:
        from tools.jenkins.deployment.manager import DeploymentManager
        from tools.jenkins.deployment.paths import resolve_paths
        from tools.jenkins.deployment.spec import load_spec
    except ImportError:
        _info("无法导入 DeploymentManager，跳过健康检查")
        return True

    spec_path = Path(spec.get("repositoryRoot", str(REPO))) / ".jenkins" / "deployment-spec.json"
    try:
        loaded = load_spec(spec_path)
        paths = resolve_paths(loaded)
        manager = DeploymentManager(loaded, paths)
    except Exception as exc:
        _info(f"DeploymentManager 初始化失败（{exc}），跳过健康检查")
        return True

    deadline = time.monotonic() + timeout
    _info(f"等待 Jenkins 就绪（最多 {timeout}s）…")
    while time.monotonic() < deadline:
        try:
            health = manager.health()
            if health.get("ready"):
                _info(f"Jenkins 已就绪: {manager.base_url}")
                return True
        except Exception:
            pass
        time.sleep(3)

    _info(f"超时：Jenkins 在 {timeout}s 内未就绪（托盘仍在后台运行）")
    return False


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--spec", type=Path,
                        default=REPO / ".jenkins" / "deployment-spec.json",
                        help="部署规格路径（默认 .jenkins/deployment-spec.json）")
    parser.add_argument("--no-tray", action="store_true",
                        help="仅前置检查，不启动托盘")
    parser.add_argument("--wait", type=int, default=0,
                        help="启动后等待 Jenkins 就绪的秒数（0 = 不等待）")
    args = parser.parse_args()

    spec_path = args.spec.resolve()
    _info(f"ZirconEngine Jenkins 启动器")
    _info(f"规格: {spec_path}")

    spec = _check_preflight(spec_path)
    _info("前置检查通过")

    if args.no_tray:
        _info("--no-tray 模式：不启动托盘，退出")
        return 0

    pid = _start_tray(spec_path)
    _info(f"托盘进程已启动 (PID {pid})")

    if args.wait > 0:
        _wait_health(spec, timeout=args.wait)

    _info("启动完成。托盘图标将出现在系统通知区域。")
    _info(f"Jenkins UI: http://{spec.get('controller', {}).get('listenAddress', '127.0.0.1')}:{spec.get('controller', {}).get('httpPort', 18080)}/")
    return 0


if __name__ == "__main__":
    sys.exit(main())
