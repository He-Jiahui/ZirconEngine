from __future__ import annotations

"""Durable Jenkins lifecycle host.

This process owns the NativeJob handles.  It is a lifecycle supervisor only:
it accepts start/stop/reconcile commands from files and never schedules builds.
"""

import argparse
import json
import os
import subprocess
import time
import base64
import re
import urllib.request
import sys
from pathlib import Path

from .lifecycle import check_port_available, launch_command, spawn_owned, wait_for_health
from .paths import resolve_paths
from .spec import load_spec, JenkinsError
from .manager import DeploymentManager
from ..state import State
from tools.jenkins.processes.identity import current_identity, identity_from_popen, process_alive
from .driver import active_driver_binding, runtime_agent_name
from ..processes.job import NativeJob
from ..processes.registry import ProcessRegistry
from ..contracts import digest
from .quarantine import QuarantineBinding, classify_launch, matches_binding
from uuid import uuid4


def _atomic(path: Path, value: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_suffix(".tmp")
    tmp.write_text(json.dumps(value, sort_keys=True, indent=2), encoding="utf-8")
    tmp.replace(path)


class LifecycleHost:
    def __init__(self, spec_path: Path, java: Path | None = None, war: Path | None = None, operation_id: str | None = None):
        self.spec = load_spec(spec_path)
        self.paths = resolve_paths(self.spec)
        self.java = java or Path(self.spec.controller["java"]["executable"])
        self.war = war or Path(self.spec.controller["warPath"])
        self.root = self.paths.state / "deployment"
        self.state = self.root / "host.json"
        self.commands = self.root / "commands"
        self.job = None
        self.agent_job = None
        self.operation_id = operation_id or f"host-{time.time_ns()}"
        self.launch_state = self.paths.state / "coordination.sqlite3"
        self._launches: dict[str, tuple[NativeJob, ProcessRegistry]] = {}

    def _save(self, value: dict) -> None:
        _atomic(self.state, value)

    def _start(self) -> dict:
        if self.job is not None:
            return {"status": "running", "identity": self.job.identity.to_dict()}
        self.agent_launch_attempted = False
        # Fail before creating a new native job when Windows rejects the
        # configured endpoint (including an excluded port range).
        check_port_available(self.spec.controller["listenAddress"], int(self.spec.controller["httpPort"]))
        args, env = launch_command(self.paths, self.java, self.war,
                                   int(self.spec.controller["httpPort"]))
        self.job = spawn_owned(self.paths, args, env, component="controller")
        value = {"status": "starting", "identity": self.job.identity.to_dict(), "agentLaunchAttempted": False,
                 "hostPid": os.getpid(), "startedAt": time.time(), "operationId": self.operation_id,
                 "generation": self.operation_id}
        self._save(value)
        headers = self._auth_headers()
        if not wait_for_health(f"http://{self.spec.controller['listenAddress']}:{self.spec.controller['httpPort']}", 300,
                               headers=headers, process=self.job):
            self._save({**value, "status": "failed", "reasonCode": "health_timeout"})
            try:
                self.job.wait(timeout_seconds=30)
            except TimeoutError:
                # A controller that ignored its endpoint failure may still
                # own descendants.  Retain the NativeJob handle and request
                # its whole-tree termination before publishing any receipt.
                proof = self.job.terminate(timeout_seconds=30)
            else:
                proof = self.job.proof(requested=False)
            self._save({**value, "status": "failed", "nativeTerminationProof": proof.to_dict()})
            self.job.close(); self.job = None
            raise JenkinsError("controller_health_timeout", "controller did not become healthy")
        try:
            manager = DeploymentManager(self.spec, self.paths, self.java, self.war)
            manager.bootstrap()
            manager.register_jobs()
            if os.environ.get("ZIRCON_JENKINS_RECOVERY_CONTROL_ONLY") != "1":
                self._start_agent()
        except Exception as exc:
            # Startup is all-or-nothing.  Never leave a controller running when
            # bootstrap or the inbound-agent handshake fails; persist both
            # native proofs for offline maintenance before releasing handles.
            value["agentLaunchAttempted"] = self.agent_launch_attempted
            if self.agent_job is not None:
                value["agentIdentity"] = self.agent_job.identity.to_dict()
            agent_proof = None
            if self.agent_job is not None:
                try:
                    if self.agent_job.process.poll() is None:
                        agent_proof = self.agent_job.terminate(timeout_seconds=30).to_dict()
                    else:
                        self.agent_job.wait(timeout_seconds=30)
                        agent_proof = self.agent_job.proof(requested=False).to_dict()
                except Exception:
                    agent_proof = {"complete": False, "reasonCode": "agent_cleanup_failed"}
                try:
                    self.agent_job.close()
                finally:
                    self.agent_job = None
            controller_proof = None
            if self.job is not None:
                try:
                    if self.job.process.poll() is None:
                        controller_proof = self.job.terminate(timeout_seconds=30).to_dict()
                    else:
                        self.job.wait(timeout_seconds=30)
                        controller_proof = self.job.proof(requested=False).to_dict()
                except Exception:
                    controller_proof = {"complete": False, "reasonCode": "controller_cleanup_failed"}
                try:
                    self.job.close()
                finally:
                    self.job = None
            self._save({**value, "status": "failed", "reasonCode": getattr(exc, "code", "startup_failed"),
                        "nativeTerminationProof": controller_proof,
                        "agentTerminationProof": agent_proof})
            raise
        value["controllerIdentity"] = self.job.identity.to_dict()
        value["agentIdentity"] = self.agent_job.identity.to_dict() if self.agent_job else None
        value["agentLaunchAttempted"] = self.agent_launch_attempted
        value["controlPlaneOnly"] = os.environ.get("ZIRCON_JENKINS_RECOVERY_CONTROL_ONLY") == "1"
        # A control-plane recovery serves the UI/API while the agent and
        # execution broker remain stopped. Do not mark full runtime readiness.
        value["status"] = "running"
        value["healthProof"] = {"controller": True, "agent": self.agent_job is not None,
                                "ready": self.agent_job is not None}
        self._save(value)
        return value

    def _auth_headers(self) -> dict[str, str]:
        value = json.loads((self.paths.state / "credentials.json").read_text(encoding="utf-8"))
        token = value.get("apiToken") or value.get("password")
        encoded = base64.b64encode(f"{value['username']}:{token}".encode()).decode()
        return {"Authorization": "Basic " + encoded}

    def _start_agent(self) -> None:
        node = self.spec.agent
        binding = active_driver_binding(self.paths.repo)
        runtime_operation_id = str(binding.get("runtimeOperationId") or binding.get("generation") or "")
        if not runtime_operation_id:
            raise JenkinsError("runtime_binding_missing", "agent startup requires selected runtime operation")
        node = dict(node, name=runtime_agent_name(str(node["name"]), runtime_operation_id))
        jar = self.paths.cache / "remoting" / "agent.jar"
        jar.parent.mkdir(parents=True, exist_ok=True)
        req = urllib.request.Request(f"http://{self.spec.controller['listenAddress']}:{self.spec.controller['httpPort']}/jnlpJars/agent.jar", headers=self._auth_headers())
        with urllib.request.urlopen(req, timeout=20) as response:
            data = response.read()
        jar.write_bytes(data)
        jnlp = urllib.request.Request(f"http://{self.spec.controller['listenAddress']}:{self.spec.controller['httpPort']}/computer/{node['name']}/jenkins-agent.jnlp", headers=self._auth_headers())
        with urllib.request.urlopen(jnlp, timeout=20) as response:
            text = response.read().decode("utf-8", "replace")
        match = re.search(r"<argument>([0-9a-fA-F]{32,})</argument>", text)
        if not match:
            raise JenkinsError("agent_secret_missing", "Jenkins did not return an inbound agent secret")
        secret = self.paths.state / "deployment" / "agent.secret"
        secret.write_text(match.group(1), encoding="ascii")
        try:
            os.chmod(secret, 0o600)
        except OSError:
            pass
        work = Path(node["workDir"]); work.mkdir(parents=True, exist_ok=True)
        # Remoting accepts the secret as a value, not an @file reference. Use
        # the Java launcher argument-file feature so the secret never appears
        # in the process command line; the argument file is private and erased
        # after the child has parsed it.
        argfile = self.paths.state / "deployment" / "agent.args"
        argfile.write_text("\n".join(["-jar", str(jar), "-url",
            f"http://{self.spec.controller['listenAddress']}:{self.spec.controller['httpPort']}/",
            "-name", str(node["name"]), "-secret", match.group(1), "-webSocket", "-workDir", str(work)]) + "\n", encoding="utf-8")
        try:
            os.chmod(argfile, 0o600)
        except OSError:
            pass
        agent_tmp = self.paths.tmp / "agent"; agent_tmp.mkdir(parents=True, exist_ok=True)
        args = [str(self.java), f"-Djava.io.tmpdir={agent_tmp}", "@" + str(argfile)]
        env = dict(os.environ, TMP=str(agent_tmp), TEMP=str(agent_tmp))
        self.agent_launch_attempted = True
        self.agent_job = spawn_owned(self.paths, args, env, component="agent")
        # Keep the argument file until Jenkins reports this exact node online;
        # process survival alone does not prove that the secret was consumed.
        deadline = time.monotonic() + 300
        online = False
        while time.monotonic() < deadline:
            if self.agent_job.process.poll() is not None:
                raise JenkinsError("agent_start_failed", "remoting exited before handshake")
            try:
                req = urllib.request.Request(
                    f"http://{self.spec.controller['listenAddress']}:{self.spec.controller['httpPort']}/computer/{node['name']}/api/json",
                    headers=self._auth_headers())
                with urllib.request.urlopen(req, timeout=3) as response:
                    payload = json.loads(response.read().decode("utf-8"))
                online = not payload.get("offline", True) and not payload.get("temporarilyOffline", False)
                if online:
                    break
            except Exception:
                pass
            time.sleep(.5)
        if not online:
            try:
                argfile.unlink()
            except OSError:
                pass
            raise JenkinsError("agent_handshake_timeout", "Jenkins did not report the inbound agent online")
        try:
            argfile.unlink()
        except OSError:
            pass

    def _stop(self) -> dict:
        if self.job is None:
            current = json.loads(self.state.read_text(encoding="utf-8")) if self.state.exists() else {}
            if current.get("status") == "stopped":
                return current
            raise JenkinsError("native_handle_missing", "host has no live controller handle")
        controller_identity = self.job.identity.to_dict()
        agent_identity = self.agent_job.identity.to_dict() if self.agent_job is not None else None
        # Only carry lifecycle flags forward from a durable record when that
        # record is bound to this exact operation/generation.  A stale record
        # must never turn an unknown agent state into a false negative proof.
        durable = {}
        state_path = getattr(self, "state", None)
        if state_path is not None and state_path.exists():
            try:
                candidate = json.loads(state_path.read_text(encoding="utf-8"))
                if (candidate.get("operationId") == self.operation_id and
                        candidate.get("generation") == self.operation_id):
                    durable = candidate
            except (OSError, ValueError, TypeError):
                durable = {}
        attempted = getattr(self, "agent_launch_attempted", None)
        if self.agent_job is not None:
            attempted = True
        elif not isinstance(attempted, bool):
            recorded_attempted = durable.get("agentLaunchAttempted")
            attempted = recorded_attempted if isinstance(recorded_attempted, bool) else None
        control_env = os.environ.get("ZIRCON_JENKINS_RECOVERY_CONTROL_ONLY")
        if control_env == "1":
            control_plane_only = True
        elif isinstance(durable.get("controlPlaneOnly"), bool):
            control_plane_only = durable["controlPlaneOnly"]
        else:
            control_plane_only = None
        # A known controller-only launch is proof that no agent launch was
        # attempted.  For every other missing value retain explicit unknown.
        if attempted is None and control_plane_only is True:
            attempted = False
        agent_proof = None
        if self.agent_job is not None:
            if self.agent_job.process.poll() is None:
                agent_proof = self.agent_job.terminate(timeout_seconds=30).to_dict()
            else:
                self.agent_job.wait(timeout_seconds=30)
                agent_proof = self.agent_job.proof(requested=False).to_dict()
            self.agent_job.close(); self.agent_job = None
        if self.job.process.poll() is None:
            proof = self.job.terminate(timeout_seconds=30)
        else:
            self.job.wait(timeout_seconds=30)
            proof = self.job.proof(requested=False)
        result = {"status": "stopped", "operationId": self.operation_id,
                  "generation": self.operation_id, "identity": controller_identity,
                  "controllerIdentity": controller_identity, "agentIdentity": agent_identity,
                   "hostPid": os.getpid(), "agentLaunchAttempted": attempted,
                   "controlPlaneOnly": control_plane_only,
                  "nativeTerminationProof": proof.to_dict(), "agentTerminationProof": agent_proof, "stoppedAt": time.time()}
        self.job.close(); self.job = None; self._save(result)
        return result

    def _execution_broker_tick(self) -> None:
        """Claim and spawn durable execution launches outside Jenkins agents.

        ``execution_launch`` is the broker queue.  Claims are conditional on
        the record version and the selected execution bindings, so retries are
        idempotent and stale generations fail closed.
        """
        # A control-plane recovery deliberately preserves old launches and
        # reservations. It provides the UI/API without resuming compilation.
        if os.environ.get("ZIRCON_JENKINS_RECOVERY_CONTROL_ONLY") == "1":
            return
        if not self.launch_state.exists():
            return
        state = State(self.launch_state)
        manager = DeploymentManager(self.spec, self.paths, self.java, self.war)
        try:
            driver_env = manager._driver_env()
        except Exception:
            return
        launcher = driver_env.get("ZIRCON_DRIVER_LAUNCHER")
        python = driver_env.get("JENKINS_PYTHON")
        if not launcher or not python:
            return
        # Every launch is tied to the selected sealed driver and the runtime
        # operation recorded by deployment.  A lifecycle restart must fail
        # closed when that binding is unavailable or has changed.
        try:
            binding = active_driver_binding(self.paths.repo)
        except Exception:
            binding = None
        if binding is None or str(binding.get("launcher")) != str(Path(launcher).absolute()):
            return
        selected_digest = str(binding["digest"])
        selected_operation = str(binding["generation"])
        expected_python = (self.paths.repo / ".jenkins/runtime/python/python.exe").absolute()
        if Path(python).absolute() != expected_python or not expected_python.is_file():
            return
        for row in state.list("execution_launch"):
            payload = row["payload"]
            key = row["key"]
            status = payload.get("status")
            execution_id = str(payload.get("executionId", ""))
            execution = state.get("execution", execution_id)
            ep = execution["payload"] if execution else None
            attempt_generation = (str(ep.get("generation"))
                                  if isinstance(ep, dict) and ep.get("generation") is not None else "")
            quarantine_binding = (QuarantineBinding(selected_operation, attempt_generation, selected_digest)
                                  if attempt_generation else None)
            eligible = (quarantine_binding is not None
                        and classify_launch(payload, ep, quarantine_binding) == "eligible")
            if status != "requested":
                # Started/launching rows are never repaired from a partial
                # legacy binding.  Require the immutable payload, execution,
                # and sealed driver identities to agree exactly.
                eligible = bool(isinstance(ep, dict) and matches_binding(
                    payload, ep, selected_digest, selected_operation))
            if not eligible:
                continue
            if status == "requested":
                binding_ok = matches_binding(payload, ep, selected_digest, selected_operation)
                if not binding_ok:
                    continue
                claimed = {**payload, "status": "launching", "claimedAt": time.time(),
                           "brokerPid": os.getpid(), "brokerGeneration": self.operation_id,
                           "launchIntentDigest": __import__("hashlib").sha256(json.dumps(payload, sort_keys=True, separators=(",", ":")).encode()).hexdigest()}
                try:
                    claimed_row = state.put("execution_launch", key, claimed, expected_version=row["version"])
                except Exception:
                    continue
                args = [str(python), str(launcher), "__execution-host", "--state", str(self.launch_state),
                        "--execution", execution_id, "--operation", str(payload["operationId"]),
                        "--generation", str(payload["generation"])]
                log_dir = self.paths.logs / "executions"; log_dir.mkdir(parents=True, exist_ok=True)
                env = dict(os.environ, **driver_env, PYTHONPATH=str(self.paths.repo), JENKINS_HOME=str(self.paths.home))
                proc = None
                registry = ProcessRegistry(state)
                guard = None
                try:
                    proc = NativeJob.launch(args, cwd=self.paths.repo, env=env,
                                           log_dir=log_dir / execution_id, suspended=True,
                                           job_name="Local\\ZirconJenkinsExecution-" + uuid4().hex)
                    # args[0] is the sealed repository Python.  Recording
                    # sys.executable here would hide a wrong interpreter and
                    # make PID reuse checks ineffective.
                    ident = proc.identity
                    guard = registry.register(proc, execution_id=execution_id, owner=execution_id,
                                              role="execution_host_guard")
                    self._launches[key] = (proc, registry)
                    state.put("execution_launch", key, {**claimed, "status": "started",
                               "launcherIdentity": ident.to_dict(),
                               "launcherPid": ident.pid, "launcherBirthToken": ident.creation_time,
                               "launcherExecutable": ident.executable,
                               "nativeHostUUID": None, "guardNativeJobId": guard.native_job_id}, expected_version=claimed_row["version"])
                    proc.resume()
                except Exception as exc:
                    current = state.get("execution_launch", key)
                    failed = {**current["payload"], "status": "failed", "reasonCode": "spawn_failed", "details": str(exc)}
                    if proc is not None:
                        try:
                            proc.terminate(timeout_seconds=30)
                            if guard is None:
                                guard = registry.register(proc, execution_id=execution_id, owner=execution_id,
                                                          role="execution_host_guard")
                            terminal = registry.record_terminal(proc)
                            failed["guardNativeJobId"] = guard.native_job_id
                            if terminal.payload.get("completeProof"):
                                state.put("execution_abort", key, {"executionId": key,
                                    "guardNativeJobId": guard.native_job_id,
                                    "proofDigest": digest(terminal.payload["completeProof"]),
                                    "outcome": "failed", "artifactPublicationAllowed": False})
                            proc.close()
                            self._launches.pop(key, None)
                        except Exception as proof_error:
                            failed["terminationError"] = str(proof_error)
                    state.put("execution_launch", key, failed)
            elif status == "launching":
                # A controller/lifecycle crash after the CAS claim leaves an
                # unknown launch outcome.  Never submit it a second time.
                broker_pid = int(payload.get("brokerPid", 0) or 0)
                alive = process_alive(broker_pid) if broker_pid else False
                if alive is False:
                    state.put("execution_launch", key, {**payload, "status": "failed", "reasonCode": "launch_ambiguous"}, expected_version=row["version"])
            elif status == "started":
                host = state.get("execution_host", key)
                if host:
                    hp = host["payload"]
                    if (str(hp.get("operationId")) != str(payload.get("operationId"))
                            or str(hp.get("generation")) != str(payload.get("generation"))):
                        state.put("execution_launch", key, {**payload, "status": "failed", "reasonCode": "host_binding_mismatch"}, expected_version=row["version"])
                        continue
                    if hp.get("nativeJobId") and payload.get("nativeHostUUID") != hp.get("nativeJobId"):
                        state.put("execution_launch", key, {**payload, "nativeHostUUID": hp.get("nativeJobId"),
                                   "nativeHostIdentity": {"hostPid": hp.get("hostPid"), "hostBirthToken": hp.get("hostBirthToken")}}, expected_version=row["version"])
                        payload = {**payload, "nativeHostUUID": hp.get("nativeJobId")}
                handle = self._launches.get(key)
                if handle and handle[0].poll() is not None:
                    proc, registry = handle
                    exit_code = proc.poll()
                    try:
                        proc.wait(timeout_seconds=1)
                    except TimeoutError:
                        proc.terminate(timeout_seconds=30)
                    guard = registry.record_terminal(proc)
                    if not guard.payload.get("completeProof"):
                        continue
                    proc.close(); self._launches.pop(key, None)
                    failed_host = not host or host["payload"].get("status") not in {"terminal", "cancelled"}
                    if failed_host:
                        state.put("execution_abort", key, {"executionId": key,
                            "guardNativeJobId": guard.native_job_id,
                            "proofDigest": digest(guard.payload["completeProof"]),
                            "outcome": "failed", "artifactPublicationAllowed": False})
                    if not host:
                        state.put("execution_launch", key, {**payload, "status": "failed", "reasonCode": "launch_ambiguous"}, expected_version=row["version"])
                    else:
                        state.put("execution_launch", key, {**payload, "status": "exited", "exitCode": exit_code, "exitedAt": time.time()}, expected_version=row["version"])
                elif not handle:
                    # After a broker restart, an existing launcher can be
                    # observed by its durable PID/birth identity.  If it is
                    # gone without a host record, keep the outcome unknown.
                    lp = payload.get("launcherIdentity") or {}
                    alive = process_alive(int(lp.get("pid", 0) or 0)) if lp.get("pid") else False
                    if alive is False and not host:
                        state.put("execution_launch", key, {**payload, "status": "failed", "reasonCode": "launch_ambiguous"}, expected_version=row["version"])

    @staticmethod
    def _execution_launch_binding(payload: dict, execution: dict, selected_digest: str,
                                  selected_operation: str) -> bool:
        """Return true only for a launch bound to the current sealed runtime."""
        if not execution:
            return False
        driver = execution.get("driver") if isinstance(execution.get("driver"), dict) else {}
        return all((
            payload.get("operationId") == execution.get("operationId"),
            payload.get("generation") == execution.get("generation"),
            payload.get("driverDigest") == execution.get("driverDigest"),
            str(payload.get("operationId")) == str(selected_operation),
            str(driver.get("runtimeOperationId", payload.get("operationId"))) == str(selected_operation),
            str(driver.get("generation", payload.get("generation"))) == str(payload.get("generation")),
            str(payload.get("driverDigest")) == str(selected_digest),
        ))

    def run(self) -> None:
        self.root.mkdir(parents=True, exist_ok=True)
        self._start()
        seen: set[str] = set()
        while True:
            if self.job is None:
                return
            for request in sorted(self.commands.glob("*.json")):
                if request.name.endswith(".result.json"):
                    continue
                if request.name in seen:
                    continue
                seen.add(request.name)
                try:
                    command = json.loads(request.read_text(encoding="utf-8"))
                    current_identity = self.job.identity.to_dict() if self.job else {}
                    expected_identity = command.get("controllerIdentity") or {}
                    if (command.get("operationId") != self.operation_id or command.get("hostPid") != os.getpid()
                            or command.get("generation") != self.operation_id
                            or expected_identity.get("pid") not in (None, current_identity.get("pid"))
                            or expected_identity.get("creationTime") not in (None, current_identity.get("creationTime"))):
                        _atomic(request.with_suffix(".result.json"), {"status": "ignored", "reasonCode": "stale_host_generation"})
                        continue
                    action = command.get("action")
                    if action == "stop":
                        result = self._stop()
                    elif action == "reconcile":
                        result = {"status": "running", "identity": self.job.identity.to_dict()}
                    else:
                        raise JenkinsError("operation_unknown", "unknown lifecycle host action")
                    _atomic(request.with_suffix(".result.json"), result)
                    if self.job is None:
                        return
                except Exception as exc:
                    _atomic(request.with_suffix(".result.json"), {"status": "failed", "reasonCode": getattr(exc, "code", "host_error")})
            try:
                self._execution_broker_tick()
            except Exception as exc:
                # A failed queue record must not destroy the lifecycle owner
                # and its retained controller/agent/guard Job handles.
                try:
                    state = State(self.launch_state)
                    state.event("execution_broker_error", {
                        "operationId": self.operation_id,
                        "errorType": type(exc).__name__,
                        "reasonCode": getattr(exc, "code", "broker_error"),
                    })
                except Exception:
                    print("execution broker evidence unavailable; retained Jobs remain owned", flush=True)
            if self.job.process.poll() is not None:
                try:
                    self.job.wait(timeout_seconds=30, cleanup_descendants=True)
                except Exception:
                    try:
                        self.job.terminate(timeout_seconds=30)
                    except Exception:
                        pass
                proof = self.job.proof()
                agent_proof = None
                if self.agent_job is not None:
                    try:
                        self.agent_job.terminate(timeout_seconds=30)
                        self.agent_job.wait(timeout_seconds=30)
                        agent_proof = self.agent_job.proof(requested=False).to_dict()
                    except Exception:
                        agent_proof = {"complete": False, "reasonCode": "agent_proof_unavailable"}
                complete = proof.complete and (self.agent_job is None or bool(agent_proof and agent_proof.get("complete")))
                self._save({"status": "stopped" if complete else "termination-unproven", "operationId": self.operation_id, "generation": self.operation_id,
                            "identity": self.job.identity.to_dict(), "nativeTerminationProof": proof.to_dict(),
                            "agentTerminationProof": agent_proof, "hostPid": os.getpid()})
                if complete:
                    self.job.close()
                    if self.agent_job is not None:
                        self.agent_job.close()
                    return
            time.sleep(.2)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--spec", type=Path, required=True)
    parser.add_argument("--java", type=Path)
    parser.add_argument("--war", type=Path)
    parser.add_argument("--operation-id")
    args = parser.parse_args()
    LifecycleHost(args.spec, args.java, args.war, args.operation_id).run()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
