[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$ApprovedBaselineReportPath,
    [Parameter(Mandatory)][string]$ApprovedBaselineReceiptPath,
    [Parameter(Mandatory)][string]$CandidateReportPath,
    [Parameter(Mandatory)][string]$PolicyPath,
    [Parameter(Mandatory)][string]$OutputDirectory
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
Import-Module (Join-Path $repoRoot 'tools\WindowsPathResolver.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'MvpArtifactStoragePolicy.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'ResourceManagementComparison.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'ResourceManagementJsonEvidence.psm1') -Force -ErrorAction Stop

$script:ResourceManagementComparisonMaximumReportBytes = 64MB
$script:ResourceManagementComparisonMaximumReceiptBytes = 1MB
$script:ResourceManagementComparisonMaximumPolicyBytes = 4MB

function Assert-ResourceManagementComparisonOutputDirectory {
    param([Parameter(Mandatory)][string]$Path)

    $storage = Resolve-MvpArtifactStoragePath `
        -Path $Path `
        -NamespaceId 'resource-management-comparisons'
    if ([IO.Directory]::Exists($storage.operation_path) -or [IO.File]::Exists($storage.operation_path)) {
        throw "Resource-management comparison output must not already exist: $($storage.display_path)"
    }
    return [pscustomobject]@{
        OperationalPath = $storage.operation_path
        DisplayPath = $storage.display_path
        StoragePolicy = $storage
    }
}

function Write-ResourceManagementComparisonFileNew {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Content
    )

    $stream = [IO.FileStream]::new($Path, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
    try {
        $bytes = [Text.UTF8Encoding]::new($false).GetBytes($Content)
        $stream.Write($bytes, 0, $bytes.Length)
        $stream.Flush($true)
    }
    finally {
        $stream.Dispose()
    }
}

function Write-ResourceManagementComparisonReport {
    param(
        [Parameter(Mandatory)][string]$ApprovedBaselineReportPath,
        [Parameter(Mandatory)][string]$ApprovedBaselineReceiptPath,
        [Parameter(Mandatory)][string]$CandidateReportPath,
        [Parameter(Mandatory)][string]$PolicyPath,
        [Parameter(Mandatory)][string]$OutputDirectory
    )

    $output = Assert-ResourceManagementComparisonOutputDirectory -Path $OutputDirectory
    $approved = Get-ResourceManagementJsonEvidence `
        -Path $ApprovedBaselineReportPath `
        -Label 'Approved resource-management baseline report' `
        -MaximumBytes $script:ResourceManagementComparisonMaximumReportBytes
    $approvalReceipt = Get-ResourceManagementJsonEvidence `
        -Path $ApprovedBaselineReceiptPath `
        -Label 'Approved resource-management baseline approval receipt' `
        -MaximumBytes $script:ResourceManagementComparisonMaximumReceiptBytes
    $candidate = Get-ResourceManagementJsonEvidence `
        -Path $CandidateReportPath `
        -Label 'Candidate resource-management report' `
        -MaximumBytes $script:ResourceManagementComparisonMaximumReportBytes
    $policy = Get-ResourceManagementJsonEvidence `
        -Path $PolicyPath `
        -Label 'Resource-management comparison policy' `
        -MaximumBytes $script:ResourceManagementComparisonMaximumPolicyBytes
    $report = Compare-ResourceManagementReports `
        -ApprovedBaselineReport $approved.json `
        -ApprovedBaselineReportSha256 $approved.sha256 `
        -ApprovedBaselineReceipt $approvalReceipt.json `
        -ApprovedBaselineReceiptSha256 $approvalReceipt.sha256 `
        -CandidateReport $candidate.json `
        -CandidateReportSha256 $candidate.sha256 `
        -Policy $policy.json `
        -PolicySha256 $policy.sha256

    [IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($output.OperationalPath)) | Out-Null
    $stagingPath = "$($output.OperationalPath).partial-$([guid]::NewGuid().ToString('N'))"
    try {
        [IO.Directory]::CreateDirectory($stagingPath) | Out-Null
        $jsonPath = Join-ZirconWindowsPath -Path $stagingPath -ChildPath 'resource-management-comparison.json'
        $markdownPath = Join-ZirconWindowsPath -Path $stagingPath -ChildPath 'resource-management-comparison.md'
        Write-ResourceManagementComparisonFileNew `
            -Path $jsonPath `
            -Content ($report | ConvertTo-Json -Depth 32)
        Write-ResourceManagementComparisonFileNew `
            -Path $markdownPath `
            -Content (ConvertTo-ResourceManagementComparisonMarkdown -Report $report)
        [IO.Directory]::Move($stagingPath, $output.OperationalPath)
    }
    catch {
        if ([IO.Directory]::Exists($stagingPath)) {
            [IO.Directory]::Delete($stagingPath, $true)
        }
        throw
    }

    return [pscustomobject][ordered]@{
        report_path = (Resolve-ZirconWindowsPath -Path (Join-ZirconWindowsPath -Path $output.OperationalPath -ChildPath 'resource-management-comparison.json')).DisplayPath
        markdown_path = (Resolve-ZirconWindowsPath -Path (Join-ZirconWindowsPath -Path $output.OperationalPath -ChildPath 'resource-management-comparison.md')).DisplayPath
        diagnostic_decision = $report.diagnostic_decision
        qualification_status = $report.qualification_status
        comparison_count = $report.decision_summary.comparison_count
    }
}

Write-ResourceManagementComparisonReport `
    -ApprovedBaselineReportPath $ApprovedBaselineReportPath `
    -ApprovedBaselineReceiptPath $ApprovedBaselineReceiptPath `
    -CandidateReportPath $CandidateReportPath `
    -PolicyPath $PolicyPath `
    -OutputDirectory $OutputDirectory
