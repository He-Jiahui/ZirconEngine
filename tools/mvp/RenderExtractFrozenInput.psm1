Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$moduleRepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
Import-Module (Join-Path $PSScriptRoot 'MvpProductInputManifest.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $moduleRepoRoot 'tools\maintenance\WindowsPathResolver.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'RenderExtractSourceIdentity.psm1') -Force -ErrorAction Stop

function Get-RenderExtractProfilingArtifact {
    param(
        [Parameter(Mandatory)]$Artifacts,
        [Parameter(Mandatory)][string]$LogicalId,
        [Parameter(Mandatory)][string]$ExpectedProduct,
        [Parameter(Mandatory)][string]$ExpectedPackage,
        [AllowNull()][string]$ExpectedBin,
        [Parameter(Mandatory)][string]$ExpectedFeatures
    )

    $matches = @($Artifacts | Where-Object {
            [string](Get-RenderExtractManifestProperty -Value $_ -Name 'logical_id' -Label 'Profiling input artifact') -eq $LogicalId
        })
    if ($matches.Count -ne 1) {
        throw "Profiling input must contain exactly one '$LogicalId' artifact; found $($matches.Count)."
    }
    $artifact = $matches[0]
    $product = [string](Get-RenderExtractManifestProperty -Value $artifact -Name 'product' -Label $LogicalId)
    $package = [string](Get-RenderExtractManifestProperty -Value $artifact -Name 'package' -Label $LogicalId)
    $features = [string](Get-RenderExtractManifestProperty -Value $artifact -Name 'features' -Label $LogicalId)
    if ($product -ne $ExpectedProduct -or $package -ne $ExpectedPackage -or $features -ne $ExpectedFeatures) {
        throw "Profiling input artifact '$LogicalId' does not match the declared $ExpectedProduct profiling contract."
    }
    $binProperty = $artifact.PSObject.Properties['bin']
    $actualBin = if ($null -eq $binProperty -or $null -eq $binProperty.Value) { $null } else { [string]$binProperty.Value }
    $binMatches = if ([string]::IsNullOrEmpty($ExpectedBin)) {
        [string]::IsNullOrEmpty($actualBin)
    }
    else {
        $actualBin -eq $ExpectedBin
    }
    if (-not $binMatches) {
        throw "Profiling input artifact '$LogicalId' has unexpected bin '$actualBin'."
    }

    $path = [string](Get-RenderExtractManifestProperty -Value $artifact -Name 'path' -Label $LogicalId)
    $resolution = Resolve-ZirconWindowsPath -Path $path
    if (-not [IO.File]::Exists($resolution.OperationalPath)) {
        throw "Profiling input artifact '$LogicalId' does not exist: $($resolution.DisplayPath)"
    }
    $expectedBytes = [Int64](Get-RenderExtractManifestProperty -Value $artifact -Name 'bytes' -Label $LogicalId)
    $actualBytes = [IO.FileInfo]::new($resolution.OperationalPath).Length
    if ($actualBytes -ne $expectedBytes) {
        throw "Profiling input artifact '$LogicalId' byte length changed from $expectedBytes to $actualBytes."
    }
    $expectedHash = [string](Get-RenderExtractManifestProperty -Value $artifact -Name 'sha256' -Label $LogicalId)
    $actualHash = Get-MvpProductInputFileSha256 -Path $resolution.OperationalPath
    if (-not $actualHash.Equals($expectedHash, [StringComparison]::Ordinal)) {
        throw "Profiling input artifact '$LogicalId' SHA-256 no longer matches its manifest."
    }
    return [pscustomobject]@{
        OperationalPath = $resolution.OperationalPath
        DisplayPath = $resolution.DisplayPath
        Sha256 = $actualHash
    }
}

