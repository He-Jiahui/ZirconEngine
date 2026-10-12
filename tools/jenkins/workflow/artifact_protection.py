"""Reviewed protection provenance for one missing historical single-file blob.

This companion index never changes an old reference or object, locates a
missing blob by guessing, or grants deletion of it. Its only use is a content
and physical-path exclusion against one fully inventoried build candidate.
"""
from __future__ import annotations

import hashlib
import json
import ntpath
from pathlib import Path

from ..contracts import JenkinsError, digest, require_digest
from ..deployment.driver import _plain
from ..resources.paths import canonical_build_root, physical_path_under


KIND = "reviewed-legacy-single-blob-v1"
DOMAIN = "artifact_protection_index"
BLOB_DIGEST = "dfdf03e82f40be411a7bc3aa38e84198dd6eb75ee022b0b15ada7675311f1455"
BLOB_BYTES = 3800
FIXTURE = "1791048317218769100"
OLD_PATH = r"D:\cargo-targets\zircon-jenkins\artifacts\artifacts\objects" + "\\" + BLOB_DIGEST
CANDIDATE = (r"D:\cargo-targets\zircon-local\engine-audit-20261009-01a12097"
             r"\runtime-check-d-r1\compile\build")
RELATIVE_SUBTREE = "debug/incremental/zircon_runtime-19sglz35q0wnf"
PRODUCER = (".jenkins/state/migration-baseline/run_gc_acceptance.py",
            "502a04ba171d0ae35410bd0134330067cc27453f3d69e0d89433f0a9a2ba67a3")
ARCHIVE = (".jenkins/state/migration-baseline/gc-runtime-acceptance.json",
           "d65778fd5fa6fee838226a87ca03a426e94600843dc4e044b4428ddd6b37d019")
REF_KEY = "gc-active-" + FIXTURE
OP_KEY = "gc-op-" + FIXTURE
ROW_PINS = (
    ("artifact_ref", REF_KEY, 1, "5e00aa27bcb3f237cd58db94735f4d4bf14bb92f48dc6a87011faf8d4199844f"),
    ("artifact_object", BLOB_DIGEST, 1, "3dd2301402de91ca3dede554c00a6f9fb86b6b702b8c73c2e50d910e0ee372cf"),
    ("maintenance_gc", OP_KEY, 1, "61527004fad34ada255cf264103ded311542377b4a3fb468831c4046f1e641d3"),
    ("maintenance_inventory", OP_KEY, 1, "34580860245f4848d09dd31895790f49629432a05bf88b2efbd210988b17c8f3"),
)


def _deny(code, message):
    return JenkinsError(code, message)


def candidate_path(relative_subtree=None) -> str:
    if relative_subtree is None:
        return CANDIDATE
    if relative_subtree != RELATIVE_SUBTREE:
        raise _deny("artifact_protection_scope_invalid", "Only the fixed historical incremental subtree is supported")
    return str(Path(CANDIDATE).joinpath(*RELATIVE_SUBTREE.split("/")))


def inventory_prefix(candidate) -> str:
    if str(candidate) == CANDIDATE:
        return "build"
    if str(candidate) == candidate_path(RELATIVE_SUBTREE):
        return "build/" + RELATIVE_SUBTREE
    raise _deny("artifact_protection_scope_invalid", "Candidate must be the historical build tree or its fixed named subtree")


def review_payload(repo_root: Path, task: str, relative_subtree=None) -> dict:
    """Describe the exact provenance the authorized new request must review."""
    return {"kind": KIND, "candidatePath": candidate_path(relative_subtree), "blobDigest": BLOB_DIGEST,
            "blobByteLength": BLOB_BYTES, "oldPath": OLD_PATH,
            "producer": {"path": str(repo_root / PRODUCER[0]), "sha256": PRODUCER[1]},
            "archive": {"path": str(repo_root / ARCHIVE[0]), "sha256": ARCHIVE[1]},
            "rows": [{"domain": domain, "key": key, "version": version, "payloadDigest": checksum}
                     for domain, key, version, checksum in ROW_PINS],
            "review": {"source": "user", "task": task}}


