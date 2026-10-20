"""Public-client bootstrap contracts with memory files and real import machinery.

Existing boot and driver verification functions execute over memory-backed
paths. No source fixture, child process, service request or disk write is used.
"""
from __future__ import annotations

import ast
from contextlib import contextmanager, ExitStack
import hashlib
import importlib
import importlib.abc
import importlib.machinery
import importlib.util
import json
import os
import re
from pathlib import Path, PurePosixPath
import sys
from types import ModuleType, SimpleNamespace
import unittest
from unittest.mock import patch


HERE = Path(__file__).absolute().parents[1]
ENTRY_SOURCE = HERE / "entry.py"
entry_spec = importlib.util.spec_from_file_location("entry_bootstrap_subject", ENTRY_SOURCE)
entry = importlib.util.module_from_spec(entry_spec)
entry_spec.loader.exec_module(entry)

# Read production definitions before replacing file observations. These are
# the unchanged contracts used by the pinned runner and selected driver.
BOOT_SOURCE = HERE / "runner.py"
DRIVER_SOURCE = HERE.parent / "jenkins/deployment/driver.py"
BOOT_TREE = ast.parse(BOOT_SOURCE.read_bytes().decode("utf-8"), filename=str(BOOT_SOURCE))
DRIVER_TREE = ast.parse(DRIVER_SOURCE.read_bytes().decode("utf-8"), filename=str(DRIVER_SOURCE))
FRONTEND_SOURCE = HERE.parent / "jenkins/frontend.py"
FRONTEND_TREE = ast.parse(FRONTEND_SOURCE.read_bytes().decode("utf-8"), filename=str(FRONTEND_SOURCE))


def production_functions(tree, names, namespace, filename):
    selected = [node for node in tree.body if isinstance(node, ast.FunctionDef) and node.name in names]
    if len(selected) != len(names):
        raise AssertionError("Production bootstrap contract changed")
    future = ast.ImportFrom(module="__future__", names=[ast.alias(name="annotations")], level=0)
    module = ast.fix_missing_locations(ast.Module(body=[future, *selected], type_ignores=[]))
    exec(compile(module, str(filename), "exec"), namespace)


class MemoryLoader(importlib.abc.Loader):
    def __init__(self, case, name):
        self.case, self.name = case, name

    def create_module(self, spec):
        return None

    def exec_module(self, module):
        case = self.case
        if self.name in {"verified_ordinary_management_runner", "verified_management_client_selector"}:
            module.__dict__.update(Path=Path, json=json, os=os, sys=sys, re=re, sha=entry.sha,
                                   encoded=entry.encoded, plain=lambda path: Path(path).absolute())
            production_functions(BOOT_TREE, {"boot", "client_path"}, module.__dict__, BOOT_SOURCE)
        elif self.name == "tools.jenkins.deployment.driver":
            case.driver_imports.append(module.__file__)
            module.__dict__.update(Path=Path, PurePosixPath=PurePosixPath, hashlib=hashlib, json=json,
                os=os, JenkinsError=entry.EntryError, _plain=lambda path: None,
                canonical_json=entry.encoded, digest=lambda value: entry.sha(entry.encoded(value)))
            production_functions(DRIVER_TREE,
                {"_summary", "verify_driver", "driver_environment", "active_driver_binding"},
                module.__dict__, DRIVER_SOURCE)
            if case.wrong_environment:
                original = module.driver_environment
                def wrong_environment(*args):
                    return {**original(*args), "ZIRCON_DRIVER_DIGEST": "0" * 64}
                module.driver_environment = wrong_environment
        elif self.name == "tools.jenkins.deployment.spec":
            module.load_spec = lambda path: {"repositoryRoot": str(case.repo)}
        elif self.name == "tools.jenkins.deployment.paths":
            module.resolve_paths = lambda spec: SimpleNamespace(repository_root=case.repo)
        elif self.name == "tools.jenkins.deployment.manager":
            module.DeploymentManager = lambda spec, paths: case.manager
        elif self.name == "tools.jenkins.deployment.drain_observations":
            module.owner_observation = lambda manager: {"operationId": case.runtime}
            module.quiet_observation = lambda manager: False


