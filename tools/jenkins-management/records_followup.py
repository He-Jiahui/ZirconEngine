"""Attach supplementary inbox messages to an already admitted Jenkins task.

This fixed lifecycle operation never resubmits or edits native requests. The
existing task worker and its immutable source/queue identities stay intact.
"""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import time
import urllib.parse
from xml.sax.saxutils import escape

import records_queue as queue

HERE = Path(__file__).absolute().parent
JOB = "zircon-records-followup"
SOURCES = ("records_followup.py", "records_followup_cleanup.ps1",
           "zircon-records-followup.groovy", "tests/test_records_followup.py")


def root_for(context):
    return context["repo"] / ".jenkins/state/records-queue/followups"


def binding(repo):
    api, context, original = queue.binding(repo)
    installed, _ = queue.read_json(root_for(context) / "installed.json")
    queue.require(installed["job"] == JOB and all(installed[k] == original[k]
                  for k in ("repositoryRoot", "driverDigest", "runtimeOperationId")),
                  "Followup intake belongs to another runtime")
    for name, expected in installed["sources"].items():
        queue.require(queue.fingerprint(HERE / name)[0]["sha256"] == expected,
                      "Pinned supplementary intake source changed")
    return api, context, installed


def validate(context, inventory, original):
    queue.require(isinstance(inventory.get("tasks"), list) and len(inventory["tasks"]) == 1,
                  "One existing task per supplement")
    task = inventory["tasks"][0]
    queue.require(task["kind"] == "task" and not task.get("formalRequests"),
                  "Supplement intake cannot trigger or replace native requests")
    queue.validate_inventory(context["repo"], inventory)
    queue.require(original and original["intakeAccepted"] is True and not original.get("done")
                  and task["taskId"] == original["taskId"]
                  and task["originalIdentity"] == original["originalIdentity"]
                  and inventory.get("originalInventoryDigest") == original["inventoryDigest"],
                  "Supplement must retain the original admitted task identity")
    queue.require(isinstance(inventory.get("response"), dict)
                  and inventory["response"].get("taskId") == task["taskId"]
                  and inventory["response"].get("formalAcceptance") is False,
                  "A scoped pending response for the original task is required")
    for document in inventory.get("documents", []):
        path = queue.plain(document["path"])
        queue.require(path.is_relative_to(context["repo"] / "docs/tests"),
                      "Preparation references must be canonical docs/tests files")
        actual, _ = queue.fingerprint(path)
        queue.require(actual["sha256"] == document["sha256"], "Preparation reference changed")
    return task


