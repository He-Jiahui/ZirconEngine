from __future__ import annotations

import base64
import json
import urllib.request
import urllib.parse
import urllib.error
import hashlib
from pathlib import Path
from xml.sax.saxutils import escape

from .spec import JenkinsError
from .driver import verify_driver, runtime_agent_label


def _driver_snapshot(paths) -> tuple[Path, str]:
    record = json.loads((paths.state / "deployment" / "driver.json").read_text(encoding="utf-8"))
    root = Path(record["root"])
    digest = str(record["driverDigest"])
    verify_driver(root, expected_digest=digest)
    return root, digest


def _pipeline_source(paths, name: str, snapshot: tuple[Path, str] | None = None) -> str:
    root, _ = snapshot or _driver_snapshot(paths)
    source = root / "pipeline" / f"{name}.groovy"
    if not source.is_file():
        raise JenkinsError("pipeline_source_missing", f"pipeline source is missing: {source}")
    return source.read_text(encoding="utf-8")


def _config(source: str, driver_env: dict[str, str]) -> str:
    # CPS definition is embedded as XML text; no credentials or secrets are
    # passed as build parameters.  The source is copied from the repository
    # specification and sealed by the deployment operation digest.
    required_runtime = ("ZIRCON_AGENT_LABEL", "ZIRCON_RUNTIME_OPERATION_ID", "ZIRCON_DRIVER_DIGEST")
    if any(not isinstance(driver_env.get(key), str) or not driver_env[key] for key in required_runtime):
        raise JenkinsError("runtime_binding_missing", "job configuration requires runtime label, operation and driver digest")
    # The selected deployment supplies defaults only when a dispatch omits
    # its runtime fences. Explicit values still reach the pipeline guard.
    runtime_defaults = {
        "STAGE_IMPLEMENTATION_DIGEST": driver_env["ZIRCON_DRIVER_DIGEST"],
        "RUNTIME_OPERATION_ID": driver_env["ZIRCON_RUNTIME_OPERATION_ID"],
    }
    params = "".join(f"<hudson.model.StringParameterDefinition><name>{name}</name><description>Jenkins request identity</description><defaultValue>{escape(runtime_defaults.get(name, ''))}</defaultValue><trim>true</trim></hudson.model.StringParameterDefinition>" for name in (
        "REPOSITORY_ID", "SESSION_ID", "REQUEST_ID", "ATTEMPT_ID", "GENERATION", "SOURCE_INPUT_DIGEST", "COVERAGE_DIGEST", "STAGE_IMPLEMENTATION_DIGEST", "RUNTIME_OPERATION_ID", "EXECUTION_ID", "RECIPE_REF", "SEALED_INPUT_REF", "PATCH_OPERATION_REF", "PATCH_REQUEST_REF", "CURRENT_SOURCE_REF", "SOURCE_PREPARATION_REF", "CHANGE_SET_JSON", "TEMPLATE", "RUST_EDITION", "SEALED_RUST_SOURCE", "COMMIT_AUTHORIZATION", "CLEANUP_SCOPE_JSON", "PROTECTION_REVIEW_JSON", "CACHE_KIND", "OPERATION_ID", "ALLOW_COMMIT", "ALLOW_NOTIFY", "NOTIFY_DESTINATION", "BUILD_ROOT"))
    prelude = "; ".join("env.%s=%s" % (key, json.dumps(value, ensure_ascii=False)) for key, value in {
        "ZIRCON_REPO_ROOT": driver_env["ZIRCON_REPO_ROOT"], "JENKINS_PYTHON": driver_env["JENKINS_PYTHON"],
        "ZIRCON_SEALED_DRIVER": driver_env["ZIRCON_SEALED_DRIVER"], "ZIRCON_DRIVER_LAUNCHER": driver_env["ZIRCON_DRIVER_LAUNCHER"],
        "ZIRCON_DRIVER_DIGEST": driver_env["ZIRCON_DRIVER_DIGEST"], "ZIRCON_REPOSITORY_ID": hashlib.sha256(driver_env["ZIRCON_REPO_ROOT"].casefold().encode()).hexdigest(),
        "PYTHONDONTWRITEBYTECODE": "1", "ZIRCON_BUILD_ROOT": driver_env["ZIRCON_BUILD_ROOT"],
        "ZIRCON_AGENT_LABEL": driver_env.get("ZIRCON_AGENT_LABEL", ""),
        "ZIRCON_RUNTIME_OPERATION_ID": driver_env.get("ZIRCON_RUNTIME_OPERATION_ID", ""),}.items()) + "; "
    return ("<?xml version='1.1' encoding='UTF-8'?>"
            "<flow-definition plugin='workflow-job'>"
            "<actions/><description>Repository-local Zircon Jenkins job</description>"
            "<keepDependencies>false</keepDependencies><properties><hudson.model.ParametersDefinitionProperty><parameterDefinitions>" + params + "</parameterDefinitions></hudson.model.ParametersDefinitionProperty></properties>"
            "<definition class='org.jenkinsci.plugins.workflow.cps.CpsFlowDefinition' plugin='workflow-cps'>"
            f"<script>{escape(prelude + source)}</script><sandbox>true</sandbox></definition>"
            "<triggers/><disabled>false</disabled></flow-definition>")


def provision_jobs(spec, paths, manager) -> dict:
    jobs = spec.raw.get("pipeline", {}).get("jobs", [])
    snapshot = _driver_snapshot(paths)
    driver_env = {**manager._driver_env(), 'ZIRCON_BUILD_ROOT': str(paths.build_root)}
    selected = json.loads((paths.state / "deployment" / "driver.json").read_text(encoding="utf-8"))
    runtime_operation_id = str(selected.get("runtimeOperationId") or "")
    if not runtime_operation_id:
        raise JenkinsError("runtime_binding_missing", "job provisioning requires selected runtime operation")
    driver_env["ZIRCON_RUNTIME_OPERATION_ID"] = runtime_operation_id
    driver_env["ZIRCON_AGENT_LABEL"] = runtime_agent_label(str(spec.agent["name"]), runtime_operation_id)
    created = []
    for name in jobs:
        source = _pipeline_source(paths, name, snapshot)
        payload = _config(source, driver_env).encode("utf-8")
        user, token = manager._credentials()
        auth = base64.b64encode(f"{user}:{token}".encode()).decode()
        base = manager.base_url.rstrip("/") + "/job/" + urllib.parse.quote(name, safe="")
        request = urllib.request.Request(base + "/config.xml", data=payload, method="POST",
                                         headers={"Content-Type": "application/xml", "Authorization": "Basic " + auth})
        try:
            with urllib.request.urlopen(request, timeout=10) as response:
                if response.status not in (200, 201, 302):
                    raise JenkinsError("job_provision_failed", f"Jenkins rejected job {name}")
        except urllib.error.HTTPError as exc:
            if exc.code != 404:
                if exc.code == 403:
                    _retry_with_crumb(manager, base + "/config.xml", payload, method="POST")
                    created.append(name)
                    continue
                else:
                    raise JenkinsError("job_provision_failed", f"cannot provision job {name}") from exc
            create = urllib.request.Request(manager.base_url.rstrip("/") + "/createItem?name=" + urllib.parse.quote(name),
                                             data=payload, method="POST", headers={"Content-Type": "application/xml", "Authorization": "Basic " + auth})
            try:
                urllib.request.urlopen(create, timeout=10).close()
            except Exception as create_exc:
                if isinstance(create_exc, urllib.error.HTTPError) and create_exc.code == 403:
                    _retry_with_crumb(manager, manager.base_url.rstrip("/") + "/createItem?name=" + urllib.parse.quote(name), payload, method="POST")
                else:
                    raise JenkinsError("job_provision_failed", f"cannot create job {name}") from create_exc
        except Exception as exc:
            if isinstance(exc, JenkinsError):
                raise
            raise JenkinsError("job_provision_failed", f"cannot provision job {name}") from exc
        created.append(name)
    return {"jobsProvisioned": created}


def _retry_with_crumb(manager, url: str, payload: bytes, *, method: str) -> None:
    import json
    user, token = manager._credentials()
    auth = base64.b64encode(f"{user}:{token}".encode()).decode()
    _, body = manager._request("crumbIssuer/api/json")
    crumb = json.loads(body)
    request = urllib.request.Request(url, data=payload, method=method,
        headers={"Content-Type": "application/xml", "Authorization": "Basic " + auth,
                 crumb["crumbRequestField"]: crumb["crumb"]})
    try:
        with urllib.request.urlopen(request, timeout=10) as response:
            if response.status not in (200, 201, 302):
                raise JenkinsError("job_provision_failed", "Jenkins rejected crumb-authenticated job request")
    except Exception as exc:
        if isinstance(exc, JenkinsError):
            raise
        raise JenkinsError("job_provision_failed", "crumb-authenticated job request failed") from exc


# Service controls use the existing runtime installers, never provision_jobs().
_SERVICE_REBIND_JOBS = {}
_LOADED_SERVICE_REBIND_SHA256 = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()


def _service_ref(path):
    from .independent_service import _raw
    raw = _raw(path)
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


def _service_ref_read(ref):
    from .independent_service import _raw, _require
    path = Path(ref["path"])
    raw = _raw(path)
    _require(ref == {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)},
             "Runtime rebind input changed", "service_rebind_pending")
    return raw


