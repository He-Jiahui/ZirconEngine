"""Explicit transition from a proven terminal controller-only generation."""
from __future__ import annotations

from dataclasses import dataclass
import hashlib
import json
from pathlib import Path
import sqlite3
import time

from ..contracts import JenkinsError, digest
from ..state.locks import process_lock
from .driver import _LAUNCHER, _plain, verify_driver
from .recovery import has_terminal_lifecycle_proofs
from .startup_observations import observe_departed_identity


def _sha(path: Path) -> str:
    _plain(path)
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _json(path: Path) -> dict:
    _plain(path)
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise JenkinsError("execution_transition_invalid", "Transition evidence must be an object")
    return value


def _write_json_atomic(path: Path, value: dict) -> None:
    pending = path.with_name(path.name + ".pending-" + str(time.time_ns()))
    pending.write_text(json.dumps(value, sort_keys=True, indent=2) + "\n", encoding="utf-8")
    pending.replace(path)


def support_sources(repo: Path) -> dict[str, str]:
    paths = sorted((repo / "tools/jenkins").rglob("*.py"))
    paths += sorted((repo / ".jenkins/pipeline").glob("*.groovy"))
    paths += [repo / ".jenkins/deployment-spec.json", repo / ".jenkins/workflow-spec.json"]
    return {path.relative_to(repo).as_posix(): _sha(path) for path in paths if "__pycache__" not in path.parts}


def protected_holds(state_root: Path) -> list[dict]:
    database = state_root / "coordination.sqlite3"
    if not database.is_file():
        raise JenkinsError("execution_transition_holds", "Durable coordination database is missing")
    _plain(database)
    try:
        connection = sqlite3.connect(database.as_uri() + "?mode=ro", uri=True)
    except sqlite3.Error as exc:
        raise JenkinsError("execution_transition_holds", "Durable coordination database is unreadable") from exc
    try:
        try:
            rows = connection.execute("SELECT domain,key,version,payload FROM records WHERE domain IN "
                                      "('resource_reservation','pool_writer_hold') ORDER BY domain,key").fetchall()
        except sqlite3.Error as exc:
            raise JenkinsError("execution_transition_holds", "Durable coordination records are unreadable") from exc
        return [{"domain": domain, "key": key, "version": version, "payload": json.loads(payload)}
                for domain, key, version, payload in rows if json.loads(payload).get("status") == "active"]
    finally:
        connection.close()


def _driver(repo: Path, value: dict) -> dict:
    driver_digest = value.get("driverDigest", "")
    root = repo / ".jenkins/runtime/drivers" / driver_digest
    if (not isinstance(driver_digest, str) or len(driver_digest) != 64
            or any(c not in "0123456789abcdef" for c in driver_digest)
            or Path(value.get("root", "")).absolute() != root.absolute()):
        raise JenkinsError("execution_transition_driver", "Driver must be in this deployment's immutable runtime")
    return verify_driver(root, expected_digest=driver_digest)


def verify_regression(repo: Path, reference: dict, candidate: dict) -> dict:
    path = Path(reference.get("path", "")).absolute()
    expected_root = repo / ".jenkins/state/migration-baseline"
    if path.parent != expected_root or _sha(path) != reference.get("sha256"):
        raise JenkinsError("execution_transition_regression", "Complete support regression reference changed")
    receipt = _json(path)
    if (receipt.get("scope") != "complete-current-Jenkins-support-pytest-discovery"
            or receipt.get("status") != "passed" or receipt.get("sourceIdentityStable") is not True
            or type(receipt.get("testsRun")) is not int or receipt["testsRun"] <= 0
            or receipt.get("exitCode") != 0 or receipt.get("failures") != 0 or receipt.get("errors") != 0
            or receipt.get("collectionErrors") != []):
        raise JenkinsError("execution_transition_regression", "Complete current support regression did not pass")
    log = Path(receipt.get("log", "")).absolute()
    if log.parent != expected_root or _sha(log) != receipt.get("logSha256"):
        raise JenkinsError("execution_transition_regression", "Support regression output changed")
    if receipt.get("sourceFiles") != support_sources(repo):
        raise JenkinsError("execution_transition_source_changed", "Tested support source or specification changed")
    manifest = _json(Path(candidate["manifest"]))
    files = manifest.get("files")
    if manifest.get("driverDigest") != candidate.get("driverDigest") or not isinstance(files, dict):
        raise JenkinsError("execution_transition_driver", "Candidate manifest identity is not bound to its driver")
    expected = {}
    for name, source_hash in support_sources(repo).items():
        if name.startswith("tools/jenkins/") and "/tests/" in name:
            continue
        if name in {".jenkins/deployment-spec.json", ".jenkins/workflow-spec.json"}:
            continue
        expected[name] = source_hash
    expected["launcher.py"] = hashlib.sha256(_LAUNCHER.encode("utf-8")).hexdigest()
    expected["tools/__init__.py"] = hashlib.sha256(
        b'"""This package belongs to one verified Jenkins snapshot."""\n').hexdigest()
    observed = {}
    for name, entry in manifest["files"].items():
        source = ".jenkins/" + name if name.startswith("pipeline/") else name
        if source not in expected:
            raise JenkinsError("execution_transition_driver", "Candidate manifest contains undeclared runtime files")
        observed[source] = entry.get("sha256")
    if set(observed) != set(expected) or any(observed[k] != v for k, v in expected.items()):
        raise JenkinsError("execution_transition_driver", "Candidate does not cover the tested implementation")
    return receipt


