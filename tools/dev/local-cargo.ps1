Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
# A PowerShell advanced parameter block consumes Cargo's -p as PipelineVariable.
# Parse only wrapper options before the Cargo verb and forward the rest exactly.
try {
$localArguments = @($args)
$RepoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$TargetDir = $null
$BuildRoot = $null
$QueryOnly = $false
$NoLocked = $false
$DryRun = $false
$JenkinsIdentityFile = $null
$PythonInterpreter = $null
$IndependentPreview = $false
$EvidenceDirectory = $null
$EvidenceInputManifest = $null
$EvidenceTimeoutSeconds = $null
$argumentIndex = 0
while ($argumentIndex -lt $localArguments.Count) {
    $option = [string]$localArguments[$argumentIndex]
    if ($option -eq '-QueryOnly') { $QueryOnly = $true; $argumentIndex++; continue }
    if ($option -eq '-NoLocked') { $NoLocked = $true; $argumentIndex++; continue }
    if ($option -eq '-DryRun') { $DryRun = $true; $argumentIndex++; continue }
    if ($option -eq '-IndependentPreview') { $IndependentPreview = $true; $argumentIndex++; continue }
    if ($option -in @('-RepoRoot', '-TargetDir', '-BuildRoot', '-JenkinsIdentityFile', '-PythonInterpreter', '-EvidenceDirectory', '-EvidenceInputManifest', '-EvidenceTimeoutSeconds')) {
        if ($argumentIndex + 1 -ge $localArguments.Count) { throw "$option requires a value" }
        $argumentIndex++
        if ($option -eq '-RepoRoot') { $RepoRoot = [string]$localArguments[$argumentIndex] }
        elseif ($option -eq '-TargetDir') { $TargetDir = [string]$localArguments[$argumentIndex] }
        elseif ($option -eq '-BuildRoot') { $BuildRoot = [string]$localArguments[$argumentIndex] }
        elseif ($option -eq '-PythonInterpreter') { $PythonInterpreter = [string]$localArguments[$argumentIndex] }
        elseif ($option -eq '-EvidenceDirectory') { $EvidenceDirectory = [string]$localArguments[$argumentIndex] }
        elseif ($option -eq '-EvidenceInputManifest') { $EvidenceInputManifest = [string]$localArguments[$argumentIndex] }
        elseif ($option -eq '-EvidenceTimeoutSeconds') { $EvidenceTimeoutSeconds = [string]$localArguments[$argumentIndex] }
        else { $JenkinsIdentityFile = [string]$localArguments[$argumentIndex] }
        $argumentIndex++
        continue
    }
    break
}
$CargoArguments = if ($argumentIndex -lt $localArguments.Count) {
    @($localArguments[$argumentIndex..($localArguments.Count - 1)])
} else { @() }
$resolvedRepo = (Resolve-Path -LiteralPath $RepoRoot).Path
$jenkinsPython = if ($PythonInterpreter) { $PythonInterpreter } else { Join-Path (Split-Path -Parent (Split-Path -Parent $PSScriptRoot)) '.jenkins\runtime\python\python.exe' }
if (-not (Test-Path -LiteralPath $jenkinsPython -PathType Leaf)) { throw "Jenkins runtime Python is missing: $jenkinsPython" }
$pythonArguments = @('-B', '-X', 'utf8', '-m', 'tools.dev.local_cargo', '--repo-root', $resolvedRepo)
if ($JenkinsIdentityFile) { $pythonArguments += @('--jenkins-identity-file', $JenkinsIdentityFile) }
if (-not [string]::IsNullOrWhiteSpace($TargetDir)) { $pythonArguments += @('--target-dir', $TargetDir) }
if ($BuildRoot) { $pythonArguments += @('--build-root', $BuildRoot) }
if ($QueryOnly) { $pythonArguments += '--query-only' }
if ($NoLocked) { $pythonArguments += '--unlocked' }
if ($DryRun) { $pythonArguments += '--dry-run' }
if ($IndependentPreview) { $pythonArguments += '--independent-preview' }
if ($null -ne $EvidenceDirectory) { $pythonArguments += @('--evidence-directory', $EvidenceDirectory) }
if ($null -ne $EvidenceInputManifest) { $pythonArguments += @('--evidence-input-manifest', $EvidenceInputManifest) }
if ($null -ne $EvidenceTimeoutSeconds) { $pythonArguments += @('--evidence-timeout-seconds', $EvidenceTimeoutSeconds) }
$pythonArguments += @('--') + @($CargoArguments)
Push-Location -LiteralPath $resolvedRepo
try {
    & $jenkinsPython @pythonArguments
    $cargoExitCode = $LASTEXITCODE
}
finally { Pop-Location }
exit $cargoExitCode

} catch {
    [ordered]@{status='pending';reasonCode='ordinary_entry_pending';message=$_.Exception.Message;
        formalAcceptance=$false;m8Accepted=$false} | ConvertTo-Json -Compress
    exit 2
}
