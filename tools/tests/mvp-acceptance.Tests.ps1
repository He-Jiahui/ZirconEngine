Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$acceptanceDriver = Join-Path $PSScriptRoot '..\mvp\Invoke-MvpAcceptance.ps1'
$driver = Join-Path $PSScriptRoot 'Invoke-MvpAcceptanceTestDriver.ps1'
$buildSummaryEvidenceModule = Join-Path $PSScriptRoot '..\mvp\MvpBuildSummaryEvidence.psm1'
$buildGateRegistryModule = Join-Path $PSScriptRoot '..\mvp\MvpBuildGateRegistry.psm1'
$artifactStoragePolicyModule = Join-Path $PSScriptRoot '..\mvp\MvpArtifactStoragePolicy.psm1'
$datePreservingJsonModule = Join-Path $PSScriptRoot '..\mvp\MvpDatePreservingJson.psm1'
$fixturePathsModule = Join-Path $PSScriptRoot '..\mvp\MvpTestFixturePaths.psm1'
Import-Module $fixturePathsModule -Force -ErrorAction Stop
Import-Module $buildGateRegistryModule -Force -ErrorAction Stop
Import-Module $artifactStoragePolicyModule -Force -ErrorAction Stop
Import-Module $datePreservingJsonModule -Force -ErrorAction Stop

function Assert-True {
    param(
        [Parameter(Mandatory)][bool]$Condition,
        [Parameter(Mandatory)][string]$Message
    )

    if (-not $Condition) {
        throw $Message
    }
}

function ConvertTo-FixtureProcessText {
    param([AllowEmptyCollection()][object[]]$Output)

    $text = @($Output | ForEach-Object { $_.ToString() }) -join [Environment]::NewLine
    return $text -replace ([char]27 + '\[[0-?]*[ -/]*[@-~]'), ''
}

function Get-FixtureFileSha256 {
    param([Parameter(Mandatory)][string]$Path)

    $stream = $null
    $hasher = $null
    try {
        $stream = [IO.File]::OpenRead($Path)
        $hasher = [Security.Cryptography.SHA256]::Create()
        return ([BitConverter]::ToString($hasher.ComputeHash($stream))).Replace('-', '')
    }
    finally {
        if ($null -ne $hasher) {
            $hasher.Dispose()
        }
        if ($null -ne $stream) {
            $stream.Dispose()
        }
    }
}

function Invoke-MvpAcceptanceDriverForTest {
    param([Parameter(Mandatory)][hashtable]$Parameters)

    $previousErrorActionPreference = $ErrorActionPreference
    try {
        # Windows PowerShell promotes stderr from nested pwsh to a terminating
        # NativeCommandError under Stop. Preserve the child result for assertions.
        $ErrorActionPreference = 'Continue'
        $output = @(& pwsh -NoProfile -File $acceptanceDriver @Parameters 2>&1)
        return [pscustomobject]@{
            ExitCode = $LASTEXITCODE
            Output = $output
        }
    }
    finally {
        $ErrorActionPreference = $previousErrorActionPreference
    }
}

$workflowPath = Join-Path $PSScriptRoot '..\..\.github\workflows\mvp-editor-windows.yml'
$workflowSource = Get-Content -LiteralPath $workflowPath -Raw
$acceptanceSource = Get-Content -LiteralPath $acceptanceDriver -Raw
$acceptanceTestsSource = Get-Content -LiteralPath $PSCommandPath -Raw
$buildSummaryEvidenceSource = Get-Content -LiteralPath (Join-Path $PSScriptRoot '..\mvp\MvpBuildSummaryEvidence.psm1') -Raw
Assert-True `
    ($workflowSource -match 'Copy-Item -LiteralPath \$evidenceRoot -Destination \$artifactRoot -Recurse -Force') `
    'Windows MVP workflow must upload the complete detached EvidenceRoot instead of a partial staging projection.'
Assert-True ($acceptanceSource -match '\$candidateResolution = Resolve-ZirconWindowsPath -Path \$candidate') 'MVP acceptance evidence files must resolve through the shared Windows path resolver.'
$acceptanceTestDriverSource = Get-Content -LiteralPath $driver -Raw
Assert-True `
    ($acceptanceTestDriverSource -match '\[string\]\$StagingRoot' -and $acceptanceTestDriverSource -match '\[string\]\$EvidenceRoot') `
    'MVP acceptance test driver must bind its required production parameters by name.'
Assert-True `
    ($acceptanceTestDriverSource -match '& \$acceptancePath @driverParameters' -and $acceptanceTestDriverSource -notmatch '@driverArguments') `
    'MVP acceptance test driver must use a named parameter map rather than positional argument replay.'
Assert-True `
    ($acceptanceTestDriverSource -match 'Start-Process' -and
        $acceptanceTestDriverSource -match 'WaitForExit\(120000\)' -and
        $acceptanceTestDriverSource -match 'taskkill\.exe /PID \$childProcess\.Id /T /F' -and
        $acceptanceTestDriverSource -match '\$childProcess\.Refresh\(\)' -and
        $acceptanceTestDriverSource -match '\$exitCode = \[int\]\$childProcess\.ExitCode' -and
        $acceptanceTestDriverSource -match '\$exitCode -eq 0 -and \$childDetail -match' -and
        $acceptanceTestDriverSource -match 'ZIRCON_MVP_ACCEPTANCE_CHILD_PAYLOAD_PATH' -and
        $acceptanceTestDriverSource -match '\[Convert\]::ToBase64String' -and
        $acceptanceTestDriverSource -match '\[Text\.UTF8Encoding\]::new\(\$false\)') `
    'MVP acceptance test driver must bound nested driver execution, terminate its process tree on timeout, and preserve UTF-8 child payloads.'
Assert-True `
    ($acceptanceSource -match 'function ConvertTo-MvpAcceptanceUpperHex' -and $acceptanceSource -match '\[char\[\]\]::new\(\$Bytes\.Length \* 2\)') `
    'MVP acceptance evidence hashes must use one fixed-size uppercase encoding buffer.'
Assert-True `
    ($acceptanceSource -notmatch "ToString\('X2'\)") `
    'MVP acceptance evidence hashes must not use per-byte string formatting.'
Assert-True ($acceptanceTestsSource -notmatch ('Get-' + 'FileHash')) 'MVP acceptance fixtures must remain independent of the optional file-hash cmdlet.'
Assert-True ($acceptanceSource -match '\$canonicalPath = \(Resolve-ZirconWindowsPath -Path \$candidatePath\)\.OperationalPath') 'MVP acceptance must deduplicate reopened-project evidence through the resolver physical path.'
Assert-True ($acceptanceSource -match '\$resolvedTargetPath = \(Resolve-ZirconWindowsPath -Path \$targetPath\)\.OperationalPath') 'MVP acceptance manifest entries must compare physical target identities.'
Assert-True ($acceptanceSource -match '\$stagingResolution\.OperationalPath') 'MVP acceptance manifest containment must use the resolver operational staging path.'
Assert-True ($acceptanceSource -match '\$evidenceRootResolution\.OperationalPath') 'MVP acceptance EvidenceRoot isolation must use physical identity.'
Assert-True ($acceptanceSource -match 'function Assert-MvpAcceptanceArtifactRoot') 'MVP acceptance must define one approved artifact-root contract for staging and evidence.'
Assert-True ($acceptanceSource -match 'Get-MvpAcceptanceNativeDirectoryIdentity -Path \$OriginalPath' -and $acceptanceSource -match 'Assert-MvpAcceptanceArtifactRoot -Resolution \$stagingRootResolution -OriginalPath \$StagingRoot -Label ''StagingRoot''') 'MVP acceptance must no-follow validate the original StagingRoot before snapshot publication.'
Assert-True ($acceptanceSource -match 'Assert-MvpAcceptanceArtifactRoot -Resolution \$evidenceRootResolution -OriginalPath \$EvidenceRoot -Label ''EvidenceRoot''') 'MVP acceptance must no-follow validate an existing EvidenceRoot before publication.'
Assert-True ($acceptanceSource -match 'MvpArtifactStoragePolicy\.psm1') 'MVP acceptance must import the shared artifact storage policy.'
Assert-True ($acceptanceSource -match 'Resolve-MvpArtifactStoragePath') 'MVP acceptance artifact roots must resolve through registered storage namespaces.'
Assert-True ($acceptanceSource -match 'mvp-staging-runs' -and $acceptanceSource -match 'mvp-acceptance-evidence' -and $acceptanceSource -match 'mvp-test-fixtures') 'MVP acceptance must authorize only registered production or Coordinator fixture namespaces.'
Assert-True ($acceptanceSource -match 'Assert-MvpArtifactStorageCapabilityEvidence') 'MVP acceptance must revalidate staging volume capability evidence through the current storage policy.'
Assert-True ($acceptanceSource -match '-ExpectedPath \$CapabilityRoot') 'MVP acceptance capability validation must bind the original staging root instead of its detached snapshot path.'
Assert-True ($acceptanceSource -match 'function ConvertFrom-MvpJsonText') 'MVP acceptance must centralize date-preserving JSON parsing.'
Assert-True ($acceptanceSource -match 'MvpDatePreservingJson\.psm1' -and $acceptanceSource -notmatch 'function ConvertFrom-MvpJsonToken') 'MVP acceptance must preserve PS7.4 JSON date lexemes through the shared native conversion authority.'
Assert-True ($acceptanceSource -match '\$entry = ConvertFrom-MvpJsonText -Json \$line') 'MVP acceptance must preserve UTC timestamp lexemes while reading process journal JSONL.'
Assert-True ($acceptanceSource -match '\[IO\.StreamReader\]::new\(' -and $acceptanceSource -match '\$reader\.ReadLine\(\)' -and $acceptanceSource -notmatch '\$lines = @\(Get-Content -LiteralPath \$journalPath') 'MVP acceptance process journal reads must stream strict UTF-8 lines instead of materializing the whole JSONL file.'
Assert-True ($acceptanceSource -notmatch '\^\[D-F\]:' -and $acceptanceSource -notmatch '[D-F]:\\ZirconBuilds') 'MVP acceptance must not duplicate physical artifact-root literals.'
Assert-True ($acceptanceSource -match '\$decoderPath = \(Resolve-ZirconWindowsPath -Path \$Path\)\.DisplayPath') 'MVP acceptance PNG decoding must use the resolver display path at the System.Drawing API boundary.'
Assert-True ($acceptanceSource -match 'ZirconMvpAcceptancePngEvidence\]::Inspect\(\$decoderPath\)') 'MVP acceptance PNG decoding must pass the display path to System.Drawing.'
Assert-True ($acceptanceSource -notmatch '\(Resolve-Path -LiteralPath \$candidate\)\.Path') 'MVP acceptance evidence files must not fall back to PowerShell provider path resolution.'
Assert-True ($acceptanceSource -notmatch '\(Resolve-Path -LiteralPath \$targetPath\)\.Path') 'MVP acceptance manifest entries must not fall back to PowerShell provider path resolution.'
Assert-True ($buildSummaryEvidenceSource -match 'WindowsPathResolver\.psm1') 'MVP build-summary evidence must import the shared Windows path resolver.'
Assert-True ($buildSummaryEvidenceSource -match '\$resolvedPath = \(Resolve-ZirconWindowsPath -Path \$Path\)\.OperationalPath') 'MVP build-summary evidence must use the resolver physical identity for source-bound summaries.'
Assert-True ($buildSummaryEvidenceSource -match 'function Assert-MvpBuildSummaryOperationalChildPath') 'MVP build-summary evidence must define a physical containment boundary for gate logs.'
Assert-True ($buildSummaryEvidenceSource -match '\$resolvedEvidencePath = \(Resolve-ZirconWindowsPath -Path \$sourceEvidencePath\)\.OperationalPath') 'MVP build-summary evidence must resolve each gate log through the shared resolver.'
Assert-True ($buildSummaryEvidenceSource -match 'Assert-MvpBuildSummaryOperationalChildPath') 'MVP build-summary evidence must reject a gate log that resolves outside its summary directory.'
Assert-True ($buildSummaryEvidenceSource -notmatch '\(Resolve-Path -LiteralPath \$Path\)\.Path') 'MVP build-summary evidence must not fall back to PowerShell provider path resolution.'
Assert-True ($acceptanceSource -match 'function Assert-MvpProductInputManifestEvidence') 'F5 acceptance must validate the staged product-input manifest evidence.'
Assert-True ($acceptanceSource -match 'Get-MvpAcceptanceStagingTreeManifestPath -StagingRoot \$StagingRoot') 'MVP acceptance evidence publication must replace the source tree inventory with a partial-tree inventory.'
Assert-True ($acceptanceSource -match 'Assert-MvpAcceptanceStagingSnapshotLeaseTreeManifestMembership\s+`\s+-Lease \$acceptanceStagingSnapshotLease') 'MVP acceptance evidence publication must reject source-tree membership drift that occurs during recursive copy.'
Assert-True ($acceptanceSource -match 'Write-MvpAcceptanceStagingTreeManifest -StagingRoot \$partialRoot') 'MVP acceptance evidence publication must write its own complete tree manifest before leasing the partial tree.'
Assert-True ($acceptanceSource -match '\$manifest\[''product_input_manifest''\] = \$productInputManifestEvidence') 'F5 acceptance must retain validated product-input provenance in the final evidence manifest.'
Assert-True ($acceptanceSource -match '\$manifestEntryLogicalId = ''product-input-manifest''') 'F5 acceptance must require the original product-input manifest as a staged entry.'
Assert-True ($acceptanceSource -match "\.Equals\('build/mvp-product-inputs\.json'") 'F5 acceptance must require the canonical staged product-input manifest path.'
Assert-True ($acceptanceSource -match 'byte count differs from its staged manifest') 'F5 acceptance must compare product-input manifest bytes with the staged entry.'
Assert-True ($acceptanceSource -match 'hash differs from its staged manifest') 'F5 acceptance must compare the product-input manifest digest with the staged entry.'

function Write-FixtureJson {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)]$Value
    )

    [IO.File]::WriteAllText(
        $Path,
        ($Value | ConvertTo-Json -Depth 8),
        [Text.UTF8Encoding]::new($false)
    )
}

function ConvertFrom-FixtureJson {
    param([Parameter(Mandatory, ValueFromPipeline)][string]$Json)

    begin {
        $jsonBuilder = [Text.StringBuilder]::new()
    }
    process {
        if ($jsonBuilder.Length -gt 0) {
            [void]$jsonBuilder.AppendLine()
        }
        [void]$jsonBuilder.Append($Json)
    }
    end {
        $jsonText = $jsonBuilder.ToString()
        if ($null -eq (Get-Command ConvertFrom-MvpDatePreservingJson -ErrorAction SilentlyContinue)) {
            Import-Module $datePreservingJsonModule -Force -ErrorAction Stop
        }
        return ConvertFrom-MvpDatePreservingJson -Json $jsonText -AllowObjectArray
    }
}

function Copy-FixtureProductRuns {
    param([Parameter(Mandatory)]$Runs)

    $json = ConvertTo-Json -InputObject $Runs -Depth 12
    $decoded = ConvertFrom-FixtureJson -Json $json
    return $decoded
}

function Add-FixtureProcessTiming {
    param(
        [Parameter(Mandatory)]$Evidence,
        [int]$OffsetSeconds = 0
    )

    $startedAt = [DateTimeOffset]::Parse('2026-08-01T00:00:00Z').AddSeconds($OffsetSeconds)
    $endedAt = $startedAt.AddMilliseconds(250)
    $Evidence | Add-Member -NotePropertyName 'started_at_utc' -NotePropertyValue $startedAt.ToString('o') -Force
    $Evidence | Add-Member -NotePropertyName 'ended_at_utc' -NotePropertyValue $endedAt.ToString('o') -Force
    if ($null -eq $Evidence.PSObject.Properties['exit_code']) {
        $Evidence | Add-Member -NotePropertyName 'exit_code' -NotePropertyValue 0
    }
    return $Evidence
}

function Write-FixtureBuildSummary {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][ValidateSet('profile-contract', 'workspace')][string]$SummaryKind,
        [string]$SourceFingerprint = 'fixture-source-fingerprint'
    )

    Import-Module $buildGateRegistryModule -Force -ErrorAction Stop
    $gateRegistrySnapshot = Get-MvpBuildGateRegistrySnapshot
    $gateContracts = @(Get-MvpBuildGateContract `
            -SummaryKind $SummaryKind `
            -RegistrySnapshot $gateRegistrySnapshot)
    $summaryRoot = Split-Path -Parent $Path
    $gateLogRoot = Join-Path $summaryRoot 'logs'
    New-Item -ItemType Directory -Force -Path $gateLogRoot | Out-Null
    $gates = @()
    for ($index = 0; $index -lt $gateContracts.Count; $index++) {
        $gateContract = $gateContracts[$index]
        $gateLogPath = Join-Path $gateLogRoot "$($gateContract.gate_id).log"
        [IO.File]::WriteAllText($gateLogPath, "$($gateContract.command)`nresult=passed`n", [Text.UTF8Encoding]::new($false))
        $startedAt = [DateTimeOffset]::Parse('2026-08-01T00:00:00Z').AddSeconds($index)
        $gates += [ordered]@{
            gate_id = $gateContract.gate_id
            command = $gateContract.command
            status = 'passed'
            started_at_utc = $startedAt.ToString('o')
            ended_at_utc = $startedAt.AddMilliseconds(250).ToString('o')
            exit_code = 0
            evidence = [ordered]@{
                path = "logs/$($gateContract.gate_id).log"
                sha256 = Get-FixtureFileSha256 -Path $gateLogPath
                size_bytes = (Get-Item -LiteralPath $gateLogPath).Length
            }
        }
    }

    Write-FixtureJson -Path $Path -Value ([ordered]@{
        schema_version = 2
        summary_kind = $SummaryKind
        source_fingerprint = $SourceFingerprint
        status = 'passed'
        gate_registry = $gateRegistrySnapshot.receipt
        gates = $gates
    })
}

function Get-FixtureFileEvidence {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$RelativePath
    )

    return [ordered]@{
        path = $RelativePath.Replace('\', '/')
        sha256 = Get-FixtureFileSha256 -Path $Path
        size_bytes = (Get-Item -LiteralPath $Path).Length
    }
}

function New-FixtureSceneNodes {
    param(
        [Parameter(Mandatory)][double]$CubeTranslationX,
        [double]$CubeScaleX = 1.0
    )

    return @(
        [ordered]@{
            id = 1
            name = 'Camera'
            kind = 'Camera'
            parent = $null
            transform = [ordered]@{
                translation = @(0.0, 2.0, 5.0)
                rotation = @(0.0, 0.0, 0.0, 1.0)
                scale = @(1.0, 1.0, 1.0)
            }
            camera = [ordered]@{ z_near = 0.1; z_far = 1000.0 }
            mesh = $null
            directional_light = $null
            active = $true
            render_layer_mask = 1
            mobility = 'Static'
        },
        [ordered]@{
            id = 2
            name = 'Sun'
            kind = 'DirectionalLight'
            parent = $null
            transform = [ordered]@{
                translation = @(0.0, 0.0, 0.0)
                rotation = @(0.0, 0.0, 0.0, 1.0)
                scale = @(1.0, 1.0, 1.0)
            }
            camera = $null
            mesh = $null
            directional_light = [ordered]@{ intensity = 3.0; direction = @(-0.4, -1.0, -0.2) }
            active = $true
            render_layer_mask = 1
            mobility = 'Static'
        },
        [ordered]@{
            id = 3
            name = 'Cube'
            kind = 'Cube'
            parent = $null
            transform = [ordered]@{
                translation = @($CubeTranslationX, 0.0, 0.0)
                rotation = @(0.0, 0.0, 0.0, 1.0)
                scale = @($CubeScaleX, 1.0, 1.0)
            }
            camera = $null
            mesh = [ordered]@{
                model = 'fixture-cube-model-resource'
                material = 'fixture-default-material-resource'
            }
            directional_light = $null
            active = $true
            render_layer_mask = 1
            mobility = 'Static'
        }
    )
}

$preflightEvidenceModule = Join-Path $PSScriptRoot '..\mvp\MvpStagingPreflightEvidence.psm1'
$projectSaveEvidenceModule = Join-Path $PSScriptRoot '..\mvp\MvpProjectSaveEvidence.psm1'
$scenePersistenceEvidenceModule = Join-Path $PSScriptRoot '..\mvp\MvpScenePersistenceEvidence.psm1'
Import-Module $preflightEvidenceModule -Force
Import-Module $projectSaveEvidenceModule -Force
Import-Module $scenePersistenceEvidenceModule -Force
$snapshotContractTest = Join-Path $PSScriptRoot 'mvp-acceptance-staging-snapshot.Tests.ps1'
& pwsh -NoProfile -File $snapshotContractTest
if ($LASTEXITCODE -ne 0) {
    throw "MVP acceptance staging snapshot contract failed with exit code $LASTEXITCODE."
}
# The nested fixtures reload their private resolver dependencies. Keep the command that this
# top-level fixture invokes directly in the caller scope, matching the production driver.
Import-Module (Join-Path $PSScriptRoot '..\WindowsPathResolver.psm1') -Force -ErrorAction Stop

