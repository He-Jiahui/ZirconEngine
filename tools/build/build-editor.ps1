<# Build an independent Windows editor preview below D/E/F:\cargo-targets\zircon-local.
Development is the default. Shipping uses shipping-editor and the shipping profile.
No retired coordinator is used. This command does not grant formal acceptance.
#>
[CmdletBinding()]
param(
    [string]$OutputDirectory,
    [string]$TargetDir,
    [ValidateSet("reuse", "compact", "diagnostic")]
    [string]$StorageMode = "reuse",
    [ValidateSet("development", "shipping")]
    [string]$CargoProfile = "development",
    [string]$SourceSnapshot,
    [string]$SourceSnapshotDigest,
    [string]$JenkinsIdentityFile,
    [string]$PythonInterpreter,
    [string]$CrtDirectory,
    [int]$Jobs = 2,
    [switch]$Ephemeral,
    [switch]$SkipSmokeTest
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
if (-not $PythonInterpreter) { $PythonInterpreter = Join-Path $repoRoot '.jenkins\runtime\python\python.exe' }
if (-not (Test-Path -LiteralPath $PythonInterpreter -PathType Leaf)) { throw "Python interpreter missing: $PythonInterpreter" }
$buildArguments = @('-B', '-X', 'utf8', '-m', 'tools.build.build_editor', '--repo-root', $repoRoot,
    '--cargo-profile', $CargoProfile, '--storage-mode', $StorageMode, '--jobs', [string]$Jobs)
foreach ($pair in @(
    @('--output-directory', $OutputDirectory), @('--target-dir', $TargetDir),
    @('--source-snapshot', $SourceSnapshot), @('--source-snapshot-digest', $SourceSnapshotDigest),
    @('--jenkins-identity-file', $JenkinsIdentityFile), @('--crt-directory', $CrtDirectory))) {
    if ($pair[1]) { $buildArguments += @($pair[0], $pair[1]) }
}
if ($Ephemeral) { $buildArguments += '--ephemeral' }
if ($SkipSmokeTest) { $buildArguments += '--skip-smoke-test' }
Push-Location -LiteralPath $repoRoot
try {
    & $PythonInterpreter @buildArguments
    $buildExitCode = $LASTEXITCODE
} finally { Pop-Location }
exit $buildExitCode
