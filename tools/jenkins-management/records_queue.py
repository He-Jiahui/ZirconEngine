"""Durable Jenkins handoff intake. Intake acceptance never means validation acceptance.

The fixed Jenkins job owns message archiving, task records and guarded inbox
cleanup. Execution is delegated only to the verified ordinary management entry.
Unrepresentable requests retain their complete contracts and remain waiting.
"""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import sys
import time
import urllib.parse
from xml.sax.saxutils import escape

HERE = Path(__file__).absolute().parent
JOB = "zircon-records"
ID = re.compile(r"[A-Za-z0-9][A-Za-z0-9_.-]{0,79}\Z")
SHA = re.compile(r"[0-9a-f]{64}\Z")
SOURCES = ("records_queue.py", "records_cleanup.ps1", "zircon-records.groovy", "tests/test_records_queue.py")


def encoded(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False,
                      allow_nan=False).encode("utf-8")


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def require(value, message):
    if not value:
        raise ValueError(message)


def plain(value):
    path = Path(value).absolute()
    require(".." not in Path(value).parts, "Parent traversal is not accepted")
    for item in (path, *path.parents):
        if item.exists() or item.is_symlink():
            require(not item.is_symlink() and not getattr(item.lstat(), "st_file_attributes", 0) & 0x400,
                    "Path aliases are not accepted")
    return path


def fingerprint(path):
    path = plain(path)
    before = path.stat()
    require(stat.S_ISREG(before.st_mode) and before.st_nlink == 1 and before.st_size <= 8 * 1024**2,
            "A bounded ordinary file with one physical link is required")
    raw = path.read_bytes()
    after = path.stat()
    require((before.st_ino, before.st_size, before.st_mtime_ns) ==
            (after.st_ino, after.st_size, after.st_mtime_ns), "File changed while reading")
    return {"path": str(path), "bytes": before.st_size, "mtimeNs": before.st_mtime_ns,
            "mtimeUtcTicks": before.st_mtime_ns // 100 + 621355968000000000,
            "sha256": sha(raw)}, raw


def read_json(path):
    pin, raw = fingerprint(path)
    def pairs(items):
        result = {}
        for name, value in items:
            require(name not in result, "Duplicate JSON field")
            result[name] = value
        return result
    return json.loads(raw, object_pairs_hook=pairs), pin


