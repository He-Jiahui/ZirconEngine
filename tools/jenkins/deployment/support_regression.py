"""Verify full support evidence or an explicit full baseline plus focused delta.

A focused run remains focused. It may cover only the bounded startup/recovery
changes below; untouched files retain their immutable full-discovery baseline.
"""
from __future__ import annotations

import ast
import hashlib
import json
from pathlib import Path, PurePosixPath

from ..contracts import JenkinsError, require_digest
from .driver import _LAUNCHER, _plain, verify_driver

FULL_SCOPE = "complete-current-Jenkins-support-pytest-discovery"
FOCUSED_SCOPE = "explicit-current-Jenkins-support-pytest-modules"
SCOPED_SCOPE = "baseline-plus-focused-Jenkins-support"
_TESTS = "tools/jenkins/tests/"
_CLOSURE = {
    "drain_observations.py": ("test_drain_observations.py",),
    "bootstrap.py": ("test_agent_bootstrap_protocol.py", "test_agent_bootstrap_timeout.py", "test_agent_node_migration.py"),
    "execution_activation.py": ("test_execution_activation.py", "test_execution_recovery.py"),
    "recovery_keeper.py": ("test_recovery_keeper.py", "test_failed_upgrade_recovery.py",
                           "test_prelaunch_upgrade_recovery.py", "test_pre_spawn_recovery_cancel.py"),
    "owned_upgrade.py": ("test_owned_recovery_upgrade.py", "test_failed_upgrade_recovery.py", "test_external_native_stop_upgrade.py"),
    "lost_owner.py": ("test_failed_recovery_start.py", "test_lost_formal_owner_recovery.py"),
    "lost_recovery_runtime.py": ("test_lost_recovery_runtime.py", "test_lost_recovery_runtime_lineage.py"),
    "failed_upgrade_recovery.py": ("test_failed_upgrade_recovery.py",),
    "support_regression.py": ("test_support_regression.py", "test_execution_activation.py"),
}
_CLOSURE = {"tools/jenkins/deployment/" + name: {_TESTS + test for test in tests}
            for name, tests in _CLOSURE.items()}
_CLOSURE["tools/jenkins/tray/formal_startup.py"] = {"tools/jenkins/tray/tests/test_formal_safety.py"}
_CLOSURE["tools/dev/jenkins_desktop_tray.py"] = {_TESTS + "test_desktop_tray.py"}
_ALLOWED_TESTS = set().union(*_CLOSURE.values())
_SPECS = {".jenkins/deployment-spec.json", ".jenkins/workflow-spec.json"}


def _need(condition, message, code="execution_transition_regression"):
    if not condition:
        raise JenkinsError(code, message)


def _sha(path):
    _plain(path)
    return hashlib.sha256(path.read_bytes()).hexdigest()


def support_sources(repo):
    paths = sorted((repo / "tools/jenkins").rglob("*.py"))
    paths += sorted((repo / ".jenkins/pipeline").glob("*.groovy"))
    paths += [repo / name for name in sorted(_SPECS)]
    return {path.relative_to(repo).as_posix(): _sha(path) for path in paths if "__pycache__" not in path.parts}


def _reference(root, reference):
    _need(isinstance(reference, dict), "Support evidence requires an immutable file reference")
    name = reference.get("path")
    _need(isinstance(name, str) and bool(name), "Support evidence path is missing")
    path = Path(name)
    _need(path.is_absolute() and path == path.absolute() and ".." not in path.parts and path.parent == root,
          "Support evidence escaped its immutable deployment namespace")
    require_digest(reference.get("sha256"), "supportReceiptSha256")
    _need(_sha(path) == reference["sha256"], "Support evidence reference changed")
    value = json.loads(path.read_text(encoding="utf-8"))
    _need(isinstance(value, dict), "Support evidence must be a receipt object")
    return value


def _inventory(value):
    _need(isinstance(value, dict) and bool(value), "Complete support source inventory is missing")
    for name, checksum in value.items():
        _need(isinstance(name, str), "Support inventory path is invalid")
        relative = PurePosixPath(name)
        _need(relative.as_posix() == name and not relative.is_absolute() and "\\" not in name and ":" not in name
              and not any(part in {".", ".."} for part in relative.parts)
              and ((name.startswith("tools/jenkins/") and name.endswith(".py"))
                   or (name.startswith(".jenkins/pipeline/") and name.endswith(".groovy")) or name in _SPECS),
              "Support inventory contains an undeclared path")
        require_digest(checksum, "supportSourceSha256")
    _need(_SPECS <= set(value), "Support inventory lacks its deployment specifications")
    return value


