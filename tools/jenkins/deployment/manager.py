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
from .driver import verify_driver, driver_environment, runtime_agent_name
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

    def _archive_deployment(self, operation: dict, host: dict, evidence: dict) -> None:
        """Preserve proofs for each replaced generation without overwriting history."""
        history = self.paths.state / "deployment" / "history"
        history.mkdir(parents=True, exist_ok=True)
        record = {"operation": operation, "host": host, "deploymentRecoveryEvidence": evidence}
        archive = history / (digest(str(operation["operationId"])) + "-" + digest(record) + ".json")
        contents = json.dumps(record, sort_keys=True, indent=2)
        if archive.exists():
            if archive.read_text(encoding="utf-8") != contents:
                raise JenkinsError("deployment_history_mismatch", "Deployment history identity changed")
            return
        pending = archive.with_suffix(".pending.json")
        pending.write_text(contents, encoding="utf-8")
        pending.rename(archive)

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
                operation = self._read() or {}
                selector = json.loads((self.paths.state / "deployment/driver.json").read_text(encoding="utf-8"))
                runtime_operation = selector.get("runtimeOperationId")
                agent_name = ""
                if (runtime_operation and runtime_operation == operation.get("operationId")
                        and runtime_operation == operation.get("generation")):
                    agent_name = runtime_agent_name(self.spec.agent["name"], runtime_operation)
                node = next((n for n in nodes if agent_name and n.get("displayName") == agent_name), None)
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

    def _start_locked(self, lock: Path, *, recovery_driver: dict | None = None, execution_activation=None) -> dict:
        start_directive = None
        if execution_activation is None:
            from .execution_activation import assert_activation_start_allowed
            start_directive = assert_activation_start_allowed(self)
            if start_directive is not None:
                receipt = start_directive.get("recoveryReceipt") if isinstance(start_directive, dict) else None
                if not (isinstance(start_directive, dict) and start_directive.get("controlPlaneOnly") is True
                        and isinstance(receipt, dict) and isinstance(receipt.get("path"), str)
                        and isinstance(receipt.get("sha256"), str) and len(receipt["sha256"]) == 64):
                    raise JenkinsError("execution_recovery_unresolved", "Verified recovery startup directive is incomplete")
        old = self._read() or {}
        # A controller-only deployment remains controller-only across a
        # normal stop/start cycle. Enabling an agent or execution broker is a
        # separate, explicit runtime transition.
        control_plane_only = old.get("controlPlaneOnly") is True or start_directive is not None
        if execution_activation is not None:
            from .execution_activation import ActivationPlan
            if not isinstance(execution_activation, ActivationPlan):
                raise JenkinsError("execution_activation_invalid", "Execution activation requires a verified transition plan")
            execution_activation.validate(self, candidate_selected=True)
            if old.get("state") not in {"stopped", "failed", "interrupted"}:
                raise JenkinsError("execution_transition_requires_terminal", "Execution activation requires a terminal previous generation")
            control_plane_only = False
        if old.get("state") in {"starting", "running", "stopping", "unknown-owner", "interrupted", "failed", "stopped"}:
            host_state = self.paths.state / "deployment" / "host.json"
            host_record = {}
            try:
                host_record = json.loads(host_state.read_text(encoding="utf-8"))
            except (OSError, ValueError):
                host_record = {}
            from .recovery import has_terminal_deployment_proofs, has_terminal_lifecycle_proofs
            terminal = has_terminal_deployment_proofs(old, host_record, str(self.paths.home.absolute()))
            host_observation = None
            if terminal:
                from .startup_observations import observe_departed_identity
                host_observation = observe_departed_identity(old)
                if not has_terminal_lifecycle_proofs(old, host_record, str(self.paths.home.absolute()), host_observation):
                    raise JenkinsError("deployment_termination_unproven", "Lifecycle supervisor departure is unproven",
                                       retryable=True)
            if not terminal:
                observed = self.reconcile()
                host_proof = observed.get("hostIdentityProof") or {}
                host_binding = all(host_proof.get(field) is True for field in (
                    "operationMatches", "generationMatches", "hostPidMatches")) \
                    and host_proof.get("status") == "running"
                # A healthy HTTP endpoint is not an ownership proof. Return
                # an existing deployment only when the lifecycle host itself
                # is alive under the recorded birth identity and its durable
                # host record is bound to this operation. This prevents a
                # different Jenkins instance on the same port from being
                # adopted by a stale journal.
                if observed.get("observedAlive") and host_binding:
                    return observed
                if observed.get("healthProof", {}).get("controller"):
                    raise JenkinsError(
                        "deployment_owner_unproven",
                        "A healthy Jenkins endpoint cannot be attributed to this lifecycle host",
                        retryable=True,
                        details={"hostIdentityProof": host_proof},
                    )
                from .recovery import replacement_deployment_evidence
                previous_driver = recovery_driver
                if previous_driver is None:
                    try:
                        previous_driver = json.loads((self.paths.state / "deployment/driver.json").read_text(encoding="utf-8"))
                    except (OSError, ValueError):
                        # An unaudited or missing selector cannot use the
                        # pre-agent exception. Preserve the boot-boundary gate.
                        previous_driver = {}
                evidence = replacement_deployment_evidence(old, host_record, str(self.paths.home.absolute()),
                    self.paths.repo, previous_driver, self.spec.controller["listenAddress"], int(self.spec.controller["httpPort"]))
                control_plane_only = control_plane_only or evidence.get("controlPlaneOnly") is True
                self._archive_deployment(old, host_record, evidence)
                self._write({**old, "state": "interrupted", "deploymentRecoveryEvidence": evidence})
            else:
                self._archive_deployment(old, host_record, {
                    "kind": "owned-native-terminal-proof", "operationId": old["operationId"],
                    "hostObservation": host_observation,
                    "executionHoldsReleaseAllowed": False, "validationAcceptanceAllowed": False})
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
        if control_plane_only:
            env["ZIRCON_JENKINS_RECOVERY_CONTROL_ONLY"] = "1"
        else:
            env.pop("ZIRCON_JENKINS_RECOVERY_CONTROL_ONLY", None)
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
        operation["controlPlaneOnly"] = control_plane_only
        if start_directive is not None:
            operation["executionActivationRecoveryReceipt"] = dict(start_directive["recoveryReceipt"])
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
                proof = self.health()
                ready = (proof["controller"] and proof["plugins"]) if control_plane_only else proof.get("ready", False)
                if ready and control_plane_only:
                    try:
                        host_state = json.loads((self.paths.state / "deployment/host.json").read_text(encoding="utf-8"))
                        ready = (host_state.get("operationId") == operation_id
                                 and host_state.get("status") == "running"
                                 and host_state.get("controlPlaneOnly") is True)
                    except (OSError, ValueError):
                        ready = False
                if ready:
                    break
                time.sleep(.5)
            if not ready:
                raise JenkinsError("deployment_not_ready", "controller or agent did not reach ready state")
            operation["healthProof"] = self.health()
            operation["serviceReady"] = ready
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

    def _control_plane_drain(self, operation: dict, host: dict, timeout: float = 30.0) -> dict:
        """Drain only the current controller generation in controller-only mode.

        Historical execution records belong to retired lifecycle generations
        and remain protected; they must not make a controller-only UI stop
        impossible. Any active record that cannot be proved historical, or any
        current-generation native execution, still blocks the stop.
        """
        if (operation.get("controlPlaneOnly") is not True
                or host.get("controlPlaneOnly") is not True
                or host.get("operationId") != operation.get("operationId")
                or host.get("generation") != operation.get("generation")
                or host.get("hostPid") != operation.get("hostPid")
                or host.get("agentLaunchAttempted") is not False
                or host.get("agentIdentity")):
            raise JenkinsError("control_plane_stop_unproven", "controller-only stop ownership is unproven")

        def matches(expected: dict | None, observed: dict | None) -> bool:
            if not expected or not observed:
                return False
            expected_birth = expected.get("creationTime", expected.get("birth"))
            observed_birth = observed.get("creationTime", observed.get("birth"))
            expected_exe = expected.get("executable")
            observed_exe = observed.get("executable")
            return (str(expected_birth) == str(observed_birth) and bool(expected_exe and observed_exe)
                    and os.path.normcase(os.path.abspath(str(expected_exe))) ==
                    os.path.normcase(os.path.abspath(str(observed_exe))))

        observed_host = identity(int(operation.get("pid", 0))) if operation.get("pid") else None
        observed_controller = None
        controller = host.get("controllerIdentity") or host.get("identity")
        if controller and controller.get("pid"):
            observed_controller = identity(int(controller["pid"]))
        if not matches(operation, observed_host) or not matches(controller, observed_controller):
            raise JenkinsError("control_plane_stop_unproven", "controller-only process identity is not provably owned",
                               retryable=True)

        active = self._active_execution_records()
        current = []
        for record in active:
            payload = record.get("payload") or {}
            if not payload.get("operationId") or not payload.get("generation"):
                raise JenkinsError("control_plane_execution_ambiguous", "active execution ownership is unknown",
                                   retryable=True)
            if str(payload.get("operationId")) == str(operation.get("operationId")):
                current.append(record)
        if current:
            raise JenkinsError("control_plane_execution_active", "current-generation execution is still active",
                               retryable=True, details={"executions": [r["key"] for r in current]})

        status, _ = self._request("quietDown", method="POST")
        if status not in (200, 201, 302):
            raise JenkinsError("drain_failed", "Jenkins refused quietDown")
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            computer_status, computer_body = self._request("computer/api/json?tree=computer[displayName,busyExecutors,offline,temporarilyOffline]")
            jobs_status, jobs_body = self._request("api/json?tree=jobs[name,builds[building]]")
            if computer_status != 200 or jobs_status != 200:
                raise JenkinsError("control_plane_drain_unproven", "Jenkins drain observations were incomplete", retryable=True)
            computers = json.loads(computer_body or b"{}").get("computer", [])
            jobs = json.loads(jobs_body or b"{}").get("jobs", [])
            node = next((c for c in computers if c.get("displayName") == self.spec.agent.get("name")), None)
            busy = sum(int(c.get("busyExecutors", 0) or 0) for c in computers)
            building = [j.get("name") for j in jobs if any(b.get("building") for b in j.get("builds", []))]
            if node is not None and busy == 0 and node.get("offline") is True:
                return {"status": "drained", "proof": {"busyExecutors": busy, "buildingJobs": building,
                    "agentOffline": True, "protectedHistoricalExecutions": [r["key"] for r in active]}}
            time.sleep(.25)
        raise JenkinsError("drain_timeout", "controller-only Jenkins state did not drain", retryable=True)

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
        host_state = self.paths.state / "deployment" / "host.json"
        host_record = None
        if operation.get("controlPlaneOnly") is True:
            try:
                host_record = json.loads(host_state.read_text(encoding="utf-8"))
            except (OSError, ValueError) as exc:
                raise JenkinsError("host_state_missing", "trusted lifecycle host state is unavailable") from exc
            self._control_plane_drain(operation, host_record)
        else:
            self.drain()
        pid = int(operation.get("pid", 0)); host = _LIVE_JOBS.get(pid)
        command = self.paths.state / "deployment" / "commands" / f"stop-{time.time_ns()}.json"
        command.parent.mkdir(parents=True, exist_ok=True)
        try:
            if host_record is None:
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

    def recover_activation(self, *, transition_digest: str) -> dict:
        if (not isinstance(transition_digest, str) or len(transition_digest) != 64
                or any(c not in "0123456789abcdef" for c in transition_digest)):
            raise JenkinsError("execution_recovery_identity", "Recovery requires the saved activation transition digest")
        from .execution_recovery import recover_execution_activation
        try:
            return recover_execution_activation(self, transition_digest=transition_digest)
        except (OSError, ValueError, KeyError, TypeError) as error:
            raise JenkinsError("execution_recovery_evidence_unavailable",
                               "Activation recovery evidence is unavailable", retryable=True) from error

    def reconcile(self) -> dict:
        operation = self._read() or {}
        observed = identity(int(operation["pid"])) if operation.get("pid") else None
        operation["observedAlive"] = bool(observed and (
            observed.get("creationTime", observed.get("birth")) == operation.get("creationTime", operation.get("birth"))
            and observed.get("executable") and operation.get("executable")
            and os.path.normcase(os.path.abspath(str(observed["executable"]))) ==
            os.path.normcase(os.path.abspath(str(operation["executable"])))))
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
        host_proof = operation.get("hostIdentityProof", {})
        owner_proven = bool(operation.get("observedAlive") and operation.get("operationId")
                            and operation.get("generation") and operation.get("hostPid")
                            and host_proof.get("operationMatches")
                            and host_proof.get("generationMatches")
                            and host_proof.get("hostPidMatches")
                            and host_proof.get("status") == "running")
        if not owner_proven and operation.get("state") in {"starting", "running", "stopping"}:
            operation["state"] = "unknown-owner"
        service_health = (operation["healthProof"].get("controller") and operation["healthProof"].get("plugins")
                          if operation.get("controlPlaneOnly") is True else operation["healthProof"].get("ready"))
        operation["serviceReady"] = bool(owner_proven and service_health)
        if operation["serviceReady"]:
            operation["state"] = "running"
        self._write(operation)
        return operation

    def bootstrap(self) -> dict:
        result = JenkinsBootstrap(self.spec, self.paths, self).apply()
        self._write({**(self._read() or {}), **result})
        return result

    def register_jobs(self) -> dict:
        return provision_jobs(self.spec, self.paths, self)
