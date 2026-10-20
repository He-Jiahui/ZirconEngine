"""Focused proof and dispatch tests for actual-checkout validation."""
from __future__ import annotations

import copy
from contextlib import ExitStack, nullcontext
import importlib.util
import json
from pathlib import Path
import stat
from types import SimpleNamespace
import unittest
from unittest.mock import Mock, patch


ROOT = Path(__file__).resolve().parents[3]
RUNNER_PATH = ROOT / "tools/jenkins-management/runner.py"
_SPEC = importlib.util.spec_from_file_location("checkout_validation_test_runner", RUNNER_PATH)
runner = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(runner)

ENTRY_PATH = ROOT / "tools/jenkins-management/entry.py"
_ENTRY_SPEC = importlib.util.spec_from_file_location("checkout_validation_test_entry", ENTRY_PATH)
entry = importlib.util.module_from_spec(_ENTRY_SPEC)
_ENTRY_SPEC.loader.exec_module(entry)


class CheckoutValidationTests(unittest.TestCase):
    def actual_proof(self):
        source_root = str(ROOT.resolve())
        git_path = str(Path(ROOT.anchor) / "Program Files" / "Git" / "cmd" / "git.exe")
        git_fact = {
            "kind": "installed-git",
            "path": git_path,
            "sha256": "f" * 64,
            "device": 1,
            "fileId": "0" * 32,
            "size": 128,
            "mtimeNs": 256,
            "creationTime": 512,
            "linkCount": 2,
            "attributes": 0,
        }
        trusted_native_inputs = {git_path: git_fact}
        files = {"Cargo.toml": "b" * 64, git_path: git_fact["sha256"]}
        checkout_inputs = {
            "files": files,
            "sourceRoot": source_root,
            "trustedNativeInputs": trusted_native_inputs,
        }
        expected_inputs_digest = runner.checkout_expected_inputs_digest(checkout_inputs)
        guard = {
            "sourceRoot": source_root,
            "expectedInputsDigest": expected_inputs_digest,
            "trustedNativeInputs": trusted_native_inputs,
            "trustedNativeInputsDigest": runner.sha(runner.encoded(trusted_native_inputs)),
            "complete": True,
            "stable": True,
            "sourceNoWrites": True,
            "membershipStable": True,
            "cancellationComplete": True,
            "waitersJoined": True,
            "inputCopiesCreated": 0,
            "fileIdentityDigest": "c" * 64,
            "handleIdentities": [],
        }
        read_ref = {"path": str(ROOT / "synthetic-checkout-read.json"), "sha256": "d" * 64}
        result = {
            "validationInputPolicy": "actual-checkout",
            "capturedInputVerified": True,
            "inputsUnchanged": True,
            "originalEngineUnchanged": True,
            "dependencyCacheUnchanged": True,
            "externalInputsUnchanged": True,
            "checkoutInputsUnchanged": True,
            "sourceNoWrites": True,
            "membershipStable": True,
            "sourceRoot": source_root,
            "executionRoot": source_root,
            "sourceDigest": "e" * 64,
            "checkoutInputsDigest": runner.sha(runner.encoded(checkout_inputs)),
            "expectedInputsDigest": expected_inputs_digest,
            "checkoutReadRef": read_ref,
            "checkoutGuard": guard,
            "checkoutGuardRef": {
                "path": str(ROOT / "synthetic-checkout-guard.json"),
                "sha256": runner.sha(runner.encoded(guard) + b"\n"),
            },
        }
        request = {
            "validationInputPolicy": "actual-checkout",
            "capturedInputVerified": True,
            "payload": {"sourceView": "checkout"},
            "source": {"sourceDigest": result["sourceDigest"]},
            "sourceRoot": source_root,
            "executionRoot": source_root,
            "checkoutInputsDigest": result["checkoutInputsDigest"],
            "expectedInputsDigest": result["expectedInputsDigest"],
            "checkoutReadRef": read_ref,
            "checkoutInputs": checkout_inputs,
        }
        return result, request

    def test_actual_checkout_accepts_complete_bound_guard_proof(self):
        result, request = self.actual_proof()

        self.assertEqual(
            runner.checkout_expected_inputs_digest(request["checkoutInputs"]),
            result["expectedInputsDigest"],
        )
        self.assertTrue(runner.checkout_trusted_native_verified(
            request["checkoutInputs"], result["checkoutGuard"]
        ))
        self.assertTrue(runner.validation_inputs_verified(result))
        self.assertTrue(runner.validation_inputs_verified(result, request))

        actor = runner.Management.__new__(runner.Management)
        engine_files = {"src/owned.rs": "a" * 64}
        external_files = {f"dep/{index}.rs": "b" * 64 for index in range(128)}
        declared_dependencies = {
            **engine_files,
            **{f"external-zr_vm/{path}": checksum for path, checksum in external_files.items()},
        }
        captured = {
            "source": {"files": engine_files},
            "externalRoots": [{"source": {"files": external_files}}],
        }
        declarations = {
            "ownedPaths": ["src/owned.rs"],
            "sourceHashes": engine_files,
            "dependencyHashes": declared_dependencies,
        }
        self.assertEqual(len(declared_dependencies), 129)
        self.assertIsNone(actor.verify_declared_inputs(captured, declarations))

    def test_actual_checkout_rejects_incomplete_or_mismatched_proof(self):
        result, request = self.actual_proof()
        cases = []

        changed = copy.deepcopy(result)
        changed["checkoutGuard"]["waitersJoined"] = False
        changed["checkoutGuardRef"]["sha256"] = runner.sha(runner.encoded(changed["checkoutGuard"]) + b"\n")
        cases.append((changed, request))

        changed = copy.deepcopy(result)
        changed["sourceNoWrites"] = False
        cases.append((changed, request))

        changed = copy.deepcopy(result)
        changed["checkoutGuard"]["trustedNativeInputsDigest"] = "0" * 64
        changed["checkoutGuardRef"]["sha256"] = runner.sha(
            runner.encoded(changed["checkoutGuard"]) + b"\n"
        )
        cases.append((changed, request))

        changed = copy.deepcopy(result)
        trust_path = next(iter(changed["checkoutGuard"]["trustedNativeInputs"]))
        changed["checkoutGuard"]["trustedNativeInputs"][trust_path]["linkCount"] += 1
        changed["checkoutGuard"]["trustedNativeInputsDigest"] = runner.sha(
            runner.encoded(changed["checkoutGuard"]["trustedNativeInputs"])
        )
        changed["checkoutGuardRef"]["sha256"] = runner.sha(
            runner.encoded(changed["checkoutGuard"]) + b"\n"
        )
        cases.append((changed, request))

        changed_request = copy.deepcopy(request)
        changed_request["checkoutInputs"]["files"]["Cargo.toml"] = "f" * 64
        cases.append((result, changed_request))

        changed_request = copy.deepcopy(request)
        trust_path = next(iter(changed_request["checkoutInputs"]["trustedNativeInputs"]))
        changed_request["checkoutInputs"]["trustedNativeInputs"][trust_path]["linkCount"] += 1
        changed_request["checkoutInputsDigest"] = runner.sha(
            runner.encoded(changed_request["checkoutInputs"])
        )
        changed_request["expectedInputsDigest"] = runner.checkout_expected_inputs_digest(
            changed_request["checkoutInputs"]
        )
        cases.append((result, changed_request))

        for candidate, candidate_request in cases:
            with self.subTest(result=candidate, request=candidate_request):
                self.assertFalse(runner.validation_inputs_verified(candidate, candidate_request))


    def test_single_checkout_dispatch_skips_sealed_source_preparation(self):
        actor = runner.Management.__new__(runner.Management)
        actor.payload = {"sourceView": "checkout"}
        expected = {"status": "stub"}
        actor.validate_checkout = Mock(return_value=expected)

        with patch.object(runner, "source_helper") as seal_or_materialize:
            self.assertIs(actor.validate_prepared(), expected)

        actor.validate_checkout.assert_called_once_with()
        seal_or_materialize.assert_not_called()

    def test_batch_checkout_dispatch_skips_sealed_source_preparation(self):
        actor = runner.Management.__new__(runner.Management)
        actor.payload = {"sourceView": "checkout"}
        batch = {"steps": []}
        expected = {"status": "stub"}
        actor.validation_batch = Mock(return_value=batch)
        actor.validate_checkout = Mock(return_value=expected)

        with patch.object(runner, "source_helper") as seal_or_materialize:
            self.assertIs(actor.validate_batch(), expected)

        actor.validation_batch.assert_called_once_with()
        actor.validate_checkout.assert_called_once_with(batch=batch)
        seal_or_materialize.assert_not_called()


    def test_legacy_sealed_checkpoint_keeps_worktree_contract_only(self):
        source_digest, sealed_ref = "a" * 64, "b" * 64
        preparation = {
            "sealedInputRef": sealed_ref,
            "sourceDigest": sealed_ref,
            "capturedSourceDigest": source_digest,
        }
        legacy_result = {
            "validationInputPolicy": "sealed-source",
            "capturedInputVerified": True,
            "originalEngineUnchanged": True,
            "dependencyCacheUnchanged": True,
            "preparedInputsUnchanged": True,
            "sourceDigest": source_digest,
            "preparation": preparation,
        }
        legacy_request = {
            "validationInputPolicy": "sealed-source",
            "capturedInputVerified": True,
            "preparation": preparation,
            "preparedSource": {"sourceDigest": sealed_ref},
            "source": {"sourceDigest": source_digest},
            "inputObservation": {
                "originalEngineUnchanged": True,
                "dependencyCacheUnchanged": True,
            },
        }
        self.assertTrue(runner.validation_inputs_verified(legacy_result, legacy_request))
        legacy_result["validationInputPolicy"] = "actual-checkout"
        legacy_request["validationInputPolicy"] = "actual-checkout"
        legacy_request["payload"] = {"sourceView": "checkout"}
        self.assertFalse(runner.validation_inputs_verified(legacy_result, legacy_request))

    def checkout_git_input_error(self, git_path, *, root, cache, externals):
        """Exercise the real input collector with only filesystem/native observations mocked."""
        actor = Mock(spec=runner.Management)
        actor._checkout_git_binding = {
            "kind": "installed-git", "path": str(git_path), "sha256": "f" * 64,
            "device": 7, "fileId": "b" * 32,
        }
        actor._checkout_native_environment = None
        actor.text.side_effect = lambda path, *_args: str(path / ".git")
        actor.git.return_value = SimpleNamespace(stdout=b"")
        preparation, helper = Mock(), Mock()
        preparation.source_files.return_value = {}
        helper._file_fact.side_effect = FileNotFoundError
        before = {
            "sourceRoot": str(root), "source": {"files": {}},
            "externalRoots": [{"path": str(path), "source": {"files": {}}} for path in externals],
            "dependencyCache": {"home": str(cache), "configSha256": {}},
        }

        def lstat(path):
            if path.name == ".git":
                return SimpleNamespace(st_mode=stat.S_IFDIR)
            raise FileNotFoundError(path)

        guard_helper = Mock()
        guard_helper.capture_trusted_git_input.return_value = dict(actor._checkout_git_binding)
        with ExitStack() as stack:
            stack.enter_context(patch.object(runner, "plain", side_effect=Path))
            stack.enter_context(patch.object(runner, "checkout_guard_helper", return_value=guard_helper))
            stack.enter_context(patch.object(Path, "is_dir", return_value=True))
            stack.enter_context(patch.object(Path, "glob", return_value=()))
            stack.enter_context(patch.object(Path, "lstat", autospec=True, side_effect=lstat))
            stack.enter_context(patch.object(Path, "home", return_value=root.parent / "home"))
            with self.assertRaises(runner.CheckoutInputIncomplete) as caught:
                runner.Management.checkout_read_inputs(
                    actor, before, {"plainFileFacts": {}}, preparation, helper)
        return caught.exception

    def test_installed_git_rejects_checkout_cache_and_external_inputs(self):
        base = Path(ROOT.anchor) / "synthetic-checkout-git-inputs"
        root, cache, external = (base / name for name in ("checkout", "cargo-home", "external"))
        for role, source in (("checkout", root), ("dependency-cache", cache), ("external", external)):
            for git_path in (source, source / "cmd/git.exe"):
                with self.subTest(role=role, path=str(git_path)):
                    error = self.checkout_git_input_error(
                        git_path, root=root, cache=cache, externals=[external])
                    self.assertEqual(str(error), "Installed Git cannot be a checkout or dependency-cache input")
                    self.assertEqual(error.input_diagnostic, {
                        "schemaVersion": 1, "kind": "installed-git-input-overlap",
                        "selectedGit": {"path": str(git_path), "sha256": "f" * 64},
                        "matchedInput": {"role": role, "root": str(source)},
                    })
        for nested_cache, nested_external, role, source in (
                (root / "cache", root / "cache/external", "checkout", root),
                (cache, cache / "external", "dependency-cache", cache)):
            with self.subTest(first_matching_root=role):
                error = self.checkout_git_input_error(
                    nested_external / "cmd/git.exe", root=root, cache=nested_cache, externals=[nested_external])
                self.assertEqual(error.input_diagnostic["matchedInput"], {"role": role, "root": str(source)})

    def test_installed_git_allows_separate_and_sibling_tool_roots(self):
        base = Path(ROOT.anchor) / "synthetic-checkout-git-inputs"
        root, cache, external = (base / name for name in ("checkout", "cargo-home", "external"))
        for tools in (base / "installed-tools", *(Path(str(path) + "-tools") for path in (root, cache, external))):
            with self.subTest(tools=str(tools)):
                error = self.checkout_git_input_error(
                    tools / "cmd/git.exe", root=root, cache=cache, externals=[external])
                self.assertEqual(str(error), "The actual trusted native environment is unavailable")
                self.assertIsNone(error.input_diagnostic)

    def test_checkout_input_incomplete_serializes_bounded_prelaunch_diagnostic(self):
        git_path = ROOT / ".jenkins/runtime/git/cmd/git.exe"
        cache = Path(ROOT.anchor) / "cargo-targets/synthetic-checkout-git-cache"
        overlap = self.checkout_git_input_error(git_path, root=ROOT, cache=cache, externals=[])
        diagnostic = copy.deepcopy(overlap.input_diagnostic)
        supplied = {
            **diagnostic, "PATH": "must-not-be-serialized", "status": "success",
            "selectedGit": {**diagnostic["selectedGit"], "fileId": "must-not-be-serialized"},
            "matchedInput": {**diagnostic["matchedInput"], "environment": {"SECRET": "must-not-be-serialized"}},
        }
        errors = (
            runner.CheckoutInputIncomplete(str(overlap), input_diagnostic=supplied),
            runner.CheckoutInputIncomplete("An unrelated checkout input is incomplete"),
        )
        tool_bytes = b"synthetic checkout guard"
        checksum = runner.sha(tool_bytes)
        trusted = {key: str(ROOT / (tool + ".exe")) if key.endswith("Path") else checksum
                   for tool in ("cargo", "rustc") for key in (tool + "Path", tool + "Sha256")}
        policy = {"cpuBudget": 8, "memoryBudget": 64 * 1024**3, "diskBudget": 1024**4}
        build_root = str(Path(ROOT.anchor) / "cargo-targets")
        output = Path(build_root) / "synthetic-checkout-git-diagnostic-no-files"
        before = {"sourceRoot": str(ROOT), "sourceDigest": "a" * 64, "source": {"files": {}},
                  "externalRoots": [], "dependencyCache": {"home": str(cache)}}
        for error in errors:
            for guarded in (False, True):
                with self.subTest(diagnostic=error.input_diagnostic is not None, guard_acquired=guarded):
                    actor = Mock(spec=runner.Management)
                    actor.repo, actor.repo_id, actor.key = ROOT, "repo", "git-diagnostic"
                    actor.payload = {"sourceView": "checkout"}
                    actor.config = {"files": {"checkout_guard.py": checksum},
                                    "clientSources": {"tools/dev/local_cargo.py": checksum},
                                    "driverDigest": "b" * 64, "runtimeOperationId": "c" * 64}
                    actor.validation_root.return_value = ROOT
                    actor.validation_recipe.return_value = {
                        "jobs": 1, "testThreads": 1, "verb": "test", "noRun": False, "allFeatures": False,
                        "packages": ["zircon_runtime"], "features": ["dynamic-api"],
                        "profile": "test", "noDefaultFeatures": True,
                    }
                    actor.build_root.return_value, actor.outputs.return_value = build_root, output
                    actor.state = Mock()
                    actor.state.get.side_effect = lambda table, _key: (
                        {"payload": policy} if table == "resource_policy" else None)
                    actor.validation_environment.return_value = ({"PATH": "trusted"}, trusted, ROOT)
                    actor.validation_with_cache.return_value = (ROOT, before)
                    actor.validation_observation.return_value = (ROOT, before, {})
                    actor.checkout_git_read_conflict.return_value = None
                    actor.checkout_read_inputs.side_effect = [({}, []), error] if guarded else error
                    guard = actor.acquire_checkout_guard.return_value
                    preparation, helper, resources = Mock(), Mock(), Mock()
                    preparation.preparation_key.return_value = "git-diagnostic-pool"
                    preparation.trusted_toolchain_facts.return_value = {"binaries": {
                        tool: {"path": trusted[tool + "Path"], "sha256": checksum}
                        for tool in ("cargo", "rustc")}}
                    approved = SimpleNamespace(path=Path(build_root))
                    module_spec = SimpleNamespace(loader=Mock())
                    with ExitStack() as stack:
                        for target, replacement in (
                            ("tools.jenkins.resources.capacity.ResourceManager", Mock(return_value=resources)),
                            ("tools.jenkins.resources.paths.canonical_build_root", Mock(return_value=approved)),
                            ("tools.jenkins.resources.paths.physical_path_under", lambda _root, path: Path(path)),
                            ("tools.jenkins.state.locks.process_lock", lambda *_args, **_options: nullcontext()),
                        ):
                            stack.enter_context(patch(target, replacement))
                        stack.enter_context(patch.object(runner, "plain", side_effect=Path))
                        stack.enter_context(patch.object(runner, "source_helper", return_value=preparation))
                        stack.enter_context(patch.object(runner, "checkout_guard_helper", return_value=Mock()))
                        stack.enter_context(patch.object(runner.importlib.util, "spec_from_file_location",
                                                         return_value=module_spec))
                        stack.enter_context(patch.object(runner.importlib.util, "module_from_spec", return_value=helper))
                        stack.enter_context(patch.object(runner, "write_json", side_effect=AssertionError("no report write")))
                        stack.enter_context(patch.object(Path, "read_bytes", return_value=tool_bytes))
                        stack.enter_context(patch.object(Path, "exists", return_value=False))
                        stack.enter_context(patch.object(Path, "mkdir", side_effect=AssertionError("no directory creation")))
                        result = runner.Management.validate_checkout(actor)
                    expected = {
                        "status": "waiting", "reasonCode": "checkout_input_incomplete", "message": str(error),
                        "inputsComplete": False, "compileStarted": False,
                        "formalAcceptance": False, "m8Accepted": False,
                    }
                    if error is errors[0]:
                        expected["inputDiagnostic"] = diagnostic
                    self.assertEqual(json.loads(json.dumps(result)), expected)
                    self.assertEqual(actor.checkout_read_inputs.call_count, 2 if guarded else 1)
                    if guarded:
                        guard.close.assert_called_once_with()
                    else:
                        actor.acquire_checkout_guard.assert_not_called()
                    actor.begin_validation_intent.assert_not_called()
                    actor.state.put.assert_not_called()
                    resources.queue.assert_not_called()
                    resources.admit.assert_not_called()
                    helper._run_owned_command.assert_not_called()

    def test_checkout_input_identity_conflict_records_both_observations(self):
        files, folded = {}, {}
        first = r"E:\Git\ZirconEngine\docs\plans\source-comments\assignments.json"
        second = first.lower()
        runner._record_checkout_input_identity(files, folded, first, "a" * 64)

        with self.assertRaises(runner.CheckoutInputIdentityConflict) as caught:
            runner._record_checkout_input_identity(files, folded, second, "b" * 64)

        error = caught.exception
        self.assertEqual(error.code, "checkout_input_identity_conflict")
        self.assertEqual(error.path, second)
        self.assertEqual(error.previous_path, first)
        self.assertEqual(error.before_sha256, "a" * 64)
        self.assertEqual(error.after_sha256, "b" * 64)

    def test_checkout_input_conflict_is_prelaunch_only_without_claims(self):
        owner = "management-" + "a" * 64
        writer = "management-pool-" + "b" * 64
        error = runner.CheckoutInputIdentityConflict(
            r"E:\repo\docs\source.json", r"E:\repo\docs\source.json",
            "c" * 64, "d" * 64)

        class MemoryState:
            def __init__(self, rows=None):
                self.rows = dict(rows or {})

            def connect(self):
                return nullcontext(self)

            def get(self, domain, key, *, connection=None):
                return self.rows.get((domain, key))

            def list(self, domain, *, connection=None):
                return [{"domain": name, "key": key, **row}
                        for (name, key), row in self.rows.items() if name == domain]

            def put(self, *args, **kwargs):
                raise AssertionError("prelaunch conflict classification must not write State")

        actor = runner.Management.__new__(runner.Management)
        actor._checkout_context = None
        actor._validation_admission = None
        actor.state = MemoryState()
        guard = Mock()

        result = actor.checkout_input_conflict_result(
            error, owner=owner, writer=writer, guard=guard, admitted=None)

        self.assertEqual(result["status"], "conflict")
        self.assertEqual(result["reasonCode"], "checkout_input_identity_conflict")
        self.assertEqual(result["conflictingInput"], {
            "path": error.path,
            "previousPath": error.previous_path,
            "beforeSha256": "c" * 64,
            "afterSha256": "d" * 64,
        })
        self.assertFalse(result["compileStarted"])
        self.assertFalse(result["formalAcceptance"])
        self.assertEqual(actor.state.rows, {})
        guard.close.assert_called_once_with()

    def test_checkout_input_conflict_does_not_downgrade_existing_claims(self):
        owner = "management-" + "a" * 64
        writer = "management-pool-" + "b" * 64
        error = runner.CheckoutInputIdentityConflict(
            r"E:\repo\docs\source.json", r"E:\repo\docs\source.json",
            "c" * 64, "d" * 64)
        claims = [
            ("management_checkout_read", owner, {"payload": {"status": "active"}}),
            ("execution_launch", owner, {"payload": {"status": "claimed"}}),
            ("execution_host", owner, {"payload": {"status": "active"}}),
            ("resource_reservation", owner + ":" + owner, {"payload": {"status": "active"}}),
            ("native_job", owner + "-native", {"payload": {"owner": owner, "executionId": owner}}),
        ]

        class MemoryState:
            def __init__(self, row):
                self.rows = {(row[0], row[1]): row[2]}

            def connect(self):
                return nullcontext(self)

            def get(self, domain, key, *, connection=None):
                return self.rows.get((domain, key))

            def list(self, domain, *, connection=None):
                return [{"domain": name, "key": key, **row}
                        for (name, key), row in self.rows.items() if name == domain]

        for domain, key, row in claims:
            with self.subTest(domain=domain):
                actor = runner.Management.__new__(runner.Management)
                actor._checkout_context = None
                actor._validation_admission = None
                actor.state = MemoryState((domain, key, row))
                guard = Mock()

                result = actor.checkout_input_conflict_result(
                    error, owner=owner, writer=writer, guard=guard, admitted=None)

                self.assertIsNone(result)
                guard.close.assert_not_called()

    def test_unknown_checkout_preserves_reader_and_writer_reservation(self):
        owner = "management-" + "a" * 64
        read_ref = {"path": str(ROOT / "checkout-read.json"), "sha256": "b" * 64}

        class MemoryState:
            def __init__(self):
                self.rows = {
                    ("management_checkout_read", owner): {"payload": {
                        "status": "active", "checkoutReadRef": read_ref,
                        "reservationKey": "reservation-1", "preparationKey": "writer-1",
                        "pinsReleased": False, "continuousReadProven": True,
                    }, "version": 3},
                    ("execution", owner): {"payload": {"status": "active"}, "version": 2},
                    ("execution_launch", owner): {"payload": {"status": "active"}, "version": 2},
                }

            def get(self, domain, key):
                return self.rows.get((domain, key))

            def put(self, domain, key, payload, expected_version=None):
                previous = self.rows[(domain, key)]
                if expected_version != previous["version"]:
                    raise AssertionError("unexpected state version")
                updated = {"payload": payload, "version": previous["version"] + 1}
                self.rows[(domain, key)] = updated
                return updated

        state, guard, resources = MemoryState(), Mock(), Mock()
        actor = runner.Management.__new__(runner.Management)
        actor.state = state
        actor._checkout_context = {
            "owner": owner, "root": ROOT.resolve(),
            "before": {"sourceDigest": "c" * 64}, "inputsDigest": "d" * 64,
            "readRef": read_ref, "guard": guard, "resources": resources,
        }

        result = actor.checkout_unknown("native_unproven", message="native completion unknown")
        read = state.rows[("management_checkout_read", owner)]["payload"]

        self.assertEqual(result["status"], "unknown")
        self.assertTrue(result["originalReservationRetained"])
        self.assertTrue(result["originalWriterRetained"])
        self.assertEqual(read["status"], "unknown")
        self.assertEqual(read["checkoutReadRef"], read_ref)
        self.assertEqual(read["reservationKey"], "reservation-1")
        self.assertEqual(read["preparationKey"], "writer-1")
        self.assertFalse(read["pinsReleased"])
        self.assertFalse(read["continuousReadProven"])
        self.assertEqual(state.rows[("execution", owner)]["payload"]["status"], "unknown")
        self.assertEqual(state.rows[("execution_launch", owner)]["payload"]["status"], "unknown")
        guard.finish.assert_not_called()
        guard.close.assert_not_called()
        resources.release.assert_not_called()


