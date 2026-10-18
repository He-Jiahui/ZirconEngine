<# Stage and prove a first editor window. Keep evidence under an approved cargo-targets root. #>
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$PythonInterpreter,
    [Parameter(Mandatory)][string]$OutputDirectory,
    [string]$TargetDir,
    [string]$CrtDirectory,
    [int]$Jobs = 2
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
# build-editor already performs the bounded first-frame launch and records EXE/DLL hashes.
$arguments = @{
    PythonInterpreter = $PythonInterpreter
    OutputDirectory = $OutputDirectory
    Jobs = $Jobs
}
if ($TargetDir) { $arguments.TargetDir = $TargetDir }
if ($CrtDirectory) { $arguments.CrtDirectory = $CrtDirectory }
& (Join-Path $PSScriptRoot 'build-editor.ps1') @arguments
exit $LASTEXITCODE