function Resolve-RenderExtractProfilingInput {
    param([Parameter(Mandatory)][string]$ManifestPath)

    $sourceIdentity = Resolve-RenderExtractProfilingSourceIdentity -ManifestPath $ManifestPath
    $manifest = $sourceIdentity.manifest
    $manifestDirectory = $sourceIdentity.manifest_directory
    $artifacts = @(Get-RenderExtractManifestProperty -Value $manifest -Name 'artifacts' -Label 'Profiling input manifest')
    if ($artifacts.Count -ne 4) {
        throw "Profiling input manifest must contain exactly four runtime/editor artifacts; found $($artifacts.Count)."
    }
    $runtimeFeatures = 'target-client,platform-winit,input-gamepad,gamepad-gilrs,profiling'
    $editorFeatures = 'target-editor-host,profiling'
    $runtimeExecutable = Get-RenderExtractProfilingArtifact `
        -Artifacts $artifacts `
        -LogicalId 'runtime-profile-executable' `
        -ExpectedProduct 'runtime' `
        -ExpectedPackage 'zircon_app' `
        -ExpectedBin 'zircon_runtime' `
        -ExpectedFeatures $runtimeFeatures
    $runtimeLibrary = Get-RenderExtractProfilingArtifact `
        -Artifacts $artifacts `
        -LogicalId 'runtime-profile-library' `
        -ExpectedProduct 'runtime' `
        -ExpectedPackage 'zircon_runtime' `
        -ExpectedBin $null `
        -ExpectedFeatures $runtimeFeatures
    $editorExecutable = Get-RenderExtractProfilingArtifact `
        -Artifacts $artifacts `
        -LogicalId 'editor-profile-executable' `
        -ExpectedProduct 'editor' `
        -ExpectedPackage 'zircon_app' `
        -ExpectedBin 'zircon_editor' `
        -ExpectedFeatures $editorFeatures
    $editorLibrary = Get-RenderExtractProfilingArtifact `
        -Artifacts $artifacts `
        -LogicalId 'editor-profile-library' `
        -ExpectedProduct 'editor' `
        -ExpectedPackage 'zircon_runtime' `
        -ExpectedBin $null `
        -ExpectedFeatures $editorFeatures
    foreach ($product in @(
            [pscustomobject]@{ Name = 'runtime'; Executable = $runtimeExecutable; Library = $runtimeLibrary },
            [pscustomobject]@{ Name = 'editor'; Executable = $editorExecutable; Library = $editorLibrary }
        )) {
        $executableDirectory = [IO.Path]::GetDirectoryName($product.Executable.OperationalPath)
        $libraryDirectory = [IO.Path]::GetDirectoryName($product.Library.OperationalPath)
        if (-not $executableDirectory.Equals($libraryDirectory, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Profiling input $($product.Name) executable and runtime library must be in the same directory for relative runtime loading."
        }
        $expectedDirectory = Join-ZirconWindowsPath -Path $manifestDirectory -ChildPath $product.Name
        if (-not $executableDirectory.Equals($expectedDirectory, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Profiling input $($product.Name) pair must live in its managed product directory under the manifest."
        }
    }
    return [pscustomobject]@{
        manifest_path = $sourceIdentity.manifest_path
        manifest_sha256 = $sourceIdentity.manifest_sha256
        source_fingerprint = $sourceIdentity.source_fingerprint
        build_set_id = $sourceIdentity.build_set_id
        build_set_manifest_sha256 = $sourceIdentity.build_set_manifest_sha256
        runtime = [pscustomobject]@{
            executable_path = $runtimeExecutable.OperationalPath
            executable_sha256 = $runtimeExecutable.Sha256
            library_path = $runtimeLibrary.OperationalPath
            library_sha256 = $runtimeLibrary.Sha256
        }
        editor = [pscustomobject]@{
            executable_path = $editorExecutable.OperationalPath
            executable_sha256 = $editorExecutable.Sha256
            library_path = $editorLibrary.OperationalPath
            library_sha256 = $editorLibrary.Sha256
        }
    }
}

function Assert-RenderExtractProfilingInputIdentity {
    param(
        [Parameter(Mandatory)]$Expected,
        [Parameter(Mandatory)]$Actual
    )

    foreach ($name in @('manifest_path')) {
        if (-not ([string]$Expected.$name).Equals([string]$Actual.$name, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Profiling input identity changed during baseline capture ('$name')."
        }
    }
    foreach ($name in @('manifest_sha256', 'build_set_id', 'build_set_manifest_sha256')) {
        if (-not ([string]$Expected.$name).Equals([string]$Actual.$name, [StringComparison]::Ordinal)) {
            throw "Profiling input identity changed during baseline capture ('$name')."
        }
    }
    foreach ($product in @('runtime', 'editor')) {
        foreach ($name in @('executable_path', 'library_path')) {
            if (-not ([string]$Expected.$product.$name).Equals([string]$Actual.$product.$name, [StringComparison]::OrdinalIgnoreCase)) {
                throw "Profiling input identity changed during baseline capture ('$product.$name')."
            }
        }
        foreach ($name in @('executable_sha256', 'library_sha256')) {
            if (-not ([string]$Expected.$product.$name).Equals([string]$Actual.$product.$name, [StringComparison]::Ordinal)) {
                throw "Profiling input identity changed during baseline capture ('$product.$name')."
            }
        }
    }
}

function Get-RenderExtractBytesSha256 {
    param([Parameter(Mandatory)][byte[]]$Bytes)

    $hasher = [Security.Cryptography.SHA256]::Create()
    try {
        return -join ($hasher.ComputeHash($Bytes) | ForEach-Object { $_.ToString('X2') })
    }
    finally {
        $hasher.Dispose()
    }
}

function Write-RenderExtractFrozenBytesNew {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][AllowEmptyCollection()][byte[]]$Bytes,
        [Parameter(Mandatory)][string]$Label
    )

    try {
        $stream = [IO.FileStream]::new(
            $Path,
            [IO.FileMode]::CreateNew,
            [IO.FileAccess]::Write,
            [IO.FileShare]::None
        )
    }
    catch [IO.IOException] {
        throw "Refusing to overwrite frozen render-extract input ${Label}: $Path"
    }
    try {
        $stream.Write($Bytes, 0, $Bytes.Length)
        $stream.Flush($true)
    }
    finally {
        $stream.Dispose()
    }
}

function Copy-RenderExtractFrozenInputFile {
    param(
        [Parameter(Mandatory)][string]$SourcePath,
        [Parameter(Mandatory)][string]$DestinationPath,
        [Parameter(Mandatory)][string]$ExpectedSha256,
        [Parameter(Mandatory)][string]$Label
    )

    if (-not [IO.File]::Exists($SourcePath)) {
        throw "Profiling input $Label disappeared before the capture snapshot was frozen: $SourcePath"
    }
    [byte[]]$bytes = [IO.File]::ReadAllBytes($SourcePath)
    $actualSha256 = Get-RenderExtractBytesSha256 -Bytes $bytes
    if (-not $actualSha256.Equals($ExpectedSha256, [StringComparison]::Ordinal)) {
        throw "Profiling input $Label changed before the capture snapshot was frozen."
    }
    Write-RenderExtractFrozenBytesNew -Path $DestinationPath -Bytes $bytes -Label $Label
}

function Get-RenderExtractAssetFiles {
    param([Parameter(Mandatory)][string]$Root)

    $rootResolution = Resolve-ZirconWindowsPath -Path $Root
    if (-not [IO.Directory]::Exists($rootResolution.OperationalPath)) {
        throw "Render-extract engine asset root does not exist: $($rootResolution.DisplayPath)"
    }
    $files = [Collections.Generic.List[object]]::new()
    $pending = [Collections.Generic.Stack[string]]::new()
    $directorySeparator = [string][IO.Path]::DirectorySeparatorChar
    $alternateDirectorySeparator = [string][IO.Path]::AltDirectorySeparatorChar
    $rootPrefix = if ($rootResolution.OperationalPath.EndsWith($directorySeparator) -or
        $rootResolution.OperationalPath.EndsWith($alternateDirectorySeparator)) {
        $rootResolution.OperationalPath
    }
    else {
        $rootResolution.OperationalPath + $directorySeparator
    }
    $pending.Push($rootResolution.OperationalPath)
    while ($pending.Count -gt 0) {
        $directory = $pending.Pop()
        $directoryInfo = [IO.DirectoryInfo]::new($directory)
        if (($directoryInfo.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
            throw "Render-extract engine asset root must not contain reparse directories: $directory"
        }
        foreach ($child in [IO.Directory]::GetDirectories($directory)) {
            $pending.Push($child)
        }
        foreach ($path in [IO.Directory]::GetFiles($directory)) {
            $fileInfo = [IO.FileInfo]::new($path)
            if (($fileInfo.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
                throw "Render-extract engine asset root must not contain reparse files: $path"
            }
            if (-not $path.StartsWith($rootPrefix, [StringComparison]::OrdinalIgnoreCase)) {
                throw "Render-extract engine asset escaped its resolved root: $path"
            }
            # Windows PowerShell 5.1 has no Path.GetRelativePath; the no-reparse traversal above
            # makes a resolver-owned prefix removal sufficient and deterministic here.
            $relative = $path.Substring($rootPrefix.Length).Replace('\', '/')
            $files.Add([pscustomobject]@{
                    relative_path = $relative
                    source_path = $path
                    bytes = [Int64]$fileInfo.Length
                    sha256 = Get-MvpProductInputFileSha256 -Path $path
                }) | Out-Null
        }
    }
    return @($files | Sort-Object relative_path)
}

function Get-RenderExtractMergedAssetFiles {
    param([Parameter(Mandatory)][string[]]$Roots)

    $merged = @{}
    foreach ($root in $Roots) {
        foreach ($file in @(Get-RenderExtractAssetFiles -Root $root)) {
            $relativePath = [string]$file.relative_path
            if (-not $merged.ContainsKey($relativePath)) {
                $merged[$relativePath] = $file
                continue
            }
            $existing = $merged[$relativePath]
            if ([Int64]$existing.bytes -ne [Int64]$file.bytes -or
                -not ([string]$existing.sha256).Equals([string]$file.sha256, [StringComparison]::Ordinal)) {
                throw "Render-extract engine asset roots contain conflicting file '$relativePath'."
            }
        }
    }
    return @($merged.Values | Sort-Object relative_path)
}

function Copy-RenderExtractFrozenAssetTree {
    param(
        [Parameter(Mandatory)][object[]]$SourceFiles,
        [Parameter(Mandatory)][string]$ProductDirectory,
        [Parameter(Mandatory)][string]$Product
    )

    $assetRoot = Join-ZirconWindowsPath -Path $ProductDirectory -ChildPath 'assets'
    [IO.Directory]::CreateDirectory($assetRoot) | Out-Null
    $records = [Collections.Generic.List[object]]::new()
    [Int64]$totalBytes = 0
    foreach ($source in $SourceFiles) {
        $destination = Join-ZirconWindowsPath `
            -Path $assetRoot `
            -ChildPath ([string]$source.relative_path).Replace('/', '\')
        [IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($destination)) | Out-Null
        Copy-RenderExtractFrozenInputFile `
            -SourcePath $source.source_path `
            -DestinationPath $destination `
            -ExpectedSha256 $source.sha256 `
            -Label "$Product asset '$($source.relative_path)'"
        $records.Add([ordered]@{
                relative_path = $source.relative_path
                bytes = [Int64]$source.bytes
                sha256 = $source.sha256
            }) | Out-Null
        $totalBytes += [Int64]$source.bytes
    }
    $manifest = [ordered]@{
        schema_version = 1
        files = @($records)
    }
    [byte[]]$manifestBytes = [Text.UTF8Encoding]::new($false).GetBytes(
        ($manifest | ConvertTo-Json -Depth 4)
    )
    $manifestPath = Join-ZirconWindowsPath -Path $ProductDirectory -ChildPath 'asset-manifest.json'
    Write-RenderExtractFrozenBytesNew `
        -Path $manifestPath `
        -Bytes $manifestBytes `
        -Label "$Product asset manifest"
    return [pscustomobject]@{
        asset_root_path = $assetRoot
        asset_manifest_path = $manifestPath
        asset_manifest_sha256 = Get-RenderExtractBytesSha256 -Bytes $manifestBytes
        asset_files = @($records)
        asset_file_count = $records.Count
        asset_bytes = $totalBytes
    }
}

function New-RenderExtractFrozenProfilingInput {
    param(
        [Parameter(Mandatory)]$ProfilingInput,
        [Parameter(Mandatory)][string[]]$EngineAssetRoots,
        [Parameter(Mandatory)][string]$OutputDirectory,
        [Parameter(Mandatory)][string]$InvocationId
    )

    if ($InvocationId -notmatch '^[0-9A-Fa-f]{32}$') {
        throw "Render-extract capture invocation id is invalid: $InvocationId"
    }
    $snapshotDirectory = Join-ZirconWindowsPath `
        -Path (Join-ZirconWindowsPath -Path $OutputDirectory -ChildPath 'inputs') `
        -ChildPath $InvocationId
    if ([IO.Directory]::Exists($snapshotDirectory)) {
        throw "Frozen render-extract input directory already exists: $snapshotDirectory"
    }
    $sourceAssetFiles = @(Get-RenderExtractMergedAssetFiles -Roots $EngineAssetRoots)
    if ($sourceAssetFiles.Count -eq 0) {
        throw 'Render-extract engine asset roots have no files.'
    }
    [IO.Directory]::CreateDirectory($snapshotDirectory) | Out-Null
    $frozenManifestPath = Join-ZirconWindowsPath -Path $snapshotDirectory -ChildPath 'render-extract-profiling-inputs.json'
    Copy-RenderExtractFrozenInputFile `
        -SourcePath $ProfilingInput.manifest_path `
        -DestinationPath $frozenManifestPath `
        -ExpectedSha256 $ProfilingInput.manifest_sha256 `
        -Label 'manifest'
    $frozen = [ordered]@{
        manifest_path = $frozenManifestPath
        manifest_sha256 = $ProfilingInput.manifest_sha256
        build_set_id = $ProfilingInput.build_set_id
        build_set_manifest_sha256 = $ProfilingInput.build_set_manifest_sha256
    }
    foreach ($product in @('runtime', 'editor')) {
        $productDirectory = Join-ZirconWindowsPath -Path $snapshotDirectory -ChildPath $product
        [IO.Directory]::CreateDirectory($productDirectory) | Out-Null
        $executableName = if ($product -eq 'runtime') { 'zircon_runtime.exe' } else { 'zircon_editor.exe' }
        $frozenExecutablePath = Join-ZirconWindowsPath -Path $productDirectory -ChildPath $executableName
        $frozenLibraryPath = Join-ZirconWindowsPath -Path $productDirectory -ChildPath 'zircon_runtime.dll'
        Copy-RenderExtractFrozenInputFile `
            -SourcePath $ProfilingInput.$product.executable_path `
            -DestinationPath $frozenExecutablePath `
            -ExpectedSha256 $ProfilingInput.$product.executable_sha256 `
            -Label "$product executable"
        Copy-RenderExtractFrozenInputFile `
            -SourcePath $ProfilingInput.$product.library_path `
            -DestinationPath $frozenLibraryPath `
            -ExpectedSha256 $ProfilingInput.$product.library_sha256 `
            -Label "$product library"
        $assetInput = Copy-RenderExtractFrozenAssetTree `
            -SourceFiles $sourceAssetFiles `
            -ProductDirectory $productDirectory `
            -Product $product
        $frozen[$product] = [pscustomobject]@{
            executable_path = $frozenExecutablePath
            executable_sha256 = $ProfilingInput.$product.executable_sha256
            library_path = $frozenLibraryPath
            library_sha256 = $ProfilingInput.$product.library_sha256
            asset_root_path = $assetInput.asset_root_path
            asset_manifest_path = $assetInput.asset_manifest_path
            asset_manifest_sha256 = $assetInput.asset_manifest_sha256
            asset_files = $assetInput.asset_files
            asset_file_count = $assetInput.asset_file_count
            asset_bytes = $assetInput.asset_bytes
        }
    }
    return [pscustomobject]$frozen
}

function Assert-RenderExtractFrozenProductInput {
    param(
        [Parameter(Mandatory)]$ProductInput,
        [Parameter(Mandatory)][ValidateSet('runtime', 'editor')][string]$Product
    )

    $actual = [ordered]@{}
    foreach ($artifact in @(
            [pscustomobject]@{
                path = $ProductInput.executable_path
                expected_sha256 = $ProductInput.executable_sha256
                property_name = 'executable_sha256'
                label = 'executable'
            },
            [pscustomobject]@{
                path = $ProductInput.library_path
                expected_sha256 = $ProductInput.library_sha256
                property_name = 'library_sha256'
                label = 'runtime library'
            }
        )) {
        if (-not [IO.File]::Exists($artifact.path)) {
            throw "Frozen render-extract $Product $($artifact.label) disappeared before process launch: $($artifact.path)"
        }
        $actualSha256 = Get-MvpProductInputFileSha256 -Path $artifact.path
        if (-not $actualSha256.Equals([string]$artifact.expected_sha256, [StringComparison]::Ordinal)) {
            throw "Frozen render-extract $Product $($artifact.label) changed before process launch."
        }
        $actual[$artifact.property_name] = $actualSha256
    }

    $assetManifestPath = $ProductInput.asset_manifest_path
    if (-not [IO.File]::Exists($assetManifestPath)) {
        throw "Frozen render-extract $Product asset manifest disappeared before process launch: $assetManifestPath"
    }
    $assetManifestSha256 = Get-MvpProductInputFileSha256 -Path $assetManifestPath
    if (-not $assetManifestSha256.Equals([string]$ProductInput.asset_manifest_sha256, [StringComparison]::Ordinal)) {
        throw "Frozen render-extract $Product asset manifest changed before process launch."
    }
    $actual.asset_manifest_sha256 = $assetManifestSha256

    $actualFiles = @(Get-RenderExtractAssetFiles -Root $ProductInput.asset_root_path)
    $expectedFiles = @($ProductInput.asset_files)
    if ($actualFiles.Count -ne $expectedFiles.Count) {
        throw "Frozen render-extract $Product asset inventory changed before process launch."
    }
    [Int64]$assetBytes = 0
    for ($index = 0; $index -lt $expectedFiles.Count; $index++) {
        $expected = $expectedFiles[$index]
        $observed = $actualFiles[$index]
        if (-not ([string]$observed.relative_path).Equals([string]$expected.relative_path, [StringComparison]::Ordinal) -or
            [Int64]$observed.bytes -ne [Int64]$expected.bytes -or
            -not ([string]$observed.sha256).Equals([string]$expected.sha256, [StringComparison]::Ordinal)) {
            throw "Frozen render-extract $Product asset inventory changed before process launch at '$($expected.relative_path)'."
        }
        $assetBytes += [Int64]$observed.bytes
    }
    $actual.asset_file_count = $actualFiles.Count
    $actual.asset_bytes = $assetBytes
    return [pscustomobject]$actual
}

Export-ModuleMember -Function @(
    'Resolve-RenderExtractProfilingInput',
    'Assert-RenderExtractProfilingInputIdentity',
    'New-RenderExtractFrozenProfilingInput',
    'Assert-RenderExtractFrozenProductInput'
)
