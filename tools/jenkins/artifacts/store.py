from __future__ import annotations
import hashlib, json, os, shutil
from dataclasses import dataclass, asdict
from pathlib import Path
from threading import Lock
from uuid import uuid4
from ..contracts import JenkinsError, file_digest, digest


def _payload(record):
    return record.get("payload", {}) if isinstance(record, dict) else {}


def _execution_key(value) -> str:
    """Normalize planner ExecutionKey objects to a stable identity digest."""
    if isinstance(value, str):
        return value
    if isinstance(value, dict):
        return digest(value)
    if hasattr(value, "__dict__"):
        return digest(dict(value.__dict__))
    raise JenkinsError("execution_key_invalid", "Execution key must be a string or structured key")


def _build_path(build_root) -> Path:
    value = getattr(build_root, "path", build_root)
    return Path(value).absolute()


def _approved_root(build_root):
    """Return the verified root object for production callers, if supplied."""
    if os.name == "nt" and hasattr(build_root, "volume_identity"):
        from ..resources.paths import canonical_build_root
        return canonical_build_root(str(build_root.path))
    return None


def _under(root: Path, candidate: Path) -> bool:
    try:
        candidate.absolute().relative_to(root.absolute())
        return True
    except ValueError:
        return False


def _store_for_build_root(build_root) -> "ArtifactStore":
    # Production callers pass the approved root object returned by
    # ``canonical_build_root``.  Revalidate it before touching the CAS; Path
    # values remain supported for portable unit fixtures on non-Windows hosts.
    approved = _approved_root(build_root)
    root = approved.path.absolute() if approved else _build_path(build_root)
    return ArtifactStore(root / "zircon-jenkins" / "artifacts")


def _identity_from_execution(state, execution_id: str, execution_key: str | None = None) -> dict:
    execution = state.get("execution", execution_id)
    payload = _payload(execution)
    identity = dict(payload.get("identity") or payload.get("requestIdentity") or {})
    for key in ("sourceDigest", "recipeDigest", "driverDigest", "coverageDigest"):
        if key not in identity and key in payload:
            identity[key] = payload[key]
    persisted_key = payload.get("executionKey")
    if (execution_key is not None and persisted_key is not None
            and _execution_key(persisted_key) != _execution_key(execution_key)):
        raise JenkinsError("execution_identity_mismatch", "Execution key differs from State")
    return identity


def _trusted_receipt(state, execution_id: str, expected: dict) -> dict:
    receipt_record = state.get("validation_receipt", execution_id)
    attempt = state.get("validation_attempt", execution_id)
    if not receipt_record or not attempt or _payload(attempt).get("status") != "passed":
        raise JenkinsError("artifact_receipt_missing", "Only a passed State validation attempt can publish artifacts")
    receipt = _payload(receipt_record)
    if receipt.get("status") != "passed":
        raise JenkinsError("artifact_receipt_untrusted", "Validation receipt is not terminal passed")
    for key in ("executionId", "sourceDigest", "recipeDigest", "coverageDigest", "driverDigest"):
        if receipt.get(key) != expected.get(key):
            raise JenkinsError("artifact_identity_mismatch", f"Receipt {key} differs from execution identity")
    # Heavy output publication additionally requires the trusted native job
    # proof recorded by the process collector.  A caller supplied proof is not
    # considered evidence.
    terminal = receipt.get("terminalProof")
    if terminal is not None:
        native_id = terminal.get("nativeJobId") if isinstance(terminal, dict) else None
        native = state.get("native_job", native_id) if native_id else None
        proof = _payload(native).get("completeProof") if native else None
        if not isinstance(proof, dict) or proof.get("complete") is not True:
            raise JenkinsError("artifact_terminal_untrusted", "Native terminal proof is not collector-trusted")
    return receipt


