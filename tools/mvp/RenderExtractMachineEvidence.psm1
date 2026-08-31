Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Import-Module (Join-Path $PSScriptRoot '..\WindowsPathResolver.psm1') -Force -DisableNameChecking -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'RenderExtractBaselineEvidence.psm1') -Force -DisableNameChecking -ErrorAction Stop

$script:RenderExtractMachineManifestCategories = @(
    'cpu',
    'gpu',
    'memory',
    'bios',
    'os',
    'display_modes',
    'power_policy',
    'thermal_frequency',
    'background_load',
    'virtualization'
)

function Assert-RenderExtractMachineManifest {
    param([Parameter(Mandatory)]$Manifest)

    $schemaVersion = Get-RenderExtractReportProperty `
        -Value $Manifest `
        -Name 'schema_version' `
        -Label 'Render-extract machine manifest'
    if ([int]$schemaVersion -ne 1) {
        throw 'Render-extract machine manifest schema_version must be 1.'
    }
    $kind = [string](Get-RenderExtractReportProperty `
            -Value $Manifest `
            -Name 'manifest_kind' `
            -Label 'Render-extract machine manifest')
    if ($kind -cne 'zircon_performance_machine_snapshot') {
        throw 'Render-extract machine manifest has an unsupported manifest_kind.'
    }
    $capturedUtcValue = Get-RenderExtractReportProperty `
            -Value $Manifest `
            -Name 'captured_utc' `
            -Label 'Render-extract machine manifest'
    $parsedUtc = [DateTimeOffset]::MinValue
    $capturedUtcIsValid = if ($capturedUtcValue -is [DateTime]) {
        $capturedUtcValue.Kind -eq [DateTimeKind]::Utc
    }
    elseif ($capturedUtcValue -is [DateTimeOffset]) {
        $capturedUtcValue.Offset -eq [TimeSpan]::Zero
    }
    else {
        [DateTimeOffset]::TryParse(
            [string]$capturedUtcValue,
            [Globalization.CultureInfo]::InvariantCulture,
            [Globalization.DateTimeStyles]::RoundtripKind,
            [ref]$parsedUtc
        ) -and $parsedUtc.Offset -eq [TimeSpan]::Zero
    }
    if (-not $capturedUtcIsValid) {
        throw 'Render-extract machine manifest captured_utc must be an ISO-8601 UTC timestamp.'
    }
    $requiredCategories = @(Get-RenderExtractReportArrayProperty `
            -Value $Manifest `
            -Name 'required_categories' `
            -Label 'Render-extract machine manifest' | ForEach-Object { [string]$_ })
    if (($requiredCategories -join "`n") -cne ($script:RenderExtractMachineManifestCategories -join "`n")) {
        throw 'Render-extract machine manifest required_categories do not match the supported contract.'
    }

    $allCaptured = $true
    foreach ($category in $script:RenderExtractMachineManifestCategories) {
        $observation = Get-RenderExtractReportProperty `
            -Value $Manifest `
            -Name $category `
            -Label 'Render-extract machine manifest'
        $status = [string](Get-RenderExtractReportProperty `
                -Value $observation `
                -Name 'status' `
                -Label "Render-extract machine manifest category '$category'")
        if ($status -ceq 'captured') {
            $data = @(Get-RenderExtractReportArrayProperty `
                    -Value $observation `
                    -Name 'data' `
                    -Label "Render-extract machine manifest category '$category'")
            if ($data.Count -eq 0) {
                throw "Render-extract machine manifest captured category '$category' has no data."
            }
        }
        elseif ($status -ceq 'unavailable') {
            $reason = [string](Get-RenderExtractReportProperty `
                    -Value $observation `
                    -Name 'reason' `
                    -Label "Render-extract machine manifest category '$category'")
            if ([string]::IsNullOrWhiteSpace($reason)) {
                throw "Render-extract machine manifest unavailable category '$category' has no reason."
            }
            $allCaptured = $false
        }
        else {
            throw "Render-extract machine manifest category '$category' has unsupported status '$status'."
        }
    }
    $allRequiredObserved = Get-RenderExtractReportProperty `
        -Value $Manifest `
        -Name 'all_required_observed' `
        -Label 'Render-extract machine manifest'
    if ($allRequiredObserved -isnot [bool] -or [bool]$allRequiredObserved -ne $allCaptured) {
        throw 'Render-extract machine manifest all_required_observed does not match its category statuses.'
    }
    return $allCaptured
}

function Resolve-RenderExtractMachineEvidence {
    param(
        [Parameter(Mandatory)]$Reference,
        [Parameter(Mandatory)][string]$EvidenceDirectory
    )

    $root = (Resolve-ZirconWindowsPath -Path $EvidenceDirectory).OperationalPath
    $path = [string](Get-RenderExtractReportProperty `
            -Value $Reference `
            -Name 'path' `
            -Label 'Render-extract machine evidence reference')
    $resolution = Resolve-ZirconWindowsPath -Path $path
    if (-not (Test-RenderExtractPathWithinDirectory `
            -CandidatePath $resolution.OperationalPath `
            -RootPath $root) -or
        [IO.Path]::GetFileName($resolution.OperationalPath) -cne 'machine-manifest.json' -or
        -not ([IO.Path]::GetDirectoryName($resolution.OperationalPath)).Equals($root, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Render-extract machine manifest must be the direct machine-manifest.json child of its evidence directory.'
    }
    $snapshot = Read-RenderExtractJsonEvidence `
        -Path $resolution.OperationalPath `
        -Label 'Render-extract machine manifest'
    $expectedSha256 = [string](Get-RenderExtractReportProperty `
            -Value $Reference `
            -Name 'sha256' `
            -Label 'Render-extract machine evidence reference')
    if ($expectedSha256 -notmatch '^[0-9A-Fa-f]{64}$' -or
        -not $snapshot.sha256.Equals($expectedSha256, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Render-extract machine manifest SHA-256 does not match its summary reference.'
    }
    $allRequiredObserved = Assert-RenderExtractMachineManifest -Manifest $snapshot.json
    return [pscustomobject][ordered]@{
        path = $snapshot.path
        bytes = $snapshot.bytes
        sha256 = $snapshot.sha256
        all_required_observed = $allRequiredObserved
        manifest = $snapshot.json
    }
}

Export-ModuleMember -Function @(
    'Assert-RenderExtractMachineManifest',
    'Resolve-RenderExtractMachineEvidence'
)
