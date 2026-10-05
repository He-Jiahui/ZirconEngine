from __future__ import annotations

import hashlib
import importlib
import shutil
import uuid
import unittest
from pathlib import Path

from tools.jenkins.contracts import JenkinsError
from tools.jenkins.state import State
from tools.jenkins.resources import canonical_build_root
from tools.jenkins.workflow import pools

REPO_ROOT = Path(__file__).resolve().parents[3]
JENKINS_ROOT = REPO_ROOT / ".jenkins"
BUILD_ROOT = Path(r"E:\cargo-targets")
BUILD_NAMESPACE = canonical_build_root(BUILD_ROOT).namespace()
BUILD_NAMESPACE.mkdir(parents=True, exist_ok=True)


class _Raises:
    def __init__(self, typ): self.typ = typ
    def __enter__(self): return self
    def __exit__(self, typ, value, _traceback):
        if typ is None: raise AssertionError(f"expected {self.typ.__name__}")
        if not issubclass(typ, self.typ): return False
        self.value = value
        return True


def raises(typ):
    return _Raises(typ)


def _fixture(tmp_path: Path):
    root = BUILD_NAMESPACE
    unique = tmp_path / "preparation"
    objects = root / "inputs" / "objects"
    objects.mkdir(parents=True, exist_ok=True)
    source = b"alpha\n"
    digest = hashlib.sha256(source).hexdigest()
    (objects / digest).write_bytes(source)
    destination = unique / "preparations" / "key" / "inputs"
    state = State(tmp_path / "state.sqlite3")
    eid, key = "exec-1", "prep-1"
    state.put("execution", eid, {"preparationKey": key, "preparationRoot": str(destination),
                                  "sealedInputRef": "pending",
                                  "reservationKey": "res-1"})
    state.put("resource_reservation", "res-1", {"status": "active", "owner": eid})
    state.put("pool_writer_hold", key, {"status": "active", "owner": eid, "reservationKey": "res-1"})
    manifest = {"schemaVersion": 1, "entries": [{"path": "src/main.rs", "status": "present", "objectDigest": digest}]}
    from tools.jenkins.source import compute_source_digest
    source_digest = compute_source_digest(manifest)
    state.put("sealed_input", source_digest, {"sourceDigest": source_digest,
        "objectRoot": str(root), "sourceManifest": manifest})
    state.put("execution", eid, {"preparationKey": key, "preparationRoot": str(destination),
                                  "sealedInputRef": source_digest, "reservationKey": "res-1"})
    return state, root, destination, eid, key, digest, source_digest


def test_materialize_first_and_exact_reuse(tmp_path):
    state, _root, destination, eid, key, digest, _source = _fixture(tmp_path)
    first = pools.materialize_preparation(state, str(BUILD_ROOT), eid)
    assert first["reused"] is False
    assert (destination / "src/main.rs").read_bytes() == b"alpha\n"
    again = pools.materialize_preparation(state, str(BUILD_ROOT), eid)
    assert again["reused"] is True
    assert state.get("preparation_source", key)["payload"]["ownedFiles"]["src/main.rs"] == digest


def test_compatible_source_updates_owned_files_and_keeps_target(tmp_path):
    state, root, destination, eid, key, _digest, _source = _fixture(tmp_path)
    pools.materialize_preparation(state, str(BUILD_ROOT), eid)
    (destination / "target").mkdir()
    (destination / "target" / "incremental.bin").write_bytes(b"cache")
    new_data = b"beta\n"
    new_digest = hashlib.sha256(new_data).hexdigest()
    (root / "inputs" / "objects" / new_digest).write_bytes(new_data)
    manifest = {"entries": [{"path": "src/main.rs", "status": "present", "objectDigest": new_digest},
                              {"path": "src/old.rs", "status": "deleted"}]}
    from tools.jenkins.source import compute_source_digest
    new_source = compute_source_digest(manifest)
    state.put("sealed_input", new_source, {"sourceDigest": new_source, "objectRoot": str(root),
        "sourceManifest": manifest})
    execution = state.get("execution", eid)
    state.put("execution", eid, {**execution["payload"], "sealedInputRef": new_source}, expected_version=execution["version"])
    result = pools.materialize_preparation(state, str(BUILD_ROOT), eid)
    assert result["incremental"] is True
    assert (destination / "src/main.rs").read_bytes() == new_data
    assert (destination / "target" / "incremental.bin").read_bytes() == b"cache"


def test_foreign_file_and_missing_hold_fail_closed(tmp_path):
    state, _root, destination, eid, key, _digest, _source = _fixture(tmp_path)
    pools.materialize_preparation(state, str(BUILD_ROOT), eid)
    (destination / "foreign.txt").write_text("outside", encoding="utf8")
    with raises(JenkinsError) as exc:
        pools.materialize_preparation(state, str(BUILD_ROOT), eid)
    assert exc.value.code == "preparation_foreign_file"
    hold = state.get("pool_writer_hold", key)
    state.put("pool_writer_hold", key, {**hold["payload"], "status": "released"}, expected_version=hold["version"])
    with raises(JenkinsError) as exc:
        pools.materialize_preparation(state, str(BUILD_ROOT), eid)
    assert exc.value.code == "pool_writer_unreserved"