def write_once(path, raw):
    path = plain(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    try:
        with path.open("xb") as stream:
            stream.write(raw)
            stream.flush()
            os.fsync(stream.fileno())
    except FileExistsError:
        require(path.read_bytes() == raw, "Existing immutable evidence has different bytes")


def entry():
    spec = importlib.util.spec_from_file_location("records_ordinary_entry", HERE / "entry.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def transport_context(repo, api):
    """Verify the HTTP receiver without importing or observing its native owner."""
    import base64
    import urllib.error
    import urllib.request
    import xml.etree.ElementTree as ET

    repo = plain(repo)
    installed, original = api.document(repo / ".jenkins/state/management/installed.json")
    checksum = installed.get("capsuleSha256")
    require(isinstance(checksum, str) and SHA.fullmatch(checksum), "Invalid installed capsule hash")
    capsule = plain(installed["capsule"])
    require(capsule == repo / ".jenkins/runtime/management" / checksum, "Capsule escaped its runtime")
    config, raw = api.document(capsule / "capsule.json")
    require(sha(raw) == checksum and plain(config["repositoryRoot"]) == repo
            and installed.get("job") == "zircon-management"
            and config.get("schemaVersion") == installed.get("schemaVersion") == 1
            and installed.get("formalAcceptance") is False, "Installed management binding changed")
    for key in ("driverDigest", "runtimeOperationId", "ordinaryEntry", "managedWorktree"):
        require(config.get(key) == installed.get(key), "Installed runtime binding changed")
    require(config.get("ordinaryEntry") == api.CAPABILITY, "Ordinary entry capability differs")
    legacy = {"runner.py", "source_preparation.py", "zircon-management.groovy"}
    require(set(config["files"]) in (legacy, legacy | {"checkout_guard.py"},
            legacy | {"checkout_guard.py", "checkout_product.py"}), "Capsule code inventory differs")
    for name, expected in config["files"].items():
        require(isinstance(expected, str) and SHA.fullmatch(expected)
                and sha(plain(capsule / name).read_bytes()) == expected, "Pinned capsule source changed")
    # Client source hashes identify the installed version. Native boot owns their
    # admission; transport does not require mutable canonical files to be identical.
    require(set(config["clientSources"]) == api.CLIENT_SOURCES
            and all(isinstance(v, str) and SHA.fullmatch(v)
                    for v in config["clientSources"].values()), "Client version inventory differs")
    selected, _ = api.document(repo / ".jenkins/state/deployment/driver.json")
    driver = plain(repo / ".jenkins/runtime/drivers" / config["driverDigest"])
    require(plain(selected["root"]) == driver and all(selected[k] == config[k]
            for k in ("driverDigest", "runtimeOperationId")), "Selected runtime changed")
    manifest, _ = api.document(driver / "driver-manifest.json")
    require(sha(encoded({k: manifest[k] for k in ("schemaVersion", "files")}))
            == config["driverDigest"], "Driver manifest changed")
    for name, fact in manifest["files"].items():
        require(isinstance(name, str) and name and "\\" not in name and ":" not in name
                and all(p not in {"", ".", ".."} for p in name.split("/")), "Driver member escaped its root")
        data = plain(driver / name).read_bytes()
        require(len(data) == fact["bytes"] and sha(data) == fact["sha256"], "Pinned driver source changed")
    runtime, _ = api.document(repo / ".jenkins/state/deployment-operation.json")
    require(runtime.get("operationId") == runtime.get("generation") == config["runtimeOperationId"]
            and runtime.get("state") == "running", "Selected runtime is pending")
    spec, _ = api.document(repo / ".jenkins/deployment-spec.json")
    controller = spec["controller"]
    require(controller["listenAddress"] == "127.0.0.1" and type(controller["httpPort"]) is int
            and 1 <= controller["httpPort"] <= 65535, "Loopback HTTP binding differs")
    credentials, _ = api.document(repo / ".jenkins/state/credentials.json")
    token = credentials.get("apiToken") or credentials.get("password")
    require(credentials.get("authenticationMode") == "api-token"
            and credentials.get("username") and token, "API credentials unavailable")
    authorization = "Basic " + base64.b64encode(
        (str(credentials["username"]) + ":" + str(token)).encode()).decode("ascii")

    class Direct(urllib.request.HTTPRedirectHandler):
        def redirect_request(self, *args, **kwargs):
            return None

    class Transport:
        base_url = "http://127.0.0.1:" + str(controller["httpPort"])
        opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), Direct())

        def _request_with_headers(self, path, *, method="GET", data=None, headers=None, timeout=30):
            request = urllib.request.Request(self.base_url + "/" + path, data=data, method=method,
                headers={"Authorization": authorization, "Accept": "application/json", **(headers or {})})
            try:
                response = self.opener.open(request, timeout=min(timeout, 30))
            except urllib.error.HTTPError as error:
                response = error
            with response:
                body = response.read(8 * 1024**2 + 1)
                require(len(body) <= 8 * 1024**2, "HTTP response exceeds the input bound")
                return response.status, body, dict(response.headers.items())

        def _request(self, path, **kwargs):
            status, body, _ = self._request_with_headers(path, **kwargs)
            return status, body

    manager = Transport()
    status, raw = manager._request("job/zircon-management/config.xml")
    require(status == 200, "Management HTTP receiver unavailable")
    job = ET.fromstring(raw)
    script = job_script(raw)
    require(job.findtext("./disabled") == "false", "Management HTTP receiver is disabled")
    codex = re.findall(r"(?m)^def codex = '([A-Za-z]:/[^'\r\n\"]+)'$", script)
    require(len(codex) == 1, "Pinned management launcher differs")
    template = plain(capsule / "zircon-management.groovy").read_text(encoding="utf-8-sig").replace("\r\n", "\n")
    values = {"AGENT": config["agentLabel"], "REPOSITORY": str(repo),
              "PYTHON": str(repo / ".jenkins/runtime/python/python.exe"), "CODEX": codex[0],
              "CAPSULE": str(capsule), "DRIVER": config["driverDigest"], "RUNTIME": config["runtimeOperationId"]}
    for key, value in values.items():
        template = template.replace("@@" + key + "@@", value.replace("\\", "/").replace("'", "\\'"))
    require("@@" not in template and script.replace("\r\n", "\n").count(template) == 1,
            "Management HTTP receiver differs from its pinned capsule")
    require({v.findtext("name") for v in job.findall(
        "./properties/hudson.model.ParametersDefinitionProperty/parameterDefinitions/*")} ==
        {"SESSION_ID", "REQUEST_ID", "OPERATION_ID", "ACTION", "INPUT_JSON",
         "EXPECTED_DRIVER_DIGEST", "RUNTIME_OPERATION_ID", "QUERY_ONLY"}, "Management parameters differ")
    require(plain(repo / ".jenkins/state/management/installed.json").read_bytes() == original,
            "Installed capability changed during observation")
    # The standalone CLI needs the verified driver for pure atomic/lock imports.
    sys.path.insert(0, str(driver))
    return {"repo": repo, "installed": installed, "config": config, "manager": manager}


def verify_transport_job(context, installed, script_name, template_name, parameters):
    """Bind the original Records parameter protocol to its actual HTTP job."""
    import xml.etree.ElementTree as ET
    status, raw = context["manager"]._request("job/" + installed["job"] + "/config.xml")
    require(status == 200 and job_script(raw) == render_job_source(
        context, installed, script_name, template_name), "Pinned Records HTTP receiver differs")
    job = ET.fromstring(raw)
    require(job.findtext("./disabled") == "false" and
            {v.findtext("name") for v in job.findall(
                "./properties/hudson.model.ParametersDefinitionProperty/parameterDefinitions/*")} == set(parameters),
            "Records HTTP receiver is disabled or its parameters differ")


def binding(repo, *, transport_only=False):
    api = entry()
    context = transport_context(repo, api) if transport_only else api.capability(repo)
    installed, _ = read_json(Path(repo) / ".jenkins/state/records-queue/installed.json")
    require(installed["job"] == JOB and installed["driverDigest"] == context["config"]["driverDigest"]
            and installed["runtimeOperationId"] == context["config"]["runtimeOperationId"],
            "Records intake is bound to another formal runtime")
    for name, expected in installed["sources"].items():
        require(sha(plain(HERE / name).read_bytes()) == expected, "Pinned records entry source changed")
    if transport_only:
        verify_transport_job(context, installed, "records_queue.py", "zircon-records.groovy",
                             ("INVENTORY_DIGEST", "TASK_ID", "MODE"))
    return api, context, installed


