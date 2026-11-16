"""Prove retired CLIs stop before legacy imports, state, network or native work.

Only the shared stateless notice is imported. Historical modules are compiled and
executed in memory behind deny-by-default imports/I/O. Real subprocess probes are
limited to entries whose early guard (or entire literal PS stub) was just proved.
No Jenkins executor, service, native Job, Cargo or engine test is run.
"""
from __future__ import annotations

import ast
import builtins
from contextlib import ExitStack, contextmanager, redirect_stdout
import importlib
import io
import json
import os
from pathlib import Path
import re
import shutil
import socket
import sqlite3
import subprocess
import sys
from types import SimpleNamespace
import unittest
from unittest.mock import patch
import urllib.request

from tools.jenkins.retirement import MESSAGE

ROOT = Path(__file__).resolve().parents[2]
PYTHON_ENTRIES = tuple("""
tools/dev/jenkins_boot_recovery.py
tools/dev/jenkins_branch_queue.py
tools/dev/jenkins_cleanup_runtime.py
tools/dev/jenkins_commit.py
tools/dev/jenkins_delivery.py
tools/dev/jenkins_desktop_tray.py
tools/dev/jenkins_editor.py
tools/dev/jenkins_governance.py
tools/dev/jenkins_queue_reconcile.py
tools/dev/jenkins_recovery_bootstrap.py
tools/dev/jenkins_recovery_keeper.py
tools/dev/jenkins_service.py
tools/dev/jenkins_service_host.py
tools/dev/jenkins_transition_monitor.py
tools/dev/jenkins_validate.py
tools/dev/prepare_jenkins_runtime_contention_recovery.py
tools/dev/resume_jenkins_flow.py
tools/jenkins/__main__.py
tools/jenkins/cli.py
tools/jenkins/dashboard.py
tools/jenkins/editor_product.py
tools/jenkins/frontend.py
tools/jenkins/jenkins_coordinator.py
tools/jenkins/zircon-jenkins-start.py
tools/jenkins/deployment/__main__.py
tools/jenkins/deployment/host.py
tools/jenkins/deployment/independent_host.py
tools/jenkins/deployment/independent_service.py
tools/jenkins/deployment/unknown_stop_recovery.py
tools/jenkins/gap/__main__.py
tools/jenkins/gap/cli.py
tools/jenkins/governance/bootstrap.py
tools/jenkins/governance/probe_child.py
tools/jenkins/governance/scan_child.py
tools/jenkins/launcher/__main__.py
tools/jenkins/pilot/__main__.py
tools/jenkins/pilot/closure.py
tools/jenkins/pilot/fault_probe.py
tools/jenkins/pilot/governance.py
tools/jenkins/pilot/governance_recovery.py
tools/jenkins/pilot/runner.py
tools/jenkins/pilot/runtime_host.py
tools/jenkins/pilot/static_validation.py
tools/jenkins/processes/execution_host.py
tools/jenkins/tray/__main__.py
tools/jenkins/tray/launch.py
tools/jenkins-management/authorization_owner.py
tools/jenkins-management/branch_cleanup.py
tools/jenkins-management/branch_integration.py
tools/jenkins-management/cargo_cache_compaction.py
tools/jenkins-management/checkout_commit.py
tools/jenkins-management/delivery.py
tools/jenkins-management/entry.py
tools/jenkins-management/fixture_owner_recipe.py
tools/jenkins-management/governance_fixture_closeout.py
tools/jenkins-management/incremental_cache_cleanup.py
tools/jenkins-management/install_guard.py
tools/jenkins-management/managed_node.py
tools/jenkins-management/records_followup.py
tools/jenkins-management/records_queue.py
tools/jenkins-management/remote_branch_cleanup.py
tools/jenkins-management/resolver_entry_observation.py
tools/jenkins-management/resolver_failure_record.py
tools/jenkins-management/resolver_regression_recipe.py
tools/jenkins-management/reuse_runtime.py
tools/jenkins-management/run_checks.py
tools/jenkins-management/run_commit_checks.py
tools/jenkins-management/runner.py
tools/jenkins-management/scheduled_control.py
tools/jenkins-management/test_job.py
tools/jenkins-management/worktree_consolidation.py
""".split())
POWERSHELL_ENTRIES = tuple("""
tools/dev/Install-JenkinsService.ps1
tools/dev/install-jenkins-management.ps1
tools/dev/install-jenkins-scheduled-cleanup.ps1
tools/dev/jenkins-validate.ps1
tools/dev/start-jenkins-coordinator.ps1
tools/dev/start-jenkins-on-logon.ps1
tools/dev/start-jenkins-sandbox.ps1
tools/dev/submit-jenkins-management.ps1
tools/jenkins/install-coordinator-worker.ps1
tools/jenkins/install-jenkins-tray-startup.ps1
tools/jenkins-management/records_cleanup.ps1
tools/jenkins-management/records_followup_cleanup.ps1
""".split())
LAZY_EXPORTS = {
    "deployment": {
        ".spec": ("DeploymentSpec", "load_spec", "JenkinsError"),
        ".paths": ("DeploymentPaths", "resolve_paths"),
    },
    "gap": {".state": ("GapState",), ".extractor": ("extract_issues", "scan_directory"), ".cli": ("main",)},
    "processes": {
        ".identity": ("ProcessIdentity", "current_identity", "identity_matches", "process_alive"),
        ".job": ("NativeJob", "ManagedProcess", "TerminationProof"),
        ".registry": ("NativeJobRecord", "ProcessRegistry"),
        ".execution_host": ("ExecutionHost", "HostIdentity", "request_cancel"),
    },
}
MAIN_TEST = ast.dump(ast.parse("__name__ == '__main__'", mode="eval").body)
NOTICE = {
    "status": "retired", "reasonCode": "jenkins_workflow_retired", "message": MESSAGE,
    "executionOwner": "Agent", "commandExecuted": False, "hostServicesChanged": False,
    "historicalDataChanged": False, "formalAcceptance": False, "m8Accepted": False,
}


