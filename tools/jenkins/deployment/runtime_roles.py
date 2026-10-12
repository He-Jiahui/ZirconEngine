"""Explicit, immutable review of named clients sharing the dedicated Python.

The default owner inventory remains conservative. Only this issued capability
separates exact reviewed clients; journals, service commands and unknown
processes always retain ownership priority. No process is adopted or killed.
"""
from __future__ import annotations

from dataclasses import dataclass
import hashlib
import json
import os
from pathlib import Path

from ..contracts import JenkinsError, canonical_json, digest, require_digest
from ..processes.boot import windows_boot_filetime
from ..resources.paths import build_root_containing, physical_path_under
from .driver import _plain, verify_driver
from .live_observations import observe_live_identity, _windows_command_line, _digest_args
from .startup_observations import runtime_owner_inventory, observe_departed_identity

KIND = "reviewed-foreign-runtime-fence"
_ISSUED = object()
_FALSE = ("runtimeAcceptance", "cargoAcceptance", "historicalHoldsReleased", "nativeProofIssued",
          "serviceLifecycleExecuted", "loadedModuleBytesProven")
_SHIM = Path(r"D:\cargo-targets\zircon-local\penpot-workbench\01a0f04d-current\validation-r1\unreal-current-product-f-prepared-r403\cargo-shim\run-main-local-cargo.py")
_SHIM_SHA = "718eeafa3802b8f8dae95ba9889642135f88f24ec942cb4ec8a96739853fdf57"
_CARGO_CMD = _SHIM.with_name("cargo-current-main.cmd")
_CARGO_CMD_SHA = "d9e877f22a69cac4a99305157f718ad738c85f441063c5e98b9e9d6096d6ed2a"
_SHIM_TARGET = Path(r"F:\cargo-targets\zircon-local\penpot-workbench\01a0f04d-current\unreal-current-native-product-r392\product\targets")
# The client/build source closure was explicitly reviewed for this activation.
# Jenkins dependencies are additionally pinned to the reviewed sealed target.
_CLIENT_SOURCES = {'tools/__init__.py': None,
 'tools/build/__init__.py': None,
 'tools/build/zircon_build.py': '1fd8e5b10cbac8a870c1ae5f8e8695656cc3e038af89e048aa9af9f042d3d2e2',
 'tools/build/zircon_build_abi.py': '476e63da398b012d9e7218f067248bd72dee3321bfa54520d26eab8ea3017d77',
 'tools/build/zircon_build_arguments.py': 'b75c89b2e08891e6a2ddc4bcae4e3f1fbdfc132751b51ad4e47da31d52914dad',
 'tools/build/zircon_build_artifact_io.py': '7c2d148a97c02aade7124568069dec43ffe339c2c754fe20b5af84095e7a4d1d',
 'tools/build/zircon_build_asset_policy.py': '53fc5d6150c64ac773841fe2ceab1fcf97329e3e9334fa61624e7dfd77925042',
 'tools/build/zircon_build_asset_staging.py': 'd52c6ace74c0604b6644cb0e95021f027131b57a6dc963584e6395bd0d2a49a5',
 'tools/build/zircon_build_cargo_environment.py': '1960d95e5a3c9fd7e76502d74d52c5588e303c2485ee05a7c0079d5bfc447500',
 'tools/build/zircon_build_config.py': '692c73bd2fc1b1743a5dba3976dc5b25fc6f03eef2249328ef3173e5418ed06a',
 'tools/build/zircon_build_editor_font.py': '8a43fcb9ad18b18a0a445a33db500a5190cac2790ae0dcc8917fcfedae2c8244',
 'tools/build/zircon_build_feature_closure.py': 'd7442b667e98c9cb8861b047ed75554e1553e533d639098e690553efe48df519',
 'tools/build/zircon_build_filesystem.py': '4df4d9c167a2c778f87a7b9c96c1691db81c72b97d01175ef581c560320e2251',
 'tools/build/zircon_build_font_sdf.py': '3cc66dbf6949dab94351f376180f5a2383e9850221ea470ae0874d8bb9404f79',
 'tools/build/zircon_build_hub.py': '0ba052e18cfa0a04fdcaaf4dddbc303e3a4124fabbcf0a055ce9d055fac738ca',
 'tools/build/zircon_build_hub_outputs.py': 'e47b5ea758866192a6eebf76f33022be17a22530a582db7af6af847f69cf31c3',
 'tools/build/zircon_build_plugin_assets.py': 'a8cc267a14d62300f5469f896eff42ba8f04d2089fc17480e5389024f8b4f327',
 'tools/build/zircon_build_plugin_manifest_contract.py': '76b5b7033a6efd6285002a31fb59444635a872dd265c3b341c7f420edbc402d3',
 'tools/build/zircon_build_plugin_packages.py': 'cc5f76b6eb407c06f60beba7f00da463c08d0ad2b540933644f1242cff620c3d',
 'tools/build/zircon_build_plugin_selection.py': 'a9fa5dbe9a229a1f90bc9238d23b9d58a620e2f01043a5c3d8b320c79300455a',
 'tools/build/zircon_build_plugin_shader_descriptor_support.py': 'e88d87dea5834cabde8fad600d7526d2a54e0571416e76884311ae69672575b7',
 'tools/build/zircon_build_plugin_shader_descriptors.py': '5af012e2a587ae09d86e81e336fec68915add3d3fd1d8d72c2abccd7386f84db',
 'tools/build/zircon_build_plugin_workspace_crates.py': 'e48f77a07402e1fcce53ec52bd80c4245925e93d4119ec73465b58f25be6b211',
 'tools/build/zircon_build_product_output.py': 'cc3294b855c2f18337c0da08fbdf9f9de666724a288809ee99d97251dd912010',
 'tools/build/zircon_build_product_profile.py': 'd596cb727f64cb7d5fa82ab5ebd11815140000beb6f9202523aaa44278ba1a6e',
 'tools/build/zircon_build_runtime_manifest.py': '53a75ef1db7d4b40ccbc3ecc464d8f9e5f17bc416bf68a5af912a5d47dd4a85a',
 'tools/build/zircon_build_shader_prewarm.py': '7b19a012bf441633f11fa54c6c13b7b8c17d1c01813f0913ec4448a57c27fba4',
 'tools/build/zircon_build_shader_prewarm_acceptance.py': 'd792ac4701231aae5fedc729f06dc57463405a52a9b6a97e4c2cc2561a0ddb49',
 'tools/build/zircon_build_shader_prewarm_cache_artifacts.py': 'df8c74662aecad548f77de1b10591f1e8f3d9be14f3478dbc5fae5ac17384ff7',
 'tools/build/zircon_build_shader_prewarm_report_contract.py': 'd79df19df2183fb72a2d84b1d9af7fa91612d8f12ff6db2ac9fbd92ab75e7832',
 'tools/build/zircon_build_shader_prewarm_written_variants.py': '6067aa6da9f3b5f7a26c92789a8a49edac080240dda906fd61930bdf864a50c2',
 'tools/build/zircon_build_shader_resource_registry.py': 'd795bc585e013e300f5f4fa4d396d3aa627c82e968565edb4e395016a2064402',
 'tools/build/zircon_build_size_report.py': 'e3d2498d851a3a137b702744583a98e28bd20173047d5a54ee30b6eb1f8adf59',
 'tools/build/zircon_build_staging_manifest.py': '097b1bd6735359b40626ef64535badcfcd3968078cb255d17ffa2e7c7d35dbaa',
 'tools/build/zircon_build_support.py': 'e96ce6378a9949ba23f0e0765d726f360566708188162e792d3afba0dedd42f6',
 'tools/build/zircon_build_zui_assets.py': 'd90d862d7a8aa4b586d32c2d34939510e8c0002280ed56463545ed360b644f84',
 'tools/dev/__init__.py': None,
 'tools/dev/local_cargo.py': '5d1f2bb01c3ba8349dcab3342a381fcbdc3dbeba1f26ea811049251fe806637e',
 'tools/export/__init__.py': '18ee8298f07e107d0a0b28af79bda942361ba69aac188d9db9437576b6e996ff',
 'tools/export/file_digest.py': '49e883ff465eb72b60135c649aa981f14ffc68a58da9ede4fa18b3638f342649',
 'tools/export/plugin_validate_distribution_asset_matches.py': '4370afd435653182d9db5a50a6edb6e05d14b6d4aa3188af108556c3aa61be47',
 'tools/export/plugin_validate_distribution_assets.py': 'f91fd45999840c77c39077e7266018f74ab8d042da37cf1e210f571e05ccc737',
 'tools/export/plugin_validate_distribution_zui_assets.py': '9a800763adc5492b875ff63458d6ee5906e8a306c2805c4681092110084624d3'}