class MemoryFinder(importlib.abc.MetaPathFinder):
    PACKAGES = {"tools", "tools.jenkins", "tools.jenkins.deployment"}
    MODULES = {"tools.jenkins.deployment.driver", "tools.jenkins.deployment.spec",
               "tools.jenkins.deployment.paths", "tools.jenkins.deployment.manager",
               "tools.jenkins.deployment.drain_observations"}

    def __init__(self, case):
        self.case = case

    def find_spec(self, fullname, path=None, target=None):
        if fullname not in self.PACKAGES | self.MODULES:
            return None
        if fullname == "tools":
            root = self.case.driver if str(self.case.driver) in sys.path else self.case.repo
        else:
            # An existing canonical namespace constrains submodule lookup even
            # when a regular sealed parent has been added to sys.path.
            root = self.case.driver if path and Path(path[0]).is_relative_to(self.case.driver) else self.case.repo
        if fullname.endswith(".driver") and self.case.wrong_origin:
            root = self.case.repo
        package = fullname in self.PACKAGES
        location = root.joinpath(*fullname.split("."))
        location = location / "__init__.py" if package else location.with_suffix(".py")
        spec = importlib.machinery.ModuleSpec(fullname, MemoryLoader(self.case, fullname),
                                             origin=str(location), is_package=package)
        spec.has_location = True
        if package:
            spec.submodule_search_locations = [str(location.parent)]
        return spec


