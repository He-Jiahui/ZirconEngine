"""Shared command-plan helpers for export stages."""

from __future__ import annotations


def command_with_option(command: list[str], option: str, value: str) -> list[str]:
    rewritten: list[str] = []
    index = 0
    found = False
    while index < len(command):
        rewritten.append(command[index])
        if command[index] == option and index + 1 < len(command):
            rewritten.append(value)
            index += 2
            found = True
            continue
        index += 1
    if not found:
        rewritten.extend([option, value])
    return rewritten


def command_option_value_diagnostic(
    command: list[str],
    option: str,
    label: str,
) -> str | None:
    occurrences = 0
    for index, entry in enumerate(command):
        if entry != option:
            continue
        occurrences += 1
        if occurrences > 1:
            return f"{label} {option} must appear only once"
        if index + 1 >= len(command):
            return f"{label} {option} must include a value"
        if command[index + 1].startswith("-"):
            return f"{label} {option} value must not be another option"
    return None
