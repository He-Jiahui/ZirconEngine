"""Portable Jenkins pilot bootstrap.

All controller and agent state is confined below one managed cargo-targets root.
No global Java installation, Windows service, or unrelated process is touched.
"""

from __future__ import annotations

import base64
import hashlib
import json
import os
import re
import socket
import subprocess
import time
import urllib.error
import urllib.parse
import urllib.request
import zipfile
from pathlib import Path
from typing import Callable, Mapping

from . import jenkins_config as config
from .jenkins_config import (
    DEFAULT_AGENT_EXECUTORS,
    DEFAULT_AGENT_LABEL,
    DEFAULT_AGENT_NAME,
    DEFAULT_CONTROLLER_EXECUTORS,
    DEFAULT_CONTROLLER_HOST,
    DEFAULT_HTTP_PORT,
)
from .jenkins_config import PilotPaths, ensure_credentials, read_credentials, read_manifest, write_manifest
from .storage import ManagedStorage, require_managed_root
from .client import JenkinsClient
from .lifecycle import serialized_lifecycle

USER_AGENT = "ZirconEngine-JenkinsPilot/1"
DOWNLOAD_TIMEOUT_SECONDS = 120
STARTUP_TIMEOUT_SECONDS = 120
MAX_METADATA_BYTES = 16 * 1024 * 1024


class BootstrapError(RuntimeError):
    """A preparation or process ownership failure."""


def _request_bytes(url: str, *, limit: int = MAX_METADATA_BYTES, credentials: Mapping[str, str] | None = None) -> bytes:
    headers = {"User-Agent": USER_AGENT}
    if credentials:
        headers["Authorization"] = "Basic " + base64.b64encode(
            f"{credentials['username']}:{credentials['password']}".encode("utf-8")
        ).decode("ascii")
    request = urllib.request.Request(url, headers=headers)
    try:
        with urllib.request.urlopen(request, timeout=DOWNLOAD_TIMEOUT_SECONDS) as response:
            content = response.read(limit + 1)
    except (urllib.error.URLError, OSError, TimeoutError) as error:
        raise BootstrapError("Jenkins pilot HTTP request failed") from error
    if len(content) > limit:
        raise BootstrapError("download exceeded the pilot limit")
    return content


def _java_major(executable: Path) -> int | None:
    if not executable.is_file():
        return None
    try:
        result = subprocess.run(
            [str(executable), "-version"], capture_output=True, text=True, timeout=10,
            check=False, creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
        )
    except (OSError, subprocess.TimeoutExpired):
        return None
    text = (result.stderr or "") + "\n" + (result.stdout or "")
    match = re.search(r'(?i)version\s+"([0-9]+)', text)
    if not match:
        return None
    major = int(match.group(1))
    return 8 if major == 1 else major


def _write_security_bootstrap(paths: PilotPaths) -> None:
    # Jenkins receives its automation credential through a private file path.
    # API token mode preserves the user's separately saved login password.
    script = """import jenkins.model.Jenkins
import hudson.model.User
import hudson.security.HudsonPrivateSecurityRealm
import hudson.security.FullControlOnceLoggedInAuthorizationStrategy
import jenkins.model.JenkinsLocationConfiguration
import jenkins.security.ApiTokenProperty

def j = Jenkins.get()
def credentials = new File(System.getenv("JENKINS_PILOT_CREDENTIALS"))
def payload = new groovy.json.JsonSlurper().parse(credentials)
def realm = j.getSecurityRealm() instanceof HudsonPrivateSecurityRealm ? j.getSecurityRealm() : new HudsonPrivateSecurityRealm(false)
def authenticationMode = payload.containsKey("authenticationMode") ? payload.authenticationMode : "password"
if (authenticationMode == "password") {
    realm.createAccount(payload.username, payload.password)
} else if (authenticationMode == "api-token") {
    def user = User.getById(payload.username, false)
    def token = user?.getProperty(ApiTokenProperty.class)
    if (!(j.getSecurityRealm() instanceof HudsonPrivateSecurityRealm) ||
            user?.getProperty(HudsonPrivateSecurityRealm.Details.class) == null ||
            token == null || !token.matchesPassword(payload.password)) {
        throw new IllegalStateException("Configured administrator API token is unavailable")
    }
} else {
    throw new IllegalStateException("Unsupported pilot authentication mode")
}
j.setSecurityRealm(realm)
def auth = new FullControlOnceLoggedInAuthorizationStrategy()
auth.setAllowAnonymousRead(false)
j.setAuthorizationStrategy(auth)
j.setNumExecutors(0)
j.setSlaveAgentPort(-1)
JenkinsLocationConfiguration.get().setUrl(System.getenv("JENKINS_PILOT_URL"))
j.save()
"""
    with ManagedStorage(paths.root).backend() as backend:
        backend.write_bytes("jenkins_home/init.groovy.d/00-pilot-security.groovy", script.encode("utf-8"))


