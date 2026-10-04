"""Copy verified preserved binaries into a fresh independent Jenkins root."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import re
import zipfile
from contextlib import ExitStack, contextmanager

from .contracts import PilotError, canonical_json, sha256_identity
from .governance import require_live_storage_owner
from .jenkins_config import (
    PilotPaths, REQUIRED_PLUGIN_IDS, DEFAULT_AGENT_NAME, DEFAULT_AGENT_LABEL,
    DEFAULT_HTTP_PORT, ensure_credentials, restrict_private_root,
)
from .native.paths import WorkerStorage
from .native.portable_paths import normalize_portable_relative_path
from .storage import ManagedStorage, physical_identity, require_managed_root

PRESERVED_VERSION = "2.580.1"
PRESERVED_WAR_SHA256 = "393bf2476352dd726519fd1f92ce66eac7d23c0936e6967420b717060c4c40d0"
PRESERVATION_PROOF = "evidence/native-v2-preservation-001-proof.json"
ASSET_PROOF = "asset-source-manifest.json"
MAX_ASSET_FILES = 10000


def _plugins(manifest: dict) -> list[dict]:
    plugins = manifest.get("plugins")
    if manifest.get("jenkinsVersion") != PRESERVED_VERSION or not isinstance(plugins, list) or len(plugins) != 29:
        raise PilotError("preserved assets require Jenkins 2.580.1 and its fixed 29 plugin records")
    result, seen = [], set()
    for plugin in plugins:
        if not isinstance(plugin, dict):
            raise PilotError("preserved plugin metadata is invalid")
        name, version = plugin.get("id"), plugin.get("version")
        if not isinstance(name, str) or not re.fullmatch(r"[a-z0-9][a-z0-9-]{0,127}", name):
            raise PilotError("preserved plugin identifier is invalid")
        if name in seen or not isinstance(version, str) or not re.fullmatch(r"[0-9A-Za-z_.-]{1,128}", version):
            raise PilotError("preserved plugin version or identity is invalid")
        seen.add(name)
        result.append({"id": name, "version": version,
                       "sha256": sha256_identity(plugin.get("sha256"), "plugin sha256")})
    if not set(REQUIRED_PLUGIN_IDS).issubset(seen):
        raise PilotError("preserved plugin set lacks required Pipeline steps")
    return sorted(result, key=lambda item: item["id"])


def _historical_records(raw: bytes | None, origin: Path) -> dict[str, dict]:
    if raw is None:
        return {}
    try:
        proof = json.loads(raw)
        if (proof.get("allCopyHashesEqual") is not True
                or Path(proof["destinationRoot"]) != origin
                or not isinstance(proof["files"], list)
                or len(proof["files"]) > 100000):
            raise PilotError("historical preservation proof does not bind this asset origin")
        records = {}
        for item in proof["files"]:
            name = normalize_portable_relative_path(item["path"], code="asset_path", message="invalid proof path")
            if name in records or type(item["bytes"]) is not int or item["bytes"] < 0:
                raise PilotError("historical preservation proof has duplicate or invalid records")
            records[name] = {"sha256": sha256_identity(item["sha256"], "historical sha256"), "size": item["bytes"]}
        return records
    except (ValueError, KeyError, TypeError, AttributeError) as error:
        raise PilotError("historical preservation proof is invalid") from error


def _fresh_destination(destination: WorkerStorage) -> None:
    for name in ("pilot-manifest.json", ASSET_PROOF, "credentials.json", "process-state.json"):
        if destination.exists(name):
            raise PilotError("asset preparation requires a fresh independently owned destination")
    for directory in ("jdk", "war", "jenkins_home"):
        # Existing empty preparation folders are allowed; no old home is adopted.
        try:
            files = destination.list_files(directory)
        except FileNotFoundError:
            continue
        if files:
            raise PilotError("asset destination already contains runtime or controller state")


def prepare_preserved_assets(root: Path, origin: Path, *, repo_root: Path) -> dict:
    """Reuse pinned binary bytes without carrying any historic controller state.

    Only WAR, the JDK tree and explicitly listed plugin JPI files are admitted.
    A fresh manifest becomes prepared only after every copied file is verified.
    Failed preparation leaves its evidence for inspection and never adopts it.
    """
    paths = PilotPaths(root)
    origin = require_managed_root(origin, allow_root=False)
    if paths.root == origin or paths.root.is_relative_to(origin) or origin.is_relative_to(paths.root):
        raise PilotError("asset source and destination must be distinct disjoint roots")
    require_live_storage_owner(paths.root, repo_root)
    with WorkerStorage(origin, create=False) as source, ManagedStorage(paths.root).backend() as destination:
        _fresh_destination(destination)
        manifest_raw = source.read_bytes("pilot-manifest.json", max_bytes=1024 * 1024)
        try:
            old_manifest = json.loads(manifest_raw)
            plugins = _plugins(old_manifest)
        except (ValueError, AttributeError) as error:
            raise PilotError("preserved asset manifest is invalid") from error
        historical_raw = (source.read_bytes(PRESERVATION_PROOF, max_bytes=32 * 1024 * 1024)
                          if source.exists(PRESERVATION_PROOF) else None)
        historical = _historical_records(historical_raw, origin)
        manifest_hash = hashlib.sha256(manifest_raw).hexdigest()
        if historical and historical.get("pilot-manifest.json") != {"sha256": manifest_hash, "size": len(manifest_raw)}:
            raise PilotError("preserved source manifest differs from its historical copy proof")
        jdk_files, jdk_directories = source.list_files("jdk"), source.list_directories("jdk")
        if "jdk/bin/java.exe" not in jdk_files or len(jdk_files) > MAX_ASSET_FILES:
            raise PilotError("preserved JDK inventory is missing Java or exceeds its bound")
        plugin_names = {f"jenkins_home/plugins/{plugin['id']}.jpi": plugin for plugin in plugins}
        names = sorted(["war/jenkins.war", *jdk_files, *plugin_names])
        records, identities = [], {}
        for name in names:
            sha256, identity = source.sha256(name)
            expected = PRESERVED_WAR_SHA256 if name == "war/jenkins.war" else plugin_names[name]["sha256"] if name in plugin_names else sha256
            if sha256 != expected:
                raise PilotError("preserved binary differs from its pinned asset checksum")
            record = {"path": name, "sha256": sha256, "size": identity.size}
            if historical and historical.get(name) != {"sha256": sha256, "size": identity.size}:
                raise PilotError("preserved binary differs from its historical copy proof")
            records.append(record)
            identities[name] = identity
        require_live_storage_owner(paths.root, repo_root)
        restrict_private_root(paths.root)
        for directory in paths.directories():
            if directory != paths.root:
                destination.ensure_directory(directory.relative_to(paths.root).as_posix())
        for directory in reversed(jdk_directories):
            destination.ensure_directory(directory)
        for record in records:
            name, expected = record["path"], identities[record["path"]]
            if not source.file_identity(name).same_file_state(expected):
                raise PilotError("preserved asset changed before copying")
            destination.atomic_write_stream(name, source.iter_file(name),
                                            expected_sha256=record["sha256"], expected_size=record["size"])
            source_hash, source_identity = source.sha256(name)
            copied_hash, copied_identity = destination.sha256(name)
            if (source_hash != record["sha256"] or not source_identity.same_file_state(expected)
                    or copied_hash != source_hash or copied_identity.size != record["size"]):
                raise PilotError("preserved asset changed or destination copy verification failed")
        if source.list_files("jdk") != jdk_files or source.list_directories("jdk") != jdk_directories:
            raise PilotError("preserved JDK inventory changed during copying")
        if source.read_bytes("pilot-manifest.json", max_bytes=1024 * 1024) != manifest_raw:
            raise PilotError("preserved source manifest changed during copying")
        if historical_raw is not None and source.read_bytes(PRESERVATION_PROOF, max_bytes=32 * 1024 * 1024) != historical_raw:
            raise PilotError("historical asset proof changed during copying")
        # Recheck all bytes on both sides, including assets copied early in the
        # loop; size/mtime equality alone does not attest a final inventory.
        for record in records:
            final_hash, final_identity = source.sha256(record["path"])
            target_hash, target_identity = destination.sha256(record["path"])
            if (not final_identity.same_file_state(identities[record["path"]])
                    or final_hash != record["sha256"] or target_hash != final_hash
                    or target_identity.size != record["size"]):
                raise PilotError("asset bytes changed before preparation publication")
        from .bootstrap import _java_major, _write_security_bootstrap
        # Retain JDK, home and bootstrap parent pins while credential/bootstrap
        # helpers create new local files; no source credentials are ever read.
        destination.ensure_directory("jenkins_home/init.groovy.d")
        with _pinned_asset_files(destination, names):
            if (_java_major(paths.java_root / "bin/java.exe") or 0) < 21:
                raise PilotError("preserved portable JDK must provide Java 21 or newer")
            ensure_credentials(paths)
            _write_security_bootstrap(paths)
        require_live_storage_owner(paths.root, repo_root)
        proof = {"schemaVersion": 1, "sourceRoot": str(origin), "destinationRoot": str(paths.root),
                 "sourceRootIdentity": physical_identity(origin), "destinationRootIdentity": physical_identity(paths.root),
                 "sourceManifestSha256": manifest_hash, "sourceManifestPath": "pilot-manifest.json",
                 "historicalProofPath": PRESERVATION_PROOF if historical_raw is not None else None,
                 "historicalProofSha256": hashlib.sha256(historical_raw).hexdigest() if historical_raw is not None else None,
                 "jenkinsVersion": PRESERVED_VERSION, "plugins": plugins, "files": records,
                 "jdkDirectories": jdk_directories,
                 "fileCount": len(records), "copiedBytes": sum(record["size"] for record in records)}
        proof_bytes = canonical_json(proof)
        destination.write_bytes(ASSET_PROOF, proof_bytes)
        manifest = {"schemaVersion": 1, "jenkinsVersion": PRESERVED_VERSION,
                    "controllerHost": "127.0.0.1", "controllerPort": DEFAULT_HTTP_PORT,
                    "controllerExecutors": 0, "agentExecutors": 1,
                    "agentName": DEFAULT_AGENT_NAME, "agentLabel": DEFAULT_AGENT_LABEL,
                    "warPath": str(paths.war_path), "javaPath": str(paths.java_root / "bin/java.exe"),
                    "plugins": [{**plugin, "path": str(paths.plugins / (plugin["id"] + ".jpi"))} for plugin in plugins],
                    "repoRoot": str(repo_root), "assetSourceManifest": str(paths.root / ASSET_PROOF),
                    "assetSourceManifestSha256": hashlib.sha256(proof_bytes).hexdigest(), "prepared": True}
        destination.write_bytes("pilot-manifest.json", canonical_json(manifest))
        return manifest


def verify_prepared_assets(root: Path) -> str:
    """Verify the independently copied byte inventory before a runtime launch."""
    paths = PilotPaths(root)
    with WorkerStorage(paths.root, create=False) as backend:
        try:
            manifest = json.loads(backend.read_bytes("pilot-manifest.json", max_bytes=1024 * 1024))
            expected_digest = sha256_identity(manifest.get("assetSourceManifestSha256"), "asset proof sha256")
            if (manifest.get("prepared") is not True
                    or manifest.get("assetSourceManifest") != str(paths.root / ASSET_PROOF)
                    or manifest.get("warPath") != str(paths.war_path)
                    or manifest.get("javaPath") != str(paths.java_root / "bin/java.exe")
                    or manifest.get("controllerHost") != "127.0.0.1"
                    or manifest.get("controllerExecutors") != 0 or manifest.get("agentExecutors") != 1):
                raise PilotError("prepared asset manifest does not bind this independent runtime root")
            plugins = _plugins(manifest)
            for plugin in manifest["plugins"]:
                if plugin.get("path") != str(paths.plugins / (plugin["id"] + ".jpi")):
                    raise PilotError("prepared plugin path does not bind this independent runtime root")
            proof_raw = backend.read_bytes(ASSET_PROOF, expected_sha256=expected_digest, max_bytes=32 * 1024 * 1024)
            proof = json.loads(proof_raw)
            origin = require_managed_root(proof["sourceRoot"], allow_root=False)
            if (proof.get("schemaVersion") != 1 or proof.get("destinationRoot") != str(paths.root)
                    or origin == paths.root or origin.is_relative_to(paths.root) or paths.root.is_relative_to(origin)
                    or proof.get("destinationRootIdentity") != physical_identity(paths.root)
                    or proof.get("jenkinsVersion") != PRESERVED_VERSION or proof.get("plugins") != plugins):
                raise PilotError("prepared asset proof root or version binding changed")
            sha256_identity(proof["sourceManifestSha256"], "source manifest sha256")
            if proof.get("sourceManifestPath") != "pilot-manifest.json":
                raise PilotError("asset source manifest provenance path is invalid")
            if proof.get("historicalProofSha256") is not None:
                sha256_identity(proof["historicalProofSha256"], "historical proof sha256")
                if proof.get("historicalProofPath") != PRESERVATION_PROOF:
                    raise PilotError("asset preservation provenance path is invalid")
            elif proof.get("historicalProofPath") is not None:
                raise PilotError("asset preservation provenance lacks its checksum")
            records = proof["files"]
            if not isinstance(records, list) or not records or len(records) > MAX_ASSET_FILES + 30:
                raise PilotError("prepared asset proof inventory is invalid")
            expected_plugins = {f"jenkins_home/plugins/{plugin['id']}.jpi": plugin["sha256"] for plugin in plugins}
            admitted, jdk_files, total_bytes = set(), [], 0
            for record in records:
                name = normalize_portable_relative_path(record["path"], code="asset_path", message="invalid prepared path")
                sha256 = sha256_identity(record["sha256"], "prepared file sha256")
                if (name != record["path"] or name in admitted or type(record["size"]) is not int
                        or record["size"] < 0):
                    raise PilotError("prepared asset proof contains duplicate or invalid file records")
                admitted.add(name)
                total_bytes += record["size"]
                if name == "war/jenkins.war":
                    if sha256 != PRESERVED_WAR_SHA256:
                        raise PilotError("prepared WAR differs from the fixed Jenkins checksum")
                elif name in expected_plugins:
                    if sha256 != expected_plugins[name]:
                        raise PilotError("prepared plugin differs from the source manifest checksum")
                elif name.startswith("jdk/"):
                    jdk_files.append(name)
                else:
                    raise PilotError("prepared proof contains an unadmitted runtime or controller-state file")
                actual_hash, identity = backend.sha256(name)
                if actual_hash != sha256 or identity.size != record["size"]:
                    raise PilotError("prepared asset bytes differ from their exact copy proof")
            if ("war/jenkins.war" not in admitted or "jdk/bin/java.exe" not in admitted
                    or not set(expected_plugins).issubset(admitted)
                    or proof.get("fileCount") != len(records) or proof.get("copiedBytes") != total_bytes
                    or backend.list_files("jdk") != sorted(jdk_files)
                    or backend.list_directories("jdk") != proof.get("jdkDirectories")):
                raise PilotError("prepared asset inventory differs from its exact copy proof")
            _verify_plugin_code(backend, plugins)
            if backend.read_bytes(ASSET_PROOF, expected_sha256=expected_digest, max_bytes=32 * 1024 * 1024) != proof_raw:
                raise PilotError("prepared asset proof changed during launch verification")
            return expected_digest
        except (ValueError, KeyError, TypeError, AttributeError, zipfile.BadZipFile) as error:
            raise PilotError("prepared asset proof is invalid or bytes changed") from error


def _verify_plugin_code(backend: WorkerStorage, plugins: list[dict]) -> None:
    """Admit only exact JPI archives and their verified expanded ZIP bytes."""
    base = "jenkins_home/plugins"
    actual = backend.list_files(base)
    directories = backend.list_directories(base)
    known = {plugin["id"] for plugin in plugins}
    expected_archives = {f"{base}/{name}.jpi" for name in known}
    if {name for name in actual if name.count("/") == 2} != expected_archives:
        raise PilotError("plugin root contains unlisted files or HPI executable archives")
    if any(name.split("/")[2] not in known for name in directories):
        raise PilotError("plugin root contains an unlisted expanded executable directory")
    for plugin in plugins:
        prefix = f"{base}/{plugin['id']}"
        expanded = {name[len(prefix)+1:] for name in actual if name.startswith(prefix + "/")}
        if prefix not in directories:
            continue
        with backend._windows.open_relative(prefix + ".jpi", "rb") as stream, zipfile.ZipFile(stream) as archive:
            entries, seen = {}, set()
            expected_directories = {prefix}
            for info in archive.infolist():
                raw = info.filename.rstrip("/") if info.is_dir() else info.filename
                name = normalize_portable_relative_path(raw, code="plugin_zip", message="unsafe expanded plugin entry")
                if name != raw or name.casefold() in seen:
                    raise PilotError("plugin ZIP contains unsafe or colliding path names")
                seen.add(name.casefold())
                if info.is_dir():
                    expected_directories.add(prefix + "/" + name)
                    continue
                entries[name] = info
                parts = name.split("/")[:-1]
                expected_directories.update(prefix + "/" + "/".join(parts[:index]) for index in range(1, len(parts)+1))
            # Jenkins' expanded-plugin timestamp marker has no executable data.
            marker = ".timestamp2"
            if expanded - {marker} != set(entries):
                raise PilotError("expanded plugin files differ from their immutable JPI ZIP inventory")
            observed_directories = {name for name in directories if name == prefix or name.startswith(prefix + "/")}
            if observed_directories != expected_directories:
                raise PilotError("expanded plugin directories differ from their immutable JPI ZIP inventory")
            if marker in expanded and backend.read_bytes(prefix + "/" + marker, max_bytes=0) != b"":
                raise PilotError("generated plugin timestamp marker must be empty")
            for name, info in entries.items():
                digest = hashlib.sha256()
                with archive.open(info) as payload:
                    for chunk in iter(lambda: payload.read(1024 * 1024), b""):
                        digest.update(chunk)
                actual_hash, identity = backend.sha256(prefix + "/" + name)
                if actual_hash != digest.hexdigest() or identity.size != info.file_size:
                    raise PilotError("expanded plugin code differs from its immutable JPI ZIP input")


@contextmanager
def _pinned_asset_files(backend: WorkerStorage, files: list[str]):
    """Pin existing runtime bytes and identities inside the private namespace."""
    with ExitStack() as pins:
        directories = {"jdk", "jenkins_home/plugins"}
        for name in files:
            parts = name.split("/")[:-1]
            if parts and parts[0] == "war":
                # The checked WAR leaf is immutable, while the same folder
                # receives the exact remoting JAR extracted for agent launch.
                continue
            # Home is mutable controller state; its plugins subtree owns code.
            first = 2 if parts[:2] == ["jenkins_home", "plugins"] else 1
            directories.update("/".join(parts[:index]) for index in range(first, len(parts)+1))
        for name in sorted(directories, key=lambda value: (value.count("/"), value)):
            pins.enter_context(backend._windows.pin_readonly_directory(name))
        for name in files:
            pins.enter_context(backend._windows.open_relative(name, "rb"))
        yield


@contextmanager
def prepared_assets_context(root: Path, *, expected_digest: str | None = None,
                            retain_manifest: bool = True):
    """Verify and retain the complete admitted executable inventory through launch."""
    if expected_digest is not None:
        sha256_identity(expected_digest, "expected asset proof sha256")
    with WorkerStorage(PilotPaths(root).root, create=False) as backend:
        files = ["asset-source-manifest.json", *(["pilot-manifest.json"] if retain_manifest else []), *backend.list_files("jdk"),
                 "war/jenkins.war", *backend.list_files("jenkins_home/plugins")]
        # The proof file lives directly at the mutable root and does not turn
        # unrelated credential/controller state into a read-only directory.
        with _pinned_asset_files(backend, files):
            actual_digest = verify_prepared_assets(root)
            if expected_digest is not None and actual_digest != expected_digest:
                raise PilotError("runtime asset proof differs from its immutable launch binding")
            yield actual_digest
