"""Exercise public and Job-level automatic managed context routing."""
from __future__ import annotations

from contextlib import contextmanager, closing, redirect_stdout
import importlib.util
import io
import json
import sqlite3
import tempfile
import os
from pathlib import Path
from types import SimpleNamespace
import unittest
from unittest.mock import Mock, patch

ROOT = Path(__file__).resolve().parents[3]


def load(name, filename):
    spec = importlib.util.spec_from_file_location(name, ROOT / "tools/jenkins-management" / filename)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


entry = load("managed_context_public_entry_test", "entry.py")
runner = load("managed_context_job_entry_test", "runner.py")
historical_helper = load("historical_query_source_helper_test", "source_preparation.py")


class ManagedContextEntryTests(unittest.TestCase):
    def setUp(self):
        self.repo = Path("E:/managed-context-memory").absolute()
        self.config = {"driverDigest": "a" * 64, "runtimeOperationId": "runtime",
                       "managedWorktree": entry.MANAGED_WORKTREE_CAPABILITY}
        self.original = {"action": "prepare-worktree", "repositoryId": entry.sha(str(self.repo).casefold().encode()),
                         "sessionId": "writer-chat", "requestId": "request", "operationId": "prepare",
                         "ownedPaths": ["src/lib.rs"], **{k: self.config[k] for k in ("driverDigest", "runtimeOperationId")}}
        self.routed = {**self.original, "sessionId": "managed-reader"}
        self.route = {"originIdentity": {"sessionId": "writer-chat"},
                      "routedIdentity": {"sessionId": "managed-reader"},
                      "writerSessionId": "writer-chat", "readerSessionId": "managed-reader"}
        self.module = SimpleNamespace(route_request_context=Mock(return_value=(self.routed, self.route)),
                                      ensure_request_context=Mock())
        self.context = {"repo": self.repo, "config": self.config, "manager": object(), "module": self.module}
        self.environment = patch.dict(os.environ, {"ZIRCON_JENKINS_SESSION_ID": "writer-chat"})
        self.environment.start()
        self.addCleanup(self.environment.stop)

    def test_original_arguments_and_current_chat_are_checked_before_routing(self):
        with self.assertRaises(entry.EntryError):
            entry.input_payload(self.context, self.original, identity={"sessionId": "another-chat"})
        with patch.dict(os.environ, {"ZIRCON_JENKINS_SESSION_ID": "another-chat"}):
            with self.assertRaises(entry.EntryError):
                entry.input_payload(self.context, self.original)
        self.module.route_request_context.assert_not_called()

    def test_public_entry_routes_before_authorize_without_changing_original_input(self):
        routing = {}
        with patch.object(entry, "authorize") as authorize:
            value = entry.input_payload(self.context, self.original, routing=routing)
        self.assertEqual(value["sessionId"], "managed-reader")
        self.assertEqual(self.original["sessionId"], "writer-chat")
        self.assertEqual(routing, self.route)
        self.assertNotIn("managedContext", value)
        authorize.assert_called_once_with(self.repo, self.routed, context=self.context,
                                         automatic=True, origin_session="writer-chat")

    def test_query_uses_read_only_route_and_submit_response_keeps_receipt_out_of_payload(self):
        with patch.object(entry, "authorize"):
            entry.input_payload(self.context, self.original, automatic_context=False)
        self.assertTrue(self.module.route_request_context.call_args.kwargs["query_only"])
        with patch.object(entry, "capability", return_value=self.context):
            with patch.object(entry, "authorize"):
                with patch.object(entry, "submit_payload", return_value={"status": "submitted"}) as submit:
                    result = entry.dispatch(self.repo, self.original)
        self.assertEqual(result["managedContext"], self.route)
        self.assertNotIn("managedContext", submit.call_args.args[1])
        self.assertEqual(submit.call_args.args[1]["sessionId"], "managed-reader")

    def row(self, kind, actions, principal):
        evidence = {"source": "automatic-local-request", "policy": "local-validation-v1", "kind": kind,
                    "repositoryId": self.original["repositoryId"], "repositoryRoot": str(self.repo),
                    "ownerPrincipal": "actual-windows-owner", "writerPrincipal": principal, "inputDigest": "b" * 64}
        snapshot = {"repositoryId": self.original["repositoryId"], "sessionId": "managed-reader",
                    "principal": principal, "paths": ["src/lib.rs"], "actions": actions, "evidence": evidence}
        row = {"status": "active", "principal": principal, "paths": json.dumps(snapshot["paths"]),
               "actions": json.dumps(actions), "evidence": json.dumps(evidence),
               "snapshot_digest": entry.sha(entry.encoded(snapshot))}

        @contextmanager
        def connection(repo):
            yield SimpleNamespace(execute=lambda *args: SimpleNamespace(fetchone=lambda: row))

        return connection

    def test_reader_retains_legacy_writer_principal_and_verifies_actual_runtime_owner(self):
        helper = SimpleNamespace(local_request_principal=Mock(return_value="actual-windows-owner"))
        self.module.source_helper = lambda: helper
        connection = self.row("managed-worktree-reader", ["validation"], "legacy-writer-label")
        with patch.object(entry, "connection", side_effect=connection):
            entry.authorize(self.repo, self.routed, context=self.context, origin_session="writer-chat")
            helper.local_request_principal.return_value = "different-windows-owner"
            with self.assertRaises(entry.EntryError):
                entry.authorize(self.repo, self.routed, context=self.context, origin_session="writer-chat")

    def test_automatic_exact_writer_is_accepted_without_a_manual_user_grant(self):
        connection = self.row("managed-worktree-writer", ["implementation", "validation"], "actual-windows-owner")
        with patch.object(entry, "connection", side_effect=connection):
            entry.authorize(self.repo, {**self.routed, "action": "inspect-worktree"},
                            context=self.context, origin_session="writer-chat")


