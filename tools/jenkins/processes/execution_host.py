"""Persistent owner host for one state-backed execution.

The host is the only component allowed to launch a native execution.  It
loads its command and working directory from the durable ``recipe_plan`` and
``sealed_input`` records, registers the resulting NativeJob, and publishes a
terminal proof through ProcessRegistry.  Callers communicate through the
SQLite control record; a PID or a caller supplied command is never enough to
control an owner.
"""
from __future__ import annotations

import argparse
import os
import sys
import shutil
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Mapping

from ..contracts import JenkinsError, digest, file_digest
from ..state import State
from ..validation.receipts import TrustedReceiptAuthority, closure_manifest
from ..workflow.planning import get_recipe_plan
from ..resources.paths import canonical_build_root, physical_path_under
from .identity import ProcessIdentity, current_identity, identity_matches
from .job import NativeJob, TerminationProof
from .registry import ProcessRegistry


@dataclass(frozen=True, slots=True)
class HostIdentity:
    pid: int
    birth_token: str
    operation_id: str
    generation: str

    @classmethod
    def current(cls, operation_id: str, generation: str) -> "HostIdentity":
        ident = current_identity(os.getpid(), executable=sys.executable,
                                 command_line=tuple(sys.argv))
        return cls(ident.pid, ident.creation_time, operation_id, generation)

    def to_dict(self) -> dict[str, Any]:
        return {"hostPid": self.pid, "hostBirthToken": self.birth_token,
                "operationId": self.operation_id, "generation": self.generation}