def install(repo, *, upgrade_from=None, expected_job_script_sha256=None, rebind_runtime=False):
    api, context, original = queue.binding(repo)
    installed = {"schemaVersion": 1, "job": JOB, "intakeOnlyAcceptance": True,
                 "formalAcceptance": False,
                 **{k: original[k] for k in ("repositoryRoot", "driverDigest", "runtimeOperationId")},
                 "sources": {name: queue.fingerprint(HERE / name)[0]["sha256"] for name in SOURCES}}
    source = queue.render_job_source(context, installed, "records_followup.py", "zircon-records-followup.groovy")
    parameters = "".join("<hudson.model.StringParameterDefinition><name>" + name +
                         "</name><defaultValue></defaultValue><trim>true</trim></hudson.model.StringParameterDefinition>"
                         for name in ("FOLLOWUP_DIGEST", "TASK_ID", "MODE"))
    xml = ("<flow-definition><description>Attach messages to an existing task; no native validation retry</description>"
           "<properties><hudson.model.ParametersDefinitionProperty><parameterDefinitions>" + parameters +
           "</parameterDefinitions></hudson.model.ParametersDefinitionProperty></properties>"
           "<definition class='org.jenkinsci.plugins.workflow.cps.CpsFlowDefinition' plugin='workflow-cps'><script>" +
           escape(source) + "</script><sandbox>true</sandbox></definition><triggers/><disabled>false</disabled></flow-definition>")
    manager = context["manager"]
    installed_path = root_for(context) / "installed.json"
    previous = queue.read_json(installed_path) if installed_path.exists() else None
    if rebind_runtime:
        queue.require(previous is not None and previous[1]["sha256"] == upgrade_from
                      and isinstance(expected_job_script_sha256, str) and queue.SHA.fullmatch(expected_job_script_sha256),
                      "Runtime rebind requires an exactly pinned existing supplementary installation")
    status, raw = manager._request("job/" + JOB + "/config.xml")
    if status == 200:
        old_source = queue.job_script(raw)
        # The template carries source pins, while runtime ownership also lives
        # in installed.json. Metadata-only rebinds need the same full proof.
        if old_source != source or (previous and previous[0] != installed):
            history = queue.verify_install_upgrade(context, previous, installed, old_source, source_names=SOURCES,
                script_name="records_followup.py", template_name="zircon-records-followup.groovy",
                upgrade_from=upgrade_from, expected_job_script_sha256=expected_job_script_sha256,
                rebind_runtime=rebind_runtime)
            queue.recheck_install_inputs(api, repo, context, previous, installed, installed_path)
            queue.write_once(root_for(context) / "install-history" / (upgrade_from + ".json"), queue.encoded(history) + b"\n")
            status, raw = manager._request("crumbIssuer/api/json")
            queue.require(status == 200, "Jenkins crumb unavailable")
            crumb = json.loads(raw)
            queue.require_job_idle(context, JOB)
            queue.recheck_install_inputs(api, repo, context, previous, installed, installed_path)
            status, raw = manager._request("job/" + JOB + "/config.xml")
            queue.require(status == 200 and queue.job_script(raw) == old_source,
                          "Pinned prior supplementary job changed before update")
            status, _ = manager._request("job/" + JOB + "/config.xml", method="POST", data=xml.encode("utf-8"),
                headers={"Content-Type": "application/xml", crumb["crumbRequestField"]: crumb["crumb"]})
            queue.require(status == 200, "Exact supplementary job upgrade was not accepted")
            status, raw = manager._request("job/" + JOB + "/config.xml")
            queue.require(status == 200 and queue.job_script(raw) == source, "Updated supplementary job requires reconciliation")
    else:
        queue.require(status == 404 and previous is None and not rebind_runtime
                      and upgrade_from is None and expected_job_script_sha256 is None,
                      "Cannot create or replace an unproven existing supplementary job")
        queue.recheck_install_inputs(api, repo, context, previous, installed, installed_path)
        status, raw = manager._request("crumbIssuer/api/json")
        queue.require(status == 200, "Jenkins crumb unavailable")
        crumb = json.loads(raw)
        status, _ = manager._request("createItem?name=" + JOB, method="POST", data=xml.encode("utf-8"),
                                    headers={"Content-Type": "application/xml",
                                             crumb["crumbRequestField"]: crumb["crumb"]})
        queue.require(status == 200, "Fixed supplementary intake job was not installed")
        status, raw = manager._request("job/" + JOB + "/config.xml")
        queue.require(status == 200 and queue.job_script(raw) == source, "Created supplementary job requires reconciliation")
    queue.recheck_install_inputs(api, repo, context, previous, installed, installed_path)
    if previous and previous[0] != installed:
        from tools.jenkins.state.atomic import atomic_json
        atomic_json(installed_path, installed)
    elif previous is None:
        queue.write_once(installed_path, queue.encoded(installed) + b"\n")
    return installed


def submit(repo, inventory_file):
    api, context, installed = binding(repo)
    inventory, _ = queue.read_json(inventory_file)
    task_id = inventory["tasks"][0]["taskId"]
    original = api.record(context["repo"], "records_queue_task", task_id)
    validate(context, inventory, original["payload"] if original else None)
    digest = queue.sha(queue.encoded(inventory))
    root = root_for(context)
    from tools.jenkins.state.atomic import atomic_json
    from tools.jenkins.state.locks import process_lock
    with process_lock(root / "submission.lock", reason="records_followup_busy"):
        queue.write_once(root / "inputs" / (digest + ".json"), queue.encoded(inventory) + b"\n")
        marker = root / "submissions" / (digest + ".json")
        if marker.exists():
            return queue.read_json(marker)[0]
        receipt = {"taskId": task_id, "followupDigest": digest,
                   "originalInventoryDigest": inventory["originalInventoryDigest"],
                   "originalIdentity": original["payload"]["originalIdentity"],
                   "driverDigest": installed["driverDigest"], "runtimeOperationId": installed["runtimeOperationId"],
                   "phase": "dispatching", "queueUrl": None, "submittedAt": time.time(),
                   "formalAcceptance": False, "newNativeRequest": False}
        queue.write_once(marker, queue.encoded(receipt) + b"\n")
        try:
            status, raw = context["manager"]._request("crumbIssuer/api/json")
            queue.require(status == 200, "Jenkins crumb unavailable")
            crumb = json.loads(raw)
            data = urllib.parse.urlencode({"TASK_ID": task_id, "FOLLOWUP_DIGEST": digest, "MODE": "attach"}).encode()
            status, _, headers = context["manager"]._request_with_headers(
                "job/" + JOB + "/buildWithParameters", method="POST", data=data,
                headers={"Content-Type": "application/x-www-form-urlencoded",
                         crumb["crumbRequestField"]: crumb["crumb"]}, timeout=30)
            location = next((v for k, v in headers.items() if k.lower() == "location"), None)
            queue.require(status == 201 and location and location.startswith(
                context["manager"].base_url.rstrip("/") + "/queue/item/"), "Followup delivery acknowledgement unknown")
            receipt.update(phase="queued", queueUrl=location)
        except Exception as error:
            receipt.update(phase="delivery_unknown", error=str(error))
        atomic_json(marker, receipt)
        return receipt


