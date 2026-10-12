"""A fresh management review is specific to its exact departed generation."""
import copy
import hashlib
import unittest
from unittest.mock import patch

from tools.jenkins.contracts import JenkinsError, canonical_json
from tools.jenkins.deployment import recovery
import tools.jenkins.tests.test_control_plane_recovery as fixtures


class ManagementGenerationReviewTests(unittest.TestCase):
    def setUp(self):
        self.case = fixtures.ManagementRecoveryTests(methodName='runTest')
        self.case.setUp()
        self.addCleanup(self.case.doCleanups)
        case = self.case
        case.op.update(operationId='reviewed', generation='reviewed', pid=31, hostPid=31, creationTime='300')
        case.host.update(operationId='reviewed', generation='reviewed', hostPid=31,
                         controllerIdentity={**case.host['controllerIdentity'], 'pid': 32, 'creationTime': '310'})
        case.selected['runtimeOperationId'] = 'reviewed'
        reviewed = copy.deepcopy(case.audit)
        reviewed.update(runtimeOperationId='reviewed', operationBinding=copy.deepcopy(case.op),
                        hostBinding=copy.deepcopy(case.host))
        self.review_path = case.audit_path.with_name('reviewed-generation.json')
        self.review_path.write_bytes(canonical_json(reviewed))
        self.reference = (self.review_path.relative_to(case.repo).as_posix(),
                          hashlib.sha256(self.review_path.read_bytes()).hexdigest())
        for path, value in ((case.repo / '.jenkins/state/deployment-operation.json', case.op),
                            (case.repo / '.jenkins/state/deployment/host.json', case.host),
                            (case.repo / '.jenkins/state/deployment/driver.json', case.selected)):
            path.write_bytes(canonical_json(value))

    def test_previous_review_does_not_authorize_a_later_owner(self):
        with self.case._observations(), patch.object(recovery, '_CONTROL_PLANE_MANAGEMENT_GENERATION_AUDITS', {}):
            with self.assertRaises(JenkinsError) as error:
                self.case._call()
        self.assertEqual('deployment_management_audit_invalid', error.exception.code)

    def test_explicit_current_review_reopens_only_quiet_management(self):
        reviews = {(self.case.driver, 'reviewed'): self.reference}
        with self.case._observations(), patch.object(recovery, '_CONTROL_PLANE_MANAGEMENT_GENERATION_AUDITS', reviews):
            evidence = self.case._call()
        self.assertEqual('reviewed', evidence['operationId'])
        self.assertEqual(str(self.review_path), evidence['auditReceipt']['path'])
        self.assertEqual(self.reference[1], evidence['auditReceipt']['sha256'])
        self.assertIs(evidence['controlPlaneOnly'], True)
        for field in ('nativeProofIssued', 'validationAcceptanceAllowed', 'executionHoldsReleaseAllowed'):
            self.assertIs(evidence[field], False)

    def test_changed_review_bytes_or_owner_birth_remain_blocked(self):
        reviews = {(self.case.driver, 'reviewed'): self.reference}
        with self.case._observations(), patch.object(recovery, '_CONTROL_PLANE_MANAGEMENT_GENERATION_AUDITS', reviews):
            with self.assertRaises(JenkinsError):
                self.case._call(operation={**self.case.op, 'creationTime': '301'})
            self.review_path.write_bytes(self.review_path.read_bytes() + b' ')
            with self.assertRaises(JenkinsError):
                self.case._call()


if __name__ == '__main__':
    unittest.main()
