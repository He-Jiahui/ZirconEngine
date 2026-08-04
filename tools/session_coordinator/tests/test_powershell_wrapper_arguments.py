from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
import tempfile
import threading
import unittest
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path


class PowerShellWrapperArgumentTests(unittest.TestCase):
    def test_wrapper_omits_empty_tokens_and_emits_one_json_document(self) -> None:
        shells = [
            (name, executable)
            for name, executable in (
                ("PowerShell 7", shutil.which("pwsh")),
                ("Windows PowerShell 5.1", shutil.which("powershell.exe")),
            )
            if executable is not None
        ]
        if not shells:
            self.skipTest("PowerShell is unavailable")

        wrapper = Path(__file__).resolve().parents[3] / "tools" / "zircon-session.ps1"
        invocation = r"""
switch ($env:ZIRCON_TEST_CASE) {
    'status' {
        $result = & $env:ZIRCON_TEST_WRAPPER `
            -Command status `
            -RepoRoot $env:ZIRCON_TEST_REPO `
            -Port ([int]$env:ZIRCON_TEST_PORT) `
            -Json
    }
    'claim' {
        $result = & $env:ZIRCON_TEST_WRAPPER `
            -Command lease `
            -RepoRoot $env:ZIRCON_TEST_REPO `
            -Port ([int]$env:ZIRCON_TEST_PORT) `
            -Json `
            -Arguments @('claim', '--session-id', 'session-a', 'owned path.txt')
    }
    'release' {
        $result = & $env:ZIRCON_TEST_WRAPPER `
            -Command lease `
            -RepoRoot $env:ZIRCON_TEST_REPO `
            -Port ([int]$env:ZIRCON_TEST_PORT) `
            -Json `
            -Arguments @('release', '--session-id', 'session-a', 'owned path.txt')
    }
}
$nativeExit = $LASTEXITCODE
$result
exit $nativeExit
"""
        fake_python = """
import json
import sys

print(json.dumps({"argv": sys.argv[1:]}, separators=(",", ":")))
"""

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = root / "repo"
            repo.mkdir()
            fake_python_path = root / "fake_python.py"
            fake_python_path.write_text(fake_python, encoding="utf-8")
            (root / "python.cmd").write_text(
                '@echo off\r\n"%ZIRCON_TEST_REAL_PYTHON%" '
                '"%ZIRCON_TEST_FAKE_PYTHON%" %*\r\n',
                encoding="ascii",
            )

            class HealthHandler(BaseHTTPRequestHandler):
                def do_GET(self) -> None:
                    payload = json.dumps(
                        {"status": "ok", "repo_root": str(repo)}
                    ).encode("utf-8")
                    self.send_response(200)
                    self.send_header("Content-Type", "application/json")
                    self.send_header("Content-Length", str(len(payload)))
                    self.end_headers()
                    self.wfile.write(payload)

                def log_message(self, _format: str, *_args: object) -> None:
                    return

            server = ThreadingHTTPServer(("127.0.0.1", 0), HealthHandler)
            server_thread = threading.Thread(target=server.serve_forever, daemon=True)
            server_thread.start()
            port = server.server_address[1]
            base_environment = {
                **os.environ,
                "PATH": f"{root}{os.pathsep}{os.environ['PATH']}",
                "ZIRCON_TEST_FAKE_PYTHON": str(fake_python_path),
                "ZIRCON_TEST_REAL_PYTHON": sys.executable,
                "ZIRCON_TEST_REPO": str(repo),
                "ZIRCON_TEST_PORT": str(port),
                "ZIRCON_TEST_WRAPPER": str(wrapper),
            }
            cases = (
                ("status", ("status",)),
                (
                    "claim",
                    ("lease", "claim", "--session-id", "session-a", "owned path.txt"),
                ),
                (
                    "release",
                    ("lease", "release", "--session-id", "session-a", "owned path.txt"),
                ),
            )

            try:
                for shell_name, executable in shells:
                    for case, expected_tail in cases:
                        with self.subTest(shell=shell_name, command=expected_tail):
                            environment = {
                                **base_environment,
                                "ZIRCON_TEST_CASE": case,
                            }
                            completed = subprocess.run(
                                [
                                    executable,
                                    "-NoProfile",
                                    "-ExecutionPolicy",
                                    "Bypass",
                                    "-Command",
                                    invocation,
                                ],
                                cwd=wrapper.parents[1],
                                env=environment,
                                capture_output=True,
                                text=True,
                                encoding="utf-8",
                                timeout=30,
                                check=False,
                            )

                            self.assertEqual(0, completed.returncode, completed.stderr)
                            lines = [
                                line
                                for line in completed.stdout.splitlines()
                                if line.strip()
                            ]
                            self.assertEqual(1, len(lines), completed.stdout)
                            native_arguments = json.loads(lines[0])["argv"]
                            self.assertNotIn("", native_arguments)
                            self.assertEqual(
                                list(expected_tail), native_arguments[-len(expected_tail) :]
                            )
            finally:
                server.shutdown()
                server.server_close()
                server_thread.join(timeout=2)


if __name__ == "__main__":
    unittest.main()
