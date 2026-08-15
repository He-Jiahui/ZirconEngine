[CmdletBinding()]
param(
    [string]$ProjectRoot,
    [ValidateRange(1, 100)][int]$ChangePercent = 1
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
Import-Module (Join-Path $PSScriptRoot 'MvpProductInputManifest.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'ResourceManagementScaleInventory.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $repoRoot 'tools\WindowsPathResolver.psm1') -Force -ErrorAction Stop

function Assert-ResourceManagementScaleMutationProjectDirectory {
    param([Parameter(Mandatory)][string]$Path)

    $resolution = Resolve-ZirconWindowsPath -Path $Path
    if ($resolution.DisplayPath -notmatch '^E:\\ZirconBuilds\\mvp-resource-management-projects\\(?:[A-Za-z0-9][A-Za-z0-9._-]*)(?:\\|$)') {
        throw "-ProjectRoot resource-management change set must resolve under E:\ZirconBuilds\mvp-resource-management-projects\<session>: $($resolution.DisplayPath)"
    }
    if (-not [IO.Directory]::Exists($resolution.OperationalPath)) {
        throw "-ProjectRoot resource-management scale project does not exist: $($resolution.DisplayPath)"
    }
    return $resolution
}

function New-ResourceManagementScaleChangeSetLease {
    param([Parameter(Mandatory)]$ProjectResolution)

    $leasePath = Join-ZirconWindowsPath `
        -Path $ProjectResolution.OperationalPath `
        -ChildPath '.zircon\resource-management-scale-change-set.active'
    [IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($leasePath)) | Out-Null
    try {
        return [IO.FileStream]::new(
            $leasePath,
            [IO.FileMode]::CreateNew,
            [IO.FileAccess]::ReadWrite,
            [IO.FileShare]::None,
            1,
            [IO.FileOptions]::DeleteOnClose
        )
    }
    catch [IO.IOException] {
        throw "Resource-management scale change set is already active or changed after preflight: $($ProjectResolution.DisplayPath)"
    }
}

function Get-ResourceManagementScaleProjectMetadata {
    param(
        [Parameter(Mandatory)]$ProjectResolution,
        [Parameter(Mandatory)][string]$ExpectedSourceFingerprint
    )

    if ($ExpectedSourceFingerprint -notmatch '^[0-9A-F]{64}$') {
        throw 'Resource-management change-set expected source fingerprint must be an uppercase SHA-256 value.'
    }
    $metadataPath = Join-ZirconWindowsPath `
        -Path $ProjectResolution.OperationalPath `
        -ChildPath 'resource-management-scale-project.json'
    if (-not [IO.File]::Exists($metadataPath)) {
        throw "Resource-management scale project metadata does not exist: $($ProjectResolution.DisplayPath)"
    }
    try {
        $metadata = [IO.File]::ReadAllText($metadataPath) | ConvertFrom-Json -ErrorAction Stop
    }
    catch {
        throw "Resource-management scale project metadata is not valid JSON: ${metadataPath}: $($_.Exception.Message)"
    }
    if ($null -eq $metadata -or [int]$metadata.schema_version -ne 1) {
        throw "Resource-management scale project metadata has an unsupported schema_version: $metadataPath"
    }
    if ([string]$metadata.source_fingerprint -notmatch '^[0-9A-F]{64}$') {
        throw "Resource-management scale project metadata has an invalid source fingerprint: $metadataPath"
    }
    if (-not ([string]$metadata.source_fingerprint).Equals($ExpectedSourceFingerprint, [StringComparison]::Ordinal)) {
        throw 'Resource-management scale project belongs to a different source snapshot. Regenerate it before applying a change set.'
    }
    if ([int]$metadata.data_asset_count -lt 1 -or [int]$metadata.data_asset_count -gt 100000) {
        throw "Resource-management scale project metadata has an invalid data_asset_count: $metadataPath"
    }
    if ([string]$metadata.data_inventory_sha256 -notmatch '^[0-9A-F]{64}$') {
        throw "Resource-management scale project metadata has an invalid data_inventory_sha256: $metadataPath"
    }
    if ([string]$metadata.asset_kind -ne 'Data' -or
        [string]$metadata.importer_id -ne 'zircon.builtin.data.json' -or
        [string]$metadata.data_virtual_prefix -ne 'res://data/' -or
        [string]$metadata.data_source_pattern -ne 'res://data/catalog_*.json') {
        throw "Resource-management scale project metadata does not describe the supported JSON data workload: $metadataPath"
    }
    return $metadata
}

function Assert-ResourceManagementScaleDataInventory {
    param(
        [Parameter(Mandatory)]$ProjectResolution,
        [Parameter(Mandatory)][ValidateRange(1, 100000)][int]$DataAssetCount
    )

    $dataRoot = Resolve-ZirconWindowsPath -Path (Join-ZirconWindowsPath `
            -Path $ProjectResolution.OperationalPath `
            -ChildPath 'assets\data')
    if (-not [IO.Directory]::Exists($dataRoot.OperationalPath)) {
        throw "Resource-management scale data source inventory is missing: $($dataRoot.DisplayPath)"
    }
    $sourceFiles = @([IO.Directory]::EnumerateFiles(
            $dataRoot.OperationalPath,
            '*.json',
            [IO.SearchOption]::TopDirectoryOnly
        ))
    if ($sourceFiles.Count -ne $DataAssetCount) {
        throw "Resource-management scale data source inventory count $($sourceFiles.Count) does not match data_asset_count $DataAssetCount."
    }
    for ($index = 1; $index -le $DataAssetCount; $index++) {
        $expectedPath = Join-ZirconWindowsPath `
            -Path $dataRoot.OperationalPath `
            -ChildPath ('catalog_{0:D6}.json' -f $index)
        if (-not [IO.File]::Exists($expectedPath)) {
            throw ("Resource-management scale data source inventory is missing catalog_{0:D6}.json." -f $index)
        }
    }
    return $dataRoot
}

function Write-ResourceManagementScaleChangeSetFileNew {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][byte[]]$Bytes
    )

    [IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($Path)) | Out-Null
    $stream = $null
    $created = $false
    try {
        try {
            $stream = [IO.FileStream]::new(
                $Path,
                [IO.FileMode]::CreateNew,
                [IO.FileAccess]::Write,
                [IO.FileShare]::None
            )
            $created = $true
        }
        catch [IO.IOException] {
            throw "Refusing to overwrite resource-management scale change-set file: $Path"
        }
        $stream.Write($Bytes, 0, $Bytes.Length)
        $stream.Flush($true)
    }
    catch {
        if ($null -ne $stream) {
            $stream.Dispose()
            $stream = $null
        }
        if ($created -and [IO.File]::Exists($Path)) {
            [IO.File]::Delete($Path)
        }
        throw
    }
    finally {
        if ($null -ne $stream) {
            $stream.Dispose()
        }
    }
}