def first_executable(tree):
    for node in tree.body:
        if isinstance(node, ast.Expr) and isinstance(node.value, ast.Constant) and isinstance(node.value.value, str):
            continue
        if isinstance(node, ast.ImportFrom) and node.module == "__future__":
            continue
        return node
    raise AssertionError("An executable entry has no early retirement guard")


@contextmanager
def no_legacy_io():
    original_import = builtins.__import__

    def guarded_import(name, globals=None, locals=None, fromlist=(), level=0):
        if level or name not in {"__future__", "sys", "pathlib", "importlib", "tools.jenkins.retirement"}:
            raise AssertionError("Legacy import reached before terminal exit: " + name)
        return original_import(name, globals, locals, fromlist, level)

    def denied(*_args, **_kwargs):
        raise AssertionError("Retired entry attempted state, file, network or process I/O")

    with ExitStack() as stack:
        stack.enter_context(patch("builtins.__import__", side_effect=guarded_import))
        stack.enter_context(patch("builtins.open", side_effect=denied))
        stack.enter_context(patch.object(Path, "open", side_effect=denied))
        stack.enter_context(patch.object(sqlite3, "connect", side_effect=denied))
        stack.enter_context(patch.object(subprocess, "Popen", side_effect=denied))
        stack.enter_context(patch.object(socket, "create_connection", side_effect=denied))
        stack.enter_context(patch.object(socket.socket, "connect", side_effect=denied))
        stack.enter_context(patch.object(urllib.request, "urlopen", side_effect=denied))
        stack.enter_context(patch.object(sys, "path", list(sys.path)))
        stack.enter_context(patch.object(sys, "dont_write_bytecode", True))
        yield


