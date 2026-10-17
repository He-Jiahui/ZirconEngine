"""Restore ordinary regression routing without discarding a bounded diagnostic route.

This changes only the existing Jenkins test job. It never installs a capsule,
starts a service, submits a build, changes Git, or adopts another task's inputs.
"""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import xml.etree.ElementTree as ET

ROOT = Path("E:/Git/ZirconEngine")
ENDPOINT = "job/zircon-management-tests/config.xml"
BEGIN = "// BEGIN ZIRCON RETAINED SINGLE-RUN TEST"
END = "// END ZIRCON RETAINED SINGLE-RUN TEST"
ORDINARY = "// BEGIN ZIRCON ORDINARY MANAGEMENT TESTS\n"
ID = re.compile(r"[A-Za-z0-9][A-Za-z0-9_.-]{0,79}\Z")


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def normalized(text):
    return text.lstrip("\ufeff").replace("\r\n", "\n").rstrip("\n") + "\n"


def xml_value(element):
    return (element.tag, sorted(element.attrib.items()),
            element.text if element.text and element.text.strip() else "",
            [xml_value(child) for child in element])


def compose(original, ordinary, addon, preserve_run_id):
    """Retain the reviewed old program under its existing exact run selector."""
    original, ordinary, addon = map(normalized, (original, ordinary, addon))
    prefix = addon.rstrip("\n") + "\n\n"
    if original.startswith(prefix):
        core = original[len(prefix):]
    else:
        require("// BEGIN ZIRCON SCOPED COMMIT TESTS" not in original,
                "Scoped commit extension differs; preserve its owner and reconcile it")
        core, prefix = original, ""
    if core == ordinary:
        return original
    require(isinstance(preserve_run_id, str) and ID.fullmatch(preserve_run_id),
            "The existing diagnostic route requires its exact reviewed RUN_ID")
    header = BEGIN + "\nif (params.RUN_ID != null && params.RUN_ID.toString().trim() == '" + preserve_run_id + "') {\n"
    footer = "    return\n}\n" + END + "\n\n" + ORDINARY
    if core.startswith(header):
        require(core.count(BEGIN) == core.count(END) == 1 and footer in core,
                "Retained route framing differs")
        retained, previous_ordinary = core[len(header):].split(footer, 1)
        require(previous_ordinary == ordinary, "Installed ordinary template changed; review before updating")
        return original
    require(BEGIN not in core and END not in core and ORDINARY not in core,
            "Unrecognized mixed test routes require explicit reconciliation")
    guard = "if (runId != '" + preserve_run_id + "') error('Unexpected RUN_ID')"
    require(core.count(guard) == 1 and "node('zircon-windows-agent')" in core
            and "env.JOB_NAME != 'zircon-management-tests'" in core,
            "Existing program is not the reviewed single-run diagnostic route")
    return prefix + header + core + footer + ordinary


