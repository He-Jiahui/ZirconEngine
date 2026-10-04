from __future__ import annotations

import hashlib
import json
import os
import subprocess
import tempfile
import uuid
from dataclasses import dataclass, field
from enum import StrEnum
from pathlib import Path
from typing import Any, Mapping, Protocol

from tools.jenkins.contracts import JenkinsError, canonical_json, digest, file_digest


class StateLike(Protocol):
    def put_operation(self, kind: str, operation_id: str, payload: dict,
                      status: str = "prepared") -> Any: ...
    def transition_operation(self, operation_id: str, expected_status: str,
                             status: str, payload: dict) -> Any: ...
    def get_operation(self, operation_id: str) -> Any: ...


class JournalState(StrEnum):
    PREPARED = "prepared"
    REF_UPDATED = "ref_updated"
    INDEX_PENDING = "index_pending"
    COMPLETE = "complete"
    BLOCKED = "blocked"
    UNKNOWN = "unknown"


@dataclass(frozen=True)
class CandidatePath:
    path: str
    blob_oid: str | None
    mode: str = "100644"
    before_digest: str | None = None
    after_digest: str | None = None
    deleted: bool = False


@dataclass(frozen=True)
class CandidateManifest:
    candidate_id: str
    repository_root: str
    base_head: str
    paths: tuple[CandidatePath, ...]
    source_digest: str
    coverage_digest: str
    acceptance_receipts: tuple[str, ...] = ()
    coauthor_map: Mapping[str, str] = field(default_factory=dict)

    @property
    def digest(self) -> str:
        return digest({"candidateId": self.candidate_id, "baseHead": self.base_head,
                       "paths": [p.__dict__ for p in self.paths],
                       "sourceDigest": self.source_digest,
                       "coverageDigest": self.coverage_digest,
                       "acceptanceReceipts": list(self.acceptance_receipts),
                       "coauthorMap": dict(self.coauthor_map)})


@dataclass(frozen=True)
class IntegrationPlan:
    operation_id: str
    expected_old_head: str
    planned_tree: str
    planned_commit: str
    index_before: str
    index_after: str
    candidates: tuple[str, ...]
    authorization_digest: str
    owned_paths: tuple[str, ...] = ()


@dataclass(frozen=True)
class IntegrationReceipt:
    operation_id: str
    status: str
    commit_sha: str | None
    tree_sha: str | None
    head_before: str
    head_after: str | None
    index_before: str
    index_after: str | None
    candidate_ids: tuple[str, ...]
    journal_state: str
    details: Mapping[str, Any] = field(default_factory=dict)


def _norm(path: str) -> str:
    path = path.replace("\\", "/").lstrip("./")
    if not path or path.startswith("/") or "\x00" in path or any(x == ".." for x in path.split("/")):
        raise JenkinsError("invalid_owned_path", f"Invalid owned path: {path!r}")
    return path


