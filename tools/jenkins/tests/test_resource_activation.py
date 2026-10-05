import hashlib
import json
import tempfile
import time
import unittest
from pathlib import Path
from unittest.mock import patch

from tools.jenkins.contracts import JenkinsError, digest
from tools.jenkins.state import State
from tools.jenkins.resources import activation, Capacity, ResourceManager
from tools.jenkins.resources.paths import ApprovedBuildRoot


class ActivationTests(unittest.TestCase):
    def setUp(self):
        parent = Path('E:/cargo-targets/zircon-local/evidence/resource-activation-tests')
        parent.mkdir(parents=True, exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(dir=parent)
        self.addCleanup(self.temp.cleanup)
        self.repo = Path(self.temp.name)
        self.state = State(self.repo / 'test.db')
        self.old = self.state.put('resource_policy', 'default', {'buildRoot': 'old', 'cpuBudget': 3})
        self.state.put('resource_reservation', 'old:1', {'status': 'active', 'owner': 'old', 'cpu': 2, 'memoryBytes': 1, 'diskBytes': 1, 'buildRoot': 'old'})
        self.state.put('pool_writer_hold', 'old-writer', {'status': 'active', 'owner': 'old', 'reservationKey': 'old:1'})
        self.state.put('managed_storage_registration', 'old-namespace', {'buildRoot': 'old'})
        self.policy = dict(cpuBudget=3, memoryBudget=100, diskBudget=100, maxHeavyWriters=3,
                           inventoryMaxAgeSeconds=30, diskReserveBytes=1, buildRoot='D:\\cargo-targets')
        self.policy['recipeEstimates'] = {'default': {'cpu': 1, 'memoryBytes': 10, 'diskBytes': 10},
                                         'tiny-fixture': {'cpu': 1, 'memoryBytes': 2, 'diskBytes': 2}}
        self.spec_path = self.repo / '.jenkins/deployment-spec.json'
        self.spec_path.parent.mkdir()
        self.spec_path.write_text(json.dumps({'policy': self.policy}))
        self.args = {'expectedPolicyVersion': self.old['version'], 'expectedSpecSha256': hashlib.sha256(self.spec_path.read_bytes()).hexdigest()}
        self.root = ApprovedBuildRoot(Path('D:/cargo-targets'), 'native-volume')
        self.namespace = self.repo / 'namespace'
        self.spec = type('Spec', (), {'storage': {'resourcePolicy': self.policy, 'allowedPhysicalRoots': ['D:\\cargo-targets']}, 'build_root': self.root.path})()
        self.inventory = dict(generation='new-inventory', cpu=3, memoryBytes=100, diskBytes=100, complete=True, observedAt=time.time(), buildRoot=str(self.root.path))
        for name, value in [('load_spec', self.spec), ('canonical_build_root', self.root), ('physical_path_under', self.namespace)]:
            mock = patch.object(activation, name, return_value=value)
            mock.start(); self.addCleanup(mock.stop)
        self.capture = patch.object(ResourceManager, 'capture_inventory', return_value=self.inventory).start()
        self.addCleanup(patch.stopall)

    def run_activation(self):
        return activation.activate_policy(self.state, self.repo, self.args)

    def test_activation_preserves_historical_holds_and_budget_charge(self):
        before = activation._active_binding(self.state)
        result = self.run_activation()
        self.assertEqual(before, activation._active_binding(self.state))
        self.assertEqual(digest(before), result['preservedActiveDigest'])
        self.assertEqual(self.policy, {key: result['policy']['payload'][key] for key in self.policy})
        self.assertIsNotNone(self.state.get('managed_storage_registration', 'old-namespace'))
        manager = ResourceManager(self.state, Capacity(3,100,100))
        self.assertIsNone(manager.admit('new', Capacity(2,1,1), inventory_generation='new-inventory', build_root=str(self.root.path)))
        with self.assertRaises(JenkinsError) as caught:
            manager.admit('new', Capacity(1,1,1), inventory_generation='new-inventory', build_root=str(self.root.path), writer_key='old-writer')
        self.assertEqual('pool_writer_active', caught.exception.code)

    def test_conflict_incomplete_stale_and_untrusted_fail_closed(self):
        for mutation in ('version', 'stale', 'partial', 'override', 'missing_budget', 'explicit_root', 'spec_digest', 'future', 'bool_capacity'):
            with self.subTest(mutation=mutation):
                original_args = dict(self.args); original_inventory = dict(self.inventory); original_policy = dict(self.policy)
                if mutation == 'version': self.args['expectedPolicyVersion'] = 999
                if mutation == 'stale': self.inventory['observedAt'] = time.time()-31
                if mutation == 'partial': self.inventory['complete'] = False
                if mutation == 'override': self.args['cpuBudget'] = 999
                if mutation == 'missing_budget': self.policy.pop('cpuBudget')
                if mutation == 'explicit_root': self.args['buildRoot'] = 'F:\\cargo-targets'
                if mutation == 'spec_digest': self.args['expectedSpecSha256'] = '0'*64
                if mutation == 'future': self.inventory['observedAt'] = time.time()+60
                if mutation == 'bool_capacity': self.inventory['cpu'] = True
                with self.assertRaises(JenkinsError): self.run_activation()
                self.assertEqual(self.old, self.state.get('resource_policy', 'default'))
                self.assertIsNone(self.state.get('capacity_inventory', 'new-inventory'))
                self.args.clear(); self.args.update(original_args)
                self.inventory.clear(); self.inventory.update(original_inventory)
                self.policy.clear(); self.policy.update(original_policy)

    def test_holds_changing_during_scan_aborts_policy_publication(self):
        def change(root):
            self.state.put('resource_reservation', 'racer', {'status': 'active'})
            return self.inventory
        self.capture.side_effect = change
        with self.assertRaises(JenkinsError) as caught: self.run_activation()
        self.assertEqual('activation_holds_changed', caught.exception.code)
        self.assertEqual(self.old, self.state.get('resource_policy', 'default'))

    def test_spec_changing_during_scan_aborts_policy_publication(self):
        def change(root):
            self.spec_path.write_text('{}')
            return self.inventory
        self.capture.side_effect = change
        with self.assertRaises(JenkinsError) as caught: self.run_activation()
        self.assertEqual('deployment_spec_changed', caught.exception.code)
        self.assertEqual(self.old, self.state.get('resource_policy', 'default'))

    def test_query_policy_reconciles_without_scan_or_creating_namespace(self):
        from tools.jenkins.resources import handle
        with patch.object(ResourceManager, 'capture_inventory', side_effect=AssertionError('scan')):
            result = handle('query', {'view': 'policy'}, self.state, self.repo)
        self.assertEqual(self.old, result['result']['policy'])
        self.assertFalse(self.namespace.exists())

    def test_policy_CAS_rolls_back_inventory_if_version_changes_during_scan(self):
        def change(root):
            self.state.put('resource_policy', 'default', {'buildRoot': 'racer'})
            return self.inventory
        self.capture.side_effect = change
        with self.assertRaises(JenkinsError) as caught: self.run_activation()
        self.assertEqual('state_conflict', caught.exception.code)
        self.assertIsNone(self.state.get('capacity_inventory', 'new-inventory'))

    def test_missing_default_or_malformed_recipe_estimates_are_rejected(self):
        for value in (None, [], {}, {'tiny-fixture': {'cpu': 1, 'memoryBytes': 2, 'diskBytes': 2}},
                      {'default': None}, {'default': {'cpu': 1}},
                      {'default': {'cpu': True, 'memoryBytes': 2, 'diskBytes': 2}},
                      {'default': {'cpu': 4, 'memoryBytes': 2, 'diskBytes': 2}},
                      {'default': {'cpu': 1, 'memoryBytes': 101, 'diskBytes': 2}},
                      {'default': {'cpu': 1, 'memoryBytes': 2, 'diskBytes': 101}},
                      {'default': {'cpu': -1, 'memoryBytes': 2, 'diskBytes': 2}},
                      {'default': {'cpu': 1, 'memoryBytes': 0, 'diskBytes': 2}}):
            with self.subTest(value=value):
                self.policy['recipeEstimates'] = value
                with self.assertRaises(JenkinsError) as caught: self.run_activation()
                self.assertEqual('resource_policy_invalid', caught.exception.code)
                self.assertEqual(self.old, self.state.get('resource_policy', 'default'))
                self.assertIsNone(self.state.get('capacity_inventory', 'new-inventory'))

    def test_payload_cannot_override_trusted_recipe_estimates(self):
        self.args['recipeEstimates'] = {'default': {'cpu': 1, 'memoryBytes': 1, 'diskBytes': 1}}
        with self.assertRaises(JenkinsError) as caught: self.run_activation()
        self.assertEqual('activation_policy_untrusted', caught.exception.code)
        self.assertEqual(self.old, self.state.get('resource_policy', 'default'))