def _service_job(manager, name):
    from .independent_service import _raw, _require
    endpoint = "job/" + name + "/config.xml"
    status, raw = manager._request(endpoint)
    path = manager.paths.home / "jobs" / name / "config.xml"
    home = _raw(path)
    again_status, again = manager._request(endpoint)
    _require(status == again_status == 200 and raw == again and home == _raw(path)
             and _service_xml_shape(raw) == _service_xml_shape(home),
             "Runtime job API/Home changed: " + name, "service_rebind_pending")
    return {"name": name, "apiSha256": hashlib.sha256(raw).hexdigest(),
            "homeRef": {"path": str(path), "sha256": hashlib.sha256(home).hexdigest(), "bytes": len(home)},
            "xml": raw.decode("utf-8"), "homeXml": home.decode("utf-8")}


def _service_xml_shape(raw, *, bindings=False):
    import xml.etree.ElementTree as ET
    root = ET.fromstring(raw)
    actions = root.findall("actions")
    if len(actions) == 1 and not actions[0].attrib and not actions[0].text and len(actions[0]) == 0:
        root.remove(actions[0])
    if bindings:
        script = root.find("definition/script")
        if script is not None:
            script.text = "__CPS__"
        for param in root.findall(".//hudson.model.StringParameterDefinition"):
            if param.findtext("name") in {"RUNTIME_OPERATION_ID", "STAGE_IMPLEMENTATION_DIGEST", "EXPECTED_DRIVER_DIGEST"}:
                value = param.find("defaultValue")
                if value is not None:
                    value.text = "__RUNTIME__"
    def shape(node):
        text = node.text or ""
        return (node.tag, sorted(node.attrib.items()), text if text.strip() else "",
                tuple(shape(child) for child in node))
    return shape(root)


def _service_records_shape(raw, *, followup):
    """The official Records installers replace XML; reject foreign fields first."""
    import xml.etree.ElementTree as ET
    from .independent_service import _require
    root = ET.fromstring(raw)
    allowed = {"actions", "description", "keepDependencies", "properties", "definition", "triggers", "disabled"}
    _require(root.tag == "flow-definition" and set(root.attrib) <= {"plugin"}
             and len({n.tag for n in root}) == len(root)
             and all(n.tag in allowed for n in root),
             "Records has foreign top-level configuration", "service_rebind_pending")
    for key in ("actions", "triggers"):
        node = root.find(key)
        _require(node is None or (not node.attrib and not list(node) and not (node.text or "").strip()),
                 "Records has foreign actions or timers", "service_rebind_pending")
    _require(root.findtext("keepDependencies", "false") == "false"
             and root.findtext("disabled") == "false",
             "Records default lifecycle differs", "service_rebind_pending")
    description = ("Attach messages to an existing task; no native validation retry" if followup else
                   "Hash-bound handoff intake and deferred formal validation; intake is not test acceptance")
    _require(root.findtext("description") == description, "Records description differs", "service_rebind_pending")
    properties = root.find("properties")
    _require(properties is not None and not properties.attrib and len(properties) == 1
             and properties[0].tag == "hudson.model.ParametersDefinitionProperty"
             and not properties[0].attrib and len(properties[0]) == 1
             and properties[0][0].tag == "parameterDefinitions" and not properties[0][0].attrib,
             "Records has foreign properties", "service_rebind_pending")
    expected = ("FOLLOWUP_DIGEST", "TASK_ID", "MODE") if followup else ("INVENTORY_DIGEST", "TASK_ID", "MODE")
    parameters = list(properties[0][0])
    _require(len(parameters) == 3 and tuple(p.findtext("name") for p in parameters) == expected,
             "Records parameter inventory differs", "service_rebind_pending")
    for p in parameters:
        _require(p.tag == "hudson.model.StringParameterDefinition" and not p.attrib
                 and tuple(n.tag for n in p) == ("name", "defaultValue", "trim")
                 and all(not n.attrib and not list(n) for n in p)
                 and not p.findtext("defaultValue", "") and p.findtext("trim") == "true",
                 "Records has foreign parameter fields", "service_rebind_pending")
    definition = root.find("definition")
    _require(definition is not None and set(definition.attrib) <= {"class", "plugin"}
             and definition.get("class") == "org.jenkinsci.plugins.workflow.cps.CpsFlowDefinition"
             and tuple(n.tag for n in definition) == ("script", "sandbox")
             and definition.findtext("sandbox") == "true"
             and all(not n.attrib and not list(n) for n in definition),
             "Records has foreign definition fields", "service_rebind_pending")


def _service_command(manager, output, key, argv):
    """One suspended NativeJob per durable intent; unknown launches are only queried."""
    import os
    import time
    from ..state.atomic import atomic_json
    from ..processes.job import NativeJob
    from .independent_service import _read, _require
    directory = output / key
    intent = directory / "intent.json"
    terminal = directory / "native-terminal.json"
    if terminal.exists():
        saved = _read(terminal)
        _require(saved["command"] == argv and saved.get("nativeComplete") is True
                 and saved["rootExitCode"] == 0, "Original rebind helper did not complete", "service_rebind_pending")
        return saved
    job = _SERVICE_REBIND_JOBS.get(str(intent))
    if intent.exists():
        _require(_read(intent)["command"] == argv and job is not None,
                 "Original helper launch needs native reconciliation", "service_rebind_pending")
    else:
        atomic_json(intent, {"command": argv, "createdAt": time.time(), "status": "launch-intent"})
        environment = dict(os.environ)
        for name in ("TEMP", "TMP", "TMPDIR"):
            environment[name] = str(manager.paths.repo / ".jenkins/tmp")
        job = NativeJob.launch(argv, cwd=manager.paths.repo, env=environment,
                               log_dir=directory, suspended=True)
        _SERVICE_REBIND_JOBS[str(intent)] = job
        atomic_json(directory / "started.json", {"command": argv, "identity": job.identity.to_dict()})
        job.resume()
    try:
        code = job.wait(timeout_seconds=600, cleanup_descendants=False)
        proof = job.proof().to_dict()
        joined = all(not t.is_alive() for t in job.process._pump_threads)
        _require(proof.get("complete") is True and proof.get("activeProcesses") == 0
                 and proof.get("stdoutEof") is True and proof.get("stderrEof") is True and joined,
                 "Rebind helper native termination is incomplete", "service_rebind_pending")
        job.close()
        record = {"command": argv, "rootExitCode": code, "native": proof,
                  "readerThreadsJoined": joined, "waitReturned": True, "jobClosedAfterZero": True,
                  "nativeComplete": True, "observedAt": time.time()}
        atomic_json(terminal, record)
        _SERVICE_REBIND_JOBS.pop(str(intent), None)
        _require(code == 0, "Original rebind helper failed", "service_rebind_pending")
        return record
    except Exception:
        # Retain the actual native object; never close an unknown live Job or launch a replacement.
        raise


def _service_management_core(manager, installed, codex):
    from .independent_service import _read, _require
    capsule = Path(installed["capsule"])
    metadata = _read(capsule / "capsule.json")
    _require(capsule == manager.paths.repo / ".jenkins/runtime/management" / installed["capsuleSha256"]
             and _service_ref(capsule / "capsule.json")["sha256"] == installed["capsuleSha256"]
             and metadata["driverDigest"] == installed["driverDigest"]
             and metadata["runtimeOperationId"] == installed["runtimeOperationId"],
             "Management capsule envelope changed", "service_rebind_pending")
    for name, checksum in metadata["files"].items():
        _require(_service_ref(capsule / name)["sha256"] == checksum,
                 "Management code changed", "service_rebind_pending")
    client_runner = None
    if "clientRoot" in metadata:
        import importlib.util
        spec = importlib.util.spec_from_file_location("service_rebind_client_selector", capsule / "runner.py")
        client_runner = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(client_runner)
    for name, checksum in metadata["clientSources"].items():
        path = (client_runner.client_path(metadata, manager.paths.repo, name, capsule=capsule)
                if client_runner is not None else manager.paths.repo / name)
        _require(_service_ref(path)["sha256"] == checksum,
                 "Management client code changed", "service_rebind_pending")
    template = _service_ref_read(_service_ref(capsule / "zircon-management.groovy")).decode("utf-8-sig").replace("\r\n", "\n")
    tokens = {"AGENT": metadata["agentLabel"], "REPOSITORY": str(manager.paths.repo),
              "PYTHON": str(manager.paths.repo / ".jenkins/runtime/python/python.exe"), "CODEX": codex,
              "CAPSULE": str(capsule), "DRIVER": installed["driverDigest"], "RUNTIME": installed["runtimeOperationId"]}
    for key, value in tokens.items():
        template = template.replace("@@" + key + "@@", value.replace("\\", "/").replace("'", "\\'"))
    _require("@@" not in template, "Management core has unresolved tokens", "service_rebind_pending")
    return template, metadata



def _service_env_value(script, key, *, required=True):
    import re
    from .independent_service import _require
    rows = list(re.finditer(r"(env\." + re.escape(key) + r"\s*=\s*)(['\"])(.*?)\2", script))
    if not rows and not required:
        return None
    _require(len(rows) == 1, "Runtime environment field is not unique: " + key, "service_rebind_pending")
    item = rows[0]
    value = json.loads(item[2] + item[3] + item[2]) if item[2] == '"' else re.sub(
        r"\\([\\'])", lambda match: match[1], item[3])
    return item, value


