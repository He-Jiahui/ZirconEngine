"""Seal an explicit Git input scope, including uncommitted source changes."""

from __future__ import annotations

import hashlib
import io
import json
import os
import re
import stat
import subprocess
import zipfile
from dataclasses import dataclass
from pathlib import Path
from typing import Sequence

from .native.portable_paths import normalize_portable_relative_path

from .contracts import PilotError, canonical_json, digest, sha256_identity
from .storage import ManagedStorage, require_managed_root

MAX_BUNDLE_BYTES = 32 * 1024 * 1024
MAX_FILE_BYTES = 16 * 1024 * 1024
MAX_MANIFEST_BYTES = 8 * 1024 * 1024
_GIT_OBJECT = re.compile(r"[0-9a-f]{40}\Z")
_PRIVATE_KEY = re.compile(rb"-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----")
_SECRET_NAMES = frozenset({".env", "auth.json", "credentials.json", "id_rsa", "id_ed25519"})


def portable_path(value: str) -> str:
    if not isinstance(value, str):
        raise PilotError("source path must be text")
    raw = value.replace("\\", "/")
    if any(part in {"", ".", ".."} for part in raw.split("/")):
        raise PilotError("source paths must be canonical portable relative paths")
    try:
        return normalize_portable_relative_path(raw, code="pilot_source", message="unsafe source path")
    except (ValueError, PilotError) as error:
        raise PilotError("source path is unsafe") from error


def _admit_source(name: str, data: bytes | None = None) -> None:
    parts = name.casefold().split("/")
    if ".git" in parts or any(part in _SECRET_NAMES or part.startswith(".env.") for part in parts):
        raise PilotError("source scope contains a private configuration path")
    if data is not None and _PRIVATE_KEY.search(data):
        raise PilotError("source scope contains private key material")


def _git(repo_root: Path, *arguments: str) -> bytes:
    from .trusted_tools import git_command, git_identity
    argv, identity = git_command(repo_root, *arguments)
    try:
        result = subprocess.check_output(
            argv,
            stderr=subprocess.PIPE, creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
        )
    except subprocess.CalledProcessError as error:
        raise PilotError("Git could not resolve the sealed source base or scope") from error
    if git_identity() != identity:
        raise PilotError("trusted Git executable changed during source resolution")
    return result


def _base_tree(repo_root: Path, base: str) -> dict[str, tuple[str, str]]:
    result = {}
    for record in _git(repo_root, "ls-tree", "-r", "-z", base).split(b"\0"):
        if not record:
            continue
        metadata, raw_name = record.split(b"\t", 1)
        mode, kind, object_id = metadata.decode("ascii").split(" ")
        name = raw_name.decode("utf-8")
        result[name] = (mode, object_id)
    return result


def _in_scope(name: str, scopes: Sequence[str]) -> bool:
    return any(scope == "." or name == scope or name.startswith(scope + "/") for scope in scopes)


def _blobs(repo_root: Path, object_ids: Sequence[str]) -> dict[str, bytes]:
    """Read immutable Git objects with one subprocess, rather than one per file."""
    unique = sorted(set(object_ids))
    from .trusted_tools import git_command, git_identity
    argv, identity = git_command(repo_root, "cat-file", "--batch")
    process = subprocess.run(
        argv,
        input="".join(value + "\n" for value in unique).encode("ascii"),
        stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False,
        creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
    )
    if git_identity() != identity:
        raise PilotError("trusted Git executable changed during source resolution")
    if process.returncode or len(process.stdout) > 256 * 1024 * 1024:
        raise PilotError("fixed Git base could not be read within the pilot scope limit")
    stream, result = io.BytesIO(process.stdout), {}
    for expected in unique:
        fields = stream.readline().strip().split(b" ")
        if len(fields) != 3 or fields[0].decode("ascii") != expected or fields[1] != b"blob":
            raise PilotError("source base contains an unavailable or non-file object")
        size = int(fields[2])
        if size > MAX_FILE_BYTES:
            raise PilotError("source base file exceeds the pilot file limit")
        result[expected] = stream.read(size)
        if len(result[expected]) != size or stream.read(1) != b"\n":
            raise PilotError("source base object stream is incomplete")
        if hashlib.sha1(b"blob " + str(size).encode("ascii") + b"\0" + result[expected]).hexdigest() != expected:
            raise PilotError("fixed Git base blob content does not match its object identity")
    return result