def _receipt_ids(stage_receipts, state=None) -> list[str]:
    if isinstance(stage_receipts, dict):
        # Cargo target directories are mutable while later phases run.  The
        # terminal test/acceptance closure is the immutable producer boundary;
        # earlier receipts remain authoritative evidence but are not copied.
        preferred = ("acceptance", "regression_test", "integration_test", "unit_test", "test", "build")
        candidates = [stage_receipts[name] for name in preferred if stage_receipts.get(name)]
        selected = None
        if state is not None:
            selected = next((rid for rid in candidates
                             if isinstance(rid, str) and isinstance(_payload(state.get("validation_receipt", rid)).get("outputClosure", {}).get("files"), list)
                             and _payload(state.get("validation_receipt", rid)).get("outputClosure", {}).get("files")), None)
        if selected is None and candidates:
            selected = candidates[0]
        values = [selected] if selected is not None else stage_receipts.values()
    elif isinstance(stage_receipts, (list, tuple, set)):
        values = stage_receipts
    else:
        raise JenkinsError("artifact_receipts_invalid", "Stage receipts must be a mapping or sequence")
    result = [str(v) for v in values if isinstance(v, str) and v]
    if not result:
        raise JenkinsError("artifact_receipts_invalid", "No stage receipts were supplied")
    return result


def publish_execution_outputs(state, build_root, execution_id: str, execution_key: str,
                              stage_receipts) -> dict:
    """Publish an accepted execution's closure into immutable CAS.

    Receipt and native process records are read from State; the caller cannot
    upload an ad-hoc output list or terminal proof.  Publication is idempotent
    for the same execution identity and records a complete bundle index.
    """
    execution_key = _execution_key(execution_key)
    execution_record = state.get("execution", execution_id)
    if not execution_record:
        raise JenkinsError("execution_unknown", "Execution is not present in State")
    execution_payload = _payload(execution_record)
    # Publication is the transition after the engine has validated all native
    # phase proofs.  ``passed`` is accepted for compatibility with light
    # executions, while heavy execution must be at least ``validated``.
    if execution_payload.get("status") not in {"validated", "passed", "accepted"}:
        raise JenkinsError("execution_not_validated", "Execution outputs cannot be published before validation")
    identity = _identity_from_execution(state, execution_id, execution_key)
    required = ("sourceDigest", "recipeDigest", "driverDigest", "coverageDigest")
    if any(not identity.get(k) for k in required):
        raise JenkinsError("artifact_identity_missing", "Execution identity is incomplete")
    store = _store_for_build_root(build_root)
    root = _build_path(build_root)
    approved = _approved_root(build_root)
    files: list[dict] = []
    seen: set[tuple[str, str]] = set()
    for receipt_id in _receipt_ids(stage_receipts, state):
        receipt = _trusted_receipt(state, receipt_id, {**identity, "executionId": receipt_id})
        closure = receipt.get("outputClosure")
        if not isinstance(closure, dict) or not isinstance(closure.get("files"), list):
            raise JenkinsError("artifact_closure_missing", "Receipt has no output closure")
        closure_root = Path(str(closure.get("root", ""))).absolute()
        if not _under(root, closure_root):
            raise JenkinsError("artifact_output_root_unapproved", "Receipt output root is outside build root")
        if approved is not None:
            from ..resources.paths import physical_path_under
            # Validate every existing ancestor, including junctions/reparse
            # points, rather than trusting lexical containment.
            physical_path_under(approved, str(closure_root), allow_missing=False)
        for entry in closure["files"]:
            rel = entry.get("path")
            expected_digest = entry.get("sha256")
            if not isinstance(rel, str) or not rel or Path(rel).is_absolute() or ".." in Path(rel).parts:
                raise JenkinsError("artifact_output_path_invalid", "Receipt contains an unsafe output path")
            source = (closure_root / rel).absolute()
            if not _under(closure_root, source) or source.is_symlink() or not source.is_file():
                raise JenkinsError("artifact_output_missing", f"Output is not a regular file: {rel}")
            if approved is not None:
                physical_path_under(approved, str(source), allow_missing=False)
            actual = file_digest(source)
            if actual != expected_digest:
                raise JenkinsError("artifact_output_changed", f"Output bytes differ from receipt: {rel}")
            key = (rel.replace("\\", "/"), actual)
            if key in seen:
                continue
            seen.add(key)
            manifest = store.publish(source, execution_id=execution_id, relative_name=key[0])
            files.append({"path": key[0], "digest": manifest.digest, "size": manifest.size,
                          "executionId": execution_id, "stage": receipt.get("stage")})
    if not files:
        raise JenkinsError("artifact_outputs_empty", "A successful execution must publish at least one output")
    files.sort(key=lambda item: (item["path"], item["digest"]))
    bundle_digest = digest({"executionKey": execution_key, "identity": identity, "files": files})
    existing = state.get("artifact_bundle", execution_id)
    if existing:
        if _payload(existing).get("bundleDigest") != bundle_digest:
            raise JenkinsError("artifact_bundle_conflict", "Execution already has a different immutable output bundle")
        return _payload(existing)
    payload = {"status": "published", "executionId": execution_id, "executionKey": execution_key,
               "buildRoot": str(root),
               "identity": identity, "files": files, "bundleDigest": bundle_digest}
    state.put("artifact_bundle", execution_id, payload)
    for item in files:
        key = item["digest"]
        old = state.get("artifact_object", key)
        obj = {"digest": key, "status": "ready", "size": item["size"],
               "path": item["path"], "executionId": execution_id, "executionKey": execution_key,
               "identity": identity}
        if old and _payload(old).get("status") not in {"ready", "published"}:
            raise JenkinsError("artifact_object_unavailable", "Artifact object is not reusable")
        if old and _payload(old).get("digest") != key:
            raise JenkinsError("artifact_object_conflict", "Artifact object identity changed")
        if not old:
            state.put("artifact_object", key, obj)
    return payload


