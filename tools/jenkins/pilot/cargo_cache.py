"""Exclusive reusable Cargo generations with persistent fail-closed ownership."""
from __future__ import annotations
from contextlib import ExitStack
import json
import hashlib
from pathlib import Path
import shutil
import time
from .contracts import PilotError, canonical_json, digest
from .storage import ManagedStorage, physical_identity, require_managed_root
from .native.paths import WorkerStorage

MIN_FREE_BYTES = 35 * 1024 ** 3
WRITABLE_DIRECTORIES = ("target", "cargo-home", "build", "scratch", "sccache")
GENERATION_LEAF_LENGTH = 20
WINDOWS_LINK_PATH_LIMIT = 260
# Fixed package closure: longest build-script package is proc-macro2. Reserve
# additional room beyond the observed .exe/.pdb output for linker siblings.
FIXED_LINK_OUTPUT = "build/debug/build/proc-macro2-0123456789abcdef/build_script_build-0123456789abcdef.pdb"
LINK_OUTPUT_RESERVE = 12


def generation_path(root: Path, key: str) -> Path:
    return root / "cargo-generations" / key[:GENERATION_LEAF_LENGTH] / "g1"


def require_link_path_budget(path: Path) -> int:
    length = len(str(path.joinpath(*FIXED_LINK_OUTPUT.split("/")))) + LINK_OUTPUT_RESERVE
    if length >= WINDOWS_LINK_PATH_LIMIT:
        raise PilotError("fixed Cargo linker output exceeds the native Windows path budget")
    return length


def pin_writable_tree(backend, path: Path, pins: ExitStack):
    """Reject every existing descendant alias, then retain all directory handles."""
    for name in WRITABLE_DIRECTORIES:
        directories = (backend.list_compiled_directories(name, allowed_directories=("build",))
                       if name == "build" else backend.list_directories(name))
        for relative in sorted(directories, key=lambda item: (item.count("/"), item)):
            pins.enter_context(WorkerStorage(path.joinpath(*relative.split("/")), create=False))
        # The second native traversal closes the scan-to-pin window and checks
        # all files for reparse points, nonregular leaves, and multiple links.
        if name == "build":
            backend.list_compiled_products(name, allowed_directories=("build",))
        else:
            backend.list_files(name)


def require_capacity(root: Path) -> int:
    free = shutil.disk_usage(require_managed_root(root, allow_root=False)).free
    if free < MIN_FREE_BYTES:
        raise PilotError("Cargo requires at least 35 GiB free on the physical managed volume")
    return free


