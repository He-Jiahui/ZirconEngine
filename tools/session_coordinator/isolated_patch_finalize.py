from __future__ import annotations

import hashlib
import json
import os
import subprocess
import tempfile
import uuid
from contextlib import contextmanager
from pathlib import Path
from sqlite3 import IntegrityError
from typing import Callable, Iterator

from .baselines import BaselineService
from .database import Database
from .git_finalize import (
    GitFinalizeService,
    _StagedSecretStreamScanner,
    _TYPE_DECLARATION_SOURCE_SUFFIXES,
)
from .git_index_lock import IndexLockRecoveryRefused, recover_stale_index_lock
from .isolated_patch_contract import (
    VALIDATION_ENVIRONMENT_KEYS,
    IsolatedPatchFinalizeResult,
    object_id,
    patch_bytes,
    required_text,
    validation_commands,
)
from .leases import LeaseService
from .models import CoordinatorError, utc_text


_GIT_ERROR_LIMIT = 2_048
class IsolatedPatchFinalizeService:
    """Publish one HEAD-derived patch without reading or writing live target bytes."""

    _object_id = staticmethod(object_id)
    _patch = staticmethod(patch_bytes)
    _text = staticmethod(required_text)
    _validation_commands = staticmethod(validation_commands)

    def __init__(
        self,
        database: Database,
        repo_root: str | Path,
        baselines: BaselineService,
        leases: LeaseService,
        *,
        index_lock_recoverer: Callable[[Path], object | None] | None = None,
    ):
        self.database = database
        self.repo_root = Path(repo_root).resolve()
        self.baselines = baselines
        self.leases = leases
        self.index_lock_recoverer = index_lock_recoverer or recover_stale_index_lock

    def finalize(
        self,
        *,
        session_id: str,
        target: str,
        patch: bytes,
        expected_head: str,
        expected_blob: str,
        message: str,
        validation_commands: tuple[tuple[str, ...], ...],
    ) -> IsolatedPatchFinalizeResult:
        session_id = self._text("session_id", session_id)
        normalized = self.leases.path_policy.normalize(target)
        target = normalized.display
        patch = self._patch(patch)
        expected_head = self._object_id("expected_head", expected_head)
        expected_blob = self._object_id("expected_blob", expected_blob)
        message = GitFinalizeService._format_message(message)
        validation_commands = self._validation_commands(validation_commands)
        self._require_target_file(normalized.absolute)
        self._require_lease(session_id, target)

        request_id = uuid.uuid4().hex
        patch_hash = hashlib.sha256(patch).hexdigest()
        self._insert_request(
            request_id,
            session_id=session_id,
            target=target,
            message=message,
            validation_commands=validation_commands,
        )

        commit_sha = ""
        ref_updated = False
        retain_mutex = False
        index_snapshot_persisted = False
        try:
            with self._git_mutex(
                f"isolated-patch:{request_id}",
                retain_on_error=lambda: retain_mutex,
            ):
                parent_head = self._git_text("rev-parse", "refs/heads/main")
                self._require_main_head(parent_head)
                self._require_head_identity(
                    parent_head,
                    target=target,
                    expected_head=expected_head,
                    expected_blob=expected_blob,
                )
                self._require_lease(session_id, target)
                worktree_snapshot = normalized.absolute.read_bytes()
                index_path = self._index_path()
                self._recover_index_lock(request_id, session_id, index_path)
                index_existed = index_path.exists()
                if not index_existed:
                    raise CoordinatorError(
                        "isolated_patch_index_missing",
                        "Shared Git index is missing before isolated finalize",
                    )
                index_snapshot = index_path.read_bytes() if index_existed else b""
                self._persist_start(
                    request_id,
                    start_head=parent_head,
                    index_existed=index_existed,
                    index_snapshot=index_snapshot,
                )
                index_snapshot_persisted = True
                with self._index_from_snapshot(index_snapshot) as snapshot_environment:
                    self._require_target_unstaged(
                        parent_head,
                        target,
                        environment=snapshot_environment,
                    )
                    staged_paths = self._staged_paths(
                        parent_head, environment=snapshot_environment
                    )
                    staged_projection = self._staged_projection(
                        parent_head, environment=snapshot_environment
                    )
                staged_paths_fingerprint = self._paths_fingerprint(staged_paths)
                staged_projection_fingerprint = hashlib.sha256(
                    staged_projection
                ).hexdigest()

                with self._temporary_index(parent_head) as (
                    index_environment,
                    _patch_index,
                ):
                    self._apply_patch(index_environment, patch)
                    changed = self._staged_paths(parent_head, environment=index_environment)
                    if changed != (target,):
                        raise CoordinatorError(
                            "isolated_patch_scope_mismatch",
                            "Isolated patch must change exactly its declared target",
                            details={"target": target, "changed": list(changed)},
                        )
                    derived_blob = self._staged_blob(index_environment, target)
                    if derived_blob == expected_blob:
                        raise CoordinatorError(
                            "isolated_patch_empty",
                            "Isolated patch did not derive a new target blob",
                        )
                    self._require_no_secrets(target, patch, derived_blob)
                    tree = self._git_text("write-tree", environment=index_environment)
                    prepared = self._identity_payload(
                        request_id=request_id,
                        session_id=session_id,
                        target=target,
                        base_head=expected_head,
                        base_blob=expected_blob,
                        parent_head=parent_head,
                        patch_hash=patch_hash,
                        derived_blob=derived_blob,
                        staged_paths=staged_paths,
                        staged_paths_fingerprint=staged_paths_fingerprint,
                        staged_projection_fingerprint=staged_projection_fingerprint,
                    )
                    self._record_event(
                        session_id,
                        "maintenance.isolated_patch_prepared",
                        prepared,
                    )
                    self._validate_isolated_tree(
                        index_environment,
                        validation_commands,
                        prepared,
                    )
                    validated = {
                        **prepared,
                        "validationCommands": [
                            list(command) for command in validation_commands
                        ],
                        "validationStatus": "passed",
                    }
                    self._record_event(
                        session_id,
                        "maintenance.isolated_patch_validated",
                        validated,
                    )

                self._require_unchanged_before_publish(
                    session_id=session_id,
                    target=target,
                    target_path=normalized.absolute,
                    worktree_snapshot=worktree_snapshot,
                    parent_head=parent_head,
                    expected_head=expected_head,
                    expected_blob=expected_blob,
                    index_path=index_path,
                    index_snapshot=index_snapshot,
                )
                commit_sha = self._git_text(
                    "commit-tree", tree, "-p", parent_head, "-m", message
                )
                with self.database.transaction() as connection:
                    connection.execute(
                        "UPDATE finalize_requests SET ref_updated_sha=? WHERE request_id=?",
                        (commit_sha, request_id),
                    )
                try:
                    with self._aligned_index(
                        index_snapshot,
                        target=target,
                        derived_blob=derived_blob,
                    ) as (aligned_environment, aligned_index):
                        after_paths = self._staged_paths(
                            commit_sha, environment=aligned_environment
                        )
                        after_projection = self._staged_projection(
                            commit_sha, environment=aligned_environment
                        )
                        if (
                            after_paths != staged_paths
                            or after_projection != staged_projection
                        ):
                            raise CoordinatorError(
                                "isolated_patch_shared_index_changed",
                                "Aligned index changed the foreign staged projection",
                                details={
                                    "beforeCount": len(staged_paths),
                                    "afterCount": len(after_paths),
                                    "beforeFingerprint": staged_projection_fingerprint,
                                    "afterFingerprint": hashlib.sha256(
                                        after_projection
                                    ).hexdigest(),
                                },
                            )
                        with self._index_publish_lock(
                            index_path, index_snapshot
                        ) as lock_path:
                            self._require_unchanged_before_publish(
                                session_id=session_id,
                                target=target,
                                target_path=normalized.absolute,
                                worktree_snapshot=worktree_snapshot,
                                parent_head=parent_head,
                                expected_head=expected_head,
                                expected_blob=expected_blob,
                                index_path=index_path,
                                index_snapshot=index_snapshot,
                            )
                            self._record_event(
                                session_id,
                                "maintenance.isolated_patch_index_locked",
                                {
                                    "requestId": request_id,
                                    "lockPath": self._display_path(lock_path),
                                    "indexSnapshotHash": hashlib.sha256(
                                        index_snapshot
                                    ).hexdigest(),
                                },
                            )
                            self._require_lease(session_id, target)
                            if normalized.absolute.read_bytes() != worktree_snapshot:
                                raise CoordinatorError(
                                    "isolated_patch_worktree_changed",
                                    "Mixed worktree target changed before main publication",
                                    details={"target": target},
                                )
                            self._git_text(
                                "update-ref",
                                "refs/heads/main",
                                commit_sha,
                                parent_head,
                            )
                            ref_updated = True
                            self._replace_index(aligned_index, index_path)
                    self.baselines.accept_commit(
                        (target,),
                        commit_sha=commit_sha,
                        reason=f"isolated maintenance finalize {commit_sha}",
                    )
                    finalized = {
                        **validated,
                        "commitSha": commit_sha,
                    }
                    with self.database.transaction() as connection:
                        connection.execute(
                            """
                            UPDATE finalize_requests
                            SET status='committed', commit_sha=?, ref_updated_sha=?,
                                completed_at=?, index_snapshot=NULL
                            WHERE request_id=?
                            """,
                            (commit_sha, commit_sha, utc_text(), request_id),
                        )
                        connection.execute(
                            """
                            INSERT INTO events(session_id, event_type, payload_json, created_at)
                            VALUES (?, 'maintenance.isolated_patch_finalized', ?, ?)
                            """,
                            (session_id, json.dumps(finalized, sort_keys=True), utc_text()),
                        )
                except BaseException:
                    retain_mutex = True
                    raise
        except BaseException as error:
            if not ref_updated:
                self._mark_failed(request_id, str(error))
            elif index_snapshot_persisted:
                retain_mutex = True
            raise

        return IsolatedPatchFinalizeResult(
            request_id=request_id,
            session_id=session_id,
            target=target,
            base_head=expected_head,
            base_blob=expected_blob,
            parent_head=parent_head,
            patch_hash=patch_hash,
            derived_blob=derived_blob,
            commit_sha=commit_sha,
            staged_path_count=len(staged_paths),
            staged_paths_fingerprint=staged_paths_fingerprint,
            staged_projection_fingerprint=staged_projection_fingerprint,
        )

    def _insert_request(
        self,
        request_id: str,
        *,
        session_id: str,
        target: str,
        message: str,
        validation_commands: tuple[tuple[str, ...], ...],
    ) -> None:
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO finalize_requests(
                    request_id, session_id, message, paths_json, categories_json,
                    untracked_json, validation_json, maintenance, status, created_at
                ) VALUES (?, ?, ?, ?, ?, '[]', ?, 1, 'previewed', ?)
                """,
                (
                    request_id,
                    session_id,
                    message,
                    json.dumps((target,)),
                    json.dumps({"isolatedMaintenance": [target]}),
                    json.dumps(validation_commands),
                    utc_text(),
                ),
            )

    def _persist_start(
        self,
        request_id: str,
        *,
        start_head: str,
        index_existed: bool,
        index_snapshot: bytes,
    ) -> None:
        with self.database.transaction() as connection:
            connection.execute(
                """
                UPDATE finalize_requests
                SET status='finalizing', start_head=?, index_existed=?, index_snapshot=?
                WHERE request_id=? AND status='previewed'
                """,
                (start_head, 1 if index_existed else 0, index_snapshot, request_id),
            )

    def _mark_failed(self, request_id: str, error_text: str) -> None:
        with self.database.transaction() as connection:
            connection.execute(
                """
                UPDATE finalize_requests
                SET status='failed', error_text=?, completed_at=?, index_snapshot=NULL
                WHERE request_id=? AND status!='committed'
                """,
                (error_text[:4096], utc_text(), request_id),
            )

    def _require_head_identity(
        self,
        parent_head: str,
        *,
        target: str,
        expected_head: str,
        expected_blob: str,
    ) -> None:
        expected_mode, expected_oid = self._tree_entry(expected_head, target)
        if expected_mode != "100644" or expected_oid != expected_blob:
            raise CoordinatorError(
                "isolated_patch_base_identity_mismatch",
                "Expected HEAD does not contain the declared regular-file blob",
                details={
                    "expectedHead": expected_head,
                    "expectedBlob": expected_blob,
                    "actualBlob": expected_oid,
                    "mode": expected_mode,
                },
            )
        if not self._is_ancestor(expected_head, parent_head):
            raise CoordinatorError(
                "isolated_patch_base_not_ancestor",
                "Expected patch HEAD is not an ancestor of current main",
                details={"expectedHead": expected_head, "currentHead": parent_head},
            )
        current_mode, current_oid = self._tree_entry(parent_head, target)
        if current_mode != "100644" or current_oid != expected_blob:
            raise CoordinatorError(
                "isolated_patch_target_blob_changed",
                "Current main changed the isolated patch target blob",
                details={
                    "target": target,
                    "expectedBlob": expected_blob,
                    "currentBlob": current_oid,
                    "mode": current_mode,
                },
            )

    def _require_unchanged_before_publish(
        self,
        *,
        session_id: str,
        target: str,
        target_path: Path,
        worktree_snapshot: bytes,
        parent_head: str,
        expected_head: str,
        expected_blob: str,
        index_path: Path,
        index_snapshot: bytes,
    ) -> None:
        self._require_lease(session_id, target)
        self._require_main_head(parent_head)
        self._require_head_identity(
            parent_head,
            target=target,
            expected_head=expected_head,
            expected_blob=expected_blob,
        )
        if target_path.read_bytes() != worktree_snapshot:
            raise CoordinatorError(
                "isolated_patch_worktree_changed",
                "Mixed worktree target changed while the isolated patch was being validated",
                details={"target": target},
            )
        current_index = index_path.read_bytes() if index_path.exists() else b""
        if current_index != index_snapshot:
            raise CoordinatorError(
                "isolated_patch_shared_index_changed",
                "Shared Git index changed while the isolated patch was being validated",
                details={
                    "expectedFingerprint": hashlib.sha256(index_snapshot).hexdigest(),
                    "currentFingerprint": hashlib.sha256(current_index).hexdigest(),
                },
            )

    def _require_main_head(self, parent_head: str) -> None:
        try:
            symbolic_head = self._git_text("symbolic-ref", "--quiet", "HEAD")
        except CoordinatorError as error:
            raise CoordinatorError(
                "isolated_patch_branch_changed",
                "Isolated maintenance finalize requires symbolic main HEAD",
            ) from error
        current_main = self._git_text("rev-parse", "refs/heads/main")
        current_head = self._git_text("rev-parse", "HEAD")
        if (
            symbolic_head != "refs/heads/main"
            or current_main != parent_head
            or current_head != parent_head
        ):
            raise CoordinatorError(
                "isolated_patch_branch_changed",
                "Main branch identity changed while the isolated patch was active",
                details={
                    "symbolicHead": symbolic_head,
                    "expectedHead": parent_head,
                    "mainHead": current_main,
                    "currentHead": current_head,
                },
            )

    @contextmanager
    def _temporary_index(self, parent_head: str) -> Iterator[tuple[dict[str, str], Path]]:
        git_dir = Path(self._git_text("rev-parse", "--git-dir"))
        if not git_dir.is_absolute():
            git_dir = (self.repo_root / git_dir).resolve()
        descriptor, raw_path = tempfile.mkstemp(prefix="isolated-patch-index-", dir=git_dir)
        os.close(descriptor)
        index_path = Path(raw_path)
        index_path.unlink(missing_ok=True)
        environment = self._git_environment(GIT_INDEX_FILE=str(index_path))
        try:
            self._git_text("read-tree", parent_head, environment=environment)
            yield environment, index_path
        finally:
            index_path.unlink(missing_ok=True)
            index_path.with_name(index_path.name + ".lock").unlink(missing_ok=True)

    @contextmanager
    def _index_from_snapshot(
        self, index_snapshot: bytes
    ) -> Iterator[dict[str, str]]:
        with self._index_file("isolated-patch-snapshot-", index_snapshot) as index_path:
            yield self._git_environment(GIT_INDEX_FILE=str(index_path))

    @contextmanager
    def _aligned_index(
        self,
        index_snapshot: bytes,
        *,
        target: str,
        derived_blob: str,
    ) -> Iterator[tuple[dict[str, str], Path]]:
        with self._index_file("isolated-patch-aligned-", index_snapshot) as index_path:
            environment = self._git_environment(GIT_INDEX_FILE=str(index_path))
            self._git_text(
                "update-index",
                "--add",
                "--cacheinfo",
                f"100644,{derived_blob},{target}",
                environment=environment,
            )
            yield environment, index_path

    @contextmanager
    def _index_file(
        self, prefix: str, content: bytes
    ) -> Iterator[Path]:
        git_dir = Path(self._git_text("rev-parse", "--git-dir"))
        if not git_dir.is_absolute():
            git_dir = (self.repo_root / git_dir).resolve()
        descriptor, raw_path = tempfile.mkstemp(prefix=prefix, dir=git_dir)
        index_path = Path(raw_path)
        try:
            with os.fdopen(descriptor, "wb") as stream:
                stream.write(content)
                stream.flush()
                os.fsync(stream.fileno())
            yield index_path
        finally:
            index_path.unlink(missing_ok=True)
            index_path.with_name(index_path.name + ".lock").unlink(missing_ok=True)

    @contextmanager
    def _index_publish_lock(
        self, index_path: Path, index_snapshot: bytes
    ) -> Iterator[Path]:
        lock_path = index_path.with_name(index_path.name + ".lock")
        try:
            descriptor = os.open(
                lock_path,
                os.O_CREAT | os.O_EXCL | os.O_WRONLY,
                0o600,
            )
        except FileExistsError as error:
            raise CoordinatorError(
                "isolated_patch_index_lock_occupied",
                "Shared Git index lock appeared before isolated publication",
                details={"lockPath": self._display_path(lock_path)},
            ) from error
        os.close(descriptor)
        try:
            current = index_path.read_bytes() if index_path.exists() else b""
            if current != index_snapshot:
                raise CoordinatorError(
                    "isolated_patch_shared_index_changed",
                    "Shared Git index changed before isolated publication",
                    details={
                        "expectedFingerprint": hashlib.sha256(
                            index_snapshot
                        ).hexdigest(),
                        "currentFingerprint": hashlib.sha256(current).hexdigest(),
                    },
                )
            yield lock_path
        finally:
            lock_path.unlink(missing_ok=True)

    def _apply_patch(self, environment: dict[str, str], patch: bytes) -> None:
        self._git_bytes(
            "apply",
            "--cached",
            "--whitespace=nowarn",
            "-",
            environment=environment,
            input_bytes=patch,
            error_code="isolated_patch_apply_failed",
        )

    def _staged_blob(self, environment: dict[str, str], target: str) -> str:
        raw = self._git_bytes(
            "ls-files", "--stage", "-z", "--", target, environment=environment
        )
        entries = [entry for entry in raw.split(b"\0") if entry]
        if len(entries) != 1 or b"\t" not in entries[0]:
            raise CoordinatorError(
                "isolated_patch_target_missing",
                "Isolated patch must retain one regular target file",
            )
        metadata, raw_path = entries[0].split(b"\t", 1)
        fields = metadata.decode("ascii", errors="replace").split()
        path = raw_path.decode("utf-8", errors="surrogateescape").replace("\\", "/")
        if len(fields) != 3 or fields[0] != "100644" or fields[2] != "0" or path != target:
            raise CoordinatorError(
                "isolated_patch_target_mode_changed",
                "Isolated patch may not rename or change the target file mode",
                details={"target": target, "entry": entries[0].decode("utf-8", errors="replace")},
            )
        return fields[1]

    def _validate_isolated_tree(
        self,
        index_environment: dict[str, str],
        commands: tuple[tuple[str, ...], ...],
        identity: dict[str, object],
    ) -> None:
        with tempfile.TemporaryDirectory(prefix="zircon-isolated-patch-") as directory:
            root = Path(directory).resolve()
            prefix = str(root) + os.sep
            self._git_text(
                "checkout-index",
                "--all",
                "--force",
                f"--prefix={prefix}",
                environment=index_environment,
            )
            environment = {
                key: os.environ[key]
                for key in VALIDATION_ENVIRONMENT_KEYS
                if key in os.environ
            }
            environment.update(
                {
                    "ZR_ISOLATED_PATCH_ROOT": str(root),
                    "ZR_ISOLATED_PATCH_TARGET": str(identity["target"]),
                    "ZR_ISOLATED_PATCH_BASE_HEAD": str(identity["baseHead"]),
                    "ZR_ISOLATED_PATCH_BASE_BLOB": str(identity["baseBlob"]),
                    "ZR_ISOLATED_PATCH_HASH": str(identity["patchHash"]),
                    "ZR_ISOLATED_PATCH_DERIVED_BLOB": str(identity["derivedBlob"]),
                }
            )
            for command in commands:
                try:
                    result = subprocess.run(
                        list(command), cwd=root, env=environment, check=False
                    )
                except OSError as error:
                    raise CoordinatorError(
                        "isolated_patch_validation_failed",
                        "Cannot start isolated maintenance validation command",
                        details={"command": list(command), "error": str(error)},
                    ) from error
                if result.returncode != 0:
                    raise CoordinatorError(
                        "isolated_patch_validation_failed",
                        "Isolated maintenance validation failed with exit code "
                        f"{result.returncode}",
                        details={"command": list(command), "exitCode": result.returncode},
                    )

    def _recover_index_lock(
        self, request_id: str, session_id: str, index_path: Path
    ) -> None:
        lock_path = index_path.with_name(index_path.name + ".lock")
        try:
            recovery = self.index_lock_recoverer(lock_path)
        except IndexLockRecoveryRefused as error:
            raise CoordinatorError(
                "isolated_patch_index_lock_recovery_refused",
                "Git index lock cannot be recovered safely for isolated finalize",
                details={"reason": error.reason, **error.details},
            ) from error
        if recovery is None:
            return
        payload = {
            "requestId": request_id,
            "lockPath": self._display_path(lock_path),
            **recovery.to_event_payload(),
        }
        self._record_event(session_id, "git.index_lock_recovered", payload)

    def _require_target_unstaged(
        self,
        parent_head: str,
        target: str,
        *,
        environment: dict[str, str] | None = None,
    ) -> None:
        changed = self._staged_paths(
            parent_head,
            environment=environment,
            paths=(target,),
        )
        if changed:
            raise CoordinatorError(
                "isolated_patch_target_staged",
                "Isolated patch target must not already be staged",
                details={"target": target},
            )

    def _require_no_secrets(
        self, target: str, patch: bytes, derived_blob: str
    ) -> None:
        derived = self._git_bytes("cat-file", "blob", derived_blob)
        allow_type_declaration = (
            Path(target).suffix.casefold() in _TYPE_DECLARATION_SOURCE_SUFFIXES
        )
        for content in (patch, derived):
            scanner = _StagedSecretStreamScanner(
                allow_type_declaration=allow_type_declaration
            )
            if scanner.feed(content) or scanner.finish():
                raise CoordinatorError(
                    "isolated_patch_secret_detected",
                    "Isolated patch content contains a maintenance capability or credential",
                )

    def _require_target_file(self, target: Path) -> None:
        if not target.is_file() or target.is_symlink():
            raise CoordinatorError(
                "isolated_patch_target_invalid",
                "Isolated patch target must be a regular worktree file",
            )

    def _require_lease(self, session_id: str, target: str) -> None:
        self.leases.require_owned_live(
            session_id,
            (target,),
            error_code="isolated_patch_lease_missing",
            message="Isolated patch target requires a live lease owned by the target Session",
        )

    def _tree_entry(self, treeish: str, target: str) -> tuple[str, str]:
        raw = self._git_bytes("ls-tree", "-z", treeish, "--", target)
        entries = [entry for entry in raw.split(b"\0") if entry]
        if len(entries) != 1 or b"\t" not in entries[0]:
            return "", ""
        metadata, raw_path = entries[0].split(b"\t", 1)
        fields = metadata.decode("ascii", errors="replace").split()
        path = raw_path.decode("utf-8", errors="surrogateescape").replace("\\", "/")
        if len(fields) != 3 or fields[1] != "blob" or path != target:
            return "", ""
        return fields[0], fields[2]

    def _staged_paths(
        self,
        head: str,
        *,
        environment: dict[str, str] | None = None,
        paths: tuple[str, ...] = (),
    ) -> tuple[str, ...]:
        arguments = ["diff", "--cached", "--name-only", "-z", "--no-renames", head, "--"]
        arguments.extend(paths)
        raw = self._git_bytes(*arguments, environment=environment)
        return tuple(
            item.decode("utf-8", errors="surrogateescape").replace("\\", "/")
            for item in raw.split(b"\0")
            if item
        )

    def _staged_projection(
        self,
        head: str,
        *,
        environment: dict[str, str] | None = None,
    ) -> bytes:
        return self._git_bytes(
            "diff",
            "--cached",
            "--binary",
            "--no-ext-diff",
            "--no-renames",
            head,
            "--",
            environment=environment,
        )

    @staticmethod
    def _paths_fingerprint(paths: tuple[str, ...]) -> str:
        return hashlib.sha256(
            b"\0".join(path.encode("utf-8", errors="surrogateescape") for path in paths)
        ).hexdigest()

    @staticmethod
    def _identity_payload(
        *,
        request_id: str,
        session_id: str,
        target: str,
        base_head: str,
        base_blob: str,
        parent_head: str,
        patch_hash: str,
        derived_blob: str,
        staged_paths: tuple[str, ...],
        staged_paths_fingerprint: str,
        staged_projection_fingerprint: str,
    ) -> dict[str, object]:
        return {
            "requestId": request_id,
            "sessionId": session_id,
            "target": target,
            "baseHead": base_head,
            "baseBlob": base_blob,
            "parentHead": parent_head,
            "patchHash": patch_hash,
            "derivedBlob": derived_blob,
            "stagedPathCount": len(staged_paths),
            "stagedPathsFingerprint": staged_paths_fingerprint,
            "stagedProjectionFingerprint": staged_projection_fingerprint,
        }

    def _record_event(
        self, session_id: str, event_type: str, payload: dict[str, object]
    ) -> None:
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO events(session_id, event_type, payload_json, created_at)
                VALUES (?, ?, ?, ?)
                """,
                (session_id, event_type, json.dumps(payload, sort_keys=True), utc_text()),
            )

    @contextmanager
    def _git_mutex(
        self,
        owner_id: str,
        *,
        retain_on_error: Callable[[], bool],
    ) -> Iterator[None]:
        try:
            with self.database.transaction() as connection:
                connection.execute(
                    """
                    INSERT INTO git_mutex(lock_name, owner_id, acquired_at)
                    VALUES ('index', ?, ?)
                    """,
                    (owner_id, utc_text()),
                )
        except IntegrityError as error:
            raise CoordinatorError(
                "git_mutex_occupied",
                "Another finalize operation owns the Git index mutex",
            ) from error
        failed = False
        try:
            yield
        except BaseException:
            failed = True
            raise
        finally:
            if not failed or not retain_on_error():
                with self.database.transaction() as connection:
                    connection.execute(
                        "DELETE FROM git_mutex WHERE lock_name='index' AND owner_id=?",
                        (owner_id,),
                    )

    def _index_path(self) -> Path:
        value = Path(self._git_text("rev-parse", "--git-path", "index"))
        return value if value.is_absolute() else self.repo_root / value

    @staticmethod
    def _replace_index(source: Path, target: Path) -> None:
        os.replace(source, target)

    def _is_ancestor(self, ancestor: str, descendant: str) -> bool:
        result = subprocess.run(
            ["git", "merge-base", "--is-ancestor", ancestor, descendant],
            cwd=self.repo_root,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=False,
        )
        if result.returncode in {0, 1}:
            return result.returncode == 0
        raise CoordinatorError(
            "isolated_patch_git_failed",
            "Git could not compare isolated patch ancestry",
        )

    def _git_text(
        self,
        *arguments: str,
        environment: dict[str, str] | None = None,
    ) -> str:
        return self._git_bytes(*arguments, environment=environment).decode(
            "utf-8", errors="replace"
        ).strip()

    def _git_bytes(
        self,
        *arguments: str,
        environment: dict[str, str] | None = None,
        input_bytes: bytes | None = None,
        error_code: str = "isolated_patch_git_failed",
    ) -> bytes:
        environment = environment or self._git_environment()
        try:
            result = subprocess.run(
                ["git", *arguments],
                cwd=self.repo_root,
                env=environment,
                input=input_bytes,
                check=True,
                capture_output=True,
            )
        except subprocess.CalledProcessError as error:
            raise CoordinatorError(
                error_code,
                f"Git {arguments[0]} failed with exit code {error.returncode}",
                details={
                    "command": f"git {arguments[0]}",
                    "exitCode": error.returncode,
                    "stderr": error.stderr.decode("utf-8", errors="replace")[:_GIT_ERROR_LIMIT],
                },
            ) from error
        except OSError as error:
            raise CoordinatorError(
                error_code,
                "Cannot start Git isolated patch command",
                details={"error": str(error)},
            ) from error
        return result.stdout

    @staticmethod
    def _git_environment(**overrides: str) -> dict[str, str]:
        environment = {**os.environ, "GIT_OPTIONAL_LOCKS": "0"}
        environment.update(overrides)
        return environment

    def _display_path(self, path: Path) -> str:
        try:
            return path.resolve().relative_to(self.repo_root).as_posix()
        except ValueError:
            return str(path.resolve())