def validate_inventory(repo, inventory):
    root = plain(Path(repo) / ".jenkins/records")
    require(inventory.get("schemaVersion") == 1 and inventory.get("authorization") ==
            "user: every 30 minutes; enqueue all tasks, archive evidence, then empty inbox",
            "Current enqueue and cleanup authorization must be explicit")
    seen, identities = set(), set()
    require(0 < len(inventory["tasks"]) <= 128, "A bounded nonempty task inventory is required")
    for task in inventory["tasks"]:
        require(ID.fullmatch(task["taskId"]) and task["taskId"] not in identities, "Duplicate or invalid task identity")
        identities.add(task["taskId"])
        require(task["kind"] in {"task", "historical-evidence"} and task.get("messages"), "Task kind/messages required")
        require(task.get("originalIdentity") and isinstance(task.get("pendingRequirements"), list),
                "Original identity and open requirements must be preserved")
        for message in task["messages"]:
            path = plain(message["path"])
            require(path.parent == root and path.suffix.lower() in {".md", ".json"}
                    and str(path).casefold() not in seen and SHA.fullmatch(message["sha256"]),
                    "Message must be a unique exact ordinary inbox leaf")
            seen.add(str(path).casefold())
        for payload in task.get("formalRequests", []):
            require(payload.get("action") == "validate", "Intake delegates only scoped validation")
            require(payload["sessionId"] == task["originalIdentity"]["sessionId"], "Original session must be retained")
    return inventory



OUTCOME_STATUSES = {"input-provided", "route-ready", "pending", "execution-failed"}


def outcome_provider(context):
    return {**{name: context["config"][name] for name in ("driverDigest", "runtimeOperationId")},
            "capsuleSha256": context["installed"]["capsuleSha256"]}


def validate_requirement_outcome(context, value, original):
    """Bind operator observations to existing requirements; never accept a task."""
    if value is None:
        return None
    require(isinstance(value, dict), "Requirement outcome schema differs")
    version = value.get("schemaVersion")
    fields = {"schemaVersion", "taskId", "originalInventoryDigest", "originalIdentity",
              "requirementsDigest", "provider", "outcomes", "formalAcceptance"}
    if version == 2:
        fields.add("requirementsSnapshot")
    require(type(version) is int and version in {1, 2} and set(value) == fields,
            "Requirement outcome schema differs")
    # A queued observation covers its exact earlier requirements. Later
    # append-only supplements do not invalidate those still-unchanged items.
    # Legacy observations keep their exact-current-inventory contract.
    covered = value.get("requirementsSnapshot", original["pendingRequirements"])
    if version == 2:
        require(isinstance(covered, list) and 0 < len(covered) <= 128
                and all(isinstance(text, str) and text for text in covered),
                "Explicit bounded requirement snapshot required")
    require(value["formalAcceptance"] is False
            and value["taskId"] == original["taskId"]
            and value["originalInventoryDigest"] == original["inventoryDigest"]
            and value["originalIdentity"] == original["originalIdentity"]
            and value["requirementsDigest"] == sha(encoded(covered))
            and (version == 1 or original["pendingRequirements"][:len(covered)] == covered)
            and value["provider"] == outcome_provider(context),
            "Requirement outcome original identity, requirements or provider changed")
    entries = value["outcomes"]
    require(isinstance(entries, list) and 0 < len(entries) <= 128, "Bounded requirement outcomes required")
    seen = set()
    for item in entries:
        require(isinstance(item, dict) and set(item) == {
            "requirementText", "requirementDigest", "status", "note", "evidenceRefs"},
            "Requirement outcome item schema differs")
        text = item["requirementText"]
        require(isinstance(text, str) and text in covered
                and item["requirementDigest"] == sha(text.encode("utf-8"))
                and item["requirementDigest"] not in seen
                and item["status"] in OUTCOME_STATUSES
                and isinstance(item["note"], str) and 0 < len(item["note"]) <= 2000,
                "Outcome must name one original requirement and a nonacceptance status")
        seen.add(item["requirementDigest"])
        refs = item["evidenceRefs"]
        require(isinstance(refs, list) and 0 < len(refs) <= 8, "Exact outcome evidence required")
        route_proven = False
        for reference in refs:
            require(isinstance(reference, dict) and set(reference) == {"path", "sha256"}
                    and isinstance(reference["sha256"], str) and SHA.fullmatch(reference["sha256"]),
                    "Outcome evidence reference differs")
            path = plain(reference["path"])
            roots = [context["repo"] / ".jenkins/state/management",
                     context["repo"] / ".jenkins/state/records-queue",
                     *(Path(d + ":/cargo-targets/zircon-local") for d in "DEF")]
            require(any(path.is_relative_to(plain(root)) and path != plain(root) for root in roots),
                    "Outcome evidence must be retained management or managed artifact evidence")
            pin, raw = fingerprint(path)
            require(pin["sha256"] == reference["sha256"], "Requirement outcome evidence changed")
            if item["status"] == "route-ready" and path.suffix == ".json":
                try:
                    proof = json.loads(raw)
                except (ValueError, UnicodeError):
                    continue
                if (isinstance(proof, dict) and proof.get("status") == "ready"
                        and all(proof.get(k) == v for k, v in value["provider"].items())
                        and proof.get("formalAcceptance") is False
                        and isinstance(proof.get("regression"), dict)):
                    regression, regression_pin = read_json(proof["regression"]["path"])
                    require(regression_pin["sha256"] == proof["regression"]["sha256"],
                            "Route regression evidence changed")
                    archive = regression.get("archiveByteVerification")
                    route_proven = (regression.get("jenkinsResult") == "SUCCESS"
                                    and regression.get("nativeProofComplete") is True
                                    and regression.get("sourceHashesMatch") is True
                                    and regression.get("sameNativeOwner") is True
                                    and isinstance(archive, list) and bool(archive)
                                    and all(x.get("archiveMatches") is True for x in archive))
        require(item["status"] != "route-ready" or route_proven,
                "Route readiness requires actual tested deployment evidence")
    return value


