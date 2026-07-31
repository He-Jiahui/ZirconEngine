from __future__ import annotations

import hashlib
import json
import subprocess
import tarfile
from dataclasses import dataclass
from enum import StrEnum
from pathlib import Path
from sqlite3 import Row

from .database import Database
from .event_payloads import baseline_degraded_payload
from .models import CoordinatorError, utc_text


class BaselineHealth(StrEnum):
    HEALTHY = "healthy"
    DEGRADED = "degraded"


@dataclass(frozen=True, slots=True)
class BaselineEpoch:
    epoch_id: int
    head_commit: str
    index_tree: str
    health: BaselineHealth
    manifest: dict[str, str]
    degraded_reason: str | None


@dataclass(frozen=True, slots=True)
class WorkspaceChange:
    path: str
    kind: str
    baseline_hash: str | None
    current_hash: str | None


@dataclass(frozen=True, slots=True)
class PreparedWorkspaceScan:
    """Filesystem-heavy observation awaiting a short serialized apply step."""

    source_epoch_id: int
    observed_head: str
    baseline_manifest: dict[str, str]
    current_manifest: dict[str, str]


@dataclass(frozen=True, slots=True)
class WorkspaceScanResult:
    applied: bool
    changes: tuple[WorkspaceChange, ...]


def hash_bytes(content: bytes) -> str:
    return hashlib.sha256(content).hexdigest()


def hash_file(path: Path) -> str | None:
    if not path.is_file():
        return None
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