def _service_replace_env(script, key, expected, value):
    from .independent_service import _require
    item, actual = _service_env_value(script, key)
    path_value = isinstance(expected, Path)
    _require(Path(actual) == expected if path_value else actual == expected,
             "Retained runtime environment changed: " + key, "service_rebind_pending")
    if isinstance(value, Path):
        value = value.as_posix() if "/" in actual and "\\" not in actual else str(value)
    quoted = json.dumps(value) if item[2] == '"' else "'" + value.replace("\\", "\\\\").replace("'", "\\'") + "'"
    return script[:item.start()] + item[1] + quoted + script[item.end():]


def _service_driver_environment(manager, script, before, current, driver, *, runtime=True):
    if runtime:
        script = _service_replace_env(script, "ZIRCON_RUNTIME_OPERATION_ID",
                                      before["runtimeOperationId"], current["operationId"])
    if driver != before["driverDigest"]:
        old_root = manager.paths.runtime / "drivers" / before["driverDigest"]
        new_root = manager.paths.runtime / "drivers" / driver
        old = verify_driver(old_root, expected_digest=before["driverDigest"])
        new = verify_driver(new_root, expected_digest=driver)
        for key, field in (("ZIRCON_SEALED_DRIVER", "steps"), ("ZIRCON_DRIVER_LAUNCHER", "launcher")):
            script = _service_replace_env(script, key, Path(old[field]), Path(new[field]))
        script = _service_replace_env(script, "ZIRCON_DRIVER_DIGEST", before["driverDigest"], driver)
    return script


def _service_cleanup_control(script, manager, before, binding_ref):
    """Retain the already-installed scheduled helper and its exact small plan."""
    import re
    from .independent_service import _require
    selected = _service_env_value(script, "ZIRCON_SCHEDULED_CONTROL_ENTRY", required=False)
    pinned_args = len(re.findall(r"--expected-driver-digest\s+(['\"])" + re.escape(before["driverDigest"]) + r"\1", script))
    if selected is None:
        return {"scheduled": None, "pinnedDriverArgCount": pinned_args}
    entry = _service_ref(Path(selected[1]))
    _, entry_sha = _service_env_value(script, "ZIRCON_SCHEDULED_CONTROL_SHA256")
    _, plan_path = _service_env_value(script, "ZIRCON_SCHEDULED_CONTROL_PLAN")
    _, plan_sha = _service_env_value(script, "ZIRCON_SCHEDULED_CONTROL_PLAN_SHA256")
    plan_ref = _service_ref(Path(plan_path))
    plan = json.loads(_service_ref_read(plan_ref))
    _require(entry["sha256"] == entry_sha and plan_ref["sha256"] == plan_sha
             and Path(entry["path"]).is_relative_to(manager.paths.repo / ".jenkins/worktrees")
             and Path(entry["path"]).name == "scheduled_control.py"
             and plan.get("schemaVersion") == 1 and plan.get("driverDigest") == before["driverDigest"]
             and plan.get("runtimeOperationId") == before["runtimeOperationId"]
             and set(plan) == {"schemaVersion", "driverDigest", "runtimeOperationId", "bindingModule",
                              "nativeModule", "checksModule", "pins"}
             and Path(plan["bindingModule"]) == Path(binding_ref["path"])
             and Path(plan["nativeModule"]) == manager.paths.repo / "tools/dev/local_cargo.py"
             and Path(plan["checksModule"]) == Path(entry["path"]).parent / "run_checks.py"
             and set(plan["pins"]) == {plan[key] for key in ("bindingModule", "nativeModule", "checksModule")},
             "Scheduled cleanup has an unsupported existing runtime closure", "service_rebind_pending")
    for path, expected in plan["pins"].items():
        _require(_service_ref(Path(path))["sha256"] == expected,
                 "Scheduled cleanup dependency changed", "service_rebind_pending")
    return {"scheduled": {"entryRef": entry, "planRef": plan_ref, "plan": plan},
            "pinnedDriverArgCount": pinned_args}


def _service_render_cleanup(manager, control, output, key, driver, runtime):
    """The existing render CLI is native-qualified before any lifecycle Stop."""
    from .independent_service import _read, _require
    if control["scheduled"] is None:
        return None
    scheduled = control["scheduled"]
    _service_ref_read(scheduled["entryRef"])
    _service_ref_read(scheduled["planRef"])
    for path, expected in scheduled["plan"]["pins"].items():
        _require(_service_ref(Path(path))["sha256"] == expected,
                 "Scheduled cleanup dependency changed before render", "service_rebind_pending")
    args = [str(manager.paths.repo / ".jenkins/runtime/python/python.exe"), "-B", "-X", "utf8",
            scheduled["entryRef"]["path"], "render", "--driver", driver, "--runtime", runtime]
    _service_command(manager, output, key, args)
    result = _read(output / key / "stdout.log")
    raw = result["planJson"].encode("utf-8")
    _require(json.loads(raw) == result["plan"]
             and hashlib.sha256(raw).hexdigest() == result["sha256"]
             and result["plan"] == {**scheduled["plan"], "driverDigest": driver,
                                   "runtimeOperationId": runtime},
             "Existing cleanup render changed source or protocol fields", "service_rebind_pending")
    return result


def _service_cleanup_driver_script(manager, script, before, current, driver, control):
    import re
    from .independent_service import _require
    if driver != before["driverDigest"]:
        _require(script.count(before["driverDigest"]) == 3 + control["pinnedDriverArgCount"],
                 "Cleanup contains an undeclared fixed driver operand", "service_rebind_pending")
        script = _service_driver_environment(manager, script, before, current, driver, runtime=False)
        pattern = r"(--expected-driver-digest\s+)(['\"])" + re.escape(before["driverDigest"]) + r"\2"
        script, count = re.subn(pattern, lambda item: item[1] + item[2] + driver + item[2], script)
        _require(count == control["pinnedDriverArgCount"],
                 "Cleanup fixed driver argument changed", "service_rebind_pending")
    return script


def _service_cleanup_script(manager, before, current, output, driver):
    import re
    import xml.etree.ElementTree as ET
    from .independent_service import _require, _write_control_once
    from ..state.atomic import atomic_json
    old = before["jobs"]["zircon-scheduled-cleanup"]
    cleanup = before["wrappers"]["cleanup"]
    control = cleanup["control"]
    script = ET.fromstring(old["xml"]).findtext("definition/script")
    script = _service_cleanup_driver_script(manager, script, before, current, driver, control)
    plan_ref = None
    if control["scheduled"] is not None:
        rendered = _service_render_cleanup(manager, control, output, "cleanup-render", driver, current["operationId"])
        original = control["scheduled"]["planRef"]
        path = Path(original["path"]).parent / "runtime-plans" / (current["operationId"] + ".json")
        atomic_json(output / "cleanup-plan-publication.artifacts.json",
                    {"schemaVersion": 1, "taskId": current["operationId"],
                     "owner": "independent-service-runtime-rebind", "outputs": [str(path)],
                     "dependencies": [original, control["scheduled"]["entryRef"]],
                     "retention": "Keep original scheduled plan and exact successor; no cache/source cleanup"})
        _write_control_once(path, rendered["plan"])
        plan_ref = _service_ref(path)
        script = _service_replace_env(script, "ZIRCON_SCHEDULED_CONTROL_PLAN",
                                      Path(original["path"]), path)
        script = _service_replace_env(script, "ZIRCON_SCHEDULED_CONTROL_PLAN_SHA256",
                                      original["sha256"], plan_ref["sha256"])
    return _service_script_xml(old["xml"], script), plan_ref


def _service_management_args(manager, registration, before, runtime, *, driver=None):
    from .independent_service import _require
    driver = before["driverDigest"] if driver is None else driver
    installer = registration["managementInstaller"]
    _service_ref_read(installer)
    _service_ref_read(registration["codex"])
    codex = registration["codex"]["path"]
    args = [str(Path(__import__("os").environ["SystemRoot"]) / "System32/WindowsPowerShell/v1.0/powershell.exe"),
            "-NoLogo", "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass",
            "-File", installer["path"], "-RepoRoot", str(manager.paths.repo),
            "-ExpectedDriverDigest", driver, "-ExpectedRuntimeOperationId", runtime,
            "-CodexExe", codex, "-ReuseCapsuleSha256", before["installed"]["capsuleSha256"],
            "-ExpectedInstalledSha256", before["installedRef"]["sha256"],
            "-ExpectedPreviousJobConfigSha256", before["jobs"]["zircon-management"]["apiSha256"],
            "-ExpectedRetainedPrefixSha256", before["managementPrefixSha256"]]
    if runtime != before["runtimeOperationId"]:
        args += ["-ExpectedPreviousRuntimeOperationId", before["runtimeOperationId"]]
    if driver != before["driverDigest"]:
        args += ["-ExpectedPreviousDriverDigest", before["driverDigest"]]
    return args


def verify_runtime_rebind_inputs(manager, registration, before, *, offline=False):
    from .independent_service import _require
    _service_ref_read(registration["managementInstaller"])
    _service_ref_read(registration["codex"])
    _service_ref_read(before["installedRef"])
    _service_management_core(manager, before["installed"], registration["codex"]["path"])
    for name, old in before["jobs"].items():
        _service_ref_read(old["homeRef"])
        if not offline:
            _require(_service_job(manager, name) == old, "Original job binding changed", "service_rebind_pending")
    for row in before["records"]:
        _service_ref_read(row["installedRef"])
        for name, checksum in row["installed"]["sources"].items():
            _require(_service_ref(manager.paths.repo / "tools/jenkins-management" / name)["sha256"] == checksum,
                     "Records source changed", "service_rebind_pending")
    return before


