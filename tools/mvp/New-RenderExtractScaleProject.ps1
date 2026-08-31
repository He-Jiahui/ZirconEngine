[CmdletBinding()]
param(
    [string]$ProjectRoot,
    [ValidateRange(1, 100000)]
    [int]$PrimitiveCount = 1,
    [string]$ProfilingInputManifestPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
Import-Module (Join-Path $PSScriptRoot 'RenderExtractSourceIdentity.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'MvpArtifactStoragePolicy.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $repoRoot 'tools\WindowsPathResolver.psm1') -Force -ErrorAction Stop

if ([string]::IsNullOrWhiteSpace($ProjectRoot)) {
    $ProjectRoot = New-MvpArtifactStoragePath -NamespaceId 'render-extract-scale-projects'
}

function Assert-RenderExtractScaleProjectDirectory {
    param([Parameter(Mandatory)][string]$Path)

    $storage = Resolve-MvpArtifactStoragePath `
        -Path $Path `
        -NamespaceId 'render-extract-scale-projects'
    if ([IO.Directory]::Exists($storage.operation_path) -or [IO.File]::Exists($storage.operation_path)) {
        throw "-ProjectRoot must not already exist so the generated scale project has one immutable input identity: $($storage.display_path)"
    }
    return [pscustomobject]@{
        OperationalPath = $storage.operation_path
        DisplayPath = $storage.display_path
        StoragePolicy = $storage
    }
}

function Write-RenderExtractScaleFileNew {
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
            throw "Refusing to overwrite generated scale-project file: $Path"
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

function Write-RenderExtractScaleScene {
    param(
        [Parameter(Mandatory)][IO.TextWriter]$Writer,
        [Parameter(Mandatory)][int]$PrimitiveCount
    )

    $lineEnding = [Environment]::NewLine
    $header = @(
        '[[entities]]',
        'entity = 1',
        'name = "Camera"',
        'parent = 0',
        'active = true',
        'render_layer_mask = 1',
        'transform = { translation = [21.0, 2.0, 14.5], rotation = [0.0, 0.0, 0.0, 1.0], scale = [1.0, 1.0, 1.0] }',
        'camera = { fov_y_radians = 1.7453293, z_near = 0.1, z_far = 2000.0 }',
        '',
        '[[entities]]',
        'entity = 2',
        'name = "Sun"',
        'parent = 0',
        'active = true',
        'render_layer_mask = 1',
        'mobility = "Static"',
        'transform = { translation = [0.0, 400.0, 0.0], rotation = [0.0, 0.0, 0.0, 1.0], scale = [1.0, 1.0, 1.0] }',
        'directional_light = { direction = [-0.361772505316908, -0.904431263292269, -0.226107815823067], color = [1.0, 1.0, 1.0], intensity = 3.0 }'
    ) -join $lineEnding
    $Writer.Write($header)

    $entityFormat = '{0}{0}[[entities]]{0}entity = {1}{0}name = "Cube_{2:D6}"{0}parent = 0{0}active = true{0}render_layer_mask = 1{0}mobility = "Static"{0}transform = {{ translation = [{3}, 0.0, {4}], rotation = [0.0, 0.0, 0.0, 1.0], scale = [1.0, 1.0, 1.0] }}{0}{0}[entities.mesh.model]{0}kind = "project"{0}guid = "00000000-0000-0000-0000-000000000002"{0}path_hint = "assets/models/cube.obj"{0}{0}[entities.mesh.material]{0}kind = "project"{0}guid = "00000000-0000-0000-0000-000000000003"{0}path_hint = "assets/materials/default.zmaterial"'

    $gridWidth = [int][Math]::Ceiling([Math]::Sqrt($PrimitiveCount))
    $invariantCulture = [Globalization.CultureInfo]::InvariantCulture
    for ($primitiveIndex = 0; $primitiveIndex -lt $PrimitiveCount; $primitiveIndex++) {
        $column = $primitiveIndex % $gridWidth
        $row = [int][Math]::Floor($primitiveIndex / $gridWidth)
        $x = ($column - (($gridWidth - 1) / 2.0)) * 2.0
        $z = ($row - (($gridWidth - 1) / 2.0)) * 2.0
        $xText = $x.ToString('0.0', $invariantCulture)
        $zText = $z.ToString('0.0', $invariantCulture)

        $Writer.Write(($entityFormat -f @(
                    $lineEnding,
                    ($primitiveIndex + 3),
                    ($primitiveIndex + 1),
                    $xText,
                    $zText
                )))
    }
    $Writer.WriteLine()
}

function Get-RenderExtractScaleSceneContent {
    param([Parameter(Mandatory)][int]$PrimitiveCount)

    $scene = [Text.StringBuilder]::new()
    $writer = [IO.StringWriter]::new($scene, [Globalization.CultureInfo]::InvariantCulture)
    try {
        Write-RenderExtractScaleScene -Writer $writer -PrimitiveCount $PrimitiveCount
        return $scene.ToString()
    }
    finally {
        $writer.Dispose()
    }
}

function Write-RenderExtractScaleSceneFileNew {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][int]$PrimitiveCount
    )

    [IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($Path)) | Out-Null
    $stream = $null
    $writer = $null
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
            throw "Refusing to overwrite generated scale-project scene: $Path"
        }
        $writer = [IO.StreamWriter]::new($stream, [Text.UTF8Encoding]::new($false), 65536, $true)
        Write-RenderExtractScaleScene -Writer $writer -PrimitiveCount $PrimitiveCount
        $writer.Flush()
        $stream.Flush($true)
    }
    finally {
        if ($null -ne $writer) {
            $writer.Dispose()
        }
        if ($null -ne $stream) {
            $stream.Dispose()
        }
    }
}

