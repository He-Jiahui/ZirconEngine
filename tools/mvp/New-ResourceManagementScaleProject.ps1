[CmdletBinding()]
param(
    [string]$ProjectRoot,
    [ValidateRange(1, 100000)][int]$DataAssetCount = 1,
    [string]$ProductInputManifestPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
Import-Module (Join-Path $PSScriptRoot 'MvpProductSourceIdentity.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'MvpArtifactStoragePolicy.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'ResourceManagementScaleInventory.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $repoRoot 'tools\WindowsPathResolver.psm1') -Force -ErrorAction Stop
if ([string]::IsNullOrWhiteSpace($ProjectRoot)) {
    $ProjectRoot = New-MvpArtifactStoragePath -NamespaceId 'resource-management-projects'
}

function Assert-ResourceManagementScaleProjectDirectory {
    param([Parameter(Mandatory)][string]$Path)

    $storage = Resolve-MvpArtifactStoragePath -Path $Path -NamespaceId 'resource-management-projects'
    if ([IO.Directory]::Exists($storage.operation_path) -or [IO.File]::Exists($storage.operation_path)) {
        throw "-ProjectRoot must not already exist so the generated resource-management project has one immutable input identity: $($storage.display_path)"
    }
    return [pscustomobject]@{
        OperationalPath = $storage.operation_path
        DisplayPath = $storage.display_path
        StoragePolicy = $storage
    }
}

function Write-ResourceManagementScaleFileNew {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][byte[]]$Bytes
    )

    [IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($Path)) | Out-Null
    $stream = $null
    try {
        try {
            $stream = [IO.FileStream]::new(
                $Path,
                [IO.FileMode]::CreateNew,
                [IO.FileAccess]::Write,
                [IO.FileShare]::None
            )
        }
        catch [IO.IOException] {
            throw "Refusing to overwrite generated resource-management scale file: $Path"
        }
        $stream.Write($Bytes, 0, $Bytes.Length)
        $stream.Flush($true)
    }
    finally {
        if ($null -ne $stream) {
            $stream.Dispose()
        }
    }
}

function Copy-ResourceManagementScaleTemplate {
    param(
        [Parameter(Mandatory)]$BuildSet,
        [Parameter(Mandatory)][string]$DestinationRoot
    )

    $templatePrefix = 'templates/projects/renderable-empty/'
    $templateFiles = @($BuildSet.files | Where-Object {
            ([string]$_.relative_path).StartsWith($templatePrefix, [StringComparison]::Ordinal)
        })
    if ($templateFiles.Count -eq 0) {
        throw "Resource-management scale template is absent from BuildSet $($BuildSet.build_set_id)."
    }
    foreach ($file in $templateFiles) {
        $buildSetRelativePath = [string]$file.relative_path
        $relativePath = $buildSetRelativePath.Substring($templatePrefix.Length)
        $sourceFile = [IO.Path]::Combine(
            [string]$BuildSet.snapshot_root,
            $buildSetRelativePath.Replace('/', [IO.Path]::DirectorySeparatorChar)
        )
        $destinationPath = [IO.Path]::Combine(
            $DestinationRoot,
            $relativePath.Replace('/', [IO.Path]::DirectorySeparatorChar)
        )
        Write-ResourceManagementScaleFileNew -Path $destinationPath -Bytes ([IO.File]::ReadAllBytes($sourceFile))
    }
}

function Write-ResourceManagementScaleDataSources {
    param(
        [Parameter(Mandatory)][string]$ProjectRoot,
        [Parameter(Mandatory)][ValidateRange(1, 100000)][int]$DataAssetCount
    )

    # Keep source roots independent and leave sidecars to the real project scanner/importer.
    $dataRoot = Join-ZirconWindowsPath -Path $ProjectRoot -ChildPath 'assets\data'
    [IO.Directory]::CreateDirectory($dataRoot) | Out-Null
    $encoding = [Text.UTF8Encoding]::new($false)
    for ($index = 1; $index -le $DataAssetCount; $index++) {
        $sourcePath = Join-ZirconWindowsPath `
            -Path $dataRoot `
            -ChildPath ('catalog_{0:D6}.json' -f $index)
        $payload = '{{"index":{0},"payload":"resource-management-scale"}}{1}' -f $index, [Environment]::NewLine
        Write-ResourceManagementScaleFileNew -Path $sourcePath -Bytes $encoding.GetBytes($payload)
    }
    return $dataRoot
}