def attach_execution_outputs(state, build_root, execution_id: str, consumer_id: str) -> dict:
    bundle = state.get("artifact_bundle", execution_id)
    if not bundle or _payload(bundle).get("status") != "published":
        raise JenkinsError("artifact_bundle_missing", "No reusable successful execution outputs exist")
    payload = _payload(bundle)
    store = _store_for_build_root(build_root)
    refs = []
    for item in payload["files"]:
        store._verify_object(store.objects / item["digest"], item["digest"], item["size"])
        key = f"{consumer_id}:{item['digest']}"
        old = state.get("artifact_ref", key)
        if old and _payload(old).get("status") == "active":
            refs.append({"key": key, **_payload(old)})
            continue
        state.put("artifact_ref", key, {"digest": item["digest"], "owner": consumer_id,
                                        "kind": "consumer", "executionId": execution_id,
                                        "status": "active"}, expected_version=old["version"] if old else None)
        refs.append({"key": key, "digest": item["digest"], "owner": consumer_id,
                     "executionId": execution_id, "status": "active"})
    return {"executionId": execution_id, "executionKey": payload["executionKey"],
            "bundleDigest": payload["bundleDigest"], "refs": refs,
            "identity": payload["identity"]}


def release_execution_outputs(state, build_root, execution_id: str, consumer_id: str) -> dict:
    """Detach only one consumer's references; producer objects remain indexed."""
    bundle = state.get("artifact_bundle", execution_id)
    if not bundle:
        return {"executionId": execution_id, "consumerId": consumer_id, "released": 0}
    released = 0
    for item in _payload(bundle).get("files", []):
        key = f"{consumer_id}:{item.get('digest')}"
        row = state.get("artifact_ref", key)
        if row and _payload(row).get("status") == "active":
            state.put("artifact_ref", key, {**_payload(row), "status": "released"},
                     expected_version=row["version"])
            released += 1
    return {"executionId": execution_id, "consumerId": consumer_id, "released": released}


def inspect_execution_outputs(state, execution_id: str) -> dict | None:
    row = state.get("artifact_bundle", execution_id)
    return _payload(row) if row else None