@dataclass(frozen=True)
class ActivationPlan:
    evidence_json: str
    transition_digest: str

    @property
    def previous_operation(self) -> str:
        return json.loads(self.evidence_json)["previous"]["operationId"]

    def validate(self, manager, *, candidate_selected: bool) -> dict:
        evidence = json.loads(self.evidence_json)
        if digest(evidence) != self.transition_digest:
            raise JenkinsError("execution_transition_plan", "Transition identity changed")
        repo = manager.paths.repo.absolute()
        if (evidence["repository"] != str(repo) or evidence["home"] != str(manager.paths.home.absolute())
                or evidence["specPath"] != str(manager.spec.path.absolute())
                or evidence["specSha256"] != _sha(manager.spec.path)):
            raise JenkinsError("execution_transition_spec", "Transition repository, Home or specification changed")
        current = manager._read() or {}
        host = _json(manager.paths.state / "deployment/host.json")
        if current != evidence["previous"] or host != evidence["host"]:
            raise JenkinsError("execution_transition_changed", "Durable lifecycle evidence changed")
        if (current.get("state") not in {"stopped", "failed", "interrupted"}
                or current.get("controlPlaneOnly") is not True):
            raise JenkinsError("execution_transition_live_generation", "Previous generation is not terminal controller-only")
        observed = observe_departed_identity(current)
        if not has_terminal_lifecycle_proofs(current, host, str(manager.paths.home.absolute()), observed):
            raise JenkinsError("execution_transition_terminal_missing", "Owned component and supervisor departure proof required")
        if protected_holds(manager.paths.state) != evidence["protectedHolds"]:
            raise JenkinsError("execution_transition_holds", "Protected historical reservations or writers changed")
        candidate = _driver(repo, evidence["candidateDriver"])
        _driver(repo, evidence["oldDriver"])
        if evidence["oldDriver"].get("runtimeOperationId") != current["operationId"]:
            raise JenkinsError("execution_transition_driver", "Previous selected driver belongs to another runtime")
        verify_regression(repo, evidence["regression"], candidate)
        selector = _json(manager.paths.state / "deployment/driver.json")
        if candidate_selected:
            if (selector.get("driverDigest") != candidate["driverDigest"]
                    or selector.get("executionActivationDigest") != self.transition_digest
                    or selector.get("state") != "selected-for-runtime-start"
                    or selector.get("executionActivationReceipt") != str(
                        manager.paths.state / "deployment/history" /
                        ("execution-activation-" + self.transition_digest + ".json"))
                    or selector.get("supportRegression") != evidence["regression"]
                    or selector.get("runtimeAcceptance") is not False
                    or selector.get("runtimeOperationId") is not None
                    or any(selector.get(key) != value for key, value in candidate.items())):
                raise JenkinsError("execution_transition_selection", "Selected candidate is not bound to this transition")
        elif selector != evidence["oldDriver"]:
            raise JenkinsError("execution_transition_selection", "Previous selected driver changed")
        return evidence