_SERVICE_FILES = ("deployment-operation.json", "deployment/host.json", "deployment/driver.json",
    "formal-lifecycle/recovery-keeper.json", "formal-lifecycle/keeper.json", "recovery-lifecycle/command.json")
_SERVICE_ARGS = ("__lifecycle-host", "__execution-host", "jenkins_recovery_keeper.py",
    "jenkins.war", "remoting.jar", "tools.jenkins.deployment", "tools.jenkins.processes",
    "tools.jenkins.tray", "tools.jenkins.launcher", "tools.jenkins.keeper")


def _require(condition, message):
    if not condition:
        raise JenkinsError("runtime_role_unproven", message, retryable=True)


def _raw(path):
    path = Path(path)
    _plain(path)
    return path.read_bytes()


def _sha(raw):
    return hashlib.sha256(raw).hexdigest()


def _identity(value, repo):
    _require(isinstance(value, dict) and set(value) == {"pid", "creationTime", "executable", "commandDigest"}
             and type(value["pid"]) is int and value["pid"] > 0 and isinstance(value["creationTime"], str)
             and value["creationTime"].isdigit() and Path(value["executable"]) ==
             repo / ".jenkins/runtime/python/python.exe", "An exact dedicated Python birth/argv identity is required")
    require_digest(value["commandDigest"])
    return value


