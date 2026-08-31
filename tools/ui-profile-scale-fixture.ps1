function Get-ZirconUiProfileScaleFileFingerprint {
    param(
        [string]$Path,
        [string]$RelativePath
    )

    $item = Get-Item -LiteralPath $Path
    $hash = Get-FileHash -LiteralPath $Path -Algorithm SHA256
    return [pscustomobject]@{
        relative_path = $RelativePath.Replace("\", "/")
        path = $item.FullName
        sha256 = $hash.Hash.ToLowerInvariant()
        byte_length = [int64]$item.Length
    }
}

function Test-ZirconUiProfileScaleSystemDriveRoot {
    param([string]$Root)

    $normalized = $Root.Replace('/', '\')
    return $normalized -match '^(?:[Cc]:\\|\\\\\?\\[Cc]:\\|\\\\\.\\[Cc]:\\)$'
}

function Assert-ZirconUiProfileScaleProjectRoot {
    param(
        [string]$RepoRoot,
        [string]$ProjectRoot
    )

    if (-not [System.IO.Path]::IsPathRooted($ProjectRoot)) {
        throw "UI profile scale project root must be absolute."
    }
    $repo = [System.IO.Path]::GetFullPath($RepoRoot).TrimEnd('\')
    $project = [System.IO.Path]::GetFullPath($ProjectRoot).TrimEnd('\')
    $repoPrefix = $repo + [System.IO.Path]::DirectorySeparatorChar
    if ($project.Equals($repo, [System.StringComparison]::OrdinalIgnoreCase) -or
        $project.StartsWith($repoPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "UI profile scale projects must be generated outside the repository."
    }
    $root = [System.IO.Path]::GetPathRoot($project)
    if (Test-ZirconUiProfileScaleSystemDriveRoot -Root $root) {
        throw "UI profile scale projects cannot be generated on the C: system drive."
    }
    if (Test-Path -LiteralPath $project) {
        throw "UI profile scale project root already exists: $project"
    }
    return $project
}

function Write-ZirconUiHierarchyScaleScene {
    param(
        [string]$ScenePath,
        [int]$LogicalNodeCount
    )

    $encoding = New-Object System.Text.UTF8Encoding($false)
    $writer = New-Object System.IO.StreamWriter($ScenePath, $false, $encoding)
    try {
        for ($entity = 1; $entity -le $LogicalNodeCount; $entity++) {
            $writer.Write(
                "[[entities]]`nentity = $entity`nname = `"Profile Hierarchy Node $($entity.ToString('D6'))`"`nparent = 0`nactive = true`nrender_layer_mask = 1`ntransform = { translation = [0.0, 0.0, 0.0], rotation = [0.0, 0.0, 0.0, 1.0], scale = [1.0, 1.0, 1.0] }`n`n"
            )
        }
    }
    finally {
        $writer.Dispose()
    }
}

function Write-ZirconViewportPointerScaleScene {
    param(
        [string]$ScenePath,
        [ValidateRange(1, 10000)]
        [int]$SelectableNodeCount,
        [ValidateSet("static", "dynamic")]
        [string]$Mobility
    )

    $mobilityValue = if ($Mobility -eq "static") { "Static" } else { "Dynamic" }
    $columnCount = [Math]::Ceiling([Math]::Sqrt([double]$SelectableNodeCount))
    $rowCount = [Math]::Ceiling($SelectableNodeCount / $columnCount)
    $cameraDistance = [Math]::Max(10.0, $columnCount * 1.6)
    $encoding = New-Object System.Text.UTF8Encoding($false)
    $writer = New-Object System.IO.StreamWriter($ScenePath, $false, $encoding)
    try {
        $writer.Write(
            "[[entities]]`nentity = 1`nname = `"Profile Camera`"`nparent = 0`nactive = true`nrender_layer_mask = 1`ntransform = { translation = [0.0, 0.0, $cameraDistance], rotation = [0.0, 0.0, 0.0, 1.0], scale = [1.0, 1.0, 1.0] }`ncamera = { fov_y_radians = 1.7453293, z_near = 0.1, z_far = 10000.0 }`n`n"
        )
        $writer.Write(
            "[[entities]]`nentity = 2`nname = `"Profile Sun`"`nparent = 0`nactive = true`nrender_layer_mask = 1`ntransform = { translation = [0.0, 4.0, 0.0], rotation = [0.0, 0.0, 0.0, 1.0], scale = [1.0, 1.0, 1.0] }`ndirectional_light = { direction = [-0.361772505316908, -0.904431263292269, -0.226107815823067], color = [1.0, 1.0, 1.0], intensity = 3.0 }`n`n"
        )
        for ($index = 0; $index -lt $SelectableNodeCount; $index++) {
            $column = $index % $columnCount
            $row = [Math]::Floor($index / $columnCount)
            $x = ([double]$column - ($columnCount - 1) / 2.0) * 2.0
            $y = ([double]$row - ($rowCount - 1) / 2.0) * 2.0
            $entity = $index + 3
            $writer.Write(
                "[[entities]]`nentity = $entity`nname = `"Profile Viewport Node $(( $entity - 2).ToString('D6'))`"`nparent = 0`nactive = true`nrender_layer_mask = 1`nmobility = `"$mobilityValue`"`ntransform = { translation = [$x, $y, 0.0], rotation = [0.0, 0.0, 0.0, 1.0], scale = [1.0, 1.0, 1.0] }`n`n[entities.mesh.model]`nkind = `"project`"`nguid = `"00000000-0000-0000-0000-000000000002`"`npath_hint = `"assets/models/cube.obj`"`n`n[entities.mesh.material]`nkind = `"project`"`nguid = `"00000000-0000-0000-0000-000000000003`"`npath_hint = `"assets/materials/default.zmaterial`"`n`n"
            )
        }
    }
    finally {
        $writer.Dispose()
    }
}

function Get-ZirconUiAssetCatalogScaleSetFingerprint {
    param(
        [string]$ProjectRoot,
        [ValidateRange(1, 10000)]
        [int]$ExpectedCount
    )

    $assetsRoot = Join-Path $ProjectRoot "assets"
    $actualCount = [int64](Get-ChildItem -LiteralPath $assetsRoot `
            -Filter "profile_catalog_asset_*.json" -File |
            Measure-Object).Count
    if ($actualCount -ne $ExpectedCount) {
        throw "UI profile asset catalog set does not contain the declared file count."
    }

    $aggregate = [System.Security.Cryptography.IncrementalHash]::CreateHash(
        [System.Security.Cryptography.HashAlgorithmName]::SHA256
    )
    $fileHasher = [System.Security.Cryptography.SHA256]::Create()
    $encoding = New-Object System.Text.UTF8Encoding($false)
    $totalByteLength = [int64]0
    try {
        for ($index = 0; $index -lt $ExpectedCount; $index++) {
            $expectedName = "profile_catalog_asset_$('{0:D6}' -f ($index + 1)).json"
            $path = Join-Path $assetsRoot $expectedName
            if (-not [System.IO.File]::Exists($path)) {
                throw "UI profile asset catalog set does not contain the declared ordered names."
            }
            $bytes = [System.IO.File]::ReadAllBytes($path)
            $fileDigest = $fileHasher.ComputeHash($bytes)
            $relativePath = "assets/$expectedName"
            $aggregate.AppendData($encoding.GetBytes("$relativePath`0$($bytes.LongLength)`0"))
            $aggregate.AppendData($fileDigest)
            $aggregate.AppendData([byte[]](10))
            $totalByteLength += $bytes.LongLength
        }
        $digest = -join ($aggregate.GetHashAndReset() |
                ForEach-Object { $_.ToString('x2') })
    }
    finally {
        $fileHasher.Dispose()
        $aggregate.Dispose()
    }

    return [pscustomobject]@{
        relative_directory = "assets"
        file_name_prefix = "profile_catalog_asset_"
        extension = "json"
        file_count = $actualCount
        total_byte_length = $totalByteLength
        sha256 = $digest
    }
}

function Write-ZirconUiAssetCatalogScaleSources {
    param(
        [string]$AssetsRoot,
        [ValidateRange(1, 10000)]
        [int]$AssetItemCount
    )

    $encoding = New-Object System.Text.UTF8Encoding($false)
    for ($index = 1; $index -le $AssetItemCount; $index++) {
        $name = "profile_catalog_asset_$('{0:D6}' -f $index).json"
        $path = Join-Path $AssetsRoot $name
        [System.IO.File]::WriteAllText(
            $path,
            "{`"profile_asset_index`":$index}",
            $encoding
        )
    }
}

function Write-ZirconUiAssetBrowserWorkspace {
    param([string]$ProjectRoot)

    $pageId = "page:editor.asset_browser#profile"
    $instanceId = "editor.asset_browser#profile"
    $relativePath = ".zircon/editor-workspace.json"
    $path = Join-Path $ProjectRoot $relativePath
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $path) | Out-Null
    $document = [ordered]@{
        format_version = 1
        editor_workspace = [ordered]@{
            layout_version = 1
            workbench = [ordered]@{
                active_main_page = $pageId
                main_pages = @(
                    [ordered]@{
                        ExclusiveActivityWindowPage = [ordered]@{
                            id = $pageId
                            title = "Asset Browser"
                            window_instance = $instanceId
                        }
                    }
                )
                drawers = [ordered]@{}
                activity_windows = [ordered]@{}
                floating_windows = @()
                region_overrides = [ordered]@{}
                view_overrides = [ordered]@{}
            }
            open_view_instances = @(
                [ordered]@{
                    instance_id = $instanceId
                    descriptor_id = "editor.asset_browser"
                    title = "Asset Browser"
                    serializable_payload = [ordered]@{
                        source = "ui-profile-asset-catalog-scale"
                    }
                    dirty = $false
                    host = [ordered]@{ ExclusivePage = $pageId }
                }
            )
            focused_view = $instanceId
            active_drawers = @()
        }
    }
    $encoding = New-Object System.Text.UTF8Encoding($false)
    [System.IO.File]::WriteAllText(
        $path,
        ($document | ConvertTo-Json -Depth 12),
        $encoding
    )
    return Get-ZirconUiProfileScaleFileFingerprint `
        -Path $path `
        -RelativePath $relativePath
}

function New-ZirconUiHierarchyScaleFixture {
    param(
        [string]$RepoRoot,
        [string]$ProjectRoot,
        [ValidateRange(1, 100000)]
        [int]$LogicalNodeCount
    )

    $project = Assert-ZirconUiProfileScaleProjectRoot `
        -RepoRoot $RepoRoot `
        -ProjectRoot $ProjectRoot
    $templateRoot = Join-Path $RepoRoot "templates\projects\renderable-empty"
    if (-not (Test-Path -LiteralPath $templateRoot -PathType Container)) {
        throw "Canonical renderable-empty project template is missing: $templateRoot"
    }

    $parent = Split-Path -Parent $project
    New-Item -ItemType Directory -Force -Path $parent | Out-Null
    Copy-Item -LiteralPath $templateRoot -Destination $project -Recurse

    $sceneRelativePath = "assets/scenes/main.scene.toml"
    $scenePath = Join-Path $project $sceneRelativePath
    $projectManifestRelativePath = "zircon-project.toml"
    $projectManifestPath = Join-Path $project $projectManifestRelativePath
    Write-ZirconUiHierarchyScaleScene `
        -ScenePath $scenePath `
        -LogicalNodeCount $LogicalNodeCount

    return [pscustomobject]@{
        schema_version = 1
        kind = "hierarchy_scene"
        project_root = $project
        template_relative_path = "templates/projects/renderable-empty"
        logical_node_count = $LogicalNodeCount
        scene_entity_count = $LogicalNodeCount
        project_manifest = Get-ZirconUiProfileScaleFileFingerprint `
            -Path $projectManifestPath `
            -RelativePath $projectManifestRelativePath
        scene = Get-ZirconUiProfileScaleFileFingerprint `
            -Path $scenePath `
            -RelativePath $sceneRelativePath
    }
}

function New-ZirconViewportPointerScaleFixture {
    param(
        [string]$RepoRoot,
        [string]$ProjectRoot,
        [ValidateRange(1, 10000)]
        [int]$SelectableNodeCount,
        [ValidateSet("static", "dynamic")]
        [string]$Mobility = "static"
    )

    $project = Assert-ZirconUiProfileScaleProjectRoot `
        -RepoRoot $RepoRoot `
        -ProjectRoot $ProjectRoot
    $templateRoot = Join-Path $RepoRoot "templates\projects\renderable-empty"
    if (-not (Test-Path -LiteralPath $templateRoot -PathType Container)) {
        throw "Canonical renderable-empty project template is missing: $templateRoot"
    }

    $parent = Split-Path -Parent $project
    New-Item -ItemType Directory -Force -Path $parent | Out-Null
    Copy-Item -LiteralPath $templateRoot -Destination $project -Recurse

    $sceneRelativePath = "assets/scenes/main.scene.toml"
    $scenePath = Join-Path $project $sceneRelativePath
    $projectManifestRelativePath = "zircon-project.toml"
    $projectManifestPath = Join-Path $project $projectManifestRelativePath
    Write-ZirconViewportPointerScaleScene `
        -ScenePath $scenePath `
        -SelectableNodeCount $SelectableNodeCount `
        -Mobility $Mobility

    return [pscustomobject]@{
        schema_version = 1
        kind = "viewport_pointer_scene"
        project_root = $project
        template_relative_path = "templates/projects/renderable-empty"
        selectable_node_count = $SelectableNodeCount
        scene_entity_count = $SelectableNodeCount + 2
        mobility = $Mobility
        project_manifest = Get-ZirconUiProfileScaleFileFingerprint `
            -Path $projectManifestPath `
            -RelativePath $projectManifestRelativePath
        scene = Get-ZirconUiProfileScaleFileFingerprint `
            -Path $scenePath `
            -RelativePath $sceneRelativePath
    }
}

function New-ZirconUiAssetCatalogScaleFixture {
    param(
        [string]$RepoRoot,
        [string]$ProjectRoot,
        [ValidateRange(1, 10000)]
        [int]$AssetItemCount
    )

    $project = Assert-ZirconUiProfileScaleProjectRoot `
        -RepoRoot $RepoRoot `
        -ProjectRoot $ProjectRoot
    $templateRoot = Join-Path $RepoRoot "templates\projects\renderable-empty"
    if (-not (Test-Path -LiteralPath $templateRoot -PathType Container)) {
        throw "Canonical renderable-empty project template is missing: $templateRoot"
    }

    $parent = Split-Path -Parent $project
    New-Item -ItemType Directory -Force -Path $parent | Out-Null
    Copy-Item -LiteralPath $templateRoot -Destination $project -Recurse

    $projectManifestRelativePath = "zircon-project.toml"
    $projectManifestPath = Join-Path $project $projectManifestRelativePath
    $sceneRelativePath = "assets/scenes/main.scene.toml"
    $scenePath = Join-Path $project $sceneRelativePath
    Write-ZirconUiAssetCatalogScaleSources `
        -AssetsRoot (Join-Path $project "assets") `
        -AssetItemCount $AssetItemCount
    $assetSources = Get-ZirconUiAssetCatalogScaleSetFingerprint `
        -ProjectRoot $project `
        -ExpectedCount $AssetItemCount
    $workspace = Write-ZirconUiAssetBrowserWorkspace -ProjectRoot $project

    return [pscustomobject]@{
        schema_version = 1
        kind = "asset_catalog_json"
        project_root = $project
        template_relative_path = "templates/projects/renderable-empty"
        asset_item_count = $AssetItemCount
        source_extension = "json"
        project_manifest = Get-ZirconUiProfileScaleFileFingerprint `
            -Path $projectManifestPath `
            -RelativePath $projectManifestRelativePath
        scene = Get-ZirconUiProfileScaleFileFingerprint `
            -Path $scenePath `
            -RelativePath $sceneRelativePath
        workspace = $workspace
        asset_sources = $assetSources
    }
}