def attach_requirement_outcome(current, value, *, followup_digest, response):
    if value is None:
        return
    current.setdefault("requirementOutcomeHistory", []).append({
        "followupDigest": followup_digest, "response": response, "projection": value})
    latest = current.setdefault("currentRequirementOutcomes", {})
    for item in value["outcomes"]:
        latest[item["requirementDigest"]] = {
            **item, "provider": value["provider"], "followupDigest": followup_digest,
            "response": response, "formalAcceptance": False}


def task_outcome_view(context, record):
    """Expose fresh evidence separately from append-only historical requirements."""
    if record is None:
        return None
    original = record["payload"]
    latest = original.get("currentRequirementOutcomes", {})
    outcomes = []
    for text in dict.fromkeys(original.get("pendingRequirements", [])):
        key = sha(text.encode("utf-8"))
        item = latest.get(key)
        status, note = "pending", "No current operator outcome has been attached"
        refs = []
        if item:
            status, note, refs = item["status"], item["note"], item["evidenceRefs"]
            try:
                current = item["provider"] == outcome_provider(context)
                current = current and all(fingerprint(ref["path"])[0]["sha256"] == ref["sha256"] for ref in refs)
            except (OSError, ValueError):
                current = False
            if not current:
                status, note = "stale-evidence", "Provider or referenced evidence changed; reconcile this requirement"
        outcomes.append({"requirementText": text, "requirementDigest": key, "status": status,
                         "note": note, "evidenceRefs": refs, "formalAcceptance": False})
    view = {"schemaVersion": 1, "provider": outcome_provider(context),
            "requirements": outcomes, "counts": {status: sum(x["status"] == status for x in outcomes)
             for status in sorted({x["status"] for x in outcomes})},
            "originalDone": original.get("done", False), "formalAcceptance": False}
    return {**record, "payload": {**original, "currentOutcomeView": view}}

def _template_text(raw):
    # Match read_text's newline conversion without changing source bytes.
    return raw.decode("utf-8").replace("\r\n", "\n").replace("\r", "\n")


def render_job_source(context, installed, script_name, template_name, *, previous=False):
    pin, raw = fingerprint(HERE / template_name)
    expected = installed["sources"][template_name]
    source = _template_text(raw)
    require(pin["sha256"] == expected or (previous and sha(source.encode("utf-8")) == expected),
            "Pinned intake job template changed")
    values = {"REPO": str(context["repo"]),
              "PYTHON": str(context["repo"] / ".jenkins/runtime/python/python.exe"),
              "SCRIPT": str(HERE / script_name), "SOURCE_SHA": installed["sources"][script_name],
              "DRIVER": installed["driverDigest"], "RUNTIME": installed["runtimeOperationId"]}
    for name, value in values.items():
        require(isinstance(value, str) and not any(c in value for c in ("\n", "\r", "'", '"', "%")),
                "Unsafe fixed job value")
        source = source.replace("@@" + name + "@@", value.replace("\\", "/"))
    require("@@" not in source, "Unresolved intake job marker")
    return source


def job_script(raw):
    import xml.etree.ElementTree as ET
    root = ET.fromstring(raw)
    definition = root.find("definition")
    require(root.tag == "flow-definition" and definition is not None
            and definition.get("class") == "org.jenkinsci.plugins.workflow.cps.CpsFlowDefinition"
            and definition.findtext("sandbox") == "true", "Existing intake job owner or sandbox changed")
    source = definition.findtext("script")
    require(isinstance(source, str) and source, "Existing intake job script is missing")
    return source


def require_job_idle(context, job):
    manager = context["manager"]
    status, raw = manager._request("job/" + job + "/api/json?tree=builds[building,result]")
    require(status == 200, "Cannot inspect intake job builds")
    builds = json.loads(raw).get("builds")
    require(isinstance(builds, list) and all(isinstance(build, dict) and build.get("building") is False
            and build.get("result") in {"SUCCESS", "UNSTABLE", "FAILURE", "NOT_BUILT", "ABORTED"}
            for build in builds), "Do not update an intake job with a nonterminal build")
    status, raw = manager._request("queue/api/json?tree=items[task[name,url]]")
    require(status == 200, "Cannot inspect queued intake work")
    items = json.loads(raw).get("items")
    require(isinstance(items, list), "Queued intake work is unproven")
    job_url = manager.base_url.rstrip("/") + "/job/" + job + "/"
    for item in items:
        task = item.get("task") if isinstance(item, dict) else None
        require(isinstance(task, dict) and isinstance(task.get("name"), str)
                and bool(task["name"]) and isinstance(task.get("url"), str) and bool(task["url"]),
                "Queued intake work is unproven")
        require(task["name"] != job and task["url"] != job_url, "Do not update an intake job with queued work")