def _same(left, right):
    return all(str(left.get(name)) == str(right.get(name)) for name in ("pid", "creationTime", "executable"))


def _source_files(repo, target):
    _require(target == verify_driver(Path(target.get("root", "")), expected_digest=target.get("driverDigest"))
             and Path(target["root"]) == repo / ".jenkins/runtime/drivers" / target["driverDigest"],
             "Client dependencies need their verified canonical sealed target")
    manifest = json.loads(_raw(Path(target["manifest"])))
    sources = {str(repo / name): checksum for name, checksum in _CLIENT_SOURCES.items()}
    for name, entry in manifest["files"].items():
        if name.startswith("tools/jenkins/") and name.endswith(".py"):
            sources[str(repo / name)] = entry["sha256"]
    _require(str(repo / "tools/jenkins/frontend.py") in sources
             and str(repo / "tools/jenkins/submission.py") in sources,
             "The reviewed client submission closure is incomplete")
    for name, checksum in sources.items():
        path = Path(name)
        _plain(path)
        _require((not path.exists()) if checksum is None else _sha(_raw(path)) == checksum,
                 "The reviewed client source closure or namespace anchor changed")
    return sources


def _directory(value, *, leaf=False):
    path = Path(value)
    root = build_root_containing(path)
    physical = physical_path_under(root, path, allow_missing=leaf)
    _require(path.is_absolute() and ".." not in path.parts and physical == path
             and path != root.path and path.is_relative_to(root.path / "zircon-local"),
             "A reviewed client output escaped the physical external build namespace")
    directory = path.parent if leaf else path
    physical_path_under(root, directory, allow_missing=False)
    _require(directory.is_dir(), "The actual external output directory does not exist")
    stat = directory.stat()
    return {"path": str(path), "directory": str(directory), "device": stat.st_dev,
            "fileId": stat.st_ino, "volumeIdentity": root.volume_identity}


def _option(arguments, name):
    _require(arguments.count(name) == 1, "An exact single client output option is required")
    index = arguments.index(name)
    _require(index + 1 < len(arguments) and not arguments[index + 1].startswith("--"),
             "A client output value is missing")
    return arguments[index + 1]


