from __future__ import annotations

import contextlib
import io
import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from tools.dev import local_cargo


@unittest.skipUnless(os.name == "nt", "Windows storage contract")
class LocalCargoTests(unittest.TestCase):
    def test_only_physical_drive_roots_are_accepted(self) -> None:
        for path in (r"D:\cargo-targets\zircon-local\test", r"E:\cargo-targets\zircon-local\test", r"F:\cargo-targets\zircon-local\test"):
            self.assertEqual(Path(path), local_cargo.validate_output_path(path))
        for path in (r"C:\cargo-targets\test", r"D:\targets\test", r"E:\ZirconBuilds\test", r"E:\nested\cargo-targets\test", r"D:\cargo-targets", r"D:\cargo-targets\..\elsewhere", "target"):
            with self.subTest(path=path), self.assertRaises(ValueError):
                local_cargo.validate_output_path(path)

    def test_junction_alias_cannot_admit_an_outside_directory(self) -> None:
        from tools.jenkins.resources import paths
        def attributes(path):
            return paths._REPARSE if path.endswith("aliased") else 0
        with patch.object(paths, "_attrs", side_effect=attributes), patch.object(paths, "_final_path", side_effect=lambda value: value if value == r"D:\cargo-targets" else (_ for _ in ()).throw(AssertionError("resolved alias"))):
            with self.assertRaises(ValueError):
                local_cargo.validate_output_path(r"D:\cargo-targets\aliased\test")

    def test_raw_windows_alias_components_are_rejected(self) -> None:
        for value in (r"D:\cargo-targets\target.", r"D:\cargo-targets\target ",
                      r"D:\cargo-targets\NUL.bin", r"D:\cargo-targets\target:stream",
                      r"D:\cargo-targets\a\.\b", r"D:\cargo-targets\a\\b"):
            with self.subTest(value=value), self.assertRaises(ValueError):
                local_cargo.validate_output_path(value)

    def test_dry_run_never_creates_output_or_contacts_coordinator(self) -> None:
        out = io.StringIO()
        with patch.object(Path, "mkdir", side_effect=AssertionError("directory created")), patch.object(local_cargo.subprocess, "run", side_effect=AssertionError("process started")), contextlib.redirect_stdout(out):
            code = local_cargo.main(["--target-dir", r"D:\cargo-targets\zircon-local\dry-test", "--dry-run", "--", "check", "-p", "zircon_runtime", "--locked"])
        self.assertEqual(0, code)
        report = json.loads(out.getvalue())
        self.assertEqual("dry_run", report["status"])
        self.assertIn("--target-dir", report["command"])
        self.assertFalse(report["formalAcceptance"])

    def test_target_override_and_cargo_configuration_cannot_escape_preflight(self) -> None:
        for args in (["check", "--target-dir", "C:/bad"], ["check", "--target-dir=C:/bad"], ["--config", "build.target-dir='C:/bad'", "check"], ["check", "--config=build.build-dir='C:/bad'"]):
            with self.subTest(args=args), self.assertRaises(ValueError):
                local_cargo.prepare_command(args, Path(r"D:\cargo-targets\zircon-local\test"))

    def test_runtime_cdylib_command_uses_same_physical_preflight(self) -> None:
        command = local_cargo.prepare_command(['rustc', '-p', 'zircon_runtime', '--lib', '--crate-type', 'cdylib'], Path(r'D:\cargo-targets\zircon-local\runtime-dll'))
        self.assertEqual(['cargo', 'rustc', '-p', 'zircon_runtime', '--lib', '--crate-type', 'cdylib'], command[:7])
        self.assertIn('--locked', command)
        self.assertIn('--target-dir', command)
        for arguments in (['rustc', '--', '-o', 'C:/escape.exe'],
                          ['rustc', '--artifact-dir', 'C:/escape']):
            with self.assertRaises(ValueError):
                local_cargo.prepare_command(arguments, Path(r'D:\cargo-targets\zircon-local\runtime-dll'))

    def test_test_binary_arguments_are_preserved_after_separator(self) -> None:
        command = local_cargo.prepare_command(["test", "--lib", "--", "--exact", "--nocapture"], Path(r"D:\cargo-targets\zircon-local\test"))
        self.assertEqual(["--exact", "--nocapture"], command[command.index("--") + 1:])
        self.assertLess(command.index("--target-dir"), command.index("--"))

    def test_low_disk_rejects_before_creating_directories_or_running_cargo(self) -> None:
        with patch.object(local_cargo, "free_bytes", return_value=1024), patch.object(Path, "mkdir", side_effect=AssertionError("directory created")), patch.object(local_cargo.subprocess, "run", side_effect=AssertionError("Cargo ran")), contextlib.redirect_stderr(io.StringIO()):
            self.assertEqual(2, local_cargo.main(["--target-dir", r"D:\cargo-targets\zircon-local\low-disk", "--", "check"]))

    def test_powershell_wrapper_preserves_package_and_test_scope(self) -> None:
        wrapper = Path(__file__).resolve().parents[1] / 'dev/local-cargo.ps1'
        shell = Path(os.environ['SystemRoot']) / 'System32/WindowsPowerShell/v1.0/powershell.exe'
        result = subprocess.run([str(shell), '-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass', '-File', str(wrapper),
                                 '-DryRun', 'test', '-p', 'zircon_runtime', '--lib', 'focused_filter',
                                 '--locked', '--', '--exact'], capture_output=True, text=True,
                                encoding='utf-8', errors='replace', check=False)
        self.assertEqual(0, result.returncode, result.stderr)
        command = json.loads(result.stdout)['command']
        self.assertEqual('zircon_runtime', command[command.index('-p') + 1])
        self.assertIn('focused_filter', command)
        self.assertEqual(['--exact'], command[command.index('--') + 1:])

    def test_retired_conventions_do_not_delegate_to_old_managed_validator(self) -> None:
        from tools.audits.check_conventions import convention_commands_for_repo

        with tempfile.TemporaryDirectory() as folder:
            repo = Path(folder)
            marker = repo / '.codex/coordinator-retirement.json'
            marker.parent.mkdir()
            marker.write_text('{"schemaVersion":1,"status":"retired"}', encoding='utf-8')
            with patch('tools.audits.check_conventions._requires_managed_cargo_delegation', side_effect=AssertionError('old delegation used')):
                commands = convention_commands_for_repo(repo)
            for command in commands:
                if command.name in {'structure', 'clippy'}:
                    self.assertIn('tools.dev.local_cargo', command.argv)
                    self.assertNotIn('validate-matrix.ps1', ' '.join(command.argv))
                    self.assertIn('--locked', command.argv)


