from pathlib import Path
import json
import os
import subprocess
import unittest
from unittest.mock import patch
from tools.jenkins.pilot.contracts import RequestIdentity, PilotError, canonical_json, digest
from tools.jenkins.pilot.managed_cargo import (CargoExecution, collect_evidence, metadata_execution, terminal_reconcile,
    configuration_identity, execution_environment, executor_manifest, terminal_verified, validation_command)
from tools.jenkins.pilot.cargo_cache import CargoGeneration, MIN_FREE_BYTES, require_capacity, require_link_path_budget, generation_path
from tools.jenkins.pilot.storage import DEFAULT_PILOT_ROOT, ManagedStorage
from tools.jenkins.pilot.native.paths import StorageSecurityError
from tools.jenkins.pilot.tests.fixtures import temporary_directory


def observation(command):
    return {"command": command, "commandHash": digest(command), "exitCode": 0,
        "processTree": {"terminal": True, "readersFinished": True, "pipeEOF": True,
            "pid": 123, "creationTime": "134353522223204528",
            "pipes": {"stdout": {"eof": True}, "stderr": {"eof": True}}},
        "nativeJobEvidence": {"records": {"afterClose": {"activeProcesses": 0}}, "errors": []},
        "terminalErrors": []}


class ManagedCargoTests(unittest.TestCase):
    def test_retired_command_cannot_launch(self):
        with self.assertRaises(PilotError):
            validation_command()

    def test_native_terminal_requires_zero_descendants_and_actual_eof(self):
        proof = observation(["cargo"])
        self.assertTrue(terminal_verified(proof))
        for field in ("terminal", "pipeEOF", "readersFinished"):
            failed = json.loads(canonical_json(proof))
            failed["processTree"][field] = False
            self.assertFalse(terminal_verified(failed))
        for count in (True, 1, None):
            failed = json.loads(canonical_json(proof))
            failed["nativeJobEvidence"]["records"]["afterClose"]["activeProcesses"] = count
            self.assertFalse(terminal_verified(failed))
        failed = json.loads(canonical_json(proof))
        failed["processTree"]["pipes"]["stderr"]["error"] = "read failed"
        self.assertFalse(terminal_verified(failed))

    def test_no_inherited_cache_or_compiler_wrapper(self):
        toolchain = {"binaries": {name: {"path": name + ".exe"} for name in ("rustc", "rustdoc")}}
        with patch.dict(os.environ, {"CARGO_HOME": "C:/bad", "RUSTC_WRAPPER": "sccache",
                                   "RUSTFLAGS": "--out-dir=C:/bad", "SCCACHE_SERVER_PORT": "4226",
                                   "CARGO_HTTP_PROXY": "http://127.0.0.1:7897", "HTTPS_PROXY": "http://foreign.invalid"}):
            env = execution_environment(toolchain, Path("D:/cargo-targets/test/g1"))
        self.assertEqual(env["RUSTC_WRAPPER"], "")
        self.assertNotIn("RUSTFLAGS", env)
        self.assertNotIn("SCCACHE_SERVER_PORT", env)
        self.assertEqual(env["CARGO_HTTP_PROXY"], "")
        self.assertNotIn("HTTPS_PROXY", env)
        self.assertEqual(env["CARGO_HOME"], str(Path("D:/cargo-targets/test/g1/cargo-home")))

    def test_executor_manifest_excludes_retired_namespace(self):
        manifest = executor_manifest(Path(__file__).resolve().parents[3])
        self.assertTrue(manifest)
        self.assertTrue(all(p.startswith("tools/jenkins_pilot/") and "/tests/" not in p for p in manifest))
        self.assertIn("tools/jenkins_pilot/cargo_cache.py", manifest)

    def test_redirecting_config_is_rejected(self):
        with temporary_directory("cargo-config-") as temporary:
            source = Path(temporary)
            (source / ".cargo").mkdir()
            (source / ".cargo/config.toml").write_text('[build]\ntarget-dir="C:/bad"\n')
            with self.assertRaises(PilotError):
                configuration_identity(source)

    def test_minimum_capacity_is_not_relaxed(self):
        with temporary_directory("cargo-free-") as temporary:
            with patch("tools.jenkins.pilot.cargo_cache.shutil.disk_usage") as usage:
                usage.return_value.free = MIN_FREE_BYTES - 1
                with self.assertRaises(PilotError):
                    require_capacity(Path(temporary))
                usage.return_value.free = MIN_FREE_BYTES
                self.assertEqual(require_capacity(Path(temporary)), MIN_FREE_BYTES)

    def test_short_physical_key_keeps_full_identity_and_rejects_collision(self):
        with temporary_directory("cargo-short-") as temporary, patch(
                "tools.jenkins.pilot.cargo_cache.require_capacity", return_value=MIN_FREE_BYTES):
            root = Path(temporary)
            first = CargoGeneration(root, {"fixed": "one"}, {"request": "one"}).acquire()
            self.assertEqual(len(first.path.parent.name), 20)
            self.assertEqual(first.state["compatibilityKey"], first.key)
            self.assertEqual(len(first.key), 64)
            first.release({"proof": "terminal"})
            second = CargoGeneration(root, {"fixed": "two"}, {"request": "two"})
            second.key = first.key[:20] + "f" * 44
            second.path = generation_path(root, second.key)
            second.storage = ManagedStorage(second.path)
            with self.assertRaisesRegex(PilotError, "collision"):
                second.acquire()

    def test_real_link_failure_path_has_safe_shortened_budget(self):
        root = Path(r"D:\cargo-targets\zircon-jenkins\jenkins-pilot-independent-001-abb0b3960a4a29e420358419ea5687a7")
        key = "51998974d4788311d4c6499d12745f36292c2697e08295b8a8d9fc0e01f7740b"
        sample = "build/debug/build/proc-macro2-27e7ed934e6eec10/build_script_build-27e7ed934e6eec10.exe"
        self.assertGreaterEqual(len(str(root / "cargo-generations" / key / "g1" / sample)), 260)
        short = generation_path(root, key)
        self.assertLess(require_link_path_budget(short), 260)
        self.assertLess(len(str(short / sample)), 260)
        with self.assertRaises(PilotError):
            require_link_path_budget(short / ("extra" * 20))

    def test_single_writer_unknown_owner_and_reuse(self):
        with temporary_directory("cargo-exclusive-") as temporary, patch(
                "tools.jenkins.pilot.cargo_cache.require_capacity", return_value=MIN_FREE_BYTES):
            root = Path(temporary)
            first = CargoGeneration(root, {"tool": "fixed"}, {"request": "one"}).acquire()
            self.assertFalse(first.reused)
            second = CargoGeneration(root, {"tool": "fixed"}, {"request": "two"})
            with self.assertRaises(OSError):
                second.acquire()
            first.close()
            with self.assertRaises(PilotError):
                second.acquire()
            # Explicit test-only repair, representing independently verified terminal evidence.
            state = json.loads(first.storage.path("state.json").read_bytes())
            state["state"] = "released"
            state["terminalBinding"] = {"proof": "test-independent-terminal"}
            first.storage.atomic_write("releases/test.json", canonical_json(state))
            state.update({"releaseRecord": "releases/test.json", "releaseRecordHash": digest(state)})
            first.storage.atomic_write("state.json", canonical_json(state))
            second.acquire()
            self.assertTrue(second.reused)
            second.release({"proof": "terminal"})

    def test_reused_nested_directories_are_pinned_and_aliases_rejected(self):
        with temporary_directory("cargo-recursive-") as temporary, patch(
                "tools.jenkins.pilot.cargo_cache.require_capacity", return_value=MIN_FREE_BYTES):
            root = Path(temporary)
            first = CargoGeneration(root, {"tool": "nested"}, {"request": "one"}).acquire()
            first.release({"proof": "first-terminal"})
            with first.storage.backend() as backend:
                backend.ensure_directory("target/debug/deps")
                backend.write_bytes("target/debug/deps/artifact", b"compiled")
            second = CargoGeneration(root, first.compatibility, {"request": "two"}).acquire()
            with self.assertRaises(OSError):
                (second.path / "target/debug/deps").rename(second.path / "target/debug/replaced")
            second.release({"proof": "second-terminal"})
            os.link(second.path / "target/debug/deps/artifact", second.path / "target/debug/deps/alias")
            with self.assertRaises(StorageSecurityError):
                CargoGeneration(root, first.compatibility, {"request": "three"}).acquire()

    def test_nested_junction_rejected_before_reuse_and_at_terminal(self):
        # Keep these malicious fixture trees as physical review evidence.
        root = ManagedStorage(Path(os.environ.get("JENKINS_PILOT_ROOT", str(DEFAULT_PILOT_ROOT)))).temporary_directory(prefix="cargo-junction-")
        with patch("tools.jenkins.pilot.cargo_cache.require_capacity", return_value=MIN_FREE_BYTES):
            foreign = root / "foreign"
            with ManagedStorage(root).backend() as backend:
                backend.ensure_directory("foreign")
            first = CargoGeneration(root, {"tool": "junction"}, {"request": "one"}).acquire()
            first.release({"proof": "terminal"})
            subprocess.run(["cmd.exe", "/c", "mklink", "/J", str(first.path / "target/debug"), str(foreign)],
                           check=True, capture_output=True)
            with self.assertRaises(StorageSecurityError):
                CargoGeneration(root, first.compatibility, {"request": "two"}).acquire()
            live = CargoGeneration(root, {"tool": "terminal-junction"}, {"request": "live"}).acquire()
            subprocess.run(["cmd.exe", "/c", "mklink", "/J", str(live.path / "scratch/new-alias"), str(foreign)],
                           check=True, capture_output=True)
            try:
                with self.assertRaises(StorageSecurityError):
                    live.release({"proof": "terminal"})
                self.assertEqual(json.loads(live.backend.read_bytes("state.json"))["state"], "running")
            finally:
                live.close()

    def test_metadata_generation_is_fresh_and_unknown_terminal_never_reused(self):
        with temporary_directory("metadata-fresh-") as temporary, patch(
                "tools.jenkins.pilot.managed_cargo.require_capacity", return_value=MIN_FREE_BYTES), patch(
                "tools.jenkins.pilot.managed_cargo.toolchain_identity", return_value={"host": "host", "binaries": {
                    name: {"path": name + ".exe"} for name in ("cargo", "rustc", "rustdoc")}}):
            root = Path(temporary)
            source = root / "source"
            source.mkdir()
            (source / "Cargo.toml").write_text("[workspace]\nmembers=[]\n")
            unknown_root = root / "unknown-generation"
            with metadata_execution(source, unknown_root) as binding:
                bad = observation(list(binding.command))
                bad["processTree"]["pipeEOF"] = False
                with self.assertRaises(PilotError):
                    binding.complete(bad)
            self.assertEqual(json.loads((unknown_root / "metadata-state.json").read_bytes())["state"], "running")
            with self.assertRaises(OSError):
                with metadata_execution(source, unknown_root):
                    self.fail("unknown metadata generation was reused")
            terminal_root = root / "terminal-generation"
            with metadata_execution(source, terminal_root) as binding:
                good = observation(list(binding.command))
                good["exitCode"] = 1
                binding.complete(good)
            self.assertEqual(json.loads((terminal_root / "metadata-state.json").read_bytes())["state"], "released")

    def test_fixed_command_and_independent_receipt_binding(self):
        with temporary_directory("cargo-proof-") as temporary, patch(
                "tools.jenkins.pilot.cargo_cache.require_capacity", return_value=MIN_FREE_BYTES):
            root = Path(temporary)
            run = root / "runs/one"
            source = run / "sealed/source"
            source.mkdir(parents=True)
            (source / "Cargo.toml").write_text("[workspace]\nmembers=[]\n")
            (source / "Cargo.lock").write_text("version=4\n")
            tool = {"binaries": {name: {"path": name + ".exe", "sha256": "a" * 64}
                                 for name in ("cargo", "rustc", "rustdoc")}}
            identity = RequestIdentity("session", "request", "attempt", 1, "a" * 64, "managed-cargo-check-v2")
            repo = Path(__file__).resolve().parents[3]
            with patch("tools.jenkins.pilot.managed_cargo.toolchain_identity", return_value=tool), patch(
                    "tools.jenkins.pilot.managed_cargo.executor_manifest", return_value={"fixed-driver": "b" * 64}):
                execution = CargoExecution(repo_root=repo, source_root=source, run_root=run,
                                           identity=identity, managed_root=root)
                self.assertEqual(execution.command[:5], ["cargo.exe", "check", "--locked", "-p", "zircon_reflect_derive"])
                proof = observation(execution.command)
                unknown = json.loads(canonical_json(proof))
                unknown["processTree"]["pipes"]["stderr"]["eof"] = False
                intermediate = execution.finalize(unknown)
                self.assertFalse(intermediate["lockReleased"])
                self.assertTrue(execution.generation.locked)
                self.assertEqual(json.loads(execution.generation.storage.path("state.json").read_bytes())["state"], "running")
                for path in (root, execution.generation.path, *(execution.generation.path / name for name in
                        ("target", "cargo-home", "build", "scratch", "sccache"))):
                    with self.assertRaises(OSError):
                        path.rename(path.with_name(path.name + "-replacement"))
                execution.finalize(proof)
                accepted = collect_evidence(repo_root=repo, run_root=run, exit_code=0, observation=proof)
                self.assertTrue(accepted["accepted"], accepted)
                self.assertFalse(accepted["formalAcceptance"])
                with execution.generation.storage.backend() as backend:
                    backend.write_bytes("cargo-home/registry/cache/crates/foo-1.0.crate", b"immutable-archive")
                    backend.write_bytes("cargo-home/registry/index/crates/config.json", b"index-config")
                    backend.write_bytes("cargo-home/registry/src/crates/do-not-copy", b"unpacked")
                next_run = root / "runs/two"
                next_source = next_run / "sealed/source"
                next_source.mkdir(parents=True)
                (next_source / "Cargo.toml").write_text("[workspace]\nmembers=[]\n")
                (next_source / "Cargo.lock").write_text("version=4\n#new-locked-input\n")
                next_identity = RequestIdentity("session", "request-two", "attempt", 1, "a" * 64, "managed-cargo-check-v2")
                next_execution = CargoExecution(repo_root=repo, source_root=next_source, run_root=next_run,
                                               identity=next_identity, managed_root=root)
                try:
                    import msvcrt
                    with execution.generation.storage.backend() as source_backend, source_backend.open_lock_file("writer.lock") as source_writer:
                        source_writer.seek(0)
                        msvcrt.locking(source_writer.fileno(), msvcrt.LK_NBLCK, 1)
                        try:
                            with self.assertRaises(OSError):
                                next_execution.seed_registry(execution.generation.path)
                            self.assertFalse(next_execution.generation.backend.exists("registry-seed.json"))
                        finally:
                            source_writer.seek(0)
                            msvcrt.locking(source_writer.fileno(), msvcrt.LK_UNLCK, 1)
                    seed = next_execution.seed_registry(execution.generation.path)
                    self.assertEqual(len(seed["files"]), 2)
                    self.assertFalse(next_execution.generation.backend.exists("cargo-home/registry/src/crates/do-not-copy"))
                    self.assertEqual(next_execution.generation.backend.read_bytes("cargo-home/registry/cache/crates/foo-1.0.crate"), b"immutable-archive")
                    self.assertEqual(next_execution.request["generation"]["registrySeedHash"], digest(seed))
                    next_proof = observation(next_execution.command)
                    next_execution.finalize(next_proof)
                    self.assertTrue(collect_evidence(repo_root=repo, run_root=next_run, exit_code=0, observation=next_proof)["accepted"])
                    reused_seed = CargoGeneration(root, next_execution.generation.compatibility, next_identity.to_dict()).acquire()
                    self.assertEqual(reused_seed.state["registrySeedHash"], digest(seed))
                    reused_seed.release({"proof": "seeded-reuse-terminal"})
                    next_execution.generation.storage.atomic_write("registry-seed.json", canonical_json({**seed, "filesHash": "f" * 64}))
                    self.assertFalse(collect_evidence(repo_root=repo, run_root=next_run, exit_code=0, observation=next_proof)["accepted"])
                    next_execution.generation.storage.atomic_write("registry-seed.json", canonical_json(seed))
                    with self.assertRaises(PilotError):
                        next_execution.seed_registry(execution.generation.path)
                finally:
                    next_execution.close()
                with patch("tools.jenkins.pilot.managed_cargo.executor_manifest", return_value={"changed-driver": "c" * 64}):
                    self.assertFalse(collect_evidence(repo_root=repo, run_root=run, exit_code=0, observation=proof)["accepted"])
                self.assertFalse(collect_evidence(repo_root=repo, run_root=run, exit_code=0)["accepted"])
                changed = json.loads(canonical_json(proof))
                changed["processTree"]["creationTime"] = "134353522223204529"
                self.assertFalse(collect_evidence(repo_root=repo, run_root=run, exit_code=0, observation=changed)["accepted"])
                reused = CargoGeneration(root, execution.generation.compatibility, {"request": "next"}).acquire()
                reused.release({"proof": "next-terminal"})
                self.assertTrue(collect_evidence(repo_root=repo, run_root=run, exit_code=0, observation=proof)["accepted"])
                request = json.loads((run / "managed-request.json").read_bytes())
                request["command"].append("--release")
                (run / "managed-request.json").write_bytes(canonical_json(request))
                self.assertFalse(collect_evidence(repo_root=repo, run_root=run, exit_code=0, observation=proof)["accepted"])

    def test_terminal_reconciliation_preserves_failed_original_evidence(self):
        with temporary_directory("cargo-reconcile-") as temporary, patch(
                "tools.jenkins.pilot.cargo_cache.require_capacity", return_value=MIN_FREE_BYTES):
            root = Path(temporary)
            run = root / "runs/reconcile"
            source = run / "sealed/source"
            source.mkdir(parents=True)
            (source / "Cargo.toml").write_text("[workspace]\nmembers=[]\n")
            (source / "Cargo.lock").write_text("version=4\n")
            tool = {"binaries": {n: {"path": n + ".exe"} for n in ("cargo", "rustc", "rustdoc")}}
            identity = RequestIdentity("session", "reconcile", "attempt", 1, "a" * 64, "managed-cargo-check-v2")
            repo = Path(__file__).resolve().parents[3]
            with patch("tools.jenkins.pilot.managed_cargo.toolchain_identity", return_value=tool), patch(
                    "tools.jenkins.pilot.managed_cargo.executor_manifest", return_value={"driver": "b" * 64}):
                execution = CargoExecution(repo_root=repo, source_root=source, run_root=run, identity=identity, managed_root=root)
                proof = observation(execution.command)
                with patch.object(execution.generation, "release", side_effect=StorageSecurityError("old audit rejected native internal link")):
                    with self.assertRaises(StorageSecurityError):
                        execution.finalize(proof)
                error = {"stage": "managed_cargo_collector", "type": "StorageSecurityError",
                         "message": "worker file has a hard-link alias"}
                receipt = {**proof, "identity": identity.to_dict(), "outcome": "failed",
                    "reason": "managed_cargo_collector_failed", "terminalErrors": [error],
                    "managedCargo": {"accepted": False, "formalAcceptance": False, "error": error}}
                (run / "receipt.json").write_bytes(canonical_json(receipt))
                originals = {n: (run / n).read_bytes() for n in ("managed-request.json", "managed-result.json", "receipt.json")}
                arguments = dict(run_root=run, expected_identity=identity, original_driver_root=repo, audit_driver_root=repo)
                with self.assertRaises(OSError):
                    terminal_reconcile(**arguments)
                execution.close()
                altered_receipt = {**receipt, "terminalErrors": [error, {"stage": "native_job", "type": "Unknown"}]}
                (run / "receipt.json").write_bytes(canonical_json(altered_receipt))
                with self.assertRaises(PilotError):
                    terminal_reconcile(**arguments)
                (run / "receipt.json").write_bytes(originals["receipt.json"])
                failed = json.loads(originals["managed-result.json"])
                failed["observation"]["processTree"]["pipeEOF"] = False
                (run / "managed-result.json").write_bytes(canonical_json(failed))
                with self.assertRaises(PilotError):
                    terminal_reconcile(**arguments)
                (run / "managed-result.json").write_bytes(originals["managed-result.json"])
                reconciliation = terminal_reconcile(**arguments)
                self.assertEqual(reconciliation["originalOutcome"], "failed")
                for name, payload in originals.items():
                    self.assertEqual((run / name).read_bytes(), payload)
                self.assertEqual(json.loads(execution.generation.storage.path("state.json").read_bytes())["state"], "released")


if __name__ == "__main__":
    unittest.main()
