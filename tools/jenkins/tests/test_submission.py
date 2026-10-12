import tempfile
import unittest
from pathlib import Path

from tools.jenkins.state import State
from tools.jenkins.submission import dispatch_registered, request_payload, submit
from tools.jenkins.contracts import JenkinsError


class SubmissionTests(unittest.TestCase):
    def test_managed_dispatch_persists_actual_location_and_replay_is_read_only(self):
        with tempfile.TemporaryDirectory() as directory:
            state = State(Path(directory) / "state.sqlite3")
            payload = {"repositoryId": "repo", "sessionId": "session", "requestId": "managed-request"}
            class Manager:
                base_url = "http://127.0.0.1:18080"
                def __init__(self):
                    self.calls = []
                def _request_with_headers(self, path, **kwargs):
                    self.calls.append((path, kwargs))
                    return 201, b"", {"location": self.base_url + "/queue/item/21/"}
                def _request(self, *args, **kwargs):
                    raise AssertionError("Response headers must not be discarded")
            manager = Manager()
            first = dispatch_registered(state, payload, {"GENERATION": "1"}, manager=manager)
            second = dispatch_registered(state, payload, {"GENERATION": "1"}, manager=manager)
            self.assertEqual("http://127.0.0.1:18080/queue/item/21/", first["queueUrl"])
            self.assertEqual(first["queueUrl"], second["queueUrl"])
            self.assertEqual(1, len(manager.calls))
            self.assertEqual("POST", manager.calls[0][1]["method"])
            self.assertFalse(first["formalAcceptance"])

    def test_duplicate_request_has_no_second_http_side_effect(self):
        with tempfile.TemporaryDirectory() as directory:
            state = State(Path(directory) / "state.sqlite3")
            payload = request_payload(repository_id="repo", session_id="session",
                                      request_id="request", source_ref="main",
                                      source_digest="source", coverage_ref="unit",
                                      coverage_digest="coverage")
            calls = []
            class Response:
                status = 201
                headers = {"Location": "http://127.0.0.1:18080/queue/item/1"}
            def transport(request):
                calls.append(request)
                return Response()
            first = submit(state, payload, transport=transport)
            second = submit(state, payload, transport=transport)
            self.assertEqual(first["queueUrl"], "http://127.0.0.1:18080/queue/item/1")
            self.assertEqual(second["request"]["requestId"], "request")
            self.assertEqual(len(calls), 1)
            self.assertFalse(first["formalAcceptance"])

    def test_registered_dispatch_rejects_changed_form_identity_before_claim(self):
        with tempfile.TemporaryDirectory() as directory:
            state = State(Path(directory) / "state.sqlite3")
            payload = {"repositoryId": "repo", "sessionId": "session", "requestId": "request",
                       "patchRequestRef": "patch-a"}
            for fields in ({"REQUEST_ID": "foreign"}, {"PATCH_REQUEST_REF": "patch-b"}):
                with self.subTest(fields=fields), self.assertRaises(JenkinsError):
                    dispatch_registered(state, payload, fields, transport=lambda _: self.fail("unexpected POST"))
            self.assertIsNone(state.get("jenkins_submission", "repo:session:request:flow"))

    def test_duplicate_dispatch_binds_target_and_exact_form(self):
        with tempfile.TemporaryDirectory() as directory:
            state = State(Path(directory) / "state.sqlite3")
            payload = {"repositoryId": "repo", "sessionId": "session", "requestId": "request"}
            calls = []
            class Response:
                status = 201
                headers = {}
            def transport(request):
                calls.append(request)
                return Response()
            dispatch_registered(state, payload, {"GENERATION": "1"}, transport=transport)
            for changes in ({"fields": {"GENERATION": "2"}}, {"job": "foreign-job"},
                            {"base_url": "http://127.0.0.1:28080"}):
                with self.subTest(changes=changes), self.assertRaises(JenkinsError):
                    options = {"fields": {"GENERATION": "1"}, **changes}
                    dispatch_registered(state, payload, transport=transport, **options)
            self.assertEqual(1, len(calls))

    def test_historical_marker_without_dispatch_binding_is_preserved(self):
        from tools.jenkins.contracts import digest
        with tempfile.TemporaryDirectory() as directory:
            state = State(Path(directory) / "state.sqlite3")
            payload = {"repositoryId": "repo", "sessionId": "session", "requestId": "request"}
            original = state.put("jenkins_submission", "repo:session:request:flow",
                                 {**payload, "payloadDigest": digest(payload), "delivery": "submitted"})
            with self.assertRaises(JenkinsError) as raised:
                dispatch_registered(state, payload, {}, transport=lambda _: self.fail("unexpected POST"))
            self.assertEqual("delivery_unknown", raised.exception.code)
            self.assertEqual(original, state.get("jenkins_submission", "repo:session:request:flow"))

    def test_missing_identity_is_rejected(self):
        with self.assertRaises(Exception):
            request_payload(repository_id="repo", session_id="session", request_id="request",
                            source_ref="main", source_digest="", coverage_ref="unit",
                            coverage_digest="coverage")

    def test_secondary_submission_cannot_replace_registered_input_identity(self):
        with tempfile.TemporaryDirectory() as directory:
            state = State(Path(directory) / "state.sqlite3")
            original = request_payload(repository_id="repo", session_id="session",
                                       request_id="request", source_ref="source-a",
                                       source_digest="a" * 64, coverage_ref="unit",
                                       coverage_digest="b" * 64)
            state.submit_request(original)
            for replacement in ({"sourceDigest": "c" * 64}, {"coverageDigest": "c" * 64}):
                with self.subTest(replacement=replacement):
                    payload = dict(original, **replacement,
                                   parameters={"submissionRequestId": "execution-" + next(iter(replacement))})
                    calls = []

                    class Response:
                        status = 201
                        headers = {}

                    def transport(request):
                        calls.append(request)
                        return Response()

                    with self.assertRaisesRegex(JenkinsError, "identity"):
                        submit(state, payload, transport=transport)
                    self.assertEqual([], calls)
                    self.assertEqual(original, state.get_request("repo", "session", "request")["payload"])

    def test_parameters_cannot_override_identity_in_jenkins_form(self):
        with tempfile.TemporaryDirectory() as directory:
            state = State(Path(directory) / "state.sqlite3")
            for name in ("REPOSITORY_ID", "repositoryId", "SESSION_ID", "REQUEST_ID",
                         "SEALED_INPUT_REF", "SOURCE_INPUT_DIGEST", "COVERAGE_REF", "COVERAGE_DIGEST"):
                with self.subTest(parameter=name):
                    payload = request_payload(repository_id="repo", session_id="session",
                                              request_id="request-" + name,
                                              source_ref="a" * 64, source_digest="a" * 64,
                                              coverage_ref="b" * 64, coverage_digest="b" * 64,
                                              parameters={name: "replacement"})
                    calls = []

                    class Response:
                        status = 201
                        headers = {}

                    def transport(request):
                        calls.append(request)
                        return Response()

                    with self.assertRaisesRegex(JenkinsError, "identity"):
                        submit(state, payload, transport=transport)
                    self.assertEqual([], calls)
                    self.assertIsNone(state.get_request("repo", "session", payload["requestId"]))

    def test_registered_patch_intent_dispatch_does_not_submit_final_source_identity(self):
        with tempfile.TemporaryDirectory() as directory:
            state = State(Path(directory) / "state.sqlite3")
            payload = {
                "repositoryId": "repo", "sessionId": "session", "requestId": "patch-request",
                "patchRequestRef": "patch-" + "a" * 64,
                "parameters": {"submissionRequestId": "flow", "patchRequestRef": "patch-" + "a" * 64},
            }
            fields = {"REPOSITORY_ID": "repo", "SESSION_ID": "session", "REQUEST_ID": "patch-request",
                      "PATCH_REQUEST_REF": "patch-" + "a" * 64}
            calls = []

            class Response:
                status = 201
                headers = {}

            def transport(request):
                calls.append(request)
                return Response()

            registered = {"payload": payload, "status": "patch-intent"}
            result = dispatch_registered(state, payload, fields, registered_request=registered,
                                         transport=transport)
            self.assertEqual(result["request"], registered)
            self.assertEqual(len(calls), 1)
            self.assertIsNone(state.get_request("repo", "session", "patch-request"))

    def test_patch_marker_collision_is_rejected_without_second_post(self):
        with tempfile.TemporaryDirectory() as directory:
            state = State(Path(directory) / "state.sqlite3")
            calls = []

            class Response:
                status = 201
                headers = {}

            def transport(request):
                calls.append(request)
                return Response()

            first = {"repositoryId": "repo", "sessionId": "session", "requestId": "patch-request",
                     "patchRequestRef": "patch-" + "a" * 64,
                     "parameters": {"patchRequestRef": "patch-" + "a" * 64}}
            second = dict(first, patchRequestRef="patch-" + "b" * 64,
                          parameters={"patchRequestRef": "patch-" + "b" * 64})
            dispatch_registered(state, first, {"PATCH_REQUEST_REF": first["patchRequestRef"]}, transport=transport)
            with self.assertRaisesRegex(JenkinsError, "different payload"):
                dispatch_registered(state, second, {"PATCH_REQUEST_REF": second["patchRequestRef"]}, transport=transport)
            self.assertEqual(len(calls), 1)

    def test_non_success_or_missing_http_status_is_unknown_and_not_replayed(self):
        for status in (500, None):
            with self.subTest(status=status), tempfile.TemporaryDirectory() as directory:
                state = State(Path(directory) / "state.sqlite3")
                payload = {"repositoryId": "repo", "sessionId": "session", "requestId": "patch-request",
                           "patchRequestRef": "patch-" + "a" * 64}
                calls = []

                Response = type("Response", (), {"headers": {}, **({"status": status} if status is not None else {})})

                def transport(request):
                    calls.append(request)
                    return Response()

                with self.assertRaises(JenkinsError) as raised:
                    dispatch_registered(state, payload, {"PATCH_REQUEST_REF": payload["patchRequestRef"]}, transport=transport)
                self.assertEqual(raised.exception.code, "delivery_unknown")
                self.assertEqual(state.get("jenkins_submission", "repo:session:patch-request:flow")["payload"]["delivery"], "unknown")
                with self.assertRaises(JenkinsError) as raised:
                    dispatch_registered(state, payload, {"PATCH_REQUEST_REF": payload["patchRequestRef"]}, transport=transport)
                self.assertEqual(raised.exception.code, "delivery_unknown")
                self.assertEqual(len(calls), 1)

    def test_patch_parameter_alias_is_emitted_without_source_fields(self):
        from tools.jenkins.submission import _submission_fields
        fields = _submission_fields({"repositoryId": "repo", "sessionId": "session", "requestId": "r",
                                     "parameters": {"patchRequestRef": "patch-ref"}})
        self.assertEqual(fields["PATCH_REQUEST_REF"], "patch-ref")
        self.assertNotIn("SEALED_INPUT_REF", fields)
        self.assertNotIn("SOURCE_INPUT_DIGEST", fields)

    def test_typed_top_level_source_and_runtime_refs_use_uppercase_form_fields(self):
        from tools.jenkins.submission import _submission_fields
        payload = {"repositoryId": "repo", "sessionId": "session", "requestId": "typed-request",
                   "currentSourceRef": "current-ref", "sourcePreparationRef": "preparation-ref",
                   "runtimeOperationId": "start-owned", "attemptId": "attempt-owned", "generation": 1,
                   "driverDigest": "d" * 64, "parameters": {"rustEdition": "2021"}}
        fields = _submission_fields(payload)
        for name, value in (("CURRENT_SOURCE_REF", "current-ref"), ("SOURCE_PREPARATION_REF", "preparation-ref"),
                            ("RUNTIME_OPERATION_ID", "start-owned"), ("ATTEMPT_ID", "attempt-owned"),
                            ("GENERATION", "1"), ("STAGE_IMPLEMENTATION_DIGEST", "d" * 64), ("RUST_EDITION", "2021")):
            self.assertEqual(value, fields[name])
        self.assertNotIn("currentSourceRef", fields)
        self.assertNotIn("sourcePreparationRef", fields)

    def test_typed_identity_cannot_be_overridden_through_parameter_aliases(self):
        with tempfile.TemporaryDirectory() as directory:
            state = State(Path(directory) / "state.sqlite3")
            protected = (("currentSourceRef", "CURRENT_SOURCE_REF"), ("sourcePreparationRef", "SOURCE_PREPARATION_REF"),
                         ("runtimeOperationId", "RUNTIME_OPERATION_ID"), ("attemptId", "ATTEMPT_ID"),
                         ("generation", "GENERATION"), ("stageImplementationDigest", "STAGE_IMPLEMENTATION_DIGEST"))
            for field, uppercase in protected:
                for alias in (field, uppercase):
                    with self.subTest(field=field, alias=alias):
                        payload = {"repositoryId": "repo", "sessionId": "session", "requestId": "typed-request",
                                   field: "owned", "parameters": {alias: "foreign"}}
                        with self.assertRaises(JenkinsError) as raised:
                            dispatch_registered(state, payload, {}, transport=lambda _: self.fail("unexpected POST"))
                        self.assertEqual("submission_identity_override", raised.exception.code)
            self.assertEqual([], state.list("jenkins_submission"))

    def test_conflicting_aliases_without_top_level_identity_are_order_independent(self):
        from tools.jenkins.submission import _submission_fields
        for field, uppercase in (("currentSourceRef", "CURRENT_SOURCE_REF"), ("sourcePreparationRef", "SOURCE_PREPARATION_REF"),
                                 ("runtimeOperationId", "RUNTIME_OPERATION_ID"), ("driverDigest", "STAGE_IMPLEMENTATION_DIGEST")):
            for parameters in ({field: "owned", uppercase: "foreign"}, {uppercase: "foreign", field: "owned"}):
                with self.subTest(field=field, parameters=parameters), self.assertRaises(JenkinsError) as raised:
                    _submission_fields({"parameters": parameters})
                self.assertEqual("submission_parameter_conflict", raised.exception.code)

    def test_secondary_submission_cannot_change_or_drop_registered_typed_identity(self):
        with tempfile.TemporaryDirectory() as directory:
            state = State(Path(directory) / "state.sqlite3")
            original = request_payload(repository_id="repo", session_id="session", request_id="typed-request",
                                       source_ref="a" * 64, source_digest="a" * 64,
                                       coverage_ref="b" * 64, coverage_digest="b" * 64)
            original.update(currentSourceRef="current-owned", sourcePreparationRef="preparation-owned",
                            runtimeOperationId="start-owned", attemptId="attempt-owned", generation=1,
                            stageImplementationDigest="d" * 64, buildRoot="D:/cargo-targets")
            state.submit_request(original)
            for field in ("currentSourceRef", "sourcePreparationRef", "runtimeOperationId", "attemptId",
                          "generation", "stageImplementationDigest", "buildRoot"):
                for action in ("replace", "drop"):
                    with self.subTest(field=field, action=action):
                        payload = {**original, "parameters": {"submissionRequestId": field + action}}
                        if action == "replace":
                            payload[field] = "foreign"
                        else:
                            payload.pop(field)
                        with self.assertRaises(JenkinsError) as raised:
                            submit(state, payload, transport=lambda _: self.fail("unexpected POST"))
                        self.assertEqual("submission_identity_mismatch", raised.exception.code)
            self.assertEqual(original, state.get_request("repo", "session", "typed-request")["payload"])
            self.assertEqual([], state.list("jenkins_submission"))

    def test_secondary_submission_cannot_invent_typed_identity_for_legacy_request(self):
        with tempfile.TemporaryDirectory() as directory:
            state = State(Path(directory) / "state.sqlite3")
            original = request_payload(repository_id="repo", session_id="session", request_id="legacy-request",
                                       source_ref="a" * 64, source_digest="a" * 64,
                                       coverage_ref="b" * 64, coverage_digest="b" * 64)
            state.submit_request(original)
            for field in ("currentSourceRef", "sourcePreparationRef", "runtimeOperationId", "attemptId", "generation"):
                with self.subTest(field=field), self.assertRaises(JenkinsError) as raised:
                    submit(state, {**original, field: "foreign", "parameters": {"submissionRequestId": "flow"}},
                           transport=lambda _: self.fail("unexpected POST"))
                self.assertEqual("submission_identity_mismatch", raised.exception.code)
            self.assertEqual([], state.list("jenkins_submission"))

    def test_registered_dispatch_normalizes_and_checks_supplied_form_aliases(self):
        with tempfile.TemporaryDirectory() as directory:
            state = State(Path(directory) / "state.sqlite3")
            payload = {"repositoryId": "repo", "sessionId": "session", "requestId": "typed-request",
                       "currentSourceRef": "current-owned", "runtimeOperationId": "start-owned"}
            for fields in ({"currentSourceRef": "foreign"}, {"RUNTIME_OPERATION_ID": "foreign"},
                           {"runtimeOperationId": "foreign", "RUNTIME_OPERATION_ID": "start-owned"}):
                with self.subTest(fields=fields), self.assertRaises(JenkinsError):
                    dispatch_registered(state, payload, fields, transport=lambda _: self.fail("unexpected POST"))
            self.assertEqual([], state.list("jenkins_submission"))


if __name__ == "__main__":
    unittest.main()
