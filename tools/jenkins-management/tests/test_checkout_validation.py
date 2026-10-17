"""Focused proof and dispatch tests for actual-checkout validation."""
from __future__ import annotations

import copy
from contextlib import nullcontext
import importlib.util
from pathlib import Path
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
            "module": SimpleNamespace(Management=self.TinyManagement),
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


if __name__ == "__main__":
    unittest.main()