class BootstrapCase:
    def __init__(self):
        self.repo = Path("E:/entry-bootstrap-memory").absolute()
        self.runtime = "start-bootstrap"
        self.files = {}
        self.driver_imports = []
        self.wrong_origin = self.wrong_environment = False
        source_files = {"tools/__init__.py": b"regular sealed parent\n",
                        "tools/jenkins/__init__.py": b"sealed Jenkins package\n",
                        "tools/jenkins/deployment/driver.py": b"production driver contract\n",
                        "launcher.py": b"sealed launcher\n", "pipeline/steps.groovy": b"sealed steps\n"}
        facts = {name: {"bytes": len(raw), "sha256": entry.sha(raw)} for name, raw in source_files.items()}
        self.digest = entry.sha(entry.encoded({"schemaVersion": 1, "files": facts}))
        self.driver = self.repo / ".jenkins/runtime/drivers" / self.digest
        for name, raw in source_files.items():
            self.files[self.driver / name] = raw
        self.files[self.driver / "driver-manifest.json"] = entry.encoded(
            {"schemaVersion": 1, "driverDigest": self.digest, "files": facts})
        clients = {name: ("pinned client " + name).encode() for name in entry.CLIENT_SOURCES}
        clients["tools/jenkins-management/entry.py"] = ENTRY_SOURCE.read_bytes()
        code = {"runner.py": b"existing production boot\n", "source_preparation.py": b"pinned preparation\n",
                "zircon-management.groovy": b"pinned management pipeline\n"}
        self.config = {"schemaVersion": 1, "repositoryRoot": str(self.repo), "driverDigest": self.digest,
                       "runtimeOperationId": self.runtime, "ordinaryEntry": entry.CAPABILITY,
                       "files": {name: entry.sha(raw) for name, raw in code.items()},
                       "clientSources": {name: entry.sha(raw) for name, raw in clients.items()}}
        raw_config = entry.encoded(self.config)
        checksum = entry.sha(raw_config)
        self.capsule = self.repo / ".jenkins/runtime/management" / checksum
        self.files[self.capsule / "capsule.json"] = raw_config
        for name, raw in code.items():
            self.files[self.capsule / name] = raw
        for name, raw in clients.items():
            self.files[self.repo / name] = raw
        installed = {"schemaVersion": 1, "job": "zircon-management", "capsule": str(self.capsule),
                     "capsuleSha256": checksum, "driverDigest": self.digest, "runtimeOperationId": self.runtime,
                     "ordinaryEntry": entry.CAPABILITY, "formalAcceptance": False}
        self.files[self.repo / ".jenkins/state/management/installed.json"] = entry.encoded(installed)
        self.files[self.repo / ".jenkins/state/deployment/driver.json"] = entry.encoded({
            "driverDigest": self.digest, "root": str(self.driver), "runtimeOperationId": self.runtime})
        self.files[self.repo / ".jenkins/state/deployment-operation.json"] = entry.encoded(
            {"operationId": self.runtime, "state": "running"})
        self.files[self.repo / ".jenkins/runtime/python/python.exe"] = b"read-only runtime identity"
        self.manager = SimpleNamespace(health=lambda: {"controller": True, "agent": True, "plugins": True,
            "ready": True, "acceptingRequests": True, "quietingDown": False})

    def anchor_clients(self):
        old = self.capsule
        self.config = {**self.config, "clientRoot": "clients"}
        raw = entry.encoded(self.config)
        checksum = entry.sha(raw)
        self.capsule = self.repo / ".jenkins/runtime/management" / checksum
        self.files[self.capsule / "capsule.json"] = raw
        for name in self.config["files"]:
            self.files[self.capsule / name] = self.files[old / name]
        for name in self.config["clientSources"]:
            self.files[self.capsule / "clients" / name] = self.files[self.repo / name]
        path = self.repo / ".jenkins/state/management/installed.json"
        installed = json.loads(self.files[path])
        self.files[path] = entry.encoded({**installed, "capsule": str(self.capsule),
                                         "capsuleSha256": checksum})

    def preload(self):
        self.canonical_parent = ModuleType("tools")
        self.canonical_parent.__path__ = [str(self.repo / "tools")]
        self.canonical_parent.__spec__ = importlib.machinery.ModuleSpec("tools", loader=None, is_package=True)
        self.canonical_parent.__spec__.submodule_search_locations = self.canonical_parent.__path__
        self.dev = ModuleType("tools.dev")
        self.cargo = ModuleType("tools.dev.local_cargo")
        self.dev.local_cargo = self.cargo
        self.canonical_parent.dev = self.dev
        sys.modules.update({"tools": self.canonical_parent, "tools.dev": self.dev, "tools.dev.local_cargo": self.cargo})
        for name in ("tools.jenkins", "tools.jenkins.frontend", "tools.jenkins.contracts",
                     "tools.jenkins.deployment", "tools.jenkins.deployment.driver"):
            module = ModuleType(name)
            module.__file__ = str(self.repo.joinpath(*name.split(".")).with_suffix(".py"))
            module.__path__ = [str(self.repo.joinpath(*name.split(".")))]
            sys.modules[name] = module
        self.canonical_driver = sys.modules["tools.jenkins.deployment.driver"]

    def capsule_spec(self, name, path):
        if name not in {"verified_ordinary_management_runner", "verified_management_client_selector"} or Path(path) != self.capsule / "runner.py":
            raise AssertionError("Unexpected capsule import")
        spec = importlib.machinery.ModuleSpec(name, MemoryLoader(self, name), origin=str(path))
        spec.has_location = True
        return spec

    def read(self, path):
        return self.files[Path(path)]

    def walk(self, path, pattern):
        root = Path(path)
        return iter(item for item in self.files if item != root and item.is_relative_to(root))

    @contextmanager
    def active(self):
        saved = {name: module for name, module in sys.modules.items() if name == "tools" or name.startswith("tools.")}
        paths, finders = sys.path[:], sys.meta_path[:]
        try:
            for name in tuple(saved):
                del sys.modules[name]
            self.preload()
            sys.meta_path.insert(0, MemoryFinder(self))
            with ExitStack() as stack:
                stack.enter_context(patch.dict(os.environ, {}, clear=False))
                stack.enter_context(patch.object(entry, "plain", side_effect=lambda path: Path(path).absolute()))
                stack.enter_context(patch.object(Path, "read_bytes", new=lambda path: self.read(path)))
                stack.enter_context(patch.object(Path, "read_text",
                    new=lambda path, encoding=None, errors=None: self.read(path).decode(encoding or "utf-8")))
                stack.enter_context(patch.object(Path, "is_file", new=lambda path: Path(path) in self.files))
                stack.enter_context(patch.object(Path, "exists",
                    new=lambda path: Path(path) in self.files or any(item.is_relative_to(path) for item in self.files)))
                stack.enter_context(patch.object(Path, "is_symlink", new=lambda path: False))
                stack.enter_context(patch.object(Path, "lstat",
                    new=lambda path: SimpleNamespace(st_file_attributes=0)))
                stack.enter_context(patch.object(Path, "stat",
                    new=lambda path, **kwargs: SimpleNamespace(st_size=len(self.read(path)))))
                stack.enter_context(patch.object(Path, "rglob", new=lambda path, pattern: self.walk(path, pattern)))
                stack.enter_context(patch.object(entry.importlib.util, "spec_from_file_location", side_effect=self.capsule_spec))
                yield self
        finally:
            for name in tuple(sys.modules):
                if name == "tools" or name.startswith("tools."):
                    del sys.modules[name]
            sys.modules.update(saved)
            sys.path[:] = paths
            sys.meta_path[:] = finders


