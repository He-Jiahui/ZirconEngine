from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from ..migrations import LATEST_SCHEMA_VERSION
from ..processes import ProcessIdentity
from .repository_identity import RepositoryIdentity


RUNTIME_DESCRIPTOR_VERSION = 2
SUPERVISION_API_VERSION = 1


@dataclass(frozen=True, slots=True)
class RuntimeDescriptor:
    host: str
    port: int
    token: str
    repo_root: Path
    repository: RepositoryIdentity
    instance_id: str
    started_at: str
    process: ProcessIdentity

    def to_payload(self) -> dict[str, object]:
        if self.host != "127.0.0.1":
            raise ValueError("Runtime descriptor host must be exact IPv4 loopback")
        if not self.token:
            raise ValueError("Runtime descriptor token must be non-empty")
        return {
            "descriptor_version": RUNTIME_DESCRIPTOR_VERSION,
            "host": self.host,
            "port": self.port,
            "token": self.token,
            "pid": self.process.pid,
            "process_creation_time": self.process.creation_time,
            "executable": self.process.executable,
            "command_line": list(self.process.command_line),
            "repo_root": str(self.repo_root),
            "repository_identity_version": self.repository.version,
            "repository_key": self.repository.key,
            "instance_id": self.instance_id,
            "started_at": self.started_at,
            "schema_version": LATEST_SCHEMA_VERSION,
            "control_api_versions": [1],
            "supervision_api_versions": [SUPERVISION_API_VERSION],
        }

    def diagnostic_payload(self) -> dict[str, object]:
        payload = self.to_payload()
        payload.pop("token")
        return payload
