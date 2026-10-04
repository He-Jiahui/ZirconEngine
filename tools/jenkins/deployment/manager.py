from __future__ import annotations

import json
import os
import subprocess
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path
from typing import Any

from .bootstrap import JenkinsBootstrap
from .job_provisioning import provision_jobs
from .driver import verify_driver, driver_environment
from .lifecycle import check_port_available, launch_command, spawn_owned, wait_for_health
from .paths import DeploymentPaths
from .spec import DeploymentSpec, JenkinsError
from tools.jenkins.processes.identity import current_identity
from tools.jenkins.state.locks import process_lock
from tools.jenkins.contracts import digest

_LIVE_JOBS: dict[int, Any] = {}


def identity(pid: int) -> dict | None:
    """Compatibility observation helper used by reconciliation callers."""
    try:
        return current_identity(pid).to_dict()
    except Exception:
        return None


class DeploymentManager:
    """Short-lived controller lifecycle executor with persisted identity proofs."""

    def __init__(self, spec: DeploymentSpec, paths: DeploymentPaths,
                 java: Path | None = None, war: Path | None = None):
        self.spec, self.paths = spec, paths
        self.java = java or Path(spec.controller["java"]["executable"])
        self.war = war or Path(spec.controller["warPath"])
        self.journal = paths.state / "deployment-operation.json"

    @property
    def base_url(self) -> str:
        return f"http://{self.spec.controller['listenAddress']}:{self.spec.controller['httpPort']}"

    def _write(self, value: dict) -> None:
        self.paths.state.mkdir(parents=True, exist_ok=True)
        tmp = self.journal.with_suffix(".tmp")
        tmp.write_text(json.dumps(value, sort_keys=True, indent=2), encoding="utf-8")
        tmp.replace(self.journal)

    def _read(self) -> dict | None:
        try:
            return json.loads(self.journal.read_text(encoding="utf-8"))
        except (OSError, ValueError):
            return None

    def _credentials(self) -> tuple[str, str]:
        try:
            value = json.loads((self.paths.state / "credentials.json").read_text(encoding="utf-8"))
        except (OSError, ValueError) as exc:
            raise JenkinsError("credentials_missing", "Jenkins API credentials are unavailable") from exc
        token = value.get("apiToken") or value.get("password")
        if value.get("authenticationMode") != "api-token" or not value.get("username") or not token:
            raise JenkinsError("credentials_invalid", "Jenkins API token credentials are unavailable")
        return str(value["username"]), str(token)

    def _driver_env(self) -> dict[str, str]:
        record_path = self.paths.state / "deployment" / "driver.json"
        try:
            record = json.loads(record_path.read_text(encoding="utf-8"))
            digest = str(record["driverDigest"])
            root = Path(record["root"]).absolute()
        except (OSError, ValueError, KeyError, TypeError) as exc:
            raise JenkinsError("driver_environment_missing", "sealed driver record is unavailable") from exc
        expected_root = (self.paths.repo / ".jenkins" / "runtime" / "drivers" / digest).absolute()
        if root != expected_root:
            raise JenkinsError("driver_path_mismatch", "sealed driver root is outside the approved runtime")
        snapshot = verify_driver(root, expected_digest=digest)
        python = self.paths.repo / ".jenkins" / "runtime" / "python" / "python.exe"
        return driver_environment(snapshot, python, self.paths.repo)

    def _request(self, path: str, *, method: str = "GET", data: bytes | None = None, headers: dict[str, str] | None = None, timeout: float = 5.0) -> tuple[int, bytes]:
        import base64
        user, token = self._credentials()
        auth = base64.b64encode(f"{user}:{token}".encode()).decode()
        request_headers = {"Accept": "application/json", "Authorization": "Basic " + auth}
        request_headers.update(headers or {})
        req = urllib.request.Request(self.base_url.rstrip("/") + "/" + path.lstrip("/"), data=data, method=method,
                                     headers=request_headers)
        try:
            class DirectResponse(urllib.request.HTTPRedirectHandler):
                def redirect_request(self, req, fp, code, msg, headers, newurl):
                    return None
            # Maintenance POSTs often return 302. Their response is the
            # operation evidence; following the dashboard can obscure it.
            opener = urllib.request.build_opener(DirectResponse())
            with opener.open(req, timeout=timeout) as response:
                return response.status, response.read()
        except urllib.error.HTTPError as exc:
            return exc.code, exc.read()
        except (urllib.error.URLError, TimeoutError) as exc:
            raise JenkinsError("controller_unreachable", "Jenkins controller is unreachable", retryable=True) from exc

    def health(self) -> dict:
        controller = agent = plugins = False
        try:
            status, body = self._request("api/json")
            controller = status == 200 and bool(json.loads(body or b"{}"))
            if controller:
                _, body = self._request("computer/api/json?depth=1")
                nodes = json.loads(body).get("computer", [])
                node = next((n for n in nodes if n.get("displayName") == self.spec.agent["name"]), None)
                agent = bool(node and not node.get("offline", True) and not node.get("temporarilyOffline", False))
                _, body = self._request("pluginManager/api/json?depth=1")
                entries = json.loads(body).get("plugins", [])
                expected = {}
                try:
                    manifest = json.loads((self.paths.repo / ".jenkins" / "plugin-checksums.json").read_text(encoding="utf-8"))
                    expected = {str(k): str(v.get("version", "")) for k, v in manifest.get("plugins", {}).items()}
                except (OSError, ValueError):
                    expected = None
                if not expected:
                    plugins = False
                else:
                    observed = {str(p.get("shortName")): p for p in entries}
                    plugins = bool(entries) and all(observed.get(k, {}).get("active") and
                                                    not observed.get(k, {}).get("hasFailed", False) and
                                                    observed.get(k, {}).get("version") == v
                                                    for k, v in expected.items())
                    plugins = plugins and set(expected).issubset(observed)
        except (Exception, ValueError):
            pass
        return {"controller": controller, "agent": agent, "plugins": plugins, "ready": controller and agent and plugins}

    def start(self) -> dict:
        with process_lock(self.paths.state / "deployment-start.lock", reason="deployment_start_busy"):
            return self._start_locked(self.paths.state / "deployment-start.lock")

    def _start_locked(self, lock: Path) -> dict:
        old = self._read() or {}
        if old.get("state") in {"starting", "running", "stopping", "unknown-owner", "interrupted", "failed", "stopped"}:
            host_state = self.paths.state / "deployment" / "host.json"
            host_record = {}
            try:
                host_record = json.loads(host_state.read_text(encoding="utf-8"))
            except (OSError, ValueError):
                host_record = {}
            from .recovery import has_terminal_deployment_proofs
            if not has_terminal_deployment_proofs(old, host_record, str(self.paths.home.absolute())):
                observed = self.reconcile()
                if observed.get("observedAlive") or observed.get("healthProof", {}).get("controller"):
                    return observed
                from .recovery import previous_boot_deployment_evidence
                evidence = previous_boot_deployment_evidence(old, host_record, str(self.paths.home.absolute()))
                history = self.paths.state / "deployment" / "history"
                history.mkdir(parents=True, exist_ok=True)
                history_path = history / (digest(str(old["operationId"])) + ".json")
                if not history_path.exists():
                    history_path.write_text(json.dumps({"operation": old, "host": host_record,
                        "bootRecoveryEvidence": evidence}, indent=2), encoding="utf-8")
                self._write({**old, "state": "interrupted", "bootRecoveryEvidence": evidence})
        # Check endpoint availability before creating the lifecycle host.  A
        # bind failure must not create a new generation that cannot return a
        # trustworthy NativeJob termination receipt.
        check_port_available(self.spec.controller["listenAddress"], int(self.spec.controller["httpPort"]))
        operation_id = f"start-{time.time_ns()}"
        driver_env = self._driver_env()
        launcher = driver_env["ZIRCON_DRIVER_LAUNCHER"]
        driver_digest = driver_env["ZIRCON_DRIVER_DIGEST"]
        python = driver_env["JENKINS_PYTHON"]
        driver_record_path = self.paths.state / "deployment" / "driver.json"
        selected = json.loads(driver_record_path.read_text(encoding="utf-8"))
        if selected.get("driverDigest") != driver_digest:
            raise JenkinsError("driver_generation_changed", "The sealed driver changed during startup")
        selected.update(runtimeOperationId=operation_id, state="selected-for-runtime-start")
        pending_driver = driver_record_path.with_name("driver.pending-" + str(time.time_ns()) + ".json")
        pending_driver.write_text(json.dumps(selected, indent=2), encoding="utf-8")
        pending_driver.replace(driver_record_path)
        args = [python, launcher, "__lifecycle-host", "--spec", str(self.spec.path),
                "--java", str(self.java), "--war", str(self.war), "--operation-id", operation_id]
        env = dict(os.environ, **driver_env, PYTHONPATH=str(self.paths.repo), JENKINS_HOME=str(self.paths.home))
        host_log = (self.paths.logs / "lifecycle-host.log"); host_log.parent.mkdir(parents=True, exist_ok=True)
        log_handle = host_log.open("ab")
        process = subprocess.Popen(args, cwd=str(self.paths.repo), env=env,
                                   stdin=subprocess.DEVNULL, stdout=log_handle,
                                   stderr=subprocess.STDOUT,
                                   creationflags=getattr(subprocess, "CREATE_NEW_PROCESS_GROUP", 0) |
                                   getattr(subprocess, "CREATE_NO_WINDOW", 0))
        host_identity = current_identity(process.pid, executable=sys.executable, command_line=tuple(args))
        _LIVE_JOBS[process.pid] = (process, log_handle)
        operation = {"schemaVersion": 2, "operationId": operation_id, "state": "starting",
                     "component": "lifecycle-host", "hostPid": process.pid, **host_identity.to_dict(), "generation": operation_id, "health": False}
        # Bind every lifecycle operation to the exact Jenkins Home it owns.
        # A stale journal must never be used to stop a controller from another
        # deployment or a moved checkout.
        operation["homePath"] = str(self.paths.home.absolute())
        self._write(operation)
        operation["health"] = wait_for_health(self.base_url, 300, headers={"Authorization": "Basic " + __import__('base64').b64encode(f"{self._credentials()[0]}:{self._credentials()[1]}".encode()).decode()}, process=process)
        if not operation["health"]:
            operation.update({"state": "failed", "reasonCode": "controller_health_timeout"})
            # The lifecycle host owns the controller NativeJob and is the only
            # process allowed to publish its termination proof.  Wait for the
            # host to finish its failure cleanup, then copy only a complete
            # proof from host.json.  PID disappearance alone is deliberately
            # insufficient and leaves restart recovery blocked when the host
            # did not return evidence.
            host_process = _LIVE_JOBS.get(process.pid, (None, None))[0]
            if host_process is not None:
                try:
                    host_process.wait(timeout=35)
                except subprocess.TimeoutExpired:
                    operation["terminationProofPending"] = True
            host_state_path = self.paths.state / "deployment" / "host.json"
            try:
                host_state = json.loads(host_state_path.read_text(encoding="utf-8"))
            except (OSError, ValueError):
                host_state = {}
            proof = host_state.get("nativeTerminationProof")
            if isinstance(proof, dict) and proof.get("complete"):
                operation["nativeTerminationProof"] = proof
                operation["terminationProofPending"] = False
            else:
                operation["terminationProofPending"] = True
            self._write(operation)
            return operation
        try:
            deadline = time.monotonic() + 300
            ready = False
            while time.monotonic() < deadline:
                ready = self.health().get("ready", False)
                if ready:
                    break
                time.sleep(.5)
            if not ready:
                raise JenkinsError("deployment_not_ready", "controller or agent did not reach ready state")
            operation["healthProof"] = self.health()
            operation["state"] = "running"
        except JenkinsError as exc:
            operation.update({"state": "failed", "reasonCode": exc.code, "details": exc.details})
        self._write(operation)
        return operation

    def drain(self, timeout: float = 30.0) -> dict:
        status, _ = self._request("quietDown", method="POST")
        if status not in (200, 201, 302):
            raise JenkinsError("drain_failed", "Jenkins refused quietDown")
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            _, computer_body = self._request("computer/api/json?tree=computer[displayName,busyExecutors]")
            _, jobs_body = self._request("api/json?tree=jobs[name,builds[building]]")
            computers = json.loads(computer_body or b"{}").get("computer", [])
            jobs = json.loads(jobs_body or b"{}").get("jobs", [])
            busy = sum(int(c.get("busyExecutors", 0) or 0) for c in computers)
            building = [j.get("name") for j in jobs if any(b.get("building") for b in j.get("builds", []))]
            payload = {"busyExecutors": busy, "buildingJobs": building}
            # Durable execution records outlive Pipeline callers.  A Jenkins
            # quietDown alone is insufficient while an owner host still has a
            # queued or running execution; wait for its terminal state before
            # allowing controller shutdown.
            active_executions = self._active_execution_records()
            payload["activeExecutions"] = [r["key"] for r in active_executions]
            if busy == 0 and not building and not active_executions:
                return {"status": "drained", "proof": payload}
            time.sleep(.25)
        raise JenkinsError("drain_timeout", "active Jenkins builds did not drain", retryable=True)

    def _active_execution_records(self) -> list[dict]:
        """Return durable executions which still own work during shutdown.

        State is optional for older deployments; a missing database means
        there are no durable executions to drain.  Reads are intentionally
        fail-closed on malformed state so stop cannot claim a false drain.
        """
        state_path = self.paths.state / "coordination.sqlite3"
        if not state_path.exists():
            return []
        try:
            from ..state import State
            state = State(state_path)
            active = {"pending", "admitted", "claimed", "launching", "started", "running", "cancel_requested", "reconciling"}
            return [r for r in state.list("execution") if r.get("payload", {}).get("status") in active]
        except Exception as exc:
            raise JenkinsError("execution_state_unavailable", "durable execution state could not be inspected",
                               retryable=True) from exc

    def stop(self) -> dict:
        with process_lock(self.paths.state / "deployment-start.lock", reason="deployment_operation_busy"):
            return self._stop_locked()

    def _stop_locked(self) -> dict:
        operation = self._read()
        if not operation:
            return {"status": "stopped"}
        expected_home = str(self.paths.home.absolute())
        recorded_home = operation.get("homePath")
        if recorded_home != expected_home:
            raise JenkinsError("home_path_mismatch", "deployment journal is not bound to this Jenkins Home",
                               details={"expectedHomePath": expected_home, "recordedHomePath": recorded_home})
        self.drain()
        pid = int(operation.get("pid", 0)); host = _LIVE_JOBS.get(pid)
        command = self.paths.state / "deployment" / "commands" / f"stop-{time.time_ns()}.json"
        command.parent.mkdir(parents=True, exist_ok=True)
        host_state = self.paths.state / "deployment" / "host.json"
        try:
            host_record = json.loads(host_state.read_text(encoding="utf-8"))
        except (OSError, ValueError) as exc:
            raise JenkinsError("host_state_missing", "trusted lifecycle host state is unavailable") from exc
        if host_record.get("operationId") != operation.get("operationId") or host_record.get("generation") != operation.get("generation"):
            raise JenkinsError("host_generation_mismatch", "lifecycle host generation changed before stop")
        command.write_text(json.dumps({"action": "stop", "operationId": host_record.get("operationId"),
                                       "hostPid": host_record.get("hostPid"),
                                       "controllerIdentity": host_record.get("controllerIdentity", host_record.get("identity")),
                                       "generation": host_record.get("generation")}), encoding="utf-8")
        result_path = command.with_suffix(".result.json")
        deadline = time.monotonic() + 45
        while time.monotonic() < deadline and not result_path.exists():
            time.sleep(.25)
        if not result_path.exists():
            raise JenkinsError("host_stop_timeout", "lifecycle host did not return termination proof")
        result = json.loads(result_path.read_text(encoding="utf-8"))
        if result.get("status") != "stopped":
            raise JenkinsError("host_stop_failed", "lifecycle host failed to stop controller")
        operation.update({"state": "stopped", "nativeTerminationProof": result.get("nativeTerminationProof"),
                          "agentTerminationProof": result.get("agentTerminationProof")})
        if host is not None:
            proc, handle = host; proc.wait(timeout=10); handle.close(); _LIVE_JOBS.pop(pid, None)
        self._write(operation)
        return operation

    def reconcile(self) -> dict:
        operation = self._read() or {}
        observed = identity(int(operation["pid"])) if operation.get("pid") else None
        operation["observedAlive"] = bool(observed and (
            observed.get("creationTime", observed.get("birth")) == operation.get("creationTime", operation.get("birth"))))
        host_state = self.paths.state / "deployment" / "host.json"
        try:
            host = json.loads(host_state.read_text(encoding="utf-8"))
            operation["hostIdentityProof"] = {
                "operationMatches": host.get("operationId") == operation.get("operationId"),
                "generationMatches": host.get("generation") == operation.get("generation"),
                "hostPidMatches": host.get("hostPid") == operation.get("hostPid"),
                "controllerIdentity": host.get("controllerIdentity", host.get("identity")),
                "agentIdentity": host.get("agentIdentity"),
                "status": host.get("status"),
            }
            if not all((operation["hostIdentityProof"]["operationMatches"],
                        operation["hostIdentityProof"]["generationMatches"],
                        operation["hostIdentityProof"]["hostPidMatches"])):
                operation["state"] = "unknown-owner"
        except (OSError, ValueError):
            operation["state"] = "unknown-owner"
        operation["healthProof"] = self.health()
        # A healthy HTTP endpoint does not establish ownership.  Keep
        # unknown-owner fail-closed even when another Jenkins instance happens
        # to answer on the configured port.
        owner_proven = bool(operation.get("hostIdentityProof", {}).get("operationMatches") and
                            operation.get("hostIdentityProof", {}).get("generationMatches") and
                            operation.get("hostIdentityProof", {}).get("hostPidMatches"))
        if owner_proven and operation["healthProof"]["ready"]:
            operation["state"] = "running"
        self._write(operation)
        return operation

    def bootstrap(self) -> dict:
        result = JenkinsBootstrap(self.spec, self.paths, self).apply()
        self._write({**(self._read() or {}), **result})
        return result

    def register_jobs(self) -> dict:
        return provision_jobs(self.spec, self.paths, self)
