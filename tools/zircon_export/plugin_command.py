"""Plugin subcommand dispatch for zircon_export."""

from __future__ import annotations

from typing import Sequence

from .plugin_build import parse_plugin_build_args, run_plugin_build
from .plugin_validate import parse_plugin_validate_args, run_plugin_validate


PLUGIN_COMMAND_ROOT = "plugin"
PLUGIN_BUILD_COMMAND = "build"
PLUGIN_VALIDATE_COMMAND = "validate"


def dispatch_plugin_command(argv: Sequence[str]) -> int | None:
    argv_list = list(argv)
    if len(argv_list) < 2 or argv_list[0] != PLUGIN_COMMAND_ROOT:
        return None
    command = argv_list[1]
    command_args = argv_list[2:]
    if command == PLUGIN_BUILD_COMMAND:
        return run_plugin_build(parse_plugin_build_args(command_args))
    if command == PLUGIN_VALIDATE_COMMAND:
        return run_plugin_validate(parse_plugin_validate_args(command_args))
    return None
