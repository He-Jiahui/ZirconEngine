"""Install the trusted glue into the isolated controller and Windows agent."""
from __future__ import annotations

import json
import hashlib
import sys
import urllib.parse
from pathlib import Path

from .client import JenkinsClient
from .contracts import PilotError, canonical_json, identifier
from .jenkins_config import read_credentials, read_manifest, PilotPaths, write_manifest
from .snapshot import capture, load_snapshot, materialize, verify_materialized
from .storage import ManagedStorage

DEFAULT_JOB = "zircon-pilot"


def groovy_literal(value: str) -> str:
    """Single quotes keep Pipeline ${...} expressions for Pipeline evaluation."""
    return "'" + value.replace("\\", "\\\\").replace("'", "\\'").replace("\r", "\\r").replace("\n", "\\n") + "'"


def client(root: Path) -> JenkinsClient:
    return JenkinsClient(str(read_manifest(root)["controllerUrl"]), **read_credentials(root))


def seal_driver(root: Path, repo_root: Path) -> dict[str, object]:
    files = [path.relative_to(repo_root).as_posix()
             for folder in ("tools/jenkins_pilot",)
             for path in (repo_root / folder).rglob("*.py")
             if "tests" not in path.relative_to(repo_root / folder).parts]
    files.append("tools/jenkins_pilot/pipeline.groovy")
    sealed = capture(repo_root, ManagedStorage(root), paths=files, untracked_allowlist=files)
    destination = root / ("driver-" + sealed.bundle_hash[:16])
    if destination.exists():
        verify_materialized(sealed, destination)
    else:
        materialize(sealed.bundle, destination, repo_root=repo_root, expected_input_hash=sealed.input_hash)
    proof = {"driverRoot": str(destination), "driverBundle": str(sealed.bundle), "driverInputHash": sealed.input_hash,
             "driverBundleHash": sealed.bundle_hash, "repoRoot": str(repo_root),
             "pipelineSha256": hashlib.sha256(Path(__file__).with_name("pipeline.groovy").read_bytes()).hexdigest(),
             "pythonExecutable": str(Path(sys.executable)),
             "pythonSha256": hashlib.sha256(Path(sys.executable).read_bytes()).hexdigest(),
             "executionContract": 2}
    ManagedStorage(root).atomic_write("driver-proof.json", canonical_json(proof))
    manifest = read_manifest(root)
    manifest.update(proof)
    write_manifest(PilotPaths(root), manifest)
    return proof


def verify_driver(root: Path) -> str:
    proof = json.loads((root / "driver-proof.json").read_bytes())
    sealed = load_snapshot(Path(proof["driverBundle"]), expected_input_hash=proof["driverInputHash"])
    if sealed.bundle_hash != proof["driverBundleHash"]:
        raise PilotError("trusted driver archive changed")
    return verify_materialized(sealed, Path(proof["driverRoot"]))


def install_job(root: Path, *, job: str = DEFAULT_JOB) -> dict[str, object]:
    identifier(job, "job")
    pipeline = Path(__file__).with_name("pipeline.groovy").read_text(encoding="utf-8")
    proof = json.loads((root / "driver-proof.json").read_bytes())
    if proof.get("executionContract") != 2 or hashlib.sha256(pipeline.encode("utf-8")).hexdigest() != proof.get("pipelineSha256"):
        raise PilotError("Pipeline is not bound to the current independent driver")
    script = """import jenkins.model.Jenkins
import hudson.model.*
import org.jenkinsci.plugins.workflow.job.WorkflowJob
import org.jenkinsci.plugins.workflow.job.properties.DisableConcurrentBuildsJobProperty
import org.jenkinsci.plugins.workflow.cps.CpsFlowDefinition
import org.jenkinsci.plugins.workflow.flow.FlowDurabilityHint
import org.jenkinsci.plugins.workflow.job.properties.DurabilityHintJobProperty
import io.jenkins.plugins.file_parameters.StashedFileParameterDefinition
def j=Jenkins.get()
def name=JOB
def item=j.getItem(name)
if(item==null) item=j.createProject(WorkflowJob,name)
if(item.isBuilding()) throw new IllegalStateException('active pilot build prevents configuration changes')
def params=['REQUEST_ID','SESSION_ID','ATTEMPT_ID','GENERATION','INPUT_HASH','BUNDLE_HASH','TEMPLATE'].collect { new StringParameterDefinition(it,'') }
params.add(new StashedFileParameterDefinition('SOURCE_BUNDLE'))
item.removeProperty(ParametersDefinitionProperty.class)
item.addProperty(new ParametersDefinitionProperty(params))
item.addProperty(new DisableConcurrentBuildsJobProperty())
item.addProperty(new DurabilityHintJobProperty(FlowDurabilityHint.MAX_SURVIVABILITY))
item.setDefinition(new CpsFlowDefinition(PIPELINE,true))
item.save()
println('pilot job installed')
""".replace("JOB", json.dumps(job)).replace("PIPELINE", groovy_literal(pipeline))
    data, _, status = client(root).request("/scriptText", post=True, content_type="application/x-www-form-urlencoded",
                                         data=urllib.parse.urlencode({"script": script}).encode())
    if status != 200 or data.strip() != b"pilot job installed":
        raise PilotError("Jenkins did not install the exact pilot job: " + data.decode("utf-8", errors="replace")[:1500])
    return {"job": job, "url": client(root).url + "/job/" + job + "/"}
