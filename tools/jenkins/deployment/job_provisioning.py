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
from .driver import verify_driver


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
    params = "".join(f"<hudson.model.StringParameterDefinition><name>{name}</name><description>Jenkins request identity</description><defaultValue></defaultValue><trim>true</trim></hudson.model.StringParameterDefinition>" for name in (
        "REPOSITORY_ID", "SESSION_ID", "REQUEST_ID", "ATTEMPT_ID", "GENERATION", "SOURCE_INPUT_DIGEST", "COVERAGE_DIGEST", "STAGE_IMPLEMENTATION_DIGEST", "EXECUTION_ID", "RECIPE_REF", "SEALED_INPUT_REF", "PATCH_OPERATION_REF", "PATCH_REQUEST_REF", "CHANGE_SET_JSON", "TEMPLATE", "RUST_EDITION", "SEALED_RUST_SOURCE", "COMMIT_AUTHORIZATION", "CLEANUP_SCOPE_JSON", "OPERATION_ID", "ALLOW_COMMIT", "BUILD_ROOT"))
    prelude = "; ".join("env.%s=%s" % (key, json.dumps(value, ensure_ascii=False)) for key, value in {
        "ZIRCON_REPO_ROOT": driver_env["ZIRCON_REPO_ROOT"], "JENKINS_PYTHON": driver_env["JENKINS_PYTHON"],
        "ZIRCON_SEALED_DRIVER": driver_env["ZIRCON_SEALED_DRIVER"], "ZIRCON_DRIVER_LAUNCHER": driver_env["ZIRCON_DRIVER_LAUNCHER"],
        "ZIRCON_DRIVER_DIGEST": driver_env["ZIRCON_DRIVER_DIGEST"], "ZIRCON_REPOSITORY_ID": hashlib.sha256(driver_env["ZIRCON_REPO_ROOT"].casefold().encode()).hexdigest(),
        "PYTHONDONTWRITEBYTECODE": "1", "ZIRCON_BUILD_ROOT": driver_env.get('ZIRCON_BUILD_ROOT', str(Path(driver_env['ZIRCON_REPO_ROOT']) / '.jenkins/builds'))}.items()) + "; "
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
