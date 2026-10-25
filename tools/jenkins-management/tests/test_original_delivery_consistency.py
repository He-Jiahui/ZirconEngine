"""Reconcile one original delivery without replaying an unclosed native run."""
from __future__ import annotations

from contextlib import contextmanager
from copy import deepcopy
import importlib.util
from pathlib import Path
import tempfile
from types import SimpleNamespace
from unittest.mock import Mock, patch

import pytest


ROOT = Path(__file__).resolve().parents[3]
SPEC = importlib.util.spec_from_file_location(
    "original_delivery_consistency_entry", ROOT / "tools/jenkins-management/entry.py")
entry = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(entry)
ABSENT = object()


@contextmanager
def original_case(*, state_flag=False, delivery_flag=ABSENT, state=True, phase="running"):
    with tempfile.TemporaryDirectory(prefix="original-delivery-consistency-") as temporary:
        repo = Path(temporary)
        payload = {
            "action": "validate", "repositoryId": "repository", "sessionId": "material-owner",
            "requestId": "original-material-request", "operationId": "original-single-recipe",
            "driverDigest": "d" * 64, "runtimeOperationId": "original-runtime",
            "ownedPaths": ["zircon_plugins/material_editor/editor/src/lib.rs"],
            "sourceView": "checkout", "verb": "test",
            "manifestPath": "E:/Git/ZirconEngine/zircon_plugins/Cargo.toml",
            "packages": ["zircon_plugin_material_editor_editor"], "features": [],
            "noDefaultFeatures": True, "lib": True, "profile": "test", "jobs": 1,
            "testFilter": "material_graph_", "testThreads": 1, "locked": True,
            "buildRoot": "E:/cargo-targets", "expectedSourceDigest": "e" * 64,
            "sourceHashes": {"zircon_plugins/material_editor/editor/src/lib.rs": "a" * 64},
            "dependencyHashes": {"zircon_plugins/Cargo.lock": "b" * 64},
        }
        delivered, state_input = deepcopy(payload), deepcopy(payload)
        if delivery_flag is not ABSENT:
            delivered["observationOnly"] = delivery_flag
        if state_flag is not ABSENT:
            state_input["observationOnly"] = state_flag
        names = ("repositoryId", "sessionId", "requestId", "operationId")
        key = entry.sha(entry.encoded({name: payload[name] for name in names}))
        digest = entry.sha(entry.encoded(payload))
        operation = {"key": key, "version": 1, "payload": {
            "phase": phase, "inputDigest": digest, "payload": state_input,
            "result": None if phase != "complete" else {"status": "failed", "reasonCode": "fixture-terminal"},
        }}
        parameters = {
            "SESSION_ID": delivered["sessionId"], "REQUEST_ID": delivered["requestId"],
            "OPERATION_ID": delivered["operationId"], "ACTION": delivered["action"],
            "INPUT_JSON": entry.encoded(delivered).decode(),
            "EXPECTED_DRIVER_DIGEST": delivered["driverDigest"],
            "RUNTIME_OPERATION_ID": delivered["runtimeOperationId"], "QUERY_ONLY": "false",
        }
        marker_key = entry.sha("|".join(payload[name] for name in
                               ("sessionId", "requestId", "operationId")).encode())
        marker = repo / ".jenkins/state/management/submissions" / (marker_key + ".json")
        marker.parent.mkdir(parents=True)
        saved = {**{name: delivered[name] for name in (*names, "driverDigest", "runtimeOperationId")},
            "inputDigest": entry.sha(entry.encoded(delivered)), "payload": delivered,
            "bodyDigest": entry.sha(entry.urllib.parse.urlencode(parameters).encode("ascii")),
            "phase": "submitted", "queueUrl": "http://fixture/queue/item/15149/",
            "formalAcceptance": False, "m8Accepted": False}
        marker.write_bytes(entry.encoded(saved) + b"\n")
        manager = Mock(spec=[])
        context = {"repo": repo, "manager": manager, "publicTransport": True}
        def record(_repo, domain, actual_key):
            assert _repo == repo and domain == "management_operation" and actual_key == key
            return operation if state else None
        case = SimpleNamespace(repo=repo, payload=payload, delivered=delivered,
            operation=operation, marker=marker, saved=saved, manager=manager,
            context=context, digest=digest)
        with patch.object(entry, "record", side_effect=record), \
             patch.object(entry, "_replay_route", side_effect=lambda _context, value: (deepcopy(value), None)), \
             patch.object(entry, "source_digest", side_effect=AssertionError("Original input must not be recaptured")):
            yield case


def witness(case):
    return (deepcopy(case.operation), case.marker.read_bytes(), deepcopy(case.payload),
            sorted(str(path.relative_to(case.repo)) for path in case.repo.rglob("*")))


@pytest.mark.parametrize("state_flag,delivery_flag", [
    (ABSENT, ABSENT), (False, ABSENT), (ABSENT, False), (False, False),
])
def test_false_observation_annotation_preserves_original_pending_delivery(state_flag, delivery_flag):
    with original_case(state_flag=state_flag, delivery_flag=delivery_flag) as case:
        before = witness(case)
        result = entry.submit_payload(case.context, deepcopy(case.payload))
        assert result["status"] == "pending" and result["phase"] == "running"
        assert result["inputDigest"] == case.digest and result["result"] is None
        assert result["resubmitted"] is False and result["formalAcceptance"] is False
        assert witness(case) == before and not case.manager.mock_calls