def normalize_review(value, repo_root: Path, candidate: str) -> dict:
    review = value.get("review") if isinstance(value, dict) else None
    if (not isinstance(review, dict) or set(review) != {"source", "task"} or review["source"] != "user"
            or not isinstance(review["task"], str) or not review["task"].strip() or len(review["task"]) > 512):
        raise _deny("artifact_protection_review_missing", "A registered explicit user provenance review is required")
    prefix = inventory_prefix(candidate)
    expected = review_payload(repo_root, review["task"], RELATIVE_SUBTREE if prefix != "build" else None)
    if (value != expected or set(value) != set(expected)
            or type(value.get("blobByteLength")) is not int
            or any(type(pin.get("version")) is not int or set(pin) != {"domain", "key", "version", "payloadDigest"}
                   for pin in value.get("rows", []))):
        raise _deny("artifact_protection_scope_invalid", "Only the exact reviewed legacy blob and candidate are supported")
    return expected


def _read_pin(repo_root, fact):
    path = Path(fact["path"])
    _plain(repo_root); _plain(path)
    try:
        before = path.stat()
        if not path.is_file() or before.st_nlink != 1 or before.st_size > 1024 * 1024:
            raise _deny("artifact_protection_provenance_invalid", "Reviewed provenance must be one bounded plain file")
        raw = path.read_bytes()
        _plain(path)
        after = path.stat()
    except OSError as error:
        raise _deny("artifact_protection_provenance_invalid", "Reviewed provenance is missing or cannot be read as a plain file") from error
    if (any(getattr(before, field) != getattr(after, field)
            for field in ("st_dev", "st_ino", "st_size", "st_mtime_ns", "st_ctime_ns", "st_nlink"))
            or hashlib.sha256(raw).hexdigest() != fact["sha256"]):
        raise _deny("artifact_protection_provenance_changed", "Reviewed provenance differs from its authorized pin")
    return raw


def _legacy(state, connection, repo_root, review):
    _read_pin(repo_root, review["producer"])
    try:
        archive = json.loads(_read_pin(repo_root, review["archive"]))
    except (ValueError, UnicodeError) as error:
        raise _deny("artifact_protection_provenance_invalid", "Reviewed archive must be JSON") from error
    rows = {}
    for pin in review["rows"]:
        row = state.get(pin["domain"], pin["key"], connection=connection)
        if row is None or row["version"] != pin["version"] or digest(row["payload"]) != pin["payloadDigest"]:
            raise _deny("artifact_protection_row_changed", "A pinned historical row changed or is missing")
        rows[pin["domain"]] = row["payload"]
    ref, obj = rows["artifact_ref"], rows["artifact_object"]
    old_gc, inv = rows["maintenance_gc"], rows["maintenance_inventory"]
    if (ref != {"digest": BLOB_DIGEST, "owner": "gc-fixture", "status": "active"}
            or obj != {"fixture": FIXTURE, "size": BLOB_BYTES, "status": "ready"}
            or not isinstance(archive, dict) or archive.get("operationId") != OP_KEY
            or archive.get("identity") != {"repositoryId": "maintenance-fixture", "sessionId": "gc-" + FIXTURE,
                                            "requestId": "gc-request-" + FIXTURE}
            or archive.get("objects", {}).get("active") != {"digest": BLOB_DIGEST, "size": BLOB_BYTES, "path": OLD_PATH}
            or archive.get("receipt") != old_gc or old_gc.get("operationId") != OP_KEY
            or old_gc.get("repositoryId") != "maintenance-fixture" or old_gc.get("sessionId") != "gc-" + FIXTURE
            or BLOB_DIGEST not in old_gc.get("protectedDigests", [])
            or any(item.get("digest") == BLOB_DIGEST for item in old_gc.get("removed", []))
            or inv.get("operationId") != OP_KEY or BLOB_DIGEST not in inv.get("activeDigests", [])
            or old_gc.get("buildRoot") != r"D:\cargo-targets" or inv.get("buildRoot") != r"D:\cargo-targets"):
        raise _deny("artifact_protection_protocol_invalid", "Historical rows must describe this exact single-file protected blob")
    # The reviewed producer writes a raw content-addressed file. This proves
    # the digest denotes those 3800 bytes, rather than a bundle/directory key.
    raw = ("active-acceptance-" + FIXTURE + "\n").encode("utf-8") * 100
    if len(raw) != BLOB_BYTES or hashlib.sha256(raw).hexdigest() != BLOB_DIGEST:
        raise _deny("artifact_protection_protocol_invalid", "Reviewed legacy producer content identity is inconsistent")


