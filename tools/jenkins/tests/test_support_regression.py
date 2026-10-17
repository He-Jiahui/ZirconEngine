"""Small receipt fixtures; no actual service, compiler or regression run."""
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import tempfile
import unittest

from tools.jenkins.contracts import JenkinsError
from tools.jenkins.deployment.driver import _LAUNCHER, seal_driver
from tools.jenkins.deployment.support_regression import (
    FULL_SCOPE, FOCUSED_SCOPE, SCOPED_SCOPE, support_sources, verify_support_regression)


class SupportRegressionTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix="support-regression-")
        self.addCleanup(self.tmp.cleanup)
        self.repo = Path(self.tmp.name)
        self.root = self.repo / ".jenkins/state/migration-baseline"
        self.root.mkdir(parents=True)
        self.tests = ["tools/jenkins/tests/test_agent_bootstrap_protocol.py",
                      "tools/jenkins/tests/test_agent_bootstrap_timeout.py",
                      "tools/jenkins/tests/test_agent_node_migration.py"]
        files = {"tools/jenkins/__main__.py": "pass\n",
                 "tools/jenkins/deployment/bootstrap.py": "VALUE = 'old'\n",
                 "tools/jenkins/deployment/driver.py": "_LAUNCHER = " + repr(_LAUNCHER) + "\n",
                 ".jenkins/deployment-spec.json": "{}", ".jenkins/workflow-spec.json": "{}",
                 ".jenkins/pipeline/steps.groovy": "return this\n",
                 **{name: "def test_fixture(): pass\n" for name in self.tests}}
        for name, text in files.items():
            path = self.repo / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text, encoding="utf-8")
        self.before = support_sources(self.repo)
        self.old_candidate = seal_driver(self.repo)
        self.base = self.run_receipt(FULL_SCOPE, self.before)
        self.base.update(testRoots=[str(self.repo / "tools/jenkins/tests"), str(self.repo / "tools/jenkins/tray/tests")])
        self.base_ref = self.save("base.json", self.base)
        (self.repo / "tools/jenkins/deployment/bootstrap.py").write_text("VALUE = 'new'\n", encoding="utf-8")
        self.after = support_sources(self.repo)
        self.candidate = seal_driver(self.repo)
        self.focus = self.run_receipt(FOCUSED_SCOPE, self.after)
        self.focus.update(collectedNodes=self.nodes, sourceFilesAfter=self.after,
                          testTargets=[str(self.repo / name) for name in self.tests])
        self.focus_ref = self.save("focused.json", self.focus)
        self.document = {**{name: self.focus[name] for name in
            ("status", "testsRun", "sourceIdentityStable", "exitCode", "failures", "errors",
             "collectionErrors", "testTargets", "log", "logSha256")},
            "schemaVersion": 1, "scope": SCOPED_SCOPE, "baseRegression": self.base_ref,
            "focusedRegression": self.focus_ref, "sourceFilesBefore": self.before, "sourceFiles": self.after,
            "changedFiles": ["tools/jenkins/deployment/bootstrap.py"], "fullDiscoveryRun": False,
            "runtimeAcceptance": False, "cargoAcceptance": False, "milestoneAcceptance": False}
        self.reference = self.save("scoped.json", self.document)
        self.serial = 0

    def save(self, name, value):
        path = self.root / name
        path.write_text(json.dumps(value), encoding="utf-8")
        return {"path": str(path), "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}

    def run_receipt(self, scope, sources):
        self.nodes = [name + "::test_fixture" for name in self.tests]
        log = self.root / ("full.log" if scope == FULL_SCOPE else "focus.log")
        log.write_text(str(len(self.nodes)) + " passed\n", encoding="utf-8")
        return {"schemaVersion": 1, "scope": scope, "status": "passed", "sourceIdentityStable": True,
            "testsRun": len(self.nodes), "exitCode": 0, "failures": 0, "errors": 0, "collectionErrors": [],
            "sourceFiles": sources, "log": str(log), "logSha256": hashlib.sha256(log.read_bytes()).hexdigest(),
            "reports": [{"test": node, "phase": phase, "outcome": "passed"}
                        for node in self.nodes for phase in ("setup", "call", "teardown")]}

    def reject(self, document):
        self.serial += 1
        reference = self.save("invalid-" + str(self.serial) + ".json", document)
        with self.assertRaises(JenkinsError):
            verify_support_regression(self.repo, reference, self.candidate)

    def test_focused_scope_binds_candidate_and_survives_later_canonical_edits(self):
        original = (self.root / "base.json").read_bytes()
        receipt = verify_support_regression(self.repo, self.reference, self.candidate)
        self.assertEqual(SCOPED_SCOPE, receipt["scope"])
        self.assertFalse(receipt["fullDiscoveryRun"])
        (self.repo / "tools/jenkins/deployment/bootstrap.py").write_text("VALUE = 'later'\n", encoding="utf-8")
        with self.assertRaises(JenkinsError):
            verify_support_regression(self.repo, self.reference, self.candidate)
        self.assertEqual(receipt, verify_support_regression(self.repo, self.reference, self.candidate, current=False))
        self.assertEqual(original, (self.root / "base.json").read_bytes())

    def test_verified_scoped_baseline_can_reuse_unchanged_closure(self):
        # The earlier bootstrap change retains its complete three-module proof;
        # this later delta changes only one test and executes that exact module.
        changed_test = self.tests[2]
        (self.repo / changed_test).write_text("def test_fixture(): pass\n# later test change\n", encoding="utf-8")
        after = support_sources(self.repo)
        nodes = [changed_test + "::test_fixture"]
        log = self.root / "chain-focus.log"
        log.write_text("1 passed\n", encoding="utf-8")
        focused = {**self.focus, "testsRun": 1, "sourceFiles": after, "sourceFilesAfter": after,
            "collectedNodes": nodes, "testTargets": [str(self.repo / changed_test)],
            "reports": [{"test": nodes[0], "phase": phase, "outcome": "passed"}
                        for phase in ("setup", "call", "teardown")],
            "log": str(log), "logSha256": hashlib.sha256(log.read_bytes()).hexdigest()}
        document = {**self.document, **{key: focused[key] for key in
            ("testsRun", "testTargets", "log", "logSha256")},
            "baseRegression": self.reference, "focusedRegression": self.save("chain-focus.json", focused),
            "sourceFilesBefore": self.after, "sourceFiles": after, "changedFiles": [changed_test]}
        reference = self.save("chain.json", document)
        candidate = seal_driver(self.repo)
        self.assertEqual(1, verify_support_regression(self.repo, reference, candidate)["testsRun"])
        # Reusing a scoped baseline still verifies the original full-discovery log.
        (self.root / "full.log").write_text("altered original evidence\n", encoding="utf-8")
        with self.assertRaises(JenkinsError):
            verify_support_regression(self.repo, reference, candidate)

    def test_actual_passed_unittest_subtests_keep_the_collected_node_count(self):
        focused = deepcopy(self.focus)
        focused["reports"].insert(2, deepcopy(focused["reports"][1]))
        document = {**self.document, "focusedRegression": self.save("subtests.json", focused)}
        reference = self.save("scoped-subtests.json", document)
        self.assertEqual(focused["testsRun"],
                         verify_support_regression(self.repo, reference, self.candidate)["testsRun"])

    def test_exact_delta_includes_added_and_removed_files(self):
        added = "tools/jenkins/deployment/failed_upgrade_recovery.py"
        removed = self.tests[1]
        (self.repo / added).write_text("VALUE = 'added'\n", encoding="utf-8")
        (self.repo / removed).unlink()
        after = support_sources(self.repo)
        focused = {**self.focus, "sourceFiles": after, "sourceFilesAfter": after}
        document = {**self.document, "sourceFiles": after,
                    "focusedRegression": self.save("different-focus.json", focused)}
        # The omitted addition/deletion cannot inherit the original delta.
        self.reject(document)

    def test_changed_module_closure_and_actual_results_are_required(self):
        for modify in ("missing-target", "failed-result"):
            with self.subTest(modify=modify):
                focused = deepcopy(self.focus)
                if modify == "missing-target":
                    focused["testTargets"] = focused["testTargets"][:1]
                else:
                    focused["reports"][1]["outcome"] = "failed"
                reference = self.save(modify + ".json", focused)
                self.reject({**self.document, "focusedRegression": reference})

    def test_baseline_and_its_log_cannot_be_replaced_by_focused_evidence(self):
        self.reject({**self.document, "baseRegression": self.focus_ref})
        (self.root / "full.log").write_text("changed output\n", encoding="utf-8")
        self.reject(self.document)

    def test_full_scope_still_works_and_sealed_inventory_must_match(self):
        full = {**self.base, "sourceFiles": self.after}
        reference = self.save("current-full.json", full)
        self.assertEqual(FULL_SCOPE, verify_support_regression(self.repo, reference, self.candidate)["scope"])
        with self.assertRaises(JenkinsError):
            verify_support_regression(self.repo, self.reference, self.old_candidate)

    def test_unrelated_change_cannot_use_startup_recovery_scope(self):
        path = self.repo / "tools/jenkins/workflow/unrelated.py"
        path.parent.mkdir()
        path.write_text("VALUE = 1\n", encoding="utf-8")
        after = support_sources(self.repo)
        focus = {**self.focus, "sourceFiles": after, "sourceFilesAfter": after}
        document = {**self.document, "sourceFiles": after,
                    "changedFiles": sorted([*self.document["changedFiles"], "tools/jenkins/workflow/unrelated.py"]),
                    "focusedRegression": self.save("unrelated-focus.json", focus)}
        self.reject(document)

    def test_exact_expanded_full_roots_preserve_complete_baseline_checks(self):
        roots = [str(self.repo / name) for name in ('tools/jenkins/tests', 'tools/jenkins/tray/tests', 'tools/jenkins/governance/tests.py', 'tools/jenkins-management/tests/test_checkout_validation.py', 'tools/jenkins-management/tests/test_entry_bootstrap.py', 'tools/jenkins-management/tests/test_ordinary_flow.py', 'tools/jenkins-management/tests/test_toolchain_facts.py', 'tools/tests/test_build_editor_independent.py', 'tools/tests/test_jenkins_checkout_product.py', 'tools/jenkins-management/tests/test_authorization_owner.py', 'tools/tests/test_jenkins_editor_source_root.py', 'tools/tests/test_jenkins_editor_entry.py', 'tools/tests/test_install_jenkins_management_codex_migration.py')]

        base = {**self.base, "testRoots": roots}
        doc = {**self.document, "baseRegression": self.save("expanded-base.json", base)}
        reference = self.save("expanded-scoped.json", doc)
        self.assertEqual(SCOPED_SCOPE, verify_support_regression(self.repo, reference, self.candidate)["scope"])
        for changed in ("unknown", "missing", "duplicate", "failed-report"):
            with self.subTest(changed=changed):
                invalid = deepcopy(base)
                if changed == "unknown":
                    invalid["testRoots"][-1] = str(self.repo / "tools/unknown-tests")
                elif changed == "missing":
                    invalid["testRoots"] = invalid["testRoots"][1:]
                elif changed == "duplicate":
                    invalid["testRoots"][-1] = invalid["testRoots"][0]
                else:
                    invalid["reports"][1]["outcome"] = "failed"
                self.reject({**doc, "baseRegression": self.save("expanded-" + changed + ".json", invalid)})

    def test_runtime_role_delta_requires_its_actual_focused_module(self):
        module = "tools/jenkins/deployment/runtime_roles.py"
        role_test = "tools/jenkins/tests/test_runtime_roles.py"
        (self.repo / module).write_text("VALUE = 'old role'\n", encoding="utf-8")
        (self.repo / role_test).write_text("def test_fixture(): pass\n", encoding="utf-8")
        before = support_sources(self.repo)
        base = self.run_receipt(FULL_SCOPE, before)
        base["testRoots"] = self.base["testRoots"]
        base_ref = self.save("role-base.json", base)
        (self.repo / module).write_text("VALUE = 'new role'\n", encoding="utf-8")
        after = support_sources(self.repo)
        candidate = seal_driver(self.repo)
        original_targets = self.tests
        for selected, accepted in (([role_test], True), ([original_targets[0]], False)):
            self.tests = selected
            focus = self.run_receipt(FOCUSED_SCOPE, after)
            focus.update(collectedNodes=self.nodes, sourceFilesAfter=after,
                         testTargets=[str(self.repo / name) for name in selected])
            doc = {**self.document, **{key: focus[key] for key in
                ("testsRun", "testTargets", "log", "logSha256")},
                "baseRegression": base_ref, "focusedRegression": self.save("role-focus.json", focus),
                "sourceFilesBefore": before, "sourceFiles": after, "changedFiles": [module]}
            reference = self.save("role-scoped.json", doc)
            if accepted:
                self.assertEqual(1, verify_support_regression(self.repo, reference, candidate)["testsRun"])
            else:
                with self.assertRaises(JenkinsError):
                    verify_support_regression(self.repo, reference, candidate)
        self.tests = original_targets

    def test_startup_inventory_delta_requires_its_complete_closure(self):
        module="tools/jenkins/deployment/startup_observations.py"
        inventory_test="tools/jenkins/tests/test_runtime_owner_inventory.py"
        abandoned_test="tools/jenkins/tests/test_abandoned_startup.py"
        (self.repo/module).write_text("VALUE = 'before'\n",encoding="utf-8")
        (self.repo/abandoned_test).parent.mkdir(parents=True,exist_ok=True)
        (self.repo/abandoned_test).write_text("def test_fixture(): pass\n",encoding="utf-8")
        before=support_sources(self.repo);base=self.run_receipt(FULL_SCOPE,before)
        base["testRoots"]=self.base["testRoots"];base_ref=self.save("inventory-base.json",base)
        (self.repo/module).write_text("VALUE = 'after'\n",encoding="utf-8")
        (self.repo/inventory_test).write_text("def test_fixture(): pass\n",encoding="utf-8")
        after=support_sources(self.repo);candidate=seal_driver(self.repo);original_tests=self.tests
        for selected,accepted in (([inventory_test,abandoned_test],True),([inventory_test],False),([abandoned_test],False)):
            with self.subTest(selected=selected):
                self.tests=selected;focus=self.run_receipt(FOCUSED_SCOPE,after)
                focus.update(collectedNodes=self.nodes,sourceFilesAfter=after,
                             testTargets=[str(self.repo/name) for name in selected])
                focus_ref=self.save("inventory-focus.json",focus)
                document={**self.document,**{key:focus[key] for key in ("testsRun","testTargets","log","logSha256")},
                    "baseRegression":base_ref,"focusedRegression":focus_ref,"sourceFilesBefore":before,
                    "sourceFiles":after,"changedFiles":sorted([module,inventory_test])}
                reference=self.save("inventory-scoped.json",document)
                if accepted:self.assertEqual(len(selected),verify_support_regression(self.repo,reference,candidate)["testsRun"])
                else:
                    with self.assertRaises(JenkinsError):verify_support_regression(self.repo,reference,candidate)
        self.tests=original_tests