function ConvertTo-FixtureDiagnosticToken {
    param([Parameter(Mandatory)][string]$Value)

    $builder = [Text.StringBuilder]::new()
    foreach ($byte in [Text.Encoding]::UTF8.GetBytes($Value)) {
        if (($byte -ge [byte][char]'A' -and $byte -le [byte][char]'Z') -or
            ($byte -ge [byte][char]'a' -and $byte -le [byte][char]'z') -or
            ($byte -ge [byte][char]'0' -and $byte -le [byte][char]'9') -or
            $byte -in @([byte][char]'-', [byte][char]'.', [byte][char]'_', [byte][char]'~')) {
            $null = $builder.Append([char]$byte)
        }
        else {
            $null = $builder.Append('%')
            $null = $builder.Append($byte.ToString('X2', [Globalization.CultureInfo]::InvariantCulture))
        }
    }
    return $builder.ToString()
}

function New-FixtureProjectSaveDiagnostics {
    param([Parameter(Mandatory)][string]$ProjectToken)

    return (
        "editor_project_save result=started project=$ProjectToken pre_save_dirty=true pre_save_dirty_generation=1 save_token_generation=1`n" +
        "editor_project_save result=completed project=$ProjectToken pre_save_dirty_generation=1 save_token_generation=1 persisted_generation=1 save_mark=Marked`n"
    )
}

$nonInteractivePreflight = [pscustomobject]@{
    preflight = [pscustomobject]@{
        input_copy_bytes = 1
        evidence_reserve_bytes = 512MB
        required_free_space_bytes = 512MB + 1
        available_free_space_bytes = 1GB
        staging_drive_root = [IO.Path]::GetPathRoot([IO.Path]::GetFullPath($PSScriptRoot))
        interactive_desktop = [pscustomobject]@{
            required = $false
            user_interactive = $false
            session_id = $null
            monitor_count = $null
        }
    }
}
$nonInteractivePreflightRejected = $false
try {
    Assert-MvpStagingPreflightEvidence `
        -Manifest $nonInteractivePreflight `
        -EntryBytes 1 `
        -StagingRoot $PSScriptRoot
}
catch {
    $nonInteractivePreflightRejected = $_.Exception.Message -match 'interactive desktop.*required'
}
Assert-True $nonInteractivePreflightRejected 'Acceptance preflight allowed non-interactive product evidence.'

$markedProjectPath = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot 'project!mark'))
$canonicalMarkedProjectToken = ConvertTo-FixtureDiagnosticToken -Value $markedProjectPath
Assert-MvpProjectSaveLifecycleEvidence `
    -DiagnosticText (New-FixtureProjectSaveDiagnostics -ProjectToken $canonicalMarkedProjectToken) `
    -SaveOperationId 'file.project.save' `
    -SaveGeneration 1 `
    -ExpectedProjectPath $markedProjectPath | Out-Null

$verbatimMarkedProjectPath = (Resolve-ZirconWindowsPath -Path $markedProjectPath).OperationalPath
$canonicalVerbatimMarkedProjectToken = ConvertTo-FixtureDiagnosticToken -Value $verbatimMarkedProjectPath
Assert-MvpProjectSaveLifecycleEvidence `
    -DiagnosticText (New-FixtureProjectSaveDiagnostics -ProjectToken $canonicalVerbatimMarkedProjectToken) `
    -SaveOperationId 'file.project.save' `
    -SaveGeneration 1 `
    -ExpectedProjectPath $markedProjectPath | Out-Null

$legacyRawMarkToken = $canonicalMarkedProjectToken.Replace('%21', '!')
$legacyRawMarkRejected = $false
try {
    Assert-MvpProjectSaveLifecycleEvidence `
        -DiagnosticText (New-FixtureProjectSaveDiagnostics -ProjectToken $legacyRawMarkToken) `
        -SaveOperationId 'file.project.save' `
        -SaveGeneration 1 `
        -ExpectedProjectPath $markedProjectPath | Out-Null
}
catch {
    $legacyRawMarkRejected = $_.Exception.Message -match 'canonical percent encoding'
}
Assert-True $legacyRawMarkRejected 'Acceptance allowed a raw RFC2396 mark that the production encoder percent-encodes.'

$lowercaseHexProjectToken = $canonicalMarkedProjectToken.Replace('%5C', '%5c')
$lowercaseHexRejected = $false
try {
    Assert-MvpProjectSaveLifecycleEvidence `
        -DiagnosticText (New-FixtureProjectSaveDiagnostics -ProjectToken $lowercaseHexProjectToken) `
        -SaveOperationId 'file.project.save' `
        -SaveGeneration 1 `
        -ExpectedProjectPath $markedProjectPath | Out-Null
}
catch {
    $lowercaseHexRejected = $_.Exception.Message -match 'canonical percent encoding'
}
Assert-True $lowercaseHexRejected 'Acceptance allowed lowercase percent-encoding hex digits.'

$overEncodedUnreservedProjectToken = '%45' + $canonicalMarkedProjectToken.Substring(1)
$overEncodedUnreservedRejected = $false
try {
    Assert-MvpProjectSaveLifecycleEvidence `
        -DiagnosticText (New-FixtureProjectSaveDiagnostics -ProjectToken $overEncodedUnreservedProjectToken) `
        -SaveOperationId 'file.project.save' `
        -SaveGeneration 1 `
        -ExpectedProjectPath $markedProjectPath | Out-Null
}
catch {
    $overEncodedUnreservedRejected = $_.Exception.Message -match 'canonical percent encoding'
}
Assert-True $overEncodedUnreservedRejected 'Acceptance allowed percent encoding for an RFC3986 unreserved byte.'

$rawProjectPath = [IO.Path]::GetFullPath($PSScriptRoot)
$rawProjectSaveRejected = $false
try {
    Assert-MvpProjectSaveLifecycleEvidence `
        -DiagnosticText (New-FixtureProjectSaveDiagnostics -ProjectToken $rawProjectPath) `
        -SaveOperationId 'file.project.save' `
        -SaveGeneration 1 `
        -ExpectedProjectPath $rawProjectPath | Out-Null
}
catch {
    $rawProjectSaveRejected = $_.Exception.Message -match 'canonical percent encoding'
}
Assert-True $rawProjectSaveRejected 'Acceptance allowed a raw, non-canonical project path diagnostic token.'

$cyclicSceneNodes = New-FixtureSceneNodes -CubeTranslationX 42 |
    ConvertTo-Json -Depth 8 |
    ConvertFrom-Json
$cyclicSceneNodes[0].parent = 2
$cyclicSceneNodes[1].parent = 1
$cyclicSceneRejected = $false
try {
    Assert-MvpAutomationSceneSnapshot `
        -Snapshot ([pscustomobject]@{ scene_nodes = $cyclicSceneNodes }) `
        -SceneEntryCount 3 `
        -SelectedNodeId 3 `
        -SelectedNodeName 'Cube' `
        -InspectorTranslation @(42, 0, 0) `
        -InspectorScale @(1, 1, 1) `
        -Label 'Cyclic fixture' | Out-Null
}
catch {
    $cyclicSceneRejected = $_.Exception.Message -match 'parent cycle'
}
Assert-True $cyclicSceneRejected 'Acceptance allowed a multi-node scene parent cycle.'

function New-FixtureAutomationProcessEvidence {
    param(
        [Parameter(Mandatory)]$Report,
        [Parameter(Mandatory)][string]$RequestPath,
        [Parameter(Mandatory)][string]$RequestRelativePath,
        [Parameter(Mandatory)][string]$StagingRoot,
        [Parameter(Mandatory)][string]$EvidenceLabel,
        [string]$DiagnosticText
    )

    $normalizedReport = $Report | ConvertTo-Json -Depth 16 | ConvertFrom-FixtureJson
    $logsRoot = Join-Path $StagingRoot 'logs'
    $diagnosticsRoot = Join-Path $logsRoot "$EvidenceLabel.diagnostics"
    New-Item -ItemType Directory -Force -Path $logsRoot, $diagnosticsRoot | Out-Null
    $stdoutPath = Join-Path $logsRoot "$EvidenceLabel.stdout.log"
    $stderrPath = Join-Path $logsRoot "$EvidenceLabel.stderr.log"
    $diagnosticPath = Join-Path $diagnosticsRoot 'fixture.log'
    if ([string]::IsNullOrWhiteSpace($DiagnosticText)) {
        $saveRecord = @(
            $normalizedReport.records | Where-Object {
                [string]$_.binding_path -eq 'WorkbenchMenuBar/SaveProject:onClick'
            } | Select-Object -First 1
        )
        if ($saveRecord.Count -eq 1 -and $null -ne $saveRecord[0].save_generation) {
            $persistedGeneration = [UInt64]$saveRecord[0].save_generation
            $saveTokenGeneration = $persistedGeneration
            $reportedProjectPath = [string]$normalizedReport.project_path
            $projectPath = if ([IO.Path]::IsPathRooted($reportedProjectPath)) {
                [IO.Path]::GetFullPath($reportedProjectPath)
            }
            else {
                [IO.Path]::GetFullPath((Join-Path $StagingRoot $reportedProjectPath))
            }
            $encodedProjectPath = [Uri]::EscapeDataString($projectPath)
            $DiagnosticText =
                "editor_project_save result=started project=$encodedProjectPath pre_save_dirty=true pre_save_dirty_generation=$saveTokenGeneration save_token_generation=$saveTokenGeneration`n" +
                "editor_project_save result=completed project=$encodedProjectPath pre_save_dirty_generation=$saveTokenGeneration save_token_generation=$saveTokenGeneration persisted_generation=$persistedGeneration save_mark=Marked`n"
        }
        else {
            $DiagnosticText = "$EvidenceLabel diagnostics`n"
        }
    }
    [IO.File]::WriteAllText($stdoutPath, ($normalizedReport | ConvertTo-Json -Depth 16), [Text.UTF8Encoding]::new($false))
    [IO.File]::WriteAllText($stderrPath, "$EvidenceLabel stderr`n", [Text.UTF8Encoding]::new($false))
    [IO.File]::WriteAllText($diagnosticPath, $DiagnosticText, [Text.UTF8Encoding]::new($false))
    $normalizedReport | Add-Member -NotePropertyName 'automation_request' -NotePropertyValue (Get-FixtureFileEvidence -Path $RequestPath -RelativePath $RequestRelativePath)
    $normalizedReport | Add-Member -NotePropertyName 'stdout' -NotePropertyValue (Get-FixtureFileEvidence -Path $stdoutPath -RelativePath "logs/$EvidenceLabel.stdout.log")
    $normalizedReport | Add-Member -NotePropertyName 'stderr' -NotePropertyValue (Get-FixtureFileEvidence -Path $stderrPath -RelativePath "logs/$EvidenceLabel.stderr.log")
    $normalizedReport | Add-Member -NotePropertyName 'diagnostic_logs' -NotePropertyValue @(
        Get-FixtureFileEvidence -Path $diagnosticPath -RelativePath "logs/$EvidenceLabel.diagnostics/fixture.log"
    )
    $normalizedReport | Add-Member -NotePropertyName 'exit_code' -NotePropertyValue 0
    $offsetSeconds = if ($EvidenceLabel -match '^editor-baseline') {
        1
    }
    elseif ($EvidenceLabel -match '^editor-authoring') {
        4
    }
    elseif ($EvidenceLabel -match '^editor-reopen.*-1$') {
        5
    }
    elseif ($EvidenceLabel -match '^editor-reopen.*-2$') {
        7
    }
    else {
        0
    }
    return Add-FixtureProcessTiming -Evidence $normalizedReport -OffsetSeconds $offsetSeconds
}

function Write-FixtureVisiblePng {
    param(
        [Parameter(Mandatory)][string]$Path,
        [switch]$AfterAuthoring
    )

    Add-Type -AssemblyName System.Drawing
    $bitmap = [Drawing.Bitmap]::new(16, 16)
    try {
        for ($y = 0; $y -lt 16; $y++) {
            for ($x = 0; $x -lt 16; $x++) {
                $bitmap.SetPixel($x, $y, $(if ($x -lt 8) {
                    [Drawing.Color]::Black
                }
                elseif ($AfterAuthoring) {
                    [Drawing.Color]::FromArgb(255, 48, 192, 112)
                }
                else {
                    [Drawing.Color]::FromArgb(255, 64, 128, 255)
                }))
            }
        }
        $bitmap.Save($Path, [Drawing.Imaging.ImageFormat]::Png)
    }
    finally {
        $bitmap.Dispose()
    }
}

function Get-FixturePngPixelSha256 {
    param([Parameter(Mandatory)][string]$Path)

    Add-Type -AssemblyName System.Drawing
    $bitmap = [Drawing.Bitmap]::new($Path)
    $hasher = [Security.Cryptography.SHA256]::Create()
    $pixel = [byte[]]::new(4)
    try {
        for ($y = 0; $y -lt $bitmap.Height; $y++) {
            for ($x = 0; $x -lt $bitmap.Width; $x++) {
                $color = $bitmap.GetPixel($x, $y)
                $pixel[0] = $color.B
                $pixel[1] = $color.G
                $pixel[2] = $color.R
                $pixel[3] = $color.A
                [void]$hasher.TransformBlock($pixel, 0, $pixel.Length, $pixel, 0)
            }
        }
        [void]$hasher.TransformFinalBlock([byte[]]::new(0), 0, 0)
        return -join ($hasher.Hash | ForEach-Object { $_.ToString('X2') })
    }
    finally {
        $hasher.Dispose()
        $bitmap.Dispose()
    }
}

function New-FixtureRuntimeProductDiagnosticText {
    param([Parameter(Mandatory)]$Diagnostics)

    $fields = @(
        'frame_index',
        'viewport',
        'project_identity',
        'scene_uri',
        'selected_model_resource_id',
        'selected_material_resource_id',
        'render_backend',
        'render_adapter_type',
        'device_max_bind_groups',
        'device_max_texture_dimension_2d',
        'device_max_texture_array_layers',
        'device_max_sampled_textures_per_shader_stage',
        'device_max_storage_buffers_per_shader_stage',
        'device_max_storage_buffer_binding_size',
        'graph_executed_pass_count',
        'mesh_draw_count',
        'directional_light_count',
        'material_fallback_count',
        'material_validation_error_count',
        'input_viewport_resize_count',
        'input_pointer_move_count',
        'input_mouse_button_press_count',
        'input_mouse_button_release_count',
        'input_keyboard_press_count',
        'input_keyboard_release_count'
    )
    $tokens = @(
        "render_adapter=$($Diagnostics.render_adapter)"
        "render_adapter_type=$($Diagnostics.render_adapter_type)"
    )
    foreach ($field in $fields) {
        if ($field -eq 'render_adapter_type') {
            continue
        }
        $tokens += "$field=$($Diagnostics.$field)"
    }
    return (
        "runtime_first_frame_presented`n" +
        "runtime_process_teardown_complete`n" +
        "runtime_product_frame_capture_written`n" +
        ('runtime_product_frame_diagnostics ' + ($tokens -join ' ') + "`n")
    )
}

function Write-FixtureProcessJournal {
    param(
        [Parameter(Mandatory)]$Runs,
        [Parameter(Mandatory)][string]$Path,
        [hashtable]$ExitCodeOverrides = @{},
        [object[]]$AdditionalProcesses = @()
    )

    $processes = @(
        foreach ($run in @($Runs)) {
            [pscustomobject]@{
                phase = "$($run.product)-$($run.attempt)"
                evidence = $run
            }
        }
        foreach ($process in @($AdditionalProcesses)) {
            [pscustomobject]@{
                phase = [string]$process.phase
                evidence = $process.evidence
            }
        }
    )
    $lines = foreach ($process in $processes) {
        $phase = $process.phase
        $run = $process.evidence
        $exitCode = if ($ExitCodeOverrides.ContainsKey($phase)) {
            [int]$ExitCodeOverrides[$phase]
        }
        else {
            [int]$run.exit_code
        }
        [ordered]@{
            phase = $phase
            started_at_utc = $run.started_at_utc
            ended_at_utc = $run.ended_at_utc
            exit_code = $exitCode
            outcome = if ($exitCode -eq 0) { 'exited' } else { 'crashed' }
        } | ConvertTo-Json -Compress
    }
    [IO.File]::WriteAllText($Path, ($lines -join [Environment]::NewLine) + [Environment]::NewLine, [Text.UTF8Encoding]::new($false))
}

function Write-FixtureBlankPng {
    param([Parameter(Mandatory)][string]$Path)

    Add-Type -AssemblyName System.Drawing
    $bitmap = [Drawing.Bitmap]::new(16, 16)
    try {
        for ($y = 0; $y -lt 16; $y++) {
            for ($x = 0; $x -lt 16; $x++) {
                $bitmap.SetPixel($x, $y, [Drawing.Color]::Black)
            }
        }
        $bitmap.Save($Path, [Drawing.Imaging.ImageFormat]::Png)
    }
    finally {
        $bitmap.Dispose()
    }
}

$fixtureRoot = New-MvpTestFixtureRoot -Prefix 'zircon_mvp_acceptance'