def verify_install_upgrade(context, previous, installed, old_source, *, source_names, script_name,
                           template_name, upgrade_from, expected_job_script_sha256, rebind_runtime):
    require(previous is not None and isinstance(upgrade_from, str) and SHA.fullmatch(upgrade_from)
            and previous[1]["sha256"] == upgrade_from
            and isinstance(expected_job_script_sha256, str) and SHA.fullmatch(expected_job_script_sha256)
            and sha(old_source.encode("utf-8")) == expected_job_script_sha256,
            "Existing intake upgrade requires exact previous installation and job script pins")
    old = previous[0]
    require(old.get("schemaVersion") == 1 and old.get("job") == installed["job"]
            and old.get("repositoryRoot") == str(context["repo"])
            and old.get("intakeOnlyAcceptance") is True and old.get("formalAcceptance") is False
            and isinstance(old.get("driverDigest"), str) and SHA.fullmatch(old["driverDigest"])
            and isinstance(old.get("runtimeOperationId"), str) and ID.fullmatch(old["runtimeOperationId"])
            and isinstance(old.get("sources"), dict) and set(old["sources"]) == set(source_names)
            and all(isinstance(value, str) and SHA.fullmatch(value) for value in old["sources"].values()),
            "Existing intake installation belongs to another owner or source inventory")
    same_runtime = all(old[k] == installed[k] for k in ("driverDigest", "runtimeOperationId"))
    require(same_runtime or rebind_runtime is True, "Cross-runtime intake replacement requires --rebind-runtime")
    require(old_source == render_job_source(context, old, script_name, template_name, previous=True),
            "Existing intake job differs from its exact pinned prior template")
    require_job_idle(context, installed["job"])
    return {"previous": old, "previousInstallationSha256": upgrade_from,
            "previousJobScriptSha256": expected_job_script_sha256,
            "runtimeRebound": not same_runtime, "next": installed}


def recheck_install_inputs(api, repo, context, previous, installed, installed_path):
    fresh = api.capability(repo)
    require(fresh["repo"] == context["repo"] and all(fresh["config"][key] == installed[key]
            for key in ("driverDigest", "runtimeOperationId")), "Selected intake runtime changed during installation")
    for name, expected in installed["sources"].items():
        require(fingerprint(HERE / name)[0]["sha256"] == expected, "Intake source changed during installation")
    if previous:
        require(read_json(installed_path)[1]["sha256"] == previous[1]["sha256"],
                "Previous intake installation changed during upgrade")
    else:
        require(not installed_path.exists(), "Intake installation appeared during creation")


def install(repo, *, upgrade_from=None, expected_job_script_sha256=None, rebind_runtime=False):
    api = entry()
    context = api.capability(repo)
    source_pins = {name: fingerprint(HERE / name)[0]["sha256"] for name in SOURCES}
    installed = {"schemaVersion": 1, "job": JOB, "repositoryRoot": str(context["repo"]),
                 "driverDigest": context["config"]["driverDigest"],
                 "runtimeOperationId": context["config"]["runtimeOperationId"], "sources": source_pins,
                 "intakeOnlyAcceptance": True, "formalAcceptance": False}
    source = render_job_source(context, installed, "records_queue.py", "zircon-records.groovy")
    parameters = "".join("<hudson.model.StringParameterDefinition><name>" + name +
                         "</name><defaultValue></defaultValue><trim>true</trim></hudson.model.StringParameterDefinition>"
                         for name in ("INVENTORY_DIGEST", "TASK_ID", "MODE"))
    xml = ("<?xml version='1.0' encoding='UTF-8'?><flow-definition><description>" +
           "Hash-bound handoff intake and deferred formal validation; intake is not test acceptance" +
           "</description><properties><hudson.model.ParametersDefinitionProperty><parameterDefinitions>" +
           parameters + "</parameterDefinitions></hudson.model.ParametersDefinitionProperty></properties>" +
           "<definition class='org.jenkinsci.plugins.workflow.cps.CpsFlowDefinition' plugin='workflow-cps'><script>" +
           escape(source) + "</script><sandbox>true</sandbox></definition><triggers/><disabled>false</disabled></flow-definition>")
    manager = context["manager"]
    installed_path = context["repo"] / ".jenkins/state/records-queue/installed.json"
    previous = read_json(installed_path) if installed_path.exists() else None
    if rebind_runtime:
        require(previous is not None and previous[1]["sha256"] == upgrade_from
                and isinstance(expected_job_script_sha256, str) and SHA.fullmatch(expected_job_script_sha256),
                "Runtime rebind requires an exactly pinned existing intake installation")
    status, old = manager._request("job/" + JOB + "/config.xml")
    if status == 200:
        old_source = job_script(old)
        if old_source != source or (previous and previous[0] != installed):
            history = verify_install_upgrade(context, previous, installed, old_source, source_names=SOURCES,
                script_name="records_queue.py", template_name="zircon-records.groovy", upgrade_from=upgrade_from,
                expected_job_script_sha256=expected_job_script_sha256, rebind_runtime=rebind_runtime)
            recheck_install_inputs(api, repo, context, previous, installed, installed_path)
            write_once(installed_path.parent / "install-history" / (upgrade_from + ".json"), encoded(history) + b"\n")
            status, raw = manager._request("crumbIssuer/api/json")
            require(status == 200, "Jenkins crumb is unavailable")
            crumb = json.loads(raw)
            require_job_idle(context, JOB)
            recheck_install_inputs(api, repo, context, previous, installed, installed_path)
            status, raw = manager._request("job/" + JOB + "/config.xml")
            require(status == 200 and job_script(raw) == old_source, "Pinned prior intake job changed before update")
            status, _ = manager._request("job/" + JOB + "/config.xml", method="POST", data=xml.encode("utf-8"),
                                        headers={"Content-Type": "application/xml", crumb["crumbRequestField"]: crumb["crumb"]})
            require(status == 200, "Pinned records job update was not accepted")
            status, raw = manager._request("job/" + JOB + "/config.xml")
            require(status == 200 and job_script(raw) == source, "Updated records job requires reconciliation")
    else:
        require(status == 404 and previous is None and not rebind_runtime
                and upgrade_from is None and expected_job_script_sha256 is None,
                "Cannot create or replace an unproven existing records job")
        recheck_install_inputs(api, repo, context, previous, installed, installed_path)
        status, raw = manager._request("crumbIssuer/api/json")
        require(status == 200, "Jenkins crumb is unavailable")
        crumb = json.loads(raw)
        status, _ = manager._request("createItem?name=" + JOB, method="POST", data=xml.encode("utf-8"),
                                    headers={"Content-Type": "application/xml", crumb["crumbRequestField"]: crumb["crumb"]})
        require(status == 200, "Jenkins did not create the fixed intake job")
        status, raw = manager._request("job/" + JOB + "/config.xml")
        require(status == 200 and job_script(raw) == source, "Created records job requires reconciliation")
    recheck_install_inputs(api, repo, context, previous, installed, installed_path)
    if previous and previous[0] != installed:
        from tools.jenkins.state.atomic import atomic_json
        atomic_json(installed_path, installed)
    elif previous is None:
        write_once(installed_path, encoded(installed) + b"\n")
    return installed