class ExecutionActivation:
    def __init__(self, manager):
        self.manager = manager

    def prepare(self, *, candidate_driver: dict, regression_ref: dict) -> ActivationPlan:
        manager = self.manager
        previous = manager._read() or {}
        if previous.get("state") not in {"stopped", "failed", "interrupted"}:
            raise JenkinsError("execution_transition_live_generation", "A live lifecycle generation cannot change execution mode")
        host = _json(manager.paths.state / "deployment/host.json")
        observed = observe_departed_identity(previous)
        if not has_terminal_lifecycle_proofs(previous, host, str(manager.paths.home.absolute()), observed):
            raise JenkinsError("execution_transition_terminal_missing", "Complete owned lifecycle termination is required")
        candidate = _driver(manager.paths.repo, candidate_driver)
        old = _json(manager.paths.state / "deployment/driver.json")
        evidence = {"schemaVersion": 1, "repository": str(manager.paths.repo.absolute()),
                    "home": str(manager.paths.home.absolute()), "specPath": str(manager.spec.path.absolute()),
                    "specSha256": _sha(manager.spec.path), "previous": previous, "host": host,
                    "hostObservation": observed, "oldDriver": old, "candidateDriver": candidate,
                    "regression": regression_ref, "protectedHolds": protected_holds(manager.paths.state),
                    "nativeExecutionHoldsReleaseAllowed": False, "runtimeAcceptance": False}
        plan = ActivationPlan(json.dumps(evidence, sort_keys=True), digest(evidence))
        plan.validate(manager, candidate_selected=False)
        return plan

    def activate(self, plan: ActivationPlan) -> dict:
        manager = self.manager
        lock = manager.paths.state / "deployment-start.lock"
        with process_lock(lock, reason="deployment_operation_busy"):
            evidence = plan.validate(manager, candidate_selected=False)
            history = manager.paths.state / "deployment/history"
            history.mkdir(parents=True, exist_ok=True)
            archive = history / ("execution-activation-" + plan.transition_digest + ".json")
            if archive.exists() and _json(archive) != evidence:
                raise JenkinsError("execution_transition_history", "Transition history changed")
            if not archive.exists():
                archive.write_text(json.dumps(evidence, sort_keys=True, indent=2) + "\n", encoding="utf-8")
            archive_ref = {"path": str(archive), "sha256": _sha(archive)}
            selected = {**evidence["candidateDriver"], "state": "selected-for-runtime-start",
                        "executionActivationDigest": plan.transition_digest,
                        "executionActivationReceipt": str(archive), "supportRegression": evidence["regression"],
                        "runtimeAcceptance": False}
            path = manager.paths.state / "deployment/driver.json"
            journal = manager.paths.state / "deployment" / "execution-activation.json"
            pending_journal = journal.with_name("execution-activation.pending-" + str(time.time_ns()) + ".json")
            pending_journal.write_text(json.dumps({"status": "pending", "transitionDigest": plan.transition_digest,
                                                   "previous": evidence["previous"], "candidate": selected,
                                                   "archiveRef": archive_ref, "protectedHolds": evidence["protectedHolds"]},
                                                  sort_keys=True, indent=2) + "\n", encoding="utf-8")
            pending_journal.replace(journal)
            pending = path.with_name("driver.activation-" + str(time.time_ns()) + ".json")
            try:
                pending.write_text(json.dumps(selected, indent=2) + "\n", encoding="utf-8")
                pending.replace(path)
            except Exception as exc:
                _write_json_atomic(journal, {**_json(journal), "status": "unknown",
                                             "error": type(exc).__name__, "phase": "selector-swap"})
                raise
            try:
                result = manager._start_locked(lock, recovery_driver=evidence["oldDriver"], execution_activation=plan)
            except Exception as exc:
                _write_json_atomic(journal, {**_json(journal), "status": "unknown", "error": type(exc).__name__})
                raise
            if not isinstance(result, dict) or result.get("state") != "running":
                _write_json_atomic(journal, {**_json(journal), "status": "failed", "result": result})
            else:
                if not isinstance(result.get("operationId"), str) or not result["operationId"]:
                    _write_json_atomic(journal, {**_json(journal), "status": "unknown", "error": "missing-operation-id"})
                    raise JenkinsError("execution_transition_unresolved", "Successful activation lacks operation identity")
                _write_json_atomic(journal, {**_json(journal), "status": "succeeded",
                                             "operationId": result["operationId"]})
            return result


def assert_activation_start_allowed(manager) -> None:
    """Guard normal starts until an activation outcome is durably resolved."""
    journal = manager.paths.state / "deployment/execution-activation.json"
    if not journal.is_file():
        return
    try:
        record = _json(journal)
    except Exception as exc:
        raise JenkinsError("execution_transition_unresolved", "Activation journal is corrupt") from exc
    if record.get("status") != "succeeded":
        if record.get("status") == "rolled_back":
            from .execution_recovery import assert_recovery_start_allowed
            return assert_recovery_start_allowed(manager)
        raise JenkinsError("execution_transition_unresolved", "An execution activation requires explicit recovery")
    if not (isinstance(record.get("transitionDigest"), str) and len(record["transitionDigest"]) == 64
            and isinstance(record.get("operationId"), str) and record["operationId"]
            and isinstance(record.get("candidate"), dict)
            and record["candidate"].get("runtimeAcceptance") is False):
        raise JenkinsError("execution_transition_unresolved", "Activation success journal is incomplete")
