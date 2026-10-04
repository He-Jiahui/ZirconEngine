from __future__ import annotations
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from tools.jenkins.tray.formal import status
from tools.jenkins.tray.formal_startup import VALUE

class FormalSafetyTests(unittest.TestCase):
    def test_lost_lifecycle_is_degraded_and_cannot_be_stopped_by_pid(self):
        import json
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / 'deployment-operation.json').write_text(json.dumps({
                'component': 'lifecycle-host', 'state': 'running', 'hostPid': 1,
                'creationTime': 'old', 'executable': 'python.exe'}))
            config = type('C', (), {'url': 'http://127.0.0.1:53748/',
                'paths': type('P', (), {'state': root})(), 'spec': object(),
                'java': root / 'java', 'war': root / 'war'})()
            with patch('tools.jenkins.tray.formal.DeploymentManager.health', return_value={
                    'controller': False, 'ready': False}), \
                 patch('tools.jenkins.tray.formal.identity', return_value=None):
                result = status(config)
            self.assertEqual('degraded', result['state'])
            self.assertTrue(result['ownerLost'])
            self.assertTrue(result['canStart'])
            self.assertFalse(result['canStop'])

    def test_unknown_owner_cannot_stop(self):
        class C:
            url='http://127.0.0.1:53748/'
            state_file=Path(tempfile.gettempdir())/'missing-formal-state.json'
            paths=type('P',(),{'state':Path(tempfile.gettempdir())})()
            spec=type('S',(),{'controller':{'listenAddress':'127.0.0.1','httpPort':53748}})()
            java=war=Path('missing')
        with patch('tools.jenkins.tray.formal.DeploymentManager.health', return_value={'controller':True,'agent':True,'plugins':True,'ready':True}), patch('tools.jenkins.tray.formal.identity', return_value=None):
            result=status(C())
        self.assertFalse(result['canStop'])

    def test_startup_value_name_is_formal(self):
        self.assertEqual(VALUE, 'ZirconFormalJenkinsTray')

if __name__ == '__main__': unittest.main()