class EntryBootstrapTests(unittest.TestCase):
    def test_preloaded_public_client_namespace_rebinds_to_sealed_regular_parent(self):
        for anchored in (False, True):
            with self.subTest(anchored=anchored):
                case = BootstrapCase()
                if anchored:
                    case.anchor_clients()
                    for name in case.config["clientSources"]:
                        case.files[case.repo / name] += b"foreign canonical migration"
                namespace = {"Path": Path, "json": json, "JenkinsError": entry.EntryError}
                production_functions(FRONTEND_TREE, {"management_client_path"}, namespace, FRONTEND_SOURCE)
                with case.active():
                    path = namespace["management_client_path"](case.repo, "tools/jenkins-management/entry.py")
                    expected = (case.capsule / "clients" if anchored else case.repo) / "tools/jenkins-management/entry.py"
                    self.assertEqual(expected, path)
                    result = entry.capability(case.repo)
                    driver = importlib.import_module("tools.jenkins.deployment.driver")
                    self.assertEqual(Path(driver.__file__), case.driver / "tools/jenkins/deployment/driver.py")
                    self.assertIsNot(driver, case.canonical_driver)
                    self.assertIsNot(sys.modules["tools"], case.canonical_parent)
                    self.assertIsNotNone(sys.modules["tools"].__spec__.loader)
                    self.assertEqual(result["config"]["driverDigest"], case.digest)
                    self.assertFalse(result["installed"]["formalAcceptance"])

    def test_repeated_capability_rebinds_without_reusing_a_cached_driver(self):
        case = BootstrapCase()
        with case.active():
            first = entry.capability(case.repo)
            first_driver = sys.modules["tools.jenkins.deployment.driver"]
            second = entry.capability(case.repo)
            self.assertIsNot(sys.modules["tools.jenkins.deployment.driver"], first_driver)
            self.assertEqual(first["config"], second["config"])
            self.assertEqual(case.driver_imports, [str(case.driver / "tools/jenkins/deployment/driver.py")] * 2)

    def test_existing_tools_dev_siblings_are_preserved(self):
        case = BootstrapCase()
        with case.active():
            entry.capability(case.repo)
            entry.capability(case.repo)
            self.assertIs(sys.modules["tools.dev"], case.dev)
            self.assertIs(sys.modules["tools.dev.local_cargo"], case.cargo)

    def test_changed_client_is_rejected_before_package_or_environment_rebind(self):
        for anchored in (False, True):
            with self.subTest(anchored=anchored):
                case = BootstrapCase()
                if anchored:
                    case.anchor_clients()
                with case.active():
                    before = os.environ.get("ZIRCON_DRIVER_DIGEST")
                    root = case.capsule / "clients" if anchored else case.repo
                    case.files[root / "tools/jenkins/frontend.py"] += b"foreign change"
                    error = ValueError if anchored else entry.EntryError
                    expected_message = ('Pinned management client implementation changed' if anchored
                                        else 'Pinned public entry source changed')
                    with self.assertRaisesRegex(error, expected_message):
                        entry.capability(case.repo)
                    self.assertIs(sys.modules["tools"], case.canonical_parent)
                    self.assertEqual(case.driver_imports, [])
                    self.assertEqual(os.environ.get("ZIRCON_DRIVER_DIGEST"), before)

    def test_changed_driver_bytes_are_rejected_by_production_boot_before_import(self):
        case = BootstrapCase()
        with case.active():
            case.files[case.driver / "tools/jenkins/deployment/driver.py"] += b"foreign change"
            with self.assertRaisesRegex(ValueError, "Pinned driver source changed"):
                entry.capability(case.repo)
            self.assertEqual(case.driver_imports, [])

    def test_changed_manifest_cannot_redirect_the_selected_driver(self):
        case = BootstrapCase()
        with case.active():
            manifest_path = case.driver / "driver-manifest.json"
            manifest = json.loads(case.files[manifest_path])
            manifest["files"]["launcher.py"]["sha256"] = "0" * 64
            case.files[manifest_path] = entry.encoded(manifest)
            with self.assertRaisesRegex(ValueError, "Driver manifest differs"):
                entry.capability(case.repo)
            self.assertEqual(case.driver_imports, [])

    def test_driver_file_identity_is_still_enforced(self):
        case = BootstrapCase()
        with case.active():
            case.wrong_origin = True
            with self.assertRaises(entry.EntryError) as error:
                entry.capability(case.repo)
            self.assertEqual(error.exception.code, "driver_context_mismatch")

    def test_driver_environment_identity_is_still_enforced(self):
        case = BootstrapCase()
        with case.active():
            case.wrong_environment = True
            with self.assertRaises(entry.EntryError) as error:
                entry.capability(case.repo)
            self.assertEqual(error.exception.code, "driver_context_mismatch")



