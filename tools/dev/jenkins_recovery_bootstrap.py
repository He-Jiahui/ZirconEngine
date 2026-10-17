"""Bounded offline regression and preparation for the stopped Jenkins runtime.

This is recovery evidence, not Jenkins execution or runtime acceptance. It never
changes the selected driver, configuration, Git, queues or historical requests.
The normal strict recovery entry remains responsible for actually starting it.
"""
from __future__ import annotations

import argparse
from concurrent.futures import ThreadPoolExecutor, as_completed
import contextlib
import hashlib
import io
import json
import os
from pathlib import Path
import sys
import time

REPO = Path(__file__).absolute().parents[2]
STATE = REPO / ".jenkins/state"
HISTORY = STATE / "migration-baseline"
REPORT = Path("E:/cargo-targets/zircon-local/reports/disk-governance-20261015")
OUTPUT = Path("E:/cargo-targets/zircon-local/tests/gr-20261015")
ORIGINAL = STATE / "deployment/owned-recovery-upgrade-release-2d8b05eccee1de1a-1792055121191311000.json"
SESSION = "01a13b17-3395-72b1-8100-6cc3f5400796"
sys.dont_write_bytecode = True
sys.path.insert(0, str(REPO))


def need(condition, message):
    if not condition:
        raise RuntimeError(message)


def raw(path):
    from tools.jenkins.deployment.driver import _plain
    path = Path(path).absolute()
    _plain(path)
    return path.read_bytes()


def sha(path):
    return hashlib.sha256(raw(path)).hexdigest()


def read(path):
    return json.loads(raw(path))


def reference(path):
    return {"path": str(Path(path).absolute()), "sha256": sha(path)}


def immutable(path, value):
    from tools.jenkins.contracts import canonical_json
    data = canonical_json(value) + b"\n"
    with path.open("xb") as stream:
        stream.write(data)
        stream.flush()
        os.fsync(stream.fileno())
    need(raw(path) == data, "Immutable recovery evidence changed")
    return reference(path)


def manager():
    from tools.jenkins.deployment.manager import DeploymentManager
    from tools.jenkins.deployment.paths import resolve_paths
    from tools.jenkins.deployment.spec import load_spec
    spec = load_spec(REPO / ".jenkins/deployment-spec.json")
    return DeploymentManager(spec, resolve_paths(spec))


def stopped():
    from tools.jenkins.deployment.lost_owner import _lifecycle_fence
    from tools.jenkins.deployment.lifecycle import check_port_available
    m = manager()
    operation, host = read(STATE / "deployment-operation.json"), read(STATE / "deployment/host.json")
    need(operation.get("state") == host.get("status") == "stopped", "The original deployment is no longer stopped")
    need(operation["operationId"] == host["operationId"], "Stopped runtime ownership differs")
    for record in (operation, host):
        for key in ("nativeTerminationProof", "agentTerminationProof"):
            proof = record.get(key, {})
            need(proof.get("complete") is True and proof.get("childrenGone") is True
                 and proof.get("activeProcesses") == 0 and proof.get("stdoutEof") is True
                 and proof.get("stderrEof") is True, "Native predecessor termination is incomplete")
    check_port_available(m.spec.controller["listenAddress"], int(m.spec.controller["httpPort"]))
    original = read(ORIGINAL)
    for area in ("requests", "results"):
        need(not (STATE / "recovery-lifecycle" / area / (original["requestId"] + ".json")).exists(),
             "Original upgrade has been admitted; reconcile it instead of preparing a retry")
    return _lifecycle_fence(m)