def find_reusable_outputs(state, *, execution_key: str, identity: dict,
                          build_root=None) -> dict | None:
    """Return a verified bundle with the exact full input identity."""
    execution_key = _execution_key(execution_key)
    for row in state.list("artifact_bundle"):
        payload = _payload(row)
        if payload.get("status") != "published" or payload.get("executionKey") != execution_key:
            continue
        if payload.get("identity") != identity:
            continue
        root_value = build_root if build_root is not None else payload.get("buildRoot")
        if root_value is None:
            raise JenkinsError("artifact_build_root_missing", "Reusable bundle has no approved build root")
        store = _store_for_build_root(root_value)
        for item in payload.get("files", []):
            store._verify_object(store.objects / item["digest"], item["digest"], item.get("size"))
        return payload
    return None

@dataclass(frozen=True)
class ArtifactManifest:
    digest: str
    size: int
    files: tuple[tuple[str, str, int], ...]
    execution_id: str

@dataclass(frozen=True)
class ArtifactRef:
    ref_id: str
    digest: str
    owner: str
    kind: str

class ArtifactStore:
    """Filesystem content store. State/DB registration is deliberately external."""
    def __init__(self, namespace: str | os.PathLike[str]):
        # Keep the lexical path.  ``resolve`` follows junctions/reparse points
        # and therefore cannot be used as an admission check for a build root.
        self.root = Path(namespace).absolute()
        self._reject_reparse_ancestors(self.root)
        self.objects = self.root / "artifacts" / "objects"
        self.runs = self.root / "runs"
        self.objects.mkdir(parents=True, exist_ok=True)
        self.runs.mkdir(parents=True, exist_ok=True)
        self._lock = Lock()

    @staticmethod
    def _reject_reparse_ancestors(path: Path) -> None:
        if os.name != "nt":
            return
        current = path
        while current != current.parent:
            try:
                attrs = getattr(current.stat(), "st_file_attributes", 0)
            except FileNotFoundError:
                current = current.parent
                continue
            if attrs & 0x0400:
                raise JenkinsError("artifact_alias", f"Artifact path contains a reparse point: {current}")
            current = current.parent

    def _verify_object(self, path: Path, digest: str, size: int | None = None) -> None:
        self._reject_reparse_ancestors(path)
        if not path.is_file() or path.is_symlink():
            raise JenkinsError("artifact_missing", f"Artifact object is not a regular file: {path}")
        actual_size = path.stat().st_size
        if size is not None and actual_size != size:
            raise JenkinsError("artifact_digest_mismatch", "Artifact object size differs from its manifest")
        if file_digest(path) != digest:
            raise JenkinsError("artifact_digest_mismatch", "Artifact object bytes do not match its digest")

    def publish(self, source: str | os.PathLike[str], *, execution_id: str,
                relative_name: str | None = None) -> ArtifactManifest:
        # A source may be an external sealed input, so its parent is checked by
        # the caller's receipt authority.  We still reject aliases at the file
        # itself on Windows and never follow a symlink as an input.
        src = Path(source).absolute()
        if os.name == "nt" and src.is_symlink():
            raise JenkinsError("artifact_alias", "Artifact input cannot be a symlink")
        if not src.is_file(): raise JenkinsError("artifact_input_missing", f"Artifact input missing: {src}")
        digest = file_digest(src)
        dest = self.objects / digest
        temp = self.runs / f"publish-{uuid4().hex}.tmp"
        shutil.copyfile(src, temp)
        if hashlib.sha256(temp.read_bytes()).hexdigest() != digest:
            temp.unlink(missing_ok=True); raise JenkinsError("artifact_digest_mismatch", "Published bytes changed during copy")
        dest.parent.mkdir(parents=True, exist_ok=True)
        if not dest.exists():
            os.replace(temp, dest)
        else:
            # Existing CAS objects are immutable.  Never treat a corrupt
            # object as a successful deduplicated publish.
            temp.unlink(missing_ok=True)
            self._verify_object(dest, digest, src.stat().st_size)
        name = relative_name or src.name
        return ArtifactManifest(digest, dest.stat().st_size, ((name, digest, dest.stat().st_size),), execution_id)

    def add_ref(self, digest: str, owner: str, *, kind: str = "consumer") -> ArtifactRef:
        self._verify_object(self.objects / digest, digest)
        ref = ArtifactRef(uuid4().hex, digest, owner, kind)
        return ref

    def release_ref(self, ref_id: str) -> bool:
        # Ref lifecycle is owned by ArtifactRegistry/State. This compatibility
        # method cannot mutate an authoritative refcount and therefore reports
        # false; callers should use ArtifactRegistry.detach.
        return False

    def gc(self, *, protected: set[str] = frozenset(), deletable: set[str] | None = None) -> list[str]:
        if deletable is None:
            raise JenkinsError("artifact_gc_scope_missing", "artifact deletion requires an explicit registered object scope")
        live = set(protected)
        removed = []
        for obj in self.objects.iterdir():
            if obj.is_file() and obj.name not in live and obj.name in deletable:
                self._verify_object(obj, obj.name)
                obj.unlink(); removed.append(obj.name)
        return removed