def restore(manager, expected_sha256, ordinary, addon, preserve_run_id, observe):
    """Pin the live config, reject active tests, POST once, verify read-back."""
    status, raw = manager._request(ENDPOINT)
    require(status == 200 and sha(raw) == expected_sha256, "Test job changed since review")
    document = ET.fromstring(raw)
    definition = document.find("definition")
    require(document.tag == "flow-definition" and document.findtext("disabled") == "false"
            and definition is not None and definition.get("class") == "org.jenkinsci.plugins.workflow.cps.CpsFlowDefinition"
            and definition.findtext("sandbox") == "true", "Unexpected regression job definition")
    script = definition.find("script")
    require(script is not None and script.text, "Regression script is missing")
    wanted = compose(script.text, ordinary, addon, preserve_run_id)
    if script.text == wanted:
        return {"status": "installed", "unchanged": True, "configSha256": sha(raw)}
    status, inventory = manager._request("job/zircon-management-tests/api/json?tree=builds[number,building]{0,128},inQueue")
    require(status == 200, "Test job activity is unavailable")
    activity = json.loads(inventory)
    require(activity.get("inQueue") is False and not any(v.get("building") for v in activity.get("builds", [])),
            "A regression delivery is active; preserve it before changing routes")
    status, queue = manager._request("queue/api/json?tree=items[task[name]]")
    require(status == 200 and not any(v.get("task", {}).get("name") == "zircon-management-tests"
                                    for v in json.loads(queue).get("items", [])), "Test delivery is queued")
    observe()
    status, check = manager._request(ENDPOINT)
    require(status == 200 and check == raw, "Test job changed before route update")
    script.text = wanted
    updated = ET.tostring(document, encoding="utf-8", xml_declaration=True)
    status, crumb_raw = manager._request("crumbIssuer/api/json")
    require(status == 200, "Jenkins crumb is unavailable")
    crumb = json.loads(crumb_raw)
    # An uncertain POST is reconciled by read-back, never blindly repeated.
    post_error = None
    try:
        status, _ = manager._request(ENDPOINT, method="POST", data=updated,
            headers={"Content-Type": "application/xml; charset=utf-8", crumb["crumbRequestField"]: crumb["crumb"]})
        require(status in (200, 201, 302), "Test job update was rejected: HTTP " + str(status))
    except Exception as error:
        post_error = type(error).__name__
    status, after = manager._request(ENDPOINT)
    require(status == 200, "Test route update has an unknown result; reconcile the current config")
    actual = ET.fromstring(after)
    # Jenkins may reserialize XML; compare the entire parsed definition.
    require(xml_value(actual) == xml_value(document),
            "Test route read-back differs; retain the original delivery, do not replay")
    observe()
    return {"status": "installed", "unchanged": False, "configSha256Before": sha(raw),
            "configSha256": sha(after), "scriptSha256": sha(wanted.encode()),
            "retainedRunId": preserve_run_id, "postTransportError": post_error,
            "ordinaryRoute": True, "scopedCommitRoutePreserved": addon.strip() in wanted,
            "restartPerformed": False, "buildSubmitted": False, "gitChanged": False}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--expected-job-sha256", required=True)
    parser.add_argument("--preserve-run-id", required=True)
    parser.add_argument("--codex-exe", required=True, type=Path)
    args = parser.parse_args(argv)
    require(re.fullmatch(r"[0-9a-f]{64}", args.expected_job_sha256), "Expected config hash is invalid")
    spec = importlib.util.spec_from_file_location("test_job_verified_entry", ROOT / "tools/jenkins-management/entry.py")
    entry = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(entry)
    context = entry.capability(ROOT)
    require(entry.plain(args.codex_exe).is_file(), "Pinned Codex executable is unavailable")
    config = context["config"]
    template = ROOT / "tools/jenkins-management/zircon-management-tests.groovy"
    addon = ROOT / "tools/jenkins-management/checkout-commit-tests.groovy"
    raw_template, raw_addon = template.read_bytes(), addon.read_bytes()
    ordinary = raw_template.decode("utf-8-sig")
    pins = {"AGENT": config["agentLabel"], "REPOSITORY": str(ROOT).replace("\\", "/"),
            "PYTHON": str(ROOT / ".jenkins/runtime/python/python.exe").replace("\\", "/"),
            "CODEX": str(args.codex_exe).replace("\\", "/"),
            "DRIVER": config["driverDigest"], "RUNTIME": config["runtimeOperationId"]}
    for name, value in pins.items():
        require(isinstance(value, str) and not any(c in value for c in "'\r\n"), "Unsafe pipeline binding")
        ordinary = ordinary.replace("@@" + name + "@@", value)
    require("@@" not in ordinary, "Unresolved test template binding")

    def observe():
        from tools.jenkins.deployment.drain_observations import owner_observation
        owner_observation(context["manager"], expected=context["owner"])
        require(template.read_bytes() == raw_template and addon.read_bytes() == raw_addon,
                "Canonical test routes changed during restoration")
    result = restore(context["manager"], args.expected_job_sha256, ordinary,
                     raw_addon.decode("utf-8-sig"), args.preserve_run_id, observe)
    result.update(driverDigest=config["driverDigest"], runtimeOperationId=config["runtimeOperationId"],
                  ordinaryTemplateSha256=sha(raw_template), commitTemplateSha256=sha(raw_addon))
    print(json.dumps(result, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