function New-ResourceManagementScaleProject {
    param(
        [Parameter(Mandatory)][string]$ProjectRoot,
        [Parameter(Mandatory)][ValidateRange(1, 100000)][int]$DataAssetCount,
        [Parameter(Mandatory)]$SourceIdentity
    )

    $sourceFingerprint = [string]$SourceIdentity.source_fingerprint
    $buildSetId = [string]$SourceIdentity.build_set_id
    $productInputManifestSha256 = [string]$SourceIdentity.manifest_sha256
    if ($sourceFingerprint -notmatch '^[0-9A-F]{64}$' -or
        $buildSetId -notmatch '^[0-9A-F]{64}$' -or
        -not $sourceFingerprint.Equals($buildSetId, [StringComparison]::Ordinal) -or
        -not $buildSetId.Equals([string]$SourceIdentity.build_set.build_set_id, [StringComparison]::Ordinal)) {
        throw 'Resource-management scale project source identity must bind one verified BuildSet.'
    }
    if ($productInputManifestSha256 -notmatch '^[0-9A-F]{64}$') {
        throw 'Resource-management scale project ProductInputManifest identity must be an uppercase SHA-256 value.'
    }
    $projectResolution = Assert-ResourceManagementScaleProjectDirectory -Path $ProjectRoot
    $templateRoot = [IO.Path]::Combine(
        [string]$SourceIdentity.build_set.snapshot_root,
        'templates\projects\renderable-empty'
    )
    if (-not [IO.Directory]::Exists($templateRoot)) {
        throw "Resource-management scale template does not exist in BuildSet $buildSetId."
    }

    $destinationParent = [IO.Path]::GetDirectoryName($projectResolution.OperationalPath)
    [IO.Directory]::CreateDirectory($destinationParent) | Out-Null
    $partialProjectRoot = "$($projectResolution.OperationalPath).partial-$([guid]::NewGuid().ToString('N'))"
    if ([IO.Directory]::Exists($partialProjectRoot) -or [IO.File]::Exists($partialProjectRoot)) {
        throw "Generated resource-management scale-project temporary directory already exists: $partialProjectRoot"
    }

    try {
        [IO.Directory]::CreateDirectory($partialProjectRoot) | Out-Null
        Copy-ResourceManagementScaleTemplate `
            -BuildSet $SourceIdentity.build_set `
            -DestinationRoot $partialProjectRoot
        $dataRoot = Write-ResourceManagementScaleDataSources `
            -ProjectRoot $partialProjectRoot `
            -DataAssetCount $DataAssetCount
        $dataInventorySha256 = Get-ResourceManagementScaleInventorySha256 `
            -DataRoot $dataRoot `
            -DataAssetCount $DataAssetCount

        $manifest = [ordered]@{
            schema_version       = 2
            source_fingerprint   = $buildSetId
            build_set_id         = $buildSetId
            product_input_manifest_sha256 = $productInputManifestSha256
            data_asset_count     = $DataAssetCount
            data_inventory_sha256 = $dataInventorySha256
            asset_kind           = 'Data'
            importer_id          = 'zircon.builtin.data.json'
            data_virtual_prefix  = 'res://data/'
            data_source_pattern  = 'res://data/catalog_*.json'
        }
        $manifestBytes = [Text.UTF8Encoding]::new($false).GetBytes(($manifest | ConvertTo-Json -Depth 3))
        Write-ResourceManagementScaleFileNew `
            -Path (Join-ZirconWindowsPath -Path $partialProjectRoot -ChildPath 'resource-management-scale-project.json') `
            -Bytes $manifestBytes
        [IO.Directory]::Move($partialProjectRoot, $projectResolution.OperationalPath)
    }
    catch {
        if ([IO.Directory]::Exists($partialProjectRoot)) {
            [IO.Directory]::Delete($partialProjectRoot, $true)
        }
        throw
    }

    return [pscustomobject]@{
        project_root        = $projectResolution.DisplayPath
        build_set_id        = $buildSetId
        data_asset_count    = $DataAssetCount
        data_inventory_sha256 = $dataInventorySha256
        asset_kind          = 'Data'
        importer_id         = 'zircon.builtin.data.json'
        data_virtual_prefix = 'res://data/'
        manifest_path       = (Resolve-ZirconWindowsPath -Path (Join-ZirconWindowsPath -Path $projectResolution.OperationalPath -ChildPath 'resource-management-scale-project.json')).DisplayPath
    }
}

if ($env:RESOURCE_MANAGEMENT_SCALE_PROJECT_TEST_MODE -ne '1') {
    if ([string]::IsNullOrWhiteSpace($ProductInputManifestPath)) {
        throw '-ProductInputManifestPath is required to bind the resource-management scale project to its BuildSet.'
    }
    $sourceIdentity = Resolve-MvpProductSourceIdentity -ManifestPath $ProductInputManifestPath
    New-ResourceManagementScaleProject `
        -ProjectRoot $ProjectRoot `
        -DataAssetCount $DataAssetCount `
        -SourceIdentity $sourceIdentity
}