def _role(repo, identity, arguments, sources):
    _require(arguments[:4] == [identity["executable"], "-B", "-X", "utf8"],
             "Only the reviewed literal Python invocation may be a foreign client")
    _require(not any(marker in "\0".join(arguments).casefold() for marker in _SERVICE_ARGS),
             "A lifecycle/service command cannot be classified as a foreign client")
    body = arguments[4:]
    if body and body[0] == str(repo / "tools/build/zircon_build.py"):
        options = body[1:]
        allowed = {"--targets", "--mode", "--runtime-features", "--out", "--jobs", "--cargo"}
        _require(len(options) % 2 == 0 and len(set(options[::2])) == len(options[::2])
                 and set(options[::2]) == allowed and _option(options, "--targets") == "editor"
                 and _option(options, "--mode") == "debug" and _option(options, "--jobs").isdigit()
                 and _option(options, "--jobs") == "1"
                 and _option(options, "--runtime-features") == "target-editor-host,profiling"
                 and _option(options, "--cargo") == str(_CARGO_CMD),
                 "This builder invocation is outside the reviewed product build route")
        _require(_sha(_raw(_CARGO_CMD)) == _CARGO_CMD_SHA and _sha(_raw(_SHIM)) == _SHIM_SHA,
                 "The exact reviewed Cargo shim changed")
        sources = {**sources, str(_CARGO_CMD): _CARGO_CMD_SHA, str(_SHIM): _SHIM_SHA}
        return "zircon-product-builder", sources, [_directory(_option(options, "--out"))]
    if body and body[0] == str(_SHIM):
        options = body[1:]
        _require(options[:1] == ["rustc"] and "--" not in options and "--config" not in options
                 and not any(value.startswith(("--config=", "--out-dir", "--artifact-dir")) for value in options)
                 and _sha(_raw(_SHIM)) == _SHIM_SHA and _sha(_raw(_CARGO_CMD)) == _CARGO_CMD_SHA,
                 "Only the reviewed Cargo shim's bounded rustc route may be a foreign client")
        target = Path(_option(options, "--target-dir"))
        _require(options == ["rustc", "-p", "zircon_runtime", "--lib", "--crate-type", "cdylib",
            "--no-default-features", "--features", "target-editor-host profiling",
            "--target-dir", str(target), "--locked", "--jobs", "1"],
            "The reviewed Cargo client flags changed")
        _require(target.is_relative_to(_SHIM_TARGET) and target != _SHIM_TARGET,
                 "This shim targets another product output")
        sources = {**sources, str(_SHIM): _SHIM_SHA, str(_CARGO_CMD): _CARGO_CMD_SHA}
        return "zircon-local-cargo-shim", sources, [_directory(target)]
    raise JenkinsError("runtime_role_unproven", "This Python command has no explicit foreign client review", retryable=True)


def _service_bindings(manager):
    identities, references = [], {}
    def collect(value):
        if isinstance(value, dict):
            if type(value.get("pid")) is int and str(value.get("creationTime", "")).isdigit():
                identities.append(value)
            elif type(value.get("hostPid")) is int and str(value.get("creationTime", "")).isdigit():
                identities.append({**value, "pid": value["hostPid"]})
            for child in value.values():
                collect(child)
        elif isinstance(value, list):
            for child in value:
                collect(child)
    for name in _SERVICE_FILES:
        path = manager.paths.state / name
        _plain(path)
        if path.exists():
            raw = _raw(path)
            references[str(path)] = _sha(raw)
            collect(json.loads(raw))
        else:
            references[str(path)] = None
    return identities, references


def inspect_runtime_role_review(manager, *, clients, reviewed_target):
    """Read-only live capture. Archive the returned document immutably."""
    repo = manager.paths.repo.absolute()
    boot = str(windows_boot_filetime())
    _require(isinstance(clients, list) and clients and len(clients) <= 8
             and len({value.get("pid") for value in clients}) == len(clients),
             "Name the exact live client identities being reviewed")
    sources = _source_files(repo, reviewed_target)
    service, bindings = _service_bindings(manager)
    rows = []
    inventory = runtime_owner_inventory(repo)
    for value in clients:
        identity = _identity(value, repo)
        _require(int(identity["creationTime"]) >= int(boot)
                 and any(_same(item, identity) for item in inventory)
                 and not any(_same(item, identity) for item in service),
                 "Persisted native ownership takes precedence over client classification")
        observe_live_identity(identity)
        arguments = list(_windows_command_line(identity["pid"]))
        _require(_digest_args(tuple(arguments)) == identity["commandDigest"], "The reviewed argv digest changed")
        role, closure, outputs = _role(repo, identity, arguments, sources)
        observe_live_identity(identity)
        rows.append({"identity": identity, "arguments": arguments, "role": role,
                     "sourceFiles": closure, "physicalOutputs": outputs})
    _require(str(windows_boot_filetime()) == boot, "Runtime role review crossed a Windows boot")
    return {"schemaVersion": 1, "kind": KIND, "repository": str(repo), "bootFileTime": boot,
            "sourceCatalogDigest": digest(_CLIENT_SOURCES), "reviewedTarget": reviewed_target,
            "clients": rows, "originalRuntimeInventory": inventory, "serviceBindingsAtReview": bindings,
            **{name: False for name in _FALSE}}