@pytest.mark.parametrize("location", ["state", "delivery", "candidate"])
@pytest.mark.parametrize("invalid", [True, None, 0, "false"])
def test_other_observation_values_cannot_be_dropped(location, invalid):
    options = {"state_flag": invalid} if location == "state" else {"delivery_flag": invalid} if location == "delivery" else {}
    with original_case(**options) as case:
        incoming = deepcopy(case.payload)
        if location == "candidate":
            incoming["observationOnly"] = invalid
        before = witness(case)
        with pytest.raises(entry.EntryError):
            entry.submit_payload(case.context, incoming)
        assert witness(case) == before and not case.manager.mock_calls


@pytest.mark.parametrize("location", ["state", "delivery", "candidate"])
def test_unknown_execution_fields_are_not_treated_as_read_annotations(location):
    with original_case() as case:
        incoming = deepcopy(case.payload)
        if location == "state":
            case.operation["payload"]["payload"]["futureRecipeOption"] = {"enabled": True}
        elif location == "candidate":
            incoming["futureRecipeOption"] = {"enabled": True}
        else:
            # An internally valid synthetic POST must still fail semantic comparison.
            # This rewrites only the test's temporary fixture, never a real delivery.
            delivered = {**case.delivered, "futureRecipeOption": {"enabled": True}}
            parameters = {
                "SESSION_ID": delivered["sessionId"], "REQUEST_ID": delivered["requestId"],
                "OPERATION_ID": delivered["operationId"], "ACTION": delivered["action"],
                "INPUT_JSON": entry.encoded(delivered).decode(),
                "EXPECTED_DRIVER_DIGEST": delivered["driverDigest"],
                "RUNTIME_OPERATION_ID": delivered["runtimeOperationId"], "QUERY_ONLY": "false",
            }
            case.marker.write_bytes(entry.encoded({
                **case.saved, "payload": delivered,
                "inputDigest": entry.sha(entry.encoded(delivered)),
                "bodyDigest": entry.sha(entry.urllib.parse.urlencode(parameters).encode("ascii")),
            }))
        before = witness(case)
        with pytest.raises(entry.EntryError):
            entry.submit_payload(case.context, incoming)
        assert witness(case) == before and not case.manager.mock_calls


@pytest.mark.parametrize("field,changed", [
    ("manifestPath", "E:/Git/ZirconEngine/Cargo.toml"),
    ("packages", ["zircon_editor"]), ("features", ["extra"]), ("testFilter", "foreign_"),
    ("profile", "release"), ("jobs", 2), ("ownedPaths", ["foreign.rs"]),
    ("sourceHashes", {"zircon_plugins/material_editor/editor/src/lib.rs": "c" * 64}),
    ("dependencyHashes", {"zircon_plugins/Cargo.lock": "c" * 64}),
    ("driverDigest", "f" * 64), ("runtimeOperationId", "different-runtime"),
])
def test_true_execution_differences_still_reject(field, changed):
    with original_case() as case:
        case.operation["payload"]["payload"][field] = changed
        before = witness(case)
        with pytest.raises(entry.EntryError, match="Original State and delivery payloads differ"):
            entry.submit_payload(case.context, deepcopy(case.payload))
        assert witness(case) == before and not case.manager.mock_calls


@pytest.mark.parametrize("field,changed", [
    ("bodyDigest", "f" * 64), ("inputDigest", "f" * 64),
    ("phase", "unknown"), ("formalAcceptance", True), ("m8Accepted", True),
    ("runtimeOperationId", "different-runtime"),
])
def test_exact_original_post_and_envelope_still_required(field, changed):
    with original_case() as case:
        case.marker.write_bytes(entry.encoded({**case.saved, field: changed}))
        before = witness(case)
        with pytest.raises(entry.EntryError):
            entry.submit_payload(case.context, deepcopy(case.payload))
        assert witness(case) == before and not case.manager.mock_calls


def test_state_digest_must_match_semantic_execution_input():
    with original_case() as case:
        case.operation["payload"]["inputDigest"] = "f" * 64
        before = witness(case)
        with pytest.raises(entry.EntryError, match="Original operation input differs"):
            entry.submit_payload(case.context, deepcopy(case.payload))
        assert witness(case) == before and not case.manager.mock_calls


@pytest.mark.parametrize("options", [{}, {"dry_run": True}, {"query_only": True}])
def test_existing_running_recipe_never_reenters_source_capture_or_post(options):
    with original_case() as case:
        before = witness(case)
        assert entry.submit_payload(case.context, deepcopy(case.payload), **options)["resubmitted"] is False
        assert witness(case) == before and not case.manager.mock_calls


def test_unclosed_single_recipe_does_not_gain_batch_resume_authority():
    with original_case() as case:
        before = witness(case)
        with pytest.raises(entry.EntryError, match="Only a durably checkpointed batch"):
            entry.submit_payload(case.context, deepcopy(case.payload), resume_original=True)
        assert witness(case) == before and not case.manager.mock_calls


def test_terminal_operation_is_only_observed_even_when_resume_was_requested():
    with original_case(phase="complete") as case:
        before = witness(case)
        result = entry.submit_payload(case.context, deepcopy(case.payload), resume_original=True)
        assert result["status"] == "observed" and result["phase"] == "complete"
        assert result["result"] == case.operation["payload"]["result"]
        assert result["resubmitted"] is False and witness(case) == before and not case.manager.mock_calls


def test_delivery_without_state_remains_reconciliation_only():
    with original_case(state=False) as case:
        before = witness(case)
        result = entry.submit_payload(case.context, deepcopy(case.payload))
        assert result["reasonCode"] == "original_delivery_requires_reconciliation"
        assert result["resubmitted"] is False and witness(case) == before and not case.manager.mock_calls
