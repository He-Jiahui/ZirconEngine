"""Continue a prepared Flow dispatch through the verified public management entry."""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
from pathlib import Path


def _entry(repo):
    installed = json.loads((repo / ".jenkins/state/management/installed.json").read_text(encoding="utf-8-sig"))
    path = repo / "tools/jenkins-management/entry.py"
    checksum = installed["capsuleSha256"]
    if len(checksum) != 64 or any(char not in "0123456789abcdef" for char in checksum):
        raise ValueError("Invalid installed management capsule")
    capsule = repo / ".jenkins/runtime/management" / checksum
    raw = (capsule / "capsule.json").read_bytes()
    if capsule.resolve() != capsule or hashlib.sha256(raw).hexdigest() != checksum:
        raise ValueError("Pinned management capsule manifest changed")
    config = json.loads(raw)
    expected = config["clientSources"]["tools/jenkins-management/entry.py"]
    if "clientRoot" in config:
        runner_path = capsule / "runner.py"
        if (config.get("repositoryRoot") != str(repo)
                or Path(installed.get("capsule", "")) != capsule
                or runner_path.resolve() != runner_path.absolute()
                or hashlib.sha256(runner_path.read_bytes()).hexdigest() != config["files"]["runner.py"]):
            raise ValueError("Pinned management client selector changed")
        runner_spec = importlib.util.spec_from_file_location("flow_dispatch_client_selector", runner_path)
        runner = importlib.util.module_from_spec(runner_spec)
        runner_spec.loader.exec_module(runner)
        path = runner.client_path(config, repo, "tools/jenkins-management/entry.py", capsule=capsule)
    if path.resolve() != path.absolute() or hashlib.sha256(path.read_bytes()).hexdigest() != expected:
        raise ValueError("Pinned management entry implementation changed")
    spec = importlib.util.spec_from_file_location("flow_dispatch_public_entry", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def resume_flow(repo, payload, *, query_only=False, entry_module=None):
    """Read original IDs; the public entry and capsule own all write authority."""
    root = Path(repo).absolute()
    entry = entry_module or _entry(root)
    context = entry.capability(root)
    from tools.jenkins.contracts import JenkinsError, digest
    config = context["config"]
    identity, reference = payload.get("identity"), payload.get("currentSourceRef")
    if (not isinstance(identity, dict) or not isinstance(reference, str) or len(reference) != 64
            or any(char not in "0123456789abcdef" for char in reference)):
        raise JenkinsError("execution_resume_identity_mismatch", "A prepared Flow identity and current-source reference are required")
    helper = context["module"].source_helper()
    state = helper._managed_route_state(root, None, query_only=True)
    if state is None:
        raise JenkinsError("execution_resume_identity_mismatch", "The original Flow authority is unavailable")
    key = ":".join(str(identity.get(name, "")) for name in ("sessionId", "requestId", "attemptId"))
    with state.connect() as connection:
        connection.execute("PRAGMA query_only=ON")
        connection.execute("BEGIN")
        intent_row = state.get("current_source_intent", reference, connection=connection)
        workflow_row = state.get("workflow", key, connection=connection)
        intent = intent_row["payload"] if intent_row else {}
        workflow = workflow_row["payload"] if workflow_row else {}
        author = intent.get("identity", {})
        if (not intent_row or digest(intent) != reference or not workflow_row
                or workflow.get("identity") != identity
                or any(identity.get(name) != value for name, value in author.items())
                or intent.get("repositoryRoot") != str(root)
                or author.get("stageImplementationDigest") != config["driverDigest"]
                or intent.get("runtimeOperationId") != config["runtimeOperationId"]):
            raise JenkinsError("execution_resume_identity_mismatch", "The Pipeline must retain the original prepared author")
        execution, recipe = workflow.get("executionId"), workflow.get("recipeRef")
        if not isinstance(execution, str) or len(execution) != 32 or not isinstance(recipe, str) or len(recipe) != 64:
            raise JenkinsError("execution_resume_identity_mismatch", "No authoritative execution claim is registered")
        claim = state.get("execution", execution, connection=connection)
        plan = state.get("recipe_plan", recipe, connection=connection)
        consumer = state.get("execution_consumer", execution + ":" + key, connection=connection)
        if (not claim or not plan or not consumer
                or claim["payload"].get("recipeRef") != recipe
                or plan["payload"].get("recipeDigest") != recipe
                or digest(plan["payload"].get("recipe")) != recipe
                or consumer["payload"].get("consumerId") != key
                or consumer["payload"].get("executionId") != execution):
            raise JenkinsError("execution_resume_identity_mismatch", "The original recipe and consumer must bind this claim")
        marker = ":".join([author["repositoryId"], author["sessionId"], author["requestId"],
                           author["requestId"] + ":execution:" + recipe[:16]])
        prior = state.get("jenkins_submission", marker, connection=connection)
        marker_status = prior["payload"].get("delivery") if prior else None
        unresolved = bool(prior and marker_status not in {"submitted", "accepted", "completed"})
        connection.rollback()
    token = digest({"identity": author, "currentSourceRef": reference, "executionId": execution,
                    "recipeRef": recipe, "kind": "prepared-flow-execution-dispatch"})[:32]
    management_identity = {**author, "requestId": "flow-dispatch-" + token,
                           "operationId": "flow-dispatch-" + token, "action": "submit-flow"}
    request = {name: management_identity[name] for name in
               ("repositoryId", "sessionId", "requestId", "operationId", "action")}
    request.update(driverDigest=config["driverDigest"], runtimeOperationId=config["runtimeOperationId"],
        ownedPaths=list(intent["ownedPaths"]), worktreeId=intent["worktreeId"], buildRoot=intent["buildRoot"],
        flowPayload={"identity": author, "currentSourceRef": reference,
                     "resumeExecution": {"executionId": execution, "recipeRef": recipe, "phase": "dispatch"}})
    delivered = entry.dispatch(root, request, query_only=query_only or unresolved)
    result = delivered.get("result") if delivered.get("phase") == "complete" else None
    if result is not None and not unresolved:
        if (result.get("status") != "submitted" or result.get("executionId") != execution
                or result.get("recipeRef") != recipe):
            raise JenkinsError("execution_resume_identity_mismatch", "Continuation result differs from the original claim")
        return {"status": "complete", "executionId": execution, "recipeRef": recipe,
                "managementIdentity": management_identity, "originalDeliveryState": marker_status,
                "formalAcceptance": False}
    if unresolved or delivered.get("phase") in {"unknown", "delivery_unknown", "dispatching", "failed"} or delivered.get("status") in {"failed", "unknown"}:
        return {"status": "unknown", "reasonCode": delivered.get("reasonCode", "delivery_unknown"),
                "executionId": execution, "recipeRef": recipe, "managementIdentity": management_identity,
                "originalDeliveryState": marker_status, "formalAcceptance": False}
    return {"status": "pending", "executionId": execution, "recipeRef": recipe,
            "managementIdentity": management_identity, "originalDeliveryState": marker_status,
            "formalAcceptance": False}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", required=True)
    parser.add_argument("--input-file", required=True)
    parser.add_argument("--output-file", required=True)
    parser.add_argument("--expected-bridge-sha256", required=True)
    parser.add_argument("--query-only", action="store_true")
    args = parser.parse_args(argv)
    root = Path(args.repo_root).absolute()
    script = root / "tools/dev/resume_jenkins_flow.py"
    if (Path(__file__).absolute() != script or script.resolve() != script
            or hashlib.sha256(script.read_bytes()).hexdigest() != args.expected_bridge_sha256):
        raise ValueError("Fixed Flow dispatch bridge implementation changed")
    controls = root / ".jenkins/state/controls"
    paths = [Path(args.input_file).absolute(), Path(args.output_file).absolute()]
    for path in paths:
        if (path.parent != controls or path.resolve() != path
                or not path.name.startswith("flow-execution-resume-") or path.suffix != ".json"):
            raise ValueError("Bridge protocol files require their fixed control namespace")
    if paths[0] == paths[1]:
        raise ValueError("Bridge input and output must remain distinct")
    if paths[0].stat().st_size > 65536:
        raise ValueError("Bridge control input exceeds its bound")
    payload = json.loads(paths[0].read_text(encoding="utf-8-sig"))
    try:
        result = resume_flow(root, payload, query_only=args.query_only)
    except Exception as error:
        code = getattr(error, "code", "flow_resume_bridge_failed")
        result = {"status": "unknown" if "unknown" in code else "failed",
                  "reasonCode": code, "message": str(error)[:1024], "formalAcceptance": False}
    paths[1].write_text(json.dumps(result, ensure_ascii=False) + "\n", encoding="utf-8")
    return 0 if result["status"] in {"complete", "pending"} else 2


if __name__ == "__main__":
    raise SystemExit(main())