def _read_plain(root: Path, name: str) -> bytes | None:
    for ancestor in (root, *root.parents):
        root_info = ancestor.lstat()
        if stat.S_ISLNK(root_info.st_mode) or getattr(root_info, "st_file_attributes", 0) & 0x400:
            raise PilotError("source root cannot cross links or reparse points")
    path = root.joinpath(*name.split("/"))
    current = root
    for part in name.split("/"):
        current = current / part
        try:
            info = current.lstat()
        except FileNotFoundError:
            return None
        if stat.S_ISLNK(info.st_mode) or getattr(info, "st_file_attributes", 0) & 0x400:
            raise PilotError("source scope cannot follow links or reparse points")
    if not stat.S_ISREG(info.st_mode) or info.st_nlink != 1 or info.st_size > MAX_FILE_BYTES:
        raise PilotError("source input is not a bounded plain file")
    before = (info.st_dev, info.st_ino, info.st_size, info.st_mtime_ns, info.st_ctime_ns)
    with path.open("rb") as stream:
        opened = os.fstat(stream.fileno())
        if (opened.st_dev, opened.st_ino) != (info.st_dev, info.st_ino):
            raise PilotError("source changed before it could be sealed")
        data = stream.read(MAX_FILE_BYTES + 1)
    after = path.lstat()
    if before != (after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns, after.st_ctime_ns):
        raise PilotError("source changed while it was being sealed")
    if len(data) != info.st_size:
        raise PilotError("source size changed while it was being sealed")
    _admit_source(name, data)
    return data


@dataclass(frozen=True, slots=True)
class SourceSnapshot:
    bundle: Path
    manifest: dict[str, object]
    bundle_hash: str

    @property
    def input_hash(self) -> str:
        return self.manifest["inputHash"]


