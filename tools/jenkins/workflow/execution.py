"""Durable shared executions backed by trusted plans and native evidence."""
from __future__ import annotations

import time
from dataclasses import dataclass
from pathlib import Path
from typing import Mapping

from ..contracts import JenkinsError, digest, require_digest
from ..resources import Capacity, ResourceManager, canonical_build_root, physical_path_under
from ..state.locks import process_lock
from ..validation import TrustedReceiptAuthority
from ..validation.receipts import validate_producer_receipt
from .planning import get_recipe_plan

TERMINAL = {"passed", "failed", "cancelled"}


@dataclass(frozen=True)
class ExecutionKey:
    source: str
    preparation: str
    recipe: str
    driver: str

    @property
    def digest(self):
        return digest(self.__dict__)


class ExecutionEngine:
    def __init__(self, state, *, repo_root, build_root, resource_manager=None, receipt_authority=None):
        self.state = state
        self.repo_root = Path(repo_root).absolute()
        self.approved_build_root = canonical_build_root(build_root)
        self.build_root = physical_path_under(self.approved_build_root,
                                             self.approved_build_root.namespace())
        policy = state.get("resource_policy", "default")
        if not policy:
            raise JenkinsError("resource_policy_missing", "A trusted resource policy is required")
        self.policy = dict(policy["payload"])
        try:
            total = Capacity(*(int(self.policy[k]) for k in ("cpuBudget", "memoryBudget", "diskBudget")))
        except (KeyError, ValueError, TypeError) as exc:
            raise JenkinsError("resource_policy_invalid", "Trusted resource budgets are incomplete") from exc
        if min(total.cpu, total.memory_bytes, total.disk_bytes) <= 0:
            raise JenkinsError("resource_policy_invalid", "Resource budgets must be positive")
        self.resources = resource_manager or ResourceManager(state, total)
        self.receipts = receipt_authority or TrustedReceiptAuthority(state, repo_root=self.repo_root)

    def _lock(self, eid):
        return process_lock(self.state.path.parent / "execution-locks" / (digest(str(eid)) + ".lock"),
                            reason="execution_busy")

    def _plan(self, recipe):
        ref = recipe.get("recipeRef") if isinstance(recipe, Mapping) else recipe
        require_digest(ref, "recipeRef")
        return get_recipe_plan(self.state, ref)

    def _driver(self, plan, supplied):
        expected = plan.get("driver")
        value = supplied if supplied is not None else expected
        if not isinstance(value, Mapping) or not isinstance(expected, Mapping):
            raise JenkinsError("driver_binding_missing", "A sealed driver binding is required")
        value = dict(value)
        ref = require_digest(value.get("digest"), "driverDigest")
        if value != dict(expected):
            raise JenkinsError("driver_identity_mismatch", "Driver differs from the immutable recipe plan")
        if str(value.get("generation")) != str(plan.get("generation")):
            raise JenkinsError("driver_generation_mismatch", "Request generation differs from its plan")
        if not isinstance(value.get("runtimeOperationId"), str) or not value["runtimeOperationId"]:
            raise JenkinsError("runtime_binding_missing", "The driver must bind the live runtime operation")
        return ref, value

    def _consumer_contract(self, consumer, source, plan, driver):
        flow = self.state.get("workflow", consumer)
        if not flow:
            raise JenkinsError("consumer_workflow_missing", "An execution consumer requires a registered workflow")
        payload = flow["payload"]
        for field, expected in (("sourceDigest", source), ("coverageDigest", plan["coverageRef"]),
                                ("driverDigest", driver)):
            if payload.get(field) != expected:
                raise JenkinsError("consumer_identity_mismatch", f"Consumer {field} differs from its recipe")
        if str(payload.get("generation")) != str(plan["generation"]):
            raise JenkinsError("consumer_generation_mismatch", "Consumer generation differs from its recipe")
        if canonical_build_root(payload.get("buildRoot")).path != self.approved_build_root.path:
            raise JenkinsError("execution_root_mismatch", "Consumer build root differs from its execution")

    def claim(self, *, source, recipe, driver=None, consumer_id, **unused):
        if not isinstance(consumer_id, str) or not consumer_id:
            raise JenkinsError("consumer_missing", "consumerId is required")
        plan = self._plan(recipe)
        src = source if isinstance(source, str) else (source or {}).get("sealedInputRef")
        require_digest(src, "sealedInputRef")
        if src != plan.get("sourceRef"):
            raise JenkinsError("execution_source_mismatch", "The plan must refer to the same sealed source")
        sealed = self.state.get("sealed_input", src)
        if not sealed or sealed["payload"].get("status") not in {"sealed", "accepted"}:
            raise JenkinsError("sealed_input_missing", "A sealed input is required")
        driver_ref, driver_value = self._driver(plan, driver)
        rp = plan["recipe"]
        if canonical_build_root(rp.get("buildRoot")).path != self.approved_build_root.path:
            raise JenkinsError("execution_root_mismatch", "The recipe cannot select a different build root")
        self._consumer_contract(consumer_id, src, plan, driver_ref)
        prep = require_digest(rp.get("preparationKey"), "PreparationKey")
        root = physical_path_under(self.approved_build_root, rp["preparationRoot"])
        expected_root = self.build_root / "preparations" / prep / "inputs" / "source"
        if root != expected_root:
            raise JenkinsError("preparation_root_mismatch", "The preparation path must derive from its trusted key")
        estimates = rp.get("resourceEstimates")
        if not isinstance(estimates, Mapping):
            raise JenkinsError("resource_estimate_missing", "The trusted recipe must provide resource estimates")
        try:
            amount = Capacity(int(estimates["cpu"]), int(estimates["memoryBytes"]), int(estimates["diskBytes"]))
        except (KeyError, ValueError, TypeError) as exc:
            raise JenkinsError("resource_estimate_invalid", "Recipe resource estimates are incomplete") from exc
        if min(amount.cpu, amount.memory_bytes, amount.disk_bytes) <= 0:
            raise JenkinsError("resource_estimate_invalid", "Heavy work requires positive resource estimates")
        key = ExecutionKey(src, prep, plan["recipeDigest"], driver_ref)
        # A consumer is idempotent. Concurrent consumers share active work; a
        # later consumer receives a new test attempt by default.
        binding_key = digest({"consumer": consumer_id, "executionKey": key.digest})
        with self.state.transaction() as conn:
            bound = self.state.get("execution_binding", binding_key, connection=conn)
            index = self.state.get("execution_identity", key.digest, connection=conn)
            previous = self.state.get("execution", index["payload"]["executionId"], connection=conn) if index else None
            if bound:
                eid = bound["payload"]["executionId"]
                old = self.state.get("execution", eid, connection=conn)
                if not old:
                    raise JenkinsError("execution_binding_corrupt", "The consumer execution record is missing")
                return {"executionId": eid, **old["payload"]}
            if previous and previous["payload"].get("status") not in TERMINAL:
                old, eid = previous, previous["key"]
                ordinal = index["payload"]["ordinal"]
            else:
                ordinal = int(index["payload"].get("ordinal", 0)) + 1 if index else 1
                eid = digest({"identity": key.digest, "ordinal": ordinal})[:32]
                old = None
            if old:
                cur = dict(old["payload"])
                if cur.get("cancelRequested"):
                    raise JenkinsError("execution_cancelling", "The shared execution is being cancelled", retryable=True)
                users = list(cur.get("consumers", []))
                if consumer_id not in users:
                    users.append(consumer_id)
                cur.update(consumers=users, refs=len(users))
                self.state.put("execution", eid, cur, expected_version=old["version"], connection=conn)
            else:
                cur = {"executionKey": key.__dict__, "executionKeyDigest": key.digest,
                       "ordinal": ordinal, "preparationKey": prep, "status": "claimed",
                       "recipeRef": plan["recipeDigest"], "recipeDigest": plan["recipeDigest"],
                       "sealedInputRef": src, "sourceDigest": src, "coverageDigest": plan["coverageRef"],
                       "preparationRoot": str(root), "driver": driver_value, "driverDigest": driver_ref,
                       "generation": str(plan["generation"]), "operationId": driver_value["runtimeOperationId"],
                       "resource": amount.__dict__, "buildRoot": str(self.approved_build_root.path),
                       "consumers": [consumer_id], "refs": 1, "createdAt": time.time(),
                       "phaseOutputRoot": str(self.build_root / "receipts" / eid),
                       "phaseLogRoot": str(self.state.path.parent / "execution-logs" / eid)}
                self.state.put("execution", eid, cur, connection=conn)
                self.state.put("execution_identity", key.digest, {"executionId": eid, "ordinal": ordinal},
                               expected_version=index["version"] if index else None, connection=conn)
            self.state.put("execution_binding", binding_key, {"executionId": eid, "consumerId": consumer_id}, connection=conn)
            self.state.put("execution_consumer", f"{eid}:{consumer_id}",
                           {"executionId": eid, "consumerId": consumer_id, "status": "active"}, connection=conn)
        return self._view(eid)

    def _view(self, eid):
        row = self.state.get("execution", eid)
        if not row:
            raise JenkinsError("execution_not_found", "The execution is not registered")
        value = {"executionId": eid, **row["payload"]}
        for domain, field in (("execution_host", "host"), ("execution_launch", "launch")):
            item = self.state.get(domain, eid)
            if item:
                value[field] = item["payload"]
        return value

    def _admit(self, eid, payload):
        amount = Capacity(**payload["resource"])
        self.resources.queue(eid, request_id=eid, amount=amount)
        _record, generation = self.resources.scan_inventory(str(self.approved_build_root.path))
        result = self.resources.admit(eid, amount, inventory_generation=generation,
                                     writer_key=payload["preparationKey"], request_id=eid,
                                     build_root=str(self.approved_build_root.path), heavy_writer=True,
                                     max_age_seconds=float(self.policy.get("inventoryMaxAgeSeconds", 30)))
        return result, generation

    def run(self, eid, *, source=None):
        with self._lock(eid):
            value = self._view(eid)
            if value["status"] in TERMINAL or value.get("cancelRequested"):
                return value
            if self.state.get("execution_host", eid) or self.state.get("execution_launch", eid):
                return self._finish(eid)
            row = self.state.get("execution", eid)
            payload = dict(row["payload"])
            try:
                reservation, generation = self._admit(eid, payload)
            except JenkinsError as error:
                if not error.retryable:
                    raise
                self.state.put("execution", eid, {**payload, "status": "pending", "pendingReason": error.code},
                               expected_version=row["version"])
                return self._view(eid)
            if reservation is None:
                self.state.put("execution", eid, {**payload, "status": "pending", "pendingReason": "capacity",
                                                 "inventoryGeneration": generation}, expected_version=row["version"])
                return self._view(eid)
            payload.update(status="preparing", reservationKey=reservation["key"], inventoryGeneration=generation)
            self.state.put("execution", eid, payload, expected_version=row["version"])
            try:
                from .pools import materialize_preparation
                materialized = materialize_preparation(self.state, self.approved_build_root, eid)
            except Exception as error:
                fresh = self.state.get("execution", eid)
                self.state.put("execution", eid, {**fresh["payload"], "status": "failed", "failedBeforeLaunch": True,
                                                 "reasonCode": getattr(error, "code", "preparation_failed")},
                               expected_version=fresh["version"])
                self.resources.release_unlaunched(reservation["key"], writer_key=payload["preparationKey"], execution_id=eid)
                raise
            fresh = self.state.get("execution", eid)
            payload = {**fresh["payload"], "status": "running", "startedAt": time.time(), "materialization": materialized}
            launch = {"executionId": eid, "status": "requested", "operationId": payload["operationId"],
                      "generation": payload["generation"], "driverDigest": payload["driverDigest"],
                      "recipeRef": payload["recipeRef"], "sourceDigest": payload["sourceDigest"]}
            with self.state.transaction() as conn:
                self.state.put("execution", eid, payload, expected_version=fresh["version"], connection=conn)
                self.state.put("execution_launch", eid, launch, expected_version=0, connection=conn)
            return self._view(eid)

    def _phase_proofs(self, eid, host):
        records = host.get("phaseRecords", [])
        if not records:
            raise JenkinsError("native_termination_unproven", "No native phase termination records exist", retryable=True)
        for phase in records:
            native = self.state.get("native_job", phase.get("nativeJobId"))
            payload = native["payload"] if native else {}
            proof = payload.get("completeProof")
            if (payload.get("status") != "terminal" or payload.get("owner") != eid
                    or payload.get("executionId") != eid or not isinstance(proof, dict) or proof.get("complete") is not True):
                raise JenkinsError("native_termination_unproven", "Every phase requires its own complete native proof", retryable=True)
            if digest(proof) != digest(phase.get("terminalProof")):
                raise JenkinsError("native_proof_mismatch", "Host phase evidence differs from its native collector")
        for row in self.state.list("native_job"):
            payload = row["payload"]
            if (payload.get("executionId") == eid and payload.get("role") == "execution_host_guard"
                    and payload.get("status") != "terminal"):
                raise JenkinsError("execution_guard_pending", "The enclosing execution Job is still being collected", retryable=True)
            if payload.get("executionId") == eid and payload.get("status") != "terminal":
                raise JenkinsError("native_termination_unproven", "An execution native job remains active", retryable=True)
        return records

    def _finish(self, eid):
        row = self.state.get("execution", eid)
        payload = dict(row["payload"])
        if payload["status"] in TERMINAL:
            if payload.get("abortProof") and payload.get("reservationKey"):
                from ..processes.guards import abort_evidence
                abort = abort_evidence(self.state, eid)
                if not abort:
                    raise JenkinsError("execution_abort_unproven", "An aborted execution lost its termination evidence")
                self.resources.release(payload["reservationKey"], writer_key=payload["preparationKey"],
                                       native_proof_ref=abort.get("guardNativeJobId"), execution_id=eid)
            if payload.get("reservationKey") and payload.get("phaseRecords"):
                records = self._phase_proofs(eid, {"phaseRecords": payload["phaseRecords"]})
                self.resources.release(payload["reservationKey"], writer_key=payload["preparationKey"],
                                       native_proof_ref=records[-1]["nativeJobId"], execution_id=eid)
            return self._view(eid)
        from ..processes.guards import abort_evidence
        abort = abort_evidence(self.state, eid)
        if abort:
            payload.update(status="failed", abortProof=abort, finishedAt=time.time())
            self.state.put("execution", eid, payload, expected_version=row["version"])
            if payload.get("reservationKey"):
                self.resources.release(payload["reservationKey"], writer_key=payload["preparationKey"],
                                       native_proof_ref=abort.get("guardNativeJobId"), execution_id=eid)
            return self._view(eid)
        host_row = self.state.get("execution_host", eid)
        if not host_row or host_row["payload"].get("status") not in {"terminal", "failed", "cancelled"}:
            return self._view(eid)
        host = host_row["payload"]
        try:
            records = self._phase_proofs(eid, host)
        except JenkinsError as error:
            self.state.put("execution", eid, {**payload, "status": "reconciling" if error.code == "execution_guard_pending" else "blocked", "pendingReason": error.code},
                           expected_version=row["version"])
            return self._view(eid)
        status = "cancelled" if host["status"] == "cancelled" else "failed"
        stage_receipts = dict(host.get("stageReceipts", {}))
        if host["status"] == "terminal":
            plan = get_recipe_plan(self.state, payload["recipeRef"])
            recipe = plan["recipe"]
            if [r.get("phase") for r in records] != recipe.get("phases"):
                raise JenkinsError("phase_contract_mismatch", "The native phase sequence differs from its plan")
            for stage in ["build", *recipe["requiredTestStages"]]:
                ref = stage_receipts.get(stage)
                receipt = self.receipts.get(ref) if ref else None
                if receipt is None:
                    raise JenkinsError("stage_receipt_missing", "A required native stage lacks its registered receipt", retryable=True)
                validate_producer_receipt(receipt, output_root=receipt["outputClosure"]["root"],
                    expected={"sourceDigest": payload["sourceDigest"],
                    "coverageDigest": payload["coverageDigest"], "recipeDigest": payload["recipeRef"],
                     "driverDigest": payload["driverDigest"], "stage": stage, "executionId": eid})
            discovery = next((p.get("discovery") for p in records if p.get("phase") == "list"), None)
            if not discovery or int(discovery.get("testCount", 0)) <= 0:
                raise JenkinsError("cargo_tests_not_discovered", "Successful Cargo validation requires real test discovery")
            status = "validated"
        payload.update(status=status, phaseRecords=records, stageReceipts=stage_receipts,
                       terminalProof=records[-1]["terminalProof"], finishedAt=time.time())
        self.state.put("execution", eid, payload, expected_version=row["version"])
        if status == "validated":
            from ..artifacts import publish_execution_outputs, attach_execution_outputs
            published = publish_execution_outputs(self.state, self.approved_build_root, eid,
                                                  payload["executionKey"], stage_receipts)
            for consumer in payload.get("consumers", []):
                attach_execution_outputs(self.state, self.approved_build_root, eid, consumer)
            fresh = self.state.get("execution", eid)
            payload = {**fresh["payload"], "status": "passed", "artifacts": published}
            self.state.put("execution", eid, payload, expected_version=fresh["version"])
        if payload.get("reservationKey"):
            self.resources.release(payload["reservationKey"], writer_key=payload["preparationKey"],
                                   native_proof_ref=records[-1]["nativeJobId"], execution_id=eid)
        return self._view(eid)

    def observe(self, eid):
        try:
            with self._lock(eid):
                return self._finish(eid)
        except JenkinsError as error:
            if error.code == "execution_busy":
                return self._view(eid)
            raise

    def cancel(self, eid, consumer_id):
        with self._lock(eid):
            with self.state.transaction() as conn:
                ref = self.state.get("execution_consumer", f"{eid}:{consumer_id}", connection=conn)
                if not ref:
                    raise JenkinsError("consumer_not_found", "The consumer is not attached")
                self.state.put("execution_consumer", ref["key"], {**ref["payload"], "status": "released"},
                               expected_version=ref["version"], connection=conn)
                row = self.state.get("execution", eid, connection=conn)
                active = [r["payload"]["consumerId"] for r in self.state.list("execution_consumer", connection=conn)
                          if r["payload"].get("executionId") == eid and r["payload"].get("status") == "active"]
                payload = {**row["payload"], "consumers": active, "refs": len(active)}
                if not active and payload["status"] not in TERMINAL:
                    queued = self.state.get("resource_queue", f"{eid}:{eid}", connection=conn)
                    if queued and queued["payload"].get("status") == "waiting":
                        self.state.put("resource_queue", queued["key"], {**queued["payload"], "status": "cancelled"},
                                       expected_version=queued["version"], connection=conn)
                    launch = self.state.get("execution_launch", eid, connection=conn)
                    host = self.state.get("execution_host", eid, connection=conn)
                    if not host and (not launch or launch["payload"]["status"] == "requested"):
                        if launch:
                            self.state.put("execution_launch", eid, {**launch["payload"], "status": "cancelled",
                                           "cancelledBeforeClaim": True}, expected_version=launch["version"], connection=conn)
                        payload.update(status="cancelled", cancelRequested=True, cancelledBeforeLaunch=True)
                    else:
                        payload.update(status="cancel_requested", cancelRequested=True)
                        control = self.state.get("execution_control", eid, connection=conn)
                        self.state.put("execution_control", eid, {**(host["payload"] if host else {}),
                            "executionId": eid, "kind": "cancel", "requestedAt": time.time(),
                            "cancelEpoch": int(control["payload"].get("cancelEpoch", 0)) + 1 if control else 1},
                            expected_version=control["version"] if control else None, connection=conn)
                self.state.put("execution", eid, payload, expected_version=row["version"], connection=conn)
            if payload.get("cancelledBeforeLaunch") and payload.get("reservationKey"):
                self.resources.release_unlaunched(payload["reservationKey"], writer_key=payload["preparationKey"], execution_id=eid)
            if payload.get("artifacts"):
                from ..artifacts import release_execution_outputs
                release_execution_outputs(self.state, self.approved_build_root, eid, consumer_id)
            return self._view(eid)

    def reconcile(self, eid):
        value = self.observe(eid)
        if value.get("status") in {"claimed", "pending", "preparing"} and not value.get("cancelRequested"):
            return self.run(eid)
        return value


__all__ = ["ExecutionKey", "ExecutionEngine"]
