import hashlib
import io
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_validate_shader_pbr_renderdoc_replay import (
    _BoundedByteTail,
    _ReplayProcessResult,
    main,
    validate_renderdoc_replay,
)


class ZirconValidateShaderPbrRenderdocReplayTests(unittest.TestCase):
    def test_replays_regular_rdc_and_reports_immutable_capture_identity(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            capture_path = Path(temp_dir) / "pbr-ready.rdc"
            capture_contents = b"RenderDoc capture fixture"
            capture_path.write_bytes(capture_contents)
            executable = Path("D:/Tools/renderdoc/renderdoccmd.exe")
            completed = _ReplayProcessResult(0, "", "")

            with mock.patch(
                "tools.zircon_validate_shader_pbr_renderdoc_replay._run_replay_process",
                return_value=completed,
            ) as run:
                evidence = validate_renderdoc_replay(capture_path, executable=executable)

            self.assertEqual(capture_path.resolve(), evidence.capture_path)
            self.assertEqual(len(capture_contents), evidence.capture_size_bytes)
            self.assertEqual(
                hashlib.sha256(capture_contents).hexdigest(), evidence.capture_sha256
            )
            self.assertEqual(0, evidence.replay_returncode)
            self.assertEqual(
                True,
                evidence.replay_uses_verified_snapshot,
            )
            command = run.call_args.args[0]
            self.assertEqual(
                [str(executable), "replay", "--loops", "1"], command[:-1]
            )
            self.assertNotEqual(str(capture_path.resolve()), command[-1])
            self.assertFalse(Path(command[-1]).exists())
            run.assert_called_once()
            self.assertEqual(
                (command, 120),
                run.call_args.args,
            )

    def test_rejects_capture_without_lowercase_rdc_extension(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            capture_path = Path(temp_dir) / "pbr-ready.RDC"
            capture_path.write_bytes(b"RenderDoc capture fixture")

            with self.assertRaisesRegex(RuntimeError, "lowercase .rdc"):
                validate_renderdoc_replay(capture_path)

    def test_reports_replay_failure_with_capture_identity(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            capture_path = Path(temp_dir) / "pbr-ready.rdc"
            capture_path.write_bytes(b"RenderDoc capture fixture")
            completed = _ReplayProcessResult(3, "replay stdout tail", "replay stderr tail")

            with mock.patch(
                "tools.zircon_validate_shader_pbr_renderdoc_replay._run_replay_process",
                return_value=completed,
            ):
                with self.assertRaisesRegex(
                    RuntimeError,
                    "returncode=3.*capture_size_bytes=.*sha256=.*command=.*"
                    "replay stdout tail.*replay stderr tail",
                ):
                    validate_renderdoc_replay(capture_path)

    def test_rejects_capture_changed_while_replay_runs(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            capture_path = Path(temp_dir) / "pbr-ready.rdc"
            capture_path.write_bytes(b"before replay")

            def replace_capture(*_args, **_kwargs):
                capture_path.write_bytes(b"after replay")
                return _ReplayProcessResult(0, "", "")

            with mock.patch(
                "tools.zircon_validate_shader_pbr_renderdoc_replay._run_replay_process",
                side_effect=replace_capture,
            ):
                with self.assertRaisesRegex(RuntimeError, "changed during replay"):
                    validate_renderdoc_replay(capture_path)

    def test_reports_source_capture_deleted_during_replay_with_prior_identity(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            capture_path = Path(temp_dir) / "pbr-ready.rdc"
            capture_path.write_bytes(b"before replay")

            def delete_capture(*_args, **_kwargs):
                capture_path.unlink()
                return _ReplayProcessResult(0, "", "")

            with mock.patch(
                "tools.zircon_validate_shader_pbr_renderdoc_replay._run_replay_process",
                side_effect=delete_capture,
            ):
                with self.assertRaisesRegex(
                    RuntimeError,
                    "changed during replay.*capture_size_bytes=.*sha256=.*command=",
                ):
                    validate_renderdoc_replay(capture_path)

    def test_reports_timeout_and_unavailable_command_with_capture_identity(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            capture_path = Path(temp_dir) / "pbr-ready.rdc"
            capture_path.write_bytes(b"RenderDoc capture fixture")
            timeout = subprocess.TimeoutExpired(["renderdoccmd"], 5)

            with mock.patch(
                "tools.zircon_validate_shader_pbr_renderdoc_replay._run_replay_process",
                side_effect=timeout,
            ):
                with self.assertRaisesRegex(
                    RuntimeError,
                    "timed out.*capture_size_bytes=.*sha256=.*command=",
                ):
                    validate_renderdoc_replay(capture_path)

            with mock.patch(
                "tools.zircon_validate_shader_pbr_renderdoc_replay._run_replay_process",
                side_effect=FileNotFoundError(2, "missing command"),
            ):
                with self.assertRaisesRegex(
                    RuntimeError,
                    "unavailable.*capture_size_bytes=.*sha256=.*command=",
                ):
                    validate_renderdoc_replay(capture_path)

    def test_rejects_invalid_timeout_and_non_regular_capture(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            capture_path = Path(temp_dir) / "pbr-ready.rdc"
            capture_path.write_bytes(b"RenderDoc capture fixture")
            for timeout_seconds in (0, 601):
                with self.subTest(timeout_seconds=timeout_seconds):
                    with self.assertRaisesRegex(ValueError, "1 and 600 seconds"):
                        validate_renderdoc_replay(
                            capture_path, timeout_seconds=timeout_seconds
                        )

            directory_capture = Path(temp_dir) / "directory.rdc"
            directory_capture.mkdir()
            with self.assertRaisesRegex(RuntimeError, "not a regular file"):
                validate_renderdoc_replay(directory_capture)

    def test_main_reports_validation_error_without_masking_it(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            capture_path = Path(temp_dir) / "missing.rdc"
            stderr = io.StringIO()
            with mock.patch.object(sys, "argv", ["replay-gate", str(capture_path)]), mock.patch.object(
                sys, "stderr", stderr
            ):
                self.assertEqual(1, main())

            self.assertIn("PBR RenderDoc replay validation failed", stderr.getvalue())
            self.assertIn("capture is unavailable", stderr.getvalue())

    def test_rejects_a_snapshot_changed_by_the_replay_process(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            capture_path = Path(temp_dir) / "pbr-ready.rdc"
            capture_path.write_bytes(b"stable capture")

            def alter_snapshot(command, _timeout_seconds):
                Path(command[-1]).write_bytes(b"altered snapshot")
                return _ReplayProcessResult(0, "", "")

            with mock.patch(
                "tools.zircon_validate_shader_pbr_renderdoc_replay._run_replay_process",
                side_effect=alter_snapshot,
            ):
                with self.assertRaisesRegex(RuntimeError, "snapshot changed during replay"):
                    validate_renderdoc_replay(capture_path)

    def test_fails_the_gate_when_verified_snapshot_cleanup_fails(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            capture_path = Path(temp_dir) / "pbr-ready.rdc"
            capture_path.write_bytes(b"stable capture")
            snapshot_paths: list[Path] = []

            def fail_snapshot_cleanup(snapshot_path: Path) -> None:
                snapshot_paths.append(snapshot_path)
                raise RuntimeError("RenderDoc replay snapshot cleanup failed")

            try:
                with mock.patch(
                    "tools.zircon_validate_shader_pbr_renderdoc_replay._run_replay_process",
                    return_value=_ReplayProcessResult(0, "", ""),
                ), mock.patch(
                    "tools.zircon_validate_shader_pbr_renderdoc_replay._remove_snapshot",
                    side_effect=fail_snapshot_cleanup,
                ):
                    with self.assertRaisesRegex(RuntimeError, "snapshot cleanup failed"):
                        validate_renderdoc_replay(capture_path)
            finally:
                for snapshot_path in snapshot_paths:
                    snapshot_path.unlink(missing_ok=True)

    def test_cleanup_failure_preserves_timeout_as_the_primary_replay_failure(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            capture_path = Path(temp_dir) / "pbr-ready.rdc"
            capture_path.write_bytes(b"stable capture")
            snapshot_paths: list[Path] = []

            def fail_snapshot_cleanup(snapshot_path: Path) -> None:
                snapshot_paths.append(snapshot_path)
                raise RuntimeError("RenderDoc replay snapshot cleanup failed")

            timeout = subprocess.TimeoutExpired(["renderdoccmd"], 5)
            try:
                with mock.patch(
                    "tools.zircon_validate_shader_pbr_renderdoc_replay._run_replay_process",
                    side_effect=timeout,
                ), mock.patch(
                    "tools.zircon_validate_shader_pbr_renderdoc_replay._remove_snapshot",
                    side_effect=fail_snapshot_cleanup,
                ):
                    with self.assertRaisesRegex(
                        RuntimeError,
                        "snapshot cleanup failed.*prior replay failure",
                    ) as raised:
                        validate_renderdoc_replay(capture_path)
            finally:
                for snapshot_path in snapshot_paths:
                    snapshot_path.unlink(missing_ok=True)

            self.assertIsNotNone(raised.exception.__cause__)
            self.assertIn("timed out", str(raised.exception.__cause__))
            self.assertIsInstance(
                raised.exception.__cause__.__cause__, subprocess.TimeoutExpired
            )

    def test_bounded_tail_keeps_only_the_most_recent_output(self):
        tail = _BoundedByteTail(8)
        tail.append(b"first-")
        tail.append(b"second-")
        tail.append(b"tail")

        self.assertEqual("ond-tail", tail.text())