class CheckoutEntryCapabilityTests(unittest.TestCase):
    checkout_capability = {
        "enabled": True,
        "policy": "actual-checkout",
        "guard": "checkout_guard.py",
    }
    legacy_files = {
        "runner.py": "a" * 64,
        "source_preparation.py": "b" * 64,
        "zircon-management.groovy": "c" * 64,
    }

    class TinyManagement:
        def validation_recipe(self):
            return None

        def validation_batch(self):
            return None

    def input_context(self, files, checkout_validation):
        return {
            "repo": ROOT,
            "config": {
                "driverDigest": "d" * 64,
                "runtimeOperationId": "e" * 64,
                "files": files,
                "checkoutValidation": checkout_validation,
            },
            "manager": None,
            "module": SimpleNamespace(Management=self.TinyManagement,
                                      route_request_context=runner.route_request_context),
        }

    def checkout_payload(self):
        return {
            "action": "validate",
            "driverDigest": "d" * 64,
            "runtimeOperationId": "e" * 64,
            "sourceView": "checkout",
            "verb": "check",
            "packages": ["zircon-engine"],
            "externalRoots": [str(entry.plain(ROOT.parent / "zr_vm"))],
        }

    def invoke(self, files, checkout_validation):
        context = self.input_context(files, checkout_validation)
        with patch.object(entry, "authorize") as authorization, \
                patch.object(entry, "output_policy", return_value={}) as output_policy:
            value = entry.input_payload(context, self.checkout_payload())
        authorization.assert_called_once()
        output_policy.assert_called_once()
        return value

    def test_checkout_payload_rejects_legacy_three_file_capsule(self):
        with self.assertRaises(entry.EntryError) as caught:
            self.invoke(self.legacy_files, None)

        self.assertEqual(caught.exception.code, "checkout_input_capability_pending")

    def test_checkout_payload_accepts_exact_four_file_guard_capability(self):
        files = {**self.legacy_files, "checkout_guard.py": "f" * 64}

        value = self.invoke(files, self.checkout_capability)

        self.assertEqual(value["sourceView"], "checkout")
        self.assertEqual(value["externalRoots"], [str(entry.plain(ROOT.parent / "zr_vm"))])

    def test_checkout_payload_rejects_missing_guard_or_wrong_marker(self):
        full_marker_without_guard = (self.legacy_files, self.checkout_capability)
        wrong_marker_with_guard = (
            {**self.legacy_files, "checkout_guard.py": "f" * 64},
            {"enabled": True, "policy": "sealed-source", "guard": "checkout_guard.py"},
        )

        for files, marker in (full_marker_without_guard, wrong_marker_with_guard):
            with self.subTest(files=sorted(files), marker=marker):
                with self.assertRaises(entry.EntryError) as caught:
                    self.invoke(files, marker)
                self.assertEqual(caught.exception.code, "checkout_input_capability_pending")


