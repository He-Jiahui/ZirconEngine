Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
# A PowerShell advanced parameter block consumes Cargo's -p as PipelineVariable.
# Parse only wrapper options before the Cargo verb and forward the rest exactly.
$localArguments = @($args)
$RepoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$TargetDir = $null
$DryRun = $false
$JenkinsIdentityFile = $null
$PythonInterpreter = $null
$IndependentPreview = $false
$argumentIndex = 0
while ($argumentIndex -lt $localArguments.Count) {
    $option = [string]$localArguments[$argumentIndex]
    if ($option -eq '-DryRun') { $DryRun = $true; $argumentIndex++; continue }
    if ($option -eq '-IndependentPreview') { $IndependentPreview = $true; $argumentIndex++; continue }
    if ($option -in @('-RepoRoot', '-TargetDir', '-JenkinsIdentityFile', '-PythonInterpreter')) {
        if ($argumentIndex + 1 -ge $localArguments.Count) { throw "$option requires a value" }
        $argumentIndex++
        if ($option -eq '-RepoRoot') { $RepoRoot = [string]$localArguments[$argumentIndex] }
        elseif ($option -eq '-TargetDir') { $TargetDir = [string]$localArguments[$argumentIndex] }
        elseif ($option -eq '-PythonInterpreter') { $PythonInterpreter = [string]$localArguments[$argumentIndex] }
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
if ($DryRun) { $pythonArguments += '--dry-run' }
if ($IndependentPreview) { $pythonArguments += '--independent-preview' }
$pythonArguments += @('--') + @($CargoArguments)
Push-Location -LiteralPath $resolvedRepo
try {
    & $jenkinsPython @pythonArguments
    $cargoExitCode = $LASTEXITCODE
}
finally { Pop-Location }
exit $cargoExitCode
