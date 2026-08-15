[CmdletBinding()]
param(
    [string]$ProjectRoot = (Join-Path 'E:\ZirconBuilds\mvp-perf-projects' ([guid]::NewGuid().ToString('N'))),
    [ValidateRange(1, 100000)]
    [int]$PrimitiveCount = 1
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
Import-Module (Join-Path $PSScriptRoot 'MvpProductInputManifest.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $repoRoot 'tools\WindowsPathResolver.psm1') -Force -ErrorAction Stop

function Assert-RenderExtractScaleProjectDirectory {
    param([Parameter(Mandatory)][string]$Path)

    $resolution = Resolve-ZirconWindowsPath -Path $Path
    if ($resolution.DisplayPath -notmatch '^E:\\ZirconBuilds\\mvp-perf-projects\\(?:[A-Za-z0-9][A-Za-z0-9._-]*)(?:\\|$)') {
        throw "-ProjectRoot scale project must resolve under E:\ZirconBuilds\mvp-perf-projects\<session>: $($resolution.DisplayPath)"
    }
    if ([IO.Directory]::Exists($resolution.OperationalPath) -or [IO.File]::Exists($resolution.OperationalPath)) {
        throw "-ProjectRoot must not already exist so the generated scale project has one immutable input identity: $($resolution.DisplayPath)"
    }
    return $resolution
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
        [Parameter(Mandatory)][string]$TemplateRoot,
        [Parameter(Mandatory)][string]$DestinationRoot
    )

    $templateRootPrefix = $TemplateRoot.TrimEnd([char[]]@('\', '/')) + [IO.Path]::DirectorySeparatorChar
    $generatedSceneRelativePath = 'assets/scenes/main.scene.toml'
    foreach ($sourceFile in [IO.Directory]::EnumerateFiles($TemplateRoot, '*', [IO.SearchOption]::AllDirectories)) {
        if (-not $sourceFile.StartsWith($templateRootPrefix, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Template file escaped its declared root: $sourceFile"
        }
        $relativePath = $sourceFile.Substring($templateRootPrefix.Length).Replace('\', '/')
        if ($relativePath.Equals($generatedSceneRelativePath, [StringComparison]::OrdinalIgnoreCase)) {
            continue
        }
        $destinationPath = Join-Path $DestinationRoot $relativePath
        Write-RenderExtractScaleFileNew -Path $destinationPath -Bytes ([IO.File]::ReadAllBytes($sourceFile))
    }
}

function New-RenderExtractScaleProject {
    param(
        [Parameter(Mandatory)][string]$ProjectRoot,
        [Parameter(Mandatory)][int]$PrimitiveCount,
        [Parameter(Mandatory)][string]$SourceFingerprint
    )

    if ($SourceFingerprint -notmatch '^[0-9A-F]{64}$') {
        throw 'Render-extract scale project source fingerprint must be an uppercase SHA-256 value.'
    }
    $projectResolution = Assert-RenderExtractScaleProjectDirectory -Path $ProjectRoot
    $templateRoot = (Resolve-ZirconWindowsPath -Path (Join-Path $repoRoot 'templates\projects\renderable-empty')).OperationalPath
    if (-not [IO.Directory]::Exists($templateRoot)) {
        throw "Render-extract scale template does not exist: $templateRoot"
    }

    $destinationParent = [IO.Path]::GetDirectoryName($projectResolution.OperationalPath)
    [IO.Directory]::CreateDirectory($destinationParent) | Out-Null
    $partialProjectRoot = "$($projectResolution.OperationalPath).partial-$([guid]::NewGuid().ToString('N'))"
    if ([IO.Directory]::Exists($partialProjectRoot) -or [IO.File]::Exists($partialProjectRoot)) {
        throw "Generated scale-project temporary directory already exists: $partialProjectRoot"
    }

    try {
        [IO.Directory]::CreateDirectory($partialProjectRoot) | Out-Null
        Copy-RenderExtractScaleTemplate -TemplateRoot $templateRoot -DestinationRoot $partialProjectRoot
        $scenePath = Join-Path $partialProjectRoot 'assets\scenes\main.scene.toml'
        Write-RenderExtractScaleSceneFileNew -Path $scenePath -PrimitiveCount $PrimitiveCount

        $manifest = [ordered]@{
            schema_version = 1
            source_fingerprint = $SourceFingerprint
            primitive_count = $PrimitiveCount
            scene_virtual_path = 'res://scenes/main.scene.toml'
            model_virtual_path = 'assets/models/cube.obj'
            material_virtual_path = 'assets/materials/default.zmaterial'
        }
        $manifestBytes = [Text.UTF8Encoding]::new($false).GetBytes(($manifest | ConvertTo-Json -Depth 3))
        Write-RenderExtractScaleFileNew `
            -Path (Join-Path $partialProjectRoot 'render-extract-scale-project.json') `
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
        primitive_count = $PrimitiveCount
        scene_virtual_path = 'res://scenes/main.scene.toml'
        manifest_path = (Resolve-ZirconWindowsPath -Path (Join-Path $projectResolution.OperationalPath 'render-extract-scale-project.json')).DisplayPath
    }
}

if ($env:RENDER_EXTRACT_SCALE_PROJECT_TEST_MODE -ne '1') {
    $sourceFingerprint = Get-MvpSourceFingerprint -RepositoryRoot $repoRoot
    New-RenderExtractScaleProject `
        -ProjectRoot $ProjectRoot `
        -PrimitiveCount $PrimitiveCount `
        -SourceFingerprint $sourceFingerprint
}