def test_update_intent_recovers_only_when_all_hashes_are_known(tmp_path):
    state, _root, destination, eid, key, digest, _source = _fixture(tmp_path)
    destination.mkdir(parents=True)
    (destination / "src").mkdir()
    (destination / "src/main.rs").write_bytes(b"alpha\n")
    state.put("preparation_source", key, {"sourceDigest": "d" * 64,
        "ownedFiles": {"src/main.rs": digest}, "manifest": {"entries": []}})
    state.put("preparation_update", key, {"status": "updating", "files": [{"path": "src/main.rs",
        "oldHash": digest, "newHash": digest, "applied": False}]})
    pools.materialize_preparation(state, str(BUILD_ROOT), eid)
    assert state.get("preparation_update", key)["payload"]["status"] == "complete"


def test_nested_module_materializes_without_flat_directory_assumption(tmp_path):
    state, root, destination, eid, key, _digest, _source = _fixture(tmp_path)
    data = b"nested"
    obj = hashlib.sha256(data).hexdigest()
    (root / "inputs" / "objects" / obj).write_bytes(data)
    manifest = {"entries": [{"path": "crates/core/src/lib.rs", "status": "present", "objectDigest": obj}]}
    from tools.jenkins.source import compute_source_digest
    state.put("sealed_input", compute_source_digest(manifest), {"sourceDigest": compute_source_digest(manifest),
        "objectRoot": str(root), "sourceManifest": manifest})
    execution = state.get("execution", eid)
    state.put("execution", eid, {**execution["payload"], "sealedInputRef": compute_source_digest(manifest)}, expected_version=execution["version"])
    result = pools.materialize_preparation(state, str(BUILD_ROOT), eid)
    assert result["physicalPath"] == str(destination)
    assert (destination / "crates/core/src/lib.rs").read_bytes() == data


def test_partial_update_is_completed_from_durable_intent(tmp_path):
    state, root, destination, eid, key, old_digest, _source = _fixture(tmp_path)
    pools.materialize_preparation(state, str(BUILD_ROOT), eid)
    old = (destination / "src/main.rs").read_bytes()
    new = b"recovered"
    new_digest = hashlib.sha256(new).hexdigest()
    (root / "inputs" / "objects" / new_digest).write_bytes(new)
    manifest = {"entries": [{"path": "src/main.rs", "status": "present", "objectDigest": new_digest}]}
    from tools.jenkins.source import compute_source_digest
    source_digest = compute_source_digest(manifest)
    state.put("sealed_input", source_digest, {"sourceDigest": source_digest,
        "objectRoot": str(root), "sourceManifest": manifest})
    execution = state.get("execution", eid)
    state.put("execution", eid, {**execution["payload"], "sealedInputRef": source_digest}, expected_version=execution["version"])
    state.put("preparation_source", key, {"sourceDigest": "e" * 64,
        "ownedFiles": {"src/main.rs": old_digest}, "manifest": {"entries": []}})
    state.put("preparation_update", key, {"executionId": eid, "preparationKey": key,
        "sourceDigest": source_digest, "oldOwnedFiles": {"src/main.rs": old_digest},
        "ownedFiles": {"src/main.rs": new_digest}, "manifest": manifest,
        "objectRoot": str(root), "status": "updating", "files": [{"path": "src/main.rs",
            "oldHash": old_digest, "newHash": new_digest, "applied": False}]})
    result = pools.materialize_preparation(state, str(BUILD_ROOT), eid)
    assert result["reused"] is True
    assert (destination / "src/main.rs").read_bytes() == new
    assert state.get("preparation_source", key)["payload"]["sourceDigest"] == source_digest


def test_reparse_in_preparation_is_rejected(tmp_path):
    state, _root, destination, eid, _key, _digest, _source = _fixture(tmp_path)
    pools.materialize_preparation(state, str(BUILD_ROOT), eid)
    link = destination / "foreign-link"
    try:
        link.symlink_to(destination / "src", target_is_directory=True)
    except (OSError, NotImplementedError):
        return
    with raises(JenkinsError) as exc:
        pools.materialize_preparation(state, str(BUILD_ROOT), eid)
    assert exc.value.code == "preparation_reparse"


class PreparationPoolTests(unittest.TestCase):
    """Discoverable wrappers for the bounded fixture checks above.

    The production suite supplies the Windows physical-root fixtures.  These
    wrappers keep the tiny CAS and recovery checks runnable by the dedicated
    Python interpreter even when pytest is not installed.
    """
    def _run(self, function):
        import tempfile
        (BUILD_NAMESPACE / "test-preparation-pools").mkdir(parents=True, exist_ok=True)
        try:
            with tempfile.TemporaryDirectory(dir=BUILD_NAMESPACE / "test-preparation-pools") as directory:
                function(Path(directory))
        finally:
            # _fixture uses local test seams; never let a test seam survive
            # into another test or a caller's interpreter.
            importlib.reload(pools)

    def test_first_reuse(self): self._run(test_materialize_first_and_exact_reuse)
    def test_incremental(self): self._run(test_compatible_source_updates_owned_files_and_keeps_target)
    def test_foreign_unheld(self): self._run(test_foreign_file_and_missing_hold_fail_closed)
    def test_nested(self): self._run(test_nested_module_materializes_without_flat_directory_assumption)
    def test_partial_recovery(self): self._run(test_partial_update_is_completed_from_durable_intent)
    def test_reparse(self): self._run(test_reparse_in_preparation_is_rejected)
    def test_intent_recovery(self): self._run(test_update_intent_recovers_only_when_all_hashes_are_known)
