"""Bounded Jenkins deletion of old, merged and unoccupied local branches."""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import importlib.util
import json
import os
from pathlib import Path
import re
import time
import urllib.parse
import xml.etree.ElementTree as ET

ROOT = Path("E:/Git/ZirconEngine")
REPORT = Path("E:/cargo-targets/zircon-local/reports/branch-cleanup-before-20261016-20261018")
BEGIN = "// BEGIN ZIRCON OLD BRANCH CLEANUP"
END = "// END ZIRCON OLD BRANCH CLEANUP"
ACTION = "prune-old-branches"


def helper():
    spec = importlib.util.spec_from_file_location("branch_cleanup_commit", Path(__file__).with_name("checkout_commit.py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def inventory(h, repo):
    merged = set(h.git(repo, "for-each-ref", "--merged=refs/heads/main", "--format=%(refname)", "refs/heads").stdout.decode().splitlines())
    raw = h.git(repo, "for-each-ref", "--format=%(refname)%09%(objectname)%09%(committerdate:unix)%09%(worktreepath)", "refs/heads").stdout.decode("utf-8")
    common = h.physical(h.git(repo, "rev-parse", "--path-format=absolute", "--git-common-dir").stdout.decode().strip())
    result = {}
    for line in raw.splitlines():
        ref, oid, commit_time, worktree = line.split("\t")
        updated = int(commit_time)
        log = h.physical(common / "logs" / ref)
        if log.is_file():
            lines = log.read_bytes().splitlines()
            if lines:
                updated = max(updated, int(lines[-1].split(b"\t", 1)[0].rsplit(b" ", 2)[-2]))
        result[ref] = {"ref": ref, "oid": oid, "updatedAt": updated,
                       "mergedMain": ref in merged, "worktree": worktree}
    return result


def validate_plan(h, plan):
    if (plan.get("action") != ACTION or plan.get("repositoryRoot") != str(ROOT)
            or plan.get("cutoff") != "2026-10-16T00:00:00+08:00"
            or not h.ID.fullmatch(plan.get("operationId", ""))):
        raise ValueError("This cleanup requires its exact repository, user cutoff and operation ID")
    branches = plan.get("branches")
    if not isinstance(branches, list) or not 1 <= len(branches) <= 256:
        raise ValueError("An explicit bounded branch list is required")
    seen = set()
    for item in branches:
        ref, oid = item.get("ref", ""), item.get("oid", "")
        if (not ref.startswith("refs/heads/") or ref in {"refs/heads/main", "refs/heads/master"}
                or ref in seen or not re.fullmatch(r"[0-9a-f]{40}", oid)):
            raise ValueError("Cleanup only accepts unique non-main local refs with recorded Git commits")
        seen.add(ref)
    return int(datetime.fromisoformat(plan["cutoff"]).timestamp())


def run(h, plan):
    cutoff = validate_plan(h, plan)
    if os.environ.get("JOB_NAME") != "zircon-management" or os.environ.get("NODE_NAME") != "zircon-windows-agent":
        raise ValueError("Branch deletion requires the actual Jenkins management agent")
    if os.environ.get("ZR_BRANCH_CLEANUP_OPERATION") != plan["operationId"]:
        raise ValueError("Cleanup differs from its Jenkins operation")
    selected = h.read_json(ROOT / ".jenkins/state/deployment/driver.json")
    for key, env in [("driverDigest", "ZR_BRANCH_CLEANUP_DRIVER"), ("runtimeOperationId", "ZR_BRANCH_CLEANUP_RUNTIME")]:
        if selected[key] != os.environ.get(env):
            raise ValueError("Jenkins runtime changed before branch cleanup")
    os.environ["ZR_COMMIT_NATIVE"] = "1"
    receipt = REPORT / "receipt.json"
    result = h.read_json(receipt) if receipt.exists() else {
        "operationId": plan["operationId"], "status": "running", "deleted": [],
        "retained": plan.get("retained", []), "startedAt": time.time(),
        "remoteMutation": False, "worktreesDeleted": 0, "objectsPruned": False}
    if result["operationId"] != plan["operationId"]:
        raise ValueError("Existing cleanup receipt belongs to another operation")
    with h.git_lock(ROOT / ".jenkins/state/locks/management-git.lock"):
        current = inventory(h, ROOT)
        done = {entry["ref"] for entry in result["deleted"]}
        # Validate the entire bounded selection before the first mutation.
        for expected in plan["branches"]:
            ref = expected["ref"]
            actual = current.get(ref)
            if ref in done:
                if actual is not None:
                    raise ValueError("A deleted branch was recreated; retain its new identity: " + ref)
                continue
            if actual is None or actual["oid"] != expected["oid"]:
                raise ValueError("Branch changed after selection: " + ref)
            if actual["updatedAt"] >= cutoff or actual["worktree"] or not actual["mergedMain"]:
                raise ValueError("Branch is recent, checked out or contains unique commits: " + ref)
        h.write_json(REPORT / "executed-plan.json", plan)
        h.write_json(receipt, result)
        for expected in plan["branches"]:
            if expected["ref"] in done:
                continue
            # Native compare-and-delete preserves a concurrently advanced ref.
            h.git(ROOT, "update-ref", "-d", expected["ref"], expected["oid"])
            result["deleted"].append(expected)
            h.write_json(receipt, result)
        remaining = inventory(h, ROOT)
        if any(item["ref"] in remaining for item in plan["branches"]):
            raise ValueError("A selected branch still exists; reconcile the deletion receipt")
        result.update(status="complete", finishedAt=time.time(), deletedBranches=len(result["deleted"]),
                      remainingBranches=len(remaining), nativeCommandsTerminated=True,
                      retainedOld=[v for v in remaining.values() if v["updatedAt"] < cutoff],
                      mainHead=h.git(ROOT, "rev-parse", "refs/heads/main").stdout.decode().strip())
        h.write_json(receipt, result)
    return result


def install(h, client):
    endpoint = "/job/zircon-management/config.xml"
    status, raw, _ = client.request(endpoint)
    if status != 200:
        raise ValueError("Existing Jenkins management job is unavailable")
    doc = ET.fromstring(raw)
    script = doc.find("definition/script")
    if script is None or "--capsule" not in script.text or "zircon-windows-agent" not in script.text:
        raise ValueError("Management core identity changed")
    old = script.text
    if BEGIN in old:
        a, b = old.index(BEGIN), old.index(END) + len(END)
        old = old[:a] + old[b:].lstrip("\r\n")
    addon = Path(__file__).with_suffix(".groovy").read_text(encoding="utf-8-sig")
    script.text = addon.rstrip() + "\n\n" + old
    choices = doc.find(".//parameterDefinitions/*[name='ACTION']/choices/a")
    if choices is not None and ACTION not in [entry.text for entry in choices]:
        ET.SubElement(choices, "string").text = ACTION
    status, _, _ = client.request(endpoint, method="POST", data=ET.tostring(doc, encoding="utf-8"), content_type="application/xml")
    if status not in (200, 201, 302):
        raise ValueError("Branch cleanup installation failed")
    status, raw, _ = client.request(endpoint)
    if status != 200 or ET.fromstring(raw).findtext("definition/script") != script.text:
        raise ValueError("Branch cleanup lacks matching Jenkins API read-back")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("submit", "run"))
    parser.add_argument("--input-file", type=Path, required=True)
    args = parser.parse_args()
    h = helper()
    plan = h.read_json(args.input_file)
    validate_plan(h, plan)
    if args.command == "run":
        result = run(h, plan)
        print(json.dumps({k: result[k] for k in ("status", "deletedBranches", "remainingBranches", "retainedOld")}, ensure_ascii=False))
        return
    client = h.transport(ROOT)
    selected = h.read_json(ROOT / ".jenkins/state/deployment/driver.json")
    operation = h.read_json(ROOT / ".jenkins/state/deployment-operation.json")
    if operation.get("operationId") != selected["runtimeOperationId"] or operation.get("state") != "running":
        raise ValueError("Selected Jenkins runtime is not running")
    marker = REPORT / "submission.json"
    if marker.exists():
        print(json.dumps(h.read_json(marker)))
        return
    install(h, client)
    fields = {"ACTION": ACTION, "SESSION_ID": plan["sessionId"], "REQUEST_ID": plan["operationId"],
              "OPERATION_ID": plan["operationId"], "INPUT_JSON": json.dumps(plan),
              "EXPECTED_DRIVER_DIGEST": selected["driverDigest"], "RUNTIME_OPERATION_ID": selected["runtimeOperationId"]}
    result = {"status": "dispatching", "operationId": plan["operationId"], "inputFile": str(args.input_file)}
    h.write_json(marker, result)
    status, _, headers = client.request("/job/zircon-management/buildWithParameters", method="POST",
        data=urllib.parse.urlencode(fields).encode(), content_type="application/x-www-form-urlencoded")
    result.update(status="queued" if status in (201, 302) else "delivery-unknown", httpStatus=status, queueUrl=headers.get("Location"))
    h.write_json(marker, result)
    print(json.dumps(result))


if __name__ == "__main__":
    main()