@unittest.skipUnless(os.name == "nt", "Real Windows retained Job evidence")
class LocalCargoEvidenceTests(unittest.TestCase):
    @contextlib.contextmanager
    def fixture(self):
        import shutil
        import uuid
        parent = local_cargo._evidence_output(Path(r"D:\cargo-targets\zircon-local\fixtures\local-cargo-evidence"))
        parent.mkdir(parents=True, exist_ok=True)
        local_cargo._evidence_output(parent)
        leaf = parent / uuid.uuid4().hex
        local_cargo._evidence_output(leaf).mkdir(exist_ok=False)
        original = leaf.stat()
        ownership = {"cleanup": True}
        try:
            yield leaf, ownership
        finally:
            if ownership["cleanup"]:
                physical = local_cargo._evidence_output(leaf)
                current = physical.stat()
                if physical.parent != parent or (current.st_dev, current.st_ino) != (original.st_dev, original.st_ino):
                    raise AssertionError("Exclusive fixture directory identity changed; preserved")
                # Refuse aliases before recursively removing only our own direct child.
                for child in physical.rglob("*"):
                    local_cargo.validate_output_path(child)
                shutil.rmtree(physical)

    def test_evidence_output_requires_physical_zircon_local_namespace(self):
        for value in (r"C:\cargo-targets\zircon-local\evidence", r"D:\cargo-targets\elsewhere",
                      r"D:\cargo-targets\zircon-local", r"D:\nested\cargo-targets\zircon-local\evidence",
                      r"D:\cargo-targets\zircon-local\a\..\evidence", r"E:\ZirconBuilds\evidence"):
            with self.subTest(path=value), self.assertRaises(ValueError):
                local_cargo._evidence_output(value)

    def test_source_facts_hash_actual_bytes_and_keep_observation_scope(self):
        import hashlib
        with self.fixture() as (leaf, _ownership):
            data = leaf / "Data.bin"
            data.write_bytes(b"first")
            expected = {"missing.bin": None, "Data.bin": hashlib.sha256(b"first").hexdigest()}
            before = local_cargo._source_facts(leaf, expected)
            self.assertEqual(expected, before["manifest"])
            canonical = json.dumps(expected, sort_keys=True, separators=(",", ":")).encode("ascii")
            self.assertEqual(hashlib.sha256(canonical).hexdigest(), before["manifestSha256"])
            self.assertFalse(before["immutableInputSeal"])
            self.assertEqual("requested-selected-files", before["scope"])
            data.write_bytes(b"second")
            self.assertNotEqual(before, local_cargo._source_facts(leaf, expected))
            with self.assertRaises(ValueError):
                local_cargo._source_facts(leaf, {"Data.bin": expected["Data.bin"], "data.bin": expected["Data.bin"]})

    def test_source_facts_rejects_git_control_components_before_hashing(self):
        from tools.jenkins.pilot import source_manifest
        repo = Path(r"D:\cargo-targets\zircon-local\input-guard-no-read")
        denied = (".git/index", ".GIT/config", "nested/.Git/logs/HEAD",
                  "external-zr_vm/.GiT/index")
        for name in denied:
            with self.subTest(name=name), patch.object(
                source_manifest, "file_hash", side_effect=AssertionError("Git control bytes read")
            ) as hasher:
                with self.assertRaisesRegex(ValueError, "Git control path"):
                    local_cargo._source_facts(repo, {name: None})
                hasher.assert_not_called()
        allowed = {".gitignore": "0" * 64, "assets/.gitkeep": "0" * 64}
        with patch.object(source_manifest, "file_hash", return_value="0" * 64) as hasher:
            observed = local_cargo._source_facts(repo, allowed)
        self.assertEqual(allowed, observed["manifest"])
        self.assertEqual(len(allowed), hasher.call_count)

    @staticmethod
    @contextlib.contextmanager
    def original_process_handle(started: dict):
        import ctypes
        from ctypes import wintypes
        api = ctypes.WinDLL("kernel32", use_last_error=True)
        api.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
        api.OpenProcess.restype = wintypes.HANDLE
        api.GetProcessTimes.argtypes = [wintypes.HANDLE] + [ctypes.POINTER(wintypes.FILETIME)] * 4
        api.GetProcessTimes.restype = wintypes.BOOL
        api.WaitForSingleObject.argtypes = [wintypes.HANDLE, wintypes.DWORD]
        api.WaitForSingleObject.restype = wintypes.DWORD
        api.GetExitCodeProcess.argtypes = [wintypes.HANDLE, ctypes.POINTER(wintypes.DWORD)]
        api.GetExitCodeProcess.restype = wintypes.BOOL
        api.CloseHandle.argtypes = [wintypes.HANDLE]
        api.CloseHandle.restype = wintypes.BOOL
        handle = api.OpenProcess(0x100000 | 0x1000, False, started["pid"])
        if not handle:
            raise ctypes.WinError(ctypes.get_last_error())
        try:
            times = [wintypes.FILETIME() for _ in range(4)]
            if not api.GetProcessTimes(handle, *(ctypes.byref(value) for value in times)):
                raise ctypes.WinError(ctypes.get_last_error())
            birth = str((times[0].dwHighDateTime << 32) | times[0].dwLowDateTime)
            if birth != started["creationTime"]:
                raise AssertionError("Original process creation time did not match")
            yield api, handle
        finally:
            api.CloseHandle(handle)

    @classmethod
    def wait_original_root(cls, started: dict):
        import ctypes
        from ctypes import wintypes
        with cls.original_process_handle(started) as (api, handle):
            if api.WaitForSingleObject(handle, 10000) != 0:
                raise AssertionError("Original root process object did not become terminal")
            code = wintypes.DWORD()
            if not api.GetExitCodeProcess(handle, ctypes.byref(code)):
                raise ctypes.WinError(ctypes.get_last_error())
            return code.value

    def test_retained_job_waits_for_real_descendant_and_actual_pipe_eof(self):
        import sys
        import threading
        import time
        with self.fixture() as (leaf, ownership):
            ready, release, finished = [leaf / name for name in ("ready", "release", "finished")]
            child = (
                "from pathlib import Path; import sys,time; "
                "ready,release,finished=map(Path,sys.argv[1:]); ready.write_text('ready'); "
                "deadline=time.monotonic()+30\n"
                "while not release.exists():\n"
                " if time.monotonic()>=deadline: raise SystemExit(9)\n"
                " time.sleep(.02)\n"
                "finished.write_text('actual child terminal'); print('actual-child-output',flush=True)"
            )
            root = (
                "import subprocess,sys; "
                "subprocess.Popen([sys.executable,'-B','-X','utf8','-c',sys.argv[1],*sys.argv[2:]],close_fds=False)"
            )
            result, failures = {}, []
            def run():
                try:
                    result.update(local_cargo._run_owned_command(
                        [sys.executable, "-B", "-X", "utf8", "-c", root, child,
                         str(ready), str(release), str(finished)],
                        leaf, dict(os.environ, TEMP=str(leaf), TMP=str(leaf), PYTHONDONTWRITEBYTECODE="1"), leaf, 40))
                except BaseException as error:
                    failures.append(error)
            worker = threading.Thread(target=run, name="real-local-command-fixture", daemon=True)
            worker.start()
            try:
                deadline = time.monotonic() + 10
                while not ready.exists() or not (leaf / "started.json").exists():
                    if failures or time.monotonic() >= deadline:
                        self.fail("Real descendant did not reach its bounded startup boundary")
                    time.sleep(.02)
                started = json.loads((leaf / "started.json").read_text(encoding="utf-8"))
                self.assertEqual(0, self.wait_original_root(started))
                self.assertTrue(worker.is_alive(), "Root exit must not finish the retained child Job")
                self.assertFalse(finished.exists())
                self.assertFalse(result)
            finally:
                release.write_text("release original child")
                worker.join(timeout=20)
                if worker.is_alive():
                    ownership["cleanup"] = False
                    self.fail("Owner did not settle after RAII release; physical fixture preserved")
            self.assertFalse(failures, failures)
            self.assertEqual("actual child terminal", finished.read_text())
            self.assertEqual(0, result["rootExitCode"])
            self.assertTrue(result["processTree"]["terminal"])
            self.assertTrue(result["processTree"]["nativeWaiterJoined"])
            self.assertTrue(result["processTree"]["pipeEOF"])
            self.assertEqual(0, result["nativeJobEvidence"]["records"]["afterClose"]["activeProcesses"])
            self.assertFalse(result["terminalErrors"])
            self.assertFalse(result["nativeJobEvidence"]["errors"])
            self.assertIn(b"actual-child-output", (leaf / "stdout.bin").read_bytes())


    def test_artifact_parser_accepts_exact_test_build_sibling_and_rejects_parent(self):
        import hashlib
        with self.fixture() as (leaf, _ownership):
            target, build = leaf / "target", leaf / "build"
            target.mkdir()
            (build / "debug/deps").mkdir(parents=True)
            stdout = leaf / "cargo-json.bin"
            binary = build / "debug/deps/parser-fixture.exe"
            data = b"parser fixture bytes; not a compiler output or provenance proof"
            binary.write_bytes(data)
            # This tests the Cargo JSON/path parser only, not a compiler or receipt.
            message = {
                "reason": "compiler-artifact", "package_id": "path+file:///parser-fixture#0.1.0",
                "manifest_path": str(leaf / "Cargo.toml"),
                "target": {"name": "parser-fixture", "kind": ["rlib"]},
                "profile": {"test": True}, "fresh": False, "executable": str(binary),
            }
            stdout.write_text(json.dumps(message) + "\n" + json.dumps({"reason": "build-finished", "success": True}) + "\n",
                              encoding="utf-8")
            fact = local_cargo._cargo_artifacts(stdout, target, build)
            self.assertEqual([str(target), str(build)], fact["outputDirectories"])
            artifact = fact["executableArtifacts"][0]
            self.assertEqual(["rlib"], artifact["target"]["kind"])
            self.assertTrue(artifact["profile"]["test"])
            self.assertEqual(hashlib.sha256(data).hexdigest(), artifact["binary"]["sha256"])
            target_binary = target / "parser-fixture.exe"
            target_binary.write_bytes(data)
            message["executable"] = str(target_binary)
            stdout.write_text(json.dumps(message) + "\n", encoding="utf-8")
            self.assertEqual(str(target_binary), local_cargo._cargo_artifacts(stdout, target, build)["executableArtifacts"][0]["binary"]["path"])
            for outside in (leaf / "outside.exe", leaf / "build-lookalike/parser-fixture.exe"):
                outside.parent.mkdir(exist_ok=True)
                outside.write_bytes(data)
                message["executable"] = str(outside)
                stdout.write_text(json.dumps(message) + "\n", encoding="utf-8")
                with self.subTest(path=outside), self.assertRaisesRegex(ValueError, "exact target/build"):
                    local_cargo._cargo_artifacts(stdout, target, build)
            with self.assertRaisesRegex(ValueError, "exact owned target sibling"):
                local_cargo._cargo_artifacts(stdout, target, leaf)

    def test_artifact_parser_proves_exact_custom_target_hardlinks(self):
        import hashlib
        from tools.jenkins.pilot.contracts import PilotError
        with self.fixture() as (leaf, _ownership):
            target, build = leaf / "compiler-cache", leaf / "build"
            (target / "debug").mkdir(parents=True)
            (build / "debug/deps").mkdir(parents=True)
            binary = target / "debug/custom-fixture.exe"
            data = b"custom-target native alias fixture; no compiler provenance"
            binary.write_bytes(data)
            os.link(binary, build / "debug/deps/custom-fixture.exe")
            stdout = leaf / "cargo-json.bin"
            message = {
                "reason": "compiler-artifact", "package_id": "custom-fixture#0.1.0",
                "manifest_path": str(leaf / "Cargo.toml"),
                "target": {"name": "custom-fixture", "kind": ["bin"]},
                "profile": {"test": False}, "fresh": False, "executable": str(binary),
            }
            stdout.write_text(json.dumps(message) + "\n", encoding="utf-8")
            fact = local_cargo._cargo_artifacts(stdout, target, build)
            self.assertEqual([str(target), str(build)], fact["outputDirectories"])
            self.assertEqual({"path": str(binary), "sha256": hashlib.sha256(data).hexdigest(),
                              "byteLength": len(data)}, fact["executableArtifacts"][0]["binary"])
            with self.assertRaises(PilotError):
                local_cargo._file_fact(binary)
            # A directory named target is outside this caller's compiler roots.
            (leaf / "target/debug").mkdir(parents=True)
            os.link(binary, leaf / "target/debug/outside-fixture.exe")
            with self.assertRaisesRegex(ValueError, "outside the exact compiler roots"):
                local_cargo._cargo_artifacts(stdout, target, build)

    def test_evidence_rejects_collapsed_compiler_roots_before_native_start(self):
        with self.fixture() as (leaf, _ownership):
            target = leaf / "build"
            output = leaf / "command-evidence"
            environment = {"CARGO_TARGET_DIR": str(target), "CARGO_BUILD_BUILD_DIR": str(target)}
            with patch.object(local_cargo, "_run_owned_command",
                              side_effect=AssertionError("native command started")):
                with self.assertRaisesRegex(ValueError, "must be distinct"):
                    local_cargo._run_with_evidence(
                        ["cargo", "build", "--message-format=json"], leaf, target, environment,
                        str(output), str(leaf / "unused-inputs.json"), 1,
                    )
            self.assertFalse(output.exists())
            self.assertFalse(target.exists())

    def test_timeout_cli_exits_and_preserves_failure_with_real_process_terminal_oracle(self):
        import sys
        import time
        with self.fixture() as (leaf, ownership):
            ready, release = leaf / "ready.json", leaf / "release"
            repo = Path(__file__).resolve().parents[2]
            blocker = (
                "from pathlib import Path; import json,os,sys,time; "
                "from tools.jenkins.pilot.native.process_identity import process_creation_time; "
                "ready,release=map(Path,sys.argv[1:]); "
                "temporary=ready.with_suffix('.tmp'); "
                "temporary.write_text(json.dumps({'pid':os.getpid(),'creationTime':process_creation_time(os.getpid())})); "
                "temporary.replace(ready); deadline=time.monotonic()+30\n"
                "while not release.exists():\n"
                " if time.monotonic()>=deadline: raise SystemExit(9)\n"
                " time.sleep(.02)\n"
            )
            cli = (
                "from pathlib import Path; import os,sys; "
                "from tools.dev.local_cargo import _run_owned_command,_write_fact; "
                "leaf,repo=map(Path,sys.argv[1:3]); "
                "result=_run_owned_command([sys.executable,'-B','-X','utf8','-c',sys.argv[3],"
                "str(leaf/'ready.json'),str(leaf/'release')],repo,dict(os.environ),leaf,5); "
                "_write_fact(leaf/'failure-result.json',result); "
                "raise SystemExit(0 if result['processTree']['terminal'] else 2)"
            )
            process = subprocess.Popen(
                [sys.executable, "-B", "-X", "utf8", "-c", cli, str(leaf), str(repo), blocker],
                cwd=repo, env=dict(os.environ, TEMP=str(leaf), TMP=str(leaf), PYTHONDONTWRITEBYTECODE="1"),
                stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
            )
            original_context = None
            context_entered = False
            try:
                deadline = time.monotonic() + 10
                while not ready.exists():
                    if process.poll() is not None or time.monotonic() >= deadline:
                        self.fail("Real timeout blocker did not reach bounded startup")
                    time.sleep(.02)
                original = json.loads(ready.read_text(encoding="utf-8"))
                original_context = self.original_process_handle(original)
                api, handle = original_context.__enter__()
                context_entered = True
                self.assertEqual(258, api.WaitForSingleObject(handle, 0), "Original blocker must really be live")
                self.assertEqual(2, process.wait(timeout=20), "Failed helper must allow its interpreter to exit")
                self.assertEqual(0, api.WaitForSingleObject(handle, 5000), "Original blocker process object must signal")
                result = json.loads((leaf / "failure-result.json").read_text(encoding="utf-8"))
                self.assertFalse(result["processTree"]["terminal"])
                self.assertTrue(any(item["type"] == "TimeoutError" for item in result["terminalErrors"]),
                                result["terminalErrors"])
                self.assertTrue(all(item["closed"] for item in result["processTree"]["pipes"].values()))
                self.assertFalse(result["formalAcceptance"])
            finally:
                # Release before any own-process kill; never target a PID lookup.
                release.write_text("release original timeout fixture")
                try:
                    if process.poll() is None:
                        process.kill()
                    process.wait(timeout=5)
                    if context_entered:
                        if api.WaitForSingleObject(handle, 5000) != 0:
                            raise AssertionError("Original blocker is still live; fixture preserved")
                    else:
                        # Without its original object, terminal ownership is unproven.
                        ownership["cleanup"] = False
                except BaseException:
                    ownership["cleanup"] = False
                    raise
                finally:
                    if context_entered:
                        original_context.__exit__(None, None, None)

if __name__ == "__main__":
    unittest.main()
