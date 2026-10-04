"""Portable path names for independently admitted Jenkins inputs and storage."""

from __future__ import annotations

from pathlib import PurePosixPath, PureWindowsPath

from ..contracts import PilotError

_WINDOWS_RESERVED_BASENAMES = frozenset({
    "con", "prn", "aux", "nul",
    *(f"com{index}" for index in range(1, 10)),
    *(f"lpt{index}" for index in range(1, 10)),
    *(f"{prefix}{index}" for prefix in ("com", "lpt") for index in ("¹", "²", "³")),
})
_WINDOWS_FORBIDDEN_CHARACTERS = frozenset('<>:"|?*')


def normalize_portable_relative_path(value: object, *, code: str, message: str) -> str:
    """Return a canonical relative name or fail before any filesystem access."""
    if not isinstance(value, str):
        raise PilotError(f"{code}: {message}")
    normalized = value.replace("\\", "/")
    path = PurePosixPath(normalized)
    windows = PureWindowsPath(value)
    if (not normalized or normalized.startswith("/") or path.is_absolute()
            or windows.is_absolute() or windows.drive or "\0" in normalized
            or not path.parts):
        raise PilotError(f"{code}: {message}")
    # Inspect raw components as PurePath collapses dot and empty components.
    for component in normalized.split("/"):
        if (component in {"", ".", ".."} or component.endswith((".", " "))
                or any(ord(character) < 32 for character in component)
                or any(character in _WINDOWS_FORBIDDEN_CHARACTERS for character in component)
                or component.split(".", 1)[0].casefold() in _WINDOWS_RESERVED_BASENAMES):
            raise PilotError(f"{code}: {message}")
    return path.as_posix()


def portable_path_key(value: str) -> str:
    return "/".join(component.casefold() for component in PurePosixPath(value).parts)