def capture(
    repo_root: Path, storage: ManagedStorage, *, paths: Sequence[str],
    untracked_allowlist: Sequence[str] = (),
    external: dict[Path, dict[str, str | None]] | None = None,
) -> SourceSnapshot:
    """Seal a declared complete read scope; unknown new files fail admission."""
    supplied_root = Path(repo_root)
    if ".." in supplied_root.parts:
        raise PilotError("source root cannot contain traversal aliases")
    repo_root = Path(os.path.abspath(supplied_root))
    for ancestor in (repo_root, *repo_root.parents):
        info = ancestor.lstat()
        if stat.S_ISLNK(info.st_mode) or getattr(info, "st_file_attributes", 0) & 0x400:
            raise PilotError("source root cannot cross links or reparse points")
    if not paths:
        raise PilotError("an explicit source read scope is required")
    scopes = sorted({"." if value == "." else portable_path(value) for value in paths})
    allowlist = [portable_path(value) for value in untracked_allowlist]
    base = _git(repo_root, "rev-parse", "HEAD").decode("ascii").strip()
    if not _GIT_OBJECT.fullmatch(base):
        raise PilotError("pilot requires a fixed SHA-1 Git commit base")
    tree = _base_tree(repo_root, base)
    originals = _blobs(repo_root, [value[1] for name, value in tree.items() if _in_scope(name, scopes) and value[0] in {"100644", "100755"}])

    def scan():
        tracked = _git(repo_root, "ls-files", "-z").decode("utf-8").split("\0")
        new = _git(repo_root, "ls-files", "--others", "--exclude-standard", "-z").decode("utf-8").split("\0")
        names = sorted({name for name in [*tree, *tracked, *new] if name and _in_scope(name, scopes)})
        if not names or len(names) > 100000:
            raise PilotError("source scope is empty or exceeds the pilot entry limit")
        entries, payloads, keys = [], {}, set()
        for raw_name in names:
            name = portable_path(raw_name)
            _admit_source(name)
            key = name.casefold()
            if key in keys:
                raise PilotError("source scope contains a case-insensitive path collision")
            keys.add(key)
            original = tree.get(name)
            if original is not None and original[0] not in {"100644", "100755"}:
                raise PilotError("source scope contains a symlink or submodule")
            if original is None and not _in_scope(name, allowlist):
                raise PilotError("new source files require an explicit allowlist")
            data = _read_plain(repo_root, name)
            if data is None and original is None:
                raise PilotError("new source disappeared during sealing")
            content_hash = hashlib.sha256(data).hexdigest() if data is not None else None
            state = "new" if original is None else "deleted" if data is None else "modified"
            if original is not None and data is not None:
                original_data = originals[original[1]]
                if data == original_data:
                    state = "base"
            payload = content_hash if state in {"new", "modified"} else None
            if payload:
                payloads[payload] = data
            entries.append({
                "path": name, "status": state, "mode": original[0] if original else "100644",
                "baseBlob": original[1] if original else None,
                "size": len(data) if data is not None else 0, "sha256": content_hash, "payload": payload,
            })
        return entries, payloads

    from . import external_snapshot
    external_sources = external_snapshot.prepare(repo_root, external or {})
    external_entries, external_payloads = external_snapshot.capture(external_sources)
    entries, payloads = scan()
    second_entries, second_payloads = scan()
    if entries != second_entries or payloads != second_payloads or _git(repo_root, "rev-parse", "HEAD").decode().strip() != base:
        raise PilotError("source scope or base changed during sealing; capture again")
    if external_snapshot.capture(external_sources) != (external_entries, external_payloads):
        raise PilotError("external source changed during sealing; capture again")
    manifest = {"schemaVersion": 1, "baseCommit": base, "paths": scopes, "entries": entries}
    if external_entries:
        manifest.update(schemaVersion=2, external=external_entries)
        payloads.update(external_payloads)
    manifest["inputHash"] = digest(manifest)
    encoded = canonical_json(manifest)
    if len(encoded) > MAX_MANIFEST_BYTES or sum(map(len, payloads.values())) > MAX_BUNDLE_BYTES:
        raise PilotError("sealed input exceeds the small-pilot transfer limit")
    output = io.BytesIO()
    with zipfile.ZipFile(output, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        for name, content in [("manifest.json", encoded), *[(f"payloads/{key}", value) for key, value in sorted(payloads.items())]]:
            info = zipfile.ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            archive.writestr(info, content)
    raw = output.getvalue()
    if len(raw) > MAX_BUNDLE_BYTES:
        raise PilotError("source archive exceeds the small-pilot transfer limit")
    bundle_hash = hashlib.sha256(raw).hexdigest()
    bundle = storage.atomic_write(f"snapshots/{bundle_hash}.zip", raw)
    return load_snapshot(bundle, expected_input_hash=manifest["inputHash"])


def _validate_manifest(manifest: object) -> dict[str, object]:
    if not isinstance(manifest, dict):
        raise PilotError("source manifest fields are invalid")
    version = manifest.get("schemaVersion")
    fields = {"schemaVersion", "baseCommit", "paths", "entries", "inputHash"}
    if version == 2:
        fields.add("external")
    if set(manifest) != fields:
        raise PilotError("source manifest fields are invalid")
    if type(version) is not int or version not in {1, 2}:
        raise PilotError("source manifest version is unsupported")
    if not isinstance(manifest["baseCommit"], str) or not _GIT_OBJECT.fullmatch(manifest["baseCommit"]):
        raise PilotError("source manifest has no fixed commit base")
    sha256_identity(manifest["inputHash"], "inputHash")
    body = {key: value for key, value in manifest.items() if key != "inputHash"}
    if digest(body) != manifest["inputHash"]:
        raise PilotError("source manifest digest does not match its content")
    scopes = manifest["paths"]
    if not isinstance(scopes, list) or not scopes or any(not isinstance(x, str) for x in scopes):
        raise PilotError("source manifest has no declared input scope")
    for scope in scopes:
        if scope != ".":
            if portable_path(scope) != scope:
                raise PilotError("source scope is not canonical")
    if scopes != sorted(set(scopes)):
        raise PilotError("source scopes are not canonically ordered")
    _validate_entries(manifest["entries"], scopes)
    if version == 2:
        from .external_snapshot import validate
        validate(manifest["external"])
    return manifest


def _validate_entries(entries: object, scopes: Sequence[str], *, full_payload: bool = False) -> None:
    if not isinstance(entries, list) or not entries or len(entries) > 100000:
        raise PilotError("source manifest entries are invalid")
    keys, names, components = set(), [], {}
    for entry in entries:
        if not isinstance(entry, dict) or set(entry) != {"path", "status", "mode", "baseBlob", "size", "sha256", "payload"}:
            raise PilotError("source entry fields are invalid")
        if not isinstance(entry["path"], str):
            raise PilotError("source entry path is invalid")
        name = portable_path(entry["path"])
        _admit_source(name)
        if name != entry["path"] or not _in_scope(name, scopes) or name.casefold() in keys:
            raise PilotError("source entry is outside scope or duplicates another path")
        keys.add(name.casefold())
        names.append(name)
        parts = name.split("/")
        for index in range(1, len(parts) + 1):
            prefix = "/".join(parts[:index])
            prior = components.setdefault(prefix.casefold(), prefix)
            if prior != prefix:
                raise PilotError("source path components contain a case-insensitive collision")
        state = entry["status"]
        if not isinstance(state, str) or state not in {"base", "modified", "deleted", "new"} or not isinstance(entry["mode"], str) or entry["mode"] not in {"100644", "100755"}:
            raise PilotError("source entry kind is invalid")
        if state == "new":
            if entry["baseBlob"] is not None:
                raise PilotError("new source cannot claim a base blob")
        elif not isinstance(entry["baseBlob"], str) or not _GIT_OBJECT.fullmatch(entry["baseBlob"]):
            raise PilotError("source entry base blob is invalid")
        if type(entry["size"]) is not int or not 0 <= entry["size"] <= MAX_FILE_BYTES:
            raise PilotError("source entry size is invalid")
        if state == "deleted":
            if entry["size"] != 0 or entry["sha256"] is not None or entry["payload"] is not None:
                raise PilotError("deleted source must have no payload")
        else:
            sha256_identity(entry["sha256"], "source sha256")
            if entry["payload"] != (entry["sha256"] if full_payload or state in {"new", "modified"} else None):
                raise PilotError("source payload identity is invalid")
    if names != sorted(names) or any("/".join(name.casefold().split("/")[:index]) in keys for name in names for index in range(1, len(name.split("/")))):
        raise PilotError("source paths are not canonical or conflict with directories")


def _all_entries(manifest: dict[str, object]):
    yield from manifest["entries"]
    for sibling in manifest.get("external", []):
        yield from sibling["entries"]


def load_snapshot(bundle: Path, *, expected_input_hash: str | None = None) -> SourceSnapshot:
    bundle = require_managed_root(bundle, allow_root=False)
    raw = bundle.read_bytes()
    if len(raw) > MAX_BUNDLE_BYTES:
        raise PilotError("source archive exceeds the pilot transfer limit")
    try:
        with zipfile.ZipFile(io.BytesIO(raw)) as archive:
            members = archive.infolist()
            names = [member.filename for member in members]
            if len(names) != len(set(names)) or "manifest.json" not in names:
                raise PilotError("source archive has duplicate or missing manifest members")
            if archive.getinfo("manifest.json").file_size > MAX_MANIFEST_BYTES:
                raise PilotError("source manifest exceeds the pilot size limit")
            manifest = _validate_manifest(json.loads(archive.read("manifest.json")))
            if expected_input_hash is not None and manifest["inputHash"] != sha256_identity(expected_input_hash, "inputHash"):
                raise PilotError("sealed source belongs to a different input identity")
            payloads = {entry["payload"] for entry in _all_entries(manifest) if entry["payload"]}
            if set(names) != {"manifest.json", *[f"payloads/{value}" for value in payloads]}:
                raise PilotError("source archive contains unexpected or missing members")
            if sum(member.file_size for member in members) > MAX_BUNDLE_BYTES + MAX_MANIFEST_BYTES:
                raise PilotError("source archive expands beyond the pilot limit")
            for entry in _all_entries(manifest):
                if entry["payload"]:
                    data = archive.read(f"payloads/{entry['payload']}")
                    if len(data) != entry["size"] or hashlib.sha256(data).hexdigest() != entry["sha256"]:
                        raise PilotError("sealed source payload digest or size differs")
                    _admit_source(entry["path"], data)
    except (zipfile.BadZipFile, UnicodeError, json.JSONDecodeError, KeyError) as error:
        raise PilotError("source archive is invalid") from error
    return SourceSnapshot(bundle, manifest, hashlib.sha256(raw).hexdigest())


def materialize(bundle: Path, destination: Path, *, repo_root: Path, expected_input_hash: str) -> SourceSnapshot:
    sealed = load_snapshot(bundle, expected_input_hash=expected_input_hash)
    destination = require_managed_root(destination, allow_root=False)
    if destination.exists():
        raise PilotError("source materialization requires a fresh independent directory")
    if sealed.manifest["schemaVersion"] == 2:
        from .external_snapshot import preflight_destination
        preflight_destination(sealed.manifest["external"], destination)
    tree = _base_tree(Path(repo_root), sealed.manifest["baseCommit"])
    originals = _blobs(Path(repo_root), [entry["baseBlob"] for entry in sealed.manifest["entries"] if entry["status"] == "base"])
    storage = ManagedStorage(destination)
    with zipfile.ZipFile(sealed.bundle) as archive, storage.backend() as backend:
        for entry in sealed.manifest["entries"]:
            name = entry["path"]
            base_entry = tree.get(name)
            if (base_entry is None) != (entry["status"] == "new") or base_entry is not None and base_entry != (entry["mode"], entry["baseBlob"]):
                raise PilotError("source entry does not belong to the fixed Git base")
            if entry["status"] == "deleted":
                continue
            data = archive.read(f"payloads/{entry['payload']}") if entry["payload"] else originals[entry["baseBlob"]]
            if len(data) != entry["size"] or hashlib.sha256(data).hexdigest() != entry["sha256"]:
                raise PilotError("base or payload content does not match the sealed source digest")
            _admit_source(name, data)
            backend.write_bytes(name, data)
    # WorkerStorage creates its fixed support directories. They stay empty and
    # are outside the manifest; validation rejects any unexpected files in them.
    if sealed.manifest["schemaVersion"] == 2:
        from .external_snapshot import materialize_external
        materialize_external(sealed, destination)
    verify_materialized(sealed, destination)
    return sealed


def verify_materialized(sealed: SourceSnapshot, destination: Path) -> str:
    destination = require_managed_root(destination, allow_root=False)
    manifest = _validate_manifest(sealed.manifest)
    _verify_entries(manifest["entries"], destination)
    if manifest["schemaVersion"] == 2:
        from .external_snapshot import verify_external
        verify_external(manifest["external"], destination)
    return manifest["inputHash"]


def _verify_entries(entries: list[dict[str, object]], destination: Path) -> None:
    require_managed_root(destination, allow_root=False)
    if not destination.is_dir():
        raise PilotError("materialized source directory is missing")
    expected = {entry["path"] for entry in entries if entry["status"] != "deleted"}
    actual = set()
    for parent, directories, filenames in os.walk(destination, followlinks=False):
        for item in [*directories, *filenames]:
            path = Path(parent) / item
            require_managed_root(path, allow_root=False)
        actual.update((Path(parent) / name).relative_to(destination).as_posix() for name in filenames)
    if actual != expected:
        raise PilotError("materialized source contains missing or unexpected input files")
    for entry in entries:
        data = _read_plain(destination, entry["path"])
        if entry["status"] == "deleted":
            if data is not None:
                raise PilotError("deleted source reappeared in the validation input")
        elif data is None or len(data) != entry["size"] or hashlib.sha256(data).hexdigest() != entry["sha256"]:
            raise PilotError("materialized source differs from its sealed input")