def _read_state(paths: PilotPaths) -> dict[str, object]:
    if not paths.process_state_path.is_file():
        return {}
    try:
        value = json.loads(paths.process_state_path.read_text(encoding="utf-8"))
    except (OSError, ValueError, UnicodeError) as error:
        raise BootstrapError("pilot process state is invalid") from error
    if not isinstance(value, dict):
        raise BootstrapError("pilot process state is not an object")
    return value


def _write_state(paths: PilotPaths, state: Mapping[str, object]) -> None:
    ManagedStorage(paths.root).atomic_write(
        paths.process_state_path.relative_to(paths.root).as_posix(),
        json.dumps(dict(state), sort_keys=True, indent=2).encode("utf-8"),
    )


def _alive(pid: object) -> bool:
    if type(pid) is not int or pid <= 0:
        return False
    from .native.process_identity import process_is_alive
    return process_is_alive(pid)


def _free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as listener:
        listener.bind((DEFAULT_CONTROLLER_HOST, 0))
        return int(listener.getsockname()[1])


def _require_no_pending_launch(state: Mapping[str, object]) -> None:
    if state.get("pendingLaunch"):
        raise BootstrapError("runtime launch remains uncertain; reconcile its existing keeper Job before mutation")


def _launch_intent(paths: PilotPaths, state: Mapping[str, object], kind: str) -> None:
    from .native.process_identity import process_creation_time
    _write_state(paths, {**state, "pendingLaunch": {"kind": kind, "launcherPid": os.getpid(),
                                                 "launcherCreationTime": process_creation_time(os.getpid())}})


def _start_process(paths: PilotPaths, kind: str, args: list[str], env: Mapping[str, str], *,
                   on_record: Callable[[dict[str, object]], None],
                   runtime_assets: Mapping[str, str]) -> dict[str, object]:
    from .governance import require_live_storage_owner
    from .lifetime import launch_owned
    lease = require_live_storage_owner(paths.root, Path.cwd())
    log = paths.controller_log if kind == "controller" else paths.agent_log
    with ManagedStorage(paths.root).backend() as backend:
        backend.ensure_directory("logs")
        with backend.open_lock_file(log.relative_to(paths.root).as_posix()) as stream:
            stream.seek(0, 2)
            record = launch_owned(args, cwd=paths.root, env=env, stream=stream, lease=lease,
                                  log_path=log, runtime_assets=runtime_assets,
                                  on_record=lambda value: on_record({**value, "kind": kind}))
    return {**record, "kind": kind}


def _wait_controller(url: str, timeout: float = STARTUP_TIMEOUT_SECONDS) -> None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        try:
            request = urllib.request.Request(url.rstrip("/") + "/login", headers={"User-Agent": USER_AGENT})
            with urllib.request.urlopen(request, timeout=3) as response:
                if response.status in {200, 403}:
                    return
        except (urllib.error.URLError, OSError, TimeoutError):
            time.sleep(0.25)
    raise BootstrapError("Jenkins controller did not become ready before timeout")


