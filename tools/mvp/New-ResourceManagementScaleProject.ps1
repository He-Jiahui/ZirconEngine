[CmdletBinding()]
param(
    [string]$ProjectRoot = (Join-Path 'E:\ZirconBuilds\mvp-resource-management-projects' ([guid]::NewGuid().ToString('N'))),
    [ValidateRange(1, 100000)][int]$DataAssetCount = 1
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
Import-Module (Join-Path $PSScriptRoot 'MvpProductInputManifest.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'ResourceManagementScaleInventory.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $repoRoot 'tools\WindowsPathResolver.psm1') -Force -ErrorAction Stop

function Assert-ResourceManagementScaleProjectDirectory {
    param([Parameter(Mandatory)][string]$Path)

    $resolution = Resolve-ZirconWindowsPath -Path $Path
    if ($resolution.DisplayPath -notmatch '^E:\\ZirconBuilds\\mvp-resource-management-projects\\(?:[A-Za-z0-9][A-Za-z0-9._-]*)(?:\\|$)') {
        throw "-ProjectRoot resource-management scale project must resolve under E:\ZirconBuilds\mvp-resource-management-projects\<session>: $($resolution.DisplayPath)"
    }
    if ([IO.Directory]::Exists($resolution.OperationalPath) -or [IO.File]::Exists($resolution.OperationalPath)) {
        throw "-ProjectRoot must not already exist so the generated resource-management project has one immutable input identity: $($resolution.DisplayPath)"
    }
    return $resolution
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
        [Parameter(Mandatory)][string]$TemplateRoot,
        [Parameter(Mandatory)][string]$DestinationRoot
    )

    $templateRootPrefix = $TemplateRoot.TrimEnd([char[]]@('\', '/')) + [IO.Path]::DirectorySeparatorChar
    foreach ($sourceFile in [IO.Directory]::EnumerateFiles($TemplateRoot, '*', [IO.SearchOption]::AllDirectories)) {
        if (-not $sourceFile.StartsWith($templateRootPrefix, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Template file escaped its declared root: $sourceFile"
        }
        $relativePath = $sourceFile.Substring($templateRootPrefix.Length)
        $destinationPath = Join-ZirconWindowsPath -Path $DestinationRoot -ChildPath $relativePath
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
        [Parameter(Mandatory)][string]$SourceFingerprint
    )

    if ($SourceFingerprint -notmatch '^[0-9A-F]{64}$') {
        throw 'Resource-management scale project source fingerprint must be an uppercase SHA-256 value.'
    }
    $projectResolution = Assert-ResourceManagementScaleProjectDirectory -Path $ProjectRoot
    $templateRoot = (Resolve-ZirconWindowsPath -Path (Join-Path $repoRoot 'templates\projects\renderable-empty')).OperationalPath
    if (-not [IO.Directory]::Exists($templateRoot)) {
        throw "Resource-management scale template does not exist: $templateRoot"
    }

    $destinationParent = [IO.Path]::GetDirectoryName($projectResolution.OperationalPath)
    [IO.Directory]::CreateDirectory($destinationParent) | Out-Null
    $partialProjectRoot = "$($projectResolution.OperationalPath).partial-$([guid]::NewGuid().ToString('N'))"
    if ([IO.Directory]::Exists($partialProjectRoot) -or [IO.File]::Exists($partialProjectRoot)) {
        throw "Generated resource-management scale-project temporary directory already exists: $partialProjectRoot"
    }

    try {
        [IO.Directory]::CreateDirectory($partialProjectRoot) | Out-Null
        Copy-ResourceManagementScaleTemplate -TemplateRoot $templateRoot -DestinationRoot $partialProjectRoot
        $dataRoot = Write-ResourceManagementScaleDataSources `
            -ProjectRoot $partialProjectRoot `
            -DataAssetCount $DataAssetCount
        $dataInventorySha256 = Get-ResourceManagementScaleInventorySha256 `
            -DataRoot $dataRoot `
            -DataAssetCount $DataAssetCount

        $manifest = [ordered]@{
            schema_version       = 1
            source_fingerprint   = $SourceFingerprint
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
        data_asset_count    = $DataAssetCount
        data_inventory_sha256 = $dataInventorySha256
        asset_kind          = 'Data'
        importer_id         = 'zircon.builtin.data.json'
        data_virtual_prefix = 'res://data/'
        manifest_path       = (Resolve-ZirconWindowsPath -Path (Join-ZirconWindowsPath -Path $projectResolution.OperationalPath -ChildPath 'resource-management-scale-project.json')).DisplayPath
    }
}

if ($env:RESOURCE_MANAGEMENT_SCALE_PROJECT_TEST_MODE -ne '1') {
    $sourceFingerprint = Get-MvpSourceFingerprint -RepositoryRoot $repoRoot
    New-ResourceManagementScaleProject `
        -ProjectRoot $ProjectRoot `
        -DataAssetCount $DataAssetCount `
        -SourceFingerprint $sourceFingerprint
}
