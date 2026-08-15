from __future__ import annotations

import re
from dataclasses import dataclass

from .models import CoordinatorError


_OBJECT_ID = re.compile(r"^[0-9a-f]{40}(?:[0-9a-f]{24})?$")
_MAX_PATCH_BYTES = 128 * 1024

VALIDATION_ENVIRONMENT_KEYS = (
    "COMSPEC",
    "NUMBER_OF_PROCESSORS",
    "PATH",
    "PATHEXT",
    "PROCESSOR_ARCHITECTURE",
    "PROCESSOR_IDENTIFIER",
    "PYTHONIOENCODING",
    "PYTHONUTF8",
    "SYSTEMROOT",
    "TEMP",
    "TMP",
    "WINDIR",
)


@dataclass(frozen=True, slots=True)
class IsolatedPatchFinalizeResult:
    request_id: str
    session_id: str
    target: str
    base_head: str
    base_blob: str
    parent_head: str
    patch_hash: str
    derived_blob: str
    commit_sha: str
    staged_path_count: int
    staged_paths_fingerprint: str
    staged_projection_fingerprint: str

    def to_dict(self) -> dict[str, object]:
        return {
            "requestId": self.request_id,
            "sessionId": self.session_id,
            "target": self.target,
            "baseHead": self.base_head,
            "baseBlob": self.base_blob,
            "parentHead": self.parent_head,
            "patchHash": self.patch_hash,
            "derivedBlob": self.derived_blob,
            "commitSha": self.commit_sha,
            "stagedPathCount": self.staged_path_count,
            "stagedPathsFingerprint": self.staged_paths_fingerprint,
            "stagedProjectionFingerprint": self.staged_projection_fingerprint,
        }


def validation_commands(
    commands: tuple[tuple[str, ...], ...],
) -> tuple[tuple[str, ...], ...]:
    if not isinstance(commands, tuple) or not commands:
        raise CoordinatorError(
            "isolated_patch_validation_required",
            "Isolated maintenance finalize requires at least one validation command",
        )
    normalized: list[tuple[str, ...]] = []
    for command in commands:
        if not isinstance(command, tuple) or not command or any(
            not isinstance(part, str) or not part for part in command
        ):
            raise CoordinatorError(
                "isolated_patch_validation_invalid",
                "Each isolated maintenance validation command must be non-empty argv",
            )
        normalized.append(command)
    return tuple(normalized)


def patch_bytes(value: bytes) -> bytes:
    if (
        not isinstance(value, bytes)
        or not value
        or len(value) > _MAX_PATCH_BYTES
        or b"\0" in value
        or b"GIT binary patch" in value
        or b"Binary files " in value
    ):
        raise CoordinatorError(
            "isolated_patch_invalid",
            "Isolated patch must be non-empty, textual, and at most 128 KiB",
        )
    return value


def object_id(field: str, value: str) -> str:
    if not isinstance(value, str) or not _OBJECT_ID.fullmatch(value):
        raise CoordinatorError(
            "isolated_patch_identity_invalid",
            f"{field} must be an exact lowercase Git object ID",
        )
    return value


def required_text(field: str, value: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise CoordinatorError(
            "isolated_patch_input_invalid",
            f"{field} must be non-empty text",
        )
    return value.strip()
