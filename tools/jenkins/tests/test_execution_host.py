from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path

from tools.jenkins.contracts import JenkinsError
from tools.jenkins.processes.execution_host import ExecutionHost, request_cancel
from tools.jenkins.state import State


class ExecutionHostTests(unittest.TestCase):
    def setUp(self):
        test_root = Path.cwd() / ".jenkins" / "tmp" / "execution-host-tests"
        test_root.mkdir(parents=True, exist_ok=True)
        self.tmp = tempfile.TemporaryDirectory(dir=test_root)
        self.root = Path(self.tmp.name)
        self.state = State(self.root / "state.sqlite3")
        self.state.put("sealed_input", "src", {"status": "sealed", "repositoryRoot": str(self.root), "preparationRoot": str(self.root / "prepared")})
        (self.root / "prepared").mkdir()
        recipe = {"kind": "fixture", "command": [sys.executable, "-B", "-c", "import time; time.sleep(30)"],
            "preparationRoot": str(self.root / "prepared"), "environment": {}}
        from tools.jenkins.contracts import digest
        recipe_ref = digest(recipe)
        self.state.put("recipe_plan", recipe_ref, {"recipe": recipe, "recipeDigest": recipe_ref, "status": "planned"})
        self.state.put("execution", "exec", {"status": "claimed", "recipeRef": recipe_ref,
            "sealedInputRef": "src", "consumers": [], "refs": 0})

    def tearDown(self):
        for obj in getattr(self, "_fixture_hosts", []):
            if obj.job is not None and obj.job.job_handle:
                if not obj.job.proof().complete:
                    obj.job.terminate(timeout_seconds=10)
                obj.job.close()
        self.tmp.cleanup()

    def test_rejects_caller_recipe_fallback(self):
        self.state.put("execution", "bad", {"status": "claimed", "recipeRef": "missing",
            "sealedInputRef": "src"})
        with self.assertRaises(JenkinsError):
            ExecutionHost(self.state, "bad", operation_id="op", generation="gen").start()

    def test_stale_control_does_not_terminate_owner(self):
        host = ExecutionHost(self.state, "exec", operation_id="op", generation="gen")
        self._fixture_hosts = [host]
        initial = host.start()
        with self.assertRaises(JenkinsError):
            request_cancel(self.state, "exec", operation_id="op", generation="gen",
                           host_pid=initial["hostPid"], host_birth_token="wrong", cancel_epoch=0)
        self.assertIsNone(host.job.poll())
        result = host.serve(timeout=0.01)
        self.assertEqual("running", result["status"])
        host.job.terminate(timeout_seconds=5)
        host.reconcile()

    def test_cross_process_cancel_requires_zero_refs_and_full_identity(self):
        # The owner itself is an independent Python process.  The test process
        # only reads its durable identity and submits a control record.
        self.state.put("execution_consumer", "exec:consumer", {"executionId": "exec", "consumerId": "consumer", "status": "active"})
        env = dict(os.environ)
        env["PYTHONPATH"] = str(Path.cwd())
        cmd = [sys.executable, "-B", "-m", "tools.jenkins.processes.execution_host",
               "--state", str(self.root / "state.sqlite3"), "--execution", "exec",
               "--operation", "op", "--generation", "gen"]
        proc = subprocess.Popen(cmd, cwd=str(Path.cwd()), env=env, stdout=subprocess.PIPE,
                                stderr=subprocess.PIPE, text=True)
        try:
            deadline = time.time() + 10
            owner = None
            while time.time() < deadline:
                row = self.state.get("execution_host", "exec")
                if row:
                    owner = row["payload"]
                    break
                time.sleep(.05)
            if owner is None:
                if proc.poll() is None:
                    proc.kill()
                out, err = proc.communicate(timeout=5)
                self.fail(f"owner host did not register: return={proc.returncode} stderr={err!r} stdout={out!r}")
            with self.assertRaises(JenkinsError):
                request_cancel(self.state, "exec", operation_id="op", generation="gen",
                               host_pid=owner["hostPid"], host_birth_token=owner["hostBirthToken"], cancel_epoch=0)
            self.state.put("execution_consumer", "exec:consumer", {"executionId": "exec", "consumerId": "consumer", "status": "released"}, expected_version=self.state.get("execution_consumer", "exec:consumer")["version"])
            request_cancel(self.state, "exec", operation_id="op", generation="gen",
                           host_pid=owner["hostPid"], host_birth_token=owner["hostBirthToken"], cancel_epoch=0)
            out, err = proc.communicate(timeout=10)
            self.assertEqual(0, proc.returncode, (err + "\nstdout=" + (out or "")))
            terminal = self.state.get("execution", "exec")["payload"]
            self.assertEqual("claimed", terminal["status"])
            host_terminal = self.state.get("execution_host", "exec")["payload"]
            self.assertIn(host_terminal["status"], {"cancelled", "terminal"})
            native = self.state.get("native_job", host_terminal["nativeJobId"])["payload"]
            self.assertTrue(native["completeProof"])
            self.assertEqual("terminal", native["status"])
            if proc.stdout: proc.stdout.close()
            if proc.stderr: proc.stderr.close()
        finally:
            if proc.poll() is None:
                proc.kill()
                proc.wait(timeout=5)
            # Ensure output pipes are closed before TemporaryDirectory cleanup.
            if proc.stdout: proc.stdout.close()
            if proc.stderr: proc.stderr.close()

    @unittest.skipUnless(os.name == "nt", "requires Windows Job and retained process handles")
    def test_restarted_cli_cannot_guess_or_pid_kill(self):
        import ctypes
        host = ExecutionHost(self.state, "exec", operation_id="op", generation="gen")
        self._fixture_hosts = [host]
        payload = host.start()
        # Retain only a test-owned process wait handle. It is not a Job
        # handle and cannot manufacture a whole-tree/EOF production proof.
        api = ctypes.WinDLL("kernel32", use_last_error=True)
        api.GetCurrentProcess.restype = ctypes.c_void_p
        api.DuplicateHandle.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p,
            ctypes.POINTER(ctypes.c_void_p), ctypes.c_uint32, ctypes.c_bool, ctypes.c_uint32]
        api.DuplicateHandle.restype = ctypes.c_bool
        api.WaitForSingleObject.argtypes = [ctypes.c_void_p, ctypes.c_uint32]
        api.WaitForSingleObject.restype = ctypes.c_uint32
        api.CloseHandle.argtypes = [ctypes.c_void_p]
        retained = ctypes.c_void_p()
        current = api.GetCurrentProcess()
        self.assertTrue(api.DuplicateHandle(current, int(host.job.process.process._handle), current,
                                            ctypes.byref(retained), 0, False, 2))
        host.job.close()  # Simulate a CLI restart: no owner handle is retained.
        try:
            restarted = ExecutionHost(self.state, "exec", operation_id="op", generation="gen")
            result = restarted.reconcile()
            self.assertEqual("unknown", result["status"])
            self.assertEqual("owner_handle_unavailable", result["reason"])
            self.assertEqual("claimed", self.state.get("execution", "exec")["payload"]["status"])
            self.assertEqual(0, api.WaitForSingleObject(retained, 10000))
            self.assertIsNone(self.state.get("native_job", payload["nativeJobId"])["payload"].get("completeProof"))
        finally:
            api.CloseHandle(retained)

    def test_multi_phase_records_terminal_proofs_and_logs(self):
        recipe_row = self.state.get("recipe_plan", next(iter([r["key"] for r in self.state.list("recipe_plan")])))
        recipe = dict(recipe_row["payload"]["recipe"])
        recipe["phases"] = ["compile", "list", "test"]
        recipe["commands"] = {
            phase: [sys.executable, "-B", "-c", f"print({phase!r})"]
            for phase in recipe["phases"]
        }
        from tools.jenkins.contracts import digest
        ref = digest(recipe)
        self.state.put("recipe_plan", ref, {"recipe": recipe, "recipeDigest": ref, "status": "planned"})
        self.state.put("execution", "exec", {"status": "claimed", "recipeRef": ref,
            "sealedInputRef": "src", "consumers": [], "refs": 0},
            expected_version=self.state.get("execution", "exec")["version"])
        host = ExecutionHost(self.state, "exec", operation_id="op", generation="gen")
        result = host.serve_phases()
        self.assertEqual("terminal", result["status"])
        self.assertEqual(["compile", "list", "test"], [r["phase"] for r in result["phaseRecords"]])
        self.assertTrue(all(r["terminalProof"]["complete"] for r in result["phaseRecords"]))
        self.assertTrue(all(Path(r["stdoutPath"]).is_file() for r in result["phaseRecords"]))
        self.assertTrue(all(Path(r["stderrPath"]).is_file() for r in result["phaseRecords"]))

    def test_multi_phase_nonzero_exit_is_durable_failure(self):
        recipe_row = self.state.get("recipe_plan", next(iter([r["key"] for r in self.state.list("recipe_plan")])))
        recipe = dict(recipe_row["payload"]["recipe"])
        recipe["phases"] = ["compile", "test"]
        recipe["commands"] = {
            "compile": [sys.executable, "-B", "-c", "print('ok')"],
            "test": [sys.executable, "-B", "-c", "raise SystemExit(7)"],
        }
        from tools.jenkins.contracts import digest
        ref = digest(recipe)
        self.state.put("recipe_plan", ref, {"recipe": recipe, "recipeDigest": ref, "status": "planned"})
        self.state.put("execution", "exec", {"status": "claimed", "recipeRef": ref,
            "sealedInputRef": "src", "consumers": [], "refs": 0},
            expected_version=self.state.get("execution", "exec")["version"])
        host = ExecutionHost(self.state, "exec", operation_id="op", generation="gen")
        with self.assertRaises(JenkinsError) as caught:
            host.serve_phases()
        self.assertEqual("phase_failed", caught.exception.code)
        saved = self.state.get("execution_host", "exec")["payload"]
        self.assertEqual("failed", saved["status"])
        self.assertEqual("test", saved["failedPhase"])
        self.assertEqual(["compile", "test"], [r["phase"] for r in saved["phaseRecords"]])

    def test_multi_phase_propagates_sealed_execution_environment(self):
        recipe_row = self.state.get("recipe_plan", next(iter([r["key"] for r in self.state.list("recipe_plan")])))
        recipe = dict(recipe_row["payload"]["recipe"])
        recipe["phases"] = ["compile"]
        recipe["commands"] = {"compile": [sys.executable, "-B", "-c", "import os; print(os.environ['ZIRCON_PHASE_TEST'])"]}
        recipe["executionEnvironment"] = {"ZIRCON_PHASE_TEST": "sealed-value"}
        from tools.jenkins.contracts import digest
        ref = digest(recipe)
        self.state.put("recipe_plan", ref, {"recipe": recipe, "recipeDigest": ref, "status": "planned"})
        self.state.put("execution", "exec", {"status": "claimed", "recipeRef": ref,
            "sealedInputRef": "src", "consumers": [], "refs": 0},
            expected_version=self.state.get("execution", "exec")["version"])
        host = ExecutionHost(self.state, "exec", operation_id="op", generation="gen")
        result = host.serve_phases()
        self.assertEqual("terminal", result["status"])
        output = Path(result["phaseRecords"][0]["stdoutPath"]).read_text(encoding="utf-8")
        self.assertIn("sealed-value", output)

    def test_multi_phase_cancel_after_current_native_identity_is_persisted(self):
        recipe_row = self.state.get("recipe_plan", next(iter([r["key"] for r in self.state.list("recipe_plan")])))
        recipe = dict(recipe_row["payload"]["recipe"])
        recipe["phases"] = ["compile", "test"]
        recipe["commands"] = {phase: [sys.executable, "-B", "-c", "import time; time.sleep(30)"] for phase in recipe["phases"]}
        from tools.jenkins.contracts import digest
        ref = digest(recipe)
        self.state.put("recipe_plan", ref, {"recipe": recipe, "recipeDigest": ref, "status": "planned"})
        self.state.put("execution", "exec", {"status": "claimed", "recipeRef": ref,
            "sealedInputRef": "src", "consumers": [], "refs": 0},
            expected_version=self.state.get("execution", "exec")["version"])
        host = ExecutionHost(self.state, "exec", operation_id="op", generation="gen", poll_interval=.01)
        import threading
        result_box = []
        thread = threading.Thread(target=lambda: result_box.append(host.serve_phases()), daemon=True)
        thread.start()
        deadline = time.time() + 5
        while time.time() < deadline:
            row = self.state.get("execution_host", "exec")
            if row and row["payload"].get("nativeJobId"):
                break
            time.sleep(.02)
        row = self.state.get("execution_host", "exec")
        self.assertTrue(row and row["payload"].get("nativeJobId"))
        self.state.put("execution_control", "exec", {"kind": "cancel", "cancelEpoch": 1})
        thread.join(timeout=10)
        self.assertFalse(thread.is_alive())
        self.assertEqual("cancelled", result_box[0]["status"])
        self.assertTrue(result_box[0]["terminalProof"]["complete"])


if __name__ == "__main__":
    unittest.main()
