"""Configuration and private paths for the portable Jenkins pilot.

The pilot is deliberately self contained.  Every downloaded artifact, Jenkins
home file, process log, cache, workspace, and credential file lives below one
approved, registered fixture below a drive-root ``D/E/F:\\cargo-targets``. This module has no
network or process side effects; :mod:`bootstrap` owns those operations.
"""

from __future__ import annotations

import json
import os
import secrets
import stat
import subprocess
import csv
import io
from dataclasses import dataclass
from pathlib import Path, PureWindowsPath
from typing import Mapping

from .storage import DEFAULT_PILOT_ROOT, require_managed_root, ManagedStorage


LATEST_STABLE_CORE_URL = "https://updates.jenkins.io/stable/latestCore.txt"
STABLE_UPDATE_CENTER_URL = "https://updates.jenkins.io/stable/update-center.actual.json"
WAR_DOWNLOAD_TEMPLATE = "https://updates.jenkins.io/download/war/{version}/jenkins.war"
WAR_CHECKSUM_TEMPLATE = "https://get.jenkins.io/war-stable/{version}/jenkins.war.sha256"
PLUGIN_DOWNLOAD_TEMPLATE = (
    "https://updates.jenkins.io/download/plugins/{plugin}/{version}/{plugin}.hpi"
)
PLUGIN_CHECKSUM_TEMPLATE = (
    "https://updates.jenkins.io/download/plugins/{plugin}/{version}/{plugin}.hpi.sha256"
)
TEMURIN_API_URL = (
    "https://api.adoptium.net/v3/assets/latest/21/hotspot"
    "?vendor=eclipse&os=windows&architecture=x64&image_type=jdk"
    "&project=jdk&jvm_impl=hotspot"
)

# Keep this fallback in source so an offline preparation can still report an
# explicit version.  prepare() refreshes it from latestCore.txt by default.
DEFAULT_JENKINS_VERSION = "2.580.1"
DEFAULT_AGENT_NAME = "zircon-windows-agent"
DEFAULT_AGENT_LABEL = "zircon-windows"
DEFAULT_CONTROLLER_HOST = "127.0.0.1"
DEFAULT_CONTROLLER_EXECUTORS = 0
DEFAULT_AGENT_EXECUTORS = 1
DEFAULT_HTTP_PORT = 0
DEFAULT_USERNAME = "admin"

REQUIRED_PLUGIN_IDS = (
    # Keep the fixed Scripted Pipeline steps and their pinned dependencies.
    "workflow-job",
    "workflow-cps",
    "workflow-basic-steps",
    "workflow-durable-task-step",
    "pipeline-stage-step",
    "file-parameters",
    "lockable-resources",
)

_ALLOWED_DRIVES = frozenset({"D:", "E:", "F:"})
_IDENTIFIER_CHARS = frozenset("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789._-")


class JenkinsConfigError(ValueError):
    """Invalid or unsafe pilot configuration."""


def validate_identifier(value: object, field: str = "identifier") -> str:
    if not isinstance(value, str) or not value or len(value) > 128:
        raise JenkinsConfigError(f"{field} must be a bounded portable identifier")
    if value[0] in ".-" or any(character not in _IDENTIFIER_CHARS for character in value):
        raise JenkinsConfigError(f"{field} must be a bounded portable identifier")
    return value


def validate_pilot_root(value: str | os.PathLike[str]) -> Path:
    """Validate the lexical Windows storage contract before touching a path."""

    raw = os.fspath(value)
    parsed = PureWindowsPath(raw)
    if not parsed.is_absolute() or parsed.drive.upper() not in _ALLOWED_DRIVES:
        raise JenkinsConfigError(
            "Jenkins pilot root must be an absolute D:, E:, or F: path"
        )
    parts = tuple(part for part in parsed.parts[1:] if part not in {"\\", "/"})
    if len(parts) < 2 or parts[0].casefold() != "cargo-targets":
        raise JenkinsConfigError(
            "Jenkins pilot root must be below a drive-root cargo-targets directory"
        )
    # Require a pilot-specific leaf so a typo cannot make bootstrap own the
    # shared cargo-targets directory itself.
    validate_identifier(parts[-1], "pilot root name")
    if not parts[-1].casefold().startswith("jenkins-pilot-"):
        raise JenkinsConfigError("Jenkins pilot root name must start with jenkins-pilot-")
    try:
        require_managed_root(raw)
    except (ValueError, OSError) as error:
        raise JenkinsConfigError(str(error)) from error
    return Path(raw)