class CargoGeneration:
    def __init__(self, root: Path, compatibility: dict, identity: dict):
        self.root = require_managed_root(root, allow_root=False)
        self.key = digest(compatibility)
        self.path = generation_path(self.root, self.key)
        require_link_path_budget(self.path)
        self.storage = ManagedStorage(self.path)
        self.compatibility, self.identity = compatibility, identity
        self.lock = None
        self.locked = False
        self.reused = False
        self.pins = ExitStack()

    def acquire(self):
        import msvcrt
        try:
            self.root_backend = self.pins.enter_context(ManagedStorage(self.root).backend())
            self.root_backend.ensure_directory(self.path.relative_to(self.root).as_posix())
            self.backend = self.pins.enter_context(self.storage.backend())
            require_capacity(self.root)
            self.lock = self.pins.enter_context(self.backend.open_lock_file("writer.lock"))
            if self.lock.seek(0, 2) == 0:
                self.lock.write(b"0")
            self.lock.seek(0)
            msvcrt.locking(self.lock.fileno(), msvcrt.LK_NBLCK, 1)
            self.locked = True
            old = json.loads(self.backend.read_bytes("state.json")) if self.backend.exists("state.json") else None
            if old is not None:
                if old.get("compatibilityKey") != self.key:
                    raise PilotError("Cargo short physical key collision; full compatibility key differs")
                if old.get("compatibility") != self.compatibility or old.get("state") != "released":
                    raise PilotError("Cargo generation has an unknown/nonterminal owner; reconciliation required")
                release = json.loads(self.backend.read_bytes(old["releaseRecord"]))
                if digest(release) != old.get("releaseRecordHash") or release.get("terminalBinding") != old.get("terminalBinding"):
                    raise PilotError("Cargo generation has no durable exact terminal release proof")
                self.reused = True
            for name in WRITABLE_DIRECTORIES:
                self.backend.ensure_directory(name)
                self.pins.enter_context(WorkerStorage(self.path / name, create=False))
            pin_writable_tree(self.backend, self.path, self.pins)
            self.identities = {name: physical_identity(self.storage.path(name))
                               for name in WRITABLE_DIRECTORIES}
            self.root_identity = physical_identity(self.root)
            self.generation_identity = physical_identity(self.path)
            if old is not None and (old.get("physicalIdentities") != self.identities or
                                    old.get("rootIdentity") != self.root_identity or
                                    old.get("generationIdentity") != self.generation_identity):
                raise PilotError("Cargo physical identity changed; cache adoption forbidden")
            self.state = {"schemaVersion": 2, "state": "running", "identity": self.identity,
                "compatibility": self.compatibility, "compatibilityKey": self.key, "generation": 1,
                "rootPath": str(self.root), "generationPath": str(self.path),
                "rootIdentity": self.root_identity, "generationIdentity": self.generation_identity,
                "physicalIdentities": self.identities, "cacheReused": self.reused}
            if old is not None and "registrySeedHash" in old:
                seed_bytes = self.backend.read_bytes("registry-seed.json")
                if digest(json.loads(seed_bytes)) != old["registrySeedHash"]:
                    raise PilotError("reused registry seed provenance changed")
                self.state["registrySeedHash"] = old["registrySeedHash"]
            self.backend.write_bytes("state.json", canonical_json(self.state))
            return self
        except Exception:
            self.close()
            raise

    def verify(self):
        self.root_backend.root_identity
        self.backend.root_identity
        if physical_identity(self.root) != self.root_identity or physical_identity(self.path) != self.generation_identity:
            raise PilotError("Cargo root identity changed")
        if any(physical_identity(self.storage.path(name)) != value for name, value in self.identities.items()):
            raise PilotError("Cargo output identity changed")
        for name in WRITABLE_DIRECTORIES:
            if name == "build":
                self.backend.list_compiled_products(name, allowed_directories=("build",))
            else:
                self.backend.list_files(name)

    def release(self, terminal_binding: dict):
        if not self.locked:
            raise PilotError("Cargo writer lock was not retained")
        self.verify()
        self.state.update({"state": "released", "terminalBinding": terminal_binding, "releasedAtNs": time.time_ns()})
        release_name = f"releases/{digest(terminal_binding)}.json"
        if self.backend.exists(release_name):
            raise PilotError("immutable Cargo release proof already exists")
        self.backend.write_bytes(release_name, canonical_json(self.state))
        self.state.update({"releaseRecord": release_name, "releaseRecordHash": digest(self.state)})
        self.backend.write_bytes("state.json", canonical_json(self.state))
        self.close()

    def close(self):
        if self.lock is not None:
            if self.locked:
                import msvcrt
                self.lock.seek(0)
                msvcrt.locking(self.lock.fileno(), msvcrt.LK_UNLCK, 1)
            self.lock = None
            self.locked = False
        self.pins.close()