@serialized_lifecycle
def start_controller(root: str | os.PathLike[str] = config.DEFAULT_PILOT_ROOT) -> dict[str, object]:
    paths = PilotPaths(root)
    from .governance import require_live_storage_owner
    lease = require_live_storage_owner(paths.root, Path.cwd())
    from .assets import prepared_assets_context
    with prepared_assets_context(paths.root) as assets_digest:
        manifest = read_manifest(root)
        java = Path(str(manifest.get("javaPath") or paths.java_root / "bin" / "java.exe"))
        if not java.is_file() or (_java_major(java) or 0) < 21:
            raise BootstrapError("prepared Java 21 executable is unavailable")
        if not paths.war_path.is_file():
            raise BootstrapError("prepared Jenkins WAR is unavailable")
        state = _read_state(paths)
        _require_no_pending_launch(state)
        existing = state.get("controller")
        if isinstance(existing, dict) and _record_alive(existing):
            _require_keeper_record(existing, lease)
            return {"url": str(state.get("url", "")), "pid": existing["pid"], "alreadyRunning": True}
        port = int(state.get("port") or _free_port())
        _write_security_bootstrap(paths)
        credentials = ensure_credentials(paths)
        env = os.environ.copy()
        env.update({
            "JENKINS_HOME": str(paths.jenkins_home),
            "JENKINS_PILOT_CREDENTIALS": str(paths.credentials_path),
            "JENKINS_PILOT_URL": f"http://{DEFAULT_CONTROLLER_HOST}:{port}/",
            "TMP": str(paths.cache_root), "TEMP": str(paths.cache_root),
        })
        args = [
            str(java), "-Xms128m", "-Xmx512m", "-Djava.io.tmpdir=" + str(paths.cache_root), "-Djenkins.install.runSetupWizard=false", "-jar", str(paths.war_path),
            "--httpListenAddress=" + DEFAULT_CONTROLLER_HOST,
            "--httpPort=" + str(port),
            "--webroot=" + str(paths.cache_root / "war"),
        ]
        url = f"http://{DEFAULT_CONTROLLER_HOST}:{port}/"
        _launch_intent(paths, state, "controller")
        record = _start_process(paths, "controller", args, env,
                                runtime_assets={"root": str(paths.root), "kind": "controller", "proofSha256": assets_digest},
                                on_record=lambda value: _write_state(paths, {**state, "controller": value, "url": url, "port": port}))
    try:
        from .lifetime import wait_runtime_started
        wait_runtime_started(record, preparation_seconds=180)
        _write_state(paths, {**state, "controller": record, "url": url, "port": port})
        manifest["controllerUrl"] = url
        write_manifest(paths, manifest)
        _wait_controller(url)
    except BaseException:
        if not _terminate_record(record) and _record_alive(record):
            raise BootstrapError("failed controller startup remains live; preserve its storage lease")
        raise
    # Keep credentials private; start_controller returns endpoint and identity only.
    _ = credentials
    return {"url": url, "pid": record["pid"], "alreadyRunning": False}


def _http_bytes(url: str, credentials: Mapping[str, str]) -> bytes:
    return _request_bytes(url, credentials=credentials)


def _http_json(url: str, credentials: Mapping[str, str]) -> Mapping[str, object]:
    try:
        value = json.loads(_http_bytes(url, credentials).decode("utf-8"))
    except (ValueError, UnicodeError) as error:
        raise BootstrapError("Jenkins endpoint returned invalid JSON") from error
    if not isinstance(value, dict):
        raise BootstrapError("Jenkins endpoint returned a non-object")
    return value


def _crumb(url: str, credentials: Mapping[str, str]) -> tuple[str, str]:
    value = _http_json(url.rstrip("/") + "/crumbIssuer/api/json", credentials)
    field, crumb = value.get("crumbRequestField"), value.get("crumb")
    if not isinstance(field, str) or not isinstance(crumb, str):
        raise BootstrapError("Jenkins CSRF crumb is unavailable")
    return field, crumb


def _agent_secret(url: str, credentials: Mapping[str, str], name: str) -> str:
    endpoint = url.rstrip("/") + "/computer/" + urllib.parse.quote(name, safe="") + "/jenkins-agent.jnlp"
    raw = _http_bytes(endpoint, credentials).decode("utf-8", errors="replace")
    match = re.search(r"<application-desc[^>]*>.*?<argument>([^<]+)</argument>", raw, re.S)
    if not match:
        raise BootstrapError("Jenkins inbound agent secret is unavailable")
    return match.group(1).strip()


def _prepare_agent_archive(paths: PilotPaths) -> str:
    """Extract the exact remoting bytes bound by the verified Jenkins WAR."""
    with ManagedStorage(paths.root).backend() as backend:
        with backend._windows.open_relative("war/jenkins.war", "rb") as stream, zipfile.ZipFile(stream) as archive:
            candidates = [info for info in archive.infolist()
                          if info.filename.startswith("WEB-INF/lib/remoting-") and info.filename.endswith(".jar")]
            if len(candidates) != 1 or candidates[0].file_size > 32 * 1024 * 1024:
                raise BootstrapError("verified WAR lacks one bounded remoting archive")
            payload = archive.read(candidates[0])
        sha256 = hashlib.sha256(payload).hexdigest()
        if backend.exists("war/agent.jar"):
            backend.read_bytes("war/agent.jar", expected_sha256=sha256, max_bytes=32 * 1024 * 1024)
        else:
            backend.write_bytes("war/agent.jar", payload, expected_sha256=sha256)
        return sha256


