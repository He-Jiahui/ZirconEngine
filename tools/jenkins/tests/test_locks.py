"""Native lock ownership survives a second opener and is released on exit."""
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from tools.jenkins.contracts import JenkinsError
from tools.jenkins.state.locks import process_lock


class ProcessLockTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.path = Path(self.temp.name) / "operation.lock"

    def test_empty_historical_file_does_not_claim_an_operation(self):
        self.path.touch()
        with process_lock(self.path):
            with self.assertRaises(JenkinsError):
                with process_lock(self.path):
                    self.fail("A second owner entered")
        with process_lock(self.path):
            pass

    def test_abrupt_owner_exit_releases_lock_without_deleting_history(self):
        script = ("import os,sys; from tools.jenkins.state.locks import process_lock; "
                  "lock=process_lock(sys.argv[1]); lock.__enter__(); "
                  "print('locked',flush=True); sys.stdin.read(1); os._exit(0)")
        child = subprocess.Popen([sys.executable, "-B", "-c", script, str(self.path)],
                                 stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                 text=True)
        try:
            self.assertEqual("locked", child.stdout.readline().strip())
            with self.assertRaises(JenkinsError):
                with process_lock(self.path):
                    self.fail("A second process entered")
            child.communicate("x", timeout=10)
            self.assertEqual(0, child.returncode)
            with process_lock(self.path):
                pass
            self.assertTrue(self.path.exists())
        finally:
            if child.poll() is None:
                child.kill()
                child.communicate()
