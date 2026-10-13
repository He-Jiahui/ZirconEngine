"""Jenkins offline deployment executor for one bounded recovery admission fix.

Reuse the selected, tested support closure and the original reviewed request.
Only immutable Jenkins operators and evidence are installed. Product source,
Git, builds, queue replay and service activation are outside this preparation.
"""
from __future__ import annotations

import ast
import contextlib
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import shutil
import sys
import time
import uuid

REPO = Path(__file__).absolute().parents[2]
STATE = REPO / ".jenkins/state"
ROOT = STATE / "migration-baseline"
DIGEST = "121af96cb9d4e5dce3f3e830373203952324f380aa4b0ed9493f16ee783d4ecf"
OLD_ROOT = REPO / ".jenkins/runtime/drivers" / DIGEST
REQUEST_ID = "53ae4a242d234da89f8a499e2d8ef7d3"
RELEASE_REF = {
    "path": str(STATE / "deployment" / ("lost-owner-recovery-release-current-driver-" + REQUEST_ID + "-revision2.json")),
    "sha256": "9b669b7bce3f01e6ef6831828918328d492599b521b37523eef0cd0092e0908c",
}
DELTA = "tools/jenkins/deployment/lost_recovery_runtime.py"
TARGETS = ["tools/jenkins/tests/test_lost_recovery_runtime.py",
           "tools/jenkins/tests/test_lost_recovery_runtime_lineage.py"]
OUTPUT = Path("E:/cargo-targets/zircon-local/reports/jenkins-transition-monitor/operator/user-start-20261013T031729Z")
TMP = OUTPUT / "contention-tests"
sys.dont_write_bytecode = True
os.environ.update(PYTHONDONTWRITEBYTECODE="1", PYTEST_DISABLE_PLUGIN_AUTOLOAD="1",
                  TEMP=str(TMP), TMP=str(TMP), TMPDIR=str(TMP))
sys.path.insert(0, str(OLD_ROOT))
from tools.jenkins.contracts import canonical_json, digest
from tools.jenkins.deployment.driver import _plain, verify_driver
from tools.jenkins.deployment.support_regression import verify_support_regression


def need(condition, message):
    if not condition:
        raise RuntimeError(message)


def raw(path):
    path = Path(path).absolute()
    _plain(path)
    return path.read_bytes()


def sha(path):
    return hashlib.sha256(raw(path)).hexdigest()


def ref(path):
    return {"path": str(path), "sha256": sha(path)}


def read(path):
    return json.loads(raw(path))


def immutable(path, value):
    data = canonical_json(value) + b"\n"
    with path.open("xb") as stream:
        stream.write(data)
        stream.flush()
        os.fsync(stream.fileno())
    need(raw(path) == data, "Immutable deployment write changed")
    return ref(path)


