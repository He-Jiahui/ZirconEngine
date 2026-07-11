from __future__ import annotations

import hashlib
import os
from dataclasses import dataclass
from pathlib import Path


REPOSITORY_IDENTITY_VERSION = 1


@dataclass(frozen=True, slots=True)
class RepositoryIdentity:
    version: int
    canonical_path: str
    key: str

    @property
    def short_key(self) -> str:
        return self.key[:10].upper()


def repository_identity(repo_root: str | Path) -> RepositoryIdentity:
    resolved = str(Path(repo_root).resolve())
    canonical = os.path.normpath(resolved).lower()
    digest = hashlib.sha256(canonical.encode("utf-8")).hexdigest()
    return RepositoryIdentity(REPOSITORY_IDENTITY_VERSION, canonical, digest)
