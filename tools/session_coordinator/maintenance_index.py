from __future__ import annotations

import os
import subprocess
import tempfile
import time
from contextlib import contextmanager
from dataclasses import dataclass
from pathlib import Path
from typing import Callable, Iterator

from .models import CoordinatorError


@dataclass(frozen=True, slots=True)
class MaintenanceIndexPublication:
    shared_index: Path
    aligned_index: Path
    index_content: bytes


class MaintenanceIndexService:
    """Build maintenance commits privately and align only their shared entries."""

    def __init__(
        self,
        repo_root: str | Path,
        *,
        error_sanitizer: Callable[[str | bytes | None], str],
    ):
        self.repo_root = Path(repo_root).resolve()
        self.error_sanitizer = error_sanitizer

    @contextmanager
    def staging_environment(self, parent_head: str) -> Iterator[dict[str, str]]:
        with self._index_file("maintenance-finalize-index-", None) as index_path:
            environment = git_environment(GIT_INDEX_FILE=str(index_path))
            self._git_text("read-tree", parent_head, environment=environment)
            yield environment

    @contextmanager
    def publication(
        self,
        *,
        expected_head: str,
        commit_sha: str,
        approved: tuple[str, ...],
        path_chunks: tuple[tuple[str, ...], ...],
        recover_lock: Callable[[Path], None],
    ) -> Iterator[MaintenanceIndexPublication]:
        index_path = self._index_path()
        if not index_path.exists():
            raise CoordinatorError(
                "finalize_index_missing",
                "Shared Git index is missing before maintenance publication",
            )
        lock_path = index_path.with_name(index_path.name + ".lock")
        self._acquire_publish_lock(lock_path, recover_lock)
        try:
            if self._git_text("rev-parse", "HEAD") != expected_head:
                raise CoordinatorError(
                    "finalize_baseline_head_changed",
                    "HEAD changed before maintenance publication",
                )
            index_content = index_path.read_bytes()
            with self._index_file(
                "maintenance-finalize-aligned-", index_content
            ) as aligned_index:
                environment = git_environment(GIT_INDEX_FILE=str(aligned_index))
                staged = set(self._staged_paths(expected_head, environment))
                approved_staged = sorted(staged.intersection(approved), key=str.casefold)
                if approved_staged:
                    raise CoordinatorError(
                        "finalize_approved_path_staged",
                        "Maintenance paths must not already be staged in the shared index",
                        details={"paths": approved_staged},
                    )
                before_projection = self._staged_projection(
                    expected_head, environment
                )
                for chunk in path_chunks:
                    self._git_text(
                        "reset",
                        "--quiet",
                        commit_sha,
                        "--",
                        *chunk,
                        environment=environment,
                    )
                after_projection = self._staged_projection(commit_sha, environment)
                if after_projection != before_projection:
                    raise CoordinatorError(
                        "finalize_foreign_index_changed",
                        "Aligning maintenance paths changed the foreign staged projection",
                    )
                yield MaintenanceIndexPublication(
                    shared_index=index_path,
                    aligned_index=aligned_index,
                    index_content=index_content,
                )
        finally:
            lock_path.unlink(missing_ok=True)

    @staticmethod
    def publish(publication: MaintenanceIndexPublication) -> None:
        os.replace(publication.aligned_index, publication.shared_index)

    @contextmanager
    def _index_file(
        self, prefix: str, content: bytes | None
    ) -> Iterator[Path]:
        git_dir = self._git_dir()
        descriptor, raw_path = tempfile.mkstemp(prefix=prefix, dir=git_dir)
        index_path = Path(raw_path)
        try:
            if content is None:
                os.close(descriptor)
                index_path.unlink(missing_ok=True)
            else:
                with os.fdopen(descriptor, "wb") as stream:
                    stream.write(content)
                    stream.flush()
                    os.fsync(stream.fileno())
            yield index_path
        finally:
            index_path.unlink(missing_ok=True)
            index_path.with_name(index_path.name + ".lock").unlink(missing_ok=True)

    def _index_path(self) -> Path:
        return self._git_dir() / "index"

    def _git_dir(self) -> Path:
        git_dir = Path(self._git_text("rev-parse", "--git-dir"))
        if not git_dir.is_absolute():
            git_dir = (self.repo_root / git_dir).resolve()
        return git_dir

    @staticmethod
    def _acquire_publish_lock(
        lock_path: Path, recover_lock: Callable[[Path], None]
    ) -> None:
        deadline = time.monotonic() + 5.0
        recovery_attempted = False
        while True:
            try:
                descriptor = os.open(
                    lock_path,
                    os.O_CREAT | os.O_EXCL | os.O_WRONLY,
                    0o600,
                )
            except FileExistsError as error:
                if not recovery_attempted:
                    recovery_attempted = True
                    try:
                        recover_lock(lock_path)
                    except CoordinatorError:
                        pass
                    continue
                if time.monotonic() >= deadline:
                    raise CoordinatorError(
                        "finalize_index_lock_occupied",
                        "Shared Git index remained locked before maintenance publication",
                    ) from error
                time.sleep(0.05)
                continue
            os.close(descriptor)
            return

    def _staged_paths(
        self, base_tree: str, environment: dict[str, str]
    ) -> tuple[str, ...]:
        raw = self._git_bytes(
            "diff",
            "--cached",
            "--name-only",
            "-z",
            "--no-renames",
            base_tree,
            "--",
            environment=environment,
        )
        return tuple(
            os.fsdecode(path).replace("\\", "/")
            for path in raw.split(b"\0")
            if path
        )

    def _staged_projection(
        self, base_tree: str, environment: dict[str, str]
    ) -> bytes:
        return self._git_bytes(
            "diff",
            "--cached",
            "--binary",
            "--no-ext-diff",
            "--no-renames",
            base_tree,
            "--",
            environment=environment,
        )

    def _git_text(
        self, *arguments: str, environment: dict[str, str] | None = None
    ) -> str:
        return self._git_bytes(*arguments, environment=environment).decode(
            "utf-8", errors="replace"
        ).strip()

    def _git_bytes(
        self, *arguments: str, environment: dict[str, str] | None = None
    ) -> bytes:
        command = ["git", *arguments]
        try:
            return subprocess.run(
                command,
                cwd=self.repo_root,
                env=environment or git_environment(),
                check=True,
                capture_output=True,
            ).stdout
        except subprocess.CalledProcessError as error:
            stderr = self.error_sanitizer(error.stderr)
            raise CoordinatorError(
                "finalize_git_command_failed",
                f"{' '.join(command[:2])} failed with exit code {error.returncode}",
                details={
                    "command": " ".join(command[:2]),
                    "exit_code": error.returncode,
                    "stderr": stderr,
                },
            ) from error
        except OSError as error:
            raise CoordinatorError(
                "finalize_git_command_failed",
                "Cannot start Git maintenance-index command",
                details={"error": self.error_sanitizer(str(error))},
            ) from error


def git_environment(**overrides: str) -> dict[str, str]:
    environment = {**os.environ, "GIT_OPTIONAL_LOCKS": "0"}
    environment.update(overrides)
    return environment