class WorkflowRetirementEntryTests(unittest.TestCase):
    def prove_python_entry(self, name):
        source = (ROOT / name).read_text(encoding="utf-8-sig")
        tree = ast.parse(source, filename=name)
        guard = first_executable(tree)
        self.assertIsInstance(guard, ast.If, name)
        self.assertEqual(MAIN_TEST, ast.dump(guard.test), name)
        depth = len(Path(name).parts) - 1
        indices = [node.slice.value for node in ast.walk(guard)
                   if isinstance(node, ast.Subscript) and isinstance(node.value, ast.Attribute)
                   and node.value.attr == "parents" and isinstance(node.slice, ast.Constant)]
        self.assertEqual([depth], indices, name)
        code = compile(tree, str(ROOT / name), "exec")
        output = io.StringIO()
        with no_legacy_io(), redirect_stdout(output), patch.object(sys, "argv", [name, "start", "--spec", "must-not-read"]):
            with self.assertRaises(SystemExit, msg=name) as raised:
                exec(code, {"__name__": "__main__", "__file__": str(ROOT / name)})
        self.assertEqual(2, raised.exception.code, name)
        self.assertEqual(NOTICE, json.loads(output.getvalue()), name)
        return guard

    def prove_package_init(self, name):
        path = ROOT / "tools/jenkins" / name / "__init__.py"
        code = compile(path.read_text(encoding="utf-8-sig"), str(path), "exec")
        namespace = {"__name__": "tools.jenkins." + name, "__file__": str(path)}
        with no_legacy_io():
            exec(code, namespace)
        return namespace

    def prove_ps_stub(self, name):
        text = (ROOT / name).read_text(encoding="utf-8-sig")
        lines = [line for line in text.splitlines() if line.strip() and not line.lstrip().startswith("#")]
        self.assertEqual(2, len(lines), name)
        match = re.fullmatch(r"\[Console\]::Out\.WriteLine\('([^']*)'\)", lines[0])
        self.assertIsNotNone(match, name)
        self.assertEqual(NOTICE, json.loads(match[1]), name)
        self.assertEqual("exit 2", lines[1], name)

    def test_manifest_covers_every_current_public_python_cli(self):
        found = {path.relative_to(ROOT).as_posix() for path in (ROOT / "tools/dev").glob("*.py")
                 if "jenkins" in path.name.casefold()}
        for directory in (ROOT / "tools/jenkins", ROOT / "tools/jenkins-management"):
            for path in directory.rglob("*.py"):
                if {"tests", "fixtures", "fixture"} & set(path.relative_to(directory).parts) or path.name == "tests.py":
                    continue
                tree = ast.parse(path.read_text(encoding="utf-8-sig"), filename=str(path))
                if path.name == "__main__.py" or any(
                    isinstance(node, ast.FunctionDef) and node.name == "main"
                    or isinstance(node, ast.If) and ast.dump(node.test) == MAIN_TEST for node in tree.body
                ):
                    found.add(path.relative_to(ROOT).as_posix())
        self.assertEqual(set(PYTHON_ENTRIES), found)
        self.assertEqual(len(PYTHON_ENTRIES), len(set(PYTHON_ENTRIES)))

    def test_all_python_entries_exit_before_legacy_imports_or_io(self):
        for name in PYTHON_ENTRIES:
            with self.subTest(entry=name):
                self.prove_python_entry(name)

    def test_guards_do_not_retire_historical_library_imports(self):
        for name in PYTHON_ENTRIES:
            with self.subTest(entry=name):
                guard = first_executable(ast.parse((ROOT / name).read_text(encoding="utf-8-sig")))
                code = compile(ast.Module(body=[guard], type_ignores=[]), name, "exec")
                with no_legacy_io(), patch("builtins.__import__", side_effect=AssertionError("Library guard ran")):
                    exec(code, {"__name__": "historical_library", "__file__": str(ROOT / name)})

    def test_package_initializers_are_lazy_and_keep_their_existing_exports(self):
        for name, groups in LAZY_EXPORTS.items():
            with self.subTest(package=name):
                namespace = self.prove_package_init(name)
                for module_name, names in groups.items():
                    for export in names:
                        sentinel = object()
                        with patch.object(importlib, "import_module", return_value=SimpleNamespace(**{export: sentinel})) as load:
                            self.assertIs(sentinel, namespace["__getattr__"](export))
                        load.assert_called_once_with(module_name, "tools.jenkins." + name)
                        self.assertIs(sentinel, namespace[export])
                with self.assertRaises(AttributeError):
                    namespace["__getattr__"]("not_a_public_export")

    def test_deployment_handle_preserves_injected_library_export_boundaries(self):
        namespace = self.prove_package_init("deployment")
        calls = []
        spec = SimpleNamespace(repository_root="fixture-repository")
        paths = SimpleNamespace(home="fixture-home", build_root="fixture-build-root")
        namespace.update(load_spec=lambda value: calls.append(("spec", value)) or spec,
                         resolve_paths=lambda value, root: calls.append(("paths", value, root)) or paths,
                         JenkinsError=ValueError)
        with no_legacy_io():
            result = namespace["handle"]("validate", {}, None, Path("fixture-repository"))
        self.assertEqual({"status": "ready", "repositoryRoot": "fixture-repository",
                          "home": "fixture-home", "buildRoot": "fixture-build-root"}, result)
        self.assertEqual([("spec", Path("fixture-repository/.jenkins/deployment-spec.json")),
                          ("paths", spec, None)], calls)

    def test_all_powershell_entries_are_literal_parameterless_terminal_stubs(self):
        found = {path.relative_to(ROOT).as_posix() for path in (ROOT / "tools/dev").glob("*.ps1")
                 if "jenkins" in path.name.casefold()}
        for directory in (ROOT / "tools/jenkins", ROOT / "tools/jenkins-management"):
            found.update(path.relative_to(ROOT).as_posix() for path in directory.glob("*.ps1"))
        self.assertEqual(set(POWERSHELL_ENTRIES), found)
        for name in POWERSHELL_ENTRIES:
            with self.subTest(entry=name):
                self.prove_ps_stub(name)

    def test_proved_direct_file_entries_ignore_stale_inputs_from_another_cwd(self):
        representatives = (
            "tools/dev/jenkins_service_host.py", "tools/dev/prepare_jenkins_runtime_contention_recovery.py",
            "tools/dev/resume_jenkins_flow.py", "tools/jenkins/__main__.py",
            "tools/jenkins/tray/launch.py", "tools/jenkins/pilot/__main__.py",
            "tools/jenkins-management/runner.py", "tools/jenkins-management/records_queue.py",
        )
        for name in representatives:
            with self.subTest(entry=name):
                self.prove_python_entry(name)  # A missing guard must fail before Popen.
                result = subprocess.run([sys.executable, "-B", str(ROOT / name), "start", "--spec", "must-not-read"],
                    cwd=ROOT.parent, env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1",
                    ZIRCON_JENKINS_IDENTITY_FILE="must-not-read", JENKINS_URL="https://must-not-contact.invalid"),
                    capture_output=True, text=True, encoding="utf-8", timeout=15, check=False)
                self.assertEqual(2, result.returncode, result.stderr)
                self.assertEqual(NOTICE, json.loads(result.stdout), name)

    def test_proved_package_module_entries_do_not_preload_legacy_owners(self):
        modules = ("tools.jenkins", "tools.jenkins.deployment", "tools.jenkins.gap", "tools.jenkins.launcher",
                   "tools.jenkins.pilot", "tools.jenkins.tray", "tools.jenkins.processes.execution_host")
        for module in modules:
            with self.subTest(module=module):
                name = module.replace(".", "/")
                entry = name + ".py" if (ROOT / (name + ".py")).is_file() else name + "/__main__.py"
                self.prove_python_entry(entry)
                for package in LAZY_EXPORTS:
                    self.prove_package_init(package)
                result = subprocess.run([sys.executable, "-B", "-m", module, "start", "--spec", "must-not-read"],
                    cwd=ROOT, env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"),
                    capture_output=True, text=True, encoding="utf-8", timeout=15, check=False)
                self.assertEqual(2, result.returncode, result.stderr)
                self.assertEqual(NOTICE, json.loads(result.stdout), module)

    @unittest.skipUnless(shutil.which("pwsh") or shutil.which("powershell"), "PowerShell is not installed")
    def test_proved_powershell_stubs_do_not_prompt_for_original_mandatory_arguments(self):
        shell = shutil.which("pwsh") or shutil.which("powershell")
        for name in ("tools/dev/Install-JenkinsService.ps1", "tools/jenkins-management/records_cleanup.ps1"):
            for arguments in ([], ["-Action", "Install", "-InputFile", "must-not-read"]):
                with self.subTest(entry=name, arguments=arguments):
                    self.prove_ps_stub(name)  # Do not launch an unproved old installer.
                    result = subprocess.run([shell, "-NoProfile", "-NonInteractive", "-File", str(ROOT / name), *arguments],
                        cwd=ROOT.parent, capture_output=True, text=True, encoding="utf-8", timeout=15, check=False)
                    self.assertEqual(2, result.returncode, result.stderr)
                    self.assertEqual(NOTICE, json.loads(result.stdout), name)


if __name__ == "__main__":
    unittest.main()
