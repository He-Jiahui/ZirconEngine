[CmdletBinding()]
param(
    [Parameter(Position = 0)]
    [ValidateSet('dry-check', 'submit', 'query', 'release-source')]
    [string]$Action = 'dry-check',
    [string]$Repository = (Split-Path (Split-Path $PSScriptRoot -Parent) -Parent),
    [Parameter(Mandatory = $true)][string]$SessionId,
    [Parameter(Mandatory = $true)][string]$RequestId,
    [string]$AttemptId,
    [string]$Patch,
    [string]$BeforeHashes,
    [switch]$AlreadyApplied,
    [string]$AfterHashes,
    [string]$DependencyHashes,
    [string]$BaselineRef,
    [string]$Coverage,
    [string]$CoverageTemplate,
    [string[]]$Dependency = @(),
    [string]$BaseHead,
    [string]$BuildRoot,
    [string]$PatchRequestRef,
    [string]$ReleaseRequestId,
    [ValidateRange(1, 300)][int]$Timeout = 60
)
$ErrorActionPreference = 'Stop'
if (-not $AlreadyApplied -and ($PSBoundParameters.ContainsKey('AfterHashes') -or
        $PSBoundParameters.ContainsKey('DependencyHashes') -or $PSBoundParameters.ContainsKey('BaselineRef'))) {
    throw '-AfterHashes, -DependencyHashes and -BaselineRef require -AlreadyApplied.'
}
$repoPath = [System.IO.Path]::GetFullPath($Repository)
$pythonPath = Join-Path $repoPath '.jenkins\runtime\python\python.exe'
$entryPath = Join-Path $PSScriptRoot 'jenkins_validate.py'
if (-not (Test-Path -LiteralPath $pythonPath -PathType Leaf)) {
    throw 'Dedicated Jenkins Python runtime is unavailable.'
}
$arguments = @('-B', $entryPath, $Action, '--repository', $repoPath,
    '--session-id', $SessionId, '--request-id', $RequestId, '--timeout', [string]$Timeout)
$optional = @{
    '--attempt-id' = $AttemptId
    '--patch' = $Patch
    '--before-hashes' = $BeforeHashes
    '--after-hashes' = $AfterHashes
    '--dependency-hashes' = $DependencyHashes
    '--baseline-ref' = $BaselineRef
    '--coverage' = $Coverage
    '--coverage-template' = $CoverageTemplate
    '--base-head' = $BaseHead
    '--build-root' = $BuildRoot
    '--patch-request-ref' = $PatchRequestRef
    '--release-request-id' = $ReleaseRequestId
}
foreach ($name in $optional.Keys) {
    if (-not [string]::IsNullOrEmpty($optional[$name])) {
        $arguments += @($name, $optional[$name])
    }
}
if ($AlreadyApplied) {
    $arguments += '--already-applied'
}
foreach ($sourcePath in $Dependency) {
    $arguments += @('--dependency', $sourcePath)
}
& $pythonPath @arguments
exit $LASTEXITCODE