def seed_independent_registry(generation: CargoGeneration, source_path: Path) -> dict:
    """Copy only registry files from a released independent sibling generation."""
    if not generation.locked or generation.reused:
        raise PilotError("registry seed requires a newly owned exclusive generation")
    source_path = require_managed_root(source_path, allow_root=False)
    if source_path.parent.parent != generation.root / "cargo-generations" or source_path.name != "g1" or source_path == generation.path:
        raise PilotError("registry seed must be a released independent sibling generation")
    with ExitStack() as pins:
        source = pins.enter_context(WorkerStorage(source_path, create=False))
        import msvcrt
        source_lock = pins.enter_context(source.open_lock_file("writer.lock"))
        source_lock.seek(0)
        msvcrt.locking(source_lock.fileno(), msvcrt.LK_NBLCK, 1)
        def unlock_source():
            source_lock.seek(0)
            msvcrt.locking(source_lock.fileno(), msvcrt.LK_UNLCK, 1)
        pins.callback(unlock_source)
        state_bytes = source.read_bytes("state.json")
        state = json.loads(state_bytes)
        release_bytes = source.read_bytes(state["releaseRecord"])
        release = json.loads(release_bytes)
        if state.get("state") != "released" or release.get("state") != "released" or digest(release) != state.get("releaseRecordHash") or release.get("terminalBinding") != state.get("terminalBinding"):
            raise PilotError("registry seed source lacks exact independent terminal release")
        if state.get("generationPath") != str(source_path) or physical_identity(source_path) != state.get("generationIdentity") or state.get("rootIdentity") != generation.root_identity:
            raise PilotError("registry seed source physical ownership differs")
        if state.get("identity", {}).get("sessionId") != generation.identity.get("sessionId"):
            raise PilotError("registry seed source belongs to another task owner")
        run_root = require_managed_root(release["terminalBinding"]["runRoot"], allow_root=False)
        if not run_root.is_relative_to(generation.root):
            raise PilotError("registry seed receipt is outside this independent task root")
        run = pins.enter_context(WorkerStorage(run_root, create=False))
        request_bytes = run.read_bytes("managed-request.json")
        result_bytes = run.read_bytes("managed-result.json")
        request = json.loads(request_bytes)
        result = json.loads(result_bytes)
        from .managed_cargo import terminal_verified
        binding = release["terminalBinding"]
        if (digest(request) != binding["requestHash"] or digest(result["observation"]) != binding["observationHash"] or
                result.get("terminalBinding") != binding or result.get("lockReleased") is not True or
                result.get("inputsUnchanged") is not True or not terminal_verified(result["observation"]) or
                request.get("identity") != state.get("identity")):
            raise PilotError("registry seed lacks exact native request/observation ownership")
        command = request.get("command", [])
        if (result["observation"].get("command") != command or result["observation"].get("commandHash") != digest(command) or
                result.get("releaseRecord") != state.get("releaseRecord") or result.get("releaseRecordHash") != state.get("releaseRecordHash") or
                request.get("generation", {}).get("generationIdentity") != state.get("generationIdentity")):
            raise PilotError("registry seed native command/release generation binding differs")
        source_directories = source.list_directories("cargo-home/registry")
        for relative in source_directories:
            pins.enter_context(WorkerStorage(source_path.joinpath(*relative.split("/")), create=False))
        records = {}
        generation.backend.ensure_directory("cargo-home/registry")
        if generation.backend.list_files("cargo-home/registry"):
            raise PilotError("registry seed destination already contains files")
        source_files = source.list_files("cargo-home/registry")
        for relative in source_files:
            if not relative.startswith(("cargo-home/registry/index/", "cargo-home/registry/cache/")):
                continue
            if relative.endswith(("/.cargo-lock", "/.package-cache")):
                continue
            if generation.backend.exists(relative):
                raise PilotError("registry seed destination is not empty")
            payload = source.read_bytes(relative)
            identity = hashlib.sha256(payload).hexdigest()
            generation.backend.write_bytes(relative, payload, expected_sha256=identity)
            records[relative] = identity
        if any(source.sha256(relative)[0] != identity for relative, identity in records.items()):
            raise PilotError("registry seed source changed during pinned copy")
        if (source.read_bytes("state.json") != state_bytes or source.read_bytes(state["releaseRecord"]) != release_bytes or
                run.read_bytes("managed-request.json") != request_bytes or run.read_bytes("managed-result.json") != result_bytes or
                source.list_files("cargo-home/registry") != source_files or source.list_directories("cargo-home/registry") != source_directories):
            raise PilotError("registry seed source proof or inventory changed during exclusive copy")
        proof = {"schemaVersion": 2, "sourceGeneration": str(source_path),
                 "sourceIdentity": state["generationIdentity"], "sourceReleaseSha256": hashlib.sha256(release_bytes).hexdigest(),
                 "destinationGeneration": str(generation.path), "destinationOwner": generation.identity,
                 "files": records, "filesHash": digest(records)}
        generation.backend.write_bytes("registry-seed.json", canonical_json(proof))
        generation.backend.write_bytes("state.json", canonical_json({**generation.state, "registrySeedHash": digest(proof)}))
        generation.state["registrySeedHash"] = digest(proof)
        pin_writable_tree(generation.backend, generation.path, generation.pins)
        return proof