def register():
    """Register every output before creating fixtures or evidence."""
    from tools.jenkins.state.atomic import atomic_json
    from tools.jenkins.state.locks import process_lock
    manifest = REPORT / "artifacts.json"
    with process_lock(REPORT / "recovery-registration.lock", reason="recovery_registration_busy"):
        value = read(manifest)
        identity = {"sessionId": SESSION, "operationId": "jenkins-recovery-20261015-1",
                    "originalUpgrade": reference(ORIGINAL), "runner": reference(Path(__file__))}
        for path, role in ((OUTPUT, "offline-recovery-test-fixtures"),
                           (REPORT / "jenkins-recovery-20261015-1", "offline-recovery-evidence")):
            existing = next((row for row in value["artifacts"] if row["path"] == str(path)), None)
            if existing:
                prior = existing.get("dependencies", {})
                need(all(prior.get(key) == identity[key] for key in ("sessionId", "operationId", "originalUpgrade")),
                     "Existing recovery output belongs to a different owner or request")
                if prior != identity:
                    existing.setdefault("dependencyHistory", []).append(prior)
                    existing["dependencies"] = identity
            else:
                value["artifacts"].append({"path": str(path), "role": role, "dependencies": identity,
                    "retainUntil": "native termination, strict restoration and Jenkins rerun reconciled"})
        atomic_json(manifest, value)
    for path in (OUTPUT, REPORT / "jenkins-recovery-20261015-1"):
        path.mkdir(parents=True, exist_ok=True)


def pytest_run(targets, temp, *, collect=False):
    sys.path.insert(0, str(REPO / ".jenkins/runtime/test-deps-coordinator"))
    import pytest

    class Evidence:
        def __init__(self):
            self.nodes, self.reports, self.errors = [], [], []
            self.completed = 0

        def pytest_collection_finish(self, session):
            self.nodes = [item.nodeid for item in session.items]

        def pytest_collectreport(self, report):
            if report.failed:
                self.errors.append({"test": report.nodeid, "reason": str(report.longrepr)})

        def pytest_runtest_logreport(self, report):
            self.reports.append({"test": report.nodeid, "phase": report.when, "outcome": report.outcome,
                "reason": str(report.longrepr) if report.longrepr else None, "duration": float(report.duration)})
            if report.when == "teardown":
                self.completed += 1
                if self.completed % 25 == 0:
                    print(json.dumps({"completedTests": self.completed, "lastTest": report.nodeid}),
                          file=sys.__stdout__, flush=True)

    temp.mkdir(parents=True, exist_ok=True)
    fixtures = temp / "gov"
    fixtures.mkdir(exist_ok=True)
    os.environ.update(TEMP=str(temp), TMP=str(temp), TMPDIR=str(temp), ZGOV_TEST_ROOT=str(fixtures),
        JOB_NAME="zircon-governance-offline-recovery", PYTEST_DISABLE_PLUGIN_AUTOLOAD="1", PYTHONDONTWRITEBYTECODE="1")
    evidence, output = Evidence(), io.StringIO()
    args = [*targets, "-q", "--assert=plain", "-p", "no:cacheprovider", "--basetemp", str(temp / "pytest"),
            "--rootdir", str(REPO), "--import-mode=importlib", "-o", "consider_namespace_packages=true"]
    if collect:
        args.append("--collect-only")
    with contextlib.redirect_stdout(output), contextlib.redirect_stderr(output):
        code = int(pytest.main(args, plugins=[evidence]))
    return code, evidence, output.getvalue()


def worker(path):
    from tools.jenkins.deployment.support_regression import support_sources
    task = read(path)
    need(path.parent == OUTPUT and str(path).endswith(".task.json"), "Worker task escaped registered recovery outputs")
    need(task["runner"] == reference(Path(__file__)) and task["sources"] == support_sources(REPO),
         "Recovery test inputs changed before worker admission")
    code, evidence, output = pytest_run(task["nodes"], Path(task["temp"]))
    log = path.with_suffix(".log")
    log.write_text(output, encoding="utf-8")
    immutable(path.with_suffix(".result.json"), {"exitCode": code, "nodes": evidence.nodes,
        "reports": evidence.reports, "collectionErrors": evidence.errors,
        "sourceIdentityStable": task["sources"] == support_sources(REPO), "log": reference(log)})
    return code


