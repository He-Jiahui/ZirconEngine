"""Reconcile pinned interrupted Jenkins runs with an isolated controller only.

This deployment maintenance entry never starts an agent or execution broker,
edits application State, changes lifecycle pointers, or writes build outcomes.
Jenkins owns interruption and persistence; a retained Windows Job owns Java.
"""
from __future__ import annotations

import argparse
import base64
import hashlib
import json
import os
import pathlib
import sys
import time
import urllib.parse
import xml.etree.ElementTree as ET


def checksum(data):
    return hashlib.sha256(data).hexdigest()


def checked(reference):
    path = pathlib.Path(reference["path"])
    for part in (path, *path.parents):
        if part.exists() and (part.is_symlink() or getattr(part.lstat(), "st_file_attributes", 0) & 0x400):
            raise RuntimeError("queue_reconciliation_path_alias")
    data = path.read_bytes()
    if checksum(data) != reference["sha256"]:
        raise RuntimeError("queue_reconciliation_preimage_changed: " + str(path))
    return data


def xml(data):
    return ET.fromstring(data.replace(b'version="1.1"', b'version="1.0"'))


def script_text(plan, action):
    encoded = base64.b64encode(json.dumps({"runs": plan["runs"], "timer": plan["timer"],
        "queueId": plan["queueId"], "action": action, "forceKnownDeparted": plan.get("forceKnownDeparted", False)}, separators=(",", ":")).encode()).decode()
    return """
import jenkins.model.Jenkins
import hudson.model.ParametersAction
import hudson.model.Result
import hudson.triggers.TimerTrigger
import hudson.triggers.Trigger
import org.jenkinsci.plugins.workflow.job.WorkflowJob
import org.jenkinsci.plugins.workflow.support.steps.ExecutorStepExecution
def p = new groovy.json.JsonSlurper().parseText(new String('%s'.decodeBase64(), 'UTF-8'))
def j = Jenkins.get()
assert j.numExecutors == 0 : 'controller_has_executors'
assert j.quietingDown : 'controller_not_quiet'
assert j.nodes*.nodeName == ['zircon-windows-agent'] : 'fixed_node_inventory_changed'
assert j.computers.findAll { it.name != '' }.every { it.offline } : 'agent_is_online'
def timerJob = j.getItemByFullName(p.timer.job, WorkflowJob)
assert timerJob != null : 'timer_job_missing'
def timers = timerJob.triggers.values().findAll { it.class == TimerTrigger }
assert timers.size() == 1 && timers[0].spec == p.timer.spec : 'timer_identity_changed'
def jobField = Trigger.getDeclaredField('job')
jobField.setAccessible(true)
assert jobField.get(timers[0]) == null || jobField.get(timers[0]).is(timerJob) : 'timer_owner_changed'
jobField.set(timers[0], null) // transient field only; job configuration is never saved
def runs = p.runs.collect { expected ->
    def job = j.getItemByFullName(expected.job, WorkflowJob)
    assert job != null : 'original_job_missing'
    def run = job.getBuildByNumber(expected.number as int)
    assert run != null && run.queueId == (expected.queueId as long) &&
        run.timeInMillis == (expected.timestamp as long) : 'original_run_identity_changed'
    def actual = run.getAction(ParametersAction)?.parameters?.collectEntries {
        [(it.name): String.valueOf(it.value)]
    } ?: [:]
    assert actual == expected.parameters : 'original_parameters_changed'
    run
}
def knownIds = runs*.externalizableId as Set
def active = j.getAllItems(WorkflowJob).collectMany { job -> job.builds.findAll { it.building } }*.externalizableId
assert active.every { knownIds.contains(it) } : 'unreviewed_active_build'
def queued = j.queue.items.collect { item ->
    assert item.task instanceof ExecutorStepExecution.PlaceholderTask : 'unreviewed_queue_task'
    def owner = item.task.ownerExecutable
    assert owner != null && knownIds.contains(owner.externalizableId) : 'unreviewed_queue_owner'
    assert item.task.assignedLabel?.name == 'zircon-windows-agent' : 'queue_label_changed'
    [id:item.id, runId:owner.externalizableId, taskClass:item.task.class.name]
}
assert queued.findAll { it.id == (p.queueId as long) }.every {
    it.runId == p.timer.job + '#65'
} : 'original_queue_identity_changed'
def before = [active:active, queue:queued]
if (p.action == 'interrupt') {
    runs.each { run ->
        if (run.building) {
            def execution = run.execution
            assert execution != null : 'flow_execution_not_loaded'
            if (p.forceKnownDeparted) {
                // Native execution trees are independently proved terminal before launch.
                run.doKill()
            } else {
                execution.interrupt(Result.ABORTED)
            }
        }
    }
    // Cancel only observed placeholders belonging to the pinned original runs.
    queued.each { item ->
        def current = j.queue.getItem(item.id as long)
        if (current != null) {
            assert current.task.ownerExecutable?.externalizableId == item.runId : 'queue_owner_changed'
            j.queue.cancel(current)
        }
    }
}
def result = [quietingDown:j.quietingDown, controllerExecutors:j.numExecutors,
    agentOnline:false, timerPaused:(jobField.get(timers[0]) == null), before:before,
    runs:runs.collect { run -> [job:run.parent.fullName, number:run.number, queueId:run.queueId,
        timestamp:run.timeInMillis, building:run.building, result:run.result?.toString(),
        executionPresent:(run.execution != null), flowComplete:run.execution?.complete] },
    remainingQueue:j.queue.items.collect { [id:it.id, taskClass:it.task.class.name] }]
if (p.action == 'observe' && result.remainingQueue.isEmpty() &&
        result.runs.every { !it.building && (it.flowComplete || (p.forceKnownDeparted && !it.executionPresent && it.result == "ABORTED")) }) {
    j.queue.save()
}
println('ZR_QUEUE_RECONCILE=' + groovy.json.JsonOutput.toJson(result))
""" % encoded


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--plan", required=True)
    parser.add_argument("--plan-sha256", required=True)
    args = parser.parse_args()
    plan_path = pathlib.Path(args.plan)
    plan = json.loads(checked({"path": str(plan_path), "sha256": args.plan_sha256}))
    repo = pathlib.Path(plan["repositoryRoot"])
    assert plan["schemaVersion"] == 1 and plan["kind"] == "controller-only-queue-reconciliation"
    assert plan_path.parent == repo / ".jenkins/state/queue-reconciliation" / plan["requestId"]
    assert pathlib.Path(plan["operator"]["path"]) == pathlib.Path(__file__).absolute()
    checked(plan["operator"])
    selector = json.loads(checked(plan["selector"]))
    sys.path.insert(0, selector["root"])
    sys.path.append(str(repo))
    from tools.jenkins.deployment.driver import verify_driver
    from tools.jenkins.deployment.spec import load_spec
    from tools.jenkins.deployment.paths import resolve_paths
    from tools.jenkins.deployment.manager import DeploymentManager
    from tools.jenkins.deployment.lifecycle import launch_command, spawn_owned, check_port_available
    from tools.jenkins.deployment.startup_observations import (
        observe_departed_identity, runtime_owner_inventory, observe_home_available)
    from tools.jenkins.deployment.lost_owner import _lifecycle_fence, _empty_persisted_queue
    from tools.jenkins.state.locks import process_lock
    from tools.jenkins.state.atomic import atomic_json
    verified = verify_driver(pathlib.Path(selector["root"]), expected_digest=selector["driverDigest"])
    assert all(selector.get(key) == value for key, value in verified.items())
    spec = load_spec(repo / ".jenkins/deployment-spec.json")
    manager = DeploymentManager(spec, resolve_paths(spec))
    receipt_path = pathlib.Path(plan["receiptPath"])
    assert receipt_path.parent == plan_path.parent and not receipt_path.exists()
    receipt = {"schemaVersion": 1, "kind": plan["kind"], "requestId": plan["requestId"],
        "plan": {"path": str(plan_path), "sha256": args.plan_sha256},
        "startedAt": time.time(), "phase": "preflight", "agentStarted": False,
        "brokerStarted": False, "sourceChanged": False, "queueStateClearedOffline": False, "originalQueueRestoredByteForByte": False,
        "buildResultsWrittenOffline": False, "formalAcceptance": False, "reclaimedBytes": 0}
    job = None
    result = 2

    def save(phase, **fields):
        receipt.update(phase=phase, updatedAt=time.time(), **fields)
        atomic_json(receipt_path, receipt)
        summary = {"phase": phase}
        if "controllerIdentity" in fields:
            summary["controllerPid"] = fields["controllerIdentity"]["pid"]
            summary["controllerBirth"] = fields["controllerIdentity"]["creationTime"]
        if "observation" in fields:
            summary["runs"] = fields["observation"]["runs"]
            summary["queueCount"] = len(fields["observation"]["remainingQueue"])
        for key in ("reasonCode", "message", "persistedQueueEmpty", "lifecyclePointersUnchanged"):
            if key in fields:
                summary[key] = fields[key]
        print(json.dumps(summary, ensure_ascii=False), flush=True)

    api_sequence = 0

    def api(action):
        nonlocal api_sequence
        api_sequence += 1
        status, body = manager._request("scriptText", method="POST",
            data=urllib.parse.urlencode({"script": script_text(plan, action)}).encode(),
            headers=request_headers, timeout=30)
        response_path = plan_path.parent / ("attempt" + str(plan["attempt"]) + "-api-" + str(api_sequence) + "-" + action + ".txt")
        assert not response_path.exists() and len(body) <= 1048576
        response_path.write_bytes(body)
        receipt.setdefault("apiResponses", []).append({"path": str(response_path),
            "sha256": checksum(body), "bytes": len(body), "action": action, "status": status})
        atomic_json(receipt_path, receipt)
        marker = b"ZR_QUEUE_RECONCILE="
        if status != 200 or marker not in body:
            # Error text can contain the pinned script/parameters, never credentials.
            tail = body.decode("utf-8", "replace")[:2400]
            raise RuntimeError("queue_reconciliation_api_rejected: " + str(status) + " " + tail)
        return json.loads(body.split(marker, 1)[1].splitlines()[0])

    request_headers = {"Content-Type": "application/x-www-form-urlencoded"}
    with process_lock(manager.paths.state / "deployment-start.lock", reason="deployment_operation_busy"):
        try:
            assert _lifecycle_fence(manager) == plan["lifecycleFence"]
            if plan.get("forceKnownDeparted"):
                departure = json.loads(checked(plan["nativeDepartureProof"]))
                assert departure["passed"] and departure["originalExecutionAndManagementNativeProved"]
                assert departure["owner"] == plan["originalNativeOwner"]
                assert departure["selectedDriverDigest"] == selector["driverDigest"]
                assert departure["lifecycleFence"] == plan["lifecycleFence"]
                assert json.loads(checked(departure["originalDescriptor"]))["protectedHolds"] == departure["protectedHolds"]
                checked(departure["originalCommand"])
                from tools.jenkins.processes.boot import windows_boot_filetime
                from tools.jenkins.deployment.lost_recovery_runtime import (
                    _execution_gate, _management_quarantine, _execution_comparison,
                    _management_comparison, digest)
                assert str(windows_boot_filetime()) == departure["bootFileTime"]
                executions = _execution_gate(manager, departure["owner"], selector,
                    departure["protectedHolds"], departure["bootFileTime"], same_boot=True)
                management = _management_quarantine(manager, departure["owner"], selector, executions["records"])
                assert digest(_execution_comparison(executions)) == departure["executionComparisonDigest"]
                assert digest(_management_comparison(management)) == departure["managementComparisonDigest"]
                receipt["originalNativeDepartureRevalidated"] = True

            for reference in plan["protectedInputs"]:
                checked(reference)
            for identity in plan["departedIdentities"]:
                observe_departed_identity(identity)
            assert runtime_owner_inventory(repo) == []
            observe_home_available(manager.paths.home)
            check_port_available(spec.controller["listenAddress"], int(spec.controller["httpPort"]))
            original = checked(plan["originalQueue"])
            if not pathlib.Path(plan["originalQueue"]["path"]).exists():
                raise RuntimeError("original_queue_missing_before_restore")
            queue_path = manager.paths.home / "queue.xml"
            if plan.get("restoreConsumedQueue"):
                assert pathlib.Path(plan["originalQueue"]["path"]) == manager.paths.home / "queue.xml.bak"
                assert not queue_path.exists()
                assert checked(plan["archivedQueue"]) == original
                with queue_path.open("xb") as handle:
                    handle.write(original)
                    handle.flush()
                    os.fsync(handle.fileno())
                receipt["originalQueueRestoredByteForByte"] = True
            else:
                assert queue_path == pathlib.Path(plan["originalQueue"]["path"])
            assert checked(plan["archivedQueue"]) == original
            items = xml(original).findall("items/*")
            assert not items if plan.get("originalQueueAlreadyEmpty") else (len(items) == 1 and items[0].findtext("id") == str(plan["queueId"]))
            assert not items or items[0].findtext(".//runId") == "zircon-scheduled-cleanup#65"
            for expected in plan["runs"]:
                assert checked(expected["originalMetadata"]) == checked(expected["archivedMetadata"])
            save("controller-only-preflight-passed", originalQueueSha256=checksum(original),
                originalRunCount=len(plan["runs"]), lifecyclePointersUnchanged=True)
            command, env = launch_command(manager.paths, manager.java, manager.war, int(spec.controller["httpPort"]))
            job = spawn_owned(manager.paths, command, env, component="queue-reconciliation-" + plan["requestId"] + "-attempt" + str(plan["attempt"]))
            save("controller-only-started", controllerIdentity=job.identity.to_dict(),
                controllerLogDirectory=str(manager.paths.logs / "native" / ("queue-reconciliation-" + plan["requestId"] + "-attempt" + str(plan["attempt"]))))
            deadline = time.monotonic() + 300
            while time.monotonic() < deadline:
                if job.poll() is not None:
                    raise RuntimeError("queue_reconciliation_controller_exited_during_start")
                try:
                    status, body = manager._request("api/json", timeout=3)
                    health = json.loads(body)
                    if status == 200 and health.get("quietingDown") is True:
                        break
                except Exception:
                    pass
                time.sleep(1)
            else:
                raise RuntimeError("queue_reconciliation_health_timeout")
            crumb_status, crumb_body, crumb_headers = manager._request_with_headers("crumbIssuer/api/json", timeout=5)
            if crumb_status == 200:
                crumb = json.loads(crumb_body)
                request_headers[crumb["crumbRequestField"]] = crumb["crumb"]
                from http.cookies import SimpleCookie
                cookies = SimpleCookie()
                cookies.load(crumb_headers.get("Set-Cookie", ""))
                if cookies:
                    request_headers["Cookie"] = "; ".join(k + "=" + v.value for k, v in cookies.items())
            observed = api("observe")
            save("original-runs-and-offline-agent-verified", observation=observed)
            interrupted = api("interrupt")
            save("exact-original-interruptions-requested", observation=interrupted)
            deadline = time.monotonic() + 120
            while time.monotonic() < deadline:
                observed = api("observe")
                if not observed["remainingQueue"] and all(not x["building"] and (x["flowComplete"] or (plan.get("forceKnownDeparted") and not x["executionPresent"] and x["result"] == "ABORTED")) for x in observed["runs"]):
                    break
                if any(not x["building"] and x["result"] != "ABORTED" for x in observed["runs"]):
                    raise RuntimeError("queue_reconciliation_unexpected_terminal_result")
                time.sleep(2)
            else:
                raise RuntimeError("queue_reconciliation_original_runs_not_terminal")
            assert all(x["result"] == "ABORTED" for x in observed["runs"])
            save("jenkins-original-runs-terminal-and-queue-empty", observation=observed)
            status, _ = manager._request("safeExit", method="POST", data=b"", headers=request_headers, timeout=10)
            assert status in (200, 302)
            job.wait(timeout_seconds=90, cleanup_descendants=False)
            proof = job.proof(requested=False).to_dict()
            assert proof["complete"] and proof["activeProcesses"] == 0 and proof["processExitCode"] == 0
            assert _lifecycle_fence(manager) == plan["lifecycleFence"]
            for reference in plan["protectedInputs"]:
                checked(reference)
            queue = (manager.paths.home / "queue.xml").read_bytes()
            assert _empty_persisted_queue(xml(queue))
            save("controller-only-reconciliation-complete", terminationProof=proof,
                persistedQueueSha256=checksum(queue), persistedQueueEmpty=True,
                lifecyclePointersUnchanged=True, originalHistoryPreserved=True)
            result = 0
        except Exception as error:
            save("controller-only-reconciliation-failed", reasonCode=getattr(error, "code", type(error).__name__),
                message=str(error))
        finally:
            if job is not None:
                proof = job.proof(requested=False).to_dict()
                if not proof["complete"] and job.poll() is None:
                    try:
                        status, _ = manager._request("safeExit", method="POST", data=b"", headers=request_headers, timeout=10)
                        if status in (200, 302):
                            job.wait(timeout_seconds=45, cleanup_descendants=False)
                            proof = job.proof(requested=False).to_dict()
                            receipt["ownedFailureNormalExit"] = proof
                            atomic_json(receipt_path, receipt)
                    except Exception as stop_error:
                        receipt["ownedFailureNormalExitError"] = getattr(stop_error, "code", type(stop_error).__name__)
                        atomic_json(receipt_path, receipt)
                if not proof["complete"]:
                    # This retained Job contains only this maintenance controller.
                    # Preserve a distinct requested termination proof on failure.
                    proof = job.terminate(timeout_seconds=30).to_dict()
                    receipt["ownedFailureTerminationRequested"] = True
                    receipt["terminationProof"] = proof
                    atomic_json(receipt_path, receipt)
                if not proof["complete"]:
                    print("Owned controller termination remains unproven; retaining the maintenance host.", flush=True)
                    while True:
                        time.sleep(10)
                job.close()
    print(json.dumps({"phase": receipt["phase"], "receipt": str(receipt_path),
        "receiptSha256": checksum(receipt_path.read_bytes()), "exitCode": result}), flush=True)
    return result


if __name__ == "__main__":
    raise SystemExit(main())