def work(repo, task_id, digest):
    api, context, installed = binding(repo)
    queue.require(queue.ID.fullmatch(task_id) and queue.SHA.fullmatch(digest), "Invalid supplementary identity")
    queue.require(os.environ.get("JOB_NAME") == JOB and os.environ.get("BUILD_URL")
                  and os.environ.get("BUILD_NUMBER"), "Fixed Jenkins supplementary worker required")
    root = root_for(context)
    inventory, _ = queue.read_json(root / "inputs" / (digest + ".json"))
    marker, _ = queue.read_json(root / "submissions" / (digest + ".json"))
    queue.require(queue.sha(queue.encoded(inventory)) == digest and marker["phase"] == "queued",
                  "Supplement input or durable reception acknowledgement differs")
    status, raw = context["manager"]._request("job/" + JOB + "/" + os.environ["BUILD_NUMBER"] +
        "/api/json?tree=url,building,queueId,actions[parameters[name,value]]")
    queue.require(status == 200, "Cannot verify actual supplementary build")
    live = json.loads(raw)
    params = {p["name"]: p.get("value") for a in live.get("actions", []) for p in a.get("parameters", [])}
    queue.require(live["url"] == os.environ["BUILD_URL"] and live["building"] is True
                  and params == {"TASK_ID": task_id, "FOLLOWUP_DIGEST": digest, "MODE": "attach"}
                  and marker["queueUrl"].rstrip("/").endswith("/" + str(live["queueId"])),
                  "Supplementary build differs from the original queued operation")
    from tools.jenkins.state import State
    from tools.jenkins.state.locks import process_lock
    state = State(context["repo"] / ".jenkins/state/coordination.sqlite3")
    intake_root = root.parent
    with process_lock(intake_root / "locks" / (task_id + ".lock"), reason="records_task_busy"):
        parent = state.get("records_queue_task", task_id)
        task = validate(context, inventory, parent["payload"] if parent else None)
        previous = state.get("records_queue_followup", digest)
        if previous:
            queue.require(previous["payload"]["buildUrl"] == os.environ["BUILD_URL"],
                          "Reconcile the original supplementary operation, do not repeat it")
            result = previous["payload"]
        else:
            retained = queue.archive_messages(context["repo"], intake_root, task, digest)
            response = {**inventory["response"], "followupDigest": digest,
                        "originalIdentity": task["originalIdentity"], "originalBuildUrl": parent["payload"]["buildUrl"],
                        "originalInventoryDigest": inventory["originalInventoryDigest"],
                        "pendingRequirements": task["pendingRequirements"],
                        "supplementBuildUrl": os.environ["BUILD_URL"], "ordinaryEntry": api.CAPABILITY,
                        "intakeAccepted": True, "nativeValidationAccepted": False}
            response_path = root / "responses" / (digest + ".json")
            queue.write_once(response_path, queue.encoded(response) + b"\n")
            response_pin, _ = queue.fingerprint(response_path)
            result = {"taskId": task_id, "followupDigest": digest, "originalInventoryDigest": inventory["originalInventoryDigest"],
                      "originalIdentity": task["originalIdentity"], "messages": retained,
                      "buildUrl": os.environ["BUILD_URL"], "buildNumber": os.environ["BUILD_NUMBER"],
                      "queueReceipt": marker, "response": response_pin, "phase": "attached",
                      "intakeAccepted": True, "formalAcceptance": False, "newNativeRequest": False,
                      "pendingRequirements": task["pendingRequirements"], "acceptedAt": time.time()}
            with state.transaction() as conn:
                current = parent["payload"]
                current.setdefault("supplements", []).append({"followupDigest": digest,
                    "response": response_pin, "buildUrl": result["buildUrl"], "intakeAccepted": True})
                current.setdefault("contractAmendments", []).append(inventory["response"].get("contractAmendment", {}))
                current["pendingRequirements"] += [p for p in task["pendingRequirements"] if p not in current["pendingRequirements"]]
                state.put("records_queue_task", task_id, current, expected_version=parent["version"], connection=conn)
                state.put("records_queue_followup", digest, result, expected_version=0, connection=conn)
                for message in retained:
                    key = queue.sha(message["path"].casefold().encode())
                    state.put("records_queue_message", key, {**message, "taskId": task_id,
                        "followupDigest": digest, "originalInventoryDigest": inventory["originalInventoryDigest"],
                        "buildUrl": result["buildUrl"]}, expected_version=0, connection=conn)
        cleanup_id = "followup-" + digest
        receipt_path = intake_root / "cleanup" / (cleanup_id + ".receipt.json")
        if not receipt_path.exists():
            helper = Path("D:/CodexData/.codex/skills/artifact-lifecycle/scripts/GuardedFileDelete.cs")
            config = context["repo"] / ".codex/config.toml"
            plan = {"taskId": cleanup_id, "parentTaskId": task_id, "inventoryDigest": digest,
                    "repositoryRoot": str(context["repo"]),
                    "authorization": "user: enqueue all tasks, archive evidence, then empty inbox",
                    "permissionConfigSha256": queue.fingerprint(config)[0]["sha256"],
                    "helperSha256": queue.fingerprint(helper)[0]["sha256"], "files": result["messages"]}
            plan_path = intake_root / "cleanup" / (cleanup_id + ".json")
            queue.write_once(plan_path, queue.encoded(plan) + b"\n")
            proc = subprocess.run(["powershell.exe", "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "RemoteSigned",
                "-File", str(HERE / "records_followup_cleanup.ps1"), "-PlanPath", str(plan_path),
                "-ReceiptPath", str(receipt_path), "-ExpectedScriptSha256", installed["sources"]["records_followup_cleanup.ps1"]],
                cwd=context["repo"], capture_output=True, timeout=120)
            queue.require(proc.returncode == 0 and receipt_path.exists(), "Supplementary guarded cleanup failed: " +
                          proc.stderr.decode("utf-8", errors="replace")[-1200:])
        # Reuse the original accounting/reconciliation contract. An existing
        # failed receipt is never deleted, rewritten or blindly replayed.
        result["cleanup"] = queue.cleanup(context, installed, intake_root, cleanup_id, digest, result["messages"])
        result.update(phase="attached-and-cleared", finishedAt=time.time())
        old = state.get("records_queue_followup", digest)
        state.put("records_queue_followup", digest, result, expected_version=old["version"])
        output = root / "controls" / (digest + ".json")
        from tools.jenkins.state.atomic import atomic_json
        atomic_json(output, result)
        return result


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("install", "submit", "work", "self-test"))
    parser.add_argument("--repo-root", required=True)
    parser.add_argument("--inventory-file")
    parser.add_argument("--task-id")
    parser.add_argument("--followup-digest")
    parser.add_argument("--expected-source-sha256")
    parser.add_argument("--upgrade-from")
    parser.add_argument("--expected-job-script-sha256")
    parser.add_argument("--rebind-runtime", action="store_true")
    args = parser.parse_args(argv)
    try:
        if args.expected_source_sha256:
            queue.require(queue.fingerprint(HERE / "records_followup.py")[0]["sha256"] == args.expected_source_sha256,
                          "Executing supplementary source changed")
        if args.command == "install":
            result = install(args.repo_root, upgrade_from=args.upgrade_from,
                             expected_job_script_sha256=args.expected_job_script_sha256,
                             rebind_runtime=args.rebind_runtime)
        elif args.command == "submit":
            result = submit(args.repo_root, args.inventory_file)
        elif args.command == "work":
            result = work(args.repo_root, args.task_id, args.followup_digest)
        else:
            binding(args.repo_root)
            queue.require(os.environ.get("JOB_NAME") == JOB, "Use the fixed Jenkins test worker")
            subprocess.run([sys.executable, "-B", "-X", "utf8", str(HERE / "tests/test_records_followup.py")], check=True)
            result = {"status": "passed", "formalAcceptance": False}
        print(queue.encoded(result).decode())
        return 0
    except Exception as error:
        print(queue.encoded({"status": "pending", "message": str(error), "formalAcceptance": False}).decode())
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