def validate():
    from tools.jenkins.deployment.support_regression import support_sources, _EXPANDED_FULL_ROOTS, _results
    from tools.jenkins.processes.job import NativeJob
    from tools.jenkins.state.locks import process_lock
    register()
    # Opening for update without writing proves that this actual parent is strict.
    denied = False
    try:
        with (REPO / ".codex/config.toml").open("r+b"):
            pass
    except PermissionError:
        denied = True
    need(denied, "Run offline bootstrap inside the official zircon-strict sandbox")
    need(not (OUTPUT / "regression.json").exists(), "Original offline regression exists; inspect its result")
    with process_lock(STATE / "deployment-start.lock", reason="deployment_start_busy"):
        fence, sources, runner = stopped(), support_sources(REPO), reference(Path(__file__))
        roots = [str(REPO / name) for name in _EXPANDED_FULL_ROOTS]
        code, discovery, log = pytest_run(roots, OUTPUT / "collect", collect=True)
        (OUTPUT / "collection.log").write_text(log, encoding="utf-8")
        need(code == 0 and discovery.nodes and not discovery.errors, "Complete recovery test discovery failed")
        need(len(discovery.nodes) == len(set(discovery.nodes)), "Duplicate support test discovery")
        old = read(HISTORY / "full-support-admission-20261015-r10.json")
        durations = {}
        for report in old.get("reports", []):
            node = report["test"]
            durations[node] = durations.get(node, 0) + report.get("duration", 0)
        batches, loads = [[] for _ in range(4)], [0.0] * 4
        for node in sorted(discovery.nodes, key=lambda n: (-durations.get(n, 1), n)):
            index = min(range(4), key=loads.__getitem__)
            batches[index].append(node)
            loads[index] += durations.get(node, 1)
        tasks = []
        for index, nodes in enumerate(batches):
            path = OUTPUT / f"w{index}.task.json"
            immutable(path, {"runner": runner, "sources": sources, "nodes": nodes, "temp": str(OUTPUT / f"w{index}")})
            tasks.append(path)
        print(json.dumps({"status": "offline-recovery-tests-started", "tests": len(discovery.nodes),
                          "workers": 4, "jenkinsExecution": False}), flush=True)

        def launch(path):
            job = NativeJob.launch([sys.executable, "-B", "-X", "utf8", str(Path(__file__).absolute()),
                "--worker", str(path)], cwd=REPO, env=os.environ.copy(), log_dir=path.with_suffix(".native"))
            try:
                result = job.wait(timeout_seconds=3600, cleanup_descendants=True)
                proof = job.proof().to_dict()
                immutable(path.with_suffix(".terminal.json"), proof)
                need(proof["complete"], "Offline recovery test descendants are not terminal")
                print(json.dumps({"worker": path.stem, "exitCode": result, "nativeComplete": proof["complete"]}), flush=True)
                return path, result, proof
            finally:
                job.close()

        with ThreadPoolExecutor(max_workers=4) as pool:
            outcomes = [future.result() for future in as_completed([pool.submit(launch, path) for path in tasks])]
        observations, reports, errors, output = [], [], [], [log]
        stable = support_sources(REPO) == sources and runner == reference(Path(__file__)) and stopped() == fence
        for path, result, proof in outcomes:
            value = read(path.with_suffix(".result.json"))
            observations.extend(value["nodes"])
            reports.extend(value["reports"])
            errors.extend(value["collectionErrors"])
            stable = stable and value["sourceIdentityStable"]
            output.append(raw(value["log"]["path"]).decode("utf-8"))
        exact = len(observations) == len(set(observations)) and set(observations) == set(discovery.nodes)
        failures = sum(report["outcome"] == "failed" and report["phase"] == "call" for report in reports)
        error_count = len(errors) + sum(report["outcome"] == "failed" and report["phase"] != "call" for report in reports)
        passed = exact and stable and not failures and not error_count and all(result == 0 for _, result, _ in outcomes)
        log_path = HISTORY / "governance-recovery-offline-20261015-1.log"
        with log_path.open("x", encoding="utf-8") as stream:
            stream.write("\n".join(output))
        receipt = {"schemaVersion": 1, "scope": "complete-current-Jenkins-support-pytest-discovery",
            "status": "passed" if passed else "failed", "testsRun": len(discovery.nodes), "exitCode": 0 if passed else 2,
            "failures": failures, "errors": error_count, "collectionErrors": errors, "testRoots": roots,
            "discoveredNodes": discovery.nodes, "reports": reports, "sourceFiles": sources, "sourceIdentityStable": stable,
            "log": str(log_path), "logSha256": sha(log_path), "runner": runner,
            "nativeTerminals": [reference(path.with_suffix(".terminal.json")) for path, _, _ in outcomes],
            "allDiscoveredTestsExecutedExactlyOnce": exact, "offlineServiceBootstrapChecks": True,
            "jenkinsExecution": False, "jenkinsRerunRequired": True, "runtimeAcceptance": False,
            "cargoAcceptance": False, "milestoneAcceptance": False, "formalAcceptance": False}
        if passed:
            _results(receipt, focused=False)
        ref = immutable(HISTORY / "governance-recovery-offline-20261015-1.json", receipt)
        immutable(OUTPUT / "regression.json", ref)
        print(json.dumps({"status": receipt["status"], "testsRun": receipt["testsRun"], "failures": failures,
                          "errors": error_count, "sourceIdentityStable": stable, "receipt": ref}), flush=True)
        return 0 if passed else 2


