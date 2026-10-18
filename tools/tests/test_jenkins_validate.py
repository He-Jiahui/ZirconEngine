from __future__ import annotations

from contextlib import closing
import hashlib
import json
from pathlib import Path
import sqlite3
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from tools.dev import jenkins_validate as entry


def checksum(data):
    return hashlib.sha256(data).hexdigest()


class JenkinsValidateTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="jenkins-entry-test-", dir=Path(__file__).resolve().parent)
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / "src").mkdir()
        (self.root / "src/lib.rs").write_bytes(b"before\n")
        (self.root / "Cargo.toml").write_bytes(b"[workspace]\n")
        self.patch_file = self.root / "change.patch"
        self.patch_file.write_bytes(
            b"diff --git a/src/lib.rs b/src/lib.rs\n"
            b"--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1 +1 @@\n-before\n+after\n")
        self.hashes = self.root / "before.json"
        self.hashes.write_text(json.dumps({"src/lib.rs": checksum(b"before\n")}), encoding="utf-8")
        self.driver = self.install_driver()
        self.calls, self.responses, self.patch_ref = [], {}, "f" * 64

    def install_driver(self):
        contents = {"launcher.py": b"# fixture launcher\n", "pipeline/steps.groovy": b"// fixture pipeline\n"}
        files = {name: {"sha256": checksum(data), "bytes": len(data)} for name, data in contents.items()}
        driver_digest = entry.digest({"schemaVersion": 1, "files": files})
        driver_root = self.root / ".jenkins/runtime/drivers" / driver_digest
        for name, data in contents.items():
            target = driver_root / name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
        (driver_root / "driver-manifest.json").write_bytes(
            entry.canonical_json({"schemaVersion": 1, "driverDigest": driver_digest, "files": files}))
        python = self.root / ".jenkins/runtime/python/python.exe"
        python.parent.mkdir(parents=True, exist_ok=True)
        python.write_bytes(b"fixture runtime")
        marker = self.root / ".jenkins/state/deployment/driver.json"
        marker.parent.mkdir(parents=True, exist_ok=True)
        marker.write_bytes(entry.canonical_json({
            "driverDigest": driver_digest, "root": str(driver_root),
            "launcher": str(driver_root / "launcher.py"), "steps": str(driver_root / "pipeline/steps.groovy"),
            "runtimeOperationId": "runtime-fixture"}))
        (self.root / ".jenkins/deployment-spec.json").write_bytes(
            entry.canonical_json({"storage": {"buildRoot": r"D:\cargo-targets"}}))
        return entry.selected_driver(self.root)

    def args(self, action="dry-check", *extra):
        return entry.parser().parse_args([
            action, "--repository", str(self.root), "--session-id", "session",
            "--request-id", "request", "--patch", str(self.patch_file),
            "--before-hashes", str(self.hashes), "--coverage-template", "module_unit",
            "--dependency", "Cargo.toml", "--build-root", r"F:\cargo-targets", *extra])

    def fake_protocol(self, root, driver, folder, domain, action, payload, timeout):
        self.calls.append((domain, action, payload))
        value = self.responses.get((domain, action))
        if isinstance(value, Exception):
            raise value
        if value is not None:
            return value
        if (domain, action) == ("patch", "register"):
            return {"status": "registered", "patchRequestRef": self.patch_ref}
        if (domain, action) == ("patch", "submit"):
            return {"status": 201, "delivery": "submitted", "queueUrl": "http://fixture/queue/1/"}
        return {"status": "registered"}

    def execute(self, args):
        return entry.execute(args, call=self.fake_protocol)

    def files(self):
        return {p.relative_to(self.root).as_posix(): p.read_bytes()
                for p in self.root.rglob("*") if p.is_file()}

    def write_workflow(self, identity, status="accepted", *, change=None, save_acceptance=True):
        # Use the actual backend State/accept_flow record shapes and digest rules.
        from tools.jenkins.state import SCHEMA
        from tools.jenkins.validation.receipts import validate_acceptance_receipt
        workflow = {"identity": identity, "repositoryId": identity["repositoryId"],
                    "sourceDigest": identity["sourceInputDigest"], "coverageDigest": identity["coverageDigest"],
                    "driverDigest": identity["stageImplementationDigest"], "buildRoot": r"F:\cargo-targets",
                    "status": status, "executionId": "execution-fixture", "requiredStages": ["syntax"],
                    "stageReceipts": {"syntax": "syntax-fixture"}, "stageRecipeDigests": {"syntax": "c" * 64},
                    "stages": [{"name": "syntax", "status": "passed"}]}
        flow_key = ":".join(identity[key] for key in ("sessionId", "requestId", "attemptId"))
        flow_identity = {key: workflow.get(key) for key in
                         ("sourceDigest", "coverageDigest", "driverDigest", "generation", "beforePatch", "dagDigest")}
        acceptance_id = entry.digest({"flow": flow_key, "identity": flow_identity, "stages": ["syntax"]})
        receipt = {"schemaVersion": 1, "kind": "jenkins.authoritative.acceptance-receipt", "status": "accepted",
                   "authority": "jenkins-validation", "acceptanceId": acceptance_id, "acceptedAt": 1,
                   "sourceDigest": identity["sourceInputDigest"], "coverageDigest": identity["coverageDigest"],
                   "driverDigest": identity["stageImplementationDigest"], "executionId": "syntax-fixture",
                   "stage": "syntax", "recipeDigest": "c" * 64, "producerReceiptDigest": "d" * 64,
                   "outputClosure": {"root": "fixture", "files": [], "digest": entry.digest([])},
                   "flowIdentity": flow_identity, "stages": ["syntax"]}
        receipt["receiptDigest"] = entry.digest(receipt)
        validate_acceptance_receipt(receipt)
        workflow["acceptance"] = receipt
        if change:
            change(workflow)
        database = self.root / ".jenkins/state/coordination.sqlite3"
        with closing(sqlite3.connect(database)) as connection, connection:
            connection.executescript(SCHEMA)
            connection.execute("DELETE FROM records")
            for domain, key, payload in [("workflow", flow_key, workflow)] + (
                    [("acceptance", acceptance_id, receipt)] if save_acceptance else []):
                connection.execute("INSERT INTO records VALUES (?,?,?,?,?)",
                                   (domain, key, entry.canonical_json(payload).decode(), 1, 1))
        return workflow

    def test_default_dry_check_reads_only_and_preserves_explicit_f_root(self):
        before = self.files()
        result = self.execute(self.args())
        self.assertEqual("dry-check", result["status"])
        self.assertEqual(r"F:\cargo-targets", result["buildRoot"])
        self.assertEqual([], self.calls)
        self.assertEqual(before, self.files())
        self.assertFalse(result["formalAcceptance"])
        self.assertEqual("dry-check", entry.parser().parse_args(
            ["--session-id", "s", "--request-id", "r"]).action)

    def test_configured_storage_root_is_default(self):
        args = self.args()
        args.build_root = None
        self.assertEqual(r"D:\cargo-targets", self.execute(args)["buildRoot"])

    def test_submit_registers_then_dispatches_with_one_identity(self):
        result = self.execute(self.args("submit"))
        self.assertEqual([("patch", "register"), ("patch", "submit")],
                         [(domain, action) for domain, action, _ in self.calls])
        registered, submitted = [payload for _, _, payload in self.calls]
        self.assertEqual(registered["identity"], submitted["identity"])
        self.assertEqual("request-attempt-1", submitted["identity"]["attemptId"])
        self.assertEqual(self.driver["digest"], submitted["identity"]["stageImplementationDigest"])
        self.assertEqual(r"F:\cargo-targets", submitted["buildRoot"])
        self.assertEqual("runtime-fixture", submitted["runtimeOperationId"])
        self.assertEqual("submitted", result["status"])
        self.assertFalse(result["formalAcceptance"])
        self.assertEqual(b"before\n", (self.root / "src/lib.rs").read_bytes())

    def test_repeat_submit_queries_without_second_dispatch(self):
        first = self.execute(self.args("submit"))
        self.calls.clear()
        second = self.execute(self.args("submit"))
        self.assertEqual([("patch", "query")], [(d, a) for d, a, _ in self.calls])
        self.assertEqual(first["identity"], second["identity"])
        self.assertFalse(second["formalAcceptance"])

    def test_unknown_delivery_queries_same_identity_without_retry(self):
        self.responses["patch", "submit"] = entry.EntryError("delivery_unknown", "unknown", retryable=True)
        result = self.execute(self.args("submit"))
        self.assertEqual("delivery_unknown", result["status"])
        submit, query = self.calls[-2:]
        self.assertEqual(("patch", "query"), query[:2])
        self.assertEqual(submit[2], query[2])
        self.calls.clear()
        self.execute(self.args("submit"))
        self.assertEqual([("patch", "query")], [(d, a) for d, a, _ in self.calls])

    def test_unknown_registration_does_not_dispatch_on_resume(self):
        self.responses["patch", "register"] = entry.EntryError("protocol_delivery_unknown", "unknown", retryable=True)
        with self.assertRaises(entry.EntryError):
            self.execute(self.args("submit"))
        self.calls.clear()
        self.execute(self.args("query"))
        self.assertEqual([("request", "query")], [(d, a) for d, a, _ in self.calls])

    def test_existing_authorization_rejection_is_preserved(self):
        self.responses["patch", "register"] = entry.EntryError("session_not_authorized", "No existing authorization")
        with self.assertRaises(entry.EntryError) as caught:
            self.execute(self.args("submit"))
        self.assertEqual("session_not_authorized", caught.exception.code)
        self.assertEqual([("patch", "register")], [(d, a) for d, a, _ in self.calls])

    def test_new_file_with_null_before_hash(self):
        self.patch_file.write_bytes(
            b"diff --git a/src/new.rs b/src/new.rs\nnew file mode 100644\n"
            b"--- /dev/null\n+++ b/src/new.rs\n@@ -0,0 +1 @@\n+new\n")
        self.hashes.write_text('{"src/new.rs":null}', encoding="utf-8")
        self.assertEqual(["src/new.rs"], self.execute(self.args())["ownedPaths"])
        self.assertFalse((self.root / "src/new.rs").exists())
        self.execute(self.args("submit"))
        self.assertIsNone(self.calls[0][2]["beforeHashes"]["src/new.rs"])

    def test_new_file_requires_null_before_hash(self):
        self.patch_file.write_bytes(b"--- /dev/null\n+++ b/src/new.rs\n@@ -0,0 +1 @@\n+new\n")
        self.hashes.write_text(json.dumps({"src/new.rs": "a" * 64}), encoding="utf-8")
        with self.assertRaises(entry.EntryError) as caught:
            self.execute(self.args())
        self.assertEqual("patch_scope_mismatch", caught.exception.code)

    def test_content_and_attempt_conflict_are_rejected(self):
        self.execute(self.args("submit"))
        self.patch_file.write_bytes(self.patch_file.read_bytes().replace(b"+after", b"+other"))
        with self.assertRaises(entry.EntryError) as caught:
            self.execute(self.args("submit"))
        self.assertEqual("request_input_conflict", caught.exception.code)
        with self.assertRaises(entry.EntryError) as caught:
            self.execute(self.args("query", "--attempt-id", "other"))
        self.assertEqual("request_identity_conflict", caught.exception.code)

    def test_bad_root_and_escape_are_rejected(self):
        for root in (r"C:\cargo-targets", r"D:\targets", r"F:\cargo-targets\nested",
                     r"D:\ZirconBuilds", r"D:\nested\cargo-targets"):
            with self.subTest(root=root), self.assertRaises(entry.EntryError):
                self.execute(self.args("dry-check", "--build-root", root))
        self.hashes.write_text('{"../outside.rs":null}', encoding="utf-8")
        with self.assertRaises(entry.EntryError):
            self.execute(self.args())

    def test_invalid_hunk_and_driver_tampering_are_rejected(self):
        self.patch_file.write_bytes(b"--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1 +2,2 @@\n-before\n+after\n")
        with self.assertRaises(entry.EntryError):
            self.execute(self.args())
        self.driver["launcher"].write_bytes(b"changed bytes")
        with self.assertRaises(entry.EntryError) as caught:
            entry.selected_driver(self.root)
        self.assertEqual("driver_integrity_failed", caught.exception.code)

    def test_terminal_result_is_not_acceptance(self):
        submitted = self.execute(self.args("submit"))
        final_identity = {**submitted["identity"], "sourceInputDigest": "a" * 64, "coverageDigest": "b" * 64}
        self.responses["patch", "query"] = {
            "status": "prepared", "preparation": {"payload": {"result": {"identity": final_identity}}}}
        self.write_workflow(final_identity, "complete")
        self.assertFalse(self.execute(self.args("query"))["formalAcceptance"])
        self.write_workflow(final_identity)
        before = self.files()
        self.calls.clear()
        accepted = self.execute(self.args("query"))
        self.assertTrue(accepted["formalAcceptance"])
        self.assertEqual("accepted", accepted["status"])
        self.assertEqual([("patch", "query")], [(d, a) for d, a, _ in self.calls])
        self.assertEqual(before, self.files())

    def test_accepted_row_requires_exact_identity_and_persisted_receipt(self):
        submitted = self.execute(self.args("submit"))
        final_identity = {**submitted["identity"], "sourceInputDigest": "a" * 64, "coverageDigest": "b" * 64}
        self.responses["patch", "query"] = {
            "status": "prepared", "preparation": {"payload": {"result": {"identity": final_identity}}}}
        changes = [
            lambda flow: flow["identity"].update(sourceInputDigest="e" * 64),
            lambda flow: flow.update(coverageDigest="e" * 64),
            lambda flow: flow.update(buildRoot=r"D:\cargo-targets"),
            lambda flow: flow["acceptance"].update(receiptDigest="e" * 64),
            lambda flow: flow["acceptance"].update(acceptanceId="e" * 64),
            lambda flow: flow["stages"][0].update(status="pending"),
        ]
        for change in changes:
            with self.subTest(change=change):
                self.write_workflow(dict(final_identity), change=change)
                with self.assertRaises(entry.EntryError):
                    self.execute(self.args("query"))
        self.write_workflow(final_identity, save_acceptance=False)
        with self.assertRaises(entry.EntryError):
            self.execute(self.args("query"))

    def test_driver_environment_and_repository_identity_match_backend(self):
        from tools.jenkins.deployment.driver import driver_environment
        from tools.jenkins.source import repository_id
        actual = driver_environment({"root": str(self.driver["launcher"].parent),
                                     "driverDigest": self.driver["digest"]},
                                    self.driver["python"], self.root)
        for key, value in actual.items():
            self.assertEqual(value, self.driver["env"][key])
        self.assertEqual(repository_id(self.root), self.execute(self.args())["identity"]["repositoryId"])

    def test_protocol_selected_launcher_and_environment(self):
        identity = self.execute(self.args())["identity"]
        payload = {"identity": identity, "buildRoot": r"F:\cargo-targets"}
        completed = subprocess.CompletedProcess([], 0, '{"status":"registered","patchRequestRef":"ref"}', "")
        with patch.object(entry.subprocess, "run", return_value=completed) as run:
            entry.protocol(self.root, self.driver, self.root / ".jenkins/state/requests/test",
                           "patch", "register", payload, 30)
        command = run.call_args.args[0]
        self.assertEqual([str(self.driver["python"]), "-B", str(self.driver["launcher"])], command[:3])
        self.assertEqual(["patch", "register"], command[5:7])
        env = run.call_args.kwargs["env"]
        self.assertEqual(self.driver["digest"], env["ZIRCON_DRIVER_DIGEST"])
        self.assertEqual(str(self.driver["launcher"]), env["ZIRCON_DRIVER_LAUNCHER"])
        self.assertEqual(str(self.driver["python"]), env["JENKINS_PYTHON"])
        self.assertEqual(str(self.root), env["ZIRCON_REPO_ROOT"])
        self.assertEqual("1", env["PYTHONDONTWRITEBYTECODE"])
        self.assertEqual(payload, json.loads(Path(command[-1]).read_text(encoding="utf-8")))

    def test_interrupted_protocol_does_not_repeat_launcher(self):
        with patch.object(entry.subprocess, "run", side_effect=subprocess.TimeoutExpired("fixture", 1)) as run:
            with self.assertRaises(entry.EntryError) as caught:
                entry.protocol(self.root, self.driver, self.root / ".jenkins/state/requests/test",
                               "patch", "submit", {"identity": {}}, 1)
        self.assertEqual("protocol_delivery_unknown", caught.exception.code)
        self.assertEqual(1, run.call_count)


if __name__ == "__main__":
    unittest.main()