function Copy-RenderExtractScaleTemplate {
    param(
        [Parameter(Mandatory)]$BuildSet,
        [Parameter(Mandatory)][string]$DestinationRoot
    )

    $templateRootRelativePath = 'templates/projects/renderable-empty'
    $templatePrefix = $templateRootRelativePath + '/'
    $generatedSceneRelativePath = 'assets/scenes/main.scene.toml'
    $templateFiles = @($BuildSet.files | Where-Object {
            ([string]$_.relative_path).StartsWith($templatePrefix, [StringComparison]::Ordinal)
        })
    if ($templateFiles.Count -eq 0) {
        throw "Render-extract scale template is absent from BuildSet $($BuildSet.build_set_id)."
    }
    foreach ($file in $templateFiles) {
        $buildSetRelativePath = [string]$file.relative_path
        $relativePath = $buildSetRelativePath.Substring($templatePrefix.Length)
        if ($relativePath.Equals($generatedSceneRelativePath, [StringComparison]::OrdinalIgnoreCase)) {
            continue
        }
        $sourceFile = [IO.Path]::Combine(
            [string]$BuildSet.snapshot_root,
            $buildSetRelativePath.Replace('/', [IO.Path]::DirectorySeparatorChar)
        )
        $destinationPath = [IO.Path]::Combine(
            $DestinationRoot,
            $relativePath.Replace('/', [IO.Path]::DirectorySeparatorChar)
        )
        Write-RenderExtractScaleFileNew -Path $destinationPath -Bytes ([IO.File]::ReadAllBytes($sourceFile))
    }
}

function New-RenderExtractScaleProject {
    param(
        [Parameter(Mandatory)][string]$ProjectRoot,
        [Parameter(Mandatory)][int]$PrimitiveCount,
        [Parameter(Mandatory)][string]$ProfilingInputManifestPath
    )

    $sourceIdentity = Resolve-RenderExtractProfilingSourceIdentity `
        -ManifestPath $ProfilingInputManifestPath
    $projectResolution = Assert-RenderExtractScaleProjectDirectory -Path $ProjectRoot
    $templateRoot = Join-ZirconWindowsPath `
        -Path $sourceIdentity.build_set.snapshot_root `
        -ChildPath 'templates\projects\renderable-empty'
    if (-not [IO.Directory]::Exists($templateRoot)) {
        throw "Render-extract scale template does not exist in BuildSet $($sourceIdentity.build_set_id)."
    }

    $destinationParent = [IO.Path]::GetDirectoryName($projectResolution.OperationalPath)
    [IO.Directory]::CreateDirectory($destinationParent) | Out-Null
    $partialProjectRoot = "$($projectResolution.OperationalPath).partial-$([guid]::NewGuid().ToString('N'))"
    if ([IO.Directory]::Exists($partialProjectRoot) -or [IO.File]::Exists($partialProjectRoot)) {
        throw "Generated scale-project temporary directory already exists: $partialProjectRoot"
    }

    try {
        [IO.Directory]::CreateDirectory($partialProjectRoot) | Out-Null
        Copy-RenderExtractScaleTemplate `
            -BuildSet $sourceIdentity.build_set `
            -DestinationRoot $partialProjectRoot
        $scenePath = [IO.Path]::Combine($partialProjectRoot, 'assets\scenes\main.scene.toml')
        Write-RenderExtractScaleSceneFileNew -Path $scenePath -PrimitiveCount $PrimitiveCount

        $manifest = [ordered]@{
            schema_version = 2
            source_fingerprint = $sourceIdentity.build_set_id
            build_set_id = $sourceIdentity.build_set_id
            primitive_count = $PrimitiveCount
            scene_virtual_path = 'res://scenes/main.scene.toml'
            model_virtual_path = 'assets/models/cube.obj'
            material_virtual_path = 'assets/materials/default.zmaterial'
        }
        $manifestBytes = [Text.UTF8Encoding]::new($false).GetBytes(($manifest | ConvertTo-Json -Depth 3))
        Write-RenderExtractScaleFileNew `
            -Path ([IO.Path]::Combine($partialProjectRoot, 'render-extract-scale-project.json')) `
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
        project_root = $projectResolution.DisplayPath
        build_set_id = $sourceIdentity.build_set_id
        primitive_count = $PrimitiveCount
        scene_virtual_path = 'res://scenes/main.scene.toml'
        manifest_path = (Resolve-ZirconWindowsPath -Path ([IO.Path]::Combine(
                    $projectResolution.OperationalPath,
                    'render-extract-scale-project.json'
                ))).DisplayPath
    }
}

if ($env:RENDER_EXTRACT_SCALE_PROJECT_TEST_MODE -ne '1') {
    if ([string]::IsNullOrWhiteSpace($ProfilingInputManifestPath)) {
        throw '-ProfilingInputManifestPath is required to bind the scale project to its BuildSet.'
    }
    New-RenderExtractScaleProject `
        -ProjectRoot $ProjectRoot `
        -PrimitiveCount $PrimitiveCount `
        -ProfilingInputManifestPath $ProfilingInputManifestPath
}