def capture_runtime_rebind(manager, registration, output, *, preflight=True, target_driver=None):
    """Called before Stop: retain exact old bindings and reject unsupported preservation."""
    import xml.etree.ElementTree as ET
    from .independent_service import _read, _require
    from ..state.atomic import atomic_json
    _require(isinstance(registration, dict) and registration.get("schemaVersion") == 1,
             "Runtime rebind registration is required", "service_rebind_pending")
    selected = _read(manager.paths.state / "deployment/driver.json")
    installed_path = manager.paths.state / "management/installed.json"
    installed = _read(installed_path)
    _require(installed.get("runtimeOperationId") == selected["runtimeOperationId"]
             and installed.get("driverDigest") == selected["driverDigest"],
             "Previous management runtime is not bound", "service_rebind_pending")
    old = {"runtimeOperationId": selected["runtimeOperationId"], "driverDigest": selected["driverDigest"],
           "installed": installed, "installedRef": _service_ref(installed_path), "jobs": {}, "records": []}
    for name in ("zircon-flow", "zircon-execution", "zircon-maintenance", "zircon-management",
                 "zircon-authorization", "zircon-governance", "zircon-management-tests", "zircon-scheduled-cleanup",
                 "zircon-merge-queue"):
        old["jobs"][name] = _service_job(manager, name)
    for name, relative, script in (
            ("zircon-records", "records-queue/installed.json", "records_queue.py"),
            ("zircon-records-followup", "records-queue/followups/installed.json", "records_followup.py")):
        path = manager.paths.state / relative
        _require(path.exists(), "A required Records installed binding is absent", "service_rebind_pending")
        document = _read(path)
        _require(document.get("job") == name and document.get("runtimeOperationId") == old["runtimeOperationId"]
                 and document.get("driverDigest") == old["driverDigest"],
                 "Previous Records runtime differs", "service_rebind_pending")
        job = _service_job(manager, name)
        _service_records_shape(job["xml"], followup=script == "records_followup.py")
        for member, checksum in document["sources"].items():
            source = manager.paths.repo / "tools/jenkins-management" / member
            _require(_service_ref(source)["sha256"] == checksum,
                     "Records code changed; retain its source owner", "service_rebind_pending")
        old["jobs"][name] = job
        old["records"].append({"name": name, "script": script, "installed": document,
                               "installedRef": _service_ref(path)})
    script = ET.fromstring(old["jobs"]["zircon-management"]["xml"]).findtext("definition/script")
    core, _ = _service_management_core(manager, installed, registration["codex"]["path"])
    _require(script.endswith(core) and script.count(core) == 1,
             "Management prefix has no unique pinned core", "service_rebind_pending")
    old["managementPrefixSha256"] = hashlib.sha256(script[:-len(core)].encode("utf-8")).hexdigest()
    _require(set(old["jobs"]) == set(_SERVICE_REQUIRED_JOBS),
             "All eleven existing useful jobs are required", "service_rebind_pending")
    old["wrappers"] = _service_wrapper_bindings(manager, old)
    old["mergeQueue"] = _service_merge_queue_binding(manager, old, registration)
    verify_runtime_rebind_inputs(manager, registration, old)
    from .independent_service import _write_control_once
    _write_control_once(output / "runtime-before.json", old)
    if preflight:
        args = _service_management_args(manager, registration, old, old["runtimeOperationId"])
        _service_command(manager, output, "management-preflight", args + ["-CheckOnly"])
        control = old["wrappers"]["cleanup"]["control"]
        _service_render_cleanup(manager, control, output, "cleanup-preflight",
                                old["driverDigest"], old["runtimeOperationId"])
        if target_driver is not None and target_driver["driverDigest"] != old["driverDigest"]:
            target = target_driver["driverDigest"]
            template = {"operationId": old["runtimeOperationId"]}
            core_plans = _service_core_rebind(manager, old, template, output, driver=target, preflight=True)
            cleanup = old["jobs"]["zircon-scheduled-cleanup"]["xml"]
            script = ET.fromstring(cleanup).findtext("definition/script")
            cleanup_script = _service_cleanup_driver_script(manager, script, old, template, target, control)
            cleanup_xml = _service_script_xml(cleanup, cleanup_script)
            _require(_service_xml_shape(cleanup, bindings=True) == _service_xml_shape(cleanup_xml, bindings=True),
                     "Cleanup target template changed non-binding configuration", "service_rebind_pending")
            rendered = _service_render_cleanup(manager, control, output, "cleanup-target-preflight",
                                                target, old["runtimeOperationId"])
            _write_control_once(output / "target-driver-preflight.json",
                                {"schemaVersion": 1, "targetDriver": target_driver,
                                 "runtimeOperationId": None, "templateRuntimeOperationId": old["runtimeOperationId"],
                                 "coreTemplates": [{"job": item["job"], "sha256": item["afterSha256"]}
                                                   for item in core_plans],
                                 "cleanupTemplateSha256": hashlib.sha256(cleanup_xml.encode()).hexdigest(),
                                 "cleanupRender": rendered, "actualRuntimeBindingClaimed": False})
    return old


_SERVICE_REQUIRED_JOBS = (
    "zircon-authorization", "zircon-execution", "zircon-flow", "zircon-governance",
    "zircon-maintenance", "zircon-management", "zircon-management-tests",
    "zircon-records", "zircon-records-followup", "zircon-scheduled-cleanup", "zircon-merge-queue")


def _service_script_xml(raw, script):
    """Round-trip the exact XStream escaping before producing an expected hash."""
    import re
    from html import escape as html_escape
    from .independent_service import _require
    match = re.search(r"(<script>)(.*?)(</script>)", raw, re.S)
    _require(match is not None, "Job script XML is unavailable", "service_rebind_pending")
    def encoded(value):
        return html_escape(value, quote=True).replace("&#x27;", "&apos;")
    import xml.etree.ElementTree as ET
    original = ET.fromstring(raw).findtext("definition/script")
    _require(encoded(original) == match[2],
             "Job XML does not have the preserved canonical XStream script form",
             "service_rebind_pending")
    changed = raw[:match.start(2)] + encoded(script) + raw[match.end(2):]
    _require(ET.fromstring(changed).findtext("definition/script") == script
             and _service_xml_shape(raw, bindings=True) == _service_xml_shape(changed, bindings=True),
             "Script-only XML render changed retained configuration", "service_rebind_pending")
    return changed