def _observe_missing(approved, old_path):
    """Observe absence through pinned, non-reparse Windows parent handles."""
    from ..pilot.native.windows_storage import WindowsStorage, WindowsStorageError, _windows_error_code
    fresh = canonical_build_root(str(approved.path))
    if fresh.volume_identity != approved.volume_identity:
        raise _deny("artifact_protection_volume_changed", "Protection root physical mapping changed")
    physical_path_under(fresh, old_path, allow_missing=True)
    storage = WindowsStorage(fresh.path, expected_device=fresh.volume_identity, readonly_root=True)
    try:
        relative = Path(old_path).relative_to(fresh.path).as_posix()
        try:
            with storage.open_relative(relative, "rb"):
                raise _deny("artifact_protection_presence_changed", "The missing old blob has reappeared and needs fresh review")
        except WindowsStorageError as error:
            if _windows_error_code(error) not in {2, 3}:
                raise
        identity = storage.root_identity
        return {"status": "missing", "volumeIdentity": fresh.volume_identity,
                "rootDevicePath": storage.root_device_path, "rootFileId": identity.file_id.hex(), "device": identity.device}
    finally:
        storage.close()


def assert_inventory(inventory, candidate):
    prefix = inventory_prefix(candidate) + "/"
    if (not isinstance(inventory, dict) or inventory.get("complete") is not True
            or set(inventory) != {"candidatePath", "files", "fileCount", "totalBytes", "complete", "inventoryDigest"}
            or inventory.get("candidatePath") != str(candidate) or not isinstance(inventory.get("files"), list)
            or inventory.get("inventoryDigest") != digest({k: v for k, v in inventory.items() if k != "inventoryDigest"})
            or type(inventory.get("fileCount")) is not int or type(inventory.get("totalBytes")) is not int
            or inventory.get("fileCount") != len(inventory["files"])):
        raise _deny("artifact_protection_inventory_invalid", "Protection exclusion requires one complete immutable candidate inventory")
    total, names = 0, set()
    for item in inventory["files"]:
        if (not isinstance(item, dict) or set(item) != {"path", "sha256", "byteLength", "fileId", "device", "mtimeNs", "linkCount"}
                or not isinstance(item["path"], str) or not item["path"].startswith(prefix)
                or any(part in {"", ".", ".."} for part in item["path"].split("/")) or "\\" in item["path"] or ":" in item["path"]
                or item["path"].casefold() in names or item["linkCount"] != 1 or type(item["linkCount"]) is not int
                or type(item["byteLength"]) is not int or item["byteLength"] < 0
                or not isinstance(item["fileId"], str) or len(item["fileId"]) != 32
                or any(c not in "0123456789abcdef" for c in item["fileId"])
                or type(item["device"]) is not int or type(item["mtimeNs"]) is not int):
            raise _deny("artifact_protection_inventory_invalid", "Every inventoried file requires SHA, FileId, bytes and single-link identity")
        require_digest(item["sha256"], "candidate file SHA-256")
        if item["sha256"] == BLOB_DIGEST:
            raise _deny("artifact_protection_content_overlap", "The protected legacy blob is present inside the candidate")
        total += item["byteLength"]; names.add(item["path"].casefold())
    if total != inventory.get("totalBytes"):
        raise _deny("artifact_protection_inventory_invalid", "Candidate inventory byte accounting is inconsistent")
    old, selected = ntpath.normcase(OLD_PATH), ntpath.normcase(str(candidate))
    if old == selected or old.startswith(selected.rstrip("\\") + "\\") or selected.startswith(old.rstrip("\\") + "\\"):
        raise _deny("artifact_protection_path_overlap", "Historical object location overlaps the candidate")


def _key(authority, review):
    return digest({"authorityDigest": digest(authority), "reviewDigest": digest(review), "referenceKey": REF_KEY})


