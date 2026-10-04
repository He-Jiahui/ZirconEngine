"""
ZirconEngine Jenkins 托盘启动器 — 可打包为独立 .exe

打包方式（在仓库根目录执行）:
    pip install pyinstaller
    pyinstaller tools/jenkins/launcher/launcher.spec

打包出的 .exe 不包含任何 tools/jenkins 代码；
它只做一件事：找到部署 Python，用它启动 tray。

依赖：仅 Python 标准库（pathlib, json, subprocess, sys, os, argparse）。
"""
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path


# ── 路径约定 ────────────────────────────────────────────────────────────────
# 启动器 .exe 放在仓库根目录旁边，或者仓库内任意位置均可。
# 唯一的假设：.jenkins/deployment-spec.json 存在，其中 repositoryRoot 指向仓库根。

_SPEC_SEARCH_NAMES = [
    # 相对于 .exe 所在目录向上逐级查找
    ".jenkins/deployment-spec.json",
]


def _find_spec(start: Path) -> Path | None:
    """从 start 目录向上查找 .jenkins/deployment-spec.json。"""
    for ancestor in [start, *start.parents]:
        candidate = ancestor / ".jenkins" / "deployment-spec.json"
        if candidate.is_file():
            return candidate
    return None


def _load_spec(spec_path: Path) -> dict:
    try:
        return json.loads(spec_path.read_text(encoding="utf-8"))
    except (OSError, ValueError) as exc:
        _fail(f"无法读取部署规格: {spec_path}\n{exc}")


def _fail(message: str) -> None:
    print(f"[启动失败] {message}", file=sys.stderr)
    if sys.platform == "win32":
        try:
            import ctypes
            ctypes.windll.user32.MessageBoxW(
                0, message, "ZirconEngine Jenkins 启动失败", 0x10
            )
        except Exception:
            pass
    sys.exit(1)


def _info(message: str) -> None:
    print(f"[ZirconJenkins] {message}")


def main() -> int:
    parser = argparse.ArgumentParser(
        description="ZirconEngine Jenkins 托盘启动器",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument(
        "--spec",
        type=Path,
        default=None,
        help="deployment-spec.json 路径（默认自动向上查找）",
    )
    parser.add_argument(
        "--no-tray",
        action="store_true",
        help="仅前置检查，不启动托盘（用于调试）",
    )
    parser.add_argument(
        "--wait",
        type=int,
        default=0,
        metavar="SECONDS",
        help="启动后轮询 Jenkins 健康端点的秒数（0 = 不等待）",
    )
    args = parser.parse_args()

    # ── 找到 spec ────────────────────────────────────────────────────────────
    if args.spec:
        spec_path = args.spec.resolve()
        if not spec_path.is_file():
            _fail(f"指定的 spec 文件不存在: {spec_path}")
    else:
        # 从 .exe 所在目录向上查找
        start = Path(sys.executable).parent
        spec_path = _find_spec(start)
        if spec_path is None:
            _fail(
                "找不到 .jenkins/deployment-spec.json\n"
                "请将启动器放在仓库根目录附近，或用 --spec 指定路径。"
            )

    _info(f"规格: {spec_path}")
    spec = _load_spec(spec_path)

    # ── 从 spec 推导路径 ─────────────────────────────────────────────────────
    repo_root = Path(spec.get("repositoryRoot", spec_path.parents[1]))
    python_exe = repo_root / ".jenkins" / "runtime" / "python" / "pythonw.exe"
    if not python_exe.is_file():
        # 回退：有控制台窗口的普通 python.exe
        python_exe = repo_root / ".jenkins" / "runtime" / "python" / "python.exe"
    if not python_exe.is_file():
        _fail(
            f"找不到部署专用 Python: {python_exe}\n"
            "请先完成 .jenkins/runtime 的部署。"
        )

    tray_launch = repo_root / "tools" / "jenkins" / "tray" / "launch.py"
    if not tray_launch.is_file():
        _fail(f"托盘启动入口不存在: {tray_launch}")

    # ── 前置检查（仅核验关键二进制，不 import tools.jenkins）──────────────────
    java = Path(spec.get("controller", {}).get("java", {}).get("executable", ""))
    war = Path(spec.get("controller", {}).get("warPath", ""))
    warnings = []
    if not java.is_file():
        warnings.append(f"Java 可执行文件不存在: {java}")
    if not war.is_file():
        warnings.append(f"Jenkins WAR 不存在: {war}")
    for w in warnings:
        print(f"[警告] {w}", file=sys.stderr)

    _info("前置检查完成")

    if args.no_tray:
        _info("--no-tray 模式：退出")
        return 0

    # ── 启动托盘 ──────────────────────────────────────────────────────────────
    cmd = [
        str(python_exe),
        str(tray_launch),
        "--formal",
        "--config", str(spec_path),
        "tray",
    ]
    _info(f"启动托盘: {python_exe.name} {tray_launch.name} --formal tray")

    creation_flags = 0
    if sys.platform == "win32":
        # DETACHED_PROCESS (0x08): 托盘独立于启动器进程，启动器退出不影响托盘
        creation_flags = 0x00000008

    proc = subprocess.Popen(
        cmd,
        cwd=str(repo_root),
        creationflags=creation_flags,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    _info(f"托盘进程已启动 (PID {proc.pid})")

    # ── 可选：等待 Jenkins 就绪 ───────────────────────────────────────────────
    if args.wait > 0:
        ctrl = spec.get("controller", {})
        url = f"http://{ctrl.get('listenAddress', '127.0.0.1')}:{ctrl.get('httpPort', 18080)}/api/json"
        import time
        import urllib.request
        _info(f"等待 Jenkins 就绪（最多 {args.wait}s）…")
        deadline = time.monotonic() + args.wait
        while time.monotonic() < deadline:
            try:
                with urllib.request.urlopen(url, timeout=3) as resp:
                    if resp.status == 200:
                        _info(f"Jenkins 已就绪: {url}")
                        break
            except Exception:
                pass
            time.sleep(3)
        else:
            _info(f"超时：Jenkins 在 {args.wait}s 内未就绪（托盘继续在后台运行）")

    ctrl = spec.get("controller", {})
    _info(
        f"Jenkins UI: "
        f"http://{ctrl.get('listenAddress', '127.0.0.1')}:{ctrl.get('httpPort', 18080)}/"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
