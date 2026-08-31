import base64
from pathlib import Path
import subprocess
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
PATH_RESOLVER = REPO_ROOT / "tools" / "WindowsPathResolver.psm1"
MANAGED_STORAGE = (
    REPO_ROOT
    / ".codex"
    / "skills"
    / "zircon-dev"
    / "scripts"
    / "managed-cargo-storage.ps1"
)


def powershell_literal(path: Path) -> str:
    return str(path).replace("'", "''")


class ManagedCargoStorageWindowsPowerShellContractTests(unittest.TestCase):
    def run_windows_powershell(self, body: str) -> subprocess.CompletedProcess[str]:
        script = f"""
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
Import-Module '{powershell_literal(PATH_RESOLVER)}' -Force -DisableNameChecking
. '{powershell_literal(MANAGED_STORAGE)}'
{body}
"""
        encoded = base64.b64encode(script.encode("utf-16-le")).decode("ascii")
        return subprocess.run(
            ["powershell.exe", "-NoProfile", "-EncodedCommand", encoded],
            cwd=REPO_ROOT,
            capture_output=True,
            text=True,
            check=False,
        )

    def test_binding_marker_supports_extended_length_paths_in_windows_powershell(self) -> None:
        result = self.run_windows_powershell(
            r"""
$temporary = Resolve-ZirconWindowsPath -Path 'E:\ZirconBuilds\zircon-engine\cache\sccache-temporary'
$marker = Resolve-ManagedCompilerCacheBindingMarkerPath `
    -StableTemporaryDirectory $temporary.OperationalPath
$expected = Join-ZirconWindowsPath `
    -Path $temporary.OperationalPath `
    -ChildPath 'server-binding-v1.json'
if ($marker -ne $expected) {
    throw "Unexpected marker path: $marker"
}
"""
        )

        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_diagnostic_mode_disables_the_shared_compiler_cache(self) -> None:
        result = self.run_windows_powershell(
            """
$cache = Resolve-ManagedCompilerCacheExecutable -StorageMode diagnostic
if ($null -ne $cache) {
    throw "Diagnostic storage unexpectedly selected compiler cache: $cache"
}
"""
        )

        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main()