def submit(repo, inventory_file, *, mode="task", reconcile=False):
    api, context, installed = binding(repo, transport_only=True)
    inventory, pin = read_json(inventory_file)
    validate_inventory(repo, inventory)
    digest = sha(encoded(inventory))
    root = context["repo"] / ".jenkins/state/records-queue"
    write_once(root / "inputs" / (digest + ".json"), encoded(inventory) + b"\n")
    receipts = []
    from tools.jenkins.state.atomic import atomic_json
    from tools.jenkins.state.locks import process_lock
    with process_lock(root / "submission.lock", reason="records_intake_busy"):
        for task in inventory["tasks"]:
            task_id = task["taskId"]
            marker = root / "submissions" / (task_id + ".json")
            if marker.exists():
                old, _ = read_json(marker)
                require(old["inventoryDigest"] == digest, "Reconcile the original task before admitting a revision")
                if not reconcile:
                    receipts.append(old)
                    continue
                original = api.record(context["repo"], "records_queue_task", task_id)
                require(original and original["payload"]["inventoryDigest"] == digest,
                        "Original durable task is required for reconciliation")
                prior_build = original["payload"]["buildNumber"]
                status, raw = context["manager"]._request("job/" + JOB + "/" + prior_build + "/api/json?tree=building,result,url")
                prior = json.loads(raw) if status == 200 else {}
                require(prior.get("building") is False and prior.get("result") == "FAILURE" and
                        prior.get("url") == original["payload"]["buildUrl"], "Original worker must have a known failure before reconciliation")
                marker = root / "reconciliations" / task_id / (prior_build + ".json")
                if marker.exists():
                    receipts.append(read_json(marker)[0])
                    continue
                mode = "reconcile"
            else:
                require(not reconcile, "There is no original dispatch to reconcile")
                old, prior = None, None
            original = api.record(context["repo"], "records_queue_task", task_id)
            require(reconcile or original is None, "Task already belongs to another intake; reconcile its original identity")
            receipt = {"taskId": task_id, "inventoryDigest": digest, "phase": "dispatching", "queueUrl": None,
                       "originalIdentity": task["originalIdentity"], "driverDigest": installed["driverDigest"],
                       "runtimeOperationId": installed["runtimeOperationId"], "attempt": 1,
                       "formalAcceptance": False, "submittedAt": time.time()}
            if reconcile:
                receipt.update(reconciliationOf=prior["url"], originalDelivery=old,
                               attempt=len(original["payload"].get("receiverHistory", [])) + 2)
            write_once(marker, encoded(receipt) + b"\n")
            try:
                status, raw = context["manager"]._request("crumbIssuer/api/json")
                require(status == 200, "Jenkins crumb is unavailable")
                crumb = json.loads(raw)
                data = urllib.parse.urlencode({"INVENTORY_DIGEST": digest, "TASK_ID": task_id, "MODE": mode}).encode("ascii")
                status, _body, headers = context["manager"]._request_with_headers(
                    "job/" + JOB + "/buildWithParameters", method="POST", data=data,
                    headers={"Content-Type": "application/x-www-form-urlencoded",
                             crumb["crumbRequestField"]: crumb["crumb"]}, timeout=30)
                queue = next((v for k, v in headers.items() if k.lower() == "location"), None)
                require(status == 201 and queue and queue.startswith(context["manager"].base_url.rstrip("/") + "/queue/item/"),
                        "Delivery acknowledgement is unknown")
                receipt.update(phase="queued", queueUrl=queue)
            except Exception as error:
                receipt.update(phase="delivery_unknown", error=str(error))
            atomic_json(marker, receipt)
            receipts.append(receipt)
    return {"inventoryDigest": digest, "receipts": receipts, "formalAcceptance": False}


def archive_messages(repo, root, task, digest):
    retained = []
    for message in task["messages"]:
        source = plain(message["path"])
        target = root / "archive" / digest / source.name
        if source.exists():
            actual, raw = fingerprint(source)
            require(all(actual[name] == message[name] for name in ("path", "bytes", "mtimeNs", "sha256")),
                    "Inbox message changed; leave it for reconciliation")
            require(time.time_ns() - actual["mtimeNs"] >= 30_000_000_000, "Message may still be being written")
            write_once(target, raw)
        saved, _ = fingerprint(target)
        require(saved["sha256"] == message["sha256"] and saved["bytes"] == message["bytes"],
                "Durable archive differs from the admitted message")
        retained.append({**message, "archivePath": str(target)})
    return retained