class FixedHistoryJenkinsInputTests(unittest.TestCase):
    def archive(self):
        return {"schemaVersion": 1, "kind": "zircon-fixed-docs-history", "access": "read-only", "root": str(ROOT)}

    def test_typed_archive_is_bound_to_recipe_and_trusted_native_environment(self):
        actor = runner.Management.__new__(runner.Management)
        actor.payload = {"packages": ["zircon_runtime"], "lib": True, "noDefaultFeatures": True,
                         "fixedHistoryArchiveInput": self.archive()}
        actor.repo, actor.state = ROOT, Mock()
        recipe = actor.validation_recipe()
        self.assertEqual(recipe["fixedHistoryArchiveInput"]["root"], str(ROOT.resolve()))
        trusted = {"PATH": "trusted", "ZIRCON_DOCS_HISTORY_ARCHIVE_ROOT": "untrusted-ambient"}
        with patch("tools.jenkins.workflow.planning._environment_policy", return_value=(trusted, {}, ROOT)):
            environment, _, _ = actor.validation_environment()
            self.assertEqual(environment["ZIRCON_DOCS_HISTORY_ARCHIVE_ROOT"], str(ROOT.resolve()))
            self.assertNotEqual(runner.sha(runner.encoded(environment)), runner.sha(runner.encoded(trusted)))
            del actor.payload["fixedHistoryArchiveInput"]
            environment, _, _ = actor.validation_environment()
            self.assertNotIn("ZIRCON_DOCS_HISTORY_ARCHIVE_ROOT", environment)
        self.assertEqual(trusted["ZIRCON_DOCS_HISTORY_ARCHIVE_ROOT"], "untrusted-ambient")

    def test_three_native_routes_use_the_same_typed_environment_owner(self):
        import ast
        tree = ast.parse(RUNNER_PATH.read_text(encoding="utf-8"))
        management = next(node for node in tree.body if isinstance(node, ast.ClassDef) and node.name == "Management")
        for name in ("validate_batch", "validate_checkout", "validate_prepared"):
            method = next(node for node in management.body if isinstance(node, ast.FunctionDef) and node.name == name)
            calls = [node for node in ast.walk(method) if isinstance(node, ast.Call)
                     and isinstance(node.func, ast.Attribute) and node.func.attr == "validation_environment"]
            self.assertEqual(len(calls), 1, name)
            policy_calls = [node for node in ast.walk(method) if isinstance(node, ast.Call)
                            and isinstance(node.func, ast.Name) and node.func.id == "_environment_policy"]
            self.assertEqual(policy_calls, [], name)

    def test_archive_schema_and_readability_fail_closed(self):
        invalid = [None, {}, {**self.archive(), "access": "read-write"},
                   {**self.archive(), "schemaVersion": True}, {**self.archive(), "extra": "env"},
                   {**self.archive(), "root": "relative/archive"}, {**self.archive(), "root": ""},
                   {**self.archive(), "root": str(RUNNER_PATH)}]
        for value in invalid:
            with self.subTest(value=value), self.assertRaises(ValueError):
                runner.fixed_history_archive_input({"fixedHistoryArchiveInput": value})
        with patch.object(runner.os, "scandir", side_effect=PermissionError("unreadable")):
            with self.assertRaises(PermissionError):
                runner.fixed_history_archive_input({"fixedHistoryArchiveInput": self.archive()})
        with patch.object(runner.Path, "resolve", side_effect=FileNotFoundError("missing")):
            with self.assertRaises(FileNotFoundError):
                runner.fixed_history_archive_input({"fixedHistoryArchiveInput": self.archive()})
        self.assertIsNone(runner.fixed_history_archive_input({}))

    def test_unknown_environment_fields_remain_rejected_by_cargo_admission(self):
        actor = runner.Management.__new__(runner.Management)
        actor.payload = {"packages": ["zircon_runtime"], "environment": {"ARBITRARY": "value"}}
        with self.assertRaisesRegex(ValueError, "Unsupported Cargo fields"):
            actor.validation_recipe()


    def test_real_entry_routes_bind_archive_through_request_identity_to_native_seam(self):
        from contextlib import ExitStack
        from types import MethodType
        from tools.dev import local_cargo

        class NativeReached(BaseException):
            pass

        class MemoryState:
            def __init__(self):
                self.rows = {}

            def get(self, table, key):
                return self.rows.get((table, key))

            def put(self, table, key, payload, **_options):
                row = {"key": key, "payload": copy.deepcopy(payload),
                       "version": self.rows.get((table, key), {}).get("version", 0) + 1}
                self.rows[(table, key)] = row
                return row

        def exercise(route, archive):
            actor = Mock(spec=runner.Management)
            actor.repo, actor.repo_id = ROOT, "repo"
            actor.session, actor.request, actor.operation, actor.key = "session", "request", "operation", "archive-route"
            actor.input_digest = "b" * 64
            actor.payload = {"packages": ["zircon_runtime"], "lib": True, "noDefaultFeatures": True,
                             "sourceView": "checkout" if route == "validate_checkout" else "worktree",
                             "worktreeId": "source" if route != "validate_checkout" else None,
                             "fixedHistoryArchiveInput": {**self.archive(), "root": str(archive)}}
            actor.validation_recipe = MethodType(runner.Management.validation_recipe, actor)
            actor.validation_environment = MethodType(runner.Management.validation_environment, actor)
            recipe = actor.validation_recipe()
            output = ROOT / "synthetic-archive-routing-no-files-created"
            build_root = str(Path(ROOT.anchor) / "cargo-targets")
            before = {"sourceDigest": "a" * 64, "sourceRoot": str(ROOT), "sourceView": actor.payload["sourceView"],
                      "source": {"files": {}}, "externalRoots": [],
                      "dependencyCache": {"home": str(output / "cache")}}
            original = {"ordinal": 1, "name": "history", "cargoArguments": ["test", "-p", "zircon_runtime", "--lib",
                                                                       "--", "--exact", "history-route-case"],
                        "ownedNativeTimeoutSeconds": 10}
            coverage = {"schemaVersion": 1, "recipes": [original]}
            batch = {"recipes": [recipe], "coverage": coverage, "reuseValidation": None, "stepEnvironments": [{}]}
            if route in ("validate_batch", "validate_checkout"):
                actor.payload["batch"] = {"recipes": [original]}
            actor.validation_batch.return_value = batch
            actor.validation_with_cache.return_value = (ROOT, before)
            actor.validation_root.return_value = ROOT
            actor.validation_observation.return_value = (ROOT, before, {"originalEngineUnchanged": True,
                                                                        "dependencyCacheUnchanged": True})
            actor.build_root.return_value = build_root
            actor.outputs.return_value = output
            actor.wt.return_value = (None, ROOT)
            actor.dependency_cache.return_value = before["dependencyCache"]
            actor.batch_pool.return_value = ("archive-pool", {})
            actor.batch_claims.return_value = []
            checkpoint = {}
            actor.batch_checkpoint.side_effect = lambda **fields: (checkpoint.update(fields) or checkpoint)
            actor.task_tmp.return_value = {"path": "synthetic-temp", "sha256": "c" * 64}
            actor.checkout_git_read_conflict.return_value = None
            actor.checkout_read_inputs.return_value = ({"watchRoots": [], "watchInterests": []}, [])
            guard = Mock()
            guard.assert_stable = Mock()
            guard.facts.return_value = {"expectedInputsDigest": "d" * 64, "fileIdentityDigest": "e" * 64}
            actor.acquire_checkout_guard.return_value = guard
            tool_bytes = b"synthetic trusted compiler"
            checksum = runner.sha(tool_bytes)
            actor.config = {"files": {"checkout_guard.py": checksum},
                            "driverDigest": "f" * 64, "runtimeOperationId": "0" * 64}
            trusted = {key: str(ROOT / (tool + ".exe")) if key.endswith("Path") else checksum
                       for tool in ("cargo", "rustc", "rustdoc") for key in (tool + "Path", tool + "Sha256")}
            policy = {"cpuBudget": 8, "memoryBudget": 64 * 1024**3, "diskBudget": 1024**4, "diskReserveBytes": 0}
            actor.state = MemoryState()
            actor.state.put("resource_policy", "default", policy)
            owner = "management-" + actor.key
            reservation_key = owner + ":" + owner
            writer = "management-pool-archive-pool"
            resources = Mock()
            resources.scan_inventory.return_value = (None, 1)

            def admit(*_args, **_kwargs):
                row = actor.state.put("resource_reservation", reservation_key,
                                      {"owner": owner, "buildRoot": build_root, "status": "active"})
                actor.state.put("pool_writer_hold", writer,
                                {"owner": owner, "status": "active", "reservationKey": reservation_key})
                return row

            resources.admit.side_effect = admit
            preparation = Mock()
            preparation.preparation_key.return_value = "archive-pool"
            sealed_ref = runner.sha(runner.encoded({}))

            def seal(*_args, **options):
                return {"capturedInputVerified": True, "capturedSourceDigest": before["sourceDigest"],
                        "coverageDigest": runner.sha(runner.encoded(options["coverage"])),
                        "sealedInputRef": sealed_ref, "sourceManifest": {}, "metrics": {}}

            preparation.seal_source.side_effect = seal
            preparation.materialize_source.side_effect = lambda *_args, **options: {
                "capturedSourceDigest": before["sourceDigest"], "sealedInputRef": sealed_ref,
                "sourceDigest": sealed_ref, "physicalPath": str(options["source_path"])}
            preparation.verify_source.return_value = {"sourceDigest": sealed_ref}
            preparation.trusted_toolchain_facts.return_value = {"binaries": {
                tool: {"path": trusted[tool + "Path"], "sha256": checksum} for tool in ("cargo", "rustc", "rustdoc")}}
            helper = Mock()
            helper.free_bytes.return_value = 1024**4
            helper.prepare_command.side_effect = local_cargo.prepare_command
            documents, native_calls = {}, []

            def write_document(path, value):
                documents[str(path)] = copy.deepcopy(value)
                return {"path": str(path), "sha256": runner.sha(runner.encoded(value))}

            def native(command, cwd, environment, report, timeout):
                if route == "validate_checkout":
                    self.assertEqual(guard.assert_stable.call_count, 2)
                native_calls.append((list(command), str(cwd), dict(environment), str(report), timeout))
                raise NativeReached()

            helper._run_owned_command.side_effect = native
            loader = Mock()
            module_spec = SimpleNamespace(loader=loader)
            approved = SimpleNamespace(path=Path(build_root), namespace=lambda: output)
            with ExitStack() as stack:
                for target, replacement in [
                    ("tools.jenkins.resources.capacity.ResourceManager", Mock(return_value=resources)),
                    ("tools.jenkins.resources.paths.canonical_build_root", Mock(return_value=approved)),
                    ("tools.jenkins.resources.paths.physical_path_under", lambda _approved, path: Path(path)),
                    ("tools.jenkins.workflow.planning._environment_policy",
                     Mock(return_value=({"PATH": "trusted", "ZIRCON_DOCS_HISTORY_ARCHIVE_ROOT": "ambient"}, trusted, ROOT))),
                    ("tools.jenkins.state.locks.process_lock", lambda *_args, **_options: nullcontext()),
                    ("tools.jenkins.workflow.pools._hold", Mock()),
                ]:
                    stack.enter_context(patch(target, replacement))
                stack.enter_context(patch.object(local_cargo, "validate_output_path", side_effect=lambda value: Path(value)))
                stack.enter_context(patch.object(runner, "source_helper", return_value=preparation))
                stack.enter_context(patch.object(runner, "checkout_guard_helper", return_value=Mock()))
                stack.enter_context(patch.object(runner, "write_json", side_effect=write_document))
                stack.enter_context(patch.object(runner.importlib.util, "spec_from_file_location", return_value=module_spec))
                stack.enter_context(patch.object(runner.importlib.util, "module_from_spec", return_value=helper))
                stack.enter_context(patch.object(Path, "mkdir"))
                stack.enter_context(patch.object(Path, "exists", return_value=False))
                stack.enter_context(patch.object(Path, "read_bytes", return_value=tool_bytes))
                with self.assertRaises(NativeReached):
                    if route == "validate_checkout":
                        runner.Management.validate_checkout(actor, batch=batch)
                    else:
                        getattr(runner.Management, route)(actor)
            self.assertEqual(len(native_calls), 1, route)
            command, cwd, environment, _report, _timeout = native_calls[0]
            self.assertEqual(command[:2], [trusted["cargoPath"], "test"])
            expected_target = str(output / "pools" / "archive-pool" / "target")
            helper.prepare_command.assert_called_once()
            prepared_args, prepared_kwargs = helper.prepare_command.call_args
            self.assertEqual(str(prepared_args[1]), expected_target)
            self.assertIs(prepared_kwargs["locked"], True)
            self.assertEqual(command.count("--target-dir"), 1)
            target_option = command.index("--target-dir")
            self.assertEqual(command[target_option + 1], expected_target)
            self.assertEqual(environment["CARGO_TARGET_DIR"], expected_target)
            separator = command.index("--")
            self.assertLess(target_option + 1, separator)
            self.assertEqual(command[:separator].count("--locked"), 1)
            self.assertNotIn("--locked", command[separator + 1:])
            if route in ("validate_batch", "validate_checkout"):
                self.assertEqual(command[separator + 1:], ["--exact", "history-route-case"])
            if route == "validate_checkout":
                self.assertEqual(environment["RUSTDOC"], trusted["rustdocPath"])
                self.assertTrue(actor.state.get("execution", owner)["payload"]["batch"])
                self.assertEqual(actor._checkout_context["target"], Path(expected_target))
                actor.batch_pool.assert_called_once()
                self.assertEqual(guard.assert_stable.call_count, 2)
            parent = next(value for value in documents.values() if "executionIdentity" in value)
            environment_digest = runner.sha(runner.encoded(environment))
            self.assertEqual(environment["ZIRCON_DOCS_HISTORY_ARCHIVE_ROOT"], str(archive.resolve()))
            self.assertEqual(parent["environmentDigest"], environment_digest)
            self.assertEqual(parent["command"], command)
            self.assertEqual(parent["executionRoot"], cwd)
            self.assertEqual(parent["payload"]["fixedHistoryArchiveInput"]["root"], str(archive))
            if route == "validate_prepared":
                identity = {"sourceDigest": sealed_ref, "command": command, "cwd": cwd,
                            "environmentDigest": environment_digest,
                            "coverage": {key: value for key, value in recipe.items() if key != "command"},
                            "driverDigest": actor.config["driverDigest"]}
            else:
                identity = {"sourceDigest": before["sourceDigest"], "commands": [command], "cwd": cwd,
                            "environmentDigests": [environment_digest], "coverage": parent["coverage"],
                            "driverDigest": actor.config["driverDigest"],
                            "runtimeOperationId": actor.config["runtimeOperationId"]}
                self.assertEqual(parent["stepEnvironmentDigests"], [environment_digest])
                if route == "validate_checkout":
                    identity.update(validationInputPolicy="actual-checkout", checkoutInputsDigest=parent["checkoutInputsDigest"])
            self.assertEqual(parent["executionIdentity"], runner.sha(runner.encoded(identity)))
            return parent["executionIdentity"]

        for route in ("validate_prepared", "validate_batch", "validate_checkout"):
            with self.subTest(route=route):
                self.assertNotEqual(exercise(route, ROOT), exercise(route, ROOT / "tools"))


if __name__ == "__main__":
    unittest.main()