class GitRepository:
    def __init__(self, root: str | Path, *, state: StateLike | None = None):
        self.root = Path(root).resolve()
        if not (self.root / ".git").exists():
            raise JenkinsError("git_repository_missing", "Git repository is missing", details={"root": str(self.root)})
        self.state = state

    def _git(self, *args: str, env: Mapping[str, str] | None = None, check: bool = True) -> str:
        merged = os.environ.copy()
        if env:
            merged.update(env)
        try:
            result = subprocess.run(["git", *args], cwd=self.root, env=merged,
                                    check=check, capture_output=True, text=True)
        except (OSError, subprocess.CalledProcessError) as exc:
            stderr = getattr(exc, "stderr", "") or ""
            raise JenkinsError("git_command_failed", f"Git command failed: git {' '.join(args)}",
                               retryable=False, details={"stderr": stderr[-2000:]}) from exc
        return result.stdout.strip()

    def head(self) -> str:
        return self._git("rev-parse", "HEAD")

    def index_path(self) -> Path:
        p = Path(self._git("rev-parse", "--git-path", "index"))
        return p if p.is_absolute() else (self.root / p)

    def index_digest(self) -> str:
        p = self.index_path()
        return file_digest(p) if p.exists() else hashlib.sha256(b"").hexdigest()

    def _index_entries(self) -> dict[str, tuple[str, str, int]]:
        raw = subprocess.run(["git", "ls-files", "--stage", "-z"], cwd=self.root,
                             check=True, capture_output=True).stdout
        entries: dict[str, tuple[str, str, int]] = {}
        for item in raw.split(b"\0"):
            if not item:
                continue
            header, path = item.split(b"\t", 1)
            mode, oid, stage = header.decode().split()
            entries[path.decode()] = (mode, oid, int(stage))
        return entries

    def _is_ancestor(self, ancestor: str, descendant: str) -> bool:
        result = subprocess.run(["git", "merge-base", "--is-ancestor", ancestor, descendant],
                                cwd=self.root, capture_output=True)
        return result.returncode == 0

    def _state_authorization(self, authorization: Mapping[str, Any] | str | None,
                             paths: list[str]) -> dict[str, Any]:
        """Resolve commit authority from State, ignoring blank/raw parameters.

        Jenkins parameter binding can deliver ``COMMIT_AUTHORIZATION`` as an
        empty string.  That value is only a transport hint; the authoritative
        scope is the immutable State authorization snapshot.
        """
        if self.state is None:
            if not isinstance(authorization, Mapping):
                raise JenkinsError("authorization_missing", "Commit authorization is required")
            return dict(authorization)
        supplied = authorization if isinstance(authorization, Mapping) else {}
        repository_id = supplied.get("repositoryId") or supplied.get("repository_id")
        session_id = supplied.get("sessionId") or supplied.get("session_id")
        if not repository_id or not session_id:
            raise JenkinsError("authorization_missing", "State-backed commit authorization identity is required")
        snapshot = self.state.check_authorization(str(repository_id), str(session_id), "commit", paths)
        return {"repositoryId": str(repository_id), "sessionId": str(session_id), **snapshot}

    def _check_paths(self, candidates: tuple[CandidateManifest, ...]) -> None:
        owned: set[str] = set()
        for c in candidates:
            for p in c.paths:
                path = _norm(p.path)
                for prior in owned:
                    if path == prior or path.startswith(prior + "/") or prior.startswith(path + "/"):
                        raise JenkinsError("owned_path_overlap", "Candidate paths overlap", details={"path": path, "prior": prior})
                owned.add(path)
        entries = self._index_entries()
        for path in owned:
            head_oid = self._git("rev-parse", f"HEAD:{path}", check=False) or None
            if path in entries and (entries[path][2] != 0 or entries[path][1] != head_oid):
                raise JenkinsError("foreign_index_entry", "Owned path has foreign staged or unmerged entry",
                                   details={"path": path, "stage": entries[path][2], "headOid": head_oid,
                                            "indexOid": entries[path][1]})

    def seal_candidate(self, paths: Mapping[str, bytes | None], *, source_digest: str,
                       coverage_digest: str, acceptance_receipts: tuple[str, ...] = (),
                       candidate_id: str | None = None) -> CandidateManifest:
        if self.state and acceptance_receipts:
            for receipt_id in acceptance_receipts:
                record = self.state.get("validation_acceptance", receipt_id)
                if record is None:
                    record = self.state.get("acceptance", receipt_id)
                if record is None:
                    raise JenkinsError("acceptance_missing", "Acceptance receipt is not registered", details={"receiptId": receipt_id})
                value = record.get("payload", record)
                if value.get("status") not in {"accepted", "passed"}:
                    raise JenkinsError("acceptance_not_accepted", "Acceptance receipt is not terminal", details={"receiptId": receipt_id})
                if value.get("sourceDigest") and value["sourceDigest"] != source_digest:
                    raise JenkinsError("acceptance_source_mismatch", "Acceptance source digest does not match candidate")
                if value.get("coverageDigest") and value["coverageDigest"] != coverage_digest:
                    raise JenkinsError("acceptance_coverage_mismatch", "Acceptance coverage digest does not match candidate")
        base = self.head()
        items: list[CandidatePath] = []
        for raw_path, content in sorted(paths.items(), key=lambda item: item[0].casefold()):
            path = _norm(raw_path)
            before_oid = self._git("rev-parse", f"HEAD:{path}", check=False) or None
            before = None
            if before_oid:
                before_bytes = subprocess.run(["git", "show", f"HEAD:{path}"], cwd=self.root,
                                              capture_output=True, check=True).stdout
                before = hashlib.sha256(before_bytes).hexdigest()
            if content is None:
                items.append(CandidatePath(path, None, before_digest=None, after_digest=None, deleted=True))
                continue
            oid = subprocess.run(["git", "hash-object", "-w", "--stdin"], cwd=self.root,
                                  input=content, capture_output=True, check=True).stdout.decode().strip()
            items.append(CandidatePath(path, oid, "100644", before, hashlib.sha256(content).hexdigest()))
        return CandidateManifest(candidate_id or uuid.uuid4().hex, str(self.root), base,
                                 tuple(items), source_digest, coverage_digest, acceptance_receipts)

    def _tree(self, head: str, paths: tuple[CandidatePath, ...]) -> tuple[str, str]:
        git_dir = Path(self._git("rev-parse", "--git-dir"))
        if not git_dir.is_absolute():
            git_dir = self.root / git_dir
        # Keep all transient indexes in the repository's private Jenkins area;
        # never replace the shared index while constructing a tree.
        tmp_dir = self.root / ".jenkins" / "tmp"
        tmp_dir.mkdir(parents=True, exist_ok=True)
        with tempfile.NamedTemporaryFile(prefix="jenkins-index-", dir=tmp_dir, delete=False) as tmp:
            idx = Path(tmp.name)
        idx.unlink(missing_ok=True)
        env = {"GIT_INDEX_FILE": str(idx)}
        try:
            self._git("read-tree", head, env=env)
            for p in paths:
                if p.deleted:
                    self._git("update-index", "--remove", "--", p.path, env=env)
                else:
                    valid = bool(p.blob_oid) and subprocess.run(["git", "cat-file", "-e", str(p.blob_oid)], cwd=self.root, capture_output=True).returncode == 0
                    if not valid:
                        raise JenkinsError("sealed_object_invalid", "Sealed blob object is missing", details={"path": p.path, "oid": p.blob_oid})
                    self._git("update-index", "--add", "--cacheinfo", p.mode, p.blob_oid, p.path, env=env)
            tree = self._git("write-tree", env=env)
            return tree, file_digest(idx) if idx.exists() else hashlib.sha256(b"").hexdigest()
        finally:
            idx.unlink(missing_ok=True)
            idx.with_name(idx.name + ".lock").unlink(missing_ok=True)

    def plan(self, candidates: tuple[CandidateManifest, ...], *, authorization: Mapping[str, Any] | str,
             message: str, operation_id: str | None = None) -> IntegrationPlan:
        if not candidates:
            raise JenkinsError("empty_integration", "At least one candidate is required")
        self._check_paths(candidates)
        old = self.head()
        for c in candidates:
            if c.base_head != old and not self._is_ancestor(c.base_head, old):
                raise JenkinsError("head_drift", "Candidate base is not an ancestor of current HEAD", retryable=True)
            if not c.acceptance_receipts:
                raise JenkinsError("acceptance_missing", "Candidate has no authoritative acceptance receipt")
        paths = tuple(p for c in candidates for p in c.paths)
        authorization = self._state_authorization(authorization, [p.path for p in paths])
        tree, index_after = self._tree(old, paths)
        commit = self._git("commit-tree", tree, "-p", old, "-m", message)
        op = operation_id or uuid.uuid4().hex
        plan = IntegrationPlan(op, old, tree, commit, self.index_digest(), "",
                               tuple(c.candidate_id for c in candidates), digest(dict(authorization)),
                               tuple(p.path for p in paths))
        if self.state:
            self.state.put_operation("git_integration", op, {"plan": plan.__dict__, "authorization": dict(authorization)})
        return plan

    def publish(self, plan: IntegrationPlan, *, authorization: Mapping[str, Any] | str) -> IntegrationReceipt:
        authorization = self._state_authorization(authorization, list(plan.owned_paths))
        if digest(dict(authorization)) != plan.authorization_digest:
            raise JenkinsError("authorization_mismatch", "Integration authorization does not match plan")
        current = self.head()
        if current != plan.expected_old_head:
            raise JenkinsError("head_cas_failed", "HEAD changed before integration", retryable=True,
                               details={"expected": plan.expected_old_head, "actual": current})
        try:
            self._git("update-ref", "HEAD", plan.planned_commit, plan.expected_old_head)
        except JenkinsError:
            raise JenkinsError("head_cas_failed", "HEAD update failed", retryable=True)
        ref_head = self.head()
        if ref_head != plan.planned_commit:
            raise JenkinsError("head_reconcile_failed", "HEAD does not contain planned commit")
        self._align_index(plan)
        index_after = self.index_digest()
        receipt = IntegrationReceipt(plan.operation_id, "ref_updated", plan.planned_commit, plan.planned_tree,
                                     plan.expected_old_head, ref_head, plan.index_before, index_after,
                                     plan.candidates, JournalState.REF_UPDATED.value)
        if self.state:
            self.state.transition_operation(plan.operation_id, JournalState.PREPARED.value,
                                            JournalState.REF_UPDATED.value, receipt.__dict__)
        return receipt

    def _align_index(self, plan: IntegrationPlan) -> None:
        """Update only owned index entries, preserving foreign staging."""
        for path in plan.owned_paths:
            mode_oid = self._git("ls-tree", plan.planned_commit, "--", path)
            if not mode_oid:
                self._git("update-index", "--remove", "--", path)
                continue
            header = mode_oid.split("\t", 1)[0].split()
            mode, oid = header[0], header[2]
            self._git("update-index", "--add", "--cacheinfo", mode, oid, path)

    def reconcile(self, plan: IntegrationPlan) -> IntegrationReceipt:
        head = self.head()
        index = self.index_digest()
        if head == plan.expected_old_head:
            return IntegrationReceipt(plan.operation_id, "not_started", None, plan.planned_tree,
                                      head, None, plan.index_before, index, plan.candidates,
                                      JournalState.PREPARED.value)
        if head == plan.planned_commit:
            status = "complete" if self._owned_index_matches(plan) else "index_pending"
            return IntegrationReceipt(plan.operation_id, status, plan.planned_commit, plan.planned_tree,
                                      plan.expected_old_head, head, plan.index_before, index,
                                      plan.candidates, JournalState.COMPLETE.value if status == "complete" else JournalState.INDEX_PENDING.value)
        return IntegrationReceipt(plan.operation_id, "blocked", None, plan.planned_tree,
                                  plan.expected_old_head, head, plan.index_before, index,
                                  plan.candidates, JournalState.BLOCKED.value,
                                  {"reason": "foreign_head_change"})

    def _owned_index_matches(self, plan: IntegrationPlan) -> bool:
        for path in plan.owned_paths:
            indexed = self._git("ls-files", "--stage", "--", path, check=False)
            planned = self._git("ls-tree", plan.planned_commit, "--", path, check=False)
            if planned:
                if not indexed:
                    return False
                i = indexed.split()[0:2]
                p = planned.split("\t", 1)[0].split()[0:3:2]
                if len(i) < 2 or len(p) < 2 or i[0] != p[0] or i[1] != p[1]:
                    return False
            elif indexed:
                return False
        return True

    def compensate_uncommitted(self, paths: tuple[CandidatePath, ...], *, before_head: str,
                               after_hashes: Mapping[str, str | None], terminal_proof: Mapping[str, Any] | None = None,
                               native_job_id: str | None = None) -> dict:
        if self.state and native_job_id:
            job = self.state.get("native_job", native_job_id)
            payload = job.get("payload", {}) if job else {}
            terminal_proof = payload.get("completeProof")
            if payload.get("status") != "terminal" or not isinstance(terminal_proof, dict) or terminal_proof.get("complete") is not True:
                raise JenkinsError("native_not_terminal", "Registered native job lacks complete terminal proof")
        if not terminal_proof or not (terminal_proof.get("terminated") or terminal_proof.get("complete")):
            raise JenkinsError("native_not_terminal", "Cannot compensate before native termination proof")
        current = self.head()
        if current != before_head:
            raise JenkinsError("foreign_head_change", "HEAD changed; compensation is blocked")
        blocked: list[str] = []
        restored: list[str] = []
        for p in paths:
            file = self.root / p.path
            current_hash = file_digest(file) if file.exists() else None
            if current_hash != after_hashes.get(p.path):
                blocked.append(p.path)
                continue
            if p.before_digest is None:
                file.unlink(missing_ok=True)
            else:
                content = subprocess.run(["git", "show", f"{before_head}:{p.path}"], cwd=self.root,
                                         check=True, capture_output=True).stdout
                if hashlib.sha256(content).hexdigest() != p.before_digest:
                    blocked.append(p.path)
                    continue
                file.parent.mkdir(parents=True, exist_ok=True)
                file.write_bytes(content)
            restored.append(p.path)
        if blocked:
            raise JenkinsError("foreign_edit_during_compensation", "Compensation blocked by later edits",
                               details={"paths": blocked, "restored": restored})
        return {"status": "compensated", "paths": restored, "terminalProof": dict(terminal_proof)}

    def forward_revert(self, commit_sha: str, *, authorization: Mapping[str, Any], message: str,
                       source_digest: str, coverage_digest: str, acceptance_receipts: tuple[str, ...]) -> str:
        """Apply a forward revert as a new commit, preserving history."""
        if not acceptance_receipts:
            raise JenkinsError("acceptance_missing", "A fresh reverse acceptance is required")
        parent = self._git("rev-parse", f"{commit_sha}^", check=False)
        if not parent:
            raise JenkinsError("revert_root_commit", "Cannot reverse a root commit")
        if self.head() != commit_sha:
            raise JenkinsError("revert_target_not_head", "Reverse target must be current HEAD")
        names = self._git("diff-tree", "--no-commit-id", "--name-only", "-r", commit_sha).splitlines()
        paths: list[CandidatePath] = []
        for path in names:
            old_oid = self._git("rev-parse", f"{commit_sha}:{path}", check=False) or None
            new_oid = self._git("rev-parse", f"{parent}:{path}", check=False) or None
            if new_oid is None:
                paths.append(CandidatePath(path, None, deleted=True))
            else:
                paths.append(CandidatePath(path, new_oid))
        candidate = CandidateManifest(uuid.uuid4().hex, str(self.root), self.head(), tuple(paths),
                                      source_digest, coverage_digest, acceptance_receipts)
        plan = self.plan((candidate,), authorization=authorization, message=message)
        return self.publish(plan, authorization=authorization).commit_sha