class ArtifactRegistry:
    """State-backed refs; filesystem deletion is performed only after CAS."""
    def __init__(self, state, store: ArtifactStore): self.state, self.store = state, store
    def attach(self, digest: str, owner: str, *, kind: str = "consumer"):
        self.store._verify_object(self.store.objects / digest, digest)
        key = f"{owner}:{digest}"
        with self.state.transaction() as conn:
            obj = self.state.get("artifact_object", digest, connection=conn)
            if obj and obj["payload"].get("status") == "deleting":
                raise JenkinsError("artifact_deleting", "Artifact is pending deletion", retryable=True)
            if not obj:
                self.state.put("artifact_object", digest, {"digest":digest,"status":"ready"}, connection=conn)
            old = self.state.get("artifact_ref", key, connection=conn)
            payload = {"digest":digest,"owner":owner,"kind":kind,"status":"active"}
            return self.state.put("artifact_ref", key, payload, expected_version=old["version"] if old else None, connection=conn)
    def detach(self, key: str) -> bool:
        with self.state.transaction() as conn:
            old = self.state.get("artifact_ref", key, connection=conn)
            if not old or old["payload"].get("status") != "active": return False
            self.state.put("artifact_ref", key, {**old["payload"],"status":"released"}, expected_version=old["version"], connection=conn)
            return True
    def gc(self, *, protected= frozenset()):
        from ..maintenance import _historical_digests
        protected = set(protected) | _historical_digests(self.state)
        with self.state.transaction() as conn:
            active = {r["payload"]["digest"] for r in self.state.list("artifact_ref", connection=conn) if r["payload"].get("status") == "active"}
            deletable = set()
            for obj in self.state.list("artifact_object", connection=conn):
                if obj["key"] in active or obj["key"] in protected:
                    continue
                if obj["payload"].get("status") == "deleting": deletable.add(obj["key"])
                elif obj["payload"].get("status") == "ready":
                    self.state.put("artifact_object", obj["key"], {**obj["payload"], "status":"deleting"}, expected_version=obj["version"], connection=conn)
                    deletable.add(obj["key"])
        removed = self.store.gc(protected=set(protected) | active, deletable=deletable)
        with self.state.transaction() as conn:
            for digest in removed:
                obj=self.state.get("artifact_object",digest,connection=conn)
                if obj and not any(r["payload"].get("digest")==digest and r["payload"].get("status")=="active" for r in self.state.list("artifact_ref",connection=conn)):
                    self.state.put("artifact_object",digest,{**obj["payload"],"status":"deleted"},expected_version=obj["version"],connection=conn)
        return removed

    def query(self, digest: str):
        return self.state.get("artifact_object", digest)

    def handle(self, action: str, payload: dict):
        if action == "publish":
            return self.store.publish(payload["source"], execution_id=payload["executionId"]).__dict__
        if action == "attach": return self.attach(payload["digest"], payload["owner"], kind=payload.get("kind", "consumer"))
        if action == "query": return self.query(payload["digest"])
        if action == "gc": return {"removed": self.gc(protected=set(payload.get("protected", [])))}
        raise JenkinsError("unsupported_artifact_action", action)