class ExecutionHost:
    """Owner process for a single execution and its native Job object."""

    def __init__(self, state: State, execution_id: str, *, operation_id: str,
                 generation: str, poll_interval: float = .05) -> None:
        if not execution_id or not operation_id or not generation:
            raise JenkinsError("host_identity_missing", "execution, operation and generation are required")
        self.state = state
        self.execution_id = execution_id
        self.operation_id = operation_id
        self.generation = generation
        self.poll_interval = poll_interval
        self.host = HostIdentity.current(operation_id, generation)
        self.registry = ProcessRegistry(state)
        self.job: NativeJob | None = None
        self.native_job_id: str | None = None

    def _execution(self) -> dict[str, Any]:
        row = self.state.get("execution", self.execution_id)
        if row is None:
            raise JenkinsError("execution_not_found", "execution is not registered")
        return row

    def _trusted_recipe(self, payload: Mapping[str, Any]) -> tuple[dict[str, Any], dict[str, Any]]:
        ref = payload.get("recipeRef")
        if not ref:
            raise JenkinsError("recipe_plan_missing", "owner host requires a registered recipe plan")
        plan = get_recipe_plan(self.state, str(ref))
        recipe = plan.get("recipe") if isinstance(plan, Mapping) else None
        if not isinstance(recipe, Mapping):
            raise JenkinsError("recipe_untrusted", "owner host requires a registered recipe plan")
        plan_generation = plan.get("generation") if isinstance(plan, Mapping) else None
        execution_generation = payload.get("generation")
        if plan_generation is not None and execution_generation is not None and str(plan_generation) != str(execution_generation):
            raise JenkinsError("recipe_generation_mismatch", "execution generation does not match registered recipe plan")
        expected_source = plan.get("sourceRef") or plan.get("sealedInputRef") or plan.get("sourceInputDigest")
        supplied_source = payload.get("sealedInputRef") or payload.get("sourceRef") or payload.get("sourceInputDigest")
        if expected_source is not None and supplied_source is not None and str(expected_source) != str(supplied_source):
            raise JenkinsError("recipe_source_mismatch", "execution source does not match registered recipe plan")
        plan_driver = plan.get("driverDigest") or (recipe.get("driverDigest") if isinstance(recipe, Mapping) else None)
        supplied_driver = payload.get("driverDigest") or payload.get("driver")
        if plan_driver is not None and supplied_driver is not None:
            observed_driver = supplied_driver.get("digest") if isinstance(supplied_driver, Mapping) else supplied_driver
            if str(plan_driver) != str(observed_driver):
                raise JenkinsError("recipe_driver_mismatch", "execution driver does not match registered recipe plan")
        if recipe.get("callerCommand") is not None or recipe.get("callerCwd") is not None:
            raise JenkinsError("caller_command_forbidden", "caller command and cwd are forbidden")
        phase = str(payload.get("phase") or recipe.get("phase") or "compile")
        commands = recipe.get("commands")
        command = commands.get(phase) if isinstance(commands, Mapping) else None
        if command is None and isinstance(commands, (list, tuple)):
            phases = ("compile", "list", "test")
            command = commands[phases.index(phase)] if phase in phases and phases.index(phase) < len(commands) else None
        command = command or recipe.get("command")
        if not isinstance(command, (list, tuple)) or not command or any(not isinstance(x, str) for x in command):
            raise JenkinsError("recipe_command_invalid", "recipe plan contains no valid phase command")
        recipe = {**dict(recipe), "_selectedCommand": list(command), "_phase": phase}
        sealed_ref = payload.get("sealedInputRef") or payload.get("sourceInputDigest")
        sealed_row = self.state.get("sealed_input", str(sealed_ref)) if sealed_ref else None
        if sealed_row is None or sealed_row["payload"].get("status") not in ("sealed", "accepted"):
            raise JenkinsError("sealed_input_missing", "owner host requires a sealed input record")
        sealed = sealed_row["payload"]
        if str(recipe.get("kind", "")).startswith("cargo"):
            build_root = recipe.get("buildRoot") or sealed.get("buildRoot")
            approved = canonical_build_root(build_root)
            for field in ("preparationRoot", "materializedRoot", "outputRoot", "targetDir", "tempRoot", "cargoHome"):
                value = recipe.get(field) or sealed.get(field)
                if value:
                    physical_path_under(approved, value)
            for key in ("CARGO_TARGET_DIR", "CARGO_HOME", "CARGO_BUILD_BUILD_DIR", "SCCACHE_DIR", "TEMP", "TMP", "TMPDIR"):
                value = (recipe.get("executionEnvironment") or {}).get(key)
                if value:
                    physical_path_under(approved, value).mkdir(parents=True, exist_ok=True)
            phase_outputs = payload.get("phaseOutputRoot")
            if not phase_outputs:
                raise JenkinsError("phase_outputs_missing", "Cargo requires immutable per-phase output storage")
            physical_path_under(approved, phase_outputs)
        cwd = (recipe.get("preparationRoot") or recipe.get("materializedRoot") or
               sealed.get("preparationRoot") or sealed.get("materializedRoot"))
        if not cwd or not Path(str(cwd)).is_dir():
            raise JenkinsError("recipe_cwd_missing", "recipe must identify an existing materialized preparation root")
        if Path(str(cwd)).absolute() in {Path(str(sealed.get("objectRoot", ""))).absolute(), Path(str(sealed.get("repositoryRoot", ""))).absolute()}:
            raise JenkinsError("recipe_cwd_unsealed", "owner host cannot execute from object storage or live checkout")
        return recipe, sealed

    def _persist(self, payload: dict[str, Any], *, expected: int | None = None) -> dict[str, Any]:
        return self.state.put("execution_host", self.execution_id, payload, expected_version=expected)

    def start(self) -> dict[str, Any]:
        row = self._execution()
        payload = row["payload"]
        recipe, sealed = self._trusted_recipe(payload)
        if self.state.get("execution_host", self.execution_id):
            raise JenkinsError("host_already_started", "execution already has an owner host")
        command = tuple(recipe["_selectedCommand"])
        env = dict(os.environ)
        env.update({str(k): str(v) for k, v in (recipe.get("environment") or recipe.get("env") or {}).items()})
        cwd = str(recipe.get("preparationRoot") or recipe.get("materializedRoot") or
                   sealed.get("preparationRoot") or sealed.get("materializedRoot"))
        self.job = NativeJob.launch(command, cwd=cwd, env=env, text=True)
        record = self.registry.register(self.job, execution_id=self.execution_id, owner=self.execution_id)
        self.native_job_id = record.native_job_id
        host_payload = {**self.host.to_dict(), "status": "running", "executionId": self.execution_id,
                        "nativeJobId": record.native_job_id, "phase": recipe.get("_phase"), "commandDigest": digest(command),
                        "executable": self.job.identity.executable, "pid": self.job.identity.pid,
                        "processBirthToken": self.job.identity.creation_time, "cancelEpoch": 0}
        self._persist(host_payload)
        # The execution engine owns execution.status and final acceptance. The
        # host publishes only its native identity; this avoids CAS races between
        # host polling and the engine finalizer.
        self.state.put("execution", self.execution_id,
                       {**payload, "nativeJobId": record.native_job_id,
                        "ownerHost": host_payload}, expected_version=row["version"])
        return host_payload

    def _check_command(self, command: Mapping[str, Any]) -> dict[str, Any]:
        expected = self.state.get("execution_host", self.execution_id)
        if not expected:
            raise JenkinsError("host_not_found", "owner host is not registered")
        p = expected["payload"]
        fields = ("executionId", "operationId", "generation", "hostPid", "hostBirthToken", "cancelEpoch")
        if any(command.get(k) != p.get(k) for k in fields):
            raise JenkinsError("host_identity_mismatch", "control command does not match the current owner host")
        expected_identity = ProcessIdentity(int(p["hostPid"]), str(p["hostBirthToken"]))
        if not identity_matches(expected_identity):
            raise JenkinsError("host_identity_mismatch", "owner host PID no longer matches its recorded birth token")
        return p

    def cancel(self, command: Mapping[str, Any]) -> dict[str, Any]:
        host = self._check_command({**dict(command), "executionId": self.execution_id})
        row = self._execution()
        active = [r for r in self.state.list("execution_consumer")
                  if r["payload"].get("executionId") == self.execution_id
                  and r["payload"].get("status") == "active"]
        if active:
            raise JenkinsError("consumers_active", "owner host cannot terminate while consumers remain")
        epoch = int(host.get("cancelEpoch", 0)) + 1
        self.state.put("execution_control", self.execution_id,
                       {**dict(command), "kind": "cancel", "cancelEpoch": epoch,
                        "requestedAt": time.time()})
        return {"status": "cancel_requested", "cancelEpoch": epoch}

    def reconcile(self) -> dict[str, Any]:
        host_row = self.state.get("execution_host", self.execution_id)
        if not host_row:
            raise JenkinsError("host_not_found", "owner host is not registered")
        host = host_row["payload"]
        control = self.state.get("execution_control", self.execution_id)
        if control and control["payload"].get("kind") == "cancel" and self.job is not None:
            command = control["payload"]
            if any(command.get(k) != host.get(k) for k in ("executionId", "operationId", "generation", "hostPid", "hostBirthToken")):
                return {"status": "unknown", "nativeJobId": host.get("nativeJobId"), "reason": "owner_identity_mismatch", "debug": {k: [command.get(k), host.get(k)] for k in ("executionId", "operationId", "generation", "hostPid", "hostBirthToken")}}
            if int(command.get("cancelEpoch", -1)) != int(host.get("cancelEpoch", 0)) + 1:
                return {"status": "unknown", "nativeJobId": host.get("nativeJobId"), "reason": "cancel_epoch_mismatch"}
            if int(host["hostPid"]) != os.getpid() or str(host["hostBirthToken"]) != self.host.birth_token:
                return {"status": "unknown", "nativeJobId": host.get("nativeJobId"), "reason": "owner_identity_mismatch", "debug": {"recordedPid": host.get("hostPid"), "actualPid": os.getpid(), "recordedBirth": host.get("hostBirthToken"), "actualBirth": self.host.birth_token}}
            self.job.terminate(timeout_seconds=10)
        if self.job is None:
            # A restarted CLI cannot recreate a Windows Job handle. Preserve an
            # unknown state instead of guessing from PID or killing a reused PID.
            return {"status": "unknown", "nativeJobId": host.get("nativeJobId"), "reason": "owner_handle_unavailable"}
        if self.job.poll() is None:
            return {"status": "running", "nativeJobId": host.get("nativeJobId")}
        self.job.wait(timeout_seconds=10)
        terminal = self.registry.record_terminal(self.job)
        proof = terminal.payload.get("terminalProof")
        status = "cancelled" if control else "terminal"
        self.state.put("execution_host", self.execution_id,
                       {**host, "status": status, "terminalProof": proof,
                        "completeProof": bool(terminal.payload.get("completeProof"))}, expected_version=host_row["version"])
        return {"status": status, "nativeJobId": terminal.native_job_id, "terminalProof": proof}

    def serve(self, *, timeout: float | None = None) -> dict[str, Any]:
        started = time.monotonic()
        if self.job is None:
            self.start()
        while True:
            result = self.reconcile()
            if result["status"] in {"terminal", "cancelled", "unknown"}:
                return result
            if timeout is not None and time.monotonic() - started >= timeout:
                return result
            time.sleep(self.poll_interval)

    def _phase_names(self, recipe: Mapping[str, Any], phases: Any) -> list[str]:
        """Return the explicitly requested phases in deterministic order."""
        if phases is None:
            requested = recipe.get("phases")
            if requested is None:
                requested = ["compile"]
        else:
            requested = phases
        if isinstance(requested, str):
            requested = [requested]
        allowed = ("compile", "list", "test")
        result = [str(value) for value in requested]
        if not result or any(value not in allowed for value in result):
            raise JenkinsError("recipe_phase_invalid", "phase must be compile, list, or test")
        if len(set(result)) != len(result):
            raise JenkinsError("recipe_phase_duplicate", "recipe phases must be unique")
        return result

    def serve_phases(self, phases: Any = None) -> dict[str, Any]:
        """Run the sealed recipe phases under one owner host.

        Every phase gets a fresh native job and a durable record.  The host
        waits for process exit, stream EOF, and the native job terminal proof
        before starting the next phase.  This makes phase output and process
        identity recoverable after the caller has gone away.
        """
        execution = self._execution()
        recipe, sealed = self._trusted_recipe(execution["payload"])
        names = self._phase_names(recipe, phases)
        # Validate every requested command before creating the durable owner
        # record.  A malformed later phase must never strand a running host.
        commands_by_phase: dict[str, list[str]] = {}
        for phase in names:
            command = recipe.get("commands", {}).get(phase) if isinstance(recipe.get("commands"), Mapping) else None
            if command is None and (phase == recipe.get("phase") or phase == "compile"):
                command = recipe.get("command")
            if not isinstance(command, (list, tuple)) or not command or any(not isinstance(value, str) for value in command):
                raise JenkinsError("recipe_phase_command_missing", f"phase {phase} has no sealed command")
            commands_by_phase[phase] = list(command)
        existing = self.state.get("execution_host", self.execution_id)
        if existing:
            raise JenkinsError("host_already_started", "execution already has an owner host")
        phase_records: list[dict[str, Any]] = []
        log_root = Path(str(execution["payload"].get("phaseLogRoot") or
                            self.state.path.parent / "execution-logs" / self.execution_id))
        log_root.mkdir(parents=True, exist_ok=True)
        host_payload: dict[str, Any] = {**self.host.to_dict(), "status": "running",
                                        "executionId": self.execution_id, "phaseRecords": []}
        host_row = self._persist(host_payload)
        for phase in names:
            command = commands_by_phase[phase]
            phase_dir = log_root / phase
            try:
                phase_env = dict(os.environ)
                for source in (sealed.get("environment"), sealed.get("executionEnvironment"),
                               recipe.get("environment"), recipe.get("executionEnvironment"), recipe.get("env")):
                    if isinstance(source, Mapping):
                        phase_env.update({str(k): str(v) for k, v in source.items()})
                self.job = NativeJob.launch(tuple(command), cwd=str(recipe.get("preparationRoot") or
                                                      recipe.get("materializedRoot") or sealed.get("preparationRoot") or
                                                      sealed.get("materializedRoot")),
                                            env=phase_env,
                                            text=True, log_dir=phase_dir)
                record = self.registry.register(self.job, execution_id=self.execution_id, owner=self.execution_id)
                self.native_job_id = record.native_job_id
                # Register the immutable validation attempt only after the
                # native identity is durably recorded and before polling.  The
                # execution engine consumes this receipt; it cannot create a
                # retrospective attempt after a process exits.
                phase_stage = (recipe.get("phaseStages", {}) or {}).get(phase)
                if not phase_stage:
                    phase_stage = {"compile": "build", "list": "discovery", "test": "unit_test"}.get(phase, phase)
                required_test_stages = recipe.get("requiredTestStages") or recipe.get("requiredStages") or [phase_stage]
                if isinstance(required_test_stages, str):
                    required_test_stages = [required_test_stages]
                if phase == "test":
                    test_stages = [str(value) for value in required_test_stages if str(value) in {"unit_test", "integration_test", "regression_test"}]
                    if not test_stages:
                        raise JenkinsError("test_stage_contract_missing", "test phase requires requiredTestStages")
                elif phase == "compile":
                    test_stages = ["build"]
                else:
                    test_stages = []
                phase_execution_id = f"{self.execution_id}:{phase}"
                def _required_digest(value: Any, name: str) -> str:
                    text = value.get("digest") if isinstance(value, Mapping) else value
                    if not isinstance(text, str) or len(text) != 64 or any(ch not in "0123456789abcdefABCDEF" for ch in text):
                        if str(recipe.get("kind", "")).startswith("cargo"):
                            raise JenkinsError("digest_missing", f"{name} must be a 64-character digest")
                        return digest(text or name)
                    return text.lower()
                source_digest = _required_digest(execution["payload"].get("sourceDigest") or execution["payload"].get("sourceInputDigest") or sealed.get("sourceDigest"), "sourceDigest")
                recipe_digest = _required_digest(execution["payload"].get("recipeDigest") or execution["payload"].get("recipeRef"), "recipeDigest")
                coverage_digest = _required_digest(execution["payload"].get("coverageDigest"), "coverageDigest")
                driver_digest = _required_digest(execution["payload"].get("driverDigest") or recipe.get("driverDigest"), "driverDigest")
                phase_output_root = execution["payload"].get("phaseOutputRoot")
                if phase_output_root:
                    output_root = str(Path(str(phase_output_root)) / phase)
                else:
                    output_root = recipe.get("outputRoot") or recipe.get("targetDir") or sealed.get("outputRoot") or sealed.get("targetDir") or recipe.get("buildRoot") or sealed.get("buildRoot")
                if not output_root:
                    if str(recipe.get("kind", "")).startswith("cargo"):
                        raise JenkinsError("output_root_missing", "phase recipe must provide an external build output root")
                    output_root = str(phase_dir)
                output_root = str(output_root)
                if str(recipe.get("kind", "")).startswith("cargo"):
                    physical_path_under(canonical_build_root(recipe["buildRoot"]), output_root)
                    if Path(output_root).exists() and any(Path(output_root).iterdir()):
                        raise JenkinsError("phase_outputs_existing", "A new phase cannot replace existing output evidence")
                Path(output_root).mkdir(parents=True, exist_ok=True)
                required_outputs = ["stdout.log", "stderr.log"] if phase in {"compile", "test"} else []
                required_outputs = (recipe.get("requiredOutputsByPhase", {}) or {}).get(phase, required_outputs)
                if required_outputs is None:
                    required_outputs = (recipe.get("requiredOutputsByStage", {}) or {}).get(phase_stage, recipe.get("requiredOutputs", []))
                if not isinstance(required_outputs, list):
                    required_outputs = list(required_outputs or [])
                required_globs = recipe.get("requiredOutputGlobs") or {}
                if test_stages and required_globs and not required_outputs:
                    raise JenkinsError("required_outputs_contract_missing", "requiredOutputGlobs must resolve to non-empty required outputs")
                authority = TrustedReceiptAuthority(self.state)
                attempt_ids: list[tuple[str, str]] = []
                for stage in test_stages:
                    attempt_id = f"{self.execution_id}:{phase}" if stage == phase_stage and phase != "test" else f"{self.execution_id}:{phase}:{stage}"
                    authority.register_attempt({
                        "executionId": attempt_id, "stage": stage,
                        "sourceDigest": source_digest, "recipeDigest": recipe_digest,
                        "coverageDigest": coverage_digest, "driverDigest": driver_digest,
                        "nativeJobId": record.native_job_id, "birthToken": self.job.identity.creation_time,
                        "buildRoot": output_root, "outputRoot": output_root,
                        "requiredOutputs": required_outputs,
                        "requiredOutputGlobs": list(required_globs),
                        "phase": phase,
                    })
                    attempt_ids.append((stage, attempt_id))
                host_payload = {**host_payload, "currentPhase": phase,
                                "nativeJobId": record.native_job_id,
                                "commandDigest": digest(tuple(command)),
                                "pid": self.job.identity.pid,
                                "processBirthToken": self.job.identity.creation_time}
                host_row = self._persist(host_payload, expected=host_row["version"])
                cancelled = False
                while self.job.poll() is None:
                    control = self.state.get("execution_control", self.execution_id)
                    if control and control["payload"].get("kind") == "cancel":
                        active = [r for r in self.state.list("execution_consumer")
                                  if r["payload"].get("executionId") == self.execution_id
                                  and r["payload"].get("status") == "active"]
                        if not active:
                            self.job.terminate(timeout_seconds=10)
                            cancelled = True
                            break
                    time.sleep(self.poll_interval)
                self.job.wait(timeout_seconds=10, cleanup_descendants=True)
                terminal = self.registry.record_terminal(self.job)
                proof = terminal.payload.get("terminalProof") or {}
                phase_record = {"phase": phase, "stage": phase_stage, "nativeJobId": terminal.native_job_id,
                                "commandDigest": digest(tuple(command)), "terminalProof": proof,
                                "stdoutPath": str(phase_dir / "stdout.log"),
                                "stderrPath": str(phase_dir / "stderr.log"),
                                "validationExecutionId": phase_execution_id}
                phase_record["descendantCleanup"] = getattr(self.job, "descendant_cleanup", None)
                # Convert the collector proof to the authority's flat proof
                # shape and publish a content-addressed closure only after
                # process exit, stream EOF and Job terminal proof are known.
                trusted_proof = dict(proof)
                trusted_proof.update({"nativeJobId": terminal.native_job_id,
                                      "birthToken": self.job.identity.creation_time,
                                      "pid": self.job.identity.pid,
                                      "processExitCode": proof.get("processExitCode"),
                                      "stdoutEof": proof.get("stdoutEof"),
                                      "stderrEof": proof.get("stderrEof"),
                                      "childrenGone": proof.get("childrenGone")})
                if not cancelled:
                    if not bool(proof.get("complete")) or proof.get("processExitCode") != 0:
                        phase_records.append(phase_record)
                        failed_payload = {**host_payload, "phaseRecords": phase_records,
                                          "currentPhase": phase, "nativeJobId": terminal.native_job_id,
                                          "terminalProof": proof, "status": "failed", "failedPhase": phase,
                                          "error": {"type": "JenkinsError", "message": f"phase {phase} did not complete successfully"}}
                        self._persist(failed_payload)
                        raise JenkinsError("phase_failed", f"phase {phase} did not complete successfully")
                    phase_root = Path(output_root)
                    phase_root.mkdir(parents=True, exist_ok=True)
                    for stream_name in ("stdout", "stderr"):
                        src_log = phase_dir / f"{stream_name}.log"
                        dst_log = phase_root / f"{stream_name}.log"
                        if src_log.is_file():
                            dst_log.write_bytes(src_log.read_bytes())
                    # Capture selected Cargo binaries into an immutable
                    # per-phase closure before another phase can mutate target.
                    if phase in {"compile", "test"}:
                        target_dir = recipe.get("targetDir") or sealed.get("targetDir")
                        build_dir = (recipe.get("executionEnvironment") or {}).get("CARGO_BUILD_BUILD_DIR")
                        binary_roots = [Path(str(value)) for value in (target_dir, build_dir) if value]
                        if binary_roots:
                            binary_dir = phase_root / "binaries"
                            binary_dir.mkdir(parents=True, exist_ok=True)
                            package_names = []
                            for package in recipe.get("packages", []):
                                if isinstance(package, Mapping):
                                    package = package.get("name")
                                if package:
                                    package_names.append(str(package).replace("-", "_"))
                            candidates = []
                            for binary_root in binary_roots:
                                if str(recipe.get("kind", "")).startswith("cargo"):
                                    physical_path_under(canonical_build_root(recipe["buildRoot"]), binary_root)
                                candidates.extend(binary_root.rglob("*.exe"))
                            for candidate in candidates:
                                if package_names and not any(candidate.stem == name or candidate.stem.startswith(name + "-") for name in package_names):
                                    continue
                                if str(recipe.get("kind", "")).startswith("cargo"):
                                    physical_path_under(canonical_build_root(recipe["buildRoot"]), candidate, allow_missing=False)
                                destination = binary_dir / candidate.name
                                if destination.exists() and file_digest(destination) != file_digest(candidate):
                                    raise JenkinsError("binary_name_collision", "Different build outputs share a binary name")
                                if not destination.exists():
                                    shutil.copy2(candidate, destination)
                    closure = closure_manifest(output_root)
                    receipts = {}
                    for stage, attempt_id in attempt_ids:
                        producer_receipt = authority.record_terminal(attempt_id, trusted_proof, closure)
                        receipts[stage] = producer_receipt
                    if receipts:
                        phase_record["receipts"] = receipts
                        phase_record["receipt"] = next(iter(receipts.values()))
                        phase_record["stageReceipts"] = {stage: receipt["executionId"] for stage, receipt in receipts.items()}
                if recipe.get("kind") in {"cargo", "cargo-test"} and phase == "list":
                    output = Path(phase_record["stdoutPath"]).read_text(encoding="utf-8", errors="replace") if Path(phase_record["stdoutPath"]).is_file() else ""
                    discovered = [line for line in output.splitlines() if ": test" in line]
                    if not discovered:
                        raise JenkinsError("cargo_tests_not_discovered", "cargo --list produced no test discovery evidence")
                    names = [line.split(": test", 1)[0].strip() for line in discovered]
                    required_tests = list(recipe.get("requiredTests") or ())
                    if recipe.get("originalFailure"):
                        required_tests.append(recipe["originalFailure"])
                    missing_tests = [test for test in required_tests if not any(name == test or name.endswith("::" + test) for name in names)]
                    if missing_tests:
                        raise JenkinsError("required_test_missing", "Managed discovery did not cover required regression tests",
                                           details={"missing": missing_tests})
                    phase_record["discovery"] = {"testCount": len(discovered), "source": "cargo-test-list", "tests": names}
                phase_records.append(phase_record)
                stage_receipts = dict(host_payload.get("stageReceipts", {}))
                if phase_record.get("stageReceipts"):
                    stage_receipts.update(phase_record["stageReceipts"])
                elif phase_record.get("receipt"):
                    stage_receipts[phase_stage] = phase_record["receipt"].get("executionId")
                host_payload = {**host_payload, "phaseRecords": phase_records,
                                "stageReceipts": stage_receipts,
                                "currentPhase": phase, "nativeJobId": terminal.native_job_id,
                                "terminalProof": proof}
                host_row = self._persist(host_payload, expected=host_row["version"])
                if cancelled:
                    host_payload = {**host_payload, "status": "cancelled", "phaseRecords": phase_records}
                    self._persist(host_payload, expected=host_row["version"])
                    return {"status": "cancelled", "nativeJobId": terminal.native_job_id,
                            "terminalProof": proof, "phaseRecords": phase_records}
                if not bool(terminal.payload.get("completeProof")) or proof.get("processExitCode") != 0:
                    raise JenkinsError("phase_failed", f"phase {phase} did not complete successfully")
            except Exception as exc:
                # Never close a live native job on an error path.  Terminate,
                # wait for stream EOF and Job completion, then persist the
                # collector proof before releasing handles.
                if self.job is not None:
                    try:
                        if not self.job.proof().complete:
                            self.job.terminate(timeout_seconds=10)
                        self.job.wait(timeout_seconds=10)
                        terminal_failure = self.registry.record_terminal(self.job)
                        if not any(p.get("nativeJobId") == terminal_failure.native_job_id for p in phase_records):
                            phase_records.append({"phase": phase, "nativeJobId": terminal_failure.native_job_id,
                                                  "commandDigest": digest(tuple(command)),
                                                  "terminalProof": terminal_failure.payload.get("terminalProof")})
                        host_payload = {**host_payload, "nativeJobId": terminal_failure.native_job_id,
                                        "phaseRecords": phase_records,
                                        "terminalProof": terminal_failure.payload.get("terminalProof"),
                                        "failureCompleteProof": terminal_failure.payload.get("completeProof")}
                    except Exception as proof_error:
                        host_payload = {**host_payload, "failureProofError": str(proof_error)}
                failed = {**host_payload, "status": "failed", "failedPhase": phase,
                          "error": {"type": type(exc).__name__, "message": str(exc)}}
                self._persist(failed)
                raise
            finally:
                # wait() drains the streams; close() releases both the pipe
                # handles and the Windows Job handle before the next phase.
                if self.job is not None:
                    self.job.close()
                    self.job = None
        host_payload = {**host_payload, "status": "terminal", "executionId": self.execution_id,
                        "phaseRecords": phase_records,
                        "nativeJobId": phase_records[-1]["nativeJobId"],
                        "terminalProof": phase_records[-1]["terminalProof"]}
        self._persist(host_payload)
        return {"status": "terminal", "nativeJobId": host_payload["nativeJobId"],
                "terminalProof": host_payload["terminalProof"], "phaseRecords": phase_records}