def _service_wrapper_bindings(manager, before):
    """Qualify the actual four wrapper contracts without broad source adoption."""
    import base64
    import re
    import zlib
    import xml.etree.ElementTree as ET
    from .independent_service import _read, _require, _control_implementation
    jobs = before["jobs"]
    scripts = {name: ET.fromstring(jobs[name]["xml"]).findtext("definition/script")
               for name in ("zircon-authorization", "zircon-governance",
                            "zircon-management-tests", "zircon-scheduled-cleanup")}
    auth = scripts["zircon-authorization"]
    pattern = r"'([A-Za-z0-9_.-]+)'\s*:\s*'(E:/Git/ZirconEngine/\.jenkins/runtime/authorization/[0-9a-f]{64})'"
    pairs = re.findall(pattern, auth)
    _require(pairs and len(dict(pairs)) == len(pairs),
             "Authorization catalogue mapping is not unique", "service_rebind_pending")
    mappings = dict(pairs)
    roots = sorted(set(mappings.values()))
    _require(set(re.findall(r"E:/Git/ZirconEngine/\.jenkins/runtime/authorization/[0-9a-f]{64}", auth))
             == set(roots), "Authorization has undeclared capsule references", "service_rebind_pending")
    capsules = []
    for root in roots:
        path = Path(root)
        metadata_ref = _service_ref(path / "capsule.json")
        metadata = json.loads(_service_ref_read(metadata_ref))
        actor_ref = _service_ref(path / "authorization_owner.py")
        user_path = Path(metadata["userGrant"]["path"])
        _require(user_path.is_relative_to(manager.paths.state / "authorizations")
                 and metadata_ref["sha256"] == path.name and metadata.get("schemaVersion") == 1
                 and metadata.get("job") == "zircon-authorization"
                 and Path(metadata.get("repositoryRoot", "")) == manager.paths.repo
                 and actor_ref["sha256"] == metadata.get("actorSha256"),
                 "Authorization capsule identity differs", "service_rebind_pending")
        user_ref = _service_ref(user_path)
        user = json.loads(_service_ref_read(user_ref))
        _require(user_ref["sha256"] == metadata["userGrant"]["sha256"]
                 and user.get("source") == "user" and user.get("task")
                 and user.get("rootThreadId") == metadata.get("rootThreadId")
                 and hashlib.sha256(user.get("objective", "").encode()).hexdigest() == user.get("objectiveSha256")
                 and user.get("approvedGrants") == metadata.get("grants")
                 and {key for key, value in mappings.items() if value == root} == set(metadata["grants"]),
                 "Authorization original user rights changed", "service_rebind_pending")
        capsules.append({"root": root, "metadata": metadata, "metadataRef": metadata_ref,
                         "actorRef": actor_ref, "userRef": user_ref})
    gs = scripts["zircon-governance"]
    matches = re.findall(r"ZGOV_PINS=([A-Za-z0-9+/=]+)", gs)
    _require(len(matches) == 1, "Governance pins field is not unique", "service_rebind_pending")
    pins = json.loads(zlib.decompress(base64.b64decode(matches[0], validate=True)))
    _require(Path(pins.get("controlRoot", "")) == manager.paths.repo
             and pins.get("jobName") == "zircon-governance"
             and pins.get("driverDigest") == before["driverDigest"]
             and Path(pins.get("python", "")) == manager.paths.repo / ".jenkins/runtime/python/python.exe"
             and _service_ref(Path(pins["python"]))["sha256"] == pins.get("pythonSha256"),
             "Governance executable closure root differs", "service_rebind_pending")
    implementation = _control_implementation()
    own = {"tools/jenkins/deployment/independent_service.py": implementation["serviceSha256"],
           "tools/jenkins/deployment/job_provisioning.py": implementation["jobProvisioningSha256"]}
    _require(set(own) <= set(pins["sourcePins"]), "Governance closure lost control modules", "service_rebind_pending")
    verified = dict(pins["sourcePins"])
    verified.update(own)
    for relative, expected in verified.items():
        path = Path(relative)
        _require(not path.is_absolute() and ".." not in path.parts
                 and _service_ref(manager.paths.repo / path)["sha256"] == expected,
                 "Governance executable source needs its existing reviewed rebind: " + relative,
                 "service_rebind_pending")
    tests = scripts["zircon-management-tests"]
    marker = "// BEGIN ZIRCON ORDINARY MANAGEMENT TESTS"
    _require(tests.count(marker) == 1, "Ordinary test core is not uniquely bounded", "service_rebind_pending")
    offset = tests.index(marker) + len(marker)
    prefix, core = tests[:offset], tests[offset:]
    match = re.search(r"(?m)^def runtime = '([A-Za-z0-9_.-]+)'$", core)
    _require(match is not None and len(re.findall(r"(?m)^def runtime\s*=", core)) == 1
             and ("def driver = '" + before["driverDigest"] + "'") in core,
             "Ordinary test runtime binding differs", "service_rebind_pending")
    template_ref = _service_ref(manager.paths.repo / "tools/jenkins-management/zircon-management-tests.groovy")
    template = _service_ref_read(template_ref).decode("utf-8-sig").replace("\r\n", "\n")
    declarations = {key: re.search(r"(?m)^def " + key + r" = '([^'\r\n]+)'$", core)
                    for key in ("agentLabel", "repository", "python", "codex", "driver", "runtime")}
    _require(all(declarations.values()), "Ordinary test core declarations differ", "service_rebind_pending")
    tokens = {"AGENT": "agentLabel", "REPOSITORY": "repository", "PYTHON": "python",
              "CODEX": "codex", "DRIVER": "driver", "RUNTIME": "runtime"}
    for token, key in tokens.items():
        template = template.replace("@@" + token + "@@", declarations[key][1])
    _require(core.lstrip("\r\n") == template,
             "Ordinary test core does not match its existing template", "service_rebind_pending")
    cleanup = scripts["zircon-scheduled-cleanup"]
    checksum = re.findall(r"""ZIRCON_CLEANUP_BINDING_SHA256=[\'"]?([0-9a-f]{64})""", cleanup)
    cleanup_ref = _service_ref(manager.paths.repo / "tools/dev/jenkins_cleanup_runtime.py")
    _require(len(checksum) == 1 and checksum[0] == cleanup_ref["sha256"]
             and "bindScheduledRuntime()" in cleanup
             and "binding.runtimeOperationId" in cleanup,
             "Cleanup dynamic runtime binding is not the actual pinned implementation", "service_rebind_pending")
    cron = ET.fromstring(jobs["zircon-scheduled-cleanup"]["xml"]).findall(".//hudson.triggers.TimerTrigger/spec")
    _require(len(cron) == 1 and cron[0].text == "H/30 * * * *",
             "Cleanup H/30 timer changed", "service_rebind_pending")
    return {"authorization": {"mappings": mappings, "capsules": capsules},
            "governance": {"token": matches[0], "pins": pins, "verifiedSourcePins": verified},
            "managementTests": {"prefix": prefix, "core": core, "runtime": match[1],
                                "templateRef": template_ref},
            "cleanup": {"bindingRef": cleanup_ref, "cron": cron[0].text,
                        "control": _service_cleanup_control(cleanup, manager, before, cleanup_ref),
                        "configurationQualifiedUnchanged": True}}


def _service_authorization_capsules(manager, before, current, output, *, driver=None):
    """Inherit exact user rights and actor bytes; only runtime envelope fields change."""
    from .independent_service import _read, _require, _write_control_once
    driver = before["driverDigest"] if driver is None else driver
    rows = []
    replacements = {}
    for item in before["wrappers"]["authorization"]["capsules"]:
        _service_ref_read(item["metadataRef"])
        _service_ref_read(item["actorRef"])
        _service_ref_read(item["userRef"])
        metadata = {**item["metadata"], "runtimeOperationId": current["operationId"],
                    "driverDigest": driver}
        raw = (json.dumps(metadata, sort_keys=True, indent=2) + "\n").encode()
        checksum = hashlib.sha256(raw).hexdigest()
        root = manager.paths.repo / ".jenkins/runtime/authorization" / checksum
        rows.append({"root": str(root), "metadata": metadata, "metadataRaw": raw.decode(),
                     "metadataSha256": checksum, "original": item})
        replacements[item["root"]] = root.as_posix()
    _write_control_once(output / "authorization-artifacts.json",
                        {"schemaVersion": 1, "taskId": current["operationId"],
                         "owner": "independent-service-runtime-rebind",
                         "outputs": [{"root": row["root"],
                                      "members": {"capsule.json": row["metadataSha256"],
                                                  "authorization_owner.py": row["original"]["actorRef"]["sha256"]}}
                                     for row in rows],
                         "dependencies": [row["original"] for row in rows],
                         "retention": "Immutable original and replacement authorization envelopes; no new grants"})
    for row in rows:
        root = Path(row["root"])
        root.mkdir(parents=True, exist_ok=True)
        _write_control_once(root / "capsule.json", row["metadata"])
        actor = root / "authorization_owner.py"
        raw = _service_ref_read(row["original"]["actorRef"])
        if actor.exists():
            _require(_service_ref(actor)["sha256"] == row["original"]["actorRef"]["sha256"],
                     "Existing authorization actor differs", "service_rebind_pending")
        else:
            with actor.open("xb") as stream:
                stream.write(raw)
                stream.flush()
                __import__("os").fsync(stream.fileno())
        _require(_service_ref(root / "capsule.json")["sha256"] == row["metadataSha256"]
                 and _read(root / "capsule.json") == row["metadata"],
                 "Authorization envelope serialization differs", "service_rebind_pending")
    return replacements, rows


