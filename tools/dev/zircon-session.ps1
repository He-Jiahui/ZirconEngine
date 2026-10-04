[CmdletBinding()]
param(
    [Parameter(Position = 0)]
    [string]$Command = "status",
    [string]$RepoRoot,
    [ValidateRange(0, 65535)]
    [int]$Port = 6518,
    [switch]$Json,
    [switch]$Automatic,
    [Parameter(ValueFromPipeline = $true)]
    [AllowNull()]
    [AllowEmptyString()]
    [string]$PipelineInput,
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$Arguments
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

[Console]::Error.WriteLine('The local Zircon Session coordinator is retired. Service startup, leases, validation, integration and worker commands are disabled. Jenkins migration acceptance is pending.')
exit 3