@dataclass(frozen=True, slots=True)
class PilotPaths:
    """All files owned by one isolated pilot root."""

    root: Path

    def __init__(self, root: str | os.PathLike[str] = DEFAULT_PILOT_ROOT) -> None:
        object.__setattr__(self, "root", validate_pilot_root(root))

    @property
    def downloads(self) -> Path:
        return self.root / "downloads"

    @property
    def jenkins_home(self) -> Path:
        return self.root / "jenkins_home"

    @property
    def plugins(self) -> Path:
        return self.jenkins_home / "plugins"

    @property
    def controller_log(self) -> Path:
        return self.root / "logs" / "controller.log"

    @property
    def agent_log(self) -> Path:
        return self.root / "logs" / "agent.log"

    @property
    def java_root(self) -> Path:
        return self.root / "jdk"

    @property
    def war_path(self) -> Path:
        return self.root / "war" / "jenkins.war"

    @property
    def agent_jar(self) -> Path:
        return self.root / "war" / "agent.jar"

    @property
    def work_root(self) -> Path:
        return self.root / "work"

    @property
    def cache_root(self) -> Path:
        return self.root / "caches"

    @property
    def manifest_path(self) -> Path:
        return self.root / "pilot-manifest.json"

    @property
    def process_state_path(self) -> Path:
        return self.root / "process-state.json"

    @property
    def credentials_path(self) -> Path:
        return self.root / "credentials.json"

    @property
    def node_work_root(self) -> Path:
        return self.work_root / "agent"

    @property
    def source_root(self) -> Path:
        return self.work_root / "source"

    def directories(self) -> tuple[Path, ...]:
        return (
            self.root,
            self.downloads,
            self.jenkins_home,
            self.plugins,
            self.root / "logs",
            self.java_root,
            self.war_path.parent,
            self.work_root,
            self.cache_root,
            self.node_work_root,
            self.source_root,
        )


@dataclass(frozen=True, slots=True)
class PluginRecord:
    plugin_id: str
    version: str
    sha256: str
    path: Path
    required_by: tuple[str, ...] = ()

    def to_dict(self) -> dict[str, object]:
        return {
            "id": self.plugin_id,
            "version": self.version,
            "sha256": self.sha256,
            "path": str(self.path),
            "requiredBy": list(self.required_by),
        }


def _private_file(path: Path, payload: bytes) -> None:
    """Write private JSON and apply the strongest portable mode available."""

    require_managed_root(path, allow_root=False)
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(f".{path.name}.{secrets.token_hex(8)}.tmp")
    try:
        with temporary.open("wb") as stream:
            stream.write(payload)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary, path)
        try:
            path.chmod(stat.S_IRUSR | stat.S_IWUSR)
        except OSError:
            # Windows ACLs are inherited from the private pilot root.  Do not
            # weaken the operation or print the credential when chmod is absent.
            pass
    finally:
        try:
            temporary.unlink()
        except FileNotFoundError:
            pass


def ensure_credentials(paths: PilotPaths) -> Mapping[str, str]:
    """Create or read the private Basic Auth identity without returning it from prepare."""

    if paths.credentials_path.is_file():
        try:
            payload = json.loads(paths.credentials_path.read_text(encoding="utf-8"))
        except (OSError, ValueError, UnicodeError) as error:
            raise JenkinsConfigError("pilot credentials file is unreadable") from error
        if (
            isinstance(payload, dict)
            and payload.get("schemaVersion") == 1
            and payload.get("username") == DEFAULT_USERNAME
            and isinstance(payload.get("password"), str)
            and len(payload["password"]) >= 32
        ):
            return {"username": DEFAULT_USERNAME, "password": payload["password"]}
        raise JenkinsConfigError("pilot credentials file has an invalid private schema")
    password = secrets.token_urlsafe(32)
    _private_file(
        paths.credentials_path,
        json.dumps(
            {"schemaVersion": 1, "username": DEFAULT_USERNAME, "password": password},
            sort_keys=True,
            separators=(",", ":"),
        ).encode("utf-8"),
    )
    return {"username": DEFAULT_USERNAME, "password": password}


def read_credentials(root: str | os.PathLike[str] = DEFAULT_PILOT_ROOT) -> Mapping[str, str]:
    """Read credentials for the in-process HTTP client; callers must keep them private."""

    return ensure_credentials(PilotPaths(root))


def read_manifest(root: str | os.PathLike[str] = DEFAULT_PILOT_ROOT) -> dict[str, object]:
    paths = PilotPaths(root)
    try:
        payload = json.loads(paths.manifest_path.read_text(encoding="utf-8"))
    except (OSError, ValueError, UnicodeError) as error:
        raise JenkinsConfigError("pilot manifest is missing or invalid") from error
    if not isinstance(payload, dict) or payload.get("schemaVersion") != 1:
        raise JenkinsConfigError("pilot manifest schema is unsupported")
    return payload


def write_manifest(paths: PilotPaths, payload: Mapping[str, object]) -> None:
    data = dict(payload)
    data["schemaVersion"] = 1
    data_bytes = json.dumps(data, sort_keys=True, indent=2, ensure_ascii=True).encode("utf-8")
    ManagedStorage(paths.root).atomic_write("pilot-manifest.json", data_bytes)


def restrict_private_root(root: Path) -> None:
    """Keep loopback administration credentials private to this Windows user."""
    require_managed_root(root, allow_root=False)
    result = subprocess.run(["whoami.exe", "/user", "/fo", "csv", "/nh"], capture_output=True, check=True)
    rows = list(csv.reader(io.StringIO(result.stdout.decode("utf-8"))))
    sid = rows[0][1]
    if not sid.startswith("S-1-"):
        raise JenkinsConfigError("cannot resolve the current user's Windows identity")
    subprocess.run(["icacls.exe", str(root), "/inheritance:r", "/grant:r", f"*{sid}:(OI)(CI)F", "*S-1-5-18:(OI)(CI)F"],
                   capture_output=True, check=True, creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))


def redacted_manifest(payload: Mapping[str, object]) -> dict[str, object]:
    """Return a safe API view; never include password, secret, or command line."""

    result = dict(payload)
    result.pop("credentials", None)
    result.pop("agentSecret", None)
    result.pop("processCommand", None)
    result.pop("commandLine", None)
    return result