class BaselineService:
    def __init__(self, database: Database, repo_root: str | Path):
        self.database = database
        self.repo_root = Path(repo_root).resolve()
        self._workspace_capture_epochs: dict[int, bool] = {}

    def initialize(self) -> BaselineEpoch:
        try:
            return self.current()
        except CoordinatorError as error:
            if error.code != "baseline_missing":
                raise
        return self._capture(BaselineHealth.HEALTHY, reason="initial baseline")

    def current(self) -> BaselineEpoch:
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT * FROM baseline_epochs ORDER BY epoch_id DESC LIMIT 1"
            ).fetchone()
        if row is None:
            raise CoordinatorError("baseline_missing", "No workspace baseline has been initialized")
        return self._from_row(row)

    def refresh_for_head_change(self) -> BaselineEpoch:
        current = self.initialize()
        new_head = self._git_output("rev-parse", "HEAD")
        if current.head_commit == new_head:
            return current
        manifest = self._baseline_manifest_for_head(current, new_head)
        return self._capture(
            BaselineHealth.HEALTHY,
            reason="HEAD changed",
            manifest=dict(sorted(manifest.items(), key=lambda item: item[0].casefold())),
            head_commit=new_head,
        )

    def scan(self) -> list[WorkspaceChange]:
        return list(self.apply_scan(self.prepare_scan()).changes)

    def prepare_scan(self) -> PreparedWorkspaceScan:
        """Hash the shared workspace without holding the coordinator mutation lock."""
        baseline = self.initialize()
        observed_head = self._git_output("rev-parse", "HEAD")
        if (
            baseline.health is BaselineHealth.DEGRADED
            and baseline.head_commit == observed_head
        ):
            # A degraded baseline remains strict until an explicit reconcile or
            # acceptance.  Rehashing every file each maintenance interval cannot
            # make it healthy and can starve Session writes in a large dirty tree.
            return PreparedWorkspaceScan(
                source_epoch_id=baseline.epoch_id,
                observed_head=observed_head,
                baseline_manifest=baseline.manifest,
                current_manifest=baseline.manifest,
            )
        baseline_manifest = self._baseline_manifest_for_head(baseline, observed_head)
        return PreparedWorkspaceScan(
            source_epoch_id=baseline.epoch_id,
            observed_head=observed_head,
            baseline_manifest=baseline_manifest,
            current_manifest=self._workspace_manifest_from_baseline(
                baseline_manifest, observed_head
            ),
        )

    def apply_scan(self, observation: PreparedWorkspaceScan) -> WorkspaceScanResult:
        """Apply an observation only if its baseline epoch and HEAD are still current."""
        baseline = self.initialize()
        current_head = self._git_output("rev-parse", "HEAD")
        if (
            baseline.epoch_id != observation.source_epoch_id
            or current_head != observation.observed_head
        ):
            return WorkspaceScanResult(False, ())
        if baseline.head_commit != observation.observed_head:
            baseline = self._capture(
                BaselineHealth.HEALTHY,
                reason="HEAD changed",
                manifest=observation.baseline_manifest,
                head_commit=observation.observed_head,
            )
        else:
            baseline = self._repair_stale_commit_manifest(
                baseline, observation.baseline_manifest
            )
        changes = self._unattributed_changes(
            self._compare(observation.baseline_manifest, observation.current_manifest),
            baseline_epoch=baseline.epoch_id,
        )
        if changes:
            self._mark_degraded(baseline.epoch_id, changes)
        return WorkspaceScanResult(True, tuple(changes))

    def reconcile_health(self) -> BaselineEpoch:
        """Restore health only when every workspace difference is attributed.

        Unlike ``accept``, reconciliation never captures the current worktree and
        never creates a new epoch.  It only clears a stale degraded marker after
        exact content-hash attribution has proved that all remaining differences
        belong to registered Sessions.
        """
        baseline = self.initialize()
        baseline = self._repair_stale_commit_manifest(
            baseline,
            self._baseline_manifest_for_head(baseline, baseline.head_commit),
        )
        first_manifest = self._workspace_manifest_from_baseline(
            baseline.manifest, baseline.head_commit
        )
        changes = self._unattributed_changes(
            self._compare(baseline.manifest, first_manifest),
            baseline_epoch=baseline.epoch_id,
        )
        if changes:
            raise CoordinatorError(
                "baseline_unattributed_changes",
                "Baseline cannot be reconciled while unattributed workspace changes remain",
                details={"paths": [item.path for item in changes]},
            )
        second_manifest = self._workspace_manifest_from_baseline(
            baseline.manifest, baseline.head_commit
        )
        if second_manifest != first_manifest:
            raise CoordinatorError(
                "baseline_workspace_changing",
                "Baseline cannot be reconciled while the workspace is changing",
            )
        if baseline.health is BaselineHealth.HEALTHY:
            return baseline
        now = utc_text()
        with self.database.transaction() as connection:
            connection.execute(
                """
                UPDATE baseline_epochs
                SET health = ?, degraded_at = NULL, degraded_reason = NULL
                WHERE epoch_id = ?
                """,
                (BaselineHealth.HEALTHY.value, baseline.epoch_id),
            )
            connection.execute(
                "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, ?)",
                (
                    "baseline.reconciled",
                    json.dumps({"epoch_id": baseline.epoch_id}, sort_keys=True),
                    now,
                ),
            )
        reconciled = self.current()
        post_changes = self._unattributed_changes(
            self._compare(
                baseline.manifest,
                self._workspace_manifest_from_baseline(
                    baseline.manifest, baseline.head_commit
                ),
            ),
            baseline_epoch=baseline.epoch_id,
        )
        if post_changes:
            self._mark_degraded(baseline.epoch_id, post_changes)
            raise CoordinatorError(
                "baseline_unattributed_changes",
                "Workspace changed while baseline health was being reconciled",
                details={"paths": [item.path for item in post_changes]},
            )
        return reconciled

    def _unattributed_changes(
        self, changes: list[WorkspaceChange], *, baseline_epoch: int
    ) -> list[WorkspaceChange]:
        if not changes:
            return []
        path_keys = [change.path.casefold() for change in changes]
        placeholders = ",".join("?" for _ in path_keys)
        with self.database.connect() as connection:
            rows = connection.execute(
                f"""
                SELECT attributions.path_key, attributions.content_hash,
                       attributions.baseline_epoch, sessions.status
                FROM attributions
                JOIN sessions ON sessions.session_id = attributions.session_id
                WHERE attributions.path_key IN ({placeholders})
                """,
                tuple(path_keys),
            ).fetchall()
        attributed_hashes = {
            row["path_key"]: row["content_hash"]
            for row in rows
            if row["baseline_epoch"] is not None
            and int(row["baseline_epoch"]) == baseline_epoch
            and row["status"] not in {"stale", "archived", "cancelled"}
        }
        return [
            change
            for change in changes
            if change.path.casefold() not in attributed_hashes
            or attributed_hashes[change.path.casefold()] != change.current_hash
        ]

    def _mark_degraded(
        self, epoch_id: int, changes: list[WorkspaceChange]
    ) -> None:
        now = utc_text()
        with self.database.transaction() as connection:
            transitioned = connection.execute(
                """
                UPDATE baseline_epochs
                SET health = ?, degraded_at = ?, degraded_reason = ?
                WHERE epoch_id = ? AND health != ?
                """,
                (
                    BaselineHealth.DEGRADED.value,
                    now,
                    f"{len(changes)} unaccepted workspace change(s)",
                    epoch_id,
                    BaselineHealth.DEGRADED.value,
                ),
            ).rowcount
            if transitioned == 0:
                return
            connection.execute(
                "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, ?)",
                (
                    "baseline.degraded",
                    json.dumps(
                        baseline_degraded_payload(
                            epoch_id,
                            [item.path for item in changes],
                        ),
                        sort_keys=True,
                    ),
                    now,
                ),
            )

    def diff(self) -> list[WorkspaceChange]:
        baseline = self.initialize()
        reference_manifest = self._baseline_manifest_for_head(
            baseline, baseline.head_commit
        )
        return self._compare(
            reference_manifest,
            self._workspace_manifest_from_baseline(
                reference_manifest, baseline.head_commit
            ),
        )

    def attribute(self, session_id: str, paths: list[str] | tuple[str, ...]) -> None:
        baseline = self.initialize()
        now = utc_text()
        with self.database.transaction() as connection:
            if connection.execute(
                "SELECT 1 FROM sessions WHERE session_id = ?", (session_id,)
            ).fetchone() is None:
                raise CoordinatorError("session_not_found", f"Unknown Session {session_id}")
            for display_path in paths:
                normalized = self._normalize_repo_path(display_path)
                connection.execute(
                    """
                    INSERT INTO attributions(
                        path_key, display_path, session_id, baseline_epoch, content_hash, attributed_at
                    ) VALUES (?, ?, ?, ?, ?, ?)
                    ON CONFLICT(path_key) DO UPDATE SET
                        display_path = excluded.display_path,
                        session_id = excluded.session_id,
                        baseline_epoch = excluded.baseline_epoch,
                        content_hash = excluded.content_hash,
                        attributed_at = excluded.attributed_at
                    """,
                    (
                        normalized.casefold(),
                        normalized,
                        session_id,
                        baseline.epoch_id,
                        hash_file(self.repo_root / normalized),
                        now,
                    ),
                )

    def accept(self, *, reason: str) -> BaselineEpoch:
        if not reason.strip():
            raise ValueError("baseline acceptance requires a reason")
        return self._capture(
            BaselineHealth.HEALTHY,
            reason=reason,
            capture_mode="workspace",
        )

    def accept_commit(
        self,
        committed_paths: list[str] | tuple[str, ...],
        *,
        commit_sha: str,
        reason: str,
    ) -> BaselineEpoch:
        """Advance HEAD while preserving unrelated dirty paths as baseline differences."""
        if not reason.strip():
            raise ValueError("baseline acceptance requires a reason")
        baseline = self.current()
        # Shared main can advance through a managed commit whose exact Session
        # manifest is only a subset of the files committed at that HEAD.  The
        # baseline must nevertheless reflect *all* tracked files from the new
        # commit; otherwise every omitted committed path is misclassified as an
        # unattributed workspace change.  Dirty worktree bytes remain distinct
        # because this manifest comes from Git, not the live checkout.
        manifest = self._baseline_manifest_for_head(baseline, commit_sha)
        return self._capture(
            BaselineHealth.HEALTHY,
            reason=reason,
            manifest=dict(sorted(manifest.items(), key=lambda item: item[0].casefold())),
            head_commit=commit_sha,
        )

    def build_manifest(self) -> dict[str, str]:
        result = subprocess.run(
            ["git", "ls-files", "-z", "--cached", "--others", "--exclude-standard"],
            cwd=self.repo_root,
            check=True,
            capture_output=True,
        )
        manifest: dict[str, str] = {}
        for raw_path in result.stdout.split(b"\0"):
            if not raw_path:
                continue
            display_path = raw_path.decode("utf-8", errors="surrogateescape").replace("\\", "/")
            if display_path == ".codex/state" or display_path.startswith(".codex/state/"):
                continue
            content_hash = hash_file(self.repo_root / display_path)
            if content_hash is not None:
                manifest[display_path] = content_hash
        return dict(sorted(manifest.items(), key=lambda item: item[0].casefold()))

    def _workspace_manifest_from_baseline(
        self, baseline_manifest: dict[str, str], head_commit: str
    ) -> dict[str, str]:
        """Rehash only paths Git reports as live workspace candidates.

        A full manifest is required to create or explicitly accept a baseline.
        Normal scans, diffs, and reconciliation already have a complete commit
        manifest, so hashing every unchanged tracked file needlessly serializes
        a large shared checkout.  Git identifies modified/deleted tracked paths
        relative to the reference commit; all current and previously-baselined
        untracked paths are also rehashed to preserve exact content checks.
        """
        manifest = dict(baseline_manifest)
        tracked_paths = self._tracked_paths(head_commit)
        candidates = self._git_path_set(
            "diff", "--name-only", "-z", "--no-ext-diff", head_commit
        )
        candidates.update(
            self._git_path_set("ls-files", "--others", "--exclude-standard", "-z")
        )
        candidates.update(path for path in baseline_manifest if path not in tracked_paths)
        for display_path in candidates:
            if display_path == ".codex/state" or display_path.startswith(".codex/state/"):
                continue
            content_hash = hash_file(self.repo_root / display_path)
            if content_hash is None:
                manifest.pop(display_path, None)
            else:
                manifest[display_path] = content_hash
        return dict(sorted(manifest.items(), key=lambda item: item[0].casefold()))

    def _repair_stale_commit_manifest(
        self, baseline: BaselineEpoch, commit_manifest: dict[str, str]
    ) -> BaselineEpoch:
        """Repair only a same-HEAD manifest that drifted from committed bytes.

        A historical scoped ``accept_commit`` could save an old hash for a path
        that was nevertheless included in the shared commit.  The replacement
        manifest is derived exclusively from that same pinned commit plus the
        epoch's existing untracked entries; no live worktree byte is accepted.
        """
        if baseline.manifest == commit_manifest:
            return baseline
        now = utc_text()
        payload = json.dumps(commit_manifest, sort_keys=True)
        with self.database.transaction() as connection:
            updated = connection.execute(
                """
                UPDATE baseline_epochs
                SET manifest_json = ?
                WHERE epoch_id = ? AND head_commit = ?
                """,
                (payload, baseline.epoch_id, baseline.head_commit),
            ).rowcount
            if updated == 0:
                raise CoordinatorError(
                    "baseline_epoch_changed",
                    "Baseline changed while repairing its commit manifest",
                )
            connection.execute(
                "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, ?)",
                (
                    "baseline.commit_manifest_repaired",
                    json.dumps(
                        {
                            "epoch_id": baseline.epoch_id,
                            "head_commit": baseline.head_commit,
                        },
                        sort_keys=True,
                    ),
                    now,
                ),
            )
        return self.current()

    def _capture(
        self,
        health: BaselineHealth,
        *,
        reason: str,
        manifest: dict[str, str] | None = None,
        head_commit: str | None = None,
        capture_mode: str = "commit",
    ) -> BaselineEpoch:
        manifest = self.build_manifest() if manifest is None else manifest
        now = utc_text()
        with self.database.transaction() as connection:
            cursor = connection.execute(
                """
                INSERT INTO baseline_epochs(
                    head_commit, index_tree, health, manifest_json, created_at, degraded_reason
                ) VALUES (?, ?, ?, ?, ?, ?)
                """,
                (
                    head_commit or self._git_output("rev-parse", "HEAD"),
                    self._git_output("write-tree"),
                    health.value,
                    json.dumps(manifest, sort_keys=True),
                    now,
                    None if health is BaselineHealth.HEALTHY else reason,
                ),
            )
            epoch_id = int(cursor.lastrowid)
            connection.execute(
                "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, ?)",
                (
                    "baseline.created",
                    json.dumps(
                        {
                            "capture_mode": capture_mode,
                            "epoch_id": epoch_id,
                            "reason": reason,
                        },
                        sort_keys=True,
                    ),
                    now,
                ),
            )
        self._workspace_capture_epochs[epoch_id] = capture_mode == "workspace"
        return self.current()

    def _is_workspace_capture_epoch(self, epoch_id: int) -> bool:
        if epoch_id in self._workspace_capture_epochs:
            return self._workspace_capture_epochs[epoch_id]
        with self.database.connect() as connection:
            row = connection.execute(
                """
                SELECT payload_json FROM events
                WHERE event_type = ? AND payload_json LIKE ?
                ORDER BY event_id DESC
                LIMIT 1
                """,
                ("baseline.created", f'%"epoch_id": {epoch_id},%'),
            ).fetchone()
        if row is None:
            self._workspace_capture_epochs[epoch_id] = False
            return False
        try:
            payload = json.loads(row["payload_json"])
        except (TypeError, ValueError):
            payload = {}
        is_workspace_capture = payload.get("capture_mode") == "workspace"
        self._workspace_capture_epochs[epoch_id] = is_workspace_capture
        return is_workspace_capture

    def _git_output(self, *arguments: str) -> str:
        result = subprocess.run(
            ["git", *arguments],
            cwd=self.repo_root,
            check=True,
            capture_output=True,
            text=True,
        )
        return result.stdout.strip()

    def _git_path_set(self, *arguments: str) -> set[str]:
        result = subprocess.run(
            ["git", *arguments],
            cwd=self.repo_root,
            check=True,
            capture_output=True,
        )
        return {
            raw.decode("utf-8", errors="surrogateescape").replace("\\", "/")
            for raw in result.stdout.split(b"\0")
            if raw
        }

    def _tracked_paths(self, commit: str) -> set[str]:
        result = subprocess.run(
            ["git", "ls-tree", "-rz", "--name-only", commit],
            cwd=self.repo_root,
            check=True,
            capture_output=True,
        )
        return {
            raw.decode("utf-8", errors="surrogateescape").replace("\\", "/")
            for raw in result.stdout.split(b"\0")
            if raw
        }

    def _baseline_manifest_for_head(
        self, baseline: BaselineEpoch, new_head: str
    ) -> dict[str, str]:
        if (
            new_head == baseline.head_commit
            and self._is_workspace_capture_epoch(baseline.epoch_id)
        ):
            # ``accept`` deliberately captures the live worktree. Rebuilding
            # from the same Git tree would discard those accepted bytes and
            # immediately report them as new workspace drift.
            return baseline.manifest

        old_tracked = self._tracked_paths(baseline.head_commit)
        changes = self._commit_path_changes(baseline.head_commit, new_head)
        filters_changed = any(
            path.rsplit("/", 1)[-1] == ".gitattributes"
            for _status, paths in changes
            for path in paths
        )
        if new_head == baseline.head_commit or filters_changed or any(
            path not in baseline.manifest for path in old_tracked
        ):
            # Historical partial baselines cannot be advanced from a delta without
            # falsely treating an unchanged tracked path as absent. A non-workspace
            # epoch at the same HEAD is repaired from its pinned Git tree.
            # Rebuild once from the pinned commit, then subsequent managed
            # commits stay incremental.
            manifest = {
                path: content_hash
                for path, content_hash in baseline.manifest.items()
                if path not in old_tracked
            }
            manifest.update(self._commit_manifest(new_head))
            return dict(sorted(manifest.items(), key=lambda item: item[0].casefold()))

        # Preserve accepted untracked entries. Tracked paths are advanced from
        # the pinned Git delta, never from dirty checkout bytes.
        manifest = {
            path: content_hash
            for path, content_hash in baseline.manifest.items()
            if path not in old_tracked
        }
        manifest.update(
            {
                path: baseline.manifest[path]
                for path in old_tracked
            }
        )
        for status, paths in changes:
            if status == "D":
                manifest.pop(paths[0], None)
                continue
            path = paths[-1]
            manifest[path] = self._filtered_commit_blob_hash(new_head, path)
        return dict(sorted(manifest.items(), key=lambda item: item[0].casefold()))

    def _commit_path_changes(
        self, old_commit: str, new_commit: str
    ) -> list[tuple[str, tuple[str, ...]]]:
        """Return a no-rename recursive tree delta with exact repository paths."""
        result = subprocess.run(
            [
                "git",
                "diff-tree",
                "-r",
                "--name-status",
                "-z",
                "--no-renames",
                old_commit,
                new_commit,
            ],
            cwd=self.repo_root,
            check=True,
            capture_output=True,
        )
        fields = [item for item in result.stdout.split(b"\0") if item]
        changes: list[tuple[str, tuple[str, ...]]] = []
        index = 0
        while index < len(fields):
            status = fields[index].decode("ascii", errors="strict")
            index += 1
            if not status:
                raise CoordinatorError(
                    "baseline_commit_delta_invalid",
                    "Pinned Git tree delta contains an empty status",
                )
            path_count = 2 if status[0] in {"R", "C"} else 1
            if index + path_count > len(fields):
                raise CoordinatorError(
                    "baseline_commit_delta_invalid",
                    "Pinned Git tree delta ended before all paths were decoded",
                )
            paths = tuple(
                field.decode("utf-8", errors="surrogateescape").replace("\\", "/")
                for field in fields[index : index + path_count]
            )
            index += path_count
            changes.append((status[0], paths))
        return changes

    def _filtered_commit_blob_hash(self, commit: str, path: str) -> str:
        result = subprocess.run(
            ["git", "cat-file", "--filters", f"--path={path}", f"{commit}:{path}"],
            cwd=self.repo_root,
            check=True,
            capture_output=True,
        )
        return hash_bytes(result.stdout)

    def _commit_manifest(self, commit: str) -> dict[str, str]:
        manifest: dict[str, str] = {}
        process = subprocess.Popen(
            ["git", "archive", "--format=tar", commit],
            cwd=self.repo_root,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        stderr = b""
        try:
            if process.stdout is None:
                raise CoordinatorError(
                    "baseline_commit_archive_failed",
                    "Pinned baseline archive did not provide a readable stream",
                )
            with tarfile.open(fileobj=process.stdout, mode="r|") as archive:
                for member in archive:
                    if member.isdir():
                        continue
                    path = member.name.replace("\\", "/")
                    if member.issym():
                        manifest[path] = hash_bytes(member.linkname.encode("utf-8"))
                        continue
                    if not member.isfile():
                        continue
                    source = archive.extractfile(member)
                    if source is None:
                        raise CoordinatorError(
                            "baseline_commit_archive_invalid",
                            f"Pinned baseline archive contains unreadable path: {path}",
                        )
                    with source:
                        digest = hashlib.sha256()
                        for chunk in iter(lambda: source.read(1024 * 1024), b""):
                            digest.update(chunk)
                    manifest[path] = digest.hexdigest()
        except BaseException:
            if process.poll() is None:
                process.kill()
            raise
        finally:
            if process.stdout is not None:
                process.stdout.close()
            if process.stderr is not None:
                stderr = process.stderr.read()
                process.stderr.close()
            process.wait()
        if process.returncode != 0:
            raise CoordinatorError(
                "baseline_commit_archive_failed",
                "Could not build a baseline manifest from the pinned Git commit",
                details={"stderr": stderr.decode("utf-8", errors="replace")[-4096:]},
            )
        return manifest

    def _normalize_repo_path(self, value: str) -> str:
        candidate = (self.repo_root / value).resolve()
        try:
            relative = candidate.relative_to(self.repo_root)
        except ValueError as error:
            raise CoordinatorError("path_outside_repo", f"Path is outside repository: {value}") from error
        return relative.as_posix()

    @staticmethod
    def _compare(
        baseline: dict[str, str], current: dict[str, str]
    ) -> list[WorkspaceChange]:
        changes: list[WorkspaceChange] = []
        for path in sorted(set(baseline) | set(current), key=str.casefold):
            old_hash = baseline.get(path)
            new_hash = current.get(path)
            if old_hash == new_hash:
                continue
            kind = "added" if old_hash is None else "deleted" if new_hash is None else "modified"
            changes.append(WorkspaceChange(path, kind, old_hash, new_hash))
        return changes

    @staticmethod
    def _from_row(row: Row) -> BaselineEpoch:
        return BaselineEpoch(
            epoch_id=int(row["epoch_id"]),
            head_commit=row["head_commit"],
            index_tree=row["index_tree"],
            health=BaselineHealth(row["health"]),
            manifest=json.loads(row["manifest_json"]),
            degraded_reason=row["degraded_reason"],
        )