def request_cancel(state: State, execution_id: str, *, operation_id: str, generation: str,
                   host_pid: int, host_birth_token: str, cancel_epoch: int) -> dict[str, Any]:
    """Issue an identity-bound cancellation request for a zero-ref execution."""
    host = state.get("execution_host", execution_id)
    if not host:
        raise JenkinsError("host_not_found", "owner host is not registered")
    payload = host["payload"]
    command = {"executionId": execution_id, "operationId": operation_id, "generation": generation, "hostPid": host_pid,
               "hostBirthToken": host_birth_token, "cancelEpoch": cancel_epoch}
    if any(command[k] != payload.get(k) for k in command):
        raise JenkinsError("host_identity_mismatch", "stale owner host identity")
    with state.transaction() as connection:
        fresh_host = state.get("execution_host", execution_id, connection=connection)
        if not fresh_host or any(command[k] != fresh_host["payload"].get(k) for k in command):
            raise JenkinsError("host_identity_mismatch", "owner host changed before cancellation")
        active = [r for r in state.list("execution_consumer", connection=connection)
                  if r["payload"].get("executionId") == execution_id and r["payload"].get("status") == "active"]
        if active:
            raise JenkinsError("consumers_active", "consumer references must be released first")
        control = state.get("execution_control", execution_id, connection=connection)
        if control and control["payload"].get("cancelEpoch", -1) >= cancel_epoch + 1:
            raise JenkinsError("cancel_epoch_stale", "cancel epoch has already been consumed")
        return state.put("execution_control", execution_id,
                         {**command, "kind": "cancel", "cancelEpoch": cancel_epoch + 1,
                          "requestedAt": time.time()},
                         expected_version=control["version"] if control else None, connection=connection)


def _main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--state", required=True)
    parser.add_argument("--execution", required=True)
    parser.add_argument("--operation", required=True)
    parser.add_argument("--generation", required=True)
    args = parser.parse_args(argv)
    host = ExecutionHost(State(args.state), args.execution, operation_id=args.operation, generation=args.generation)
    # The external owner must remain alive until a terminal native proof is
    # recorded.  Timeout is intentionally a direct API/testing concern only;
    # accepting it on the process CLI would let an exit orphan a live Job.
    execution = host.state.get("execution", args.execution)
    recipe_ref = execution["payload"].get("recipeRef") if execution else None
    recipe_row = host.state.get("recipe_plan", str(recipe_ref)) if recipe_ref else None
    recipe = recipe_row["payload"].get("recipe") if recipe_row else None
    result = host.serve_phases() if isinstance(recipe, Mapping) and recipe.get("phases") else host.serve()
    print(__import__("json").dumps(result, sort_keys=True))
    return 0 if result.get("status") in {"terminal", "cancelled"} else 2


__all__ = ["ExecutionHost", "HostIdentity", "request_cancel"]

if __name__ == "__main__":
    raise SystemExit(_main())
