"""Read-only acceptance for a sealed input and its exact Jenkins build."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import urllib.parse

from .contracts import PilotError, RequestIdentity
from .client import JenkinsClient
from .deployment import client, verify_driver
from .governance import require_live_storage_owner
from .journal import SubmissionJournal
from .receipts import verify_receipt
from .runner import run_directory
from .snapshot import load_snapshot, portable_path, verify_materialized
from .storage import ManagedStorage, require_managed_root


def verify_build(*, root: Path, repo_root: Path, identity: RequestIdentity, job: str,
                 journal: SubmissionJournal) -> dict[str, object]:
    require_live_storage_owner(root, repo_root)
    driver_hash = verify_driver(root)
    storage = ManagedStorage(root)
    proof = json.loads(storage.path("driver-proof.json").read_bytes())
    driver = require_managed_root(proof["driverRoot"], allow_root=False)
    if not driver.is_relative_to(root) or proof.get("executionContract") != 2:
        raise PilotError("acceptance requires this root's independent trusted driver")
    python = require_installed_python(proof)
    api = client(root)
    record = api.reconcile(job, identity, journal)
    if record["state"] != "completed" or record["result"] != "SUCCESS" or not record["build_number"]:
        raise PilotError("Jenkins has no successful terminal build for this exact request")
    build_number = record["build_number"]
    _require_terminal_build(api, job, build_number, identity, record["bundle_hash"])
    receipt = api.receipt(job, build_number)
    current_run = run_directory(root, identity)
    source = current_run / "sealed/source"
    from .execution_dispatch import _key
    staged = storage.path(f"execution-inputs/{_key(identity)}/source.zip")
    sealed = load_snapshot(staged, expected_input_hash=identity.input_hash)
    if sealed.bundle_hash != record["bundle_hash"] or verify_materialized(sealed, source) != identity.input_hash:
        raise PilotError("build input no longer matches its sealed source and journal")
    if identity.template == "python-static":
        command = [str(python), "-B", str(driver / "tools/jenkins_pilot/static_validation.py"), str(source)]
    elif identity.template == "managed-cargo-check-v2":
        from .managed_cargo import FIXED_PACKAGE, toolchain_identity
        toolchain = toolchain_identity(source)
        request = json.loads((current_run / "managed-request.json").read_bytes())
        generation = require_managed_root(request["generation"]["generationPath"], allow_root=False)
        if not generation.is_relative_to(root / "cargo-generations"):
            raise PilotError("Cargo generation belongs to a different independent authority")
        command = [toolchain["binaries"]["cargo"]["path"], "check", "--locked", "-p", FIXED_PACKAGE,
                   "--target-dir", str(generation / "target")]
    else:
        raise PilotError("fault probes provide resource evidence and cannot grant command acceptance")
    artifacts: dict[str, bytes] = {}
    for item in receipt.get("artifacts", []):
        name = portable_path(item["path"])
        content, _headers, status = api.request(f"/job/{job}/{build_number}/artifact/{urllib.parse.quote(name, safe='/')}")
        if status != 200:
            raise PilotError("Jenkins archive is missing a declared validation artifact")
        artifacts[name] = content
    local_receipt = json.loads((current_run / "receipt.json").read_bytes())
    if receipt != local_receipt:
        raise PilotError("Jenkins archive and independent durable receipt differ")
    result = verify_receipt(receipt, identity=identity, job=job, build_number=build_number,
                            bundle_hash=record["bundle_hash"], command=command, artifacts=artifacts,
                            driver_input_hash=driver_hash)
    if identity.template == "managed-cargo-check-v2":
        from .managed_cargo import collect_evidence
        observed = {key: receipt[key] for key in (
            "command", "commandHash", "exitCode", "processTree", "nativeJobEvidence", "terminalErrors")}
        evidence = collect_evidence(repo_root=driver, run_root=current_run,
                                    exit_code=receipt["exitCode"], observation=observed, persist=False)
        if evidence != receipt["managedCargo"] or evidence.get("accepted") is not True:
            raise PilotError("independent Cargo policy no longer verifies the archived evidence")
        for name in ("managed-request.json", "managed-result.json", "managed-evidence.json"):
            if artifacts[name] != (current_run / name).read_bytes():
                raise PilotError("Cargo metrics archive differs from its independent durable record")
        if request["generation"].get("registrySeedHash"):
            with ManagedStorage(generation).backend() as backend:
                seed = backend.read_bytes("registry-seed.json", expected_sha256=request["generation"]["registrySeedHash"])
            if artifacts.get("registry-seed.json") != seed or (current_run / "registry-seed.json").read_bytes() != seed:
                raise PilotError("Cargo registry provenance archive differs from its bound generation")
    if verify_driver(root) != driver_hash or verify_materialized(sealed, source) != identity.input_hash:
        raise PilotError("trusted driver or source changed during acceptance")
    _require_terminal_build(api, job, build_number, identity, record["bundle_hash"])
    result.update(bundleHash=sealed.bundle_hash, driverInputHash=driver_hash,
                  receiptSha256=hashlib.sha256((current_run / "receipt.json").read_bytes()).hexdigest(),
                  queueId=record["queue_id"], job=job)
    return result


def _require_terminal_build(api: JenkinsClient, job: str, build_number: int,
                            identity: RequestIdentity, bundle_hash: str) -> None:
    # The journal preserves its original terminal observation. Acceptance also
    # requires a current observation of the exact build and uploaded input.
    build = api.api(
        f"/job/{job}/{build_number}/api/json?tree=number,building,result,actions[parameters[name,value]]")
    if (type(build.get("number")) is not int or build["number"] != build_number
            or build.get("building") is not False or build.get("result") != "SUCCESS"
            or not JenkinsClient._matches(build, identity, bundle_hash)):
        raise PilotError("current Jenkins build does not confirm this request's terminal success")


def require_installed_python(proof: dict) -> Path:
    python = Path(proof["pythonExecutable"])
    if not python.is_absolute() or hashlib.sha256(python.read_bytes()).hexdigest() != proof.get("pythonSha256"):
        raise PilotError("trusted Python executable changed")
    return python