def _passing(root, receipt, scope):
    _need(receipt.get("scope") == scope and receipt.get("status") == "passed"
          and receipt.get("sourceIdentityStable") is True
          and type(receipt.get("testsRun")) is int and receipt["testsRun"] > 0
          and type(receipt.get("exitCode")) is int and receipt["exitCode"] == 0
          and type(receipt.get("failures")) is int and receipt["failures"] == 0
          and type(receipt.get("errors")) is int and receipt["errors"] == 0 and receipt.get("collectionErrors") == [],
          "Support test receipt is not passing")
    log_name = receipt.get("log")
    _need(isinstance(log_name, str), "Support test log is missing")
    log = Path(log_name)
    _need(log.is_absolute() and str(log.absolute()) == log_name and ".." not in log.parts and log.parent == root,
          "Support test log escaped its immutable namespace")
    require_digest(receipt.get("logSha256"), "supportLogSha256")
    _need(_sha(log) == receipt["logSha256"], "Support test output changed")
    return _inventory(receipt.get("sourceFiles"))


def _results(receipt, *, focused):
    nodes = receipt.get("collectedNodes" if focused else "discoveredNodes")
    reports = receipt.get("reports")
    _need(isinstance(reports, list) and bool(reports), "Support run lacks its actual test reports")
    # Older single-process full runners stored collection count plus all reports.
    if nodes is None and not focused:
        nodes = sorted({report.get("test") for report in reports if isinstance(report, dict)
                        and isinstance(report.get("test"), str)})
    _need(isinstance(nodes, list) and len(nodes) == receipt["testsRun"] and len(set(nodes)) == len(nodes)
          and all(isinstance(node, str) and "::" in node for node in nodes),
          "Support collection does not match its recorded test count")
    grouped = {node: {} for node in nodes}
    for report in reports:
        _need(isinstance(report, dict) and report.get("test") in grouped
              and report.get("phase") in {"setup", "call", "teardown"}
              and report.get("outcome") in ({"passed"} if focused else {"passed", "skipped"}),
              "Support run contains failed, missing or out-of-scope results")
        phases = grouped[report["test"]]
        # pytest reports each unittest subtest and the completed parent as call.
        # Every report is checked above; the collected node count stays exact.
        if report["phase"] in phases:
            _need(report["phase"] == "call" and phases["call"] == report["outcome"] == "passed",
                  "A support setup/teardown or nonpassing call was recorded more than once")
        phases[report["phase"]] = report["outcome"]
    for phases in grouped.values():
        _need(phases.get("teardown") == "passed"
              and (phases.get("setup") == "passed" and phases.get("call") in ({"passed"} if focused else {"passed", "skipped"})
                   or not focused and phases.get("setup") == "skipped" and "call" not in phases),
              "Support run does not contain complete executed test results")
    return nodes


def _focused_targets(repo, focused, nodes):
    targets = focused.get("testTargets")
    _need(isinstance(targets, list) and bool(targets) and len(set(targets)) == len(targets),
          "Focused support run requires explicit module targets")
    selected = set()
    for target in targets:
        _need(isinstance(target, str), "Focused target is invalid")
        path = Path(target)
        _need(path.is_absolute() and path == path.absolute() and ".." not in path.parts and path.is_relative_to(repo),
              "Focused target escaped the repository")
        name = path.relative_to(repo).as_posix()
        _need(name in _ALLOWED_TESTS, "Focused target is outside the bounded startup/recovery closure")
        selected.add(name)
    observed = {node.split("::", 1)[0].replace("\\", "/") for node in nodes}
    _need(observed == selected, "Focused collection did not execute every selected module exactly")
    return selected