def cleanup(context, installed, root, task_id, digest, retained):
    control = root / "cleanup" / (task_id + ".json")
    receipt = root / "cleanup" / (task_id + ".receipt.json")
    if receipt.exists():
        result, receipt_pin = read_json(receipt)
        if result["success"]:
            return result
        # A failure while writing the final accounting can follow a completely
        # proven native deletion. Reconcile that evidence, never repeat a delete
        # or overwrite the failed receipt. Partial or unproven deletion stays open.
        require(result.get("taskId") == task_id and result.get("inventoryDigest") == digest,
                "Failed cleanup receipt belongs to another task/input")
        declared = {(m["path"].casefold(), m["bytes"], m["sha256"]) for m in retained}
        deleted = {(m["path"].casefold(), m["bytes"], m["sha256"]) for m in result.get("deletedFiles", [])}
        require(declared == deleted and len(result.get("deletedFiles", [])) == len(retained),
                "Original native deletion evidence is incomplete; preserve the exact failed receipt")
        archive_proofs = []
        for message in retained:
            require(not plain(message["path"]).exists(), "A deleted inbox path has new content; preserve it")
            archive, _ = fingerprint(message["archivePath"])
            require(archive["sha256"] == message["sha256"] and archive["bytes"] == message["bytes"],
                    "Retained contract evidence changed after deletion")
            archive_proofs.append(archive)
        reconciled = {**result, "success": True, "deletedBytes": sum(m["bytes"] for m in retained),
                      "originalFailureReceipt": receipt_pin, "archiveProofs": archive_proofs,
                      "reconciledByBuildUrl": os.environ["BUILD_URL"], "reconciledAfterReceiptFailure": True}
        reconciled_path = receipt.with_name(task_id + ".receipt.reconciled.json")
        if reconciled_path.exists():
            previous, _ = read_json(reconciled_path)
            require(previous["originalFailureReceipt"] == receipt_pin and previous["archiveProofs"] == archive_proofs,
                    "Reconciled cleanup evidence changed")
            return previous
        write_once(reconciled_path, encoded(reconciled) + b"\n")
        return reconciled
    config = context["repo"] / ".codex/config.toml"
    helper = Path("D:/CodexData/.codex/skills/artifact-lifecycle/scripts/GuardedFileDelete.cs")
    plan = {"taskId": task_id, "inventoryDigest": digest, "repositoryRoot": str(context["repo"]),
            "authorization": "user: enqueue all tasks, archive evidence, then empty inbox",
            "permissionConfigSha256": sha(plain(config).read_bytes()),
            "helperSha256": sha(plain(helper).read_bytes()), "files": retained}
    write_once(control, encoded(plan) + b"\n")
    command = ["powershell.exe", "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "RemoteSigned",
               "-File", str(HERE / "records_cleanup.ps1"),
               "-PlanPath", str(control), "-ReceiptPath", str(receipt), "-ExpectedScriptSha256",
               installed["sources"]["records_cleanup.ps1"]]
    proc = subprocess.run(command, cwd=context["repo"], capture_output=True, timeout=120)
    require(proc.returncode == 0 and receipt.exists(), "Guarded Jenkins cleanup failed: " +
            proc.stderr.decode("utf-8", errors="replace")[-1500:])
    result, _ = read_json(receipt)
    require(result["success"], "Guarded cleanup remains incomplete")
    return result