try {
    $stagingRoot = Join-Path $fixtureRoot 'staging'
    $evidenceRoot = Join-Path $fixtureRoot 'evidence'
    New-Item -ItemType Directory -Force -Path (Join-Path $stagingRoot 'project') | Out-Null
    $unapprovedArtifactRoot = 'C:\ZirconBuilds\mvp-acceptance-rejected-' + [guid]::NewGuid().ToString('N')
    $unapprovedEvidenceResult = Invoke-MvpAcceptanceDriverForTest -Parameters @{
        StagingRoot = $stagingRoot
        EvidenceRoot = $unapprovedArtifactRoot
        ExpectedSourceFingerprint = 'fixture-source-fingerprint'
    }
    $unapprovedEvidenceRejected = $unapprovedEvidenceResult.ExitCode -ne 0 -and
        ((ConvertTo-FixtureProcessText -Output $unapprovedEvidenceResult.Output) -match '(?s)EvidenceRoot.*approved.*storage roots')
    Assert-True $unapprovedEvidenceRejected 'Acceptance evidence output outside approved non-C artifact roots was not rejected before publication.'

    $unapprovedStagingResult = Invoke-MvpAcceptanceDriverForTest -Parameters @{
        StagingRoot = $unapprovedArtifactRoot
        EvidenceRoot = $evidenceRoot
        ExpectedSourceFingerprint = 'fixture-source-fingerprint'
    }
    $unapprovedStagingRejected = $unapprovedStagingResult.ExitCode -ne 0 -and
        ((ConvertTo-FixtureProcessText -Output $unapprovedStagingResult.Output) -match '(?s)StagingRoot.*approved.*storage roots')
    Assert-True $unapprovedStagingRejected 'Acceptance staging outside approved non-C artifact roots was not rejected before snapshot publication.'

    $projectManifestPath = Join-Path $stagingRoot 'project\zircon-project.toml'
    [IO.File]::WriteAllText($projectManifestPath, "name = 'Fixture'`n", [Text.UTF8Encoding]::new($false))

    $fixtureProductInputs = @(
        [ordered]@{ logical_id = 'runtime-executable'; relative_path = 'runtime/zircon_runtime.exe'; contents = 'fixture-runtime-executable' },
        [ordered]@{ logical_id = 'runtime-library/runtime'; relative_path = 'runtime/zircon_runtime.dll'; contents = 'fixture-runtime-library' },
        [ordered]@{ logical_id = 'editor-executable'; relative_path = 'editor/zircon_editor.exe'; contents = 'fixture-editor-executable' },
        [ordered]@{ logical_id = 'runtime-library/editor'; relative_path = 'editor/zircon_runtime.dll'; contents = 'fixture-editor-runtime-library' }
    )
    $fixtureProductInputEntries = foreach ($fixtureProductInput in $fixtureProductInputs) {
        $path = Join-Path $stagingRoot $fixtureProductInput.relative_path
        New-Item -ItemType Directory -Force -Path (Split-Path -Parent $path) | Out-Null
        [IO.File]::WriteAllText($path, $fixtureProductInput.contents, [Text.UTF8Encoding]::new($false))
        [ordered]@{
            logical_id = $fixtureProductInput.logical_id
            target_relative_path = $fixtureProductInput.relative_path
            sha256 = Get-FixtureFileSha256 -Path $path
            size_bytes = (Get-Item -LiteralPath $path).Length
        }
    }

    $stagingManifestEntry = [ordered]@{
        logical_id = 'project/zircon-project.toml'
        target_relative_path = 'project/zircon-project.toml'
        sha256 = Get-FixtureFileSha256 -Path $projectManifestPath
        size_bytes = (Get-Item -LiteralPath $projectManifestPath).Length
    }
    $stagedProductInputManifestPath = Join-Path $stagingRoot 'build\mvp-product-inputs.json'
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $stagedProductInputManifestPath) | Out-Null
    $fixtureProductInputManifest = [ordered]@{
        schema_version = 1
        source_fingerprint = 'fixture-source-fingerprint'
        artifacts = @(
            $fixtureProductInputEntries | ForEach-Object {
                [ordered]@{
                    LogicalId = $_.logical_id
                    Bytes = $_.size_bytes
                    Sha256 = $_.sha256
                }
            }
        )
    }
    Write-FixtureJson -Path $stagedProductInputManifestPath -Value $fixtureProductInputManifest
    $stagedProductInputManifestEntry = [ordered]@{
        logical_id = 'product-input-manifest'
        target_relative_path = 'build/mvp-product-inputs.json'
        sha256 = Get-FixtureFileSha256 -Path $stagedProductInputManifestPath
        size_bytes = (Get-Item -LiteralPath $stagedProductInputManifestPath).Length
    }
    $stagingManifestFixture = [ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        toolchain = 'rustc 1.89.0 (fixture)'
        target = 'x86_64-pc-windows-msvc'
        staged_at_utc = '2026-07-29T00:00:00Z'
        entries = @($stagingManifestEntry, $stagedProductInputManifestEntry) + @($fixtureProductInputEntries)
        product_input_manifest = [ordered]@{
            schema_version = 1
            target_relative_path = $stagedProductInputManifestEntry.target_relative_path
            size_bytes = $stagedProductInputManifestEntry.size_bytes
            sha256 = $stagedProductInputManifestEntry.sha256
            source_fingerprint = 'fixture-source-fingerprint'
            artifacts = @(
                $fixtureProductInputEntries | ForEach-Object {
                    [ordered]@{
                        logical_id = $_.logical_id
                        bytes = $_.size_bytes
                        sha256 = $_.sha256
                    }
                }
            )
        }
    }
    $authoringRequestPath = Join-Path $stagingRoot 'authoring\automation.json'
    $reopenRequestPath = Join-Path $stagingRoot 'reopen\automation.json'
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $authoringRequestPath), (Split-Path -Parent $reopenRequestPath) | Out-Null
    [IO.File]::WriteAllText($authoringRequestPath, '{"bindings":["authoring"]}', [Text.UTF8Encoding]::new($false))
    [IO.File]::WriteAllText($reopenRequestPath, '{"bindings":["reopen"]}', [Text.UTF8Encoding]::new($false))
    $stagingManifestFixture.entries += @(
        [ordered]@{
            logical_id = 'authoring-automation-request'
            target_relative_path = 'authoring/automation.json'
            sha256 = Get-FixtureFileSha256 -Path $authoringRequestPath
            size_bytes = (Get-Item -LiteralPath $authoringRequestPath).Length
        },
        [ordered]@{
            logical_id = 'reopen-automation-request'
            target_relative_path = 'reopen/automation.json'
            sha256 = Get-FixtureFileSha256 -Path $reopenRequestPath
            size_bytes = (Get-Item -LiteralPath $reopenRequestPath).Length
        }
    )
    [Int64]$fixtureInputCopyBytes = (
        $stagingManifestFixture.entries |
            ForEach-Object { [Int64]$_['size_bytes'] } |
            Measure-Object -Sum
    ).Sum
    [Int64]$fixtureEvidenceReserveBytes = 512MB
    $stagingManifestFixture['preflight'] = [ordered]@{
        input_copy_bytes = $fixtureInputCopyBytes
        evidence_reserve_bytes = $fixtureEvidenceReserveBytes
        required_free_space_bytes = $fixtureInputCopyBytes + $fixtureEvidenceReserveBytes
        available_free_space_bytes = $fixtureInputCopyBytes + $fixtureEvidenceReserveBytes + 1GB
        staging_drive_root = [IO.Path]::GetPathRoot($stagingRoot)
        interactive_desktop = [ordered]@{
            required = $true
            user_interactive = $true
            session_id = 1
            monitor_count = 1
        }
    }
    $storagePolicySnapshot = Get-MvpArtifactStoragePolicySnapshot
    $storageRootResolution = Resolve-MvpArtifactStorageRootPath `
        -Path $stagingRoot `
        -CapabilityClass 'windows-local-artifact' `
        -PolicySnapshot $storagePolicySnapshot
    $storageDriveRoot = [IO.Path]::GetPathRoot($storageRootResolution.root_display_path)
    $storageDrive = [IO.DriveInfo]::new($storageDriveRoot)
    $stagingManifestFixture['storage_capability'] = [ordered]@{
        schema_version = 1
        capability_kind = 'zircon.mvp-artifact-storage-capability'
        policy = $storagePolicySnapshot.receipt
        root_id = $storageRootResolution.root_id
        capability_class = $storageRootResolution.capability_class
        drive_root = $storageDriveRoot
        drive_type = $storageDrive.DriveType.ToString()
        file_system = $storageDrive.DriveFormat
        required_free_space_bytes = $stagingManifestFixture.preflight.required_free_space_bytes
        available_free_space_bytes = $stagingManifestFixture.preflight.available_free_space_bytes
        durable_file_flush_supported = $true
        same_volume_atomic_move_supported = $true
        captured_at_utc = [DateTimeOffset]::UtcNow.ToString('o')
    }
    $stagingManifestPath = Join-Path $stagingRoot 'staging-manifest.json'
    Write-FixtureJson -Path $stagingManifestPath -Value $stagingManifestFixture
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = @(
            [ordered]@{
                product = 'runtime'
                project = 'project'
                attempt = 1
                exit_code = 0
                first_frame_presented = $true
                teardown_complete = $true
                runtime_product_diagnostics = [ordered]@{
                    frame_index = 1
                    viewport = '16x16'
                    project_identity = 'fixture-project'
                    scene_uri = 'res://scenes/main.scene.toml'
                    selected_model_resource_id = 'fixture-cube-model-resource'
                    selected_material_resource_id = 'fixture-default-material-resource'
                    render_backend = 'fixture-wgpu'
                    render_adapter = 'Fixture WGPU Adapter'
                    render_adapter_type = 'discrete_gpu'
                    device_max_bind_groups = 5
                    device_max_texture_dimension_2d = 16384
                    device_max_texture_array_layers = 256
                    device_max_sampled_textures_per_shader_stage = 16
                    device_max_storage_buffers_per_shader_stage = 8
                    device_max_storage_buffer_binding_size = 134217728
                    graph_executed_pass_count = 1
                    mesh_draw_count = 1
                    directional_light_count = 1
                    material_fallback_count = 0
                    material_validation_error_count = 0
                    input_viewport_resize_count = 2
                    input_pointer_move_count = 1
                    input_mouse_button_press_count = 1
                    input_mouse_button_release_count = 1
                    input_keyboard_press_count = 1
                    input_keyboard_release_count = 1
                }
            },
            [ordered]@{
                product = 'runtime'
                project = 'project'
                attempt = 2
                exit_code = 0
                first_frame_presented = $true
                teardown_complete = $true
                runtime_product_diagnostics = [ordered]@{
                    frame_index = 2
                    viewport = '16x16'
                    project_identity = 'fixture-project'
                    scene_uri = 'res://scenes/main.scene.toml'
                    selected_model_resource_id = 'fixture-cube-model-resource'
                    selected_material_resource_id = 'fixture-default-material-resource'
                    render_backend = 'fixture-wgpu'
                    render_adapter = 'Fixture WGPU Adapter'
                    render_adapter_type = 'discrete_gpu'
                    device_max_bind_groups = 5
                    device_max_texture_dimension_2d = 16384
                    device_max_texture_array_layers = 256
                    device_max_sampled_textures_per_shader_stage = 16
                    device_max_storage_buffers_per_shader_stage = 8
                    device_max_storage_buffer_binding_size = 134217728
                    graph_executed_pass_count = 1
                    mesh_draw_count = 1
                    directional_light_count = 1
                    material_fallback_count = 0
                    material_validation_error_count = 0
                    input_viewport_resize_count = 2
                    input_pointer_move_count = 1
                    input_mouse_button_press_count = 1
                    input_mouse_button_release_count = 1
                    input_keyboard_press_count = 1
                    input_keyboard_release_count = 1
                }
            },
            [ordered]@{
                product = 'editor'
                project = 'project'
                attempt = 1
                exit_code = 0
                first_frame_presented = $true
                teardown_complete = $true
            },
            [ordered]@{
                product = 'editor'
                project = 'project'
                attempt = 2
                exit_code = 0
                first_frame_presented = $true
                teardown_complete = $true
            }
        )
    })

    $result = @(
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot $evidenceRoot `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -Json
    ) | ConvertFrom-FixtureJson

    Assert-True ($result.run_id -eq 'fixture-stage') 'Acceptance output lost the staging run identity.'
    Assert-True ($result.source_fingerprint -eq 'fixture-source-fingerprint') 'Acceptance output lost the source fingerprint.'
    Assert-True ($result.render_backend -eq 'fixture-wgpu') 'Acceptance output lost the stable runtime render backend.'
    Assert-True ($result.render_adapter -eq 'Fixture WGPU Adapter') 'Acceptance output lost the stable runtime render adapter.'
    Assert-True ($result.render_adapter_type -eq 'discrete_gpu') 'Acceptance output lost the stable runtime adapter type.'
    Assert-True ($result.render_device_limits.max_bind_groups -eq 5) 'Acceptance output lost negotiated device-limit evidence.'
    Assert-True ($result.staged_project_root -eq 'project') 'Acceptance output lost the canonical staged project root.'
    Assert-True ($result.staging_manifest_sha256 -match '^[0-9A-F]{64}$') 'Acceptance output did not bind the staging manifest hash.'
    Assert-True ($result.startup_summary_sha256 -match '^[0-9A-F]{64}$') 'Acceptance output did not bind the startup summary hash.'
    Assert-True ($result.product_runs.Count -eq 4) 'Acceptance output lost repeated staged product evidence.'
    Assert-True (Test-Path -LiteralPath (Join-Path $evidenceRoot 'manifest.json') -PathType Leaf) 'Acceptance output did not write manifest.json.'

    $driverRootJunction = Join-Path $fixtureRoot 'staging-root-junction'
    New-Item -ItemType Junction -Path $driverRootJunction -Target $stagingRoot -ErrorAction Stop | Out-Null
    $driverRootJunctionRejected = $false
    try {
        & $driver `
            -StagingRoot $driverRootJunction `
            -EvidenceRoot (Join-Path $fixtureRoot 'root-junction-evidence') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' | Out-Null
    }
    catch {
        $driverRootJunctionRejected = $_.Exception.Message -match 'reparse point'
    }
    finally {
        if (Test-Path -LiteralPath $driverRootJunction) {
            [IO.Directory]::Delete($driverRootJunction, $false)
        }
    }
    Assert-True $driverRootJunctionRejected 'Acceptance driver accepted a staging root junction.'

    $manifest = Get-Content -Raw (Join-Path $evidenceRoot 'manifest.json') | ConvertFrom-FixtureJson
    Assert-True ($manifest.run_id -eq 'fixture-stage') 'Evidence manifest lost the staging run identity.'
    Assert-True ($manifest.source_fingerprint -eq 'fixture-source-fingerprint') 'Evidence manifest lost the source fingerprint.'
    Assert-True ($manifest.toolchain -eq 'rustc 1.89.0 (fixture)') 'Evidence manifest lost the staged Rust toolchain.'
    Assert-True ($manifest.target -eq 'x86_64-pc-windows-msvc') 'Evidence manifest lost the staged Rust target.'
    Assert-True ($manifest.render_backend -eq 'fixture-wgpu') 'Evidence manifest lost the stable runtime render backend.'
    Assert-True ($manifest.render_adapter -eq 'Fixture WGPU Adapter') 'Evidence manifest lost the stable runtime render adapter.'
    Assert-True ($manifest.render_adapter_type -eq 'discrete_gpu') 'Evidence manifest lost the stable runtime adapter type.'
    Assert-True ($manifest.render_device_limits.max_storage_buffer_binding_size -eq 134217728) 'Evidence manifest lost negotiated device-limit evidence.'
    Assert-True ($manifest.staging_manifest_sha256 -eq $result.staging_manifest_sha256) 'Evidence manifest is not bound to the staging manifest hash.'
    Assert-True ($manifest.startup_summary_sha256 -eq $result.startup_summary_sha256) 'Evidence manifest is not bound to the startup summary hash.'
    Assert-True ($manifest.evidence_layout_version -eq 1) 'Evidence manifest did not declare its self-contained layout.'
    Assert-True ($manifest.staging_manifest -eq 'staging-manifest.json') 'Evidence manifest did not retain a local staging manifest path.'
    Assert-True ($manifest.startup_summary -eq 'startup-summary.json') 'Evidence manifest did not retain a local startup summary path.'
    Assert-True (@($manifest.evidence_files).Count -ge 3) 'Evidence manifest did not inventory its copied source evidence.'
    Assert-True (Test-Path -LiteralPath (Join-Path $evidenceRoot 'staging-manifest.json') -PathType Leaf) 'Evidence package did not copy its staging manifest.'
    Assert-True (Test-Path -LiteralPath (Join-Path $evidenceRoot 'startup-summary.json') -PathType Leaf) 'Evidence package did not copy its startup summary.'
    Assert-True (Test-Path -LiteralPath (Join-Path $evidenceRoot 'project/zircon-project.toml') -PathType Leaf) 'Evidence package did not copy its canonical project.'

    $detachedStagingRoot = Join-Path $fixtureRoot 'staging-detached'
    Move-Item -LiteralPath $stagingRoot -Destination $detachedStagingRoot
    try {
        foreach ($evidenceFile in @($manifest.evidence_files)) {
            $evidencePath = Join-Path $evidenceRoot ([string]$evidenceFile.path)
            Assert-True (Test-Path -LiteralPath $evidencePath -PathType Leaf) "Detached evidence file '$($evidenceFile.path)' is missing."
            Assert-True ((Get-FixtureFileSha256 -Path $evidencePath) -eq $evidenceFile.sha256) "Detached evidence file '$($evidenceFile.path)' has a hash mismatch."
            Assert-True ((Get-Item -LiteralPath $evidencePath).Length -eq $evidenceFile.size_bytes) "Detached evidence file '$($evidenceFile.path)' has a size mismatch."
        }
    }
    finally {
        Move-Item -LiteralPath $detachedStagingRoot -Destination $stagingRoot
    }

    $f0StartupProducts = Copy-FixtureProductRuns -Runs $manifest.product_runs
    foreach ($runtimeRun in @($f0StartupProducts | Where-Object { $_.product -eq 'runtime' })) {
        [void]$runtimeRun.PSObject.Properties.Remove('runtime_product_diagnostics')
    }
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = $f0StartupProducts
    })
    $f0EvidenceRoot = Join-Path $fixtureRoot 'evidence-f0-no-runtime-diagnostics'
    $f0Result = @(
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot $f0EvidenceRoot `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -Json
    ) | ConvertFrom-FixtureJson
    Assert-True ($null -eq $f0Result.render_backend) 'F0 startup evidence should not require a persisted-scene render backend.'
    Assert-True ($null -eq $f0Result.render_adapter) 'F0 startup evidence should not require a persisted-scene render adapter.'
    Assert-True ($null -eq $f0Result.render_device_limits) 'F0 startup evidence should not require negotiated device-limit evidence.'
    $f0Manifest = Get-Content -LiteralPath (Join-Path $f0EvidenceRoot 'manifest.json') -Raw | ConvertFrom-FixtureJson
    Assert-True ($null -eq $f0Manifest.render_backend) 'F0 evidence manifest should preserve the absence of runtime diagnostics.'
    Assert-True ($null -eq $f0Manifest.render_adapter) 'F0 evidence manifest should preserve the absence of runtime adapter diagnostics.'
    Assert-True ($null -eq $f0Manifest.render_device_limits) 'F0 evidence manifest should preserve the absence of device-limit diagnostics.'
    Assert-True (Test-Path -LiteralPath $stagingRoot -PathType Container) 'Successful F0 acceptance must retain the source staging root.'
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = $manifest.product_runs
    })

    $renderBackendDriftSummary = Get-Content -LiteralPath (Join-Path $stagingRoot 'startup-summary.json') -Raw | ConvertFrom-FixtureJson
    $renderBackendDriftRuntime = @($renderBackendDriftSummary.products | Where-Object { $_.product -eq 'runtime' } | Select-Object -Last 1)
    Assert-True ($renderBackendDriftRuntime.Count -eq 1) 'Backend drift fixture requires a second runtime run.'
    $renderBackendDriftRuntime[0].runtime_product_diagnostics.render_backend = 'fixture-vulkan'
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value $renderBackendDriftSummary
    $renderBackendDriftRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'evidence-render-backend-drift') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' | Out-Null
    }
    catch {
        $renderBackendDriftRejected = $_.Exception.Message -match "'render_backend' differs between attempts|disagree on render_backend"
    }
    finally {
        Assert-True (Test-Path -LiteralPath $stagingRoot -PathType Container) 'Rejected acceptance must retain the source staging root.'
        Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
            run_id = 'fixture-stage'
            source_fingerprint = 'fixture-source-fingerprint'
            staged_project_root = 'project'
            products = $manifest.product_runs
        })

    $stagingTreeManifestModule = Join-Path $PSScriptRoot '..\mvp\MvpAcceptanceStagingTreeManifest.psm1'
    Import-Module $stagingTreeManifestModule -Force -ErrorAction Stop
    Write-MvpAcceptanceStagingTreeManifest -StagingRoot $stagingRoot | Out-Null
    }
    Assert-True $renderBackendDriftRejected 'Acceptance did not reject render-backend drift across runtime runs.'

    $missingToolchainManifest = $stagingManifestFixture | ConvertTo-Json -Depth 8 | ConvertFrom-FixtureJson
    [void]$missingToolchainManifest.PSObject.Properties.Remove('toolchain')
    Write-FixtureJson -Path $stagingManifestPath -Value $missingToolchainManifest
    $missingToolchainRejected = $false
    $missingToolchainReturned = $false
    $missingToolchainFailure = $null
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'evidence-missing-toolchain') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' | Out-Null
        $missingToolchainReturned = $true
    }
    catch {
        $missingToolchainFailure = $_.Exception.Message
        $missingToolchainRejected = $_.Exception.Message -match "Staging manifest is missing 'toolchain'"
    }
    finally {
        Assert-True (Test-Path -LiteralPath $stagingRoot -PathType Container) 'Missing-toolchain rejection must retain the source staging root.'
        Write-FixtureJson -Path $stagingManifestPath -Value $stagingManifestFixture
    }
    Assert-True $missingToolchainRejected ("Acceptance did not reject staging evidence without toolchain provenance. returned={0}; failure={1}" -f $missingToolchainReturned, $missingToolchainFailure)

    $missingPreflightManifest = $stagingManifestFixture | ConvertTo-Json -Depth 8 | ConvertFrom-FixtureJson
    [void]$missingPreflightManifest.PSObject.Properties.Remove('preflight')
    Write-FixtureJson -Path $stagingManifestPath -Value $missingPreflightManifest
    $missingPreflightRejected = $false
    $missingPreflightFailure = $null
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'evidence-missing-preflight') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' | Out-Null
    }
    catch {
        $missingPreflightFailure = $_.Exception.Message
        $missingPreflightRejected = $_.Exception.Message -match "Staging manifest is missing 'preflight'"
    }
    finally {
        Assert-True (Test-Path -LiteralPath $stagingRoot -PathType Container) 'Missing-preflight rejection must retain the source staging root.'
        Write-FixtureJson -Path $stagingManifestPath -Value $stagingManifestFixture
    }
    Assert-True $missingPreflightRejected "Acceptance did not reject staging evidence without preflight provenance; failure=$missingPreflightFailure"

    $driftedPreflightManifest = $stagingManifestFixture | ConvertTo-Json -Depth 8 | ConvertFrom-FixtureJson
    $driftedPreflightManifest.preflight.input_copy_bytes += 1
    Write-FixtureJson -Path $stagingManifestPath -Value $driftedPreflightManifest
    $driftedPreflightRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'evidence-drifted-preflight') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' | Out-Null
    }
    catch {
        $driftedPreflightRejected = $_.Exception.Message -match 'preflight input_copy_bytes.*staging entries'
    }
    finally {
        Write-FixtureJson -Path $stagingManifestPath -Value $stagingManifestFixture
    }
    Assert-True $driftedPreflightRejected 'Acceptance did not reject a preflight byte budget detached from final staging entries.'

    $runtimeDiagnostics = $manifest.product_runs | Where-Object { $_.product -eq 'runtime' } | Select-Object -First 1 -ExpandProperty runtime_product_diagnostics
    Assert-True ($runtimeDiagnostics.input_viewport_resize_count -eq 2) 'Evidence manifest lost runtime viewport-resize input evidence.'
    Assert-True ($runtimeDiagnostics.input_keyboard_release_count -eq 1) 'Evidence manifest lost runtime input evidence.'
    Assert-True ($runtimeDiagnostics.render_adapter -eq 'Fixture WGPU Adapter') 'Evidence manifest lost the adapter identity.'
    Assert-True ($runtimeDiagnostics.render_adapter_type -eq 'discrete_gpu') 'Evidence manifest lost the adapter type.'
    Assert-True ($runtimeDiagnostics.device_max_bind_groups -eq 5) 'Evidence manifest lost actual device limits.'

    $productEvidenceRuns = Copy-FixtureProductRuns -Runs $manifest.product_runs
    Assert-True (
        $productEvidenceRuns.Count -eq $manifest.product_runs.Count
    ) 'Product evidence fixture copy must retain every staged product run.'
    $logsRoot = Join-Path $stagingRoot 'logs'
    $capturesRoot = Join-Path $stagingRoot 'captures'
    New-Item -ItemType Directory -Force -Path $logsRoot, $capturesRoot | Out-Null
    foreach ($productRun in $productEvidenceRuns) {
        $null = Add-FixtureProcessTiming -Evidence $productRun -OffsetSeconds ([int]$productRun.attempt)
        $prefix = "$($productRun.product)-$($productRun.attempt)"
        $stdoutPath = Join-Path $logsRoot "$prefix.stdout.log"
        $stderrPath = Join-Path $logsRoot "$prefix.stderr.log"
        $diagnosticRoot = Join-Path $logsRoot "$prefix.diagnostics"
        $diagnosticPath = Join-Path $diagnosticRoot 'fixture.log'
        New-Item -ItemType Directory -Force -Path $diagnosticRoot | Out-Null
        [IO.File]::WriteAllText($stdoutPath, "$prefix stdout`n", [Text.UTF8Encoding]::new($false))
        [IO.File]::WriteAllText($stderrPath, "$prefix stderr`n", [Text.UTF8Encoding]::new($false))
        $diagnosticText = if ($productRun.product -eq 'runtime') {
            New-FixtureRuntimeProductDiagnosticText -Diagnostics $productRun.runtime_product_diagnostics
        }
        else {
            "editor_first_frame_presented`neditor_process_teardown_complete`n"
        }
        [IO.File]::WriteAllText($diagnosticPath, $diagnosticText, [Text.UTF8Encoding]::new($false))
        $productRun | Add-Member -NotePropertyName 'stdout' -NotePropertyValue (Get-FixtureFileEvidence -Path $stdoutPath -RelativePath "logs/$prefix.stdout.log")
        $productRun | Add-Member -NotePropertyName 'stderr' -NotePropertyValue (Get-FixtureFileEvidence -Path $stderrPath -RelativePath "logs/$prefix.stderr.log")
        $productRun | Add-Member -NotePropertyName 'diagnostic_logs' -NotePropertyValue @(
            Get-FixtureFileEvidence -Path $diagnosticPath -RelativePath "logs/$prefix.diagnostics/fixture.log"
        )
        if ($productRun.product -eq 'runtime') {
            $capturePath = Join-Path $capturesRoot "$prefix.png"
            Write-FixtureVisiblePng -Path $capturePath
            $productRun | Add-Member -NotePropertyName 'frame_capture' -NotePropertyValue ([ordered]@{
                path = "captures/$prefix.png"
                sha256 = Get-FixtureFileSha256 -Path $capturePath
                size_bytes = (Get-Item -LiteralPath $capturePath).Length
                pixel_sha256 = Get-FixturePngPixelSha256 -Path $capturePath
                width = 16
                height = 16
                non_background_pixels = 128
                non_transparent_pixels = 256
            })
        }
    }
    $processJournalPath = Join-Path $logsRoot 'process-execution-journal.jsonl'
    Write-FixtureProcessJournal -Runs $productEvidenceRuns -Path $processJournalPath
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = $productEvidenceRuns
    })
    $productEvidence = @(
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'product-evidence') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -RequireProductEvidence `
            -Json
    ) | ConvertFrom-FixtureJson
    Assert-True ($productEvidence.product_runs.Count -eq 4) 'Acceptance output lost independently verified staged product evidence.'
    Assert-True (Test-Path -LiteralPath $logsRoot -PathType Container) 'Acceptance package publication must retain the source staging logs directory.'
    Assert-True (Test-Path -LiteralPath $processJournalPath -PathType Leaf) 'Acceptance package publication must retain the source process execution journal.'
    foreach ($productRun in $productEvidenceRuns) {
        foreach ($diagnosticLog in @($productRun.diagnostic_logs)) {
            $diagnosticPath = Join-Path $stagingRoot ([string]$diagnosticLog.path)
            Assert-True (Test-Path -LiteralPath $diagnosticPath -PathType Leaf) "Acceptance package publication must retain staged diagnostic log '$($diagnosticLog.path)'."
        }
    }

    Write-FixtureProcessJournal `
        -Runs $productEvidenceRuns `
        -Path $processJournalPath `
        -ExitCodeOverrides @{ 'runtime-1' = 1 }
    $productEvidenceJournalExitRejected = $false
    try {
        & $driver -StagingRoot $stagingRoot -EvidenceRoot (Join-Path $fixtureRoot 'product-evidence-journal-exit') -ExpectedSourceFingerprint 'fixture-source-fingerprint' -RequireProductEvidence | Out-Null
    }
    catch {
        $productEvidenceJournalExitRejected = $_.Exception.Message -match 'exit_code.*process journal|process journal.*outcome'
    }
    finally {
        Assert-True (Test-Path -LiteralPath $logsRoot -PathType Container) 'Rejected product-evidence acceptance must retain the source staging logs directory.'
        foreach ($productRun in $productEvidenceRuns) {
            foreach ($diagnosticLog in @($productRun.diagnostic_logs)) {
                $diagnosticPath = Join-Path $stagingRoot ([string]$diagnosticLog.path)
                Assert-True (Test-Path -LiteralPath $diagnosticPath -PathType Leaf) "Rejected product-evidence acceptance must retain staged diagnostic log '$($diagnosticLog.path)'."
            }
        }
        Write-FixtureProcessJournal -Runs $productEvidenceRuns -Path $processJournalPath
    }
    Assert-True $productEvidenceJournalExitRejected 'Product evidence accepted a successful summary that disagreed with its process journal exit status.'

    $zeroInputDiagnosticRuns = Copy-FixtureProductRuns -Runs $productEvidenceRuns
    $zeroInputRuntime = @($zeroInputDiagnosticRuns | Where-Object { $_.product -eq 'runtime' -and $_.attempt -eq 1 })[0]
    $zeroInputDiagnostics = $zeroInputRuntime.runtime_product_diagnostics | ConvertTo-Json -Depth 8 | ConvertFrom-FixtureJson
    $zeroInputDiagnostics.input_viewport_resize_count = 0
    $zeroInputDiagnosticPath = Join-Path $logsRoot 'runtime-1.diagnostics\fixture.log'
    [IO.File]::WriteAllText(
        $zeroInputDiagnosticPath,
        (New-FixtureRuntimeProductDiagnosticText -Diagnostics $zeroInputDiagnostics),
        [Text.UTF8Encoding]::new($false)
    )
    $zeroInputRuntime.diagnostic_logs[0] = Get-FixtureFileEvidence -Path $zeroInputDiagnosticPath -RelativePath 'logs/runtime-1.diagnostics/fixture.log'
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = $zeroInputDiagnosticRuns
    })
    $productEvidenceZeroInputRejected = $false
    try {
        & $driver -StagingRoot $stagingRoot -EvidenceRoot (Join-Path $fixtureRoot 'product-evidence-zero-input') -ExpectedSourceFingerprint 'fixture-source-fingerprint' -RequireProductEvidence | Out-Null
    }
    catch {
        $productEvidenceZeroInputRejected = $_.Exception.Message -match 'input_viewport_resize_count|diagnostics.*differs'
    }
    finally {
        $runtimeDiagnosticPath = Join-Path $logsRoot 'runtime-1.diagnostics\fixture.log'
        $runtimeDiagnostics = @($productEvidenceRuns | Where-Object { $_.product -eq 'runtime' -and $_.attempt -eq 1 })[0].runtime_product_diagnostics
        [IO.File]::WriteAllText(
            $runtimeDiagnosticPath,
            (New-FixtureRuntimeProductDiagnosticText -Diagnostics $runtimeDiagnostics),
            [Text.UTF8Encoding]::new($false)
        )
    }
    Assert-True $productEvidenceZeroInputRejected 'Product evidence accepted a summary whose runtime input counters disagreed with the captured diagnostic log.'

    $productEvidenceDuplicateArtifact = Copy-FixtureProductRuns -Runs $productEvidenceRuns
    $duplicateRuntime = @($productEvidenceDuplicateArtifact | Where-Object { $_.product -eq 'runtime' -and $_.attempt -eq 2 })[0]
    $firstRuntime = @($productEvidenceDuplicateArtifact | Where-Object { $_.product -eq 'runtime' -and $_.attempt -eq 1 })[0]
    $duplicateRuntime.stdout = $firstRuntime.stdout
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = $productEvidenceDuplicateArtifact
    })
    $productEvidenceDuplicateArtifactRejected = $false
    try {
        & $driver -StagingRoot $stagingRoot -EvidenceRoot (Join-Path $fixtureRoot 'product-evidence-duplicate-artifact') -ExpectedSourceFingerprint 'fixture-source-fingerprint' -RequireProductEvidence | Out-Null
    }
    catch {
        $productEvidenceDuplicateArtifactRejected = $_.Exception.Message -match 'reuses.*stdout|runtime-2\.stdout\.log'
    }
    finally {
        Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
            run_id = 'fixture-stage'
            source_fingerprint = 'fixture-source-fingerprint'
            staged_project_root = 'project'
            products = $productEvidenceRuns
        })
    }
    Assert-True $productEvidenceDuplicateArtifactRejected 'Product evidence accepted a stdout artifact reused by two product attempts.'

    $foreignStdoutPath = Join-Path $logsRoot 'foreign-runtime.stdout.log'
    [IO.File]::WriteAllText($foreignStdoutPath, "foreign runtime stdout`n", [Text.UTF8Encoding]::new($false))
    $productEvidenceForeignArtifact = Copy-FixtureProductRuns -Runs $productEvidenceRuns
    $foreignRuntime = @($productEvidenceForeignArtifact | Where-Object { $_.product -eq 'runtime' -and $_.attempt -eq 2 })[0]
    $foreignRuntime.stdout = Get-FixtureFileEvidence -Path $foreignStdoutPath -RelativePath 'logs/foreign-runtime.stdout.log'
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = $productEvidenceForeignArtifact
    })
    $productEvidenceForeignArtifactRejected = $false
    try {
        & $driver -StagingRoot $stagingRoot -EvidenceRoot (Join-Path $fixtureRoot 'product-evidence-foreign-artifact') -ExpectedSourceFingerprint 'fixture-source-fingerprint' -RequireProductEvidence | Out-Null
    }
    catch {
        $productEvidenceForeignArtifactRejected = $_.Exception.Message -match 'runtime-2\.stdout\.log'
    }
    finally {
        Assert-True (Test-Path -LiteralPath (Join-Path $stagingRoot 'startup-summary.json') -PathType Leaf) 'Rejected foreign-artifact acceptance must retain the source staging startup summary.'
        Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
            run_id = 'fixture-stage'
            source_fingerprint = 'fixture-source-fingerprint'
            staged_project_root = 'project'
            products = $productEvidenceRuns
        })
    }
    Assert-True $productEvidenceForeignArtifactRejected 'Product evidence accepted a unique stdout file from another product attempt.'

    $missingProjectCreationRejected = $false
    $missingProjectCreationReturned = $false
    try {
        & $driver -StagingRoot $stagingRoot -EvidenceRoot (Join-Path $fixtureRoot 'missing-project-creation') -ExpectedSourceFingerprint 'fixture-source-fingerprint' -RequireProjectCreationEvidence | Out-Null
        $missingProjectCreationReturned = $true
    }
    catch {
        $missingProjectCreationRejected = $_.Exception.Message -match 'project_creation'
    }
    Assert-True $missingProjectCreationRejected "Acceptance did not reject a fixed F5 request without staged project-creation evidence; returned=$missingProjectCreationReturned."
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $fixtureRoot 'missing-project-creation'))) 'Missing project-creation evidence left a partial evidence root.'

    $productEvidenceHashMismatch = Copy-FixtureProductRuns -Runs $productEvidenceRuns
    $productEvidenceHashMismatch[0].stdout.sha256 = ('0' * 64)
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = $productEvidenceHashMismatch
    })
    $productEvidenceHashMismatchRejected = $false
    try {
        & $driver -StagingRoot $stagingRoot -EvidenceRoot (Join-Path $fixtureRoot 'product-evidence-hash-mismatch') -ExpectedSourceFingerprint 'fixture-source-fingerprint' -RequireProductEvidence | Out-Null
    }
    catch {
        $productEvidenceHashMismatchRejected = $_.Exception.Message -match 'stdout hash mismatch'
    }
    Assert-True $productEvidenceHashMismatchRejected 'Product evidence with a stdout hash mismatch was not rejected.'

    $productEvidencePathEscape = Copy-FixtureProductRuns -Runs $productEvidenceRuns
    $productEvidencePathEscape[0].stderr.path = '../outside-staging.log'
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = $productEvidencePathEscape
    })
    $productEvidencePathEscapeRejected = $false
    try {
        & $driver -StagingRoot $stagingRoot -EvidenceRoot (Join-Path $fixtureRoot 'product-evidence-path-escape') -ExpectedSourceFingerprint 'fixture-source-fingerprint' -RequireProductEvidence | Out-Null
    }
    catch {
        $productEvidencePathEscapeRejected = $_.Exception.Message -match 'unsafe relative path'
    }
    Assert-True $productEvidencePathEscapeRejected 'Product evidence with a path escape was not rejected.'

    $productEvidenceMissingFile = Copy-FixtureProductRuns -Runs $productEvidenceRuns
    $productEvidenceMissingFile[0].diagnostic_logs[0].path = 'logs/missing.diagnostic.log'
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = $productEvidenceMissingFile
    })
    $productEvidenceMissingFileRejected = $false
    try {
        & $driver -StagingRoot $stagingRoot -EvidenceRoot (Join-Path $fixtureRoot 'product-evidence-missing-file') -ExpectedSourceFingerprint 'fixture-source-fingerprint' -RequireProductEvidence | Out-Null
    }
    catch {
        $productEvidenceMissingFileRejected = $_.Exception.Message -match 'does not exist in the staging root'
    }
    Assert-True $productEvidenceMissingFileRejected 'Product evidence with a missing diagnostic file was not rejected.'

    $productEvidencePngMetadataMismatch = Copy-FixtureProductRuns -Runs $productEvidenceRuns
    $productEvidencePngMetadataMismatch[0].frame_capture.non_background_pixels = 0
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = $productEvidencePngMetadataMismatch
    })
    $productEvidencePngMetadataMismatchRejected = $false
    try {
        & $driver -StagingRoot $stagingRoot -EvidenceRoot (Join-Path $fixtureRoot 'product-evidence-png-metadata') -ExpectedSourceFingerprint 'fixture-source-fingerprint' -RequireProductEvidence | Out-Null
    }
    catch {
        $productEvidencePngMetadataMismatchRejected = $_.Exception.Message -match 'non_background_pixels.*differs'
    }
    Assert-True $productEvidencePngMetadataMismatchRejected 'Product evidence with mismatched PNG metadata was not rejected.'

    $blankCapturePath = Join-Path $capturesRoot 'runtime-blank.png'
    Write-FixtureBlankPng -Path $blankCapturePath
    $productEvidenceBlankPng = Copy-FixtureProductRuns -Runs $productEvidenceRuns
    $blankRuntime = @($productEvidenceBlankPng | Where-Object { $_.product -eq 'runtime' })[0]
    $blankRuntime.frame_capture.path = 'captures/runtime-blank.png'
    $blankRuntime.frame_capture.sha256 = Get-FixtureFileSha256 -Path $blankCapturePath
    $blankRuntime.frame_capture.size_bytes = (Get-Item -LiteralPath $blankCapturePath).Length
    $blankRuntime.frame_capture.pixel_sha256 = Get-FixturePngPixelSha256 -Path $blankCapturePath
    $blankRuntime.frame_capture.non_background_pixels = 0
    $blankRuntime.frame_capture.non_transparent_pixels = 256
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = $productEvidenceBlankPng
    })
    $productEvidenceBlankPngRejected = $false
    try {
        & $driver -StagingRoot $stagingRoot -EvidenceRoot (Join-Path $fixtureRoot 'product-evidence-blank-png') -ExpectedSourceFingerprint 'fixture-source-fingerprint' -RequireProductEvidence | Out-Null
    }
    catch {
        $productEvidenceBlankPngRejected = $_.Exception.Message -match "blank or visually insufficient|must be 'captures/runtime-1.png'"
    }
    Assert-True $productEvidenceBlankPngRejected 'Product evidence with a blank PNG was not rejected.'

    $createdProjectRoot = Join-Path $stagingRoot 'project\ZirconMvpFixture'
    New-Item -ItemType Directory -Force -Path $createdProjectRoot | Out-Null
    $creationLogsRoot = Join-Path $stagingRoot 'logs'
    $creationDiagnosticsRoot = Join-Path $creationLogsRoot 'editor-create.diagnostics'
    New-Item -ItemType Directory -Force -Path $creationDiagnosticsRoot | Out-Null
    $creationStdoutPath = Join-Path $creationLogsRoot 'editor-create.stdout.log'
    $creationStderrPath = Join-Path $creationLogsRoot 'editor-create.stderr.log'
    $creationDiagnosticPath = Join-Path $creationDiagnosticsRoot 'fixture.log'
    [IO.File]::WriteAllText($creationStdoutPath, "created`n", [Text.UTF8Encoding]::new($false))
    [IO.File]::WriteAllText($creationStderrPath, "created stderr`n", [Text.UTF8Encoding]::new($false))
    $encodedCreatedProjectRoot = [Uri]::EscapeDataString('.\ZirconMvpFixture')
    $creationDiagnosticText =
        "editor_first_frame_presented`neditor_process_teardown_complete`neditor_product_frame_capture_written`n" +
        "editor_product_frame_diagnostics project_path=$encodedCreatedProjectRoot selected_node_id=3 selected_node_name=Cube inspector_translation_x=0 inspector_translation_y=0 inspector_translation_z=0 inspector_scale_x=1.00 inspector_scale_y=1.00 inspector_scale_z=1.00`n" +
        "editor_project_open result=completed project_root=$encodedCreatedProjectRoot manifest_identity=Fixture%20Project%40v1 scene_uri=res%3A%2F%2Fscenes%2Fmain.scene.toml registry_asset_count=4 registry_ready_asset_count=4 registry_failed_asset_count=0 registry_diagnostic_count=0 project_generation=1 project_generation_publish_epoch=1 catalog_asset_count=4 settings_source=persisted-v1`n"
    [IO.File]::WriteAllText(
        $creationDiagnosticPath,
        $creationDiagnosticText,
        [Text.UTF8Encoding]::new($false)
    )
    $creationCapturePath = Join-Path $capturesRoot 'editor-before-edit.png'
    Write-FixtureVisiblePng -Path $creationCapturePath
    $projectCreationFixture = [ordered]@{
        exit_code = 0
        started_at_utc = '2026-08-01T00:00:00.0000000+00:00'
        ended_at_utc = '2026-08-01T00:00:00.2500000+00:00'
        first_frame_presented = $true
        teardown_complete = $true
        stdout = Get-FixtureFileEvidence -Path $creationStdoutPath -RelativePath 'logs/editor-create.stdout.log'
        stderr = Get-FixtureFileEvidence -Path $creationStderrPath -RelativePath 'logs/editor-create.stderr.log'
        diagnostic_logs = @(Get-FixtureFileEvidence -Path $creationDiagnosticPath -RelativePath 'logs/editor-create.diagnostics/fixture.log')
        editor_window_capture = [ordered]@{
            path = 'captures/editor-before-edit.png'
            sha256 = Get-FixtureFileSha256 -Path $creationCapturePath
            size_bytes = (Get-Item -LiteralPath $creationCapturePath).Length
            pixel_sha256 = Get-FixturePngPixelSha256 -Path $creationCapturePath
            width = 16
            height = 16
            non_background_pixels = 128
            non_transparent_pixels = 256
        }
        editor_product_diagnostics = [ordered]@{
            project_path = 'project/ZirconMvpFixture'
            selected_node_id = 3
            selected_node_name = 'Cube'
            inspector_translation_x = '0'
            inspector_translation_y = '0'
            inspector_translation_z = '0'
            inspector_scale_x = '1.00'
            inspector_scale_y = '1.00'
            inspector_scale_z = '1.00'
        }
        project_open = [ordered]@{
            project_root = 'project/ZirconMvpFixture'
            manifest_identity = 'Fixture Project@v1'
            scene_uri = 'res://scenes/main.scene.toml'
            registry_asset_count = 4
            registry_ready_asset_count = 4
            registry_failed_asset_count = 0
            registry_diagnostic_count = 0
            project_generation = 1
            project_generation_publish_epoch = 1
            catalog_asset_count = 4
            settings_source = 'persisted-v1'
        }
    }
    $createdProjectProducts = Copy-FixtureProductRuns -Runs $manifest.product_runs
    foreach ($productRun in $createdProjectProducts) {
        $productRun.project = 'project/ZirconMvpFixture'
    }
    Write-FixtureProcessJournal `
        -Runs $productEvidenceRuns `
        -Path $processJournalPath `
        -AdditionalProcesses @(
            [pscustomobject]@{ phase = 'editor-create'; evidence = $projectCreationFixture }
        )
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project/ZirconMvpFixture'
        project_creation = $projectCreationFixture
        products = $createdProjectProducts
    })
    $createdProjectEvidence = @(
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'created-project-evidence') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -RequireProjectCreationEvidence `
            -Json
    ) | ConvertFrom-FixtureJson
    Assert-True ($createdProjectEvidence.staged_project_root -eq 'project/ZirconMvpFixture') 'Acceptance did not preserve the canonical created-project relative root.'
    Assert-True ($createdProjectEvidence.project_creation.exit_code -eq 0) 'Acceptance did not preserve verified staged editor project-creation evidence.'
    Assert-True ($createdProjectEvidence.project_creation.project_open.manifest_identity -eq 'Fixture Project@v1') 'Acceptance did not preserve the editor project-open manifest identity.'

    [IO.File]::AppendAllText($creationDiagnosticPath, "tampered`n", [Text.UTF8Encoding]::new($false))
    $projectCreationDiagnosticTamperRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'project-creation-diagnostic-tamper') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -RequireProjectCreationEvidence | Out-Null
    }
    catch {
        $projectCreationDiagnosticTamperRejected = $_.Exception.Message -match 'Project creation diagnostic log hash mismatch'
    }
    finally {
        [IO.File]::WriteAllText($creationDiagnosticPath, $creationDiagnosticText, [Text.UTF8Encoding]::new($false))
    }
    Assert-True $projectCreationDiagnosticTamperRejected 'Acceptance did not reject a modified project-creation diagnostic log.'

    $unicodeProjectName = -join ([char[]]@(0x9879, 0x76EE))
    $unicodeProjectPathSegment = -join ([char[]]@(0x8DEF, 0x5F84))
    $unicodeProjectRelativeRoot = "project/$unicodeProjectName $unicodeProjectPathSegment"
    $unicodeProjectRoot = Join-Path $stagingRoot $unicodeProjectRelativeRoot
    New-Item -ItemType Directory -Force -Path $unicodeProjectRoot | Out-Null
    $unicodeDiagnosticPath = $creationDiagnosticPath
    $encodedUnicodeProjectRoot = [Uri]::EscapeDataString(".\$unicodeProjectName $unicodeProjectPathSegment")
    [IO.File]::WriteAllText(
        $unicodeDiagnosticPath,
        "editor_first_frame_presented`neditor_process_teardown_complete`neditor_product_frame_capture_written`n" +
        "editor_product_frame_diagnostics project_path=$encodedUnicodeProjectRoot selected_node_id=3 selected_node_name=Cube inspector_translation_x=0 inspector_translation_y=0 inspector_translation_z=0 inspector_scale_x=1.00 inspector_scale_y=1.00 inspector_scale_z=1.00`n" +
        "editor_project_open result=completed project_root=$encodedUnicodeProjectRoot manifest_identity=%E9%A1%B9%E7%9B%AE%20MVP%40v1 scene_uri=res%3A%2F%2Fscenes%2Fmain.scene.toml registry_asset_count=4 registry_ready_asset_count=4 registry_failed_asset_count=0 registry_diagnostic_count=0 project_generation=1 project_generation_publish_epoch=1 catalog_asset_count=4 settings_source=persisted-v1`n",
        [Text.UTF8Encoding]::new($false)
    )
    $unicodeProjectCreation = $projectCreationFixture | ConvertTo-Json -Depth 12 | ConvertFrom-FixtureJson
    $unicodeProjectCreation.diagnostic_logs = @(Get-FixtureFileEvidence -Path $unicodeDiagnosticPath -RelativePath 'logs/editor-create.diagnostics/fixture.log')
    $unicodeProjectCreation.editor_product_diagnostics.project_path = $unicodeProjectRelativeRoot
    $unicodeProjectCreation.project_open.project_root = $unicodeProjectRelativeRoot
    $unicodeProjectCreation.project_open.manifest_identity = "$unicodeProjectName MVP@v1"
    $unicodeProjectProducts = Copy-FixtureProductRuns -Runs $manifest.product_runs
    foreach ($productRun in $unicodeProjectProducts) {
        $productRun.project = $unicodeProjectRelativeRoot
    }
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = $unicodeProjectRelativeRoot
        project_creation = $unicodeProjectCreation
        products = $unicodeProjectProducts
    })
    $unicodeProjectEvidence = @(
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'unicode-created-project-evidence') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -RequireProjectCreationEvidence `
            -Json
    ) | ConvertFrom-FixtureJson
    Assert-True `
        ($unicodeProjectEvidence.staged_project_root -eq $unicodeProjectRelativeRoot) `
        "Acceptance did not preserve a UTF-8 staged project root. actual='$($unicodeProjectEvidence.staged_project_root)' expected='$unicodeProjectRelativeRoot'."
    Assert-True ($unicodeProjectEvidence.project_creation.project_open.manifest_identity -eq "$unicodeProjectName MVP@v1") 'Acceptance did not preserve a UTF-8 project-open manifest identity.'
    [IO.File]::WriteAllText($creationDiagnosticPath, $creationDiagnosticText, [Text.UTF8Encoding]::new($false))

    $invalidProjectOpenEvidence = $projectCreationFixture | ConvertTo-Json -Depth 12 | ConvertFrom-FixtureJson
    $invalidProjectOpenEvidence.project_open.scene_uri = 'res://scenes/not-main.scene.toml'
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project/ZirconMvpFixture'
        project_creation = $invalidProjectOpenEvidence
        products = $createdProjectProducts
    })
    $invalidProjectOpenEvidenceRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'invalid-project-open-evidence') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -RequireProjectCreationEvidence | Out-Null
    }
    catch {
        $invalidProjectOpenEvidenceRejected = $_.Exception.Message -match 'project_open.*scene_uri|scene_uri.*project_open'
    }
    Assert-True $invalidProjectOpenEvidenceRejected 'Acceptance did not reject a tampered project-open diagnostic summary.'

    $degradedProjectOpenDiagnosticText = $creationDiagnosticText.Replace(
        'result=completed',
        'result=degraded'
    ).Replace(
        'registry_asset_count=4 registry_ready_asset_count=4 registry_failed_asset_count=0',
        'registry_asset_count=4 registry_ready_asset_count=3 registry_failed_asset_count=0'
    )
    [IO.File]::WriteAllText(
        $creationDiagnosticPath,
        $degradedProjectOpenDiagnosticText,
        [Text.UTF8Encoding]::new($false)
    )
    $degradedProjectOpenEvidence = $projectCreationFixture | ConvertTo-Json -Depth 12 | ConvertFrom-FixtureJson
    $degradedProjectOpenEvidence.diagnostic_logs = @(
        Get-FixtureFileEvidence -Path $creationDiagnosticPath -RelativePath 'logs/editor-create.diagnostics/fixture.log'
    )
    $degradedProjectOpenEvidence.project_open.registry_ready_asset_count = 3
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project/ZirconMvpFixture'
        project_creation = $degradedProjectOpenEvidence
        products = $createdProjectProducts
    })
    $degradedProjectOpenEvidenceRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'degraded-project-open-evidence') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -RequireProjectCreationEvidence | Out-Null
    }
    catch {
        $degradedProjectOpenEvidenceRejected = $_.Exception.Message -match 'did not complete successfully'
    }
    Assert-True $degradedProjectOpenEvidenceRejected 'Acceptance did not reject a degraded project-open diagnostic.'
    [IO.File]::WriteAllText($creationDiagnosticPath, $creationDiagnosticText, [Text.UTF8Encoding]::new($false))
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project/ZirconMvpFixture'
        project_creation = $projectCreationFixture
        products = $createdProjectProducts
    })

    $missingAuthoringEvidenceRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'missing-authoring-evidence') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -RequireAuthoringAutomation | Out-Null
    }
    catch {
        $missingAuthoringEvidenceRejected = $_.Exception.Message -match 'authoring_automation'
    }
    Assert-True $missingAuthoringEvidenceRejected 'Acceptance with required authoring evidence did not reject a startup summary without an automation report.'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $fixtureRoot 'missing-authoring-evidence'))) 'Missing authoring evidence left a partial evidence root.'

    $authoringAutomationFixture = [ordered]@{
        project_path = 'project'
        project_identity = 'fixture-project'
        manifest_identity = 'Fixture Project@v1'
        scene_uri = 'res://scenes/main.scene.toml'
        selected_model_resource_id = 'fixture-cube-model-resource'
        selected_material_resource_id = 'fixture-default-material-resource'
        opened_project_inspection_generation = 1
        records = @(
            [ordered]@{
                binding_path = 'Hierarchy/SelectCube:onClick'
                source = 'Cli'
                operation_id = $null
                transaction_id = $null
                save_generation = $null
            },
            [ordered]@{
                binding_path = 'Inspector/TransformPositionXCommit:onSubmit'
                source = 'Cli'
                operation_id = 'inspector.field.apply_batch'
                transaction_id = 1
                save_generation = $null
            },
            [ordered]@{
                binding_path = 'Inspector/TransformScaleXCommit:onSubmit'
                source = 'Cli'
                operation_id = 'inspector.field.apply_batch'
                transaction_id = 2
                save_generation = $null
            },
            [ordered]@{
                binding_path = 'WorkbenchMenuBar/Undo:onClick'
                source = 'Cli'
                operation_id = 'edit.history.undo'
                transaction_id = $null
                save_generation = $null
            },
            [ordered]@{
                binding_path = 'WorkbenchMenuBar/Redo:onClick'
                source = 'Cli'
                operation_id = 'edit.history.redo'
                transaction_id = $null
                save_generation = $null
            },
            [ordered]@{
                binding_path = 'WorkbenchMenuBar/SaveProject:onClick'
                source = 'Cli'
                operation_id = 'file.project.save'
                transaction_id = $null
                save_generation = 2
            }
        )
        snapshot = [ordered]@{
            project_open = $true
            scene_entry_count = 3
            selected_node_id = 3
            selected_node_name = 'Cube'
            inspector_translation = @('42', '0', '0')
            inspector_scale = @('1.25', '1.00', '1.00')
            scene_nodes = New-FixtureSceneNodes -CubeTranslationX 42 -CubeScaleX 1.25
        }
    }
    $reopenAutomationFixture = @(
        [ordered]@{
            project_path = 'project'
            project_identity = 'fixture-project'
            manifest_identity = 'Fixture Project@v1'
            scene_uri = 'res://scenes/main.scene.toml'
            selected_model_resource_id = 'fixture-cube-model-resource'
            selected_material_resource_id = 'fixture-default-material-resource'
            opened_project_inspection_generation = 1
            records = @(
                [ordered]@{
                    binding_path = 'Hierarchy/SelectCube:onClick'
                    source = 'Cli'
                }
            )
            snapshot = [ordered]@{
                project_open = $true
                scene_entry_count = 3
                selected_node_id = 3
                selected_node_name = 'Cube'
                inspector_translation = @('42', '0', '0')
                inspector_scale = @('1.25', '1.00', '1.00')
                scene_nodes = New-FixtureSceneNodes -CubeTranslationX 42 -CubeScaleX 1.25
            }
        },
        [ordered]@{
            project_path = 'project'
            project_identity = 'fixture-project'
            manifest_identity = 'Fixture Project@v1'
            scene_uri = 'res://scenes/main.scene.toml'
            selected_model_resource_id = 'fixture-cube-model-resource'
            selected_material_resource_id = 'fixture-default-material-resource'
            opened_project_inspection_generation = 1
            records = @(
                [ordered]@{
                    binding_path = 'Hierarchy/SelectCube:onClick'
                    source = 'Cli'
                }
            )
            snapshot = [ordered]@{
                project_open = $true
                scene_entry_count = 3
                selected_node_id = 3
                selected_node_name = 'Cube'
                inspector_translation = @('42', '0', '0')
                inspector_scale = @('1.25', '1.00', '1.00')
                scene_nodes = New-FixtureSceneNodes -CubeTranslationX 42 -CubeScaleX 1.25
            }
        }
    )
    $authoringAutomationFixture = New-FixtureAutomationProcessEvidence `
        -Report $authoringAutomationFixture `
        -RequestPath $authoringRequestPath `
        -RequestRelativePath 'authoring/automation.json' `
        -StagingRoot $stagingRoot `
        -EvidenceLabel 'editor-authoring'
    $baselineAutomationFixture = Copy-FixtureProductRuns -Runs @($reopenAutomationFixture[0])
    $baselineAutomationFixture.snapshot.inspector_translation[0] = '0'
    $baselineAutomationFixture.snapshot.inspector_scale[0] = '1.00'
    $baselineAutomationFixture.snapshot.scene_nodes[2].transform.translation[0] = 0.0
    $baselineAutomationFixture.snapshot.scene_nodes[2].transform.scale[0] = 1.0
    $baselineAutomationFixture = New-FixtureAutomationProcessEvidence `
        -Report $baselineAutomationFixture `
        -RequestPath $reopenRequestPath `
        -RequestRelativePath 'reopen/automation.json' `
        -StagingRoot $stagingRoot `
        -EvidenceLabel 'editor-baseline'
    $reopenAutomationOne = New-FixtureAutomationProcessEvidence `
        -Report $reopenAutomationFixture[0] `
        -RequestPath $reopenRequestPath `
        -RequestRelativePath 'reopen/automation.json' `
        -StagingRoot $stagingRoot `
        -EvidenceLabel 'editor-reopen-1'
    $reopenAutomationTwo = New-FixtureAutomationProcessEvidence `
        -Report $reopenAutomationFixture[1] `
        -RequestPath $reopenRequestPath `
        -RequestRelativePath 'reopen/automation.json' `
        -StagingRoot $stagingRoot `
        -EvidenceLabel 'editor-reopen-2'
    $reopenAutomationFixture = @($reopenAutomationOne, $reopenAutomationTwo)
    $null = Add-FixtureProcessTiming -Evidence $baselineAutomationFixture -OffsetSeconds 1
    $null = Add-FixtureProcessTiming -Evidence $authoringAutomationFixture -OffsetSeconds 4
    $null = Add-FixtureProcessTiming -Evidence $reopenAutomationFixture[0] -OffsetSeconds 5
    $null = Add-FixtureProcessTiming -Evidence $reopenAutomationFixture[1] -OffsetSeconds 7
    Assert-True ($authoringAutomationFixture -isnot [array]) 'Authoring process evidence fixture emitted multiple pipeline values instead of one report.'
    Assert-True ($null -ne $authoringAutomationFixture.PSObject.Properties['records']) 'Authoring process evidence fixture lost the original binding records before serialization.'
    $capturedAuthoringReportFixture = Get-Content -LiteralPath (Join-Path $stagingRoot 'logs\editor-authoring.stdout.log') -Raw | ConvertFrom-FixtureJson
    Assert-True ($null -ne $capturedAuthoringReportFixture.PSObject.Properties['records']) 'Authoring process evidence fixture did not serialize the original binding records into stdout.'
    $authoringProductRuns = Copy-FixtureProductRuns -Runs $manifest.product_runs
    $afterAuthoringRuntime = Copy-FixtureProductRuns -Runs @($manifest.product_runs | Where-Object { $_.product -eq 'runtime' } | Select-Object -Last 1)
    $afterAuthoringRuntime.attempt = 3
    $afterAuthoringRuntime.runtime_product_diagnostics.frame_index = 3
    $authoringProductRuns += $afterAuthoringRuntime
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = $authoringProductRuns
        baseline_automation = $baselineAutomationFixture
        authoring_automation = $authoringAutomationFixture
        reopen_automation = $reopenAutomationFixture
    })
    $authoringStartupFixture = Get-Content -LiteralPath (Join-Path $stagingRoot 'startup-summary.json') -Raw | ConvertFrom-FixtureJson
    Assert-True ($null -ne $authoringStartupFixture.PSObject.Properties['reopen_automation']) 'Authoring acceptance fixture did not serialize its repeated reopen reports.'
    Assert-True ($null -ne $authoringStartupFixture.authoring_automation.PSObject.Properties['records']) 'Authoring acceptance fixture did not retain authoring records in its startup summary.'
    Assert-True (@($authoringStartupFixture.authoring_automation.records).Count -eq 6) 'Authoring acceptance fixture did not retain all authoring records in its startup summary.'
    $authoringEvidenceRoot = Join-Path $fixtureRoot 'authoring-evidence'
    try {
        $authoringEvidence = @(
            & $driver `
                -StagingRoot $stagingRoot `
                -EvidenceRoot $authoringEvidenceRoot `
                -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
                -RequireAuthoringAutomation `
                -RequireReopenAutomation `
                -Json
        ) | ConvertFrom-FixtureJson
    }
    catch {
        throw "Authoring automation positive fixture was rejected: $($_.Exception.Message)"
    }
    Assert-True ($null -ne $authoringEvidence.authoring_automation) 'Acceptance output lost required authoring automation evidence.'
    Assert-True ($authoringEvidence.authoring_automation.records.Count -eq 6) 'Acceptance output lost the normal authoring binding sequence.'
    Assert-True ($authoringEvidence.reopen_automation.Count -eq 2) 'Acceptance output lost the independent reopened-project reports.'
    Assert-True ($authoringEvidence.authoring_automation.project_save_lifecycle.pre_save_dirty -eq $true) 'Acceptance output lost the pre-save dirty state.'
    Assert-True ($authoringEvidence.authoring_automation.project_save_lifecycle.pre_save_dirty_generation -eq 2) 'Acceptance output lost the pre-save dirty generation.'
    Assert-True ($authoringEvidence.authoring_automation.project_save_lifecycle.save_token_generation -eq 2) 'Acceptance output lost the save-token generation.'
    Assert-True ($authoringEvidence.authoring_automation.project_save_lifecycle.persisted_generation -eq 2) 'Acceptance output lost the persisted generation.'
    Assert-True (@($authoringEvidence.authoring_automation.project_save_lifecycle.reopened_inspection_generations).Count -eq 2) 'Acceptance output lost the two reopened project generations.'

    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = $authoringProductRuns
        baseline_automation = $baselineAutomationFixture
        authoring_automation = $authoringAutomationFixture
        reopen_automation = @($reopenAutomationFixture[0], $reopenAutomationFixture[0])
    })
    $duplicateReopenEvidenceRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'duplicate-reopen-process-evidence') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -RequireAuthoringAutomation `
            -RequireReopenAutomation | Out-Null
    }
    catch {
        $duplicateReopenEvidenceRejected = $_.Exception.Message -match 'reopened-project.*process evidence.*duplicate'
    }
    Assert-True $duplicateReopenEvidenceRejected 'Acceptance allowed one reopened-project process evidence set to masquerade as two independent runs.'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $fixtureRoot 'duplicate-reopen-process-evidence'))) 'Duplicate reopen evidence left a partial evidence root.'

    $encodedStagedProjectPath = [Uri]::EscapeDataString([IO.Path]::GetFullPath((Join-Path $stagingRoot 'project')))
    $wrongProjectSaveAuthoring = New-FixtureAutomationProcessEvidence `
        -Report (Get-Content -LiteralPath (Join-Path $stagingRoot 'logs\editor-authoring.stdout.log') -Raw | ConvertFrom-FixtureJson) `
        -RequestPath $authoringRequestPath `
        -RequestRelativePath 'authoring/automation.json' `
        -StagingRoot $stagingRoot `
        -EvidenceLabel 'editor-authoring-wrong-save-project' `
        -DiagnosticText (
            "editor_project_save result=started project=C%3A%2Fwrong%2Fproject pre_save_dirty=true pre_save_dirty_generation=2 save_token_generation=2`n" +
            "editor_project_save result=completed project=C%3A%2Fwrong%2Fproject pre_save_dirty_generation=2 save_token_generation=2 persisted_generation=2 save_mark=Marked`n"
        )
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = $authoringProductRuns
        authoring_automation = $wrongProjectSaveAuthoring
        reopen_automation = $reopenAutomationFixture
    })
    $wrongProjectSaveRejected = $false
    try {
        & $driver -StagingRoot $stagingRoot -EvidenceRoot (Join-Path $fixtureRoot 'wrong-save-project') -ExpectedSourceFingerprint 'fixture-source-fingerprint' -RequireAuthoringAutomation -RequireReopenAutomation | Out-Null
    }
    catch {
        $wrongProjectSaveRejected = $_.Exception.Message -match 'project.*differs from staged project'
    }
    Assert-True $wrongProjectSaveRejected 'Acceptance did not bind project-save diagnostics to the staged project path.'

    $reversedProjectSaveAuthoring = New-FixtureAutomationProcessEvidence `
        -Report (Get-Content -LiteralPath (Join-Path $stagingRoot 'logs\editor-authoring.stdout.log') -Raw | ConvertFrom-FixtureJson) `
        -RequestPath $authoringRequestPath `
        -RequestRelativePath 'authoring/automation.json' `
        -StagingRoot $stagingRoot `
        -EvidenceLabel 'editor-authoring-reversed-save' `
        -DiagnosticText (
            "editor_project_save result=completed project=$encodedStagedProjectPath pre_save_dirty_generation=2 save_token_generation=2 persisted_generation=2 save_mark=Marked`n" +
            "editor_project_save result=started project=$encodedStagedProjectPath pre_save_dirty=true pre_save_dirty_generation=2 save_token_generation=2`n"
        )
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = $authoringProductRuns
        authoring_automation = $reversedProjectSaveAuthoring
        reopen_automation = $reopenAutomationFixture
    })
    $reversedProjectSaveRejected = $false
    try {
        & $driver -StagingRoot $stagingRoot -EvidenceRoot (Join-Path $fixtureRoot 'reversed-save-lifecycle') -ExpectedSourceFingerprint 'fixture-source-fingerprint' -RequireAuthoringAutomation -RequireReopenAutomation | Out-Null
    }
    catch {
        $reversedProjectSaveRejected = $_.Exception.Message -match 'completed.*must follow.*started'
    }
    Assert-True $reversedProjectSaveRejected 'Acceptance allowed a completed project-save diagnostic to precede its started diagnostic.'

    $saveGenerationDriftAuthoring = New-FixtureAutomationProcessEvidence `
        -Report (Get-Content -LiteralPath (Join-Path $stagingRoot 'logs\editor-authoring.stdout.log') -Raw | ConvertFrom-FixtureJson) `
        -RequestPath $authoringRequestPath `
        -RequestRelativePath 'authoring/automation.json' `
        -StagingRoot $stagingRoot `
        -EvidenceLabel 'editor-authoring-save-generation-drift' `
        -DiagnosticText (
            "editor_project_save result=started project=$encodedStagedProjectPath pre_save_dirty=true pre_save_dirty_generation=2 save_token_generation=2`n" +
            "editor_project_save result=completed project=$encodedStagedProjectPath pre_save_dirty_generation=2 save_token_generation=2 persisted_generation=3 save_mark=Marked`n"
        )
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = $authoringProductRuns
        authoring_automation = $saveGenerationDriftAuthoring
        reopen_automation = $reopenAutomationFixture
    })
    $saveGenerationDriftRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'authoring-save-generation-drift') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -RequireAuthoringAutomation `
            -RequireReopenAutomation | Out-Null
    }
    catch {
        $saveGenerationDriftRejected = $_.Exception.Message -match 'persisted_generation.*save_generation'
    }
    Assert-True $saveGenerationDriftRejected 'Acceptance did not reject persisted generation drift from the SaveProject event result.'
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = $authoringProductRuns
        baseline_automation = $baselineAutomationFixture
        authoring_automation = $authoringAutomationFixture
        reopen_automation = $reopenAutomationFixture
    })

    $f5CreationDiagnosticPath = $creationDiagnosticPath
    $encodedF5ProjectRoot = [Uri]::EscapeDataString((Join-Path $stagingRoot 'project'))
    [IO.File]::WriteAllText(
        $f5CreationDiagnosticPath,
        "editor_first_frame_presented`neditor_process_teardown_complete`neditor_product_frame_capture_written`n" +
        "editor_product_frame_diagnostics project_path=$encodedF5ProjectRoot selected_node_id=3 selected_node_name=Cube inspector_translation_x=0 inspector_translation_y=0 inspector_translation_z=0 inspector_scale_x=1.00 inspector_scale_y=1.00 inspector_scale_z=1.00`n" +
        "editor_project_open result=completed project_root=$encodedF5ProjectRoot manifest_identity=Fixture%20Project%40v1 scene_uri=res%3A%2F%2Fscenes%2Fmain.scene.toml registry_asset_count=4 registry_ready_asset_count=4 registry_failed_asset_count=0 registry_diagnostic_count=0 project_generation=1 project_generation_publish_epoch=1 catalog_asset_count=4 settings_source=persisted-v1`n",
        [Text.UTF8Encoding]::new($false)
    )
    $f5ProjectCreation = $projectCreationFixture | ConvertTo-Json -Depth 12 | ConvertFrom-FixtureJson
    $f5ProjectCreation.diagnostic_logs = @(Get-FixtureFileEvidence -Path $f5CreationDiagnosticPath -RelativePath 'logs/editor-create.diagnostics/fixture.log')
    $f5ProjectCreation.project_open.project_root = 'project'
    $f5ProjectCreation.editor_product_diagnostics.project_path = 'project'
    $afterReopenCapturePath = Join-Path $capturesRoot 'editor-after-reopen.png'
    Write-FixtureVisiblePng -Path $afterReopenCapturePath -AfterAuthoring
    $f5ProductRuns = Copy-FixtureProductRuns -Runs $productEvidenceRuns
    $f5ReopenedEditor = @($f5ProductRuns | Where-Object { $_.product -eq 'editor' -and $_.attempt -eq 1 })[0]
    $f5ReopenedEditor | Add-Member -NotePropertyName 'editor_window_capture' -NotePropertyValue ([ordered]@{
        path = 'captures/editor-after-reopen.png'
            sha256 = Get-FixtureFileSha256 -Path $afterReopenCapturePath
        size_bytes = (Get-Item -LiteralPath $afterReopenCapturePath).Length
        pixel_sha256 = Get-FixturePngPixelSha256 -Path $afterReopenCapturePath
        width = 16
        height = 16
        non_background_pixels = 128
        non_transparent_pixels = 256
    })
    $f5ReopenedEditor | Add-Member -NotePropertyName 'editor_product_diagnostics' -NotePropertyValue ([ordered]@{
        project_path = 'project'
        selected_node_id = 3
        selected_node_name = 'Cube'
        inspector_translation_x = '42'
        inspector_translation_y = '0'
        inspector_translation_z = '0'
        inspector_scale_x = '1.25'
        inspector_scale_y = '1.00'
        inspector_scale_z = '1.00'
    })
    $f5EditorDiagnosticsRoot = Join-Path $logsRoot 'editor-1.diagnostics'
    $f5EditorDiagnosticPath = Join-Path $f5EditorDiagnosticsRoot 'fixture.log'
    New-Item -ItemType Directory -Force -Path $f5EditorDiagnosticsRoot | Out-Null
    [IO.File]::WriteAllText(
        $f5EditorDiagnosticPath,
        "editor_first_frame_presented`neditor_process_teardown_complete`neditor_product_frame_capture_written`n" +
        "editor_product_frame_diagnostics project_path=$encodedF5ProjectRoot selected_node_id=3 selected_node_name=Cube inspector_translation_x=42 inspector_translation_y=0 inspector_translation_z=0 inspector_scale_x=1.25 inspector_scale_y=1.00 inspector_scale_z=1.00`n",
        [Text.UTF8Encoding]::new($false)
    )
    $f5ReopenedEditor.diagnostic_logs = @(
        Get-FixtureFileEvidence -Path $f5EditorDiagnosticPath -RelativePath 'logs/editor-1.diagnostics/fixture.log'
    )
    $f5AfterAuthoringRuntime = Copy-FixtureProductRuns -Runs @($f5ProductRuns | Where-Object { $_.product -eq 'runtime' } | Select-Object -Last 1)
    $f5AfterAuthoringRuntime.attempt = 3
    $f5AfterAuthoringRuntime.runtime_product_diagnostics.frame_index = 3
    $f5RuntimePrefix = 'runtime-3'
    $f5RuntimeStdoutPath = Join-Path $logsRoot "$f5RuntimePrefix.stdout.log"
    $f5RuntimeStderrPath = Join-Path $logsRoot "$f5RuntimePrefix.stderr.log"
    $f5RuntimeDiagnosticsRoot = Join-Path $logsRoot "$f5RuntimePrefix.diagnostics"
    $f5RuntimeDiagnosticPath = Join-Path $f5RuntimeDiagnosticsRoot 'fixture.log'
    $f5RuntimeCapturePath = Join-Path $capturesRoot "$f5RuntimePrefix.png"
    [IO.File]::WriteAllText($f5RuntimeStdoutPath, "$f5RuntimePrefix stdout`n", [Text.UTF8Encoding]::new($false))
    [IO.File]::WriteAllText($f5RuntimeStderrPath, "$f5RuntimePrefix stderr`n", [Text.UTF8Encoding]::new($false))
    New-Item -ItemType Directory -Force -Path $f5RuntimeDiagnosticsRoot | Out-Null
    [IO.File]::WriteAllText(
        $f5RuntimeDiagnosticPath,
        (New-FixtureRuntimeProductDiagnosticText -Diagnostics $f5AfterAuthoringRuntime.runtime_product_diagnostics),
        [Text.UTF8Encoding]::new($false)
    )
    Write-FixtureVisiblePng -Path $f5RuntimeCapturePath -AfterAuthoring
    $f5AfterAuthoringRuntime.stdout = Get-FixtureFileEvidence -Path $f5RuntimeStdoutPath -RelativePath "logs/$f5RuntimePrefix.stdout.log"
    $f5AfterAuthoringRuntime.stderr = Get-FixtureFileEvidence -Path $f5RuntimeStderrPath -RelativePath "logs/$f5RuntimePrefix.stderr.log"
    $f5AfterAuthoringRuntime.diagnostic_logs = @(
        Get-FixtureFileEvidence -Path $f5RuntimeDiagnosticPath -RelativePath "logs/$f5RuntimePrefix.diagnostics/fixture.log"
    )
    $f5AfterAuthoringRuntime.frame_capture = [ordered]@{
        path = "captures/$f5RuntimePrefix.png"
            sha256 = Get-FixtureFileSha256 -Path $f5RuntimeCapturePath
        size_bytes = (Get-Item -LiteralPath $f5RuntimeCapturePath).Length
        pixel_sha256 = Get-FixturePngPixelSha256 -Path $f5RuntimeCapturePath
        width = 16
        height = 16
        non_background_pixels = 128
        non_transparent_pixels = 256
    }
    $f5ProductRuns += $f5AfterAuthoringRuntime
    foreach ($f5ProductRun in $f5ProductRuns) {
        $timingOffset = switch ("$($f5ProductRun.product)-$($f5ProductRun.attempt)") {
            'runtime-1' { 2 }
            'runtime-2' { 3 }
            'editor-1' { 6 }
            'editor-2' { 8 }
            'runtime-3' { 9 }
            default { throw "Unexpected F5 fixture product process '$($f5ProductRun.product)-$($f5ProductRun.attempt)'." }
        }
        $null = Add-FixtureProcessTiming -Evidence $f5ProductRun -OffsetSeconds $timingOffset
    }
    $f5JournalAdditionalProcesses = @(
            [pscustomobject]@{ phase = 'editor-create'; evidence = $f5ProjectCreation }
            [pscustomobject]@{ phase = 'editor-baseline'; evidence = $baselineAutomationFixture }
            [pscustomobject]@{ phase = 'editor-authoring'; evidence = $authoringAutomationFixture }
            [pscustomobject]@{ phase = 'editor-reopen-1'; evidence = $reopenAutomationFixture[0] }
            [pscustomobject]@{ phase = 'editor-reopen-2'; evidence = $reopenAutomationFixture[1] }
    )
    Write-FixtureProcessJournal `
        -Runs $f5ProductRuns `
        -Path $processJournalPath `
        -AdditionalProcesses $f5JournalAdditionalProcesses
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        project_creation = $f5ProjectCreation
        products = $f5ProductRuns
        baseline_automation = $baselineAutomationFixture
        authoring_automation = $authoringAutomationFixture
        reopen_automation = $reopenAutomationFixture
    })
    Import-Module $buildSummaryEvidenceModule -Force -ErrorAction Stop
    $profileContractSummaryPath = Join-Path $fixtureRoot 'profile-contract-summary.json'
    $workspaceSummaryPath = Join-Path $fixtureRoot 'workspace-summary.json'
    Write-FixtureBuildSummary -Path $profileContractSummaryPath -SummaryKind 'profile-contract'
    Write-FixtureBuildSummary -Path $workspaceSummaryPath -SummaryKind 'workspace'
    $canonicalF5StartupSummaryJson = Get-Content -LiteralPath (Join-Path $stagingRoot 'startup-summary.json') -Raw -Encoding UTF8

    $summaryLogReparseRoot = Join-Path $fixtureRoot 'summary-log-reparse'
    $summaryLogReparsePath = Join-Path $summaryLogReparseRoot 'profile-contract-summary.json'
    $summaryLogReparseOutside = Join-Path $fixtureRoot 'summary-log-reparse-outside'
    New-Item -ItemType Directory -Force -Path $summaryLogReparseRoot, $summaryLogReparseOutside | Out-Null
    Write-FixtureBuildSummary -Path $summaryLogReparsePath -SummaryKind 'profile-contract'
    $summaryLogReparseDocument = ConvertFrom-FixtureJson -Json (Get-Content -LiteralPath $summaryLogReparsePath -Raw -Encoding UTF8)
    $summaryLogDirectory = Join-Path $summaryLogReparseRoot 'logs'
    foreach ($gate in @($summaryLogReparseDocument.gates)) {
        $fileName = Split-Path -Leaf ([string]$gate.evidence.path)
        Copy-Item -LiteralPath (Join-Path $summaryLogDirectory $fileName) -Destination (Join-Path $summaryLogReparseOutside $fileName)
    }
    Remove-Item -LiteralPath $summaryLogDirectory -Recurse -Force
    New-Item -ItemType Junction -Path $summaryLogDirectory -Target $summaryLogReparseOutside -ErrorAction Stop | Out-Null
    $summaryLogReparseRejected = $false
    try {
        Assert-MvpBuildSummaryEvidence `
            -Path $summaryLogReparsePath `
            -ExpectedKind 'profile-contract' `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' | Out-Null
    }
    catch {
        $summaryLogReparseRejected = $_.Exception.Message -match 'resolves outside build summary directory'
    }
    Assert-True $summaryLogReparseRejected 'F5 build summary evidence accepted gate logs redirected outside its summary directory.'

    $missingBuildSummaryEvidenceRoot = Join-Path $fixtureRoot 'f5-missing-build-summaries'
    $missingBuildSummariesRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot $missingBuildSummaryEvidenceRoot `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -RequireF5Evidence | Out-Null
    }
    catch {
        $missingBuildSummariesRejected = $_.Exception.Message -match 'ProfileContractSummaryPath|WorkspaceSummaryPath|build summar'
    }
    Assert-True $missingBuildSummariesRejected 'F5 acceptance did not reject missing explicit build summary inputs.'
    Assert-True (-not (Test-Path -LiteralPath $missingBuildSummaryEvidenceRoot)) 'Missing F5 build summaries left a partial evidence root.'

    $mismatchedBuildSummaryEvidenceRoot = Join-Path $fixtureRoot 'f5-mismatched-build-summary'
    Write-FixtureBuildSummary `
        -Path $profileContractSummaryPath `
        -SummaryKind 'profile-contract' `
        -SourceFingerprint 'other-source-fingerprint'
    try {
        $mismatchedBuildSummaryRejected = $false
        try {
            & $driver `
                -StagingRoot $stagingRoot `
                -EvidenceRoot $mismatchedBuildSummaryEvidenceRoot `
                -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
                -ProfileContractSummaryPath $profileContractSummaryPath `
                -WorkspaceSummaryPath $workspaceSummaryPath `
                -RequireF5Evidence | Out-Null
        }
        catch {
            $mismatchedBuildSummaryRejected = $_.Exception.Message -match 'profile-contract.*source_fingerprint.*differs'
        }
        Assert-True $mismatchedBuildSummaryRejected 'F5 acceptance did not reject a build summary from another source fingerprint.'
        Assert-True (-not (Test-Path -LiteralPath $mismatchedBuildSummaryEvidenceRoot)) 'Mismatched F5 build summary left a partial evidence root.'
    }
    finally {
        Write-FixtureBuildSummary -Path $profileContractSummaryPath -SummaryKind 'profile-contract'
    }

    $missingRequiredGateEvidenceRoot = Join-Path $fixtureRoot 'f5-missing-required-build-gate'
    $missingRequiredGateSummary = ConvertFrom-FixtureJson -Json (Get-Content -LiteralPath $profileContractSummaryPath -Raw -Encoding UTF8)
    $missingRequiredGateSummary.gates = @($missingRequiredGateSummary.gates | Select-Object -Skip 1)
    Write-FixtureJson -Path $profileContractSummaryPath -Value $missingRequiredGateSummary
    try {
        $missingRequiredGateRejected = $false
        try {
            & $driver `
                -StagingRoot $stagingRoot `
                -EvidenceRoot $missingRequiredGateEvidenceRoot `
                -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
                -ProfileContractSummaryPath $profileContractSummaryPath `
                -WorkspaceSummaryPath $workspaceSummaryPath `
                -RequireF5Evidence | Out-Null
        }
        catch {
            $missingRequiredGateRejected = $_.Exception.Message -match 'profile-contract.*required gate set'
        }
        Assert-True $missingRequiredGateRejected 'F5 acceptance did not reject a profile summary missing a canonical gate.'
        Assert-True (-not (Test-Path -LiteralPath $missingRequiredGateEvidenceRoot)) 'Missing canonical build gate left a partial evidence root.'
    }
    finally {
        Write-FixtureBuildSummary -Path $profileContractSummaryPath -SummaryKind 'profile-contract'
    }

    $buildGateHashDriftEvidenceRoot = Join-Path $fixtureRoot 'f5-build-gate-hash-drift'
    $buildGateHashDriftSummary = ConvertFrom-FixtureJson -Json (Get-Content -LiteralPath $workspaceSummaryPath -Raw -Encoding UTF8)
    $buildGateHashDriftSummary.gates[0].evidence.sha256 = ('0' * 64)
    Write-FixtureJson -Path $workspaceSummaryPath -Value $buildGateHashDriftSummary
    try {
        $buildGateHashDriftRejected = $false
        try {
            & $driver `
                -StagingRoot $stagingRoot `
                -EvidenceRoot $buildGateHashDriftEvidenceRoot `
                -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
                -ProfileContractSummaryPath $profileContractSummaryPath `
                -WorkspaceSummaryPath $workspaceSummaryPath `
                -RequireF5Evidence | Out-Null
        }
        catch {
            $buildGateHashDriftRejected = $_.Exception.Message -match 'workspace.*evidence hash mismatch'
        }
        Assert-True $buildGateHashDriftRejected 'F5 acceptance did not reject a workspace gate detached from its build log.'
        Assert-True (-not (Test-Path -LiteralPath $buildGateHashDriftEvidenceRoot)) 'Build gate hash drift left a partial evidence root.'
    }
    finally {
        Write-FixtureBuildSummary -Path $workspaceSummaryPath -SummaryKind 'workspace'
    }

    $malformedBuildSummaryEvidenceRoot = Join-Path $fixtureRoot 'f5-malformed-build-summary'
    [IO.File]::WriteAllText($workspaceSummaryPath, '{not-json', [Text.UTF8Encoding]::new($false))
    try {
        $malformedBuildSummaryRejected = $false
        $malformedBuildSummaryFailure = $null
        try {
            & $driver `
                -StagingRoot $stagingRoot `
                -EvidenceRoot $malformedBuildSummaryEvidenceRoot `
                -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
                -ProfileContractSummaryPath $profileContractSummaryPath `
                -WorkspaceSummaryPath $workspaceSummaryPath `
                -RequireF5Evidence | Out-Null
        }
        catch {
            $malformedBuildSummaryFailure = $_.Exception.Message
            $malformedBuildSummaryRejected = $_.Exception.Message -match 'workspace build summary[\s\S]*not valid .*JSON'
        }
        Assert-True $malformedBuildSummaryRejected "F5 acceptance did not reject a malformed workspace build summary; failure=$malformedBuildSummaryFailure"
        Assert-True (-not (Test-Path -LiteralPath $malformedBuildSummaryEvidenceRoot)) 'Malformed F5 build summary left a partial evidence root.'
    }
    finally {
        Write-FixtureBuildSummary -Path $workspaceSummaryPath -SummaryKind 'workspace'
    }

    $missingProcessTimingEvidenceRoot = Join-Path $fixtureRoot 'f5-missing-process-timing'
    $missingProcessTimingSummary = ConvertFrom-FixtureJson -Json $canonicalF5StartupSummaryJson
    [void]$missingProcessTimingSummary.products[0].PSObject.Properties.Remove('ended_at_utc')
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value $missingProcessTimingSummary
    try {
        $missingProcessTimingRejected = $false
        try {
            & $driver `
                -StagingRoot $stagingRoot `
                -EvidenceRoot $missingProcessTimingEvidenceRoot `
                -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
                -ProfileContractSummaryPath $profileContractSummaryPath `
                -WorkspaceSummaryPath $workspaceSummaryPath `
                -RequireF5Evidence | Out-Null
        }
        catch {
            $missingProcessTimingRejected = $_.Exception.Message -match "Product 'runtime' attempt 1.*ended_at_utc"
        }
        Assert-True $missingProcessTimingRejected 'F5 acceptance did not reject a product process without absolute end time.'
        Assert-True (-not (Test-Path -LiteralPath $missingProcessTimingEvidenceRoot)) 'Missing F5 process timing left a partial evidence root.'
    }
    finally {
        [IO.File]::WriteAllText(
            (Join-Path $stagingRoot 'startup-summary.json'),
            $canonicalF5StartupSummaryJson,
            [Text.UTF8Encoding]::new($false)
        )
    }

    $malformedProcessTimingEvidenceRoot = Join-Path $fixtureRoot 'f5-malformed-process-timing'
    $malformedProcessTimingSummary = ConvertFrom-FixtureJson -Json $canonicalF5StartupSummaryJson
    $malformedProcessTimingSummary.products[0].started_at_utc = 'not-a-timestamp'
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value $malformedProcessTimingSummary
    try {
        $malformedProcessTimingRejected = $false
        try {
            & $driver `
                -StagingRoot $stagingRoot `
                -EvidenceRoot $malformedProcessTimingEvidenceRoot `
                -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
                -ProfileContractSummaryPath $profileContractSummaryPath `
                -WorkspaceSummaryPath $workspaceSummaryPath `
                -RequireF5Evidence | Out-Null
        }
        catch {
            $malformedProcessTimingRejected = $_.Exception.Message -match "started_at_utc.*process journal|malformed 'started_at_utc' timestamp"
        }
        Assert-True $malformedProcessTimingRejected 'F5 acceptance did not reject a malformed product process timestamp.'
        Assert-True (-not (Test-Path -LiteralPath $malformedProcessTimingEvidenceRoot)) 'Malformed F5 process timing left a partial evidence root.'
    }
    finally {
        [IO.File]::WriteAllText(
            (Join-Path $stagingRoot 'startup-summary.json'),
            $canonicalF5StartupSummaryJson,
            [Text.UTF8Encoding]::new($false)
        )
    }

    $overlappingProcessTimelineEvidenceRoot = Join-Path $fixtureRoot 'f5-overlapping-process-timeline'
    $overlappingProcessTimelineSummary = ConvertFrom-FixtureJson -Json $canonicalF5StartupSummaryJson
    $runtimeTwoTiming = @($overlappingProcessTimelineSummary.products | Where-Object { $_.product -eq 'runtime' -and $_.attempt -eq 2 })[0]
    $overlappingProcessTimelineSummary.authoring_automation.started_at_utc = $runtimeTwoTiming.started_at_utc
    $overlappingProcessTimelineSummary.authoring_automation.ended_at_utc = $runtimeTwoTiming.ended_at_utc
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value $overlappingProcessTimelineSummary
    try {
        $overlappingProcessTimelineRejected = $false
        try {
            & $driver `
                -StagingRoot $stagingRoot `
                -EvidenceRoot $overlappingProcessTimelineEvidenceRoot `
                -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
                -ProfileContractSummaryPath $profileContractSummaryPath `
                -WorkspaceSummaryPath $workspaceSummaryPath `
                -RequireF5Evidence | Out-Null
        }
        catch {
            $overlappingProcessTimelineRejected = $_.Exception.Message -match 'F5 authoring automation process.*overlaps or precedes|Authoring automation report.*process journal'
        }
        Assert-True $overlappingProcessTimelineRejected 'F5 acceptance did not reject an authoring process overlapping the second pre-edit runtime.'
        Assert-True (-not (Test-Path -LiteralPath $overlappingProcessTimelineEvidenceRoot)) 'Overlapping F5 process timeline left a partial evidence root.'
    }
    finally {
        [IO.File]::WriteAllText(
            (Join-Path $stagingRoot 'startup-summary.json'),
            $canonicalF5StartupSummaryJson,
            [Text.UTF8Encoding]::new($false)
        )
    }

    $f5EvidenceRoot = Join-Path $fixtureRoot 'f5-editor-window-evidence'
    $f5Evidence = @(
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot $f5EvidenceRoot `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -ProfileContractSummaryPath $profileContractSummaryPath `
            -WorkspaceSummaryPath $workspaceSummaryPath `
            -RequireF5Evidence `
            -Json
    ) | ConvertFrom-FixtureJson
    Assert-True ($f5Evidence.project_creation.editor_window_capture.path -eq 'captures/editor-before-edit.png') 'F5 acceptance output lost the created-project editor window evidence.'
    Assert-True ($f5Evidence.project_identity.project_root -eq 'project') 'F5 evidence manifest lost the canonical project root.'
    Assert-True ($f5Evidence.project_identity.project_identity -eq 'fixture-project') 'F5 evidence manifest lost the runtime project identity.'
    Assert-True ($f5Evidence.project_identity.manifest_identity -eq 'Fixture Project@v1') 'F5 evidence manifest lost the editor manifest identity.'
    Assert-True ($f5Evidence.project_identity.scene_uri -eq 'res://scenes/main.scene.toml') 'F5 evidence manifest lost the canonical scene URI.'
    Assert-True ($f5Evidence.project_identity.model_resource_id -eq 'fixture-cube-model-resource') 'F5 evidence manifest lost the selected Cube model reference.'
    Assert-True ($f5Evidence.project_identity.material_resource_id -eq 'fixture-default-material-resource') 'F5 evidence manifest lost the selected Cube material reference.'
    Assert-True ($f5Evidence.baseline_automation.snapshot.inspector_translation[0] -eq '0') 'F5 evidence manifest lost the pre-authoring Cube baseline.'
    Assert-True ($f5Evidence.baseline_automation.snapshot.inspector_scale[0] -eq '1.00') 'F5 evidence manifest lost the pre-authoring Cube scale.'
    $canonicalF5StagingManifestJson = Get-Content -LiteralPath $stagingManifestPath -Raw -Encoding UTF8
    $driftedProductInputManifest = ConvertFrom-FixtureJson -Json $canonicalF5StagingManifestJson
    $driftedProductInputManifest.product_input_manifest.artifacts[0].sha256 = ('0' * 64)
    Write-FixtureJson -Path $stagingManifestPath -Value $driftedProductInputManifest
    $productInputDriftRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'f5-product-input-manifest-drift') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -ProfileContractSummaryPath $profileContractSummaryPath `
            -WorkspaceSummaryPath $workspaceSummaryPath `
            -RequireF5Evidence | Out-Null
    }
    catch {
        $productInputDriftRejected = $_.Exception.Message -match 'Product input manifest artifact.*hash differs'
    }
    finally {
        [IO.File]::WriteAllText($stagingManifestPath, $canonicalF5StagingManifestJson, [Text.UTF8Encoding]::new($false))
    }
    Assert-True $productInputDriftRejected 'F5 acceptance accepted product inputs detached from their staged file hashes.'
    $driftedProductInputManifestSummary = ConvertFrom-FixtureJson -Json $canonicalF5StagingManifestJson
    $driftedProductInputManifestSummary.product_input_manifest.sha256 = ('0' * 64)
    Write-FixtureJson -Path $stagingManifestPath -Value $driftedProductInputManifestSummary
    $productInputManifestSummaryDriftRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'f5-product-input-manifest-summary-drift') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -ProfileContractSummaryPath $profileContractSummaryPath `
            -WorkspaceSummaryPath $workspaceSummaryPath `
            -RequireF5Evidence | Out-Null
    }
    catch {
        $productInputManifestSummaryDriftRejected = $_.Exception.Message -match 'Product input manifest evidence hash differs from its staged manifest'
    }
    finally {
        [IO.File]::WriteAllText($stagingManifestPath, $canonicalF5StagingManifestJson, [Text.UTF8Encoding]::new($false))
    }
    Assert-True $productInputManifestSummaryDriftRejected 'F5 acceptance accepted an unbound product-input manifest digest.'
    $comparisonRoot = Join-Path $f5EvidenceRoot 'comparison'
    $persistedBefore = Get-Content -LiteralPath (Join-Path $comparisonRoot 'persisted-state-before.json') -Raw | ConvertFrom-FixtureJson
    $persistedAfter = Get-Content -LiteralPath (Join-Path $comparisonRoot 'persisted-state-after.json') -Raw | ConvertFrom-FixtureJson
    $reopenedState = Get-Content -LiteralPath (Join-Path $comparisonRoot 'reopened-state.json') -Raw | ConvertFrom-FixtureJson
    Assert-True ($persistedBefore.snapshot.inspector_translation[0] -eq '0') 'Persisted-state-before comparison lost the baseline transform.'
    Assert-True ($persistedAfter.snapshot.inspector_translation[0] -eq '42') 'Persisted-state-after comparison lost the authored transform.'
    Assert-True ($persistedBefore.snapshot.inspector_scale[0] -eq '1.00') 'Persisted-state-before comparison lost the baseline scale.'
    Assert-True ($persistedAfter.snapshot.inspector_scale[0] -eq '1.25') 'Persisted-state-after comparison lost the authored scale.'
    Assert-True ($persistedAfter.project_save_lifecycle.persisted_generation -eq 2) 'Persisted-state-after comparison lost the save lifecycle.'
    Assert-True (@($reopenedState.runs).Count -eq 2) 'Reopened-state comparison did not retain two independent process runs.'
    Assert-True (@($reopenedState.runs | Where-Object { $_.snapshot.inspector_translation[0] -eq '42' }).Count -eq 2) 'Reopened-state comparison does not prove the authored transform twice.'
    Assert-True (@($reopenedState.runs | Where-Object { $_.snapshot.inspector_scale[0] -eq '1.25' }).Count -eq 2) 'Reopened-state comparison does not prove the authored scale twice.'
    $f5PackagedManifestJson = Get-Content -LiteralPath $f5Evidence.manifest -Raw
    $f5PackagedManifest = $f5PackagedManifestJson | ConvertFrom-FixtureJson
    Assert-True ($f5PackagedManifest.schema_version -eq 2) 'F5 evidence manifest did not declare the build-summary/timing schema.'
    Assert-True ($f5PackagedManifest.product_input_manifest.source_fingerprint -eq 'fixture-source-fingerprint') 'F5 evidence manifest lost product-input source provenance.'
    Assert-True ($f5PackagedManifest.product_input_manifest.target_relative_path -eq 'build/mvp-product-inputs.json') 'F5 evidence manifest lost the staged product-input manifest path.'
    Assert-True ($f5PackagedManifest.product_input_manifest.sha256 -eq $stagedProductInputManifestEntry.sha256) 'F5 evidence manifest lost the staged product-input manifest digest.'
    Assert-True ($f5PackagedManifest.product_input_manifest.size_bytes -eq $stagedProductInputManifestEntry.size_bytes) 'F5 evidence manifest lost the staged product-input manifest byte count.'
    Assert-True (@($f5PackagedManifest.product_input_manifest.artifacts).Count -eq 4) 'F5 evidence manifest lost the four product-input artifact identities.'
    Assert-True ($f5PackagedManifest.process_execution_journal.path -eq 'logs/process-execution-journal.jsonl') 'F5 evidence manifest did not bind the consumed process execution journal.'
    Assert-True (Test-Path -LiteralPath (Join-Path $f5EvidenceRoot 'logs/process-execution-journal.jsonl') -PathType Leaf) 'F5 evidence package did not archive the consumed process execution journal.'
    Assert-True ($f5PackagedManifest.build_summaries.profile_contract.path -eq 'build/profile-contract-summary.json') 'F5 evidence manifest lost the canonical profile summary path.'
    Assert-True ($f5PackagedManifest.build_summaries.workspace.path -eq 'build/workspace-summary.json') 'F5 evidence manifest lost the canonical workspace summary path.'
    Assert-True (Test-Path -LiteralPath (Join-Path $f5EvidenceRoot 'build/profile-contract-summary.json') -PathType Leaf) 'F5 evidence package did not archive the profile summary.'
    Assert-True (Test-Path -LiteralPath (Join-Path $f5EvidenceRoot 'build/workspace-summary.json') -PathType Leaf) 'F5 evidence package did not archive the workspace summary.'
    $packagedProcesses = @($f5PackagedManifest.project_creation)
    $packagedProcesses += @($f5PackagedManifest.product_runs)
    $packagedProcesses += @($f5PackagedManifest.baseline_automation)
    $packagedProcesses += @($f5PackagedManifest.authoring_automation)
    $packagedProcesses += @($f5PackagedManifest.reopen_automation)
    foreach ($packagedProcess in $packagedProcesses) {
        Assert-True (-not [string]::IsNullOrWhiteSpace([string]$packagedProcess.started_at_utc)) 'F5 manifest dropped a validated process start time.'
        Assert-True (-not [string]::IsNullOrWhiteSpace([string]$packagedProcess.ended_at_utc)) 'F5 manifest dropped a validated process end time.'
        Assert-True ($packagedProcess.exit_code -eq 0) 'F5 manifest dropped a validated successful process exit code.'
    }
    $packagedTimestampTokens = [regex]::Matches(
        $f5PackagedManifestJson,
        '"(?:started_at_utc|ended_at_utc)"\s*:\s*"(?<timestamp>[^"]+)"'
    )
    Assert-True (
        $packagedTimestampTokens.Count -eq ($packagedProcesses.Count * 2)
    ) 'F5 manifest did not retain exactly one UTC start/end timestamp pair for every packaged process.'
    foreach ($timestampToken in $packagedTimestampTokens) {
        $timestamp = [string]$timestampToken.Groups['timestamp'].Value
        Assert-True (
            $timestamp -match '\+00:00$'
        ) "F5 manifest process timestamp '$timestamp' must be serialized in UTC."
    }
    foreach ($buildPath in @('build/profile-contract-summary.json', 'build/workspace-summary.json')) {
        $buildInventory = @($f5PackagedManifest.evidence_files | Where-Object { $_.path -eq $buildPath })
        Assert-True ($buildInventory.Count -eq 1) "F5 evidence manifest did not inventory '$buildPath'."
        $buildManifestEntry = if ($buildPath -match 'profile-contract') {
            $f5PackagedManifest.build_summaries.profile_contract
        }
        else {
            $f5PackagedManifest.build_summaries.workspace
        }
        Assert-True ($buildInventory[0].sha256 -eq $buildManifestEntry.sha256) "F5 evidence hash inventory differs from '$buildPath' metadata."
    }
    foreach ($gateLogPath in @(
        'build/logs/zircon-app-target-server.log',
        'build/logs/zircon-app-target-client-platform.log',
        'build/logs/zircon-app-target-editor-host.log',
        'build/logs/zircon-app-target-client-shader-pbr-viewer.log',
        'build/logs/zircon-runtime-target-client.log',
        'build/logs/zircon-runtime-target-editor-host.log',
        'build/logs/zircon-runtime-target-server.log',
        'build/logs/workspace-build.log',
        'build/logs/workspace-test.log'
    )) {
        Assert-True (Test-Path -LiteralPath (Join-Path $f5EvidenceRoot $gateLogPath) -PathType Leaf) "F5 evidence package did not archive '$gateLogPath'."
        Assert-True (@($f5PackagedManifest.evidence_files | Where-Object { $_.path -eq $gateLogPath }).Count -eq 1) "F5 manifest did not inventory '$gateLogPath'."
    }
    foreach ($comparisonPath in @(
        'comparison/persisted-state-before.json',
        'comparison/persisted-state-after.json',
        'comparison/reopened-state.json'
    )) {
        Assert-True (@($f5PackagedManifest.evidence_files | Where-Object { $_.path -eq $comparisonPath }).Count -eq 1) "F5 evidence manifest did not inventory '$comparisonPath'."
    }
    $f5EvidenceEditorRun = @(
        $f5Evidence.product_runs | Where-Object {
            $_.product -eq 'editor' -and
            $null -ne $_.PSObject.Properties['editor_window_capture'] -and
            $null -ne $_.editor_window_capture
        }
    )[0]
    Assert-True ($f5EvidenceEditorRun.editor_window_capture.path -eq 'captures/editor-after-reopen.png') 'F5 acceptance output lost the reopened editor window evidence.'

    Write-FixtureProcessJournal `
        -Runs $f5ProductRuns `
        -Path $processJournalPath `
        -ExitCodeOverrides @{ 'editor-create' = 1 } `
        -AdditionalProcesses $f5JournalAdditionalProcesses
    $f5CreationJournalRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'f5-project-creation-journal-mismatch') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -ProfileContractSummaryPath $profileContractSummaryPath `
            -WorkspaceSummaryPath $workspaceSummaryPath `
            -RequireF5Evidence | Out-Null
    }
    catch {
        $f5CreationJournalRejected = $_.Exception.Message -match 'Project creation.*process journal'
    }
    finally {
        Write-FixtureProcessJournal -Runs $f5ProductRuns -Path $processJournalPath -AdditionalProcesses $f5JournalAdditionalProcesses
    }
    Assert-True $f5CreationJournalRejected 'F5 acceptance accepted project-creation success detached from the process journal.'

    Write-FixtureProcessJournal `
        -Runs $f5ProductRuns `
        -Path $processJournalPath `
        -ExitCodeOverrides @{ 'editor-authoring' = 1 } `
        -AdditionalProcesses $f5JournalAdditionalProcesses
    $f5AuthoringJournalRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'f5-authoring-journal-mismatch') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -ProfileContractSummaryPath $profileContractSummaryPath `
            -WorkspaceSummaryPath $workspaceSummaryPath `
            -RequireF5Evidence | Out-Null
    }
    catch {
        $f5AuthoringJournalRejected = $_.Exception.Message -match 'Authoring automation report.*process journal'
    }
    finally {
        Write-FixtureProcessJournal -Runs $f5ProductRuns -Path $processJournalPath -AdditionalProcesses $f5JournalAdditionalProcesses
    }
    Assert-True $f5AuthoringJournalRejected 'F5 acceptance accepted authoring success detached from the process journal.'

    $unrelatedAuthoringDriftReport = Get-Content -LiteralPath (Join-Path $stagingRoot 'logs\editor-authoring.stdout.log') -Raw | ConvertFrom-FixtureJson
    $unrelatedAuthoringDriftReport.snapshot.scene_nodes[1].transform.scale[1] = 2.0
    $unrelatedAuthoringDrift = New-FixtureAutomationProcessEvidence `
        -Report $unrelatedAuthoringDriftReport `
        -RequestPath $authoringRequestPath `
        -RequestRelativePath 'authoring/automation.json' `
        -StagingRoot $stagingRoot `
        -EvidenceLabel 'editor-authoring-unrelated-scene-drift'
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        project_creation = $f5ProjectCreation
        products = $f5ProductRuns
        baseline_automation = $baselineAutomationFixture
        authoring_automation = $unrelatedAuthoringDrift
        reopen_automation = $reopenAutomationFixture
    })
    $unrelatedAuthoringDriftRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'f5-unrelated-authoring-scene-drift') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -ProfileContractSummaryPath $profileContractSummaryPath `
            -WorkspaceSummaryPath $workspaceSummaryPath `
            -RequireF5Evidence | Out-Null
    }
    catch {
        $unrelatedAuthoringDriftRejected = $_.Exception.Message -match 'scene_nodes differs from the baseline outside the requested Cube X translation'
    }
    Assert-True $unrelatedAuthoringDriftRejected 'F5 acceptance did not reject an unrelated Sun scale mutation introduced by authoring.'

    $f5IdenticalCaptures = Copy-FixtureProductRuns -Runs $f5ProductRuns
    $f5IdenticalEditor = @($f5IdenticalCaptures | Where-Object { $_.product -eq 'editor' -and $_.attempt -eq 1 })[0]
    Copy-Item -LiteralPath $creationCapturePath -Destination $afterReopenCapturePath -Force
    $f5IdenticalEditor.editor_window_capture.sha256 = Get-FixtureFileSha256 -Path $afterReopenCapturePath
    $f5IdenticalEditor.editor_window_capture.size_bytes = (Get-Item -LiteralPath $afterReopenCapturePath).Length
    $f5IdenticalEditor.editor_window_capture.pixel_sha256 = Get-FixturePngPixelSha256 -Path $afterReopenCapturePath
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        project_creation = $f5ProjectCreation
        products = $f5IdenticalCaptures
        baseline_automation = $baselineAutomationFixture
        authoring_automation = $authoringAutomationFixture
        reopen_automation = $reopenAutomationFixture
    })
    $f5IdenticalCapturesRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'f5-identical-editor-captures') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -ProfileContractSummaryPath $profileContractSummaryPath `
            -WorkspaceSummaryPath $workspaceSummaryPath `
            -RequireF5Evidence | Out-Null
    }
    catch {
        $f5IdenticalCapturesRejected = $_.Exception.Message -match 'decoded pixels|pixel_sha256'
    }
    finally {
        Write-FixtureVisiblePng -Path $afterReopenCapturePath -AfterAuthoring
    }
    Assert-True $f5IdenticalCapturesRejected 'F5 acceptance did not reject identical before/after editor window captures.'

    $f5MetadataOnlyCaptures = Copy-FixtureProductRuns -Runs $f5ProductRuns
    $f5MetadataOnlyEditor = @($f5MetadataOnlyCaptures | Where-Object { $_.product -eq 'editor' -and $_.attempt -eq 1 })[0]
    Copy-Item -LiteralPath $creationCapturePath -Destination $afterReopenCapturePath -Force
    $metadataOnlyStream = [IO.File]::Open($afterReopenCapturePath, [IO.FileMode]::Append, [IO.FileAccess]::Write)
    try {
        $metadataOnlyStream.WriteByte(0)
    }
    finally {
        $metadataOnlyStream.Dispose()
    }
    $f5MetadataOnlyEditor.editor_window_capture.sha256 = Get-FixtureFileSha256 -Path $afterReopenCapturePath
    $f5MetadataOnlyEditor.editor_window_capture.size_bytes = (Get-Item -LiteralPath $afterReopenCapturePath).Length
    $f5MetadataOnlyEditor.editor_window_capture.pixel_sha256 = Get-FixturePngPixelSha256 -Path $afterReopenCapturePath
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        project_creation = $f5ProjectCreation
        products = $f5MetadataOnlyCaptures
        baseline_automation = $baselineAutomationFixture
        authoring_automation = $authoringAutomationFixture
        reopen_automation = $reopenAutomationFixture
    })
    $f5MetadataOnlyCapturesRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'f5-metadata-only-editor-captures') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -ProfileContractSummaryPath $profileContractSummaryPath `
            -WorkspaceSummaryPath $workspaceSummaryPath `
            -RequireF5Evidence | Out-Null
    }
    catch {
        $f5MetadataOnlyCapturesRejected = $_.Exception.Message -match 'decoded pixels'
    }
    finally {
        Write-FixtureVisiblePng -Path $afterReopenCapturePath -AfterAuthoring
    }
    Assert-True $f5MetadataOnlyCapturesRejected 'F5 acceptance did not reject PNG files with distinct bytes but identical decoded pixels.'

    $f5ReferenceDrift = Copy-FixtureProductRuns -Runs $reopenAutomationFixture
    $f5ReferenceDrift[1].selected_material_resource_id = 'replacement-material-resource'
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        project_creation = $f5ProjectCreation
        products = $f5ProductRuns
        baseline_automation = $baselineAutomationFixture
        authoring_automation = $authoringAutomationFixture
        reopen_automation = $f5ReferenceDrift
    })
    $f5ReferenceDriftRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'f5-project-reference-drift') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -ProfileContractSummaryPath $profileContractSummaryPath `
            -WorkspaceSummaryPath $workspaceSummaryPath `
            -RequireF5Evidence | Out-Null
    }
    catch {
        $f5ReferenceDriftRejected = $_.Exception.Message -match 'selected_material_resource_id.*(captured stdout report|pre-authoring baseline)'
    }
    Assert-True $f5ReferenceDriftRejected 'F5 acceptance did not reject a replaced Cube material reference after reopen.'

    $f5DriftedAuthoringReport = Get-Content -LiteralPath (Join-Path $stagingRoot 'logs\editor-authoring.stdout.log') -Raw | ConvertFrom-FixtureJson
    $f5DriftedAuthoringReport.selected_model_resource_id = 'replacement-model-resource'
    $f5DriftedAuthoringReport.selected_material_resource_id = 'replacement-material-resource'
    $f5DriftedAuthoring = New-FixtureAutomationProcessEvidence `
        -Report $f5DriftedAuthoringReport `
        -RequestPath $authoringRequestPath `
        -RequestRelativePath 'authoring/automation.json' `
        -StagingRoot $stagingRoot `
        -EvidenceLabel 'editor-authoring-reference-drift'
    $f5DriftedReopens = @()
    for ($index = 1; $index -le 2; $index++) {
        $report = Get-Content -LiteralPath (Join-Path $stagingRoot "logs\editor-reopen-$index.stdout.log") -Raw | ConvertFrom-FixtureJson
        $report.selected_model_resource_id = 'replacement-model-resource'
        $report.selected_material_resource_id = 'replacement-material-resource'
        $f5DriftedReopens += New-FixtureAutomationProcessEvidence `
            -Report $report `
            -RequestPath $reopenRequestPath `
            -RequestRelativePath 'reopen/automation.json' `
            -StagingRoot $stagingRoot `
            -EvidenceLabel "editor-reopen-reference-drift-$index"
    }
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        project_creation = $f5ProjectCreation
        products = $f5ProductRuns
        baseline_automation = $baselineAutomationFixture
        authoring_automation = $f5DriftedAuthoring
        reopen_automation = $f5DriftedReopens
    })
    $f5FirstAuthoringReferenceDriftRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'f5-first-authoring-reference-drift') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -ProfileContractSummaryPath $profileContractSummaryPath `
            -WorkspaceSummaryPath $workspaceSummaryPath `
            -RequireF5Evidence | Out-Null
    }
    catch {
        $f5FirstAuthoringReferenceDriftRejected = $_.Exception.Message -match 'selected_(model|material)_resource_id differs from the pre-authoring baseline'
    }
    Assert-True $f5FirstAuthoringReferenceDriftRejected 'F5 acceptance did not reject model/material replacement first introduced by authoring.'
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        project_creation = $f5ProjectCreation
        products = $f5ProductRuns
        baseline_automation = $baselineAutomationFixture
        authoring_automation = $authoringAutomationFixture
        reopen_automation = $reopenAutomationFixture
    })

    $f5MissingBeforeEditCapture = $f5ProjectCreation | ConvertTo-Json -Depth 12 | ConvertFrom-FixtureJson
    $f5MissingBeforeEditCapture.PSObject.Properties.Remove('editor_window_capture')
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        project_creation = $f5MissingBeforeEditCapture
        products = $f5ProductRuns
        baseline_automation = $baselineAutomationFixture
        authoring_automation = $authoringAutomationFixture
        reopen_automation = $reopenAutomationFixture
    })
    $f5MissingBeforeEditCaptureRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'f5-missing-before-edit-capture') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -ProfileContractSummaryPath $profileContractSummaryPath `
            -WorkspaceSummaryPath $workspaceSummaryPath `
            -RequireF5Evidence | Out-Null
    }
    catch {
        $f5MissingBeforeEditCaptureRejected = $_.Exception.Message -match 'editor_window_capture'
    }
    Assert-True $f5MissingBeforeEditCaptureRejected 'F5 acceptance did not reject a missing created-project editor PNG.'

    Remove-Item -LiteralPath $afterReopenCapturePath -Force -ErrorAction Stop
    try {
        Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
            run_id = 'fixture-stage'
            source_fingerprint = 'fixture-source-fingerprint'
            staged_project_root = 'project'
            project_creation = $f5ProjectCreation
            products = $f5ProductRuns
            baseline_automation = $baselineAutomationFixture
            authoring_automation = $authoringAutomationFixture
            reopen_automation = $reopenAutomationFixture
        })
        $f5MissingReopenedCaptureFileRejected = $false
        $f5MissingReopenedCaptureFileFailure = $null
        try {
            & $driver `
                -StagingRoot $stagingRoot `
                -EvidenceRoot (Join-Path $fixtureRoot 'f5-missing-reopened-capture-file') `
                -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
                -ProfileContractSummaryPath $profileContractSummaryPath `
                -WorkspaceSummaryPath $workspaceSummaryPath `
                -RequireF5Evidence | Out-Null
        }
        catch {
            $f5MissingReopenedCaptureFileFailure = $_.Exception.Message
            $f5MissingReopenedCaptureFileText = ConvertTo-FixtureProcessText `
                -Output @($f5MissingReopenedCaptureFileFailure)
            $f5MissingReopenedCaptureFileRejected =
                $f5MissingReopenedCaptureFileText -match
                    '(?s)captures/editor-after-reopen\.png.*does not exist.*staging root'
        }
        Assert-True $f5MissingReopenedCaptureFileRejected "F5 acceptance did not reject a missing reopened editor PNG file with retained metadata; failure=$f5MissingReopenedCaptureFileFailure"
    }
    finally {
        Write-FixtureVisiblePng -Path $afterReopenCapturePath -AfterAuthoring
    }

    $f5EditorCaptureDrift = Copy-FixtureProductRuns -Runs $f5ProductRuns
    $f5EditorCaptureDriftRun = @(
        $f5EditorCaptureDrift | Where-Object {
            $_.product -eq 'editor' -and
            $null -ne $_.PSObject.Properties['editor_window_capture'] -and
            $null -ne $_.editor_window_capture
        }
    )[0]
    $f5EditorCaptureDriftRun.editor_window_capture.non_background_pixels = 0
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        project_creation = $f5ProjectCreation
        products = $f5EditorCaptureDrift
        baseline_automation = $baselineAutomationFixture
        authoring_automation = $authoringAutomationFixture
        reopen_automation = $reopenAutomationFixture
    })
    $f5EditorCaptureDriftRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'f5-editor-window-drift') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -ProfileContractSummaryPath $profileContractSummaryPath `
            -WorkspaceSummaryPath $workspaceSummaryPath `
            -RequireF5Evidence | Out-Null
    }
    catch {
        $f5EditorCaptureDriftRejected = $_.Exception.Message -match 'editor window capture.*non_background_pixels.*differs'
    }
    Assert-True $f5EditorCaptureDriftRejected 'F5 acceptance did not reject mismatched reopened editor PNG evidence.'

    $authoringRequestHashDrift = $authoringAutomationFixture | ConvertTo-Json -Depth 16 | ConvertFrom-FixtureJson
    $authoringRequestHashDrift.automation_request.sha256 = ('0' * 64)
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = $authoringProductRuns
        authoring_automation = $authoringRequestHashDrift
        reopen_automation = $reopenAutomationFixture
    })
    $authoringRequestHashDriftRejected = $false
    try {
        & $driver -StagingRoot $stagingRoot -EvidenceRoot (Join-Path $fixtureRoot 'authoring-request-hash-drift') -ExpectedSourceFingerprint 'fixture-source-fingerprint' -RequireAuthoringAutomation -RequireReopenAutomation | Out-Null
    }
    catch {
        $authoringRequestHashDriftRejected = $_.Exception.Message -match 'request hash mismatch'
    }
    Assert-True $authoringRequestHashDriftRejected 'Acceptance did not reject authoring evidence detached from its staged request hash.'

    $authoringStdoutDrift = $authoringAutomationFixture | ConvertTo-Json -Depth 16 | ConvertFrom-FixtureJson
    $authoringStdoutDrift.snapshot.inspector_translation[0] = '41'
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = $authoringProductRuns
        authoring_automation = $authoringStdoutDrift
        reopen_automation = $reopenAutomationFixture
    })
    $authoringStdoutDriftRejected = $false
    try {
        & $driver -StagingRoot $stagingRoot -EvidenceRoot (Join-Path $fixtureRoot 'authoring-stdout-drift') -ExpectedSourceFingerprint 'fixture-source-fingerprint' -RequireAuthoringAutomation -RequireReopenAutomation | Out-Null
    }
    catch {
        $authoringStdoutDriftRejected = $_.Exception.Message -match 'differs from its captured stdout report'
    }
    Assert-True $authoringStdoutDriftRejected 'Acceptance did not reject authoring evidence detached from its captured process stdout.'

    $reopenTranslationDrift = Copy-FixtureProductRuns -Runs $reopenAutomationFixture
    $reopenTranslationDrift[1].snapshot.inspector_translation[0] = '43'
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = $authoringProductRuns
        baseline_automation = $baselineAutomationFixture
        authoring_automation = $authoringAutomationFixture
        reopen_automation = $reopenTranslationDrift
    })
    $reopenTranslationDriftRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'reopen-translation-drift-evidence') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -RequireAuthoringAutomation `
            -RequireReopenAutomation | Out-Null
    }
    catch {
        $reopenTranslationDriftRejected = $_.Exception.Message -match 'differs'
    }
    Assert-True $reopenTranslationDriftRejected 'Acceptance evidence with a reopened Inspector transform drift was not rejected.'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $fixtureRoot 'reopen-translation-drift-evidence'))) 'Reopen transform drift left a partial evidence root.'

    $sceneDriftReport = Get-Content -LiteralPath (Join-Path $stagingRoot 'logs\editor-reopen-2.stdout.log') -Raw | ConvertFrom-FixtureJson
    $sceneDriftReport.snapshot.scene_nodes[1].transform.scale[0] = 2.0
    $sceneDriftEvidence = New-FixtureAutomationProcessEvidence `
        -Report $sceneDriftReport `
        -RequestPath $reopenRequestPath `
        -RequestRelativePath 'reopen/automation.json' `
        -StagingRoot $stagingRoot `
        -EvidenceLabel 'editor-reopen-scene-drift'
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = $authoringProductRuns
        baseline_automation = $baselineAutomationFixture
        authoring_automation = $authoringAutomationFixture
        reopen_automation = @($reopenAutomationFixture[0], $sceneDriftEvidence)
    })
    $reopenSceneDriftRejected = $false
    $reopenSceneDriftMessage = '<no error>'
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'reopen-scene-drift-evidence') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -RequireAuthoringAutomation `
            -RequireReopenAutomation | Out-Null
    }
    catch {
        $reopenSceneDriftMessage = $_.Exception.Message
        $reopenSceneDriftRejected = $reopenSceneDriftMessage -match 'scene_nodes.*differs from the authoring snapshot'
    }
    Assert-True $reopenSceneDriftRejected "Acceptance did not reject a reopened non-selected scene transform drift with self-consistent process evidence; observed='$reopenSceneDriftMessage'."

    $projectIdentityDriftProducts = Copy-FixtureProductRuns -Runs $manifest.product_runs
    $projectIdentityDriftRuntimes = @($projectIdentityDriftProducts | Where-Object { $_.product -eq 'runtime' })
    Assert-True ($projectIdentityDriftRuntimes.Count -eq 2) "Project identity drift fixture requires two runtime attempts; manifest_products=$(@($manifest.product_runs).Count) cloned_products=$($projectIdentityDriftProducts.Count) runtime_attempts=$($projectIdentityDriftRuntimes.Count)."
    $projectIdentityDriftRuntime = $projectIdentityDriftRuntimes[1]
    $projectIdentityDriftRuntime.runtime_product_diagnostics.project_identity = 'fixture-project-drifted'
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = $projectIdentityDriftProducts
    })
    $projectIdentityDriftRejected = $false
    $projectIdentityDriftMessage = '<no error>'
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'project-identity-drift-evidence') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' | Out-Null
    }
    catch {
        $projectIdentityDriftMessage = $_.Exception.Message
        $projectIdentityDriftRejected = $projectIdentityDriftMessage -match 'project_identity.*differs'
    }
    Assert-True $projectIdentityDriftRejected "Acceptance evidence with runtime project identity drift was not rejected; observed='$projectIdentityDriftMessage'."
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $fixtureRoot 'project-identity-drift-evidence'))) 'Project identity drift left a partial evidence root.'

    $projectPathDriftProducts = Copy-FixtureProductRuns -Runs $manifest.product_runs
    $projectPathDriftEditor = @($projectPathDriftProducts | Where-Object { $_.product -eq 'editor' })[1]
    $projectPathDriftEditor.project = 'project-drifted'
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = $projectPathDriftProducts
    })
    $projectPathDriftRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'project-path-drift-evidence') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' | Out-Null
    }
    catch {
        $projectPathDriftRejected = $_.Exception.Message -match 'staged project root.*differs'
    }
    Assert-True $projectPathDriftRejected 'Acceptance evidence with a product project-path drift was not rejected.'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $fixtureRoot 'project-path-drift-evidence'))) 'Project-path drift left a partial evidence root.'

    $duplicateAttemptProducts = Copy-FixtureProductRuns -Runs $manifest.product_runs
    $duplicateAttemptRuntime = @($duplicateAttemptProducts | Where-Object { $_.product -eq 'runtime' })[1]
    $duplicateAttemptRuntime.attempt = 1
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = $duplicateAttemptProducts
    })
    $duplicateAttemptRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'duplicate-attempt-evidence') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' | Out-Null
    }
    catch {
        $duplicateAttemptRejected = $_.Exception.Message -match 'duplicate attempt'
    }
    Assert-True $duplicateAttemptRejected 'Acceptance evidence with duplicate runtime attempts was not rejected.'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $fixtureRoot 'duplicate-attempt-evidence'))) 'Duplicate attempt evidence left a partial evidence root.'

    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = @($manifest.product_runs | Where-Object { $_.attempt -eq 1 })
    })
    $singleAttemptRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'single-attempt-evidence') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' | Out-Null
    }
    catch {
        $singleAttemptRejected = $_.Exception.Message -match 'at least two successful.*runtime.*editor'
    }
    Assert-True $singleAttemptRejected 'Acceptance evidence without two successful runtime and editor runs was not rejected.'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $fixtureRoot 'single-attempt-evidence'))) 'Single-attempt acceptance input left a partial evidence root.'

    $nestedEvidenceRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $stagingRoot 'acceptance-evidence') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' | Out-Null
    }
    catch {
        $nestedEvidenceRejected = $_.Exception.Message -match 'outside StagingRoot'
    }
    Assert-True $nestedEvidenceRejected 'Acceptance evidence nested under the staging root was not rejected.'

    [IO.File]::WriteAllText($projectManifestPath, "name = 'Mutated'`n", [Text.UTF8Encoding]::new($false))
    $stagingMutationRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'mutated-staging-evidence') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' | Out-Null
    }
    catch {
        $stagingMutationRejected = $_.Exception.Message -match 'hash mismatch'
    }
    Assert-True $stagingMutationRejected 'Acceptance did not reject a staged file that diverged from its manifest hash.'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $fixtureRoot 'mutated-staging-evidence'))) 'Staging-integrity failure left a partial evidence root.'
    [IO.File]::WriteAllText($projectManifestPath, "name = 'Fixture'`n", [Text.UTF8Encoding]::new($false))

    $duplicateManifest = [ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_at_utc = '2026-07-29T00:00:00Z'
        entries = @($stagingManifestEntry, $stagingManifestEntry)
    }
    Write-FixtureJson -Path $stagingManifestPath -Value $duplicateManifest
    $duplicateEntryRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'duplicate-entry-evidence') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' | Out-Null
    }
    catch {
        $duplicateEntryRejected = $_.Exception.Message -match 'duplicate target_relative_path'
    }
    Assert-True $duplicateEntryRejected 'Acceptance did not reject duplicate staging target paths.'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $fixtureRoot 'duplicate-entry-evidence'))) 'Duplicate staging entries left a partial evidence root.'
    Write-FixtureJson -Path $stagingManifestPath -Value $stagingManifestFixture

    $missingRuntimeDiagnosticsFixture = [ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = @(
            [ordered]@{
                product = 'runtime'
                project = 'project'
                attempt = 1
                exit_code = 0
                first_frame_presented = $true
                teardown_complete = $true
            },
            [ordered]@{
                product = 'editor'
                project = 'project'
                attempt = 1
                exit_code = 0
                first_frame_presented = $true
                teardown_complete = $true
            }
        )
    }
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value $missingRuntimeDiagnosticsFixture
    $missingRuntimeDiagnosticsRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'missing-runtime-diagnostics-evidence') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' `
            -RequireProductEvidence | Out-Null
    }
    catch {
        $missingRuntimeDiagnosticsRejected = $_.Exception.Message -match 'runtime_product_diagnostics'
    }
    Assert-True $missingRuntimeDiagnosticsRejected 'Acceptance evidence without runtime diagnostics was not rejected.'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $fixtureRoot 'missing-runtime-diagnostics-evidence'))) 'Missing runtime diagnostics left a partial evidence root.'

    $zeroInputRuntimeDiagnosticsFixture = [ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = @(
            [ordered]@{
                product = 'runtime'
                project = 'project'
                attempt = 1
                exit_code = 0
                first_frame_presented = $true
                teardown_complete = $true
                runtime_product_diagnostics = [ordered]@{
                    frame_index = 1
                    viewport = '16x16'
                    project_identity = 'fixture-project'
                    scene_uri = 'res://scenes/main.scene.toml'
                    selected_model_resource_id = 'fixture-cube-model-resource'
                    selected_material_resource_id = 'fixture-default-material-resource'
                    render_backend = 'fixture-wgpu'
                    render_adapter = 'Fixture WGPU Adapter'
                    render_adapter_type = 'discrete_gpu'
                    device_max_bind_groups = 5
                    device_max_texture_dimension_2d = 16384
                    device_max_texture_array_layers = 256
                    device_max_sampled_textures_per_shader_stage = 16
                    device_max_storage_buffers_per_shader_stage = 8
                    device_max_storage_buffer_binding_size = 134217728
                    graph_executed_pass_count = 1
                    mesh_draw_count = 1
                    directional_light_count = 1
                    material_fallback_count = 0
                    material_validation_error_count = 0
                    input_viewport_resize_count = 0
                    input_pointer_move_count = 1
                    input_mouse_button_press_count = 1
                    input_mouse_button_release_count = 1
                    input_keyboard_press_count = 1
                    input_keyboard_release_count = 0
                }
            },
            [ordered]@{
                product = 'editor'
                project = 'project'
                attempt = 1
                exit_code = 0
                first_frame_presented = $true
                teardown_complete = $true
            }
        )
    }
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value $zeroInputRuntimeDiagnosticsFixture
    $zeroInputRuntimeDiagnosticsRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'zero-input-runtime-diagnostics-evidence') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' | Out-Null
    }
    catch {
        $zeroInputRuntimeDiagnosticsRejected = $_.Exception.Message -match 'input_viewport_resize_count'
    }
    Assert-True $zeroInputRuntimeDiagnosticsRejected 'Acceptance evidence with zero runtime input consumption was not rejected.'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $fixtureRoot 'zero-input-runtime-diagnostics-evidence'))) 'Zero runtime input diagnostics left a partial evidence root.'

    $zeroInputRuntimeDiagnosticsFixture.products[0].runtime_product_diagnostics.input_viewport_resize_count = 2
    $zeroInputRuntimeDiagnosticsFixture.products[0].runtime_product_diagnostics.input_keyboard_release_count = 1
    $zeroInputRuntimeDiagnosticsFixture.products[0].runtime_product_diagnostics.material_fallback_count = 1
    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value $zeroInputRuntimeDiagnosticsFixture
    $materialFallbackRuntimeDiagnosticsRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'material-fallback-runtime-diagnostics-evidence') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' | Out-Null
    }
    catch {
        $materialFallbackRuntimeDiagnosticsRejected = $_.Exception.Message -match 'material_fallback_count'
    }
    Assert-True $materialFallbackRuntimeDiagnosticsRejected 'Acceptance evidence with runtime material fallback usage was not rejected.'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $fixtureRoot 'material-fallback-runtime-diagnostics-evidence'))) 'Material fallback runtime diagnostics left a partial evidence root.'

    Write-FixtureJson -Path (Join-Path $stagingRoot 'startup-summary.json') -Value ([ordered]@{
        run_id = 'fixture-stage'
        source_fingerprint = 'fixture-source-fingerprint'
        staged_project_root = 'project'
        products = @(
            [ordered]@{
                product = 'runtime'
                project = 'project'
                attempt = 1
                exit_code = 0
                first_frame_presented = $true
                teardown_complete = $true
                runtime_product_diagnostics = [ordered]@{
                    frame_index = 1
                    viewport = '16x16'
                    project_identity = 'fixture-project'
                    scene_uri = 'res://scenes/main.scene.toml'
                    selected_model_resource_id = 'fixture-cube-model-resource'
                    selected_material_resource_id = 'fixture-default-material-resource'
                    render_backend = 'fixture-wgpu'
                    render_adapter = 'Fixture WGPU Adapter'
                    render_adapter_type = 'discrete_gpu'
                    device_max_bind_groups = 5
                    device_max_texture_dimension_2d = 16384
                    device_max_texture_array_layers = 256
                    device_max_sampled_textures_per_shader_stage = 16
                    device_max_storage_buffers_per_shader_stage = 8
                    device_max_storage_buffer_binding_size = 134217728
                    graph_executed_pass_count = 1
                    mesh_draw_count = 1
                    directional_light_count = 1
                    material_fallback_count = 0
                    material_validation_error_count = 0
                    input_viewport_resize_count = 2
                    input_pointer_move_count = 1
                    input_mouse_button_press_count = 1
                    input_mouse_button_release_count = 1
                    input_keyboard_press_count = 1
                    input_keyboard_release_count = 1
                }
            }
        )
    })
    $missingEditorRejected = $false
    try {
        & $driver `
            -StagingRoot $stagingRoot `
            -EvidenceRoot (Join-Path $fixtureRoot 'missing-editor-evidence') `
            -ExpectedSourceFingerprint 'fixture-source-fingerprint' | Out-Null
    }
    catch {
        $missingEditorRejected = $_.Exception.Message -match 'runtime and editor'
    }
    Assert-True $missingEditorRejected 'Acceptance evidence without a successful editor product run was not rejected.'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $fixtureRoot 'missing-editor-evidence'))) 'Rejected acceptance input left a partial evidence root.'

    Write-Host 'MVP acceptance manifest contract passed'
}
finally {
    if ($env:ZIRCON_KEEP_MVP_ACCEPTANCE_FIXTURE -ne '1' -and
        (Test-Path -LiteralPath $fixtureRoot)) {
        Remove-MvpTestFixtureRoot -Path $fixtureRoot
    }
}