def _review_authority(state, connection, authority, review):
    identity = authority.get("identity", {})
    request = state.get_request(identity.get("repositoryId"), identity.get("sessionId"), identity.get("requestId"),
                                connection=connection)
    if (request is None or request["payloadDigest"] != authority.get("requestPayloadDigest")
            or request["payloadDigest"] != digest(request["payload"])
            or request["generation"] != identity.get("generation")
            or request["status"] in {"cancel_requested", "cancelled", "failed"}
            or request["payload"].get("reviewedProtection") != review
            or request["payload"].get("operationId") != authority.get("operationId")
            or any(request["payload"].get(field) != value for field, value in identity.items())):
        raise _deny("artifact_protection_review_missing", "Protection review must belong to the current registered maintenance request")
    paths = request["payload"].get("ownedPaths")
    if not isinstance(paths, list) or not paths:
        raise _deny("artifact_protection_review_missing", "Protection review requires an explicit granted maintenance scope")
    auth = state.check_authorization(identity.get("repositoryId"), identity.get("sessionId"), "gc", paths, connection=connection)
    if (auth["snapshotDigest"] != authority.get("authorizationSnapshot")
            or state.get("cancellation", ":".join(str(identity.get(f, "")) for f in
                       ("repositoryId", "sessionId", "requestId")), connection=connection)):
        raise _deny("artifact_protection_review_missing", "Protection review authorization changed or was cancelled")


def register_index(state, connection, repo_root, approved, candidate, authority, review, inventory):
    review = normalize_review(review, repo_root, str(candidate))
    _review_authority(state, connection, authority, review)
    _legacy(state, connection, repo_root, review)
    assert_inventory(inventory, candidate)
    observation = _observe_missing(approved, review["oldPath"])
    value = {"schemaVersion": 1, "kind": KIND, "status": "missing-unlocated", "protectOnly": True,
             "deletionAllowed": False, "blobDigest": BLOB_DIGEST, "blobByteLength": BLOB_BYTES,
             "oldPath": OLD_PATH, "candidatePath": str(candidate), "candidateInventoryDigest": inventory["inventoryDigest"],
             "authorityDigest": digest(authority), "reviewDigest": digest(review), "reviewedProvenance": review,
             "physicalObservation": observation, "JenkinsAcceptance": False, "milestoneAcceptance": False}
    key = _key(authority, review)
    old = state.get(DOMAIN, key, connection=connection)
    if old:
        if old["version"] != 1 or old["payload"] != value:
            raise _deny("artifact_protection_index_changed", "The immutable protection index differs from its reviewed provenance")
    else:
        state.put(DOMAIN, key, value, expected_version=0, connection=connection)
    return key


def excludes_reference(state, connection, row, *, repo_root, approved, candidate, authority, review, inventory, index_ref):
    """Recognize one exact old ref; all other unlocated refs remain blocked."""
    if row["domain"] != "artifact_ref" or row["key"] != REF_KEY:
        return False
    review = normalize_review(review, repo_root, str(candidate))
    _review_authority(state, connection, authority, review)
    if index_ref != _key(authority, review):
        raise _deny("artifact_protection_index_changed", "Protection index belongs to another request or operation")
    _legacy(state, connection, repo_root, review)
    assert_inventory(inventory, candidate)
    index = state.get(DOMAIN, index_ref, connection=connection)
    if index is None or index["version"] != 1:
        raise _deny("artifact_protection_index_missing", "Reviewed protection index is missing or changed")
    value = index["payload"]
    expected = {"schemaVersion": 1, "kind": KIND, "status": "missing-unlocated", "protectOnly": True,
                "deletionAllowed": False, "blobDigest": BLOB_DIGEST, "blobByteLength": BLOB_BYTES,
                "oldPath": OLD_PATH, "candidatePath": str(candidate), "candidateInventoryDigest": inventory["inventoryDigest"],
                "authorityDigest": digest(authority), "reviewDigest": digest(review), "reviewedProvenance": review,
                "physicalObservation": _observe_missing(approved, OLD_PATH),
                "JenkinsAcceptance": False, "milestoneAcceptance": False}
    if value != expected:
        raise _deny("artifact_protection_index_changed", "Protection metadata or current physical absence changed")
    return True