def _service_jobs_cas(manager, before, current, output, key, plans, *, driver=None):
    """A bounded exact-XML QueueCAS; unknown delivery is queried, never reposted."""
    import base64
    import time
    from ..state.atomic import atomic_json
    from .independent_service import _read, _require
    driver = before["driverDigest"] if driver is None else driver
    allowed = {"zircon-flow", "zircon-execution", "zircon-maintenance",
               "zircon-authorization", "zircon-governance", "zircon-management-tests", "zircon-merge-queue",
               "zircon-scheduled-cleanup"}
    _require(plans and len({p["job"] for p in plans}) == len(plans)
             and {p["job"] for p in plans} <= allowed, "Runtime CAS job set differs", "service_rebind_pending")
    for p in plans:
        _require(p["afterSha256"] == hashlib.sha256(p["after"].encode()).hexdigest()
                 and _service_xml_shape(p["before"]["xml"], bindings=True)
                 == _service_xml_shape(p["after"], bindings=True),
                 "Runtime expected XML failed its pure render check", "service_rebind_pending")
    intent = output / (key + "-intent.json")
    if intent.exists():
        _require(_read(intent)["plans"] == plans, "Original runtime CAS needs reconciliation", "service_rebind_pending")
        for p in plans:
            actual = _service_job(manager, p["job"])
            _require(actual["apiSha256"] == p["afterSha256"]
                     and _service_xml_shape(actual["homeXml"], bindings=True)
                     == _service_xml_shape(p["before"]["homeXml"], bindings=True),
                     "Original runtime CAS unknown; query without repost", "service_rebind_pending")
        return
    for p in plans:
        _require(_service_job(manager, p["job"]) == p["before"],
                 "Job changed before runtime CAS", "service_rebind_pending")
    transmitted = [{"job": p["job"], "before": {"apiSha256": p["before"]["apiSha256"],
                    "homeRef": {"sha256": p["before"]["homeRef"]["sha256"]}},
                    "after": p["after"], "afterSha256": p["afterSha256"]} for p in plans]
    payload = base64.b64encode(json.dumps({"plans": transmitted, "runtime": current["operationId"],
                              "driver": driver, "repo": str(manager.paths.repo)},
                              separators=(",", ":")).encode()).decode()
    pieces = "[" + ",".join("'" + payload[i:i + 24000] + "'" for i in range(0, len(payload), 24000)) + "].join('')"
    script = """
import jenkins.model.Jenkins
import hudson.model.Items
import hudson.model.Queue
import javax.xml.transform.stream.StreamSource
import java.security.MessageDigest
import groovy.json.JsonSlurper
import groovy.json.JsonOutput
def p=new JsonSlurper().parseText(new String(PAYLOAD.decodeBase64(),'UTF-8'))
def sha={byte[] b -> MessageDigest.getInstance('SHA-256').digest(b).encodeHex().toString()}
def result=Queue.withLock({ ->
 def j=Jenkins.get()
 assert !j.isQuietingDown()
 def selected=new JsonSlurper().parse(new File(p.repo+'/.jenkins/state/deployment/driver.json'))
 def operation=new JsonSlurper().parse(new File(p.repo+'/.jenkins/state/deployment-operation.json'))
 assert selected.runtimeOperationId==p.runtime && selected.driverDigest==p.driver
 assert operation.operationId==p.runtime && operation.state=='running'
 def allowed=['zircon-flow','zircon-execution','zircon-maintenance','zircon-authorization','zircon-governance','zircon-management-tests','zircon-merge-queue','zircon-scheduled-cleanup']
 assert p.plans && p.plans.collect{it.job}.unique().size()==p.plans.size()
 assert p.plans.every{allowed.contains(it.job)}
 def jobs=[:]
 for(def v:p.plans){
  def job=j.getItemByFullName(v.job)
  assert job!=null && job.class.name=='org.jenkinsci.plugins.workflow.job.WorkflowJob'
  assert !job.builds.any{it.isBuilding()} && !j.queue.items.any{it.task.is(job)}
  def b=new ByteArrayOutputStream();Items.XSTREAM2.toXMLUTF8(job,b)
  assert sha(b.toByteArray())==v.before.apiSha256
  assert sha(job.configFile.file.bytes)==v.before.homeRef.sha256
  jobs[v.job]=job
 }
 for(def v:p.plans){
  def job=jobs[v.job]
  job.updateByXml(new StreamSource(new StringReader(v.after)))
  def b=new ByteArrayOutputStream();Items.XSTREAM2.toXMLUTF8(job,b)
  assert sha(b.toByteArray())==v.afterSha256
 }
 return [status:'runtime-jobs-cas-verified',runtime:p.runtime]
} as java.util.concurrent.Callable)
println(JsonOutput.toJson(result))
""".replace("PAYLOAD", pieces)
    status, raw = manager._request("crumbIssuer/api/json")
    _require(status == 200, "Jenkins crumb unavailable", "service_rebind_pending")
    crumb = json.loads(raw)
    atomic_json(intent, {"plans": plans, "runtime": current["operationId"], "postAttempted": True,
                         "scriptSha256": hashlib.sha256(script.encode()).hexdigest(), "createdAt": time.time()})
    status, raw = manager._request("scriptText", method="POST",
              data=urllib.parse.urlencode({"script": script}).encode(),
              headers={"Content-Type": "application/x-www-form-urlencoded",
                       crumb["crumbRequestField"]: crumb["crumb"]})
    atomic_json(output / (key + "-response.json"), {"httpStatus": status,
                "bodyBase64": base64.b64encode(raw).decode()})
    _require(status == 200, "Runtime CAS delivery unknown", "service_rebind_pending")
    for p in plans:
        actual = _service_job(manager, p["job"])
        _require(actual["apiSha256"] == p["afterSha256"]
                 and _service_xml_shape(p["before"]["homeXml"], bindings=True)
                 == _service_xml_shape(actual["homeXml"], bindings=True),
                 "Job API/Home runtime readback differs", "service_rebind_pending")


def _service_merge_queue_binding(manager, before, registration):
    """Read only the existing owned timer argv and its immutable plan closure."""
    import re
    import xml.etree.ElementTree as ET
    from .independent_service import _require
    bridge = registration.get("mergeQueue")
    _require(isinstance(bridge, dict), "Merge runtime-plan bridge is not registered", "service_rebind_pending")
    root = manager.paths.state / "management/reports/zircon-git-sync-20261018-r1"
    _require(Path(bridge.get("controlRoot", "")) == root
             and bridge.get("ownerThreadId") == "01a12fdb-4d74-75d1-92fa-ca578416c6ab",
             "Merge runtime-plan ownership differs", "service_rebind_pending")
    for key, name, checksum, size in (
            ("controller", "controller-r4.py", "9686b02535f8919372f5e279a557b3f1f5ceb278897aee800b34b39bb2b71e61", 7253),
            ("worker", "worker-r4.py", "8b2bd6b9e483c114c60b469921ccf750a59b94c3557732e8f2236a9384f1d59a", 25125)):
        _require(bridge.get(key) == {"path": str(root / name), "sha256": checksum, "bytes": size},
                 "Merge reviewed bridge bytes differ", "service_rebind_pending")
        _service_ref_read(bridge[key])
    manifest = json.loads(_service_ref_read(bridge["ownershipRef"]))
    _require(manifest.get("ownerThreadId") == bridge["ownerThreadId"]
             and manifest.get("taskId") == "zircon-git-sync-20261018-r1"
             and any(Path(p) == root for p in manifest.get("paths", [])),
             "Merge control root has no original managed attribution", "service_rebind_pending")
    job = before["jobs"]["zircon-merge-queue"]
    doc = ET.fromstring(job["xml"])
    script = doc.findtext("definition/script")
    pattern = (r'"(' + re.escape(root.as_posix()) + r'/controller-r(?:[34]|5-mq19)\.py)"'
               r' --self-sha ([0-9a-f]{64}) --plan-sha ([0-9a-f]{64})'
               r'(?: --plan-file "([^"\r\n]+)")?')
    matches = list(re.finditer(pattern, script))
    _require(len(matches) == 1, "Merge controller argv is not uniquely retained", "service_rebind_pending")
    match = matches[0]
    mq19 = Path(match[1]).name == "controller-r5-mq19.py"
    plan_path = Path(match[4]) if match[4] else root / "plan-r3.json"
    _require(plan_path.is_relative_to(root) and plan_path != root
             and ((match[4] is None and Path(match[1]).name == "controller-r3.py")
                  or (match[4] is not None and Path(match[1]).name in {"controller-r4.py", "controller-r5-mq19.py"})),
             "Merge plan escaped its original control root", "service_rebind_pending")
    plan_ref = _service_ref(plan_path)
    _require(plan_ref["sha256"] == match[3], "Merge argv plan digest differs", "service_rebind_pending")
    plan = json.loads(_service_ref_read(plan_ref))
    def actual(ref):
        value = {"path": str(Path(ref["path"])), "sha256": ref["sha256"],
                 "bytes": ref.get("bytes", ref.get("size"))}
        if mq19 and "bytes" not in ref and "size" not in ref:
            value["bytes"] = _service_ref(Path(value["path"]))["bytes"]
        _service_ref_read(value)
        return value
    controller = actual(plan["controller"])
    worker = actual(plan["worker"])
    _require(controller["path"] == str(Path(match[1])) and controller["sha256"] == match[2]
             and Path(controller["path"]).parent == root and Path(worker["path"]).parent == root
             and Path(worker["path"]).name == Path(controller["path"]).name.replace("controller-", "worker-", 1)
             and plan.get("job") == "zircon-merge-queue" and plan.get("schemaVersion") == 1
             and Path(plan.get("repository", "")) == manager.paths.repo
             and plan.get("runtime") == {"driverDigest": before["driverDigest"],
                                         "runtimeOperationId": before["runtimeOperationId"]}
             and actual(plan["manifest"]) == bridge["ownershipRef"],
             "Merge actual plan/runtime closure differs", "service_rebind_pending")
    allowed = {"controller-r3.py": "729cca372fd3daca2d68be61170c599e96bb27ef58c0fa5efc717fc89589c979",
               "worker-r3.py": "d69fa5f32567f99f4497eb1d5fb344a5a02abd491e52d69020c7b050d48c61c2",
               "controller-r4.py": bridge["controller"]["sha256"], "worker-r4.py": bridge["worker"]["sha256"],
               "controller-r5-mq19.py": "1ec2f68288e56c954879b2987b3e945246093b1a2405fae6283b74a02a3eae17",
               "worker-r5-mq19.py": "6e76b4a07f7949adbde3f14534c3ccdb2cbad8a602c71f545ab7226b632ce908"}
    _require(all(allowed.get(Path(r["path"]).name) == r["sha256"] for r in (controller, worker)),
             "Merge current control lineage differs", "service_rebind_pending")
    source_pins = [actual(ref) for ref in plan["sourcePins"].values()]
    if mq19:
        recovery = actual(plan["queueRecoverySource"])
        _require(recovery == {
                     "path": str(Path("E:/cargo-targets/zircon-local/reports/jenkins-merge-queue-repair-20261019/merge_queue_recovery.py")),
                     "sha256": "bdb01051ca2b09b03c1436e1985d71ac469fd4629d74e02b5af953ea06913bf2", "bytes": 6863}
                 and recovery in source_pins,
                 "Merge queue recovery closure differs", "service_rebind_pending")
        bridge = {**bridge, "controller": controller, "worker": worker}
    actual(plan["publisherSource"])
    cron = doc.findall(".//hudson.triggers.TimerTrigger/spec")
    disabled = doc.findall(".//org.jenkinsci.plugins.workflow.job.properties.DisableConcurrentBuildsJobProperty")
    _require(len(cron) == len(disabled) == 1 and cron[0].text == "H/15 * * * *"
             and disabled[0].findtext("abortPrevious", "false") == "false",
             "Merge H/15 or serialized scheduling changed", "service_rebind_pending")
    return {"bridge": bridge, "plan": plan, "planRef": plan_ref,
            "argv": match[0], "cron": cron[0].text}