def main():
    owner = read(OUTPUT.parents[1] / "artifacts.json")
    need("01a12af6-7511-70d1-a452-61b80678eb56" in str(owner["owner"]), "Operator output ownership changed")
    registered = next(row for row in owner["operator_requests"] if row.get("id") == OUTPUT.name)
    need(str(TMP) in registered["repair_outputs"], "Register deployment test outputs before preparation")
    baseline_driver = verify_driver(OLD_ROOT, expected_digest=DIGEST)
    need(sha(RELEASE_REF["path"]) == RELEASE_REF["sha256"], "Original reviewed release changed")
    old_release = read(RELEASE_REF["path"])
    baseline_ref = old_release["regression"]
    baseline = verify_support_regression(REPO, baseline_ref, baseline_driver, current=False)
    old_preparation = read(old_release["preparation"]["path"])
    need(sha(old_release["preparation"]["path"]) == old_release["preparation"]["sha256"], "Original preparation changed")
    need(old_preparation["driver"] == baseline_driver and old_preparation["regression"] == baseline_ref,
         "Bounded recovery baseline differs")
    selector_before = raw(STATE / "deployment/driver.json")
    need(read(STATE / "deployment/driver.json") == old_preparation["previousSelectedDriver"], "Selected predecessor changed")
    pins = {name: sha(REPO / name) for name in [DELTA, *TARGETS, "tools/jenkins/tests/__init__.py",
        "tools/dev/jenkins_recovery_keeper.py", "tools/dev/start-jenkins-coordinator.ps1", ".codex/config.toml",
        ".jenkins/deployment-spec.json", ".jenkins/workflow-spec.json", "tools/dev/prepare_jenkins_runtime_contention_recovery.py"]}
    manifest = read(OLD_ROOT / "driver-manifest.json")
    sources = {name: raw(OLD_ROOT / name) for name in manifest["files"]}
    sources[DELTA] = raw(REPO / DELTA)
    need(hashlib.sha256(sources[DELTA]).hexdigest() != manifest["files"][DELTA]["sha256"], "Admission repair is missing")
    for name, content in sources.items():
        if name.endswith(".py"):
            ast.parse(content, filename=name)
    files = {name: {"bytes": len(content), "sha256": hashlib.sha256(content).hexdigest()}
             for name, content in sorted(sources.items())}
    new_digest = digest({"schemaVersion": 1, "files": files})
    destination = OLD_ROOT.parent / new_digest
    _plain(destination)
    if not destination.exists():
        staging = OLD_ROOT.parent / (".incoming-runtime-contention-" + uuid.uuid4().hex)
        _plain(staging)
        staging.mkdir()
        try:
            for name, content in sources.items():
                path = staging / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(content)
            (staging / "driver-manifest.json").write_bytes(canonical_json({"schemaVersion": 1,
                "driverDigest": new_digest, "files": files}) + b"\n")
            rename_deadline = time.monotonic() + 10
            while True:
                try:
                    os.rename(staging, destination)
                    break
                except PermissionError:
                    # Windows directory observers may briefly withhold rename.
                    # Keep the same physical, fully pinned deployment staging;
                    # never change ACLs or replace an existing candidate.
                    _plain(staging)
                    _plain(destination)
                    if destination.exists():
                        verify_driver(destination, expected_digest=new_digest)
                        break
                    if time.monotonic() >= rename_deadline:
                        raise
                    time.sleep(.25)
        finally:
            if staging.exists():
                _plain(staging)
                need(staging.resolve().parent == OLD_ROOT.parent.resolve()
                     and staging.name.startswith(".incoming-runtime-contention-"), "Owned deployment staging escaped its root")
                shutil.rmtree(staging)
    candidate = verify_driver(destination, expected_digest=new_digest)
    need({name for name in sources if sources[name] != raw(OLD_ROOT / name)} == {DELTA}, "Unexpected runtime change")
    inventory = dict(baseline["sourceFiles"])
    for name, entry in files.items():
        if name.startswith(("tools/jenkins/", "pipeline/")):
            inventory[".jenkins/" + name if name.startswith("pipeline/") else name] = entry["sha256"]
    for name in TARGETS:
        inventory[name] = pins[name]
    need(inventory["tools/jenkins/tests/__init__.py"] == pins["tools/jenkins/tests/__init__.py"], "Test support changed")
    for name in (".jenkins/deployment-spec.json", ".jenkins/workflow-spec.json"):
        need(inventory[name] == pins[name], "Tested specifications changed")
    for name in list(sys.modules):
        if name == "tools" or name.startswith("tools."):
            del sys.modules[name]
    sys.path[:] = [str(destination),
                  *[p for p in sys.path if p not in (str(OLD_ROOT), str(REPO))]]
    import tools.jenkins
    package_spec = importlib.util.spec_from_file_location("tools.jenkins.tests", REPO / "tools/jenkins/tests/__init__.py",
        submodule_search_locations=[str(REPO / "tools/jenkins/tests")])
    package = importlib.util.module_from_spec(package_spec)
    sys.modules["tools.jenkins.tests"] = package
    tools.jenkins.tests = package
    package_spec.loader.exec_module(package)
    import pytest

    class Evidence:
        def __init__(self):
            self.nodes, self.reports, self.errors = [], [], []
        def pytest_collection_finish(self, session):
            self.nodes = [item.nodeid for item in session.items]
        def pytest_collectreport(self, report):
            if report.failed:
                self.errors.append({"test": report.nodeid, "reason": str(report.longrepr)})
        def pytest_runtest_logreport(self, report):
            self.reports.append({"test": report.nodeid, "phase": report.when, "outcome": report.outcome,
                                 "reason": str(report.longrepr) if report.longrepr else None})

    started, tag = time.time(), str(time.time_ns())
    _plain(TMP)
    TMP.mkdir(parents=True, exist_ok=True)
    evidence, output = Evidence(), io.StringIO()
    targets = [str(REPO / name) for name in TARGETS]
    print(json.dumps({"status": "focused-regression-started", "driverDigest": new_digest}), flush=True)
    with contextlib.redirect_stdout(output), contextlib.redirect_stderr(output):
        exit_code = int(pytest.main([*targets, "-q", "--assert=plain", "--import-mode=importlib",
            "-p", "no:cacheprovider", "--basetemp", str(TMP / ("admission-" + tag))], plugins=[evidence]))
    stable = selector_before == raw(STATE / "deployment/driver.json") and all(sha(REPO / name) == value for name, value in pins.items())
    verify_driver(destination, expected_digest=new_digest)
    for name, module in tuple(sys.modules.items()):
        if name == "tools" or name.startswith("tools."):
            location = getattr(module, "__file__", None)
            need(location is not None and (Path(location).absolute().is_relative_to(destination)
                 or name.startswith("tools.jenkins.tests") and Path(location).absolute().is_relative_to(REPO / "tools/jenkins/tests")),
                 "Focused tests imported a different implementation: " + name)
            if name.startswith("tools.jenkins.tests"):
                relative = Path(location).absolute().relative_to(REPO).as_posix()
                need(sha(location) == inventory.get(relative), "Imported canonical test changed: " + relative)
    failures = sum(r["phase"] == "call" and r["outcome"] == "failed" for r in evidence.reports)
    errors = len(evidence.errors) + sum(r["phase"] != "call" and r["outcome"] == "failed" for r in evidence.reports)
    log = ROOT / ("runtime-contention-focus-" + tag + ".log")
    with log.open("x", encoding="utf-8") as stream:
        stream.write(output.getvalue())
    focus = {"schemaVersion": 1, "scope": "explicit-current-Jenkins-support-pytest-modules",
        "status": "passed" if exit_code == 0 and stable else "failed", "startedAt": started, "finishedAt": time.time(),
        "exitCode": exit_code, "testsRun": len(evidence.nodes), "failures": failures, "errors": errors,
        "collectionErrors": evidence.errors, "collectedNodes": evidence.nodes, "reports": evidence.reports,
        "testTargets": targets, "sourceFiles": inventory, "sourceFilesAfter": inventory if stable else {},
        "sourceIdentityStable": stable, "log": str(log), "logSha256": sha(log), "testedDriver": candidate,
        "runner": ref(Path(__file__).absolute()), "canonicalOperatorInputs": pins, "baseline": baseline_ref,
        "runtimeAcceptance": False, "cargoAcceptance": False, "currentCanonicalFullDiscovery": False,
        "pytestVersion": pytest.__version__, "python": sys.executable}
    focus_ref = immutable(ROOT / ("runtime-contention-focus-" + tag + ".json"), focus)
    if focus["status"] != "passed":
        print(json.dumps({"status": "failed", "focusedRegression": focus_ref, "testsRun": focus["testsRun"],
                          "failures": failures, "errors": errors}), flush=True)
        return 2
    changed = sorted(name for name in baseline["sourceFiles"].keys() | inventory.keys()
                     if baseline["sourceFiles"].get(name) != inventory.get(name))
    scoped = {**{key: focus[key] for key in ("status", "sourceIdentityStable", "testsRun", "testTargets", "log", "logSha256",
        "exitCode", "failures", "errors", "collectionErrors")}, "schemaVersion": 1,
        "scope": "baseline-plus-focused-Jenkins-support", "baseRegression": baseline_ref, "focusedRegression": focus_ref,
        "sourceFilesBefore": baseline["sourceFiles"], "sourceFiles": inventory, "changedFiles": changed,
        "fullDiscoveryRun": False, "runtimeAcceptance": False, "cargoAcceptance": False, "milestoneAcceptance": False}
    scoped_ref = immutable(ROOT / ("baseline-runtime-contention-delta-" + tag + ".json"), scoped)
    from tools.jenkins.deployment.support_regression import verify_support_regression as verify_candidate_regression
    verify_candidate_regression(REPO, scoped_ref, candidate, current=False)
    from tools.jenkins.deployment.manager import DeploymentManager
    from tools.jenkins.deployment.paths import resolve_paths
    from tools.jenkins.deployment.spec import load_spec
    from tools.jenkins.deployment.execution_activation import protected_holds
    from tools.jenkins.deployment.recovery_keeper import _release
    from tools.jenkins.deployment.lost_recovery_runtime import prepare_lost_recovery_runtime
    from tools.jenkins.state.locks import process_lock
    manager = DeploymentManager(load_spec(REPO / ".jenkins/deployment-spec.json"), resolve_paths(load_spec(REPO / ".jenkins/deployment-spec.json")))
    need(raw(STATE / "deployment/driver.json") == selector_before, "Selector changed after regression")
    need(protected_holds(STATE) == old_preparation["protectedHolds"], "Historical holds changed")
    preparation_ref = immutable(ROOT / ("prepared-runtime-contention-" + new_digest + "-" + tag + ".json"),
        {**old_preparation, "preparedAt": time.time(), "driver": candidate, "regression": scoped_ref,
         "baselineDriver": baseline_driver, "deltaFiles": [DELTA], "affectedTests": TARGETS,
         "boundedExecutor": ref(Path(__file__).absolute()), "newServiceStarted": False})
    with process_lock(STATE / "deployment-start.lock", reason="deployment_start_busy"):
        marker = old_release["lostRecoveryRuntime"]
        plan = prepare_lost_recovery_runtime(manager, request_id=REQUEST_ID, scope=old_release["scope"],
            original_command_ref=marker["originalCommand"], original_descriptor_ref=marker["originalDescriptor"], reviewed_plan_ref=marker["plan"])
        need(plan.document == read(marker["plan"]["path"]), "Original reviewed plan changed")
        operators = dict(old_release["operatorFiles"])
        for name, entry in files.items():
            key = str(REPO / name)
            if key in operators:
                operators[key] = entry["sha256"]
        release_ref = immutable(STATE / "deployment" / ("lost-owner-recovery-release-runtime-contention-" + REQUEST_ID + "-" + tag + ".json"),
            {**old_release, "preparedAt": time.time(), "targetDriver": candidate, "actorCodeRoot": candidate["root"],
             "operatorFiles": operators, "regression": scoped_ref, "preparation": preparation_ref})
        _release(manager, release_ref, current_source=False)
    need(raw(STATE / "deployment/driver.json") == selector_before, "Preparation must not select a driver")
    result = {"status": "prepared-not-applied", "originalRequestId": REQUEST_ID, "driverDigest": new_digest,
        "focusedRegression": focus_ref, "regression": scoped_ref, "release": release_ref, "preparation": preparation_ref,
        "testsRun": focus["testsRun"], "changedFiles": changed, "serviceLifecycleExecuted": False,
        "runtimeAcceptance": False, "cargoAcceptance": False}
    immutable(OUTPUT / "repair-contention-preparation.json", result)
    print(json.dumps(result), flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