def _scoped(repo, root, receipt, *, depth=0):
    _need(depth < 32, "Support evidence reuse chain exceeds its verification bound")
    _need(receipt.get("schemaVersion") == 1 and receipt.get("scope") == SCOPED_SCOPE
          and receipt.get("status") == "passed" and receipt.get("sourceIdentityStable") is True
          and receipt.get("fullDiscoveryRun") is False and receipt.get("runtimeAcceptance") is False
          and receipt.get("cargoAcceptance") is False and receipt.get("milestoneAcceptance") is False,
          "Scoped support evidence must declare its limited acceptance")
    base = _reference(root, receipt.get("baseRegression"))
    if base.get("scope") == SCOPED_SCOPE:
        before = _scoped(repo, root, base, depth=depth + 1)
    else:
        before = _passing(root, base, FULL_SCOPE)
        _need(base.get("testRoots") == [str(repo / "tools/jenkins/tests"), str(repo / "tools/jenkins/tray/tests")],
              "Scoped reuse requires the original complete support discovery roots")
        _results(base, focused=False)
    focused = _reference(root, receipt.get("focusedRegression"))
    after = _passing(root, focused, FOCUSED_SCOPE)
    nodes = _results(focused, focused=True)
    _need(focused.get("sourceFilesAfter") == after, "Source inventory changed during focused validation")
    _need(receipt.get("sourceFilesBefore") == before and receipt.get("sourceFiles") == after,
          "Scoped inventory differs from its full baseline or actual focused run")
    changed = sorted(name for name in before.keys() | after.keys() if before.get(name) != after.get(name))
    _need(bool(changed) and receipt.get("changedFiles") == changed,
          "Scoped changedFiles must exactly include every added, changed and removed support file")
    selected = _focused_targets(repo, focused, nodes)
    _need(selected <= set(after), "Focused modules are missing from the actual tested source inventory")
    required = set()
    for name in changed:
        _need(name in _CLOSURE or name in _ALLOWED_TESTS,
              "Changes outside startup/recovery require a complete support discovery run")
        if name in _CLOSURE:
            _need(_CLOSURE[name] <= set(after), "A changed module lacks part of its required test closure")
            required.update(_CLOSURE[name])
        elif name in after:
            required.add(name)
        else:
            for closure in _CLOSURE.values():
                if name in closure:
                    required.update(closure)
    required &= set(after)
    _need(bool(required) and required <= selected, "Focused run omits the test closure for a changed module")
    _need(receipt.get("testsRun") == focused["testsRun"] and receipt.get("testTargets") == focused["testTargets"]
          and receipt.get("log") == focused["log"] and receipt.get("logSha256") == focused["logSha256"]
          and receipt.get("exitCode") == 0 and receipt.get("failures") == 0 and receipt.get("errors") == 0
          and receipt.get("collectionErrors") == [], "Scoped summary differs from its actual focused validation")
    return after


def _candidate(candidate, sources):
    root = Path(candidate.get("root", ""))
    verified = verify_driver(root, expected_digest=candidate.get("driverDigest"))
    _need(all(candidate.get(key) == value for key, value in verified.items()),
          "Candidate differs from its verified immutable driver", "execution_transition_driver")
    manifest = json.loads(Path(verified["manifest"]).read_text(encoding="utf-8"))
    expected = {name: checksum for name, checksum in sources.items() if "/tests/" not in name and name not in _SPECS}
    driver_name = "tools/jenkins/deployment/driver.py"
    launcher = _LAUNCHER
    if driver_name in expected:
        raw = (root / driver_name).read_bytes()
        assignments = [node.value for node in ast.parse(raw).body if isinstance(node, ast.Assign)
                       and any(isinstance(target, ast.Name) and target.id == "_LAUNCHER" for target in node.targets)]
        _need(len(assignments) == 1 and isinstance(assignments[0], ast.Constant)
              and isinstance(assignments[0].value, str), "Tested driver lacks one literal launcher", "execution_transition_driver")
        launcher = assignments[0].value
    expected["launcher.py"] = hashlib.sha256(launcher.encode("utf-8")).hexdigest()
    expected["tools/__init__.py"] = hashlib.sha256(b'"""This package belongs to one verified Jenkins snapshot."""\n').hexdigest()
    observed = {".jenkins/" + name if name.startswith("pipeline/") else name: entry.get("sha256")
                for name, entry in manifest["files"].items()}
    _need(observed == expected, "Candidate does not cover the exact tested support inventory", "execution_transition_driver")


def verify_support_regression(repo, reference, candidate=None, *, current=True, required_operators=(), state_root=None):
    """Read-only verification; historical callers bind tested sealed bytes only."""
    repo = Path(repo).absolute()
    root = (Path(state_root).absolute() if state_root is not None else repo / ".jenkins/state") / "migration-baseline"
    receipt = _reference(root, reference)
    if receipt.get("scope") == SCOPED_SCOPE:
        sources = _scoped(repo, root, receipt)
    else:
        sources = _passing(root, receipt, FULL_SCOPE)
    required = {name for name in required_operators if name.startswith("tools/jenkins/")}
    _need(required <= set(sources), "Tested support inventory lacks its required lifecycle operators")
    if current:
        _need(sources == support_sources(repo), "Tested support source or specification changed",
              "execution_transition_source_changed")
    if candidate is not None:
        _candidate(candidate, sources)
    return receipt