class JobManagedContextTests(unittest.TestCase):
    def test_ordinary_validation_never_creates_managed_routing(self):
        payload = {"action": "validate", "sessionId": "ordinary"}
        with patch.object(runner, "source_helper") as helper:
            routed, receipt = runner.route_request_context({}, ROOT, object(), payload)
        self.assertEqual(routed, payload)
        self.assertIsNot(routed, payload)
        self.assertIsNone(receipt)
        helper.assert_not_called()

    def test_managed_query_delegates_as_read_only(self):
        manager, state, payload = object(), object(), {"action": "prepare-flow"}
        helper = SimpleNamespace(route_managed_context=Mock(return_value=(payload, {"readerSessionId": "reader"})))
        with patch.object(runner, "source_helper", return_value=helper):
            runner.route_request_context({}, ROOT, manager, payload, state=state, query_only=True)
        helper.route_managed_context.assert_called_once_with({}, ROOT, manager, payload,
                                                            state=state, query_only=True)

    def test_direct_job_routes_before_constructing_actor_and_returns_route_receipt(self):
        payload = {"action": "prepare-worktree", "sessionId": "writer"}
        routed, receipt = {**payload, "sessionId": "reader"}, {"readerSessionId": "reader"}
        args = SimpleNamespace(capsule=ROOT / ".jenkins/runtime/management/test",
                               input_file=ROOT / ".jenkins/state/controls/test.input.json",
                               output_file=ROOT / ".jenkins/state/controls/test.output.json")
        manager, config, events = object(), {"driverDigest": "a" * 64}, []
        def route(*args, **kwargs):
            events.append("route")
            return routed, receipt
        def actor(*args):
            events.append("construct")
            self.assertIs(args[-1], routed)
            return SimpleNamespace(run=lambda: {"status": "complete"})
        with patch.object(runner.argparse.ArgumentParser, "parse_args", return_value=args):
            with patch.object(runner, "boot", return_value=(config, ROOT, manager)):
                with patch.object(runner, "plain", side_effect=lambda path: path):
                    with patch.object(Path, "stat", return_value=SimpleNamespace(st_size=128)):
                        with patch.object(Path, "read_bytes", return_value=json.dumps(payload).encode()):
                            with patch.object(Path, "write_bytes") as output:
                                with patch.object(runner, "route_request_context", side_effect=route):
                                    with patch.object(runner, "Management", side_effect=actor):
                                        result = runner.main()
        self.assertEqual(result, 0)
        self.assertEqual(events, ["route", "construct"])
        written = json.loads(output.call_args.args[0])
        self.assertEqual(written["managedContext"], receipt)



