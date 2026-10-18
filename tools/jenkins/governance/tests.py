"""Contract tests. Run only in Jenkins-owned, pre-registered fixture roots."""
from __future__ import annotations

from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import unittest
from unittest import mock
import uuid

from .authority import Governance
from .inventory import git, repository_fact, resolve_primary, scan, text_git
from .paths import build_namespace, digest, encoded, file_fact, physical
from .versioning import seal, prepare_integration, write_git
from tools.jenkins.state import State


class GovernanceTests(unittest.TestCase):
    def setUp(self):
        if not os.environ.get("ZGOV_TEST_ROOT") or not os.environ.get("JOB_NAME", "").startswith("zircon-governance"):
            raise RuntimeError("Jenkins-owned governance fixtures required")
        self.root = build_namespace(os.environ["ZGOV_TEST_ROOT"]) / (self._testMethodName + "-" + uuid.uuid4().hex[:8])
        self.root.mkdir()
        self.repo = self.root / "repo"
        self.repo.mkdir()
        write_git(self.repo, ["init", "-b", "main"])
        (self.repo / "one.txt").write_bytes(b"base\n")
        (self.repo / "foreign.txt").write_bytes(b"foreign-base\n")
        write_git(self.repo, ["add", "one.txt", "foreign.txt"])
        write_git(self.repo, ["commit", "-m", "fixture baseline"])
        self.state = State(self.root / "state.sqlite3")
        self.authority = Governance(self.state, self.repo, {"driverDigest": "fixture-driver", "capsuleDigest": "fixture-capsule",
                                  "operationId": "fixture-op", "sessionId": "fixture-session", "attemptId": "fixture-attempt"})
        self.registration = self.authority.register({"canonicalCheckout": str(self.repo),
            "buildNamespace": str(self.root / "compiler"), "validationRecipes": []})
        self.repository_id = self.registration["repositoryId"]
        observation = {"schemaVersion": 1, "roots": [str(self.root)], "repositories": [], "errors": []}
        observation["inventoryDigest"] = digest(observation)
        self.inventory = self.root / "fixture-inventory.json"
        self.inventory.write_bytes(encoded(observation))
        self.round = {"roundId": "fixture-round", "inventoryRef": file_fact(self.inventory),
                      "inventoryDigest": observation["inventoryDigest"]}
        self.authority.open_round(self.round)

    def close(self):
        response = self.authority.record_response({"roundId": self.round["roundId"], "userReply": "fixture: no further claims"})
        return self.authority.close_round({"roundId": self.round["roundId"], "replyDigest": response["replyDigest"]})

    def claim(self, session="fixture-a", paths=None):
        selected = paths or ["one.txt"]
        return self.authority.claim({"roundId": self.round["roundId"], "repositoryId": self.repository_id,
            "ownerSessionId": session, "paths": selected,
            "expectedHashes": {p: file_fact(self.repo / p)["sha256"] if (self.repo / p).exists() else None for p in selected}})

    def seal(self, paths=None, operation="fixture-seal", owner="maintenance"):
        selected = paths or ["one.txt"]
        observation = repository_fact(self.repo)
        scratch = self.root / operation
        scratch.mkdir()
        return seal(self.authority, {"roundId": self.round["roundId"], "inventoryDigest": self.round["inventoryDigest"],
            "repositoryId": self.repository_id, "ownerSessionId": owner, "operationId": operation,
            "paths": selected, "expectedHead": observation["head"], "expectedStatusDigest": observation["statusDigest"],
            "expectedIndexDigest": observation["indexDigest"],
            "expectedHashes": {p: file_fact(self.repo / p)["sha256"] if (self.repo / p).exists() else None for p in selected}}, scratch)

    def test_changed_staged_bytes_with_identical_status_and_working_bytes_are_rejected(self):
        (self.repo / "one.txt").write_bytes(b"staged first\n")
        write_git(self.repo, ["add", "one.txt"])
        (self.repo / "one.txt").write_bytes(b"unchanged working version\n")
        before = repository_fact(self.repo)
        expected = file_fact(self.repo / "one.txt")["sha256"]
        (self.repo / "one.txt").write_bytes(b"staged second\n")
        write_git(self.repo, ["add", "one.txt"])
        (self.repo / "one.txt").write_bytes(b"unchanged working version\n")
        after = repository_fact(self.repo)
        self.assertEqual(before["statusDigest"], after["statusDigest"])
        self.assertEqual(expected, file_fact(self.repo / "one.txt")["sha256"])
        self.assertNotEqual(before["indexDigest"], after["indexDigest"])
        self.close()
        scratch = self.root / "stale-index-seal"
        scratch.mkdir()
        with self.assertRaises(ValueError):
            seal(self.authority, {"roundId": "fixture-round", "inventoryDigest": self.round["inventoryDigest"],
                "repositoryId": self.repository_id, "ownerSessionId": "maintenance", "operationId": "stale-index-seal",
                "paths": ["one.txt"], "expectedHead": before["head"], "expectedStatusDigest": before["statusDigest"],
                "expectedIndexDigest": before["indexDigest"], "expectedHashes": {"one.txt": expected}}, scratch)
        self.assertEqual(b"staged second\n", git(self.repo, "show", ":one.txt"))

    def test_inventory_excludes_controller_and_authentication_directory_contents(self):
        protected = [self.repo / ".jenkins/users/admin/config.xml", self.repo / "secrets/named-without-secret-suffix.json",
                     self.repo / ".git-credentials"]
        for path in protected:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(b"synthetic credential canary\n")
            write_git(self.repo, ["add", "-f", "--", path.relative_to(self.repo).as_posix()])
        write_git(self.repo, ["commit", "-m", "synthetic protected canaries"])
        for path in protected:
            path.write_bytes(b"changed synthetic canary\n")
        (self.repo / "one.txt").write_bytes(b"ordinary changed source\n")
        original = file_fact
        def prohibit(path):
            if Path(path) in protected:
                raise AssertionError("inventory attempted protected content read")
            return original(path)
        with mock.patch("tools.jenkins.governance.inventory.file_fact", side_effect=prohibit):
            observation = repository_fact(self.repo, content_hashes=True)
        self.assertFalse(observation["errors"])
        self.assertEqual({"one.txt"}, set(observation["sourceFacts"]))

    def test_metadata_checkpoint_survives_repository_observation_interruption(self):
        checkpoint_path = self.root / "retained-metadata.json"
        def retain(phase, value):
            if phase == "metadata":
                checkpoint_path.write_bytes(encoded(value))
        with mock.patch("tools.jenkins.governance.inventory.repository_fact", side_effect=RuntimeError("fixture interruption")):
            with self.assertRaises(RuntimeError):
                scan([self.repo], checkpoint=retain)
        retained = json.loads(checkpoint_path.read_text(encoding="utf-8"))
        self.assertEqual([str(self.repo)], retained["repositoryRoots"])
        self.assertGreater(retained["files"], 0)
        self.assertEqual([], retained["repositories"])

    def resume_fixture(self):
        self.authority.runtime["taskId"] = "fixture-resume"
        report_root = self.root / "reports"
        original = report_root / "original-scan"
        (original / "native-scan").mkdir(parents=True)
        self.state.put("governance_capability", "fixture-capsule", {"status": "passed"})
        request = {"action": "scan", "taskId": "fixture-resume", "operationId": "original-scan",
                   "requestId": "fixture-original-request", "attemptId": "fixture-original-attempt"}
        self.state.put_operation("host_governance", "original-scan", {"request": request,
            "driverDigest": "fixture-driver", "capsuleDigest": "fixture-capsule"}, status="blocked")
        native = {"rootExitCode": 1223, "processTree": {"terminal": True, "nativeWaiterJoined": True, "pipeEOF": True},
                  "terminalErrors": [], "nativeJobEvidence": {"errors": [], "records": {"afterClose": {"activeProcesses": 0}}}}
        (original / "native-scan/native-receipt.json").write_bytes(encoded(native))
        lines = []
        for root in (Path("C:/"), Path("D:/"), Path("E:/")):
            path = original / "checkpoints" / root.drive.rstrip(":").lower() / "metadata.json"
            path.parent.mkdir(parents=True)
            statistics = {"directories": 1, "files": 2, "logicalBytes": 18, "errors": 0}
            value = {"schemaVersion": 3, "roots": [str(root)], "startedAt": 1, "metadataFinishedAt": 2,
                "rootCoverage": {str(root): statistics}, "repositoryRoots": [str(self.repo)] if root.drive == "E:" else [],
                "repositories": [], "links": [], "errors": [], "sourceCandidates": [], "outputCandidates": [],
                "directories": 1, "files": 2, "logicalBytes": 18}
            path.write_bytes(encoded(value))
            lines.append(encoded({"phase": "metadata-checkpoint", "root": str(root), "reference": file_fact(path)}))
        (original / "native-scan/stdout.bin").write_bytes(b"\n".join(lines))
        return report_root, original

    def test_resume_binds_original_authority_native_and_metadata(self):
        from .resumption import prepare
        report_root, original = self.resume_fixture()
        payload = {"resumeOperationId": "original-scan"}
        value = prepare(self.authority, payload, report_root)
        self.assertEqual("fixture-original-request", value["originalRequestId"])
        self.assertEqual("fixture-original-attempt", value["originalAttemptId"])
        self.assertEqual(3, len(value["checkpoints"]))
        self.authority.runtime["taskId"] = "other-task"
        with self.assertRaisesRegex(ValueError, "authority_mismatch"):
            prepare(self.authority, payload, report_root)
        self.authority.runtime["taskId"] = "fixture-resume"
        metadata = original / "checkpoints/e/metadata.json"
        previous = metadata.read_bytes()
        metadata.write_bytes(previous + b" ")
        with self.assertRaisesRegex(ValueError, "checkpoint_changed"):
            prepare(self.authority, payload, report_root)
        (original / "native-scan/native-receipt.json").write_bytes(encoded({"processTree": {"terminal": False}}))
        with self.assertRaisesRegex(ValueError, "termination_unproved"):
            prepare(self.authority, payload, report_root)

    def test_resume_triage_defers_source_hashes_and_persists_fresh_git(self):
        from .resumption import observe, prepare
        report_root, _ = self.resume_fixture()
        specification = prepare(self.authority, {"resumeOperationId": "original-scan"}, report_root)
        (self.repo / "new-binary-source.bin").write_bytes(bytes(range(256)))
        checkpoints = self.root / "resumed-checkpoints"
        checkpoints.mkdir()
        events = []
        with mock.patch("tools.jenkins.governance.inventory.file_fact", side_effect=AssertionError("triage read source contents")):
            reports = observe(specification, [Path("C:/"), Path("D:/"), Path("E:/")], {}, checkpoints, events.append)
        facts = reports[2]["repositories"]
        self.assertEqual(1, len(facts))
        self.assertEqual("main", facts[0]["primaryBranch"])
        self.assertIn("indexDigest", facts[0])
        self.assertNotIn("sourceFacts", facts[0])
        self.assertEqual(["new-binary-source.bin"], [item["path"] for item in facts[0]["changes"]])
        self.assertEqual(1, events[-1]["completed"])
        self.assertTrue((checkpoints / "e/complete.json").exists())
        from .scan_child import combine
        combined = combine([Path("C:/"), Path("D:/"), Path("E:/")], reports, 10)
        self.assertEqual(1, combined["startedAt"])
        from .scopes import inspect
        self.assertEqual(hashlib.sha256(bytes(range(256))).hexdigest(), inspect(self.repo, ["new-binary-source.bin"])["hashes"]["new-binary-source.bin"])

    def test_resume_rejects_repository_outside_checkpoint_volume(self):
        from .resumption import load_checkpoint
        _, original = self.resume_fixture()
        path = original / "checkpoints/c/metadata.json"
        value = json.loads(path.read_text(encoding="utf-8"))
        value["repositoryRoots"] = [str(self.repo)]
        path.write_bytes(encoded(value))
        with self.assertRaisesRegex(ValueError, "outside_observed_volume"):
            load_checkpoint(file_fact(path), Path("C:/"))

    def test_authentication_directory_repository_is_not_git_inspected(self):
        with mock.patch("tools.jenkins.governance.inventory.git", side_effect=AssertionError("authentication root read")):
            value = repository_fact(self.root / "secrets" / "repository", content_hashes=True)
        self.assertEqual(["authentication-directory-root-not-inspected"], value["errors"])

    def test_status_partition_preserves_deletion_rename_binary_and_index(self):
        from .inventory import _git_command, STATUS_COMMAND_UNITS
        (self.repo / "gone").mkdir()
        (self.repo / "gone/deleted.txt").write_bytes(b"retained deletion\n")
        write_git(self.repo, ["add", "gone/deleted.txt"])
        write_git(self.repo, ["commit", "-m", "retain a staged deletion input"])
        write_git(self.repo, ["rm", "gone/deleted.txt"])
        (self.repo / "moved").mkdir()
        write_git(self.repo, ["mv", "foreign.txt", "moved/new.txt"])
        (self.repo / "one.txt").write_bytes(b"staged version\n")
        write_git(self.repo, ["add", "one.txt"])
        (self.repo / "one.txt").write_bytes(b"working version\n")
        (self.repo / "untracked.bin").write_bytes(bytes(range(256)))
        excluded = []
        for index in range(650):
            directory = self.repo / "bulk" / ("group-" + str(index).zfill(4))
            directory.mkdir(parents=True)
            (directory / "source.txt").write_bytes(b"source\n")
            # Missing links model metadata that disappeared after traversal.
            excluded.append(directory.relative_to(self.repo).as_posix() + "/link-placeholder")
        previous_index = (self.repo / ".git/index").read_bytes()
        whole = repository_fact(self.repo)
        self.assertFalse(whole["errors"])
        with mock.patch("tools.jenkins.governance.inventory.git", wraps=git) as observed:
            bounded = repository_fact(self.repo, excluded_paths=excluded)
        self.assertFalse(bounded["errors"])
        status_calls = [call for call in observed.call_args_list if len(call.args) > 1 and call.args[1] == "status"]
        self.assertGreater(len(status_calls), 1)
        self.assertFalse(any(len(call.args) > 1 and call.args[1] == "ls-tree" for call in observed.call_args_list))
        for call in status_calls:
            command = subprocess.list2cmdline(_git_command(*call.args))
            self.assertLessEqual(len(command.encode("utf-16-le", "surrogatepass")) // 2, STATUS_COMMAND_UNITS)
        self.assertEqual(whole["changes"], bounded["changes"])
        self.assertEqual(whole["statusDigest"], bounded["statusDigest"])
        self.assertEqual(whole["indexDigest"], bounded["indexDigest"])
        self.assertEqual(previous_index, (self.repo / ".git/index").read_bytes())
        changes = {item["path"]: item["status"] for item in bounded["changes"]}
        self.assertEqual("D ", changes["gone/deleted.txt"])
        self.assertEqual("D ", changes["foreign.txt"])
        self.assertEqual("A ", changes["moved/new.txt"])
        self.assertEqual("MM", changes["one.txt"])
        self.assertEqual("??", changes["untracked.bin"])
        from .scopes import inspect
        inputs = inspect(self.repo, ["untracked.bin", "gone/deleted.txt"])
        self.assertEqual(hashlib.sha256(bytes(range(256))).hexdigest(), inputs["hashes"]["untracked.bin"])
        self.assertIsNone(inputs["hashes"]["gone/deleted.txt"])

    def test_closure_requires_explicit_response(self):
        with self.assertRaises(ValueError):
            self.authority.close_round({"roundId": "fixture-round", "replyDigest": digest("")})
        self.assertEqual("awaiting-user-claims", self.authority.get("round", "fixture-round")["status"])
        with self.assertRaises(ValueError):
            self.authority.record_response({"roundId": "fixture-round", "userReply": "  "})
        self.close()
        with self.assertRaises(ValueError):
            self.claim()

    def test_duplicate_and_overlapping_claims(self):
        original = self.claim()
        self.assertEqual(original, self.claim())
        with self.assertRaises(ValueError):
            self.claim(session="fixture-b")
        with self.assertRaises(ValueError):
            self.authority.claim({"roundId": "fixture-round", "repositoryId": self.repository_id,
                "ownerSessionId": "bad/owner", "paths": ["one.txt"], "expectedHashes": {}})

    def test_directory_claim_preserves_binary_and_missing_tracked_source(self):
        from .scopes import inspect
        source = self.repo / "src"
        source.mkdir()
        (source / "deleted.txt").write_bytes(b"tracked old bytes\r\n")
        write_git(self.repo, ["add", "src/deleted.txt"])
        write_git(self.repo, ["commit", "-m", "fixture directory baseline"])
        (source / "deleted.txt").unlink()
        (source / "new.bin").write_bytes(b"untracked directory bytes\x00")
        inputs = inspect(self.repo, ["src"])
        claim = self.authority.claim({"repositoryId": self.repository_id, "ownerSessionId": "directory-owner",
            "roundId": "fixture-round", "paths": ["src"], "expectedHashes": inputs["hashes"]})
        self.assertEqual(["src"], claim["paths"])
        with self.assertRaises(ValueError):
            self.claim(session="other", paths=["src/new.bin"])
        self.close()
        observed = repository_fact(self.repo)
        scratch = self.root / "directory-private-index"
        scratch.mkdir()
        sealed = seal(self.authority, {"roundId": "fixture-round", "inventoryDigest": self.round["inventoryDigest"],
            "repositoryId": self.repository_id, "ownerSessionId": "directory-owner", "operationId": "directory-seal",
            "expectedHead": observed["head"], "expectedStatusDigest": observed["statusDigest"],
            "expectedIndexDigest": observed["indexDigest"],
            "paths": ["src"], "expectedHashes": inputs["hashes"]}, scratch)
        self.assertEqual(git(self.repo, "cat-file", "blob", sealed["branch"] + ":src/new.bin"), b"untracked directory bytes\x00")
        self.assertNotIn("src/deleted.txt", text_git(self.repo, "ls-tree", "-r", "--name-only", sealed["branch"]))
        (source / ".env.production").write_bytes(b"fixture credential bytes")
        with self.assertRaises(ValueError):
            inspect(self.repo, ["src"])

    def test_non_toplevel_directory_cannot_be_registered_as_checkout(self):
        nested = self.repo / "source-subdirectory"
        nested.mkdir()
        fact = repository_fact(nested)
        self.assertTrue(fact["errors"])
        with self.assertRaises(ValueError):
            self.authority.register({"canonicalCheckout": str(nested), "buildNamespace": str(self.root / "compiler-nested")})

    def test_inventory_registration_preserves_related_clones_and_recipe_cas(self):
        from .registration import run, configure
        clone = self.root / "independent-clone"
        write_git(self.repo, ["clone", "--no-hardlinks", str(self.repo), str(clone)])
        observation = scan([self.root])
        self.inventory.write_bytes(encoded(observation))
        self.authority.put("inventory", "fixture-bulk", {"inventoryRef": file_fact(self.inventory),
                           "inventoryDigest": observation["inventoryDigest"]})
        report = self.root / "registration-report"
        report.mkdir()
        value = run(self.authority, {"inventoryId": "fixture-bulk", "operationId": "fixture-register-bulk"}, report)
        self.assertEqual(1, value["registeredCount"])
        self.assertEqual(1, value["reviewCount"])
        self.assertTrue(clone.exists())
        recipes = [{"argv": ["git", "diff", "--check"], "timeoutSeconds": 30}]
        configured = configure(self.authority, {"repositoryId": self.repository_id, "registrationDigest": digest(self.registration),
            "operationId": "fixture-configure", "validationRecipes": recipes}, report)
        self.assertEqual(digest(recipes), configured["validationRecipesDigest"])
        with self.assertRaises(ValueError):
            configure(self.authority, {"repositoryId": self.repository_id, "registrationDigest": digest(self.registration),
                "operationId": "fixture-configure-stale", "validationRecipes": recipes}, report)

    def test_claim_close_race_is_transactional(self):
        response = self.authority.record_response({"roundId": "fixture-round", "userReply": "fixture response"})
        def attempt_claim():
            try:
                return self.claim()
            except ValueError:
                return None
        def attempt_close():
            return self.authority.close_round({"roundId": "fixture-round", "replyDigest": response["replyDigest"]})
        with ThreadPoolExecutor(max_workers=2) as executor:
            a, b = executor.submit(attempt_claim), executor.submit(attempt_close)
            claim, closed = a.result(), b.result()
        rows = [r["payload"] for r in self.state.list("governance_claim")]
        self.assertEqual(closed["claimDigest"], digest(rows))
        self.assertEqual(len(rows), 1 if claim else 0)

    def test_repository_isolation_and_changed_hash(self):
        other = self.root / "other"
        other.mkdir()
        write_git(other, ["init", "-b", "main"])
        (other / "one.txt").write_bytes(b"other project")
        write_git(other, ["add", "one.txt"])
        write_git(other, ["commit", "-m", "another history"])
        with self.assertRaises(ValueError):
            self.authority.claim({"repositoryId": self.repository_id, "sourceCheckout": str(other), "roundId": "fixture-round",
                "ownerSessionId": "fixture-a", "paths": ["one.txt"], "expectedHashes": {"one.txt": file_fact(other / "one.txt")["sha256"]}})
        old = file_fact(self.repo / "one.txt")["sha256"]
        (self.repo / "one.txt").write_bytes(b"changed")
        with self.assertRaises(ValueError):
            self.authority.claim({"repositoryId": self.repository_id, "roundId": "fixture-round", "ownerSessionId": "fixture-a",
                "paths": ["one.txt"], "expectedHashes": {"one.txt": old}})

    def test_staged_unstaged_binary_and_untracked_are_recoverable(self):
        (self.repo / "one.txt").write_bytes(b"staged-version\n")
        write_git(self.repo, ["add", "one.txt"])
        (self.repo / "one.txt").write_bytes(b"unstaged-version\n")
        (self.repo / "new.bin").write_bytes(b"\x00\xff\x01binary")
        (self.repo / "foreign.txt").write_bytes(b"foreign-staged\n")
        write_git(self.repo, ["add", "foreign.txt"])
        before_index = git(self.repo, "ls-files", "--stage", "-z")
        self.close()
        value = self.seal(paths=["one.txt", "new.bin"])
        self.assertEqual(git(self.repo, "show", value["indexCommit"] + ":one.txt"), b"staged-version\n")
        self.assertEqual(git(self.repo, "show", value["commit"] + ":one.txt"), b"unstaged-version\n")
        self.assertEqual(git(self.repo, "show", value["commit"] + ":new.bin"), b"\x00\xff\x01binary")
        self.assertEqual(before_index, git(self.repo, "ls-files", "--stage", "-z"))
        self.assertEqual(text_git(self.repo, "rev-parse", value["branch"]), value["commit"])

    def test_seal_preserves_deletions(self):
        write_git(self.repo, ["rm", "one.txt"])
        self.close()
        value = self.seal()
        self.assertEqual(git(self.repo, "ls-tree", value["commit"], "--", "one.txt"), b"")

    def test_seal_rejects_another_claim(self):
        self.claim()
        self.close()
        with self.assertRaises(ValueError):
            self.seal(owner="maintenance")

    def test_merge_conflict_preserves_source_and_ref(self):
        (self.repo / "one.txt").write_bytes(b"from candidate\n")
        self.close()
        candidate = self.seal()
        write_git(self.repo, ["checkout", "--", "one.txt"])
        (self.repo / "one.txt").write_bytes(b"from primary\n")
        write_git(self.repo, ["add", "one.txt"])
        write_git(self.repo, ["commit", "-m", "primary diverged"])
        before = (self.repo / "one.txt").read_bytes()
        value = prepare_integration(self.authority, {"operationId": "fixture-merge", "sealedOperationId": "fixture-seal",
            "repositoryId": self.repository_id, "roundId": "fixture-round", "inventoryDigest": self.round["inventoryDigest"],
            "expectedPrimaryHead": text_git(self.repo, "rev-parse", "main")})
        self.assertEqual(value["status"], "conflict")
        self.assertEqual(before, (self.repo / "one.txt").read_bytes())
        self.assertEqual(candidate["commit"], text_git(self.repo, "rev-parse", candidate["branch"]))

    def test_legacy_driver_flags_do_not_prove_ownership(self):
        legacy = {"repositoryId": self.repository_id, "driverDigest": "retired-driver", "owner": "old-session"}
        self.state.put("management_worktree", "historical", legacy)
        with self.assertRaises(ValueError):
            self.authority.adopt_legacy_claim({"domain": "management_worktree", "key": "historical",
                "repositoryId": self.repository_id, "previousDigest": digest(legacy),
                "evidence": {"sourceHashesVerified": True, "referencesReconciled": True, "nativeTerminalVerified": True}})
        self.assertEqual(legacy, self.state.get("management_worktree", "historical")["payload"])

    def test_legacy_path_claim_requires_exact_supported_adoption(self):
        from .versioning import _reject_unadopted_legacy_path_claims
        legacy = {"repositoryId": self.repository_id, "owner": "retired-session", "paths": ["one.txt"],
                  "pathRoot": str(self.repo), "sourceHashes": {"one.txt": file_fact(self.repo / "one.txt")["sha256"]},
                  "retainedRefs": [{"ref": "refs/heads/main", "sha": text_git(self.repo, "rev-parse", "refs/heads/main")}],
                  "nativeJobId": "legacy-terminal", "requestId": "legacy-request", "attemptId": "legacy-attempt",
                  "driverDigest": "retired-driver"}
        self.state.put("path_claims", "legacy-owner", legacy, expected_version=0)
        with self.assertRaisesRegex(ValueError, "legacy_path_claim_requires_verified_adoption"):
            _reject_unadopted_legacy_path_claims(self.authority, self.repository_id, ["one.txt"])
        self.state.put("native_job", "legacy-terminal", {"repositoryId": self.repository_id, "status": "terminal",
            "completeProof": {"complete": True}, "requestId": "legacy-request", "attemptId": "legacy-attempt"},
            expected_version=0)
        self.authority.adopt_legacy_claim({"domain": "path_claims", "key": "legacy-owner",
            "repositoryId": self.repository_id, "previousDigest": digest(legacy), "evidence": {}})
        _reject_unadopted_legacy_path_claims(self.authority, self.repository_id, ["one.txt"])
        current = self.state.get("path_claims", "legacy-owner")
        self.state.put("path_claims", "legacy-owner", {**legacy, "owner": "changed-after-adoption"},
                       expected_version=current["version"])
        with self.assertRaisesRegex(ValueError, "legacy_path_claim_requires_verified_adoption"):
            _reject_unadopted_legacy_path_claims(self.authority, self.repository_id, ["one.txt"])


    def test_checked_out_integration_preserves_foreign_staging(self):
        from .versioning import finish_integration
        self.claim(session="foreign-session", paths=["foreign.txt"])
        (self.repo / "foreign.txt").write_bytes(b"foreign-owned-stage\n")
        write_git(self.repo, ["add", "foreign.txt"])
        foreign_before = git(self.repo, "ls-files", "--stage", "--", "foreign.txt")
        (self.repo / "one.txt").write_bytes(b"maintenance version\n")
        self.close()
        sealed = self.seal()
        before_head = text_git(self.repo, "rev-parse", "main")
        integration = prepare_integration(self.authority, {"operationId": "fixture-integration", "sealedOperationId": "fixture-seal",
            "repositoryId": self.repository_id, "roundId": "fixture-round", "inventoryDigest": self.round["inventoryDigest"],
            "expectedPrimaryHead": before_head})
        self.assertEqual("validation-required", integration["status"])
        self.authority.put("validation", "fixture-validation", {"status": "passed", "repositoryId": self.repository_id,
            "resultTree": integration["resultTree"]})
        payload = {"operationId": "fixture-finish", "roundId": "fixture-round", "inventoryDigest": self.round["inventoryDigest"],
            "integrationOperationId": "fixture-integration", "validationOperationId": "fixture-validation",
            "expectedCanonicalHashes": {"one.txt": file_fact(self.repo / "one.txt")["sha256"]}}
        with self.assertRaises(ValueError):
            finish_integration(self.authority, payload)
        self.assertEqual(before_head, text_git(self.repo, "rev-parse", "main"))
        # Explicit test State fixture models the terminal writer contract; this is not a production native receipt.
        self.state.put("native_job", "fixture-terminal-writer", {"repositoryId": self.repository_id, "status": "terminal",
            "completeProof": {"complete": True}, "writeScopes": [str(self.repo)], "origin": "unit-test-model"})
        value = finish_integration(self.authority, {**payload, "writerJobIds": ["fixture-terminal-writer"]})
        self.assertEqual(value["status"], "integrated")
        self.assertEqual(foreign_before, git(self.repo, "ls-files", "--stage", "--", "foreign.txt"))
        self.assertEqual(git(self.repo, "show", "main:one.txt"), b"maintenance version\n")
        self.assertEqual((self.repo / "foreign.txt").read_bytes(), b"foreign-owned-stage\n")

    def test_patch_handoff_is_limited_to_claimed_branch(self):
        import base64
        from .versioning import apply_patch
        claim = self.claim()
        self.close()
        sealed = self.seal(owner="fixture-a")
        inbox = self.repo / ".codex/outbox/jenkins-inbox/fixture-a"
        inbox.mkdir(parents=True)
        handoff = inbox / "patch.json"
        data = b"new claimed bytes\x00"
        content = {"claimId": claim["claimId"], "baseHead": sealed["commit"], "files": [{"path": "one.txt",
            "beforeSha256": hashlib.sha256(b"base\n").hexdigest(), "afterSha256": hashlib.sha256(data).hexdigest(),
            "contentBase64": base64.b64encode(data).decode()}]}
        handoff.write_bytes(encoded(content))
        head = text_git(self.repo, "rev-parse", "main")
        value = apply_patch(self.authority, {"operationId": "fixture-patch", "claimId": claim["claimId"],
            "sealedOperationId": "fixture-seal", "repositoryId": self.repository_id, "ownerSessionId": "fixture-a",
            "roundId": "fixture-round", "inventoryDigest": self.round["inventoryDigest"], "handoffPath": str(handoff),
            "handoffSha256": file_fact(handoff)["sha256"]}, self.root)
        self.assertEqual(head, text_git(self.repo, "rev-parse", "main"))
        self.assertEqual(data, git(self.repo, "show", value["branch"] + ":one.txt"))
        self.assertEqual((self.repo / "one.txt").read_bytes(), b"base\n")

    def test_interrupted_index_publication_reconciles_original_request(self):
        from .versioning import finish_integration, recover_integration
        from tools.jenkins.processes.identity import current_identity
        (self.repo / "one.txt").write_bytes(b"exact canonical CRLF bytes\r\n")
        (self.repo / "foreign.txt").write_bytes(b"foreign staged bytes\r\n")
        write_git(self.repo, ["add", "foreign.txt"])
        foreign = git(self.repo, "ls-files", "--stage", "--", "foreign.txt")
        original_source = (self.repo / "one.txt").read_bytes()
        original_index = (self.repo / ".git/index").read_bytes()
        original_staged = git(self.repo, "cat-file", "blob", ":foreign.txt")
        self.close()
        sealed = self.seal()
        integrated = prepare_integration(self.authority, {"operationId": "fixture-integration", "sealedOperationId": "fixture-seal",
            "repositoryId": self.repository_id, "roundId": "fixture-round", "inventoryDigest": self.round["inventoryDigest"],
            "expectedPrimaryHead": text_git(self.repo, "rev-parse", "main")})
        self.authority.put("validation", "fixture-validation", {"status": "passed", "repositoryId": self.repository_id,
            "resultTree": integrated["resultTree"]})
        self.state.put("native_job", "fixture-writer", {"repositoryId": self.repository_id, "status": "terminal",
            "completeProof": {"complete": True}, "writeScopes": [str(self.repo)], "origin": "unit-test-model"})
        payload = {"operationId": "fixture-original-finish", "roundId": "fixture-round", "inventoryDigest": self.round["inventoryDigest"],
            "integrationOperationId": "fixture-integration", "validationOperationId": "fixture-validation",
            "writerJobIds": ["fixture-writer"], "expectedCanonicalHashes": {"one.txt": file_fact(self.repo / "one.txt")["sha256"]}}
        original = self.state.put_operation("host_governance", "fixture-original-finish", {"request": {"action": "finish-integration", "payload": payload}}, "running")
        child = subprocess.Popen([sys.executable, "-B", "-c", "import sys;print('READY',flush=True);sys.stdin.readline()"],
                                 stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        try:
            self.assertEqual("READY", child.stdout.readline().strip())
            self.authority.runtime["processIdentity"] = current_identity(child.pid).to_dict()
            replace = os.replace
            def interrupted(source, destination):
                if Path(destination) == self.repo / ".git/index":
                    raise OSError("fixture publication interruption")
                return replace(source, destination)
            with mock.patch("tools.jenkins.governance.versioning.os.replace", side_effect=interrupted):
                with self.assertRaises(OSError):
                    finish_integration(self.authority, payload)
            transition = self.authority.get("source_transition", "fixture-original-finish")
            self.assertEqual("index-pending", transition["status"])
            retained = transition["preimageRef"]
            self.assertEqual(original_source, git(self.repo, "cat-file", "blob", retained + ":source/one.txt"))
            self.assertEqual(original_index, git(self.repo, "cat-file", "blob", retained + ":shared-index"))
            self.assertEqual(original_staged, git(self.repo, "cat-file", "blob", retained + ":staged/foreign.txt"))
            request = {"repositoryId": self.repository_id, "roundId": "fixture-round", "inventoryDigest": self.round["inventoryDigest"],
                "operationId": "fixture-recovery", "originalOperationId": "fixture-original-finish", "originalIntentDigest": original["payloadDigest"]}
            with self.assertRaises(ValueError):
                recover_integration(self.authority, request)
        finally:
            child.communicate("close\n", timeout=15)
        # A second interruption after the guarded write must reconcile its
        # recorded physical index, retaining the original exclusion lock.
        with mock.patch("tools.jenkins.governance.guarded_delete.remove_verified", side_effect=OSError("fixture lock retirement interruption")):
            with self.assertRaises(OSError):
                recover_integration(self.authority, request)
        self.assertTrue((self.repo / ".git/index.lock").exists())
        self.assertEqual(foreign, git(self.repo, "ls-files", "--stage", "--", "foreign.txt"))
        value = recover_integration(self.authority, request)
        self.assertEqual("reconciled", value["status"])
        self.assertEqual("reconciled", self.state.get_operation("fixture-original-finish")["status"])
        self.assertFalse((self.repo / ".git/index.lock").exists())
        self.assertEqual(foreign, git(self.repo, "ls-files", "--stage", "--", "foreign.txt"))
        self.assertEqual("reconciled", recover_integration(self.authority, request)["status"])

    def test_permission_plan_has_no_source_or_pool_write_grants(self):
        import tomllib
        from .permissions import plan
        with mock.patch("tools.jenkins.governance.worker._json"):
            value = plan(self.authority, {"operationId": "fixture-permission-plan", "repositoryId": self.repository_id,
                "ownerSessionId": "fixture-a", "expectedConfigHash": None}, self.root)
        parsed = tomllib.loads(value["profileToml"])["permissions"][value["profile"]]
        grants = {path for path, access in parsed["filesystem"].items() if access == "write"}
        self.assertEqual(grants, {Path(value["handoffRoot"]).as_posix(), Path(value["temporaryRoot"]).as_posix()})
        self.assertFalse(parsed["network"]["enabled"])
        self.assertFalse(value["machineEnforcement"])
        default = tomllib.loads(value["defaultProfileToml"])["permissions"][value["defaultProfile"]]
        self.assertNotEqual(value["defaultProfile"], value["profile"])
        self.assertNotIn("write", default["filesystem"].values())

    def test_original_operation_survives_retry(self):
        original = self.state.put_operation("host_governance", "original-operation", {"requestId": "same", "attemptId": "one"})
        self.state.transition_operation("original-operation", "prepared", "running", {"native": "pending"})
        again = self.state.put_operation("host_governance", "original-operation", original["payload"])
        self.assertEqual(again["status"], "running")
        with self.assertRaises(Exception):
            self.state.put_operation("host_governance", "original-operation", {"requestId": "same", "attemptId": "two"})

    def test_primary_selection_is_unambiguous(self):
        write_git(self.repo, ["branch", "master"])
        self.assertIsNone(resolve_primary(self.repo))
        write_git(self.repo, ["symbolic-ref", "refs/remotes/origin/HEAD", "refs/remotes/origin/main"])
        self.assertEqual(resolve_primary(self.repo), "main")
        self.assertEqual(resolve_primary(self.repo, "master"), "master")

    def test_inventory_hidden_bare_git_and_read_failures(self):
        hidden = self.root / ".hidden"
        hidden.mkdir()
        (hidden / "Cargo.toml").write_text("[package]\nname='fixture'\nversion='0.0.0'\n")
        (hidden / "auth.json").write_text("fixture secret must not be read")
        bare = self.root / "bare.git"
        bare.mkdir()
        write_git(bare, ["init", "--bare"])
        observation = scan([self.root])
        self.assertTrue(any(item["path"] == str(hidden) for item in observation["sourceCandidates"]))
        self.assertTrue(any(item["bare"] for item in observation["repositories"]))
        self.assertFalse(any(item["path"].endswith("repo\\.git") for item in observation["repositories"]))
        original = os.scandir
        def denied(path):
            if Path(path) == hidden:
                raise PermissionError("fixture denial")
            return original(path)
        with mock.patch("os.scandir", side_effect=denied):
            partial = scan([self.root])
        self.assertEqual(partial["status"], "partial")
        self.assertTrue(any(item["path"] == str(hidden) for item in partial["errors"]))
        original_fact = repository_fact
        def disappeared(root, *args, **kwargs):
            if root == self.repo:
                raise ValueError("fixture repository disappeared after traversal")
            return original_fact(root, *args, **kwargs)
        with mock.patch("tools.jenkins.governance.inventory.repository_fact", side_effect=disappeared):
            missing = scan([self.root])
        self.assertEqual("partial", missing["status"])
        self.assertTrue(any(item["path"] == str(self.repo) and item["errors"] for item in missing["repositories"]))

    def test_git_observation_excludes_authentication_before_refresh(self):
        # Explicit synthetic authentication data; no real credential is read.
        credentials = self.repo / "auth.json"
        credentials.write_bytes(b"synthetic fixture credential")
        write_git(self.repo, ["add", "auth.json"])
        write_git(self.repo, ["commit", "-m", "synthetic fixture authentication"])
        credentials.write_bytes(b"changed synthetic fixture credential")
        (self.repo / "one.txt").write_bytes(b"normal source change")
        value = repository_fact(self.repo, content_hashes=True)
        self.assertFalse(value["errors"])
        self.assertNotIn("auth.json", value["sourceFacts"])
        self.assertIn("one.txt", value["sourceFacts"])
        self.assertEqual("excluded-before-Git-refresh", value["authenticationContentInspection"])

    def test_source_cleanup_requires_retained_bytes_for_ignored_files(self):
        from .cleanup import plan, apply
        extra = self.root / "extra-worktree"
        write_git(self.repo, ["worktree", "add", "--detach", str(extra), "HEAD"])
        (self.repo / ".git/info/exclude").write_bytes(b"unknown.tmp\n")
        unknown = extra / "unknown.tmp"
        unknown.write_bytes(b"unique ignored source data")
        self.close()
        observed = repository_fact(extra)
        scratch = self.root / "source-private-index"
        scratch.mkdir()
        sealed = seal(self.authority, {"roundId": self.round["roundId"], "inventoryDigest": self.round["inventoryDigest"],
            "repositoryId": self.repository_id, "ownerSessionId": "maintenance", "operationId": "fixture-source-seal",
            "sourceCheckout": str(extra), "paths": ["one.txt"], "expectedHead": observed["head"],
            "expectedStatusDigest": observed["statusDigest"], "expectedIndexDigest": observed["indexDigest"],
            "expectedHashes": {"one.txt": file_fact(extra / "one.txt")["sha256"]}}, scratch)
        self.state.put("native_job", "fixture-source-writer", {"status": "terminal", "completeProof": {"complete": True},
            "repositoryId": self.repository_id, "writeScopes": [str(extra)], "origin": "unit-test-model"})
        request = {"repositoryId": self.repository_id, "roundId": self.round["roundId"], "inventoryDigest": self.round["inventoryDigest"],
            "operationId": "fixture-source-cleanup", "target": str(extra), "kind": "worktree",
            "sealedOperationId": "fixture-source-seal", "releasedRetentionRefs": [], "writerJobIds": ["fixture-source-writer"],
            "files": ["unknown.tmp"], "expectedHashes": {"unknown.tmp": file_fact(unknown)["sha256"]}, "maxLogicalBytes": 10000}
        with self.assertRaisesRegex(ValueError, "source_file_requires_exact_retained_Git_bytes"):
            plan(self.authority, request, self.root)
        request.update(files=["one.txt"], expectedHashes={"one.txt": file_fact(extra / "one.txt")["sha256"]})
        prepared = plan(self.authority, request, self.root)
        receipt = apply(self.authority, {**request, "planId": request["operationId"], "planDigest": prepared["planDigest"]}, self.root)
        self.assertEqual("complete", receipt["status"])
        self.assertFalse((extra / "one.txt").exists())
        self.assertEqual(b"unique ignored source data", unknown.read_bytes())

    def test_audit_distinguishes_candidates_and_ownership(self):
        from .audit import run, admission
        outside = self.repo / "build"
        outside.mkdir()
        (outside / "generated.obj").write_bytes(b"fixture output")
        observation = scan([self.root])
        self.inventory.write_bytes(encoded(observation))
        self.authority.put("inventory", "fixture-scan", {"inventoryRef": file_fact(self.inventory),
                           "inventoryDigest": observation["inventoryDigest"]})
        self.state.put("path_claims", "historical-owner", {"repositoryId": self.repository_id,
                                                        "path": "foreign.txt", "owner": "retired-session"})
        report = self.root / "audit-report"
        report.mkdir()
        result = run(self.authority, {"operationId": "fixture-audit", "inventoryId": "fixture-scan"}, report)
        self.assertEqual("awaiting-user-claims", result["status"])
        # The fixture is inside the permitted E root. That does not establish
        # ownership, regenerability, or authorization to remove it.
        review = json.loads((report / "inventory-review.json").read_text(encoding="utf-8"))
        self.assertTrue(any(item["path"] == str(outside) and item["status"] == "needs-review"
                            for item in review["outputCandidates"]))
        self.assertEqual(1, result["summary"]["legacyOwners"])
        self.assertTrue((outside / "generated.obj").exists())
        roots = admission(self.authority, {"payload": {"repositoryId": self.repository_id}})
        self.assertEqual("checked", roots["status"])
        self.assertFalse(roots["machineWideWriterConfinement"])
        self.inventory.write_bytes(b"changed")
        with self.assertRaises(ValueError):
            run(self.authority, {"operationId": "fixture-audit-changed", "inventoryId": "fixture-scan"}, report)

    def test_scheduled_cleanup_restart_binding_requires_consistent_owners(self):
        from copy import deepcopy
        from tools.dev.jenkins_cleanup_runtime import binding_from_records
        checksum = "a" * 64
        sealed = self.repo / ".jenkins/runtime/drivers" / checksum
        selected = {"driverDigest": checksum, "root": str(sealed),
                    "steps": str(sealed / "pipeline/steps.groovy"),
                    "launcher": str(sealed / "launcher.py"), "runtimeOperationId": "restarted-runtime"}
        def owner(pid):
            return {"pid": pid, "creationTime": str(134000000000000000 + pid),
                    "executable": str(self.repo / "runtime.exe"), "commandDigest": "b" * 64}
        operation = {**owner(101), "operationId": "restarted-runtime", "generation": "restarted-runtime",
                     "state": "running", "health": True}
        host = {"operationId": "restarted-runtime", "generation": "restarted-runtime",
                "hostPid": 101, "status": "running", "controllerIdentity": owner(102),
                "agentIdentity": owner(103)}
        records = {"selected": selected, "operation": operation, "host": host}
        bound = binding_from_records(self.repo, checksum, **records)
        self.assertEqual("restarted-runtime", bound["runtimeOperationId"])
        self.assertEqual({101, 102, 103}, {v["pid"] for v in bound["nativeOwners"].values()})
        for section, field, value in (
                ("selected", "driverDigest", "c" * 64),
                ("selected", "steps", str(self.repo / "unreviewed.groovy")),
                ("selected", "runtimeOperationId", "../unreviewed-runtime"),
                ("operation", "operationId", "old-runtime"),
                ("operation", "generation", "old-runtime"),
                ("operation", "state", "stopped"),
                ("operation", "health", False),
                ("host", "operationId", "old-runtime"),
                ("host", "generation", "old-runtime"),
                ("host", "hostPid", 999),
                ("host", "status", "stopped"),
                ("host", "agentIdentity", {**owner(103), "creationTime": ""}),
                ("host", "agentIdentity", owner(102))):
            changed = deepcopy(records)
            changed[section][field] = value
            with self.subTest(section=section, field=field, value=value):
                with self.assertRaises(ValueError):
                    binding_from_records(self.repo, checksum, **changed)

    def test_scheduled_cleanup_native_binding_checks_installed_hashes(self):
        repo = Path(__file__).resolve().parents[3]
        source = repo / "tools/jenkins-management/zircon-scheduled-cleanup.groovy"
        delimiter = "label: 'Bind new cleanup run to verified native runtime', script: " + chr(39) * 3
        script = source.read_text(encoding="utf-8-sig").split(delimiter, 1)[1].split(chr(39) * 3, 1)[0]
        selected = json.loads((repo / ".jenkins/state/deployment/driver.json").read_text())
        helper = repo / "tools/dev/jenkins_cleanup_runtime.py"
        environment = {**os.environ, "ZIRCON_REPO_ROOT": str(repo),
                       "ZIRCON_DRIVER_DIGEST": selected["driverDigest"], "JENKINS_PYTHON": sys.executable,
                       "ZIRCON_CLEANUP_BINDING_SHA256": file_fact(helper)["sha256"],
                       "ZIRCON_CLEANUP_PYTHON_SHA256": file_fact(Path(sys.executable))["sha256"]}
        def run(env):
            return subprocess.run(["powershell.exe", "-NoProfile", "-NonInteractive", "-Command", script],
                                  env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=90)
        result = run(environment)
        self.assertEqual(0, result.returncode, result.stderr.decode("utf-8", errors="replace"))
        proof = json.loads(result.stdout.decode("utf-8-sig"))
        self.assertEqual(selected["runtimeOperationId"], proof["runtimeOperationId"])
        self.assertEqual(proof["nativeOwners"]["controller"]["pid"], proof["endpoint"]["pid"])
        self.assertEqual(1, proof["endpoint"]["ownerCount"])
        self.assertTrue(proof["verifiedAt"])
        for key in ("ZIRCON_CLEANUP_BINDING_SHA256", "ZIRCON_CLEANUP_PYTHON_SHA256", "ZIRCON_DRIVER_DIGEST"):
            with self.subTest(changed=key):
                result = run({**environment, key: "0" * 64})
                self.assertNotEqual(0, result.returncode)
                self.assertFalse(result.stdout.strip())

    def test_scheduled_cleanup_runtime_stays_fixed_after_run_start(self):
        repo = Path(__file__).resolve().parents[3]
        source = repo / "tools/jenkins-management/zircon-scheduled-cleanup.groovy"
        delimiter = "label: 'Verify selected maintenance runtime', script: " + chr(39) * 3
        script = source.read_text(encoding="utf-8-sig").split(delimiter, 1)[1].split(chr(39) * 3, 1)[0]
        record = self.repo / ".jenkins/state/deployment/driver.json"
        record.parent.mkdir(parents=True)
        selected = {"driverDigest": "a" * 64, "runtimeOperationId": "runtime-at-start",
                    "steps": "fixed-steps", "launcher": "fixed-launcher"}
        record.write_bytes(encoded(selected))
        environment = {**os.environ, "ZIRCON_REPO_ROOT": str(self.repo),
                       "ZIRCON_DRIVER_DIGEST": selected["driverDigest"],
                       "ZIRCON_RUNTIME_OPERATION_ID": selected["runtimeOperationId"],
                       "ZIRCON_SEALED_DRIVER": selected["steps"], "ZIRCON_DRIVER_LAUNCHER": selected["launcher"]}
        def run():
            return subprocess.run(["powershell.exe", "-NoProfile", "-NonInteractive", "-Command", script],
                                  env=environment, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=60)
        result = run()
        self.assertEqual(0, result.returncode, result.stderr.decode("utf-8", errors="replace"))
        record.write_bytes(encoded({**selected, "runtimeOperationId": "restart-during-run"}))
        result = run()
        self.assertNotEqual(0, result.returncode)
        self.assertIn("Scheduled cleanup driver/runtime changed", result.stderr.decode("utf-8", errors="replace"))

    def test_scheduled_cleanup_xml11_readback(self):
        source = Path(__file__).resolve().parents[3] / "tools/dev/install-jenkins-scheduled-cleanup.ps1"
        environment = {**os.environ, "ZGOV_INSTALLER_SOURCE": str(source)}
        script = r'''
$ErrorActionPreference = 'Stop'
$tokens = $null
$parseErrors = $null
$tree = [Management.Automation.Language.Parser]::ParseFile($env:ZGOV_INSTALLER_SOURCE, [ref]$tokens, [ref]$parseErrors)
if ($parseErrors.Count) { throw 'Installer source has syntax errors' }
$definition = $tree.Find({ param($node) $node -is [Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -eq 'Read-JenkinsJobXml' }, $true)
if (-not $definition) { throw 'Installer XML reader is missing' }
. ([scriptblock]::Create($definition.Extent.Text))
$document = Read-JenkinsJobXml '<?xml version="1.1" encoding="UTF-8"?><flow-definition><definition><script>fixed &amp; bounded</script></definition></flow-definition>'
if ([string]$document.'flow-definition'.definition.script -cne 'fixed & bounded') { throw 'XML 1.1 readback lost script bytes' }
$rejected = $false
try { $null = Read-JenkinsJobXml ('<?xml version="1.1"?><flow-definition>' + [char]1 + '</flow-definition>') }
catch { $rejected = $_.Exception.Message -eq 'Cannot parse Jenkins job configuration XML' }
if (-not $rejected) { throw 'Unsupported XML content was accepted' }
'''
        result = subprocess.run(["powershell.exe", "-NoProfile", "-NonInteractive", "-Command", script],
                                env=environment, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=60)
        self.assertEqual(0, result.returncode, result.stderr.decode("utf-8", errors="replace"))

    def test_scheduled_cleanup_audit_receipt_and_review_status(self):
        from tools.dev import jenkins_governance as operator
        hook = operator.maintenance_hook("fixture-request", "// fixture-only audit hook")
        delimiter = "label: 'Verify original governance audit receipt', script: " + chr(39) * 3
        script = hook.split(delimiter, 1)[1].split(chr(39) * 3, 1)[0]
        # Execute the production validator against this Jenkins-owned fixture,
        # without writing a production receipt or manufacturing State evidence.
        literal = "'E:/cargo-targets/zircon-local/reports/" + operator.TASK + "/'"
        script = script.replace(literal, "'" + self.root.as_posix() + "/'")
        operation = "scheduled-1"
        directory = self.root / operation
        directory.mkdir()
        review_path = directory / "inventory-review.json"
        receipt_path = directory / "receipt.json"
        summary = {"omissions": 0, "legacyOwners": 1}
        review = {"missingInventory": True, "coverage": None, "summary": summary}
        receipt = {"action": "audit", "finishedAt": 1,
                   "identity": {"taskId": operator.TASK, "sessionId": "jenkins-maintenance",
                                "operationId": operation, "requestId": operation, "attemptId": "attempt-1",
                                "driverDigest": "fixture-driver", "jenkinsBuildUrl": "http://127.0.0.1:18080/job/zircon-governance/17/"},
                   "result": {"status": "needs-review", "inventoryDigest": None, "summary": summary}}
        environment = {**os.environ, "ZGOV_EXPECTED_OPERATION": operation, "ZGOV_EXPECTED_BUILD": "17",
                       "ZIRCON_DRIVER_DIGEST": "fixture-driver"}

        def execute():
            review_path.write_bytes(encoded(review))
            receipt["result"]["review"] = file_fact(review_path)
            receipt_path.write_bytes(encoded(receipt))
            return subprocess.run(["powershell.exe", "-NoProfile", "-NonInteractive", "-Command", script],
                                  env=environment, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=60)

        result = execute()
        self.assertEqual(0, result.returncode, result.stderr.decode("utf-8", errors="replace"))
        observation = json.loads(result.stdout.decode("utf-8-sig"))
        self.assertEqual("needs-review", observation["status"])
        self.assertIn("missing-inventory", observation["reasons"])
        self.assertIn("unreconciled-legacy-owners", observation["reasons"])

        receipt["identity"]["requestId"] = "another-request"
        result = execute()
        self.assertNotEqual(0, result.returncode)
        self.assertIn("original scheduled request/build", result.stderr.decode("utf-8", errors="replace"))
        receipt["identity"]["requestId"] = operation

        review.update(missingInventory=False, coverage={"status": "complete"})
        summary["legacyOwners"] = 0
        receipt["result"].update(status="complete", inventoryDigest="1" * 64)
        result = execute()
        self.assertEqual(0, result.returncode, result.stderr.decode("utf-8", errors="replace"))
        self.assertEqual("complete", json.loads(result.stdout.decode("utf-8-sig"))["status"])

        review_path.write_bytes(b"changed after the recorded review hash")
        result = subprocess.run(["powershell.exe", "-NoProfile", "-NonInteractive", "-Command", script],
                                env=environment, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=60)
        self.assertNotEqual(0, result.returncode)
        self.assertIn("Original inventory review bytes changed", result.stderr.decode("utf-8", errors="replace"))

    def test_scheduled_cleanup_disk_space_uses_native_volume(self):
        import shutil
        source = Path(__file__).resolve().parents[3] / "tools/jenkins-management/zircon-scheduled-cleanup.groovy"
        delimiter = "label: 'Measure maintenance disk headroom', script: " + chr(39) * 3
        script = source.read_text(encoding="utf-8-sig").split(delimiter, 1)[1].split(chr(39) * 3, 1)[0]
        before = {drive: shutil.disk_usage(drive + ":/").free for drive in "DEF"}
        result = subprocess.run(["powershell.exe", "-NoProfile", "-NonInteractive", "-Command", script],
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=60)
        after = {drive: shutil.disk_usage(drive + ":/").free for drive in "DEF"}
        self.assertEqual(0, result.returncode, result.stderr.decode("utf-8", errors="replace"))
        volumes = json.loads(result.stdout.decode("utf-8-sig"))
        self.assertEqual(set("DEF"), {volume["drive"] for volume in volumes})
        for volume in volumes:
            drive, free = volume["drive"], volume["freeBytes"]
            # Concurrent cleanup/build writes can change a volume during the
            # process launch. A missing PSDrive.Free property must never become 0.
            self.assertGreater(free, 0)
            self.assertLessEqual(free, volume["totalBytes"])
            self.assertGreaterEqual(free, min(before[drive], after[drive]) - 2 * 1024**3)
            self.assertLessEqual(free, max(before[drive], after[drive]) + 2 * 1024**3)

    def test_permission_config_preimage_remains_reachable(self):
        from .permissions import plan, apply
        codex = self.repo / ".codex"
        codex.mkdir()
        config = codex / "config.toml"
        config.write_text('default_permissions = "fixture-old"\n', encoding="utf-8")
        before = config.read_bytes()
        report = self.root / "permission-report"
        report.mkdir()
        prepared = plan(self.authority, {"repositoryId": self.repository_id, "ownerSessionId": "fixture-a",
            "expectedConfigHash": file_fact(config)["sha256"], "operationId": "fixture-permission-plan"}, report)
        # Lifecycle ledger exists before the controlled config transition creates any output.
        with mock.patch("tools.jenkins.governance.worker.register_artifacts", return_value=None):
            result = apply(self.authority, {"planId": "fixture-permission-plan", "planDigest": prepared["planDigest"],
                "operationId": "fixture-permission-apply"}, report)
        retained = text_git(self.repo, "rev-parse", result["retainedRef"])
        self.assertEqual(retained, result["retainedCommit"])
        self.assertEqual(before, git(self.repo, "show", retained + ":config.toml"))
        self.assertIn("outbox", prepared["handoffRoot"])
        import tomllib
        applied = tomllib.loads(config.read_text(encoding="utf-8"))
        self.assertEqual(prepared["defaultProfile"], applied["default_permissions"])
        self.assertNotIn("write", applied["permissions"][applied["default_permissions"]]["filesystem"].values())

    def test_implementation_closure_keeps_library_guards_and_rejects_unknown_entries(self):
        from tools.dev import jenkins_governance as operator
        control = Path(__file__).resolve().parents[3]
        selected = {path.relative_to(control).as_posix() for path in operator._implementation_closure(control)}
        required = {
            "tools/jenkins/governance/bootstrap.py", "tools/jenkins/governance/worker.py",
            "tools/jenkins/governance/audit.py", "tools/jenkins/governance/inventory.py",
            "tools/jenkins/governance/cleanup.py", "tools/jenkins/governance/paths.py",
            "tools/jenkins/state/__init__.py", "tools/jenkins/deployment/driver.py",
            "tools/jenkins/source/__init__.py", "tools/jenkins/processes/identity.py",
            "tools/jenkins/processes/native.py", "tools/dev/local_cargo.py",
            "tools/jenkins/pilot/native/windows_job_process.py", "tools/jenkins/pilot/process_evidence.py",
        }
        self.assertTrue(required <= selected, sorted(required - selected))
        self.assertTrue({"tools/jenkins/frontend.py", "tools/jenkins/current_source.py",
                         "tools/jenkins/workflow/handler.py"}.isdisjoint(selected))
        parse = operator.ast.parse
        worker = str(control / "tools/jenkins/governance/worker.py")
        for statement in (
                "from tools.dev.local_cargo import main",
                "from tools.jenkins.source import seal_manifest",
                "from tools.jenkins.processes import ExecutionHost",
                "from tools.jenkins.source import *",
                "import tools.dev.local_cargo as cargo",
                "from tools.jenkins import processes"):
            with self.subTest(importStatement=statement):
                def injected(source, filename="<unknown>", **options):
                    tree = parse(source, filename=filename, **options)
                    if filename == worker:
                        tree.body.extend(parse(statement).body)
                    return tree
                # Alter only the parser's in-memory input, never a source file.
                with mock.patch.object(operator.ast, "parse", side_effect=injected):
                    with self.assertRaisesRegex(ValueError, "^undeclared_governance_library_import:"):
                        operator._implementation_closure(control)

    def test_bootstrap_rejects_changed_helper_before_importing_it(self):
        import base64, zlib
        bootstrap = self.root / "bootstrap-control/tools/jenkins/governance/bootstrap.py"
        bootstrap.parent.mkdir(parents=True)
        bootstrap.write_bytes((Path(__file__).parent / "bootstrap.py").read_bytes())
        helper = bootstrap.parent / "paths.py"
        helper.write_bytes(b"raise RuntimeError('helper-executed')\n")
        control = bootstrap.parents[3]
        pins = {"controlRoot": str(control), "sourcePins": {"tools/jenkins/governance/paths.py": "0" * 64},
                "pythonSha256": file_fact(Path(sys.executable))["sha256"], "jobName": "zircon-governance"}
        result = subprocess.run([sys.executable, "-B", str(bootstrap), "--pins-base64",
            base64.b64encode(zlib.compress(encoded(pins))).decode(), "--request-base64", "e30="],
            capture_output=True, timeout=60)
        self.assertNotEqual(0, result.returncode)
        self.assertIn(b"governance_source_changed", result.stderr)
        self.assertNotIn(b"helper-executed", result.stderr)

    def test_build_output_cleanup_rejects_canonical_overlap(self):
        from .cleanup import _proof
        output = self.repo / "build-output"
        output.mkdir()
        with self.assertRaisesRegex(ValueError, "canonical_source_checkout_protected"):
            _proof(self.authority, {"repositoryId": self.repository_id, "kind": "build-output"}, output)


    def test_cleanup_refuses_content_and_path_replacement(self):
        from .guarded_delete import remove_verified
        file = self.root / "generated.obj"
        file.write_bytes(b"fixture object")
        before = file_fact(file)
        file.write_bytes(b"changed object")
        with self.assertRaises((OSError, ValueError)):
            remove_verified(before)
        self.assertTrue(file.exists())
        file.write_bytes(b"fixture object")
        before = file_fact(file)
        replacement = self.root / "replacement.obj"
        replacement.write_bytes(file.read_bytes())
        os.utime(replacement, ns=(file.stat().st_atime_ns, file.stat().st_mtime_ns))
        os.replace(replacement, file)
        with self.assertRaises((OSError, ValueError)):
            remove_verified(before)
        self.assertTrue(file.exists())

    def test_cleanup_hardlink_and_active_process_protection(self):
        from .guarded_delete import remove_verified
        file = self.root / "generated.obj"
        file.write_bytes(b"fixture object")
        twin = self.root / "hardlink.obj"
        os.link(file, twin)
        with self.assertRaises(ValueError):
            remove_verified(file_fact(file))
        twin.unlink()  # Both links are owned by this generated test fixture.
        code = "import pathlib,sys; f=pathlib.Path(sys.argv[1]).open('r+b'); print('READY',flush=True); sys.stdin.readline(); f.close()"
        child = subprocess.Popen([sys.executable, "-B", "-c", code, str(file)], stdin=subprocess.PIPE,
                                 stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        try:
            self.assertEqual(child.stdout.readline().strip(), "READY")
            with self.assertRaises((OSError, ValueError)):
                remove_verified(file_fact(file))
        finally:
            child.communicate("close\n", timeout=15)
        self.assertTrue(file.exists())
        result = remove_verified(file_fact(file))
        self.assertEqual("deleted", result["status"])
        self.assertFalse(file.exists())

    def test_parent_lock_prevents_reparse_write_access(self):
        from .guarded_delete import locked_parents
        import ctypes
        from ctypes import wintypes
        parent = self.root / "empty-parent"
        parent.mkdir()
        target = parent / "new-source.txt"
        with locked_parents(target) as api:
            handle = api.CreateFileW("\\\\?\\" + str(parent), 0x40000000, 7, None, 3, 0x02200000, None)
            if handle != wintypes.HANDLE(-1).value:
                api.CloseHandle(handle)
                self.fail("parent accepted reparse-capable write access during source transition")
            self.assertEqual(32, ctypes.get_last_error())

    def test_cleanup_rejects_replaced_parent_even_with_original_file(self):
        from .guarded_delete import remove_verified
        parent = self.root / "planned-parent"
        parent.mkdir()
        file = parent / "generated.obj"
        file.write_bytes(b"same physical file")
        fact = file_fact(file)
        information = parent.lstat()
        parents = {str(parent): [information.st_dev, information.st_ino]}
        retired = self.root / "retired-parent"
        os.rename(parent, retired)
        parent.mkdir()
        os.rename(retired / file.name, file)
        with self.assertRaises(ValueError):
            remove_verified(fact, expected_directory_identities=parents)
        self.assertEqual(b"same physical file", file.read_bytes())

    def test_junction_is_reported_not_followed(self):
        target = self.root / "outside"
        target.mkdir()
        (target / "Cargo.toml").write_text("fixture source")
        scoped = self.root / "scoped"
        scoped.mkdir()
        alias = scoped / "junction"
        result = subprocess.run(["cmd.exe", "/d", "/c", "mklink", "/J", str(alias), str(target)], capture_output=True)
        self.assertEqual(result.returncode, 0, result.stderr.decode(errors="replace"))
        try:
            observation = scan([scoped])
            self.assertEqual(len(observation["links"]), 1)
            self.assertFalse(observation["sourceCandidates"])
            with self.assertRaises(ValueError):
                physical(alias / "Cargo.toml")
        finally:
            os.rmdir(alias)  # Exact junction created by this test; never recurse into its target.

    def test_inventory_holds_directory_identity_against_replacement(self):
        from .walk import walk
        scoped = self.root / "retained-directory"
        scoped.mkdir()
        (scoped / "standalone.cpp").write_bytes(b"fixture source without a project marker")
        (scoped / "standalone.exe").write_bytes(b"fixture compiled candidate without a build directory name")
        failures, links = [], []
        traversal = walk([scoped], error=lambda path, error: failures.append((path, error)), link=links.append)
        try:
            directory, entries = next(traversal)
            self.assertEqual(scoped, directory)
            with self.assertRaises(OSError):
                os.rename(scoped, self.root / "replacement-directory")
        finally:
            traversal.close()
        self.assertFalse(failures)
        self.assertFalse(links)
        observation = scan([scoped])
        self.assertEqual(2, observation["files"])
        self.assertEqual(2, observation["rootCoverage"][str(scoped)]["files"])
        self.assertTrue(observation["sourceCandidates"])
        self.assertTrue(observation["outputCandidates"])


    def managed_fixture(self):
        directory = self.repo / ".jenkins/state"
        (directory / "deployment").mkdir(parents=True)
        (directory / "deployment/driver.json").write_bytes(encoded({
            "driverDigest": "fixture-driver", "runtimeOperationId": "fixture-runtime"}))
        (directory / "deployment-operation.json").write_bytes(encoded({
            "operationId": "fixture-runtime", "state": "running"}))
        from .worker import _managed_index
        return _managed_index(self.authority)

    def test_managed_index_is_readonly_and_protects_original_owners(self):
        import sqlite3
        from .worker import _managed_index
        self.managed_fixture()
        for key, value in (("active", {"status": "active", "driverDigest": "fixture-driver"}),
                           ("history", {"status": "released", "driverDigest": "fixture-driver"}),
                           ("unknown", {})):
            self.state.put("management_worktree", key, {**value, "path": str(self.root / key)})
        self.state.put("execution", "oversized", {"private": "x" * 70000})
        self.state.put("preparation_source", "journal-not-selected", {"private": "x" * 70000})
        def snapshot():
            with sqlite3.connect(self.state.path.as_uri() + "?mode=ro", uri=True) as connection:
                return list(connection.iterdump())
        before = snapshot()
        index = _managed_index(self.authority)
        self.assertEqual(before, snapshot())
        self.assertEqual(("ro", True, "complete"), (index["mode"], index["queryOnly"], index["status"]))
        owners = {row["key"]: row for row in index["records"] if row["domain"] == "management_worktree"}
        self.assertEqual(["protected-active", "protected-history", "protected-unknown"],
                         [owners[key]["protection"] for key in ("active", "history", "unknown")])
        oversized = next(row for row in index["records"] if row["key"] == "oversized")
        self.assertEqual("protected-unknown", oversized["protection"])
        self.assertNotIn("private", encoded(index).decode())
        self.assertFalse(any(row["domain"] == "preparation_source" for row in index["records"]))
        selected = self.repo / ".jenkins/state/deployment/driver.json"
        selected.write_bytes(encoded({"driverDigest": "other", "runtimeOperationId": "fixture-runtime"}))
        with self.assertRaisesRegex(ValueError, "runtime_changed"):
            _managed_index(self.authority)

    def test_managed_scan_prunes_protected_roots_and_has_independent_budgets(self):
        from .inventory import scan_managed
        from .walk import WalkBudget
        from .worker import _managed_index
        roots = [self.root / name for name in ("first", "second", "root-protected")]
        for root in roots:
            root.mkdir()
        self.managed_fixture()
        containers = [Path(drive + ":/cargo-targets") for drive in "DEF"]
        cases = [(domain, domain + "-" + drive, {"buildRoot": str(container), "status": "released"}, "namespace")
                 for domain in ("execution", "resource_reservation", "management_task_scratch")
                 for drive, container in zip("DEF", containers)]
        cases += [
            ("native_job", "other-domain", {"buildRoot": str(containers[2]), "status": "active"}, "retained"),
            ("execution", "other-field", {"buildNamespace": str(containers[2]), "status": "active"}, "retained"),
            ("execution", "subpath", {"buildRoot": str(containers[0] / "history"), "status": "released"}, "retained"),
            ("execution", "alias", {"buildRoot": "\\\\?\\D:\\cargo-targets"}, "retained"),
            ("execution", "dotdot", {"buildRoot": str(containers[0] / ".." / "cargo-targets")}, None),
            ("execution", "concrete-active", {"sourceRoot": str(containers[0] / "active"),
                "preparationRoot": str(containers[0] / "active"), "status": "active"}, "source"),
        ]
        for domain, key, value, _ in cases:
            self.state.put(domain, key, {**value, "driverDigest": "fixture-driver"})
        projected = _managed_index(self.authority)
        selected = {(row["domain"], row["key"]): row for row in projected["records"]}
        records = []
        for domain, key, value, role in cases:
            row = selected[domain, key]
            if role == "namespace":
                self.assertEqual("protected-history", row["protection"])
            self.assertEqual(digest({**value, "driverDigest": "fixture-driver"}), row["recordDigest"])
            if role is None:
                self.assertFalse(row["paths"])
            else:
                self.assertEqual(role, row["paths"][0]["role"])
            paths = []
            for scope in row["paths"]:
                path = Path(scope["path"])
                # Map projected metadata to owned fixtures; never walk the actual drive roots.
                for container, fixture in zip(containers, roots):
                    if path.is_relative_to(container):
                        path = fixture / path.relative_to(container)
                        break
                paths.append({**scope, "path": str(path)})
            records.append({**row, "paths": paths})
        for name in ("active", "history", "unknown"):
            child = roots[0] / name
            (child / "deep").mkdir(parents=True)
            (child / "deep/retained.obj").write_bytes(b"retained")
        (roots[1] / "one.obj").write_bytes(b"metadata")
        (roots[2] / "never-enter").mkdir()
        (roots[2] / "never-enter/retained.obj").write_bytes(b"root protected")
        index = {"records": records, "status": "complete", "omissions": []}
        value = scan_managed(roots, index)
        self.assertEqual("complete", value["status"])
        self.assertFalse(value["globalInventoryComplete"])
        self.assertEqual(0, value["rootCoverage"][str(roots[2])]["directories"])
        self.assertEqual(0, value["rootCoverage"][str(roots[2])]["files"])
        self.assertEqual(2, value["directories"])
        self.assertEqual(1, value["files"])
        self.assertEqual({"protected-active", "protected-history", "protected-unknown"},
                         {row["reason"] for row in value["protected"]})
        self.assertTrue(all(row["followed"] is False and row["cleanupAuthorized"] is False
                            for row in value["protected"]))
        active = next(row for row in value["protected"] if row["path"] == str(roots[0] / "active"))
        self.assertEqual("protected-active", active["reason"])
        self.assertEqual({"sourceRoot", "preparationRoot"}, {proof["field"] for proof in active["proofs"]})
        self.assertEqual({"source", "retained"}, {proof["role"] for proof in active["proofs"]})
        limited = scan_managed(roots[:2], index, budget=WalkBudget(entries=2))
        self.assertEqual("partial", limited["status"])
        self.assertEqual("partial", limited["rootCoverage"][str(roots[0])]["status"])
        self.assertEqual(1, limited["rootCoverage"][str(roots[1])]["entries"])
        self.assertEqual(1, limited["rootCoverage"][str(roots[1])]["files"])
        self.assertTrue((roots[0] / "active/deep/retained.obj").exists())

    def test_managed_audit_uses_exact_scan_native_and_complete_hook_preimage(self):
        from .audit import run, _inventory
        from .inventory import MANAGED_LIMITS, MANAGED_ROOTS
        from .worker import native, _managed_index
        from tools.dev import jenkins_governance as operator
        self.managed_fixture()
        operation = "scheduled-2-inventory"
        authority = Governance(self.state, self.repo, {**self.authority.runtime, "operationId": operation})
        index = _managed_index(authority)
        report = self.root / operation
        report.mkdir()
        index_path = report / "managed-index.json"
        index_path.write_bytes(encoded(index))
        # Synthetic fixed-root metadata tests the audit protocol; the preceding
        # test performs real bounded traversal. This is not actual D/E/F acceptance.
        roots = [str(Path(root)) for root in MANAGED_ROOTS]
        inventory = {"schemaVersion": 4, "inventoryPolicy": "managed-roots-v1", "roots": roots,
            "rootCoverage": {root: {"physicalRoot": {"verified": True, "afterVerified": True,
                                  "volumeIdentity": "synthetic-fixture"}, "directories": 0} for root in roots},
            "limits": MANAGED_LIMITS, "globalInventoryComplete": False, "stateIndex": index,
            "coverageScope": "index-directed-managed-root-metadata", "protected": [], "errors": [],
            "status": "complete"}
        inventory["inventoryDigest"] = digest(inventory)
        path = report / "inventory.json"
        path.write_bytes(encoded(inventory))
        evidence = native(authority, [sys.executable, "-B", "-c", "print('fixture-native-eof')"],
                          self.repo, report / "native-scan", timeout=60)
        self.assertEqual(0, evidence["rootExitCode"])
        reference = {"status": "complete", "inventoryPolicy": "managed-roots-v1", "inventoryId": operation,
            "inventoryDigest": inventory["inventoryDigest"], "inventoryRef": file_fact(path),
            "managedIndexRef": file_fact(index_path), "binding": index["binding"],
            "nativeReceipt": file_fact(report / "native-scan/native-receipt.json"),
            "nativeRecordKey": digest(str(report / "native-scan"))}
        authority.put("inventory", operation, reference)
        authority.put("inventory", "newer-decoy", {"inventoryPolicy": "managed-roots-v1"})
        audit_report = self.root / "exact-audit"
        audit_report.mkdir()
        payload = {"inventoryPolicy": "managed-roots-v1", "inventoryId": operation,
                   "inventoryDigest": inventory["inventoryDigest"], "operationId": "exact-audit"}
        result = run(authority, payload, audit_report)
        self.assertEqual("complete", result["status"])
        self.assertEqual(operation, result["inventoryId"])
        self.assertEqual(0, result["summary"]["cleanupAuthorized"])
        self.assertIsNone(_inventory(authority, operation))
        for changed in ({**payload, "inventoryId": None}, {**payload, "inventoryDigest": "0" * 64}):
            with self.assertRaises(ValueError):
                run(authority, changed, audit_report)
        receipt = {"action": "scan", "finishedAt": 1, "result": reference, "identity": {
            "taskId": operator.TASK, "sessionId": "jenkins-maintenance", "operationId": operation,
            "requestId": operation, "attemptId": "attempt-1", "driverDigest": "fixture-driver",
            "capsuleDigest": "fixture-capsule", "jenkinsBuildUrl": "http://127.0.0.1:18080/job/zircon-governance/18/"}}
        (report / "receipt.json").write_bytes(encoded(receipt))
        marker = "// fixture-managed-governance"
        old = operator._previous_maintenance_hook("fixture-request", marker)
        updated = operator._updated_maintenance_script("core\n" + old, "fixture-request", marker)
        self.assertEqual("core\n" + operator.maintenance_hook("fixture-request", marker), updated)
        self.assertIsNone(operator._updated_maintenance_script(updated, "fixture-request", marker))
        with self.assertRaisesRegex(ValueError, "unreviewed"):
            operator._updated_maintenance_script("core\n" + old.replace("wait: true", "wait: false"),
                                                 "fixture-request", marker)
        delimiter = "label: 'Verify exact bounded governance inventory', script: " + chr(39) * 3
        script = updated.split(delimiter, 1)[1].split(chr(39) * 3, 1)[0]
        literal = "'E:/cargo-targets/zircon-local/reports/" + operator.TASK + "/'"
        script = script.replace(literal, "'" + self.root.as_posix() + "/'")
        environment = {**os.environ, "ZGOV_SCAN_OPERATION": operation, "ZGOV_SCAN_BUILD": "18",
            "ZIRCON_DRIVER_DIGEST": "fixture-driver", "ZIRCON_RUNTIME_OPERATION_ID": "fixture-runtime"}
        process = subprocess.run(["powershell.exe", "-NoProfile", "-NonInteractive", "-Command", script],
                                 env=environment, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=60)
        self.assertEqual(0, process.returncode, process.stderr.decode("utf-8", errors="replace"))
        self.assertEqual(operation, json.loads(process.stdout.decode("utf-8-sig"))["inventoryId"])
        inventory.update(status="partial", errors=[{"reason": "fixture-budget-exhausted"}])
        inventory["inventoryDigest"] = digest({k: v for k, v in inventory.items() if k != "inventoryDigest"})
        path.write_bytes(encoded(inventory))
        reference.update(inventoryRef=file_fact(path), inventoryDigest=inventory["inventoryDigest"])
        authority.put("inventory", operation, reference)
        pending = run(authority, {**payload, "inventoryDigest": inventory["inventoryDigest"],
                                 "operationId": "partial-audit"}, audit_report)
        self.assertEqual("needs-review", pending["status"])
        path.write_bytes(b"tampered inventory")
        with self.assertRaisesRegex(ValueError, "retained_bytes_changed"):
            run(authority, {**payload, "inventoryDigest": inventory["inventoryDigest"]}, audit_report)


if __name__ == "__main__":
    selected = json.loads(os.environ.get("ZGOV_TEST_NAMES", "[]"))
    available = unittest.defaultTestLoader.getTestCaseNames(GovernanceTests)
    if not isinstance(selected, list) or any(name not in available for name in selected):
        raise ValueError("unknown_governance_contract_test")
    suite = unittest.TestSuite(GovernanceTests(name) for name in selected) if selected else unittest.defaultTestLoader.loadTestsFromTestCase(GovernanceTests)
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    from .fixtures import capture
    root = Path(os.environ["ZGOV_TEST_ROOT"])
    (root.parent / "fixture-inventory.json").write_bytes(encoded(capture(root)))
    raise SystemExit(0 if result.wasSuccessful() else 1)
