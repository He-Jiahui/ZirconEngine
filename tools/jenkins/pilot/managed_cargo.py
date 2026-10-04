"""Fixed direct Cargo execution and independently checked native terminal evidence."""
from __future__ import annotations
from contextlib import ExitStack, contextmanager
import hashlib
import json
import os
from pathlib import Path
import sys
import time
import tomllib
from .contracts import PilotError, RequestIdentity, canonical_json, digest
from .source_manifest import file_hash, manifest_digest, tree_manifest
from .storage import ManagedStorage, physical_identity, require_managed_root
from .cargo_cache import CargoGeneration, require_capacity, WRITABLE_DIRECTORIES, pin_writable_tree, generation_path, require_link_path_budget, seed_independent_registry
from .native.paths import WorkerStorage

FIXED_PACKAGE = "zircon_reflect_derive"
TEMPLATE = "managed-cargo-check-v2"


def executor_manifest(repo_root: Path) -> dict[str, str]:
    paths = sorted((repo_root / "tools/jenkins_pilot").rglob("*.py"))
    return {p.relative_to(repo_root).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in paths if "tests" not in p.relative_to(repo_root).parts}


def toolchain_identity(source: Path) -> dict:
    """Resolve installed binaries directly; never launch rustup or install a toolchain."""
    rustup = Path(os.environ.get("RUSTUP_HOME", str(Path.home() / ".rustup")))
    settings_path = rustup / "settings.toml"
    settings = tomllib.loads(settings_path.read_text(encoding="utf-8"))
    channel = settings.get("default_toolchain")
    selected = source / "rust-toolchain.toml"
    if selected.exists():
        channel = tomllib.loads(selected.read_text(encoding="utf-8"))["toolchain"]["channel"]
    elif (source / "rust-toolchain").exists():
        channel = (source / "rust-toolchain").read_text(encoding="utf-8").strip()
    host = settings.get("default_host_triple", "x86_64-pc-windows-msvc")
    candidates = [rustup / "toolchains" / str(channel), rustup / "toolchains" / f"{channel}-{host}"]
    selected_root = next((p for p in candidates if (p / "bin/cargo.exe").is_file()), None)
    if selected_root is None:
        raise PilotError("fixed Cargo toolchain must already be installed")
    binaries = {}
    for name in ("cargo", "rustc", "rustdoc"):
        path = selected_root / "bin" / f"{name}.exe"
        binaries[name] = {"path": str(path), "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}
    libraries = {p.relative_to(selected_root).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest()
                 for p in sorted((selected_root / "bin").glob("*.dll"))}
    sysroot_inputs = {}
    for directory in (selected_root / "lib/rustlib" / host / "lib",
                      selected_root / "lib/rustlib" / host / "bin", selected_root / "lib/rustlib/bin"):
        for path in sorted(directory.rglob("*")):
            if path.is_file():
                if path.is_symlink() or getattr(path.lstat(), "st_file_attributes", 0) & 0x400:
                    raise PilotError("toolchain sysroot inputs cannot be reparse aliases")
                sysroot_inputs[path.relative_to(selected_root).as_posix()] = hashlib.sha256(path.read_bytes()).hexdigest()
    if not any(name.startswith(f"lib/rustlib/{host}/lib/") for name in sysroot_inputs):
        raise PilotError("installed host sysroot libraries are required")
    return {"channel": channel, "host": host, "binaries": binaries, "libraries": libraries,
            "sysrootInputs": sysroot_inputs,
            "python": {"path": sys.executable, "sha256": hashlib.sha256(Path(sys.executable).read_bytes()).hexdigest()},
            "settingsSha256": hashlib.sha256(settings_path.read_bytes()).hexdigest()}


def configuration_identity(source: Path) -> dict:
    config = source / ".cargo/config.toml"
    if (source / ".cargo/config").exists():
        raise PilotError("legacy Cargo config unsupported in the fixed executor")
    for parent in source.parents:
        if any((parent / ".cargo" / name).exists() for name in ("config", "config.toml")):
            raise PilotError("ambient ancestor Cargo configuration is forbidden")
    parsed = tomllib.loads(config.read_text(encoding="utf-8")) if config.exists() else {}
    if parsed:
        raise PilotError("Cargo config contains unreviewed build/environment overrides")
    return {"files": {".cargo/config.toml": hashlib.sha256(config.read_bytes()).hexdigest()} if config.exists() else {},
            "locked": True, "package": FIXED_PACKAGE, "profile": "dev", "wrappers": "disabled"}


def terminal_verified(observation: dict) -> bool:
    tree = observation.get("processTree", {})
    job = observation.get("nativeJobEvidence", {}) or {}
    final = job.get("records", {}).get("afterClose", {})
    return (tree.get("terminal") is True and tree.get("readersFinished") is True and
            tree.get("pipeEOF") is True and type(tree.get("pid")) is int and tree["pid"] > 0 and
            isinstance(tree.get("creationTime"), str) and tree["creationTime"].isdigit() and
            type(final.get("activeProcesses")) is int and final["activeProcesses"] == 0 and
            not job.get("errors") and not observation.get("terminalErrors") and
            len(tree.get("pipes", {})) == 2 and all(p.get("eof") is True and "error" not in p
                for p in tree["pipes"].values()))


def execution_environment(toolchain: dict, generation: Path) -> dict[str, str]:
    names = ("SYSTEMROOT", "WINDIR", "COMSPEC", "PATH", "PATHEXT", "USERPROFILE", "APPDATA", "LOCALAPPDATA",
             "INCLUDE", "LIB", "LIBPATH", "VCTOOLSINSTALLDIR", "WINDOWSSDKDIR", "WINDOWSSDKVERSION")
    environment = {k: v for k, v in os.environ.items() if k.upper() in names}
    environment.update({"CARGO_HOME": str(generation / "cargo-home"), "CARGO_TARGET_DIR": str(generation / "target"),
        "CARGO_BUILD_BUILD_DIR": str(generation / "build"), "TEMP": str(generation / "scratch"),
        "TMP": str(generation / "scratch"), "TMPDIR": str(generation / "scratch"),
        "SCCACHE_DIR": str(generation / "sccache"), "RUSTC_WRAPPER": "", "RUSTC_WORKSPACE_WRAPPER": "",
        "RUSTC": toolchain["binaries"]["rustc"]["path"], "RUSTDOC": toolchain["binaries"]["rustdoc"]["path"],
        "CARGO_INCREMENTAL": "1", "CARGO_HTTP_PROXY": ""})
    return environment


@contextmanager
def metadata_execution(source: Path, target: Path, *, input_binding: dict | None = None):
    """Pin every metadata output directory and exclusive cache lock through transport."""
    import msvcrt
    generation = require_managed_root(target, allow_root=False)
    with ExitStack() as pins:
        parent = pins.enter_context(ManagedStorage(generation.parent).backend())
        parent.create_directory(generation.name)
        backend = pins.enter_context(ManagedStorage(generation).backend())
        require_capacity(generation)
        for name in WRITABLE_DIRECTORIES:
            backend.ensure_directory(name)
            pins.enter_context(WorkerStorage(generation / name, create=False))
        pin_writable_tree(backend, generation, pins)
        lock = pins.enter_context(backend.open_lock_file("metadata-writer.lock"))
        if lock.seek(0, 2) == 0:
            lock.write(b"0")
        lock.seek(0)
        msvcrt.locking(lock.fileno(), msvcrt.LK_NBLCK, 1)
        try:
            toolchain = toolchain_identity(source)
            configuration_identity(source)
            environment = execution_environment(toolchain, generation)
            command = (toolchain["binaries"]["cargo"]["path"], "metadata", "--locked", "--format-version", "1",
                       "--filter-platform", toolchain["host"])
            input_files = {name: file_hash(source / name) for name in
                ("Cargo.toml", "Cargo.lock", "rust-toolchain", "rust-toolchain.toml", ".cargo/config.toml")
                if (source / name).is_file()}
            selected_inputs = dict(input_binding or {})
            if any(file_hash(Path(name)) != identity for name, identity in selected_inputs.items()):
                raise PilotError("selected metadata inputs changed before native transport")
            state = {"schemaVersion": 2, "state": "running", "command": list(command),
                     "commandHash": digest(list(command)), "sourceRoot": str(source),
                     "metadataInputFiles": input_files, "selectedInputBinding": selected_inputs,
                     "selectedInputHash": digest(selected_inputs),
                     "toolchain": toolchain, "generationIdentity": physical_identity(generation)}
            backend.write_bytes("metadata-state.json", canonical_json(state))
            class Binding:
                def complete(self, observation):
                    if (not terminal_verified(observation) or observation.get("command") != list(command) or
                            observation.get("commandHash") != digest(list(command)) or type(observation.get("exitCode")) is not int):
                        raise PilotError("metadata native terminal is unknown; fresh generation remains blocked")
                    for name in WRITABLE_DIRECTORIES:
                        backend.list_files(name)
                    if any(file_hash(source / name) != identity for name, identity in input_files.items()):
                        raise PilotError("metadata source changed during native transport")
                    if any(file_hash(Path(name)) != identity for name, identity in selected_inputs.items()):
                        raise PilotError("selected metadata inputs changed during native transport")
                    state.update({"state": "released", "observation": observation,
                                  "observationHash": digest(observation)})
                    backend.write_bytes("metadata-state.json", canonical_json(state))
            binding = Binding()
            binding.command, binding.environment = command, environment
            yield binding
        finally:
            lock.seek(0)
            msvcrt.locking(lock.fileno(), msvcrt.LK_UNLCK, 1)


def metadata_invocation(source: Path, target: Path):
    raise PilotError("metadata requires metadata_execution pins retained through actual transport")


class CargoExecution:
    def __init__(self, *, repo_root: Path, source_root: Path, run_root: Path,
                 identity: RequestIdentity, managed_root: Path):
        if identity.template != TEMPLATE:
            raise PilotError("retired Cargo template cannot execute or be silently rebound")
        self.started = time.monotonic()
        self.repo_root = repo_root
        self.source = require_managed_root(source_root, allow_root=False)
        self.run_root = require_managed_root(run_root, allow_root=False)
        if self.source != self.run_root / "sealed/source" or not (self.source / "Cargo.lock").is_file():
            raise PilotError("Cargo requires immutable selected workspace and Cargo.lock")
        self.identity = identity
        self.source_digest = manifest_digest(tree_manifest(self.source.parent))
        self.toolchain = toolchain_identity(self.source)
        self.configuration = configuration_identity(self.source)
        self.driver = executor_manifest(repo_root)
        compatibility = {"schemaVersion": 2, "toolchain": self.toolchain, "config": self.configuration,
                         "networkPolicy": {"httpProxy": ""},
                         "discoveryEnvironment": {k: v for k, v in execution_environment(self.toolchain, Path("D:/cargo-targets/placeholder")).items()
                                                  if k.upper() in {"PATH", "INCLUDE", "LIB", "LIBPATH", "VCTOOLSINSTALLDIR", "WINDOWSSDKDIR", "WINDOWSSDKVERSION"}},
                         "command": ["check", "--locked", "-p", FIXED_PACKAGE],
                         "workspaceLock": hashlib.sha256((self.source / "Cargo.lock").read_bytes()).hexdigest()}
        self.generation = CargoGeneration(managed_root, compatibility, identity.to_dict()).acquire()
        try:
            self.command = [self.toolchain["binaries"]["cargo"]["path"], "check", "--locked", "-p", FIXED_PACKAGE,
                            "--target-dir", str(self.generation.path / "target")]
            self.environment = execution_environment(self.toolchain, self.generation.path)
            self.request = {"schemaVersion": 2, "identity": identity.to_dict(), "sourceRoot": str(self.source),
                "sourceDigest": self.source_digest, "command": self.command, "commandHash": digest(self.command),
                "executorManifest": self.driver, "toolchain": self.toolchain, "configuration": self.configuration,
                "generation": dict(self.generation.state), "environment": self.environment,
                "prepareSeconds": time.monotonic() - self.started}
            ManagedStorage(self.run_root).atomic_write("managed-request.json", canonical_json(self.request))
        except Exception:
            self.generation.close()
            raise

    def finalize(self, observation: dict) -> dict:
        terminal = terminal_verified(observation)
        if observation.get("command") != self.command or observation.get("commandHash") != digest(self.command) or type(observation.get("exitCode")) is not int:
            terminal = False
        input_error = None
        try:
            unchanged = (executor_manifest(self.repo_root) == self.driver and
                manifest_digest(tree_manifest(self.source.parent)) == self.source_digest and
                toolchain_identity(self.source) == self.toolchain and configuration_identity(self.source) == self.configuration)
        except (ValueError, OSError, KeyError) as error:
            unchanged, input_error = False, str(error)
        result = {"schemaVersion": 2, "requestHash": digest(self.request), "identity": self.identity.to_dict(),
                  "observation": observation, "terminalVerified": terminal, "inputsUnchanged": unchanged,
                  "lockReleased": False, "cacheReused": self.generation.reused,
                  "inputError": input_error,
                  "elapsedSeconds": time.monotonic() - self.started, "prepareSeconds": self.request["prepareSeconds"]}
        storage = ManagedStorage(self.run_root)
        storage.atomic_write("managed-result.json", canonical_json(result))
        if terminal:
            binding = {"requestHash": digest(self.request), "observationHash": digest(observation), "runRoot": str(self.run_root)}
            self.generation.release(binding)
            result.update({"lockReleased": True, "terminalBinding": binding})
            result["releaseRecord"] = self.generation.state["releaseRecord"]
            result["releaseRecordHash"] = self.generation.state["releaseRecordHash"]
            storage.atomic_write("managed-result.json", canonical_json(result))
        return result

    def close(self):
        self.generation.close()

    def seed_registry(self, released_source: Path):
        proof = seed_independent_registry(self.generation, released_source)
        ManagedStorage(self.run_root).atomic_write("registry-seed.json", canonical_json(proof))
        self.request["generation"] = dict(self.generation.state)
        self.request["registrySeed"] = {"sha256": hashlib.sha256(canonical_json(proof)).hexdigest(), "proofHash": digest(proof)}
        ManagedStorage(self.run_root).atomic_write("managed-request.json", canonical_json(self.request))
        return proof


def prepare_execution(**kwargs) -> CargoExecution:
    return CargoExecution(**kwargs)


def validation_command(**kwargs) -> list[str]:
    raise PilotError("Cargo v2 requires prepare_execution with retained generation ownership")


def terminal_reconcile(*, run_root: Path, expected_identity: RequestIdentity,
                       original_driver_root: Path, audit_driver_root: Path) -> dict:
    """Release only an exact recorded terminal Cargo run; original outcome stays failed."""
    import msvcrt
    with ExitStack() as pins:
        run_root = require_managed_root(run_root, allow_root=False)
        run = pins.enter_context(WorkerStorage(run_root, create=False))
        request_bytes, result_bytes = run.read_bytes("managed-request.json"), run.read_bytes("managed-result.json")
        receipt_bytes = run.read_bytes("receipt.json")
        request, result, receipt = json.loads(request_bytes), json.loads(result_bytes), json.loads(receipt_bytes)
        identity = expected_identity.to_dict()
        observation = result["observation"]
        fields = ("command", "commandHash", "exitCode", "processTree", "nativeJobEvidence")
        errors = receipt.get("terminalErrors")
        expected_collector_error = {"stage": "managed_cargo_collector", "type": "StorageSecurityError",
                                    "message": "worker file has a hard-link alias"}
        collector_failure = (errors == [expected_collector_error] and
            receipt.get("reason") == "managed_cargo_collector_failed" and
            receipt.get("managedCargo", {}).get("accepted") is False and
            receipt.get("managedCargo", {}).get("error") == expected_collector_error)
        if (request.get("identity") != identity or result.get("identity") != identity or receipt.get("identity") != identity or
                expected_identity.template != TEMPLATE or result.get("requestHash") != digest(request) or
                result.get("terminalVerified") is not True or result.get("inputsUnchanged") is not True or
                result.get("lockReleased") is not False or receipt.get("outcome") != "failed" or
                not collector_failure or observation.get("terminalErrors") != [] or
                not terminal_verified(observation) or observation.get("exitCode") != 0 or
                any(receipt.get(name) != observation.get(name) for name in fields) or
                observation.get("command") != request.get("command") or observation.get("commandHash") != digest(request["command"])):
            raise PilotError("reconciliation requires the exact failed receipt with proven native Cargo terminal")
        old_driver = executor_manifest(original_driver_root)
        if old_driver != request["executorManifest"]:
            raise PilotError("original sealed driver changed")
        source = require_managed_root(request["sourceRoot"], allow_root=False)
        if source != run_root / "sealed/source" or manifest_digest(tree_manifest(source.parent)) != request["sourceDigest"] or toolchain_identity(source) != request["toolchain"] or configuration_identity(source) != request["configuration"]:
            raise PilotError("original immutable inputs/toolchain changed")
        generation = request["generation"]
        root = require_managed_root(generation["rootPath"], allow_root=False)
        path = require_managed_root(generation["generationPath"], allow_root=False)
        if path != generation_path(root, generation["compatibilityKey"]) or digest(generation["compatibility"]) != generation["compatibilityKey"] or not run_root.is_relative_to(root):
            raise PilotError("reconciliation generation path/compatibility differs")
        backend = pins.enter_context(WorkerStorage(path, create=False))
        lock = pins.enter_context(backend.open_lock_file("writer.lock"))
        lock.seek(0)
        msvcrt.locking(lock.fileno(), msvcrt.LK_NBLCK, 1)
        def unlock():
            lock.seek(0)
            msvcrt.locking(lock.fileno(), msvcrt.LK_UNLCK, 1)
        pins.callback(unlock)
        state_bytes = backend.read_bytes("state.json")
        state = json.loads(state_bytes)
        if state != generation or state.get("state") != "running":
            raise PilotError("reconciliation state differs from original exclusive generation")
        if physical_identity(root) != generation["rootIdentity"] or physical_identity(path) != generation["generationIdentity"] or any(physical_identity(path / name) != value for name, value in generation["physicalIdentities"].items()):
            raise PilotError("reconciliation physical generation changed")
        for name in WRITABLE_DIRECTORIES:
            pins.enter_context(WorkerStorage(path / name, create=False))
        pin_writable_tree(backend, path, pins)
        if "registrySeedHash" in generation and digest(json.loads(backend.read_bytes("registry-seed.json"))) != generation["registrySeedHash"]:
            raise PilotError("reconciliation registry seed changed")
        compiled = backend.list_compiled_products("build", allowed_directories=("build",))
        proof = {"schemaVersion": 2, "acceptance": "terminal-release-reconciliation-only",
            "identity": identity, "requestHash": digest(request), "observationHash": digest(observation),
            "originalResultSha256": hashlib.sha256(result_bytes).hexdigest(),
            "originalReceiptSha256": hashlib.sha256(receipt_bytes).hexdigest(),
            "originalDriverManifest": old_driver, "auditDriverManifest": executor_manifest(audit_driver_root),
            "generationIdentity": generation["generationIdentity"], "compiledProducts": compiled,
            "originalOutcome": "failed", "formalAcceptance": False}
        proof_name = f"reconciliations/{digest(proof)}.json"
        binding = {"requestHash": digest(request), "observationHash": digest(observation), "runRoot": str(run_root),
                   "reconciliationHash": digest(proof)}
        release_name = f"releases/{digest(binding)}.json"
        if backend.exists(proof_name) or backend.exists(release_name):
            raise PilotError("immutable reconciliation already exists")
        if run.read_bytes("managed-request.json") != request_bytes or run.read_bytes("managed-result.json") != result_bytes or run.read_bytes("receipt.json") != receipt_bytes or backend.read_bytes("state.json") != state_bytes:
            raise PilotError("original reconciliation evidence changed during audit")
        released = {**state, "state": "released", "terminalBinding": binding, "releasedAtNs": time.time_ns(),
                    "reconciliationRecord": proof_name, "reconciliationHash": digest(proof)}
        backend.write_bytes(proof_name, canonical_json(proof))
        backend.write_bytes(release_name, canonical_json(released))
        backend.write_bytes("state.json", canonical_json({**released, "releaseRecord": release_name, "releaseRecordHash": digest(released)}))
        return proof


def collect_evidence(*, repo_root: Path, run_root: Path, exit_code: int, observation: dict | None = None, persist: bool = True) -> dict:
    try:
        storage = ManagedStorage(run_root)
        request = json.loads(storage.path("managed-request.json").read_bytes())
        result = json.loads(storage.path("managed-result.json").read_bytes())
        identity = RequestIdentity.from_dict(request["identity"])
        if identity.template != TEMPLATE or request["schemaVersion"] != 2 or result["schemaVersion"] != 2:
            raise PilotError("managed Cargo schema/template differs")
        if observation is None or result["observation"] != observation or not terminal_verified(observation):
            raise PilotError("independent runner native terminal observation required")
        if type(exit_code) is not int or observation.get("exitCode") != exit_code or observation.get("command") != request["command"] or observation.get("commandHash") != digest(request["command"]):
            raise PilotError("actual native Cargo command/result differs")
        source = require_managed_root(request["sourceRoot"], allow_root=False)
        if source != run_root / "sealed/source" or manifest_digest(tree_manifest(source.parent)) != request["sourceDigest"]:
            raise PilotError("immutable source differs")
        if executor_manifest(repo_root) != request["executorManifest"] or toolchain_identity(source) != request["toolchain"] or configuration_identity(source) != request["configuration"]:
            raise PilotError("trusted executor/toolchain/config differs")
        generation = request["generation"]
        if generation["compatibilityKey"] != digest(generation["compatibility"]):
            raise PilotError("cache compatibility key differs")
        directory = require_managed_root(generation["generationPath"], allow_root=False)
        expected_directory = generation_path(require_managed_root(generation["rootPath"], allow_root=False), generation["compatibilityKey"])
        require_link_path_budget(expected_directory)
        expected_command = [request["toolchain"]["binaries"]["cargo"]["path"], "check", "--locked", "-p", FIXED_PACKAGE,
                            "--target-dir", str(directory / "target")]
        if directory != expected_directory or request["command"] != expected_command or request["commandHash"] != digest(expected_command):
            raise PilotError("Cargo command or generation path is not the fixed execution contract")
        if request["environment"] != execution_environment(request["toolchain"], directory):
            raise PilotError("Cargo execution environment differs")
        if "registrySeedHash" in generation:
            seed_bytes = ManagedStorage(directory).path("registry-seed.json").read_bytes()
            seed = json.loads(seed_bytes)
            if (digest(seed) != generation["registrySeedHash"] or seed.get("destinationGeneration") != str(directory) or
                    seed.get("destinationOwner", {}).get("sessionId") != identity.session_id or
                    seed.get("filesHash") != digest(seed.get("files")) or
                    not Path(seed["sourceGeneration"]).is_relative_to(Path(generation["rootPath"]) / "cargo-generations") or
                    any(not name.startswith(("cargo-home/registry/index/", "cargo-home/registry/cache/")) for name in seed["files"])):
                raise PilotError("registry seed provenance differs from current execution request")
            if "registrySeed" in request and (storage.path("registry-seed.json").read_bytes() != seed_bytes or
                    hashlib.sha256(seed_bytes).hexdigest() != request["registrySeed"]["sha256"] or
                    digest(seed) != request["registrySeed"]["proofHash"] or seed.get("destinationOwner") != request["identity"]):
                raise PilotError("current run registry seed provenance differs")
        state = json.loads(ManagedStorage(directory).path(result["releaseRecord"]).read_bytes())
        binding = {"requestHash": digest(request), "observationHash": digest(observation), "runRoot": str(run_root)}
        if result["requestHash"] != digest(request) or result["identity"] != identity.to_dict() or result.get("lockReleased") is not True or result.get("inputsUnchanged") is not True or result.get("terminalVerified") is not True or result.get("terminalBinding") != binding:
            raise PilotError("Cargo durable terminal receipt incomplete")
        if state.get("state") != "released" or state.get("terminalBinding") != binding or state.get("identity") != identity.to_dict():
            raise PilotError("Cargo generation lacks matching release proof")
        if digest(state) != result["releaseRecordHash"] or result["releaseRecord"] != f"releases/{digest(binding)}.json":
            raise PilotError("Cargo release proof identity differs")
        if state.get("registrySeedHash") != generation.get("registrySeedHash"):
            raise PilotError("immutable release registry seed hash differs")
        if physical_identity(Path(generation["rootPath"])) != generation["rootIdentity"] or physical_identity(directory) != generation["generationIdentity"] or any(physical_identity(directory / name) != value for name, value in generation["physicalIdentities"].items()):
            raise PilotError("Cargo physical generation identity differs")
        proof = {"accepted": exit_code == 0, "formalAcceptance": False, "sourceDigest": request["sourceDigest"],
                 "compatibilityKey": generation["compatibilityKey"], "generation": generation["generation"],
                 "targetDirectory": str(directory / "target"), "receipt": result}
        if persist:
            storage.atomic_write("managed-evidence.json", canonical_json(proof))
        return proof
    except (ValueError, KeyError, TypeError, OSError, AttributeError) as error:
        return {"accepted": False, "formalAcceptance": False, "reason": str(error)}