class HistoricalQueryTests(unittest.TestCase):
    @contextmanager
    def case(self, *, cargo=False, routed=False, operation=True, marker=True):
        with tempfile.TemporaryDirectory(prefix="jenkins-historical-query-") as directory:
            repo = Path(directory).absolute()
            state_root = repo / ".jenkins/state"
            state_root.mkdir(parents=True)
            database = state_root / "coordination.sqlite3"
            with closing(sqlite3.connect(database)) as current:
                current.executescript("""
                    CREATE TABLE records(domain TEXT,key TEXT,payload TEXT,version INTEGER,updated_at REAL,
                                         PRIMARY KEY(domain,key));
                    CREATE TABLE authorizations(repository_id TEXT,session_id TEXT,principal TEXT,paths TEXT,
                        actions TEXT,evidence TEXT,snapshot_digest TEXT,status TEXT,created_at REAL,
                        PRIMARY KEY(repository_id,session_id));
                """)
            repository = entry.sha(str(repo).casefold().encode())
            def grant(session):
                evidence = {"source": "user", "task": "immutable-historical-query-fixture"}
                snapshot = {"repositoryId": repository, "sessionId": session, "principal": "legacy-owner",
                            "paths": ["src"], "actions": ["validation"], "evidence": evidence}
                checksum = entry.sha(entry.encoded(snapshot))
                with closing(sqlite3.connect(database)) as current:
                    current.execute("INSERT INTO authorizations VALUES (?,?,?,?,?,?,?,?,?)",
                        (repository, session, "legacy-owner", json.dumps(snapshot["paths"]),
                         json.dumps(snapshot["actions"]), json.dumps(evidence), checksum, "active", 1))
                    current.commit()
                return checksum
            def put(domain, key, value):
                with closing(sqlite3.connect(database)) as current:
                    current.execute("INSERT OR REPLACE INTO records VALUES (?,?,?,?,?)",
                                    (domain, key, json.dumps(value), 2, 1))
                    current.commit()
            incoming = {"repositoryId": repository, "sessionId": "original-chat", "requestId": "original-request",
                        "operationId": "original-operation", "action": "validate" if cargo else "inspect-worktree",
                        "driverDigest": "d" * 64, "runtimeOperationId": "original-runtime",
                        "ownedPaths": ["src/lib.rs"], "sourceView": "worktree", "worktreeId": "existing-wt"}
            arguments = ["test", "-p", "zircon_app", "--lib", "--offline", "--",
                         "runtime_session_create_cleanup", "--test-threads=1"]
            recipe = entry.parse_cargo(arguments, enforce_policy=False) if cargo else None
            if cargo:
                incoming.update(recipe, expectedSourceDigest="e" * 64, buildRoot="E:\\cargo-targets",
                                requestedTargetDir="E:\\cargo-targets\\historical-query-fixture")
            origin_snapshot = grant(incoming["sessionId"])
            names = ("repositoryId", "sessionId", "requestId", "operationId")
            identity = lambda value: {name: value[name] for name in names}
            delivered, route = dict(incoming), None
            if routed:
                delivered["sessionId"] = "technical-writer"
                writer_snapshot = grant(delivered["sessionId"])
                route_ref = entry.sha(entry.encoded(identity(incoming)))
                route = {"schemaVersion": 1, "routeRef": route_ref, "source": "automatic-local-request",
                    "policy": "local-validation-v1", "action": incoming["action"],
                    "originIdentity": identity(incoming), "routedIdentity": identity(delivered),
                    "originInputDigest": entry.sha(entry.encoded(incoming)),
                    "routedInputDigest": entry.sha(entry.encoded(delivered)),
                    "routeFields": {"sessionId": delivered["sessionId"]}, "repositoryRoot": str(repo),
                    "driverDigest": incoming["driverDigest"], "runtimeOperationId": incoming["runtimeOperationId"],
                    "originSnapshot": origin_snapshot, "writerSessionId": delivered["sessionId"],
                    "writerAuthorizationSnapshot": writer_snapshot, "readerSessionId": None,
                    "readerAuthorizationSnapshot": None}
                put("management_context_route", route_ref, route)
                put("management_context_route_alias", entry.sha(entry.encoded(identity(delivered))),
                    {"routeRef": route_ref})
            key = entry.sha(entry.encoded(identity(delivered)))
            operation_payload = {"action": delivered["action"], "sessionId": delivered["sessionId"],
                "requestId": delivered["requestId"], "operationId": delivered["operationId"],
                "phase": "unknown", "inputDigest": entry.sha(entry.encoded(delivered)),
                "payload": {**delivered, "observationOnly": False},
                "result": {"status": "unknown", "reasonCode": "management_operation_interrupted",
                           "operationKey": key, "runtimeOperationId": delivered["runtimeOperationId"]}}
            if operation:
                put("management_operation", key, operation_payload)
            marker_key = entry.sha("|".join(delivered[name] for name in ("sessionId", "requestId", "operationId")).encode())
            marker_path = state_root / "management/submissions" / (marker_key + ".json")
            marker_path.parent.mkdir(parents=True)
            parameters = {"SESSION_ID": delivered["sessionId"], "REQUEST_ID": delivered["requestId"],
                "OPERATION_ID": delivered["operationId"], "ACTION": delivered["action"],
                "INPUT_JSON": entry.encoded(delivered).decode(), "EXPECTED_DRIVER_DIGEST": delivered["driverDigest"],
                "RUNTIME_OPERATION_ID": delivered["runtimeOperationId"], "QUERY_ONLY": "false"}
            saved = {**{name: delivered[name] for name in (*names, "driverDigest", "runtimeOperationId")},
                "inputDigest": entry.sha(entry.encoded(delivered)), "payload": delivered,
                "bodyDigest": entry.sha(entry.urllib.parse.urlencode(parameters).encode("ascii")),
                "phase": "submitted", "queueUrl": "http://fixture/queue/item/7/",
                "formalAcceptance": False, "m8Accepted": False}
            if marker:
                marker_path.write_bytes(entry.encoded(saved) + b"\n")
            input_path = state_root / "original-input.json"
            input_path.write_bytes(entry.encoded(incoming))
            module = SimpleNamespace(source_helper=lambda: historical_helper,
                route_request_context=Mock(side_effect=AssertionError("Query must not route/create context")),
                ensure_request_context=Mock(side_effect=AssertionError("Query must not admit context")))
            context = {"repo": repo, "config": {"driverDigest": "f" * 64,
                "runtimeOperationId": "current-runtime"}, "module": module,
                "manager": Mock(spec=[])}
            yield SimpleNamespace(repo=repo, database=database, incoming=incoming, delivered=delivered,
                key=key, marker=marker_path, input=input_path, saved=saved, state=operation_payload,
                context=context, route=route, put=put, recipe=recipe, arguments=arguments)

    def witness(self, case):
        return (entry.sha(case.database.read_bytes()),
                entry.sha(case.marker.read_bytes()) if case.marker.exists() else None,
                sorted(str(path.relative_to(case.repo)) for path in case.repo.rglob("*")))

    @contextmanager
    def read_only(self, case):
        original_connect = sqlite3.connect
        statements, opens = [], []
        def connect(path, *args, **kwargs):
            self.assertTrue(path.endswith("?mode=ro"))
            self.assertTrue(kwargs.get("uri"))
            opens.append(path)
            value = original_connect(path, *args, **kwargs)
            value.set_trace_callback(statements.append)
            return value
        with patch.object(entry, "capability", return_value=case.context) as capability:
            with patch.object(entry.sqlite3, "connect", side_effect=connect):
                with patch.object(entry, "input_payload", side_effect=AssertionError("Query normalized as new input")):
                    with patch.object(entry, "submit_payload", side_effect=AssertionError("Query entered delivery path")):
                        with patch.object(entry, "output_policy", side_effect=AssertionError("Query used current output policy")):
                            with patch.object(entry, "source_digest", side_effect=AssertionError("Query captured current source")):
                                yield statements, opens
            capability.assert_called()
        case.context["module"].route_request_context.assert_not_called()
        case.context["module"].ensure_request_context.assert_not_called()

    def test_historical_main_observes_unknown_with_original_generation_and_no_mutation(self):
        with self.case() as case, patch.dict(os.environ, {"ZIRCON_JENKINS_SESSION_ID": "original-chat"}):
            before = self.witness(case)
            output = io.StringIO()
            with self.read_only(case) as (statements, opens), redirect_stdout(output):
                code = entry.main(["submit", "--repo-root", str(case.repo), "--input-file", str(case.input),
                    "--session-id", "original-chat", "--request-id", "original-request",
                    "--operation-id", "original-operation", "--action", "inspect-worktree", "--query-only"])
            result = json.loads(output.getvalue())
            self.assertEqual(2, code)
            self.assertEqual(("pending", "unknown", False), (result["status"], result["phase"], result["resubmitted"]))
            self.assertEqual(case.delivered["driverDigest"], result["driverDigest"])
            self.assertEqual(case.delivered["runtimeOperationId"], result["runtimeOperationId"])
            self.assertEqual(case.state["result"], result["result"])
            self.assertEqual(before, self.witness(case))
            self.assertEqual(1, len(opens))
            self.assertTrue(any("FROM authorizations" in sql for sql in statements))
            self.assertTrue(any("FROM records" in sql for sql in statements))
            self.assertFalse(any(sql.split()[0].upper() in {"INSERT", "UPDATE", "DELETE", "CREATE"} for sql in statements))

    def test_historical_query_rejects_identity_authority_state_and_marker_tampering(self):
        for defect in ("cli", "environment", "repository", "grant", "snapshot", "input", "state", "body", "envelope"):
            with self.subTest(defect=defect), self.case() as case:
                payload = dict(case.incoming)
                identity = {"sessionId": "different-chat"} if defect == "cli" else None
                environment = "different-chat" if defect == "environment" else "original-chat"
                if defect == "repository":
                    payload["repositoryId"] = "0" * 32
                if defect in {"grant", "snapshot"}:
                    with closing(sqlite3.connect(case.database)) as current:
                        current.execute("UPDATE authorizations SET " + ("status='revoked'" if defect == "grant"
                                        else "snapshot_digest=?"),
                                        () if defect == "grant" else ("0" * 64,))
                        current.commit()
                if defect == "input":
                    payload["worktreeId"] = "foreign-wt"
                if defect == "state":
                    changed = {**case.state, "payload": {**case.state["payload"], "worktreeId": "foreign-wt"}}
                    case.put("management_operation", case.key, changed)
                if defect in {"body", "envelope"}:
                    changed = {**case.saved, **({"bodyDigest": "0" * 64} if defect == "body"
                                               else {"runtimeOperationId": "foreign-runtime"})}
                    case.marker.write_bytes(entry.encoded(changed))
                before = self.witness(case)
                with patch.dict(os.environ, {"ZIRCON_JENKINS_SESSION_ID": environment}), self.read_only(case):
                    with self.assertRaises(entry.EntryError):
                        entry.dispatch(case.repo, payload, identity=identity, query_only=True)
                self.assertEqual(before, self.witness(case))

    def test_historical_query_replays_only_existing_routes_and_cargo_delivery_defaults(self):
        for alias in (False, True):
            with self.subTest(alias=alias), self.case(routed=True) as case:
                payload = case.delivered if alias else case.incoming
                before = self.witness(case)
                with patch.dict(os.environ, {"ZIRCON_JENKINS_SESSION_ID": payload["sessionId"]}), self.read_only(case):
                    result = entry.dispatch(case.repo, payload, query_only=True)
                self.assertEqual(case.route, result["managedContext"])
                self.assertEqual("technical-writer", result["sessionId"])
                self.assertEqual(before, self.witness(case))
        for field, changed in (("paths", ["foreign"]), ("actions", ["implementation"])):
            with self.subTest(origin_grant=field), self.case(routed=True) as case:
                with closing(sqlite3.connect(case.database)) as current:
                    current.execute("UPDATE authorizations SET " + field + "=? WHERE session_id=?",
                                    (json.dumps(changed), case.incoming["sessionId"]))
                    self.assertEqual(case.route["originSnapshot"], current.execute(
                        "SELECT snapshot_digest FROM authorizations WHERE session_id=?",
                        (case.incoming["sessionId"],)).fetchone()[0])
                    current.commit()
                before = self.witness(case)
                with patch.dict(os.environ, {"ZIRCON_JENKINS_SESSION_ID": case.delivered["sessionId"]}):
                    with self.read_only(case), self.assertRaisesRegex(
                            entry.EntryError, "Original managed route authorization changed"):
                        entry.dispatch(case.repo, case.delivered, query_only=True)
                self.assertEqual(before, self.witness(case))
        with self.case(cargo=True) as case, patch.dict(os.environ, {"ZIRCON_JENKINS_SESSION_ID": "original-chat"}):
            sparse = {name: value for name, value in case.incoming.items()
                      if name not in entry.RECIPE_FIELDS | {"expectedSourceDigest"}}
            case.input.write_bytes(entry.encoded(sparse))
            before = self.witness(case)
            with self.read_only(case):
                result = entry.cargo_entry(case.repo, case.arguments, identity_file=case.input,
                    target_dir=case.delivered["requestedTargetDir"], build_root=case.delivered["buildRoot"],
                    query_only=True)
                with self.assertRaises(entry.EntryError):
                    entry.dispatch(case.repo, sparse, recipe={**case.recipe, "packages": ["foreign-package"]},
                                   query_only=True)
                with self.assertRaises(entry.EntryError):
                    entry.dispatch(case.repo, sparse, target_dir="E:\\cargo-targets\\foreign", query_only=True)
            self.assertEqual(case.state["inputDigest"], result["inputDigest"])
            self.assertEqual(before, self.witness(case))

    def test_historical_lookup_preserves_missing_delivery_and_current_execution_boundaries(self):
        for operation, marker in ((True, False), (False, True), (False, False)):
            with self.subTest(operation=operation, marker=marker), self.case(operation=operation, marker=marker) as case:
                before = self.witness(case)
                with patch.dict(os.environ, {"ZIRCON_JENKINS_SESSION_ID": "original-chat"}), self.read_only(case):
                    result = entry.dispatch(case.repo, case.incoming, query_only=True)
                self.assertFalse(result["resubmitted"])
                self.assertEqual("pending", result["status"])
                self.assertEqual("unknown" if operation else "original_delivery_requires_reconciliation" if marker
                                 else "original_operation_missing", result.get("phase") if operation else result["reasonCode"])
                self.assertEqual(before, self.witness(case))
        with self.case() as case, patch.dict(os.environ, {"ZIRCON_JENKINS_SESSION_ID": "original-chat"}):
            before = self.witness(case)
            with patch.object(entry, "capability", return_value=case.context):
                for options in ({}, {"dry_run": True}, {"resume_original": True},
                                {"query_only": True, "dry_run": True},
                                {"query_only": True, "resume_original": True}):
                    with self.subTest(options=options), self.assertRaises(entry.EntryError):
                        entry.dispatch(case.repo, case.incoming, **options)
                with self.assertRaises(entry.EntryError):
                    entry.input_payload(case.context, case.incoming, automatic_context=False)
            self.assertEqual(before, self.witness(case))
            case.context["module"].route_request_context.assert_not_called()
            case.context["module"].ensure_request_context.assert_not_called()


if __name__ == "__main__":
    unittest.main()
