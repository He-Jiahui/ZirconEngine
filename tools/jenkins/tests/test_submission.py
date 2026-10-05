import tempfile
import unittest
from pathlib import Path

from tools.jenkins.state import State
from tools.jenkins.submission import dispatch_registered, request_payload, submit
from tools.jenkins.contracts import JenkinsError


class SubmissionTests(unittest.TestCase):
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


if __name__ == "__main__":
    unittest.main()