def prepare():
    from tools.jenkins.deployment.driver import seal_driver
    from tools.jenkins.deployment.support_regression import verify_support_regression
    from tools.jenkins.deployment.runtime_roles import inspect_runtime_role_review
    from tools.jenkins.deployment.owned_upgrade import prepare_owned_upgrade_from_release
    from tools.jenkins.deployment.recovery_keeper import _release
    from tools.jenkins.state.locks import process_lock
    with process_lock(STATE / "deployment-start.lock", reason="deployment_start_busy"):
        fence = stopped()
        regression_ref = read(OUTPUT / "regression.json")
        verify_support_regression(REPO, regression_ref)
        target = seal_driver(REPO)
        verify_support_regression(REPO, regression_ref, target)
        original = read(ORIGINAL)
        m = manager()
        previous_review = read(original["runtimeRoleReview"]["path"])
        review = inspect_runtime_role_review(m, clients=[row["identity"] for row in previous_review["clients"]],
            reviewed_target=target, profile=previous_review.get("profile"))
        review_ref = immutable(STATE / "recovery-lifecycle/history/governance-recovery-runtime-review-20261015-1.json", review)
        previous_selector = read(STATE / "deployment/driver.json")
        preparation = {"schemaVersion": 1, "state": "prepared-not-selected", "driver": target,
            "supportRegression": regression_ref, "protectedHolds": original["protectedHolds"],
            "previousSelectedDriver": previous_selector, "previousSelectorSha256": fence["deployment/driver.json"]}
        preparation_ref = immutable(HISTORY / "governance-recovery-prepared-20261015-1.json", preparation)
        successor = {**original, "actorCodeRoot": target["root"], "targetDriver": target,
            "regression": regression_ref, "preparation": preparation_ref, "runtimeRoleReview": review_ref,
            "expectedLifecycleFence": fence, "preparedAt": time.time()}
        for name, checksum in successor["operatorFiles"].items():
            need(sha(name) == checksum, "An operator changed outside the reviewed recovery input")
        successor_ref = immutable(STATE / "deployment/governance-recovery-release-20261015-1.json", successor)
        _release(m, successor_ref)
        plan = prepare_owned_upgrade_from_release(m, successor_ref, successor)
        need(plan.document["owner"] == successor["expectedPreviousOwner"], "Recovery owner changed")
        plan_ref = immutable(HISTORY / "governance-recovery-plan-20261015-1.json", plan.document)
        immutable(REPORT / "jenkins-recovery-20261015-1/prepared.json", {"release": successor_ref,
            "plan": plan_ref, "originalRequestId": successor["requestId"], "serviceLifecycleExecuted": False})
        print(json.dumps({"status": "prepared-not-started", "release": successor_ref,
                          "driverDigest": target["driverDigest"], "originalRequestId": successor["requestId"]}), flush=True)
    return 0


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--validate", action="store_true")
    group.add_argument("--prepare", action="store_true")
    group.add_argument("--worker", type=Path)
    args = parser.parse_args()
    if args.worker:
        return worker(args.worker.absolute())
    return validate() if args.validate else prepare()


if __name__ == "__main__":
    raise SystemExit(main())