def _ensure_agent_node(paths: PilotPaths, url: str, credentials: Mapping[str, str], name: str, label: str) -> None:
    endpoint = url.rstrip("/") + "/computer/" + urllib.parse.quote(name, safe="") + "/api/json"
    try:
        _http_json(endpoint, credentials)
        return
    except BootstrapError:
        pass
    field, crumb = _crumb(url, credentials)
    xml = (
        "<slave><name>{}</name><remoteFS>{}</remoteFS><numExecutors>{}</numExecutors>"
        "<mode>NORMAL</mode><label>{}</label><launcher class=\"hudson.slaves.JNLPLauncher\">"
        "<webSocket>true</webSocket></launcher><retentionStrategy class=\"hudson.slaves.RetentionStrategy$Always\"/>"
        "</slave>"
    ).format(name, str(paths.node_work_root), DEFAULT_AGENT_EXECUTORS, label).encode("utf-8")
    endpoint = url.rstrip("/") + "/computer/doCreateItem?name=" + urllib.parse.quote(name, safe="")
    auth = base64.b64encode(f"{credentials['username']}:{credentials['password']}".encode()).decode()
    request = urllib.request.Request(
        endpoint, data=xml,
        headers={"Authorization": "Basic " + auth, "Content-Type": "application/xml", field: crumb},
        method="POST",
    )
    try:
        with urllib.request.urlopen(request, timeout=15):
            return
    except (urllib.error.URLError, OSError, TimeoutError) as error:
        raise BootstrapError("could not create Jenkins inbound agent node") from error


@serialized_lifecycle
def start_agent(root: str | os.PathLike[str] = config.DEFAULT_PILOT_ROOT) -> dict[str, object]:
    paths = PilotPaths(root)
    from .governance import require_live_storage_owner
    lease = require_live_storage_owner(paths.root, Path.cwd())
    from .assets import prepared_assets_context
    with prepared_assets_context(paths.root) as assets_digest:
        manifest = read_manifest(root)
        state = _read_state(paths)
        _require_no_pending_launch(state)
        if isinstance(state.get("agent"), dict) and _record_alive(state["agent"]):
            _require_keeper_record(state["agent"], lease)
            return {"pid": state["agent"]["pid"], "name": manifest["agentName"], "alreadyRunning": True}
        controller = state.get("controller")
        if not isinstance(controller, dict) or not _record_alive(controller):
            raise BootstrapError("controller must be running before the agent starts")
        _require_keeper_record(controller, lease)
        url = str(state.get("url") or "")
        if not url.startswith("http://" + DEFAULT_CONTROLLER_HOST + ":"):
            raise BootstrapError("controller URL is outside the loopback pilot")
        credentials = read_credentials(root)
        name = str(manifest.get("agentName") or DEFAULT_AGENT_NAME)
        label = str(manifest.get("agentLabel") or DEFAULT_AGENT_LABEL)
        client = JenkinsClient(url, **credentials)
        script = """import jenkins.model.Jenkins
    import hudson.model.Node
    import hudson.slaves.DumbSlave
    import hudson.slaves.JNLPLauncher
    import hudson.slaves.RetentionStrategy
    def j=Jenkins.get()
    def name=NAME
    if(j.getNode(name)==null) {
     def launcher=new JNLPLauncher(); launcher.setWebSocket(true)
     def node=new DumbSlave(name,'isolated Jenkins pilot',ROOT,'1',Node.Mode.NORMAL,LABEL,launcher,new RetentionStrategy.Always(),[])
     j.addNode(node)
    }
    println('node ready')
    """.replace('NAME', json.dumps(name)).replace('ROOT', json.dumps(str(paths.node_work_root))).replace('LABEL', json.dumps(label))
        client.request("/scriptText", post=True, content_type="application/x-www-form-urlencoded",
                       data=urllib.parse.urlencode({"script": script}).encode())
        secret = _agent_secret(url, credentials, name)
        java = Path(str(manifest.get("javaPath")))
        if not java.is_file():
            raise BootstrapError("prepared Java executable is unavailable")
        agent_sha256 = _prepare_agent_archive(paths)
        env = os.environ.copy()
        env.update({"TMP": str(paths.cache_root), "TEMP": str(paths.cache_root),
                    "JENKINS_PILOT_ROOT": str(paths.root), "ZIRCON_REPO_ROOT": str(manifest.get("repoRoot") or Path.cwd()),
                    "JENKINS_PILOT_PYTHON": os.sys.executable, "PYTHONPATH": str(manifest["driverRoot"]),
                    "PYTHONDONTWRITEBYTECODE": "1", "PYTHONUTF8": "1"})
        args = [
            str(java), "-Xms64m", "-Xmx256m", "-jar", str(paths.agent_jar), "-url", url, "-secret", secret,
            "-name", name, "-webSocket", "-workDir", str(paths.node_work_root),
        ]
        _launch_intent(paths, state, "agent")
        with ManagedStorage(paths.root).backend() as archive_guard:
            with archive_guard._windows.open_relative("war/agent.jar", "rb"):
                archive_guard.read_bytes("war/agent.jar", expected_sha256=agent_sha256, max_bytes=32 * 1024 * 1024)
                record = _start_process(paths, "agent", args, env,
                                        runtime_assets={"root": str(paths.root), "kind": "agent",
                                                        "proofSha256": assets_digest, "agentJarSha256": agent_sha256},
                                        on_record=lambda value: _write_state(paths, {**state, "agent": value, "agentName": name}))
        try:
            from .lifetime import wait_runtime_started
            wait_runtime_started(record, preparation_seconds=180)
            _write_state(paths, {**state, "agent": record, "agentName": name})
        except BaseException:
            if not _terminate_record(record) and _record_alive(record):
                raise BootstrapError("failed agent startup remains live; preserve its storage lease")
            raise
        return {"pid": record["pid"], "name": name, "alreadyRunning": False}