function Get-ResourceManagementScaleChangeCount {
    param(
        [Parameter(Mandatory)][ValidateRange(1, 100000)][int]$DataAssetCount,
        [Parameter(Mandatory)][ValidateRange(1, 100)][int]$ChangePercent
    )

    return [Math]::Max(1, [int][Math]::Ceiling($DataAssetCount * ($ChangePercent / 100.0)))
}

function Get-ResourceManagementScaleChangeEntries {
    param(
        [Parameter(Mandatory)]$ProjectResolution,
        [Parameter(Mandatory)][ValidateRange(1, 100000)][int]$DataAssetCount,
        [Parameter(Mandatory)][ValidateRange(1, 100)][int]$ChangePercent,
        [Parameter(Mandatory)][string]$StagingRoot
    )

    $dataRoot = Resolve-ZirconWindowsPath -Path (Join-ZirconWindowsPath `
            -Path $ProjectResolution.OperationalPath `
            -ChildPath 'assets\data')
    $dataRootPrefix = $dataRoot.DisplayPath.TrimEnd('\', '/') + '\'
    $changeCount = Get-ResourceManagementScaleChangeCount `
        -DataAssetCount $DataAssetCount `
        -ChangePercent $ChangePercent
    $encoding = [Text.UTF8Encoding]::new($false)
    $entries = [Collections.Generic.List[object]]::new()
    for ($index = 1; $index -le $changeCount; $index++) {
        $fileName = 'catalog_{0:D6}.json' -f $index
        $sourceResolution = Resolve-ZirconWindowsPath -Path (Join-ZirconWindowsPath `
                -Path $dataRoot.OperationalPath `
                -ChildPath $fileName)
        if (-not $sourceResolution.DisplayPath.StartsWith($dataRootPrefix, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Resource-management scale source escaped assets/data: $($sourceResolution.DisplayPath)"
        }
        try {
            $source = [IO.File]::ReadAllText($sourceResolution.OperationalPath) | ConvertFrom-Json -ErrorAction Stop
        }
        catch {
            throw "Resource-management scale source is not valid JSON: $($sourceResolution.DisplayPath): $($_.Exception.Message)"
        }
        if ([int]$source.index -ne $index -or [string]$source.payload -ne 'resource-management-scale') {
            throw "Resource-management scale source does not match the generated workload: $($sourceResolution.DisplayPath)"
        }
        if ($null -ne $source.PSObject.Properties['workload_revision']) {
            throw "Resource-management scale source has already been changed: $($sourceResolution.DisplayPath)"
        }
        $stagedPath = Join-ZirconWindowsPath -Path $StagingRoot -ChildPath $fileName
        $backupPath = Join-ZirconWindowsPath -Path $StagingRoot -ChildPath ($fileName + '.backup')
        $payload = '{{"index":{0},"payload":"resource-management-scale","workload_revision":1}}{1}' -f $index, [Environment]::NewLine
        Write-ResourceManagementScaleChangeSetFileNew -Path $stagedPath -Bytes $encoding.GetBytes($payload)
        $entries.Add([pscustomobject]@{
                Index       = $index
                SourcePath  = $sourceResolution.OperationalPath
                StagedPath  = $stagedPath
                BackupPath  = $backupPath
                VirtualPath = 'res://data/' + $fileName
            }) | Out-Null
    }
    return @($entries)
}

function Write-ResourceManagementScaleChangeManifest {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$SourceFingerprint,
        [Parameter(Mandatory)][int]$DataAssetCount,
        [Parameter(Mandatory)][int]$ChangePercent,
        [Parameter(Mandatory)][string]$BaselineDataInventorySha256,
        [Parameter(Mandatory)][string]$ChangedDataInventorySha256,
        [Parameter(Mandatory)][object[]]$Entries
    )

    $manifest = [ordered]@{
        schema_version       = 1
        source_fingerprint   = $SourceFingerprint
        data_asset_count     = $DataAssetCount
        baseline_data_inventory_sha256 = $BaselineDataInventorySha256
        changed_data_inventory_sha256 = $ChangedDataInventorySha256
        asset_kind           = 'Data'
        importer_id          = 'zircon.builtin.data.json'
        data_virtual_prefix  = 'res://data/'
        changed_asset_count  = $Entries.Count
        change_percent       = $ChangePercent
        changed_virtual_paths = @($Entries | ForEach-Object { $_.VirtualPath })
    }
    $bytes = [Text.UTF8Encoding]::new($false).GetBytes(($manifest | ConvertTo-Json -Depth 4))
    Write-ResourceManagementScaleChangeSetFileNew -Path $Path -Bytes $bytes
    return $manifest
}

function Set-ResourceManagementScaleProjectChangeSet {
    param(
        [Parameter(Mandatory)][string]$ProjectRoot,
        [Parameter(Mandatory)][ValidateRange(1, 100)][int]$ChangePercent,
        [Parameter(Mandatory)][string]$ExpectedSourceFingerprint
    )

    $projectResolution = Assert-ResourceManagementScaleMutationProjectDirectory -Path $ProjectRoot
    $lease = New-ResourceManagementScaleChangeSetLease -ProjectResolution $projectResolution
    try {
    $metadata = Get-ResourceManagementScaleProjectMetadata `
        -ProjectResolution $projectResolution `
        -ExpectedSourceFingerprint $ExpectedSourceFingerprint
    $dataRoot = Assert-ResourceManagementScaleDataInventory `
        -ProjectResolution $projectResolution `
        -DataAssetCount ([int]$metadata.data_asset_count)
    $baselineDataInventorySha256 = Get-ResourceManagementScaleInventorySha256 `
        -DataRoot $dataRoot.OperationalPath `
        -DataAssetCount ([int]$metadata.data_asset_count)
    if (-not $baselineDataInventorySha256.Equals([string]$metadata.data_inventory_sha256, [StringComparison]::Ordinal)) {
        throw 'Resource-management scale project data inventory does not match its immutable metadata fingerprint.'
    }
    $changeManifestPath = Join-ZirconWindowsPath `
        -Path $projectResolution.OperationalPath `
        -ChildPath 'resource-management-scale-change-set.json'
    if ([IO.File]::Exists($changeManifestPath)) {
        throw "Resource-management scale project already has a change set: $($projectResolution.DisplayPath)"
    }

    $stagingRoot = Join-ZirconWindowsPath `
        -Path $projectResolution.OperationalPath `
        -ChildPath ('.zircon\resource-management-change-set-' + [guid]::NewGuid().ToString('N'))
    $entries = @()
    $committed = [Collections.Generic.List[object]]::new()
    $manifestPublished = $false
    try {
        [IO.Directory]::CreateDirectory($stagingRoot) | Out-Null
        $entries = @(Get-ResourceManagementScaleChangeEntries `
                -ProjectResolution $projectResolution `
                -DataAssetCount ([int]$metadata.data_asset_count) `
                -ChangePercent $ChangePercent `
                -StagingRoot $stagingRoot)
        foreach ($entry in $entries) {
            [IO.File]::Replace($entry.StagedPath, $entry.SourcePath, $entry.BackupPath)
            $committed.Add($entry) | Out-Null
        }
        $changedDataInventorySha256 = Get-ResourceManagementScaleInventorySha256 `
            -DataRoot $dataRoot.OperationalPath `
            -DataAssetCount ([int]$metadata.data_asset_count)
        $manifest = Write-ResourceManagementScaleChangeManifest `
            -Path $changeManifestPath `
            -SourceFingerprint ([string]$metadata.source_fingerprint) `
            -DataAssetCount ([int]$metadata.data_asset_count) `
            -ChangePercent $ChangePercent `
            -BaselineDataInventorySha256 $baselineDataInventorySha256 `
            -ChangedDataInventorySha256 $changedDataInventorySha256 `
            -Entries $entries
        $manifestPublished = $true
    }
    catch {
        $primaryError = $_
        if ($manifestPublished -and [IO.File]::Exists($changeManifestPath)) {
            [IO.File]::Delete($changeManifestPath)
        }
        for ($index = $committed.Count - 1; $index -ge 0; $index--) {
            $entry = $committed[$index]
            if ([IO.File]::Exists($entry.BackupPath)) {
                try {
                    [IO.File]::Replace($entry.BackupPath, $entry.SourcePath, $null)
                }
                catch {
                    throw "Resource-management scale change-set rollback failed after $($primaryError.Exception.Message): $($_.Exception.Message)"
                }
            }
        }
        throw $primaryError
    }
    finally {
        if ([IO.Directory]::Exists($stagingRoot)) {
            [IO.Directory]::Delete($stagingRoot, $true)
        }
    }

    return [pscustomobject]@{
        project_root          = $projectResolution.DisplayPath
        source_fingerprint    = [string]$metadata.source_fingerprint
        data_asset_count      = [int]$metadata.data_asset_count
        data_inventory_sha256 = $changedDataInventorySha256
        asset_kind            = 'Data'
        importer_id           = 'zircon.builtin.data.json'
        data_virtual_prefix   = 'res://data/'
        change_percent        = $ChangePercent
        changed_asset_count   = $entries.Count
        changed_virtual_paths = @($entries | ForEach-Object { $_.VirtualPath })
        manifest_path         = (Resolve-ZirconWindowsPath -Path $changeManifestPath).DisplayPath
    }
    }
    finally {
        $lease.Dispose()
    }
}

if ($env:RESOURCE_MANAGEMENT_SCALE_PROJECT_CHANGESET_TEST_MODE -ne '1') {
    if ([string]::IsNullOrWhiteSpace($ProjectRoot)) {
        throw '-ProjectRoot is required when applying a resource-management scale change set.'
    }
    $sourceFingerprint = Get-MvpSourceFingerprint -RepositoryRoot $repoRoot
    Set-ResourceManagementScaleProjectChangeSet `
        -ProjectRoot $ProjectRoot `
        -ChangePercent $ChangePercent `
        -ExpectedSourceFingerprint $sourceFingerprint
}