def _service_merge_queue_rebind(manager, before, current, output, *, driver=None):
    """Only the original timer's frozen controller/plan argv is replaced."""
    import copy
    import re
    import xml.etree.ElementTree as ET
    from .independent_service import _require, _write_control_once
    from ..state.atomic import atomic_json
    driver = before["driverDigest"] if driver is None else driver
    binding = before["mergeQueue"]
    bridge, old_plan = binding["bridge"], binding["plan"]
    for key in ("controller", "worker", "ownershipRef"):
        _service_ref_read(bridge[key])
    _service_ref_read(binding["planRef"])
    runtime = current["operationId"]
    _require(re.fullmatch(r"[A-Za-z0-9_.-]+", runtime) is not None,
             "Merge replacement runtime is not an immutable leaf name", "service_rebind_pending")
    root = Path(bridge["controlRoot"])
    plan_path = root / "runtime-plans" / (runtime + ".json")
    plan = copy.deepcopy(old_plan)
    plan.update(runtime={"driverDigest": driver, "runtimeOperationId": runtime},
                controller={"path": bridge["controller"]["path"], "sha256": bridge["controller"]["sha256"],
                            "size": bridge["controller"]["bytes"]},
                worker={"path": bridge["worker"]["path"], "sha256": bridge["worker"]["sha256"],
                        "size": bridge["worker"]["bytes"]},
                previousPlan={"path": binding["planRef"]["path"], "sha256": binding["planRef"]["sha256"],
                              "size": binding["planRef"]["bytes"]},
                revision=old_plan["revision"] + 1)
    for key in ("controller", "worker"):
        ref = bridge[key]
        prior = [v for v in plan["sourcePins"].values() if Path(v["path"]) == Path(ref["path"])]
        _require(not prior or all(v["sha256"] == ref["sha256"] and v.get("size", v.get("bytes")) == ref["bytes"]
                                  for v in prior), "Merge source pin was replaced", "service_rebind_pending")
        if not prior:
            plan["sourcePins"][ref["path"]] = {"path": ref["path"], "sha256": ref["sha256"], "size": ref["bytes"]}
    allowed = {"runtime", "controller", "worker", "previousPlan", "revision", "sourcePins"}
    _require({k: v for k, v in plan.items() if k not in allowed}
             == {k: v for k, v in old_plan.items() if k not in allowed}
             and all(plan["sourcePins"].get(k) == v for k, v in old_plan["sourcePins"].items()),
             "Merge authorization/publisher/remote/history changed", "service_rebind_pending")
    atomic_json(output / "merge-queue-publication.artifacts.json",
                {"schemaVersion": 1, "ownerThreadId": bridge["ownerThreadId"],
                 "taskId": "zircon-git-sync-20261018-r1", "outputs": [str(plan_path)],
                 "dependencies": [binding["planRef"], bridge["ownershipRef"], bridge["controller"], bridge["worker"]],
                 "retention": "Retain old plans and the actual successor runtime plan; no Git/source cleanup."})
    _write_control_once(plan_path, plan)
    plan_ref = _service_ref(plan_path)
    old = before["jobs"]["zircon-merge-queue"]
    script = ET.fromstring(old["xml"]).findtext("definition/script")
    argv = ('"' + Path(bridge["controller"]["path"]).as_posix() + '" --self-sha '
            + bridge["controller"]["sha256"] + " --plan-sha " + plan_ref["sha256"]
            + ' --plan-file "' + plan_path.as_posix() + '"')
    _require(script.count(binding["argv"]) == 1, "Merge original argv changed", "service_rebind_pending")
    changed = _service_script_xml(old["xml"], script.replace(binding["argv"], argv))
    _service_jobs_cas(manager, before, current, output, "merge-queue-cas",
                      [{"job": "zircon-merge-queue", "before": old, "after": changed,
                        "afterSha256": hashlib.sha256(changed.encode()).hexdigest()}], driver=driver)
    observed = _service_job(manager, "zircon-merge-queue")
    _require(ET.fromstring(observed["xml"]).findtext("definition/script") == script.replace(binding["argv"], argv)
             and _service_xml_shape(observed["xml"], bindings=True)
             == _service_xml_shape(old["xml"], bindings=True)
             and _service_ref(plan_path) == plan_ref,
             "Merge actual API/Home/plan binding differs", "service_rebind_pending")
    return {"planRef": plan_ref, "controller": bridge["controller"], "worker": bridge["worker"],
            "runtimeOperationId": runtime, "driverDigest": driver, "cron": binding["cron"],
            "jobBindingVerified": True, "gitIntegrationAcceptance": "pending-actual-cycle",
            "newRegressionPassClaimed": False, "oldGitFailurePreserved": True}


def _service_core_rebind(manager, before, current, output, *, driver=None, preflight=False):
    import re
    import xml.etree.ElementTree as ET
    from .independent_service import _require
    driver = before["driverDigest"] if driver is None else driver
    plans = []
    for name in ("zircon-flow", "zircon-execution", "zircon-maintenance"):
        old = before["jobs"][name]
        raw = old["xml"]
        script = ET.fromstring(raw).findtext("definition/script")
        new_script = _service_driver_environment(manager, script, before, current, driver)
        changed = _service_script_xml(raw, new_script)
        default = re.compile(r"(<name>RUNTIME_OPERATION_ID</name>[\s\S]*?<defaultValue>)"
                             + re.escape(before["runtimeOperationId"]) + r"(</defaultValue>)")
        changed, count = default.subn(lambda m: m[1] + current["operationId"] + m[2], changed)
        _require(count == 1, "Core runtime default is not unique", "service_rebind_pending")
        if driver != before["driverDigest"]:
            digest_default = re.compile(r"(<name>STAGE_IMPLEMENTATION_DIGEST</name>[\s\S]*?<defaultValue>)"
                                        + re.escape(before["driverDigest"]) + r"(</defaultValue>)")
            changed, count = digest_default.subn(lambda m: m[1] + driver + m[2], changed)
            _require(count == 1, "Core driver default is not unique", "service_rebind_pending")
        _require(_service_xml_shape(raw, bindings=True) == _service_xml_shape(changed, bindings=True),
                 "Core non-binding configuration changed", "service_rebind_pending")
        plans.append({"job": name, "before": old, "after": changed,
                      "afterSha256": hashlib.sha256(changed.encode()).hexdigest()})
    if preflight:
        return plans
    _service_jobs_cas(manager, before, current, output, "core-cas", plans, driver=driver)


