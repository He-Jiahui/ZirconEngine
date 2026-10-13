"""Shared validation for filesystem-facing export profile names."""

from __future__ import annotations


def valid_export_profile_name(value: str) -> bool:
    if not isinstance(value, str):
        return False
    reserved_devices = {"CON", "PRN", "AUX", "NUL"} | {
        f"{prefix}{index}"
        for prefix in ("COM", "LPT")
        for index in range(1, 10)
    }
    return bool(value) and value.upper() not in reserved_devices and all(
        character.isascii() and (character.isalnum() or character in "_-")
        for character in value
    )