def work(repo, task_id, digest, output_file):
    api, context, installed = binding(repo)
    require(ID.fullmatch(task_id) and SHA.fullmatch(digest), "Invalid task or inventory identity")
    require(os.environ.get("JOB_NAME") == JOB and os.environ.get("BUILD_URL") and os.environ.get("BUILD_NUMBER"),
            "Only the fixed Jenkins worker may archive messages or clean the inbox")
    root = context["repo"] / ".jenkins/state/records-queue"
    output = plain(output_file)
    require(output.parent == root / "controls", "Worker outputs must stay in the fixed control directory")
    inventory, _ = read_json(root / "inputs" / (digest + ".json"))
    validate_inventory(repo, inventory)
    require(sha(encoded(inventory)) == digest, "Inventory bytes changed")
    task = next(t for t in inventory["tasks"] if t["taskId"] == task_id)
    status, raw = context["manager"]._request("job/" + JOB + "/" + os.environ["BUILD_NUMBER"] +
        "/api/json?tree=url,building,queueId,actions[parameters[name,value]]")
    require(status == 200, "Current Jenkins build cannot be verified")
    live = json.loads(raw)
    params = {p["name"]: p.get("value") for action in live.get("actions", []) for p in action.get("parameters", [])}
    require(live["url"] == os.environ["BUILD_URL"] and live["building"] is True and
            params.get("INVENTORY_DIGEST") == digest and params.get("TASK_ID") == task_id and params.get("MODE") in {"task", "reconcile"},
            "Actual Jenkins build differs from the original intake parameters")
    from tools.jenkins.state import State
    from tools.jenkins.state.atomic import atomic_json
    from tools.jenkins.state.locks import process_lock
    state = State(context["repo"] / ".jenkins/state/coordination.sqlite3")
    with process_lock(root / "locks" / (task_id + ".lock"), reason="records_task_busy"):
        previous = state.get("records_queue_task", task_id)
        current = previous["payload"] if previous else None
        if current:
            require(current["inventoryDigest"] == digest, "Original task contract changed")
            if current["buildUrl"] != os.environ["BUILD_URL"]:
                require(params["MODE"] == "reconcile", "This task belongs to its original build; do not duplicate it")
                status, raw = context["manager"]._request("job/" + JOB + "/" + current["buildNumber"] + "/api/json?tree=building,result,url")
                prior = json.loads(raw) if status == 200 else {}
                require(prior.get("building") is False and prior.get("result") == "FAILURE" and
                        prior.get("url") == current["buildUrl"], "Original worker outcome requires reconciliation")
                recovery, _ = read_json(root / "reconciliations" / task_id / (current["buildNumber"] + ".json"))
                require(recovery["inventoryDigest"] == digest and recovery["reconciliationOf"] == current["buildUrl"]
                        and recovery["phase"] == "queued" and recovery["queueUrl"].rstrip('/').endswith('/' + str(live["queueId"])),
                        "Original recovery queue acknowledgement differs")
                current.setdefault("receiverHistory", []).append({"buildUrl": current["buildUrl"],
                    "buildNumber": current["buildNumber"], "result": "FAILURE", "reason": "See original Jenkins build log"})
                current.update(buildUrl=os.environ["BUILD_URL"], buildNumber=os.environ["BUILD_NUMBER"], recoveryReceipt=recovery)
                state.put("records_queue_task", task_id, current, expected_version=previous["version"])
        else:
            marker, _ = read_json(root / "submissions" / (task_id + ".json"))
            require(marker["inventoryDigest"] == digest and marker["phase"] == "queued" and
                    marker["queueUrl"].rstrip('/').endswith('/' + str(live["queueId"])),
                    "Original durable matching queue acknowledgement is missing")
            retained = archive_messages(repo, root, task, digest)
            current = {"taskId": task_id, "inventoryDigest": digest, "originalIdentity": task["originalIdentity"],
                       "queueReceipt": marker, "buildUrl": os.environ["BUILD_URL"], "buildNumber": os.environ["BUILD_NUMBER"],
                       "driverDigest": installed["driverDigest"], "runtimeOperationId": installed["runtimeOperationId"],
                       "phase": "accepted", "intakeAccepted": True, "messages": retained,
                       "pendingRequirements": task["pendingRequirements"], "formalRequests": task.get("formalRequests", []),
                       "formalReceipts": [], "formalAcceptance": False, "acceptedAt": time.time()}
            state.put("records_queue_task", task_id, current, expected_version=0)
            for message in retained:
                state.put("records_queue_message", sha(message["path"].casefold().encode()),
                          {**message, "taskId": task_id, "inventoryDigest": digest, "buildUrl": current["buildUrl"]})
        current["cleanup"] = cleanup(context, installed, root, task_id, digest, current["messages"])
        receipts = []
        for payload in current["formalRequests"]:
            # Same identities and strict ordinary parser on every poll. No local
            # execution, raw command runner, source copy, or altered selectors.
            try:
                receipt = api.dispatch(context["repo"], payload)
            except Exception as error:
                receipt = {"status": "pending", "reasonCode": getattr(error, "code", "formal_input_pending"),
                           "message": str(error), "requestId": payload["requestId"], "operationId": payload["operationId"],
                           "formalAcceptance": False}
            receipts.append(receipt)
            # Preserve ordering and reconcile an unknown or running predecessor.
            if receipt.get("phase") != "complete":
                break
            if (receipt.get("result") or {}).get("status") not in {"complete", "passed"}:
                break
        current.update(formalReceipts=receipts, phase="waiting" if task["kind"] == "task" else "evidence-archived",
                       lastObservedAt=time.time(), done=task["kind"] == "historical-evidence")
        old = state.get("records_queue_task", task_id)
        state.put("records_queue_task", task_id, current, expected_version=old["version"])
        atomic_json(output, current)
    return current


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("install", "submit", "work", "query", "self-test"))
    parser.add_argument("--repo-root", required=True)
    parser.add_argument("--inventory-file")
    parser.add_argument("--task-id")
    parser.add_argument("--inventory-digest")
    parser.add_argument("--output-file")
    parser.add_argument("--expected-source-sha256")
    parser.add_argument("--mode", choices=("task", "self-test"), default="task")
    parser.add_argument("--reconcile", action="store_true")
    parser.add_argument("--upgrade-from")
    parser.add_argument("--expected-job-script-sha256")
    parser.add_argument("--rebind-runtime", action="store_true")
    args = parser.parse_args(argv)
    try:
        if args.expected_source_sha256:
            require(sha(Path(__file__).read_bytes()) == args.expected_source_sha256, "Executing intake source changed")
        if args.command == "install":
            result = install(args.repo_root, upgrade_from=args.upgrade_from,
                             expected_job_script_sha256=args.expected_job_script_sha256,
                             rebind_runtime=args.rebind_runtime)
        elif args.command == "submit":
            result = submit(args.repo_root, args.inventory_file, mode=args.mode, reconcile=args.reconcile)
        elif args.command == "work":
            result = work(args.repo_root, args.task_id, args.inventory_digest, args.output_file)
        elif args.command == "self-test":
            _, _, installed = binding(args.repo_root)
            require(os.environ.get("JOB_NAME") == JOB, "Run operational tests through the fixed Jenkins job")
            subprocess.run([sys.executable, "-B", "-X", "utf8", str(HERE / "tests/test_records_queue.py")], check=True)
            subprocess.run(["powershell.exe", "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "RemoteSigned",
                            "-File", str(HERE / "records_cleanup.ps1"), "-Probe", "-ExpectedScriptSha256",
                            installed["sources"]["records_cleanup.ps1"]], check=True)
            result = {"status": "passed", "nativeGuardVerified": True, "formalAcceptance": False}
        else:
            api, context, _ = binding(args.repo_root, transport_only=True)
            result = task_outcome_view(context, api.record(context["repo"], "records_queue_task", args.task_id))
        print(encoded(result).decode())
        if args.command == "work" and not result.get("done"):
            # Distinguish durable waiting from failure without requiring any
            # additional Jenkins sandbox parser/constructor approval.
            return 10
        return 0
    except Exception as error:
        print(encoded({"status": "pending", "message": str(error), "formalAcceptance": False}).decode())
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
