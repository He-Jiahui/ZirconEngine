[CmdletBinding()]
param(
    [Parameter(Mandatory)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$')][string]$TaskId,
    [Parameter(Mandatory)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$')][string]$RequestId,
    [Parameter(Mandatory)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$')][string]$AttemptId,
    [Parameter(Mandatory)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$')][string]$DriverId,
    [Parameter(Mandatory)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$')][string]$Component,
    [Parameter(Mandatory)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$')][string]$CommandId,
    [Parameter(Mandatory)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$')][string]$InvocationId,
    [Parameter(Mandatory)][ValidateSet('build-editor', 'terminal-receipt', 'storage-policy', 'staging', 'acceptance', 'snapshot', 'release', 'fixture-contract')][string]$Suite,
    [Parameter(Mandatory)][string]$JournalPath,
    [Parameter(Mandatory)][string]$SuiteArgumentsJsonPath,
    [string]$PesterResultPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
if ($PSVersionTable.PSEdition -ne 'Desktop' -or $PSVersionTable.PSVersion.Major -ne 5 -or $PSVersionTable.PSVersion.Minor -ne 1) {
    throw 'Invoke-MvpTestSuite.ps1 must be the native Windows PowerShell 5.1 process entry.'
}

# Bootstrap before the first Add-Type. Jenkins has already checked these physical
# directories; reject aliases/reparse ancestors here, then check native final paths.
$compilerTemporaryPaths = @($env:TEMP, $env:TMP)
foreach ($temporaryPath in $compilerTemporaryPaths) {
    if ([string]::IsNullOrWhiteSpace($temporaryPath) -or $temporaryPath -notmatch '^E:\\cargo-targets\\' -or
        $temporaryPath -match '~[0-9]' -or -not [IO.Path]::GetFullPath($temporaryPath).TrimEnd('\').Equals($temporaryPath.TrimEnd('\'), [StringComparison]::OrdinalIgnoreCase) -or
        -not [IO.Directory]::Exists($temporaryPath)) { throw 'Compiler TEMP/TMP must be existing registered physical directories under E:\cargo-targets.' }
    $temporaryAncestor = [IO.DirectoryInfo]::new($temporaryPath)
    while ($null -ne $temporaryAncestor) {
        if ([bool]($temporaryAncestor.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'Compiler TEMP/TMP has a reparse-point ancestor.' }
        $temporaryAncestor = $temporaryAncestor.Parent
    }
}

# Pester helpers can force-reimport this module as a nested dependency. Keep the
# original module and its bound commands alive through the wrapper's finally.
$fixtureOwnerModule = Import-Module (Join-Path $PSScriptRoot 'MvpTestFixtureContext.psm1') -Force -PassThru -ErrorAction Stop
$newFixtureContextCommand = $fixtureOwnerModule.ExportedCommands['New-MvpTestFixtureContext']
$writeFixtureJournalCommand = $fixtureOwnerModule.ExportedCommands['Write-MvpFixtureJournal']
$closeFixtureContextCommand = $fixtureOwnerModule.ExportedCommands['Close-MvpTestFixtureContext']
foreach ($ownerCommand in @($newFixtureContextCommand, $writeFixtureJournalCommand, $closeFixtureContextCommand)) {
    if ($null -eq $ownerCommand) { throw 'Fixture owner module did not export a required wrapper command.' }
}
Import-Module (Join-Path $PSScriptRoot '..\maintenance\WindowsPathResolver.psm1') -Force -ErrorAction Stop
foreach ($temporaryPath in $compilerTemporaryPaths) {
    $temporaryResolution = Resolve-ZirconWindowsPath -Path $temporaryPath
    if (-not $temporaryResolution.DisplayPath.Equals($temporaryPath.TrimEnd('\'), [StringComparison]::OrdinalIgnoreCase)) { throw 'Compiler TEMP/TMP differs from its physical final path.' }
}
$contextKey = 'ZirconEngine.MvpFixtureContext.V1'
if ($null -ne [AppDomain]::CurrentDomain.GetData($contextKey)) { throw 'A fixture wrapper already owns this AppDomain.' }

$suiteFiles = @{
    'build-editor' = 'build-editor.Tests.ps1'
    'terminal-receipt' = 'mvp-staging-terminal-receipt.Tests.ps1'
    'storage-policy' = 'mvp-artifact-storage-policy.Tests.ps1'
    'staging' = 'mvp-staging.Tests.ps1'
    'acceptance' = 'mvp-acceptance.Tests.ps1'
    'snapshot' = 'mvp-acceptance-staging-snapshot.Tests.ps1'
    'release' = 'mvp-staging-release.Tests.ps1'
    'fixture-contract' = 'mvp-test-fixture-paths.Tests.ps1'
}
$isPester = $Suite -in @('build-editor', 'terminal-receipt', 'storage-policy')
$reportDirectory = "E:\cargo-targets\zircon-local\reports\$TaskId\s78-fixture-owner\$InvocationId"

function Assert-MvpSuiteReportPath {
    param([Parameter(Mandatory)][string]$Path, [Parameter(Mandatory)][string]$Name, [switch]$MustBeAbsent)
    $expected = Join-Path $reportDirectory $Name
    if (-not [IO.Path]::IsPathRooted($Path) -or -not $Path.Equals($expected, [StringComparison]::OrdinalIgnoreCase)) { throw "Unexpected fixture wrapper report path: $Path" }
    $resolution = Resolve-ZirconWindowsPath -Path $Path
    if (-not $resolution.DisplayPath.Equals($expected, [StringComparison]::OrdinalIgnoreCase)) { throw "Fixture wrapper report path is a physical alias: $Path" }
    if ($MustBeAbsent.IsPresent -and (Test-Path -LiteralPath $resolution.OperationalPath)) { throw "Fixture wrapper report already exists: $Path" }
    return $resolution
}

$context = $null
$reportLease = $null
$pesterResultLease = $null
$primaryError = $null
try {
    $reportResolution = Resolve-ZirconWindowsPath -Path $reportDirectory
    if (-not $reportResolution.DisplayPath.Equals($reportDirectory, [StringComparison]::OrdinalIgnoreCase)) { throw 'Fixture wrapper report directory is a physical alias.' }
    # Jenkins registers and creates this parent. The wrapper never creates report trees.
    $reportLease = Open-ZirconWindowsDirectoryLease -Path $reportResolution.OperationalPath -ExpectedOperationalPath $reportResolution.OperationalPath -NoFollow
    $journalResolution = Assert-MvpSuiteReportPath -Path $JournalPath -Name "fixture-owner-$InvocationId.jsonl" -MustBeAbsent
    $argvResolution = Assert-MvpSuiteReportPath -Path $SuiteArgumentsJsonPath -Name 'argv.json'
    $argvText = [IO.File]::ReadAllText($argvResolution.OperationalPath)
    if (-not $argvText.TrimStart().StartsWith('[', [StringComparison]::Ordinal)) { throw 'Suite arguments must be a JSON string array.' }
    # Preserve empty arrays and null elements on Windows PowerShell 5.1, whose
    # top-level ConvertFrom-Json output otherwise enumerates the array.
    ConvertFrom-Json -InputObject $argvText -ErrorAction Stop | Out-Null
    $argumentEnvelope = ConvertFrom-Json -InputObject ('{"argv":' + $argvText + '}') -ErrorAction Stop
    if ($argumentEnvelope.argv -isnot [Array]) { throw 'Suite arguments must be a JSON string array.' }
    $suiteArguments = @($argumentEnvelope.argv)
    foreach ($argument in $suiteArguments) { if ($argument -isnot [string]) { throw 'Every suite argument must be a JSON string.' } }
    if ($isPester -and $suiteArguments.Count -ne 0) { throw 'Pester suites require the complete original script without argument or test-name filters.' }
    if ($isPester) {
        if ([string]::IsNullOrWhiteSpace($PesterResultPath)) { throw 'Pester suites require a registered result XML path.' }
        $pesterResolution = Assert-MvpSuiteReportPath -Path $PesterResultPath -Name "fixture-owner-$InvocationId.pester.xml" -MustBeAbsent
    }
    elseif (-not [string]::IsNullOrWhiteSpace($PesterResultPath)) { throw 'PesterResultPath is only valid for a Pester suite.' }

    $suitePath = Join-Path (Join-Path $PSScriptRoot '..\tests') $suiteFiles[$Suite]
    $metadata = [ordered]@{
        taskId = $TaskId; requestId = $RequestId; attemptId = $AttemptId; driverId = $DriverId
        component = $Component; commandId = $CommandId; invocationId = $InvocationId; suite = $Suite
        suitePath = [IO.Path]::GetFullPath($suitePath); suiteArgv = $suiteArguments
        suiteSha256 = (Get-FileHash -LiteralPath $suitePath -Algorithm SHA256).Hash.ToLowerInvariant()
    }
    $context = & $newFixtureContextCommand -Metadata $metadata -JournalPath $journalResolution.OperationalPath
    [AppDomain]::CurrentDomain.SetData($contextKey, $context)
    & $writeFixtureJournalCommand $context ([ordered]@{ kind = 'suite'; state = 'Started'; suite = $Suite; suiteArgv = $suiteArguments })
    if ($isPester) {
        Import-Module Pester -ErrorAction Stop
        $pesterCommand = Get-Command Invoke-Pester -CommandType Function -ErrorAction Stop
        $pesterVersion = $pesterCommand.Module.Version.ToString()
        # Pin the exclusively created instance while Pester writes the real XML.
        $pesterResultLease = [ZirconEngine.MvpFixtures.V1.Native]::CreateResultLease($pesterResolution.OperationalPath)
        & $writeFixtureJournalCommand $context ([ordered]@{ kind = 'pester'; state = 'Started'; version = $pesterVersion; script = $metadata.suitePath; resultPath = $PesterResultPath })
        $result = Invoke-Pester -Script $suitePath -PassThru -OutputFormat NUnitXml -OutputFile $pesterResolution.OperationalPath
        foreach ($name in @('TotalCount', 'PassedCount', 'FailedCount', 'SkippedCount', 'PendingCount')) {
            if ($null -eq $result -or $null -eq $result.PSObject.Properties[$name]) { throw "Pester did not return its actual $name." }
        }
        if (-not [IO.File]::Exists($pesterResolution.OperationalPath)) { throw 'Pester did not produce its local result XML.' }
        [ZirconEngine.MvpFixtures.V1.Native]::AssertResultIdentity($pesterResolution.OperationalPath, $pesterResultLease)
        $xml = [Xml.XmlDocument]::new()
        $xml.XmlResolver = $null
        $pesterXmlSettings = [Xml.XmlReaderSettings]::new()
        $pesterXmlSettings.DtdProcessing = [Xml.DtdProcessing]::Prohibit
        $pesterXmlSettings.XmlResolver = $null
        $pesterXmlSettings.CloseInput = $false
        $pesterXmlStream = [IO.FileStream]::new(
            $pesterResolution.OperationalPath,
            [IO.FileMode]::Open,
            [IO.FileAccess]::Read,
            [IO.FileShare]::Read)
        try {
            $pesterXmlReader = [Xml.XmlReader]::Create($pesterXmlStream, $pesterXmlSettings)
            try { $xml.Load($pesterXmlReader) }
            finally { $pesterXmlReader.Dispose() }
        }
        finally { $pesterXmlStream.Dispose() }
        $cases = @($xml.SelectNodes('//test-case'))
        $xmlFailures = @($cases | Where-Object { $_.GetAttribute('success') -eq 'False' -or $_.GetAttribute('result') -in @('Failure', 'Error') }).Count
        & $writeFixtureJournalCommand $context ([ordered]@{
            kind = 'pester'; state = 'Completed'; version = $pesterVersion; resultPath = $PesterResultPath
            total = [int]$result.TotalCount; passed = [int]$result.PassedCount; failed = [int]$result.FailedCount
            skipped = [int]$result.SkippedCount; pending = [int]$result.PendingCount; xmlCases = $cases.Count; xmlFailures = $xmlFailures
            resultSha256 = (Get-FileHash -LiteralPath $pesterResolution.OperationalPath -Algorithm SHA256).Hash.ToLowerInvariant()
        })
        if ($result.TotalCount -le 0 -or $cases.Count -ne $result.TotalCount -or $result.FailedCount -ne 0 -or $xmlFailures -ne 0 -or
            ($result.PassedCount + $result.FailedCount + $result.SkippedCount + $result.PendingCount) -ne $result.TotalCount) {
            throw 'Pester suite failed or its actual counts and local XML disagree.'
        }
    }
    else {
        & $suitePath @suiteArguments
        if (-not $?) { throw "Standalone MVP suite failed: $Suite." }
    }
    & $writeFixtureJournalCommand $context ([ordered]@{ kind = 'suite'; state = 'Passed'; suite = $Suite })
}
catch {
    $primaryError = $_
    if ($null -ne $context) {
        try { & $writeFixtureJournalCommand $context ([ordered]@{ kind = 'suite'; state = 'Failed'; suite = $Suite; error = $primaryError.Exception.Message }) }
        catch { $primaryError.Exception.Data['MvpFixtureJournalError'] = $_ }
    }
    throw
}
finally {
    try {
        if ($null -ne $context) { & $closeFixtureContextCommand -Context $context -PrimaryError $primaryError }
    }
    finally {
        if ([Object]::ReferenceEquals([AppDomain]::CurrentDomain.GetData($contextKey), $context)) { [AppDomain]::CurrentDomain.SetData($contextKey, $null) }
        if ($null -ne $pesterResultLease) { $pesterResultLease.Dispose() }
        if ($null -ne $reportLease) { $reportLease.Dispose() }
    }
}
