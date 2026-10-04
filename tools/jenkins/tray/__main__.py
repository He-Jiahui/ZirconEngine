"""CLI and pythonw entry for the independent Jenkins tray."""
from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path

from .config import configure, load_config, validate_runtime, TrayError
from .persistence import read_json, write_json, safe_error
from . import native_windows as nw


def main() -> int:
    sys.dont_write_bytecode = True
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--config", type=Path)
    parser.add_argument("--formal", action="store_true", help="use repository-local formal Jenkins profile")
    parser.add_argument("--response-file", type=Path)
    commands = parser.add_subparsers(dest="action")
    commands.add_parser("tray")
    for action in ("status", "start", "stop-preview", "restart", "logs", "reconcile", "validate-config"):
        commands.add_parser(action)
    stopping = commands.add_parser("stop")
    stopping.add_argument("--confirmation-file", type=Path)
    setting = commands.add_parser("configure")
    setting.add_argument("--repo-root", type=Path, required=True)
    setting.add_argument("--root", type=Path, required=True)
    setting.add_argument("--vsdevcmd", type=Path, required=True)
    args = parser.parse_args()
    if args.formal:
        if args.config is None: parser.error("--formal requires --config deployment-spec.json")
        from .formal import load_formal_config, start, status, stop
        config = load_formal_config(args.config)
        action = args.action or "tray"
        if action == "tray":
            from .formal_app import FormalTray
            return FormalTray(config).run()
        if action in {"status", "validate-config"}: result = status(config)
        elif action == "start": result = start(config)
        elif action == "stop": result = stop(config)
        elif action == "restart": stop(config); result = start(config)
        elif action == "logs":
            if nw.IS_WINDOWS: os.startfile(str(config.paths.logs))
            result = {"logs": str(config.paths.logs)}
        else: raise ValueError("formal action unsupported")
        print(json.dumps({"ok": True, "result": result}, ensure_ascii=True, sort_keys=True)); return 0
    config = None
    try:
        if args.config is None:
            raise TrayError("必须通过 --config 显式选择 Jenkins 实例")
        if args.action == "configure":
            config = configure(repo_root=args.repo_root, pilot_root=args.root,
                               config_path=args.config, vsdevcmd_path=args.vsdevcmd)
            result = {"configured": True, "configFile": str(config.config_path), "pilotRoot": str(config.pilot_root)}
        else:
            config = load_config(args.config)
            os.chdir(config.repo_root)
            if args.action in (None, "tray"):
                from .app import run
                return run(config)
            if Path(sys.executable) != config.python_executable:
                raise TrayError("管理入口必须由已配置的 python.exe 执行")
            confirmation = None
            if getattr(args, "confirmation_file", None):
                path = args.confirmation_file
                if not path.is_absolute() or not path.is_relative_to(config.state_dir / "confirmations"):
                    raise TrayError("停止确认必须来自本托盘配置的确认目录")
                confirmation = read_json(path)
                if confirmation is None:
                    raise TrayError("停止确认记录不存在")
            if args.action == "validate-config":
                validate_runtime(config)
                result = {"valid": True, "repoRoot": str(config.repo_root), "pilotRoot": str(config.pilot_root),
                          "rootIdentity": dict(config.root_identity)}
            else:
                from .operations import execute
                result = execute(config, args.action, confirmation=confirmation)
        response, code = {"ok": True, "result": result}, 0
    except Exception as error:
        response, code = {"ok": False, "error": safe_error(error)}, 1
        if sys.stdout is None:
            from .native_windows import message_box
            message_box(0, response["error"], "Jenkins 托盘", 0x10)
    if args.response_file:
        if config is None or not args.response_file.is_absolute() or not args.response_file.is_relative_to(config.state_dir / "responses"):
            return 2
        write_json(args.response_file, response)
    if sys.stdout is not None:
        print(json.dumps(response, ensure_ascii=True, sort_keys=True))
    return code


if __name__ == "__main__":
    raise SystemExit(main())