@dataclass(frozen=True, init=False)
class ReviewedRuntimeRoles:
    snapshot: bytes
    reference: bytes
    _token: object

    def __init__(self, snapshot, reference, token=None):
        _require(token is _ISSUED and type(snapshot) is bytes and type(reference) is bytes,
                 "Only the verified immutable role review factory may issue a capability")
        object.__setattr__(self, "snapshot", snapshot)
        object.__setattr__(self, "reference", reference)
        object.__setattr__(self, "_token", token)

    @property
    def document(self):
        return json.loads(self.snapshot)

    @property
    def review_reference(self):
        return json.loads(self.reference)

    def observe(self, manager):
        _require(type(self) is ReviewedRuntimeRoles and self._token is _ISSUED,
                 "A caller dictionary, boolean or subclass cannot classify runtime owners")
        value = self.document
        reference = self.review_reference
        _require(_sha(_raw(Path(reference["path"]))) == reference["sha256"]
                 and canonical_json(json.loads(_raw(Path(reference["path"])))) == self.snapshot,
                 "The immutable runtime review changed")
        repo = manager.paths.repo.absolute()
        _require(value.get("schemaVersion") == 1 and value.get("kind") == KIND
                 and value.get("repository") == str(repo) and value.get("bootFileTime") == str(windows_boot_filetime())
                 and value.get("sourceCatalogDigest") == digest(_CLIENT_SOURCES)
                 and all(value.get(name) is False for name in _FALSE), "The reviewed runtime boundary is not exact")
        sources = _source_files(repo, value["reviewedTarget"])
        clients = value.get("clients")
        _require(isinstance(clients, list) and 0 < len(clients) <= 8
                 and len({item["identity"]["pid"] for item in clients}) == len(clients),
                 "The immutable client identity set is invalid")
        for item in clients:
            identity = _identity(item["identity"], repo)
            _require(type(item.get("arguments")) is list and _digest_args(tuple(item["arguments"])) == identity["commandDigest"],
                     "The original exact client argv changed")
            role, closure, outputs = _role(repo, identity, item["arguments"], sources)
            _require(role == item.get("role") and closure == item.get("sourceFiles")
                     and outputs == item.get("physicalOutputs"), "Client source or physical output identity changed")
        inventory = runtime_owner_inventory(repo)
        service, bindings = _service_bindings(manager)
        blocking, foreign, matched = [], [], set()
        for actual in inventory:
            candidate = next((item for item in clients if _same(actual, item["identity"])), None)
            if candidate is None or Path(actual.get("executable", "")).name.casefold() in {"java.exe", "javaw.exe"} \
                    or any(_same(actual, identity) for identity in service):
                blocking.append(actual)
                continue
            observe_live_identity(candidate["identity"])
            arguments = list(_windows_command_line(actual["pid"]))
            if arguments != candidate["arguments"] or any(marker in "\0".join(arguments).casefold() for marker in _SERVICE_ARGS):
                blocking.append(actual)
                continue
            observe_live_identity(candidate["identity"])
            matched.add(actual["pid"])
            foreign.append({**candidate, "status": "live"})
        for item in clients:
            if item["identity"]["pid"] not in matched:
                if any(_same(actual, item["identity"]) for actual in blocking):
                    continue
                departure = observe_departed_identity(item["identity"])
                foreign.append({**item, "status": "departed", "departure": departure})
        _require(str(windows_boot_filetime()) == value["bootFileTime"], "Runtime observation crossed a Windows boot")
        return {"schemaVersion": 1, "kind": "reviewed-runtime-owner-observation", "review": reference,
                "runtimeInventory": inventory, "blockingOwners": blocking, "reviewedForeign": foreign,
                "serviceBindings": bindings, **{name: False for name in _FALSE}}


def prepare_runtime_role_review(manager, *, review_reference):
    state = manager.paths.state.absolute()
    _require(isinstance(review_reference, dict) and set(review_reference) == {"path", "sha256"},
             "An exact immutable runtime review reference is required")
    path = Path(review_reference["path"]).absolute()
    _require(str(path) == review_reference["path"] and path.parent == state / "recovery-lifecycle/history"
             and path.suffix == ".json" and ".." not in path.parts
             and _sha(_raw(path)) == require_digest(review_reference["sha256"]),
             "Runtime role review escaped its immutable namespace or changed")
    capability = ReviewedRuntimeRoles(canonical_json(json.loads(_raw(path))), canonical_json(review_reference), _ISSUED)
    capability.observe(manager)
    return capability


__all__ = ["ReviewedRuntimeRoles", "inspect_runtime_role_review", "prepare_runtime_role_review"]