def _service_wrappers_rebind(manager, before, current, output, *, driver=None):
    import base64
    import re
    import zlib
    import xml.etree.ElementTree as ET
    from .independent_service import _read, _require
    driver = before["driverDigest"] if driver is None else driver
    wrappers = before["wrappers"]
    for relative, expected in wrappers["governance"]["verifiedSourcePins"].items():
        _require(_service_ref(manager.paths.repo / relative)["sha256"] == expected,
                 "Governance executable closure changed before runtime CAS: " + relative,
                 "service_rebind_pending")
    replacements, capsules = _service_authorization_capsules(manager, before, current, output, driver=driver)
    plans = []
    for name in ("zircon-authorization", "zircon-governance", "zircon-management-tests"):
        old = before["jobs"][name]
        script = ET.fromstring(old["xml"]).findtext("definition/script")
        if name == "zircon-authorization":
            for source, destination in replacements.items():
                _require(source in script, "Authorization catalogue capsule vanished", "service_rebind_pending")
                script = script.replace(source, destination)
        elif name == "zircon-governance":
            pins = {**wrappers["governance"]["pins"], "driverDigest": driver,
                    "sourcePins": wrappers["governance"]["verifiedSourcePins"]}
            token = base64.b64encode(zlib.compress(json.dumps(pins, sort_keys=True,
                     separators=(",", ":")).encode(), 9)).decode()
            script = script.replace("ZGOV_PINS=" + wrappers["governance"]["token"], "ZGOV_PINS=" + token)
        else:
            tests = wrappers["managementTests"]
            core, count = re.subn(r"(?m)^def runtime = '" + re.escape(tests["runtime"]) + "'$",
                                 "def runtime = '" + current["operationId"] + "'", tests["core"])
            _require(count == 1 and script == tests["prefix"] + tests["core"],
                     "Ordinary test prefix changed", "service_rebind_pending")
            if driver != before["driverDigest"]:
                core, count = re.subn(r"(?m)^def driver = '" + re.escape(before["driverDigest"]) + "'$",
                                      "def driver = '" + driver + "'", core)
                _require(count == 1, "Ordinary test driver is not unique", "service_rebind_pending")
            script = tests["prefix"] + core
        changed = _service_script_xml(old["xml"], script)
        if changed != old["xml"]:
            plans.append({"job": name, "before": old, "after": changed,
                          "afterSha256": hashlib.sha256(changed.encode()).hexdigest()})
    cleanup_xml, cleanup_plan_ref = _service_cleanup_script(manager, before, current, output, driver)
    cleanup_old = before["jobs"]["zircon-scheduled-cleanup"]
    if cleanup_xml != cleanup_old["xml"]:
        plans.append({"job": "zircon-scheduled-cleanup", "before": cleanup_old, "after": cleanup_xml,
                      "afterSha256": hashlib.sha256(cleanup_xml.encode()).hexdigest()})
    if plans:
        _service_jobs_cas(manager, before, current, output, "wrappers-cas", plans, driver=driver)
    # Existing immutable authorization boot proves current native admission;
    # execute()/authorize_session is never invoked by this read-only binding probe.
    refs = [{"root": row["root"], "driver": driver, "runtime": current["operationId"],
             "actorSha256": row["original"]["actorRef"]["sha256"]} for row in capsules]
    probe = ("import hashlib,importlib.util,json,pathlib;"
             "rows=json.loads(" + repr(json.dumps(refs, separators=(",", ":"))) + ");out=[]\n"
             "for row in rows:\n"
             " p=pathlib.Path(row['root']);a=p/'authorization_owner.py'\n"
             " assert hashlib.sha256(a.read_bytes()).hexdigest()==row['actorSha256']\n"
             " s=importlib.util.spec_from_file_location('runtime_authorization_binding_'+p.name,a)\n"
             " m=importlib.util.module_from_spec(s);s.loader.exec_module(m)\n"
             " c,u,r,state=m.boot(p)\n"
             " assert c['driverDigest']==row['driver'] and c['runtimeOperationId']==row['runtime']\n"
             " out.append({'capsule':str(p),'driverDigest':c['driverDigest'],"
             "'runtimeOperationId':c['runtimeOperationId'],'authorizationActionsExecuted':False})\n"
             "print(json.dumps(out,sort_keys=True))")
    _service_command(manager, output, "authorization-binding",
                     [str(manager.paths.repo / ".jenkins/runtime/python/python.exe"), "-B", "-X", "utf8", "-c", probe])
    auth_observed = json.loads((output / "authorization-binding/stdout.log").read_bytes())
    _require(len(auth_observed) == len(capsules) and all(
                 row.get("driverDigest") == driver
                 and row.get("runtimeOperationId") == current["operationId"]
                 and row.get("authorizationActionsExecuted") is False for row in auth_observed),
             "Authorization current binding observation differs", "service_rebind_pending")
    cleanup = wrappers["cleanup"]
    _service_ref_read(cleanup["bindingRef"])
    _service_command(manager, output, "cleanup-binding",
                     [str(manager.paths.repo / ".jenkins/runtime/python/python.exe"), "-B", "-X", "utf8",
                      cleanup["bindingRef"]["path"], "--repo-root", str(manager.paths.repo),
                      "--expected-driver-digest", driver])
    observed = _read(output / "cleanup-binding/stdout.log")
    _require(observed.get("driverDigest") == driver
             and observed.get("runtimeOperationId") == current["operationId"]
             and observed.get("nativeOwners") and observed.get("endpoint")
             and _service_job(manager, "zircon-scheduled-cleanup")["apiSha256"]
             == hashlib.sha256(cleanup_xml.encode()).hexdigest(),
             "Cleanup dynamic current binding/configuration differs", "service_rebind_pending")
    for relative, expected in wrappers["governance"]["verifiedSourcePins"].items():
        _require(_service_ref(manager.paths.repo / relative)["sha256"] == expected,
                 "Governance executable closure changed after runtime CAS: " + relative,
                 "service_rebind_pending")
    return {"authorization": {"capsules": refs, "nativeBindingObserved": True, "grantsChanged": False},
            "governance": {"sourcePins": wrappers["governance"]["verifiedSourcePins"], "runtimeIndependent": True},
            "managementTests": {"ordinaryRuntimeOperationId": current["operationId"],
                                "foreignPrefixSha256": hashlib.sha256(wrappers["managementTests"]["prefix"].encode()).hexdigest()},
            "scheduledCleanup": {"binding": observed, "cron": cleanup["cron"], "configurationChanged": cleanup_xml != cleanup_old["xml"],
                                 "planRef": cleanup_plan_ref}}


def rebind_runtime_preserving_jobs(manager, before, current, registration, output, *, target_driver=None):
    """Call existing installers; retain every prefix and immutable old capsule."""
    import xml.etree.ElementTree as ET
    from ..state.atomic import atomic_json
    from .independent_service import _read, _require
    driver = before["driverDigest"]
    if target_driver is not None:
        _require(set(target_driver) == {"driverDigest", "root", "launcher", "steps", "manifest"}
                 and Path(target_driver["root"]) == manager.paths.runtime / "drivers" / target_driver["driverDigest"]
                 and target_driver == verify_driver(target_driver["root"], expected_digest=target_driver["driverDigest"]),
                 "Replacement target driver is not the actual canonical component", "service_rebind_pending")
        driver = target_driver["driverDigest"]
    selected = _read(manager.paths.state / "deployment/driver.json")
    _require(current.get("state") == "running" and current["operationId"] != before["runtimeOperationId"]
             and selected["runtimeOperationId"] == current["operationId"]
             and selected["driverDigest"] == driver,
             "Replacement runtime binding changed", "service_rebind_pending")
    _service_core_rebind(manager, before, current, output, driver=driver)
    args = _service_management_args(manager, registration, before, current["operationId"], driver=driver)
    _service_command(manager, output, "management-check", args + ["-CheckOnly"])
    check = _read(output / "management-check/stdout.log")
    _require(check.get("status") == "check-only" and check.get("runtimeOperationId") == current["operationId"]
             and check.get("driverDigest") == driver
             and check.get("retainedPrefix", {}).get("prefixSha256") == before["managementPrefixSha256"],
             "Management CheckOnly does not bind this replacement", "service_rebind_pending")
    _service_command(manager, output, "management-apply", args + ["-ExpectedCapsuleSha256", check["capsuleSha256"]])
    installed_path = manager.paths.state / "management/installed.json"
    installed = _read(installed_path)
    _require(installed["driverDigest"] == driver
             and installed["runtimeOperationId"] == current["operationId"]
             and installed["capsuleSha256"] == check["capsuleSha256"],
             "Management installed runtime is pending", "service_rebind_pending")
    _, old_metadata = _service_management_core(manager, before["installed"], registration["codex"]["path"])
    new_core, new_metadata = _service_management_core(manager, installed, registration["codex"]["path"])
    _require({**old_metadata, "runtimeOperationId": current["operationId"], "driverDigest": driver} == new_metadata,
             "Runtime rebind changed capsule code, clients or capabilities", "service_rebind_pending")
    management = _service_job(manager, "zircon-management")
    script = ET.fromstring(management["xml"]).findtext("definition/script")
    _require(script.endswith(new_core) and script.count(new_core) == 1
             and hashlib.sha256(script[:-len(new_core)].encode("utf-8")).hexdigest() == before["managementPrefixSha256"]
             and _service_xml_shape(before["jobs"]["zircon-management"]["xml"], bindings=True)
             == _service_xml_shape(management["xml"], bindings=True),
             "Management prefix or non-binding XML changed", "service_rebind_pending")
    for record in before["records"]:
        prior = before["jobs"][record["name"]]
        args = [str(manager.paths.repo / ".jenkins/runtime/python/python.exe"), "-B", "-X", "utf8",
                str(manager.paths.repo / "tools/jenkins-management" / record["script"]), "install",
                "--repo-root", str(manager.paths.repo), "--rebind-runtime",
                "--upgrade-from", record["installedRef"]["sha256"],
                "--expected-job-script-sha256",
                hashlib.sha256(ET.fromstring(prior["xml"]).findtext("definition/script").encode()).hexdigest()]
        _service_command(manager, output, record["name"] + "-install", args)
        fresh = _read(Path(record["installedRef"]["path"]))
        actual = _service_job(manager, record["name"])
        _require(fresh["runtimeOperationId"] == current["operationId"]
                 and fresh["driverDigest"] == driver
                 and fresh["sources"] == record["installed"]["sources"]
                 and _service_xml_shape(prior["xml"], bindings=True) == _service_xml_shape(actual["xml"], bindings=True),
                 "Records API/Home/installed runtime differs", "service_rebind_pending")
    wrapper_result = _service_wrappers_rebind(manager, before, current, output, driver=driver)
    merge_result = _service_merge_queue_rebind(manager, before, current, output, driver=driver)
    _require(set(before["jobs"]) == set(_SERVICE_REQUIRED_JOBS),
             "All eleven jobs must have their actual binding postconditions", "service_rebind_pending")
    _require(_read(manager.journal) == current and _read(manager.paths.state / "deployment/driver.json") == selected,
             "Replacement lifecycle changed during runtime rebind", "service_rebind_pending")
    result = {"schemaVersion": 1, "status": "runtime-bindings-verified",
              "previousRuntimeOperationId": before["runtimeOperationId"], "operationId": current["operationId"],
              "driverDigest": driver, "installedRef": _service_ref(installed_path),
              "requiredJobs": list(_SERVICE_REQUIRED_JOBS), "wrapperBindings": wrapper_result,
              "mergeQueueBinding": merge_result,
              "jobs": {name: {"apiSha256": (v := _service_job(manager, name))["apiSha256"],
                               "homeRef": v["homeRef"]} for name in before["jobs"]},
              "nativeProofSynthesized": False, "foreignConfigurationPreserved": True}
    atomic_json(output / "runtime-rebind-verification.json", result)
    return _service_ref(output / "runtime-rebind-verification.json")


