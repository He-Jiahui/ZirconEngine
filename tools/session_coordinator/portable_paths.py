from __future__ import annotations

from pathlib import PurePosixPath, PureWindowsPath

from .models import CoordinatorError


_WINDOWS_RESERVED_BASENAMES = frozenset(
    {
        "con",
        "prn",
        "aux",
        "nul",
        *(f"com{index}" for index in range(1, 10)),
        *(f"lpt{index}" for index in range(1, 10)),
    }
)
_WINDOWS_FORBIDDEN_CHARACTERS = frozenset('<>:"|?*')


def normalize_portable_relative_path(
    value: object,
    *,
    code: str,
    message: str,
) -> str:
    """Return one path identity that is safe on both Git and Win32 filesystems."""
    if not isinstance(value, str):
        raise CoordinatorError(code, message)
    normalized = value.replace("\\", "/")
    path = PurePosixPath(normalized)
    windows = PureWindowsPath(value)
    if (
        not normalized
        or normalized.startswith("/")
        or path.is_absolute()
        or windows.is_absolute()
        or windows.drive
        or "\0" in normalized
        or not path.parts
    ):
        raise CoordinatorError(code, message, details={"path": value})
    for component in path.parts:
        if (
            component in {"", ".", ".."}
            or component.endswith((".", " "))
            or any(ord(character) < 32 for character in component)
            or any(character in _WINDOWS_FORBIDDEN_CHARACTERS for character in component)
            or component.split(".", 1)[0].casefold() in _WINDOWS_RESERVED_BASENAMES
        ):
            raise CoordinatorError(code, message, details={"path": value})
    return path.as_posix()


def portable_path_key(value: str) -> str:
    return "/".join(component.casefold() for component in PurePosixPath(value).parts)