def _record_alive(record: Mapping[str, object]) -> bool:
    from .native.process_identity import process_matches_creation_time
    return type(record.get("pid")) is int and bool(record.get("creationTime")) and process_matches_creation_time(record["pid"], str(record["creationTime"]))


def _require_keeper_record(record: Mapping[str, object], lease: Mapping[str, object]) -> None:
    if not record.get("keeperJobHandle") or record.get("keeperPid") != lease["owner_pid"] or record.get("keeperCreationTime") != lease["owner_process_creation_time"]:
        raise BootstrapError("live legacy runtime lacks the current keeper Job proof; reconcile it before adoption")


def _terminate_record(record: Mapping[str, object]) -> bool:
    if record.get("keeperJobHandle"):
        from .lifetime import stop_owned
        return stop_owned(record)
    if _alive(record.get("pid")):
        raise BootstrapError("legacy runtime has no native Job proof; preserve its lease and review exact process ownership")
    return False


@serialized_lifecycle
def stop(root: str | os.PathLike[str] = config.DEFAULT_PILOT_ROOT) -> dict[str, object]:
    paths = PilotPaths(root)
    state = _read_state(paths)
    _require_no_pending_launch(state)
    stopped: dict[str, bool] = {}
    for kind in ("agent", "controller"):
        record = state.get(kind)
        stopped[kind] = _terminate_record(record) if isinstance(record, dict) else False
    live = [kind for kind in ("agent", "controller") if isinstance(state.get(kind), dict) and _record_alive(state[kind])]
    if live:
        raise BootstrapError("owned pilot processes remain live; preserve the storage lease: " + ", ".join(live))
    from .native.process_identity import process_matches_creation_time
    for metadata in (paths.root / "runs").rglob("process.json"):
        require_managed_root(metadata, allow_root=False)
        process = json.loads(metadata.read_bytes())
        if process_matches_creation_time(process["pid"], process["creationTime"]):
            raise BootstrapError("owned validation process remains live; preserve the storage lease")
    for metadata in (paths.root / "runs").rglob("probe.json"):
        require_managed_root(metadata, allow_root=False)
        process = json.loads(metadata.read_bytes())
        for kind in ("parent", "child"):
            if process_matches_creation_time(process[kind + "Pid"], process[kind + "CreationTime"]):
                raise BootstrapError("owned validation descendant remains live; preserve the storage lease")
    remaining = {key: value for key, value in state.items() if key not in {"agent", "controller"}}
    _write_state(paths, remaining)
    return {"stopped": stopped}


__all__ = ["BootstrapError", "start_controller", "start_agent", "stop"]