class AutomaticEntryContextTests(unittest.TestCase):
    def authorization(self, evidence):
        repo = Path("E:/ordinary-entry-memory").absolute()
        payload = {"action": "validate", "repositoryId": entry.sha(str(repo).casefold().encode()),
                   "sessionId": "automatic-chat", "requestId": "automatic-request",
                   "operationId": "automatic-operation", "ownedPaths": ["src/lib.rs"]}
        evidence = dict(evidence, repositoryId=payload["repositoryId"], repositoryRoot=str(repo))
        snapshot = {"repositoryId": payload["repositoryId"], "sessionId": payload["sessionId"],
                    "principal": "owner-sid", "paths": ["src"],
                    "actions": ["implementation", "validation"], "evidence": evidence}
        row = {"status": "active", "principal": snapshot["principal"],
               "paths": json.dumps(snapshot["paths"]), "actions": json.dumps(snapshot["actions"]),
               "evidence": json.dumps(evidence), "snapshot_digest": entry.sha(entry.encoded(snapshot))}
        @contextmanager
        def connection(value):
            yield SimpleNamespace(execute=lambda *args: SimpleNamespace(fetchone=lambda: row))
        return repo, payload, row, connection

    def test_automatic_policy_evidence_is_accepted_without_user_grant(self):
        evidence = {"source": "automatic-local-request", "policy": "local-validation-v1",
                    "kind": "ordinary-validation", "ownerPrincipal": "owner-sid",
                    "inputDigest": "a" * 64}
        repo, payload, row, connection = self.authorization(evidence)
        with patch.dict(os.environ, {"ZIRCON_JENKINS_SESSION_ID": ""}):
            with patch.object(entry, "connection", side_effect=connection):
                relative = entry.authorize(repo, payload)
                self.assertEqual(relative("src/lib.rs"), "src/lib.rs")
                changed = {**evidence, "ownerPrincipal": "another-owner",
                           "repositoryId": payload["repositoryId"], "repositoryRoot": str(repo)}
                row["evidence"] = json.dumps(changed)
                snapshot = {"repositoryId": payload["repositoryId"], "sessionId": payload["sessionId"],
                            "principal": row["principal"], "paths": ["src"],
                            "actions": ["implementation", "validation"], "evidence": changed}
                row["snapshot_digest"] = entry.sha(entry.encoded(snapshot))
                with self.assertRaisesRegex(entry.EntryError, "Original action authorization changed"):
                    entry.authorize(repo, payload)

    def test_submission_admits_context_and_queries_and_dry_runs_remain_read_only(self):
        evidence = {"source": "automatic-local-request", "policy": "local-validation-v1",
                    "kind": "ordinary-validation", "ownerPrincipal": "owner-sid",
                    "inputDigest": "a" * 64}
        repo, payload, row, connection = self.authorization(evidence)
        helper = SimpleNamespace(_managed_route_replay=lambda *args, **kwargs: None)
        module = SimpleNamespace(ensure_request_context=lambda *args: None, source_helper=lambda: helper)
        context = {"repo": repo, "config": {}, "manager": object(), "module": module}
        with patch.dict(os.environ, {"ZIRCON_JENKINS_SESSION_ID": ""}):
            with patch.object(entry, "connection", side_effect=connection):
                with patch.object(module, "ensure_request_context") as admit:
                    entry.authorize(repo, payload, context=context, automatic=True)
                    admit.assert_called_once_with(
                        context["config"], repo, context["manager"], payload, record_request=False)
        with patch.object(entry, "capability", return_value=context):
            with patch.object(entry, "input_payload", return_value=payload) as prepare:
                with patch.object(entry, "submit_payload", return_value={"status": "pending"}):
                    for options in ({"dry_run": True}, {}):
                        entry.dispatch(repo, payload, **options)
                        self.assertEqual(prepare.call_args.kwargs["automatic_context"], not bool(options))
                    prepare.reset_mock()
                    statements = []
                    @contextmanager
                    def readonly(value):
                        def execute(sql, parameters):
                            statements.append(sql)
                            result = row if "FROM authorizations" in sql else None
                            return SimpleNamespace(fetchone=lambda: result)
                        yield SimpleNamespace(execute=execute)
                    original = {**payload, "driverDigest": "d" * 64, "runtimeOperationId": "old-runtime"}
                    with patch.dict(os.environ, {"ZIRCON_JENKINS_SESSION_ID": ""}):
                        with patch.object(entry, "connection", side_effect=readonly):
                            with patch.object(module, "ensure_request_context") as admit:
                                result = entry.dispatch(repo, original, query_only=True)
                                admit.assert_not_called()
                    self.assertEqual("original_operation_missing", result["reasonCode"])
                    self.assertFalse(result["resubmitted"])
                    self.assertTrue(any("FROM authorizations" in sql for sql in statements))
                    self.assertTrue(any("FROM records" in sql for sql in statements))
                    prepare.assert_not_called()

if __name__ == "__main__":
    unittest.main()
