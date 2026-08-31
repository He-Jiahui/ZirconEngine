[CmdletBinding()]
param(
    [string]$ArtifactOutputDirectory
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
Import-Module (Join-Path $PSScriptRoot 'MvpProductInputManifest.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'MvpArtifactStoragePolicy.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'MvpBuildSet.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $repoRoot 'tools\WindowsPathResolver.psm1') -Force -ErrorAction Stop
if ([string]::IsNullOrWhiteSpace($ArtifactOutputDirectory)) {
    $ArtifactOutputDirectory = New-MvpArtifactStoragePath -NamespaceId 'render-extract-profiling-inputs'
}

function Get-RenderExtractProfilingBuildRequests {
    $runtimeFeatures = 'target-client,platform-winit,input-gamepad,gamepad-gilrs,profiling'
    $editorFeatures = 'target-editor-host,profiling'
    return @(
        [pscustomobject]@{
            logical_id   = 'runtime-profile-executable'
            Product      = 'runtime'
            Package      = 'zircon_app'
            Bin          = 'zircon_runtime'
            Features     = $runtimeFeatures
            CargoProfile = 'profiling'
            ArtifactName = 'zircon_runtime.exe'
        },
        [pscustomobject]@{
            logical_id   = 'runtime-profile-library'
            Product      = 'runtime'
            Package      = 'zircon_runtime'
            Bin          = $null
            Features     = $runtimeFeatures
            CargoProfile = 'profiling'
            ArtifactName = 'zircon_runtime.dll'
        },
        [pscustomobject]@{
            logical_id   = 'editor-profile-executable'
            Product      = 'editor'
            Package      = 'zircon_app'
            Bin          = 'zircon_editor'
            Features     = $editorFeatures
            CargoProfile = 'profiling'
            ArtifactName = 'zircon_editor.exe'
        },
        [pscustomobject]@{
            logical_id   = 'editor-profile-library'
            Product      = 'editor'
            Package      = 'zircon_runtime'
            Bin          = $null
            Features     = $editorFeatures
            CargoProfile = 'profiling'
            ArtifactName = 'zircon_runtime.dll'
        }
    )
}

function Assert-RenderExtractProfilingInputDirectory {
    param(
        [Parameter(Mandatory)]
        [string]$Path
    )

    $resolution = Resolve-MvpArtifactStoragePath `
        -Path $Path `
        -NamespaceId 'render-extract-profiling-inputs'
    $resolvedPath = $resolution.operation_path
    $displayPath = $resolution.display_path

    $directoryExists = [System.IO.Directory]::Exists($resolvedPath)
    if ($directoryExists -and
        [System.IO.Directory]::EnumerateFileSystemEntries($resolvedPath).GetEnumerator().MoveNext()) {
        throw "-ArtifactOutputDirectory must be empty to preserve profiling evidence: $displayPath"
    }
    return $resolvedPath
}

function Get-RenderExtractProfilingValidatorArguments {
    param(
        [Parameter(Mandatory)]
        [string]$Validator,
        [Parameter(Mandatory)]
        [string]$RepositoryRoot,
        [Parameter(Mandatory)]
        [string]$OutputDirectory,
        [Parameter(Mandatory)]
        [pscustomobject]$Request
    )

    $arguments = [System.Collections.Generic.List[string]]::new()
    $arguments.AddRange([string[]]@(
        '-NoProfile',
        '-ExecutionPolicy', 'Bypass',
        '-File', $Validator,
        '-RepoRoot', $RepositoryRoot,
        '-Package', $Request.Package,
        '-NoDefaultFeatures',
        '-Features', $Request.Features
    ))
    if (-not [string]::IsNullOrWhiteSpace($Request.Bin)) {
        $arguments.Add('-Bin')
        $arguments.Add($Request.Bin)
    }
    $arguments.AddRange([string[]]@(
        '-CargoProfile', $Request.CargoProfile,
        '-SkipTest',
        '-MvpProductInputArtifactOutput',
        '-ArtifactOutputDirectory', (Join-ZirconWindowsPath -Path $OutputDirectory -ChildPath $Request.Product),
        '-PublishArtifact', $Request.ArtifactName
    ))
    return $arguments.ToArray()
}

function Write-RenderExtractProfilingJsonFileNew {
    param(
        [Parameter(Mandatory)]
        [string]$Path,
        [Parameter(Mandatory)]
        $Value,
        [Parameter(Mandatory)]
        [string]$ExistingFileMessage
    )

    $bytes = [Text.UTF8Encoding]::new($false).GetBytes(($Value | ConvertTo-Json -Depth 6))
    try {
        $stream = [IO.File]::Open(
            $Path,
            [IO.FileMode]::CreateNew,
            [IO.FileAccess]::Write,
            [IO.FileShare]::None
        )
    }
    catch [IO.IOException] {
        if ([IO.File]::Exists($Path)) {
            throw "$ExistingFileMessage`: $Path"
        }
        throw
    }
    try {
        $stream.Write($bytes, 0, $bytes.Length)
        $stream.Flush($true)
    }
    finally {
        $stream.Dispose()
    }
}

function Write-RenderExtractProfilingInputManifest {
    param(
        [Parameter(Mandatory)]
        [string]$Path,
        [Parameter(Mandatory)]
        $BuildSet,
        [Parameter(Mandatory)]
        [string]$ArtifactOutputDirectory,
        [string]$PublishedArtifactOutputDirectory = $ArtifactOutputDirectory
    )

    $buildSetId = [string]$BuildSet.build_set_id
    if ($buildSetId -notmatch '^[0-9A-F]{64}$') {
        throw 'Render-extract profiling BuildSetId must be an uppercase SHA-256 value.'
    }

    $artifacts = [System.Collections.Generic.List[object]]::new()
    foreach ($request in @(Get-RenderExtractProfilingBuildRequests)) {
        $artifactPath = Join-ZirconWindowsPath `
            -Path (Join-ZirconWindowsPath -Path $ArtifactOutputDirectory -ChildPath $request.Product) `
            -ChildPath $request.ArtifactName
        $publishedArtifactPath = Join-ZirconWindowsPath `
            -Path (Join-ZirconWindowsPath -Path $PublishedArtifactOutputDirectory -ChildPath $request.Product) `
            -ChildPath $request.ArtifactName
        $artifactResolution = Resolve-ZirconWindowsPath -Path $artifactPath
        $publishedArtifactResolution = Resolve-ZirconWindowsPath -Path $publishedArtifactPath
        if (-not [System.IO.File]::Exists($artifactResolution.OperationalPath)) {
            throw "Render-extract profiling artifact does not exist: $artifactPath"
        }

        $artifact = [System.IO.FileInfo]::new($artifactResolution.OperationalPath)
        $artifacts.Add([ordered]@{
                logical_id = $request.logical_id
                product    = $request.Product
                package    = $request.Package
                bin        = $request.Bin
                features   = $request.Features
                path       = $publishedArtifactResolution.DisplayPath
                bytes      = [Int64]$artifact.Length
                sha256     = Get-MvpProductInputFileSha256 -Path $artifactResolution.OperationalPath
            })
    }

    $manifest = [ordered]@{
        schema_version     = 3
        generated_at_utc   = [DateTime]::UtcNow.ToString('o')
        source_fingerprint = $buildSetId
        build_set          = [ordered]@{
            build_set_id            = $buildSetId
            git_revision            = [string]$BuildSet.git_revision
            dirty_overlay_sha256     = [string]$BuildSet.dirty_overlay_sha256
            manifest_relative_path  = 'build-set/build-set.json'
        }
        cargo_profile      = 'profiling'
        artifacts          = @($artifacts)
    }
    Write-RenderExtractProfilingJsonFileNew `
        -Path $Path `
        -Value $manifest `
        -ExistingFileMessage 'Refusing to overwrite existing render-extract profiling input manifest'
    return $manifest
}

function Write-RenderExtractProfilingInputAbortReceipt {
    param(
        [Parameter(Mandatory)]
        [string]$Path,
        [Parameter(Mandatory)]
        [string]$OutputDirectory,
        [Parameter(Mandatory)]
        [string]$FailureMessage
    )

    Write-RenderExtractProfilingJsonFileNew `
        -Path $Path `
        -Value ([ordered]@{
                schema_version    = 1
                receipt_kind      = 'zircon_render_extract_profiling_input_abort'
                failed_at_utc     = [DateTime]::UtcNow.ToString('o')
                output_directory  = $OutputDirectory
                failure_message   = $FailureMessage
            }) `
        -ExistingFileMessage 'Refusing to overwrite existing render-extract profiling input abort receipt'
}

function Publish-RenderExtractProfilingInputRoot {
    param(
        [Parameter(Mandatory)]
        [string]$PublicationDirectory,
        [Parameter(Mandatory)]
        [string]$OutputDirectory,
        [Parameter(Mandatory)]
        [string]$PublicationParent
    )

    if ([IO.File]::Exists($OutputDirectory)) {
        throw "Render-extract profiling publication target is a file: $OutputDirectory"
    }
    if ([IO.Directory]::Exists($OutputDirectory)) {
        if ([IO.Directory]::EnumerateFileSystemEntries($OutputDirectory).GetEnumerator().MoveNext()) {
            throw "Render-extract profiling publication target must remain empty until publication: $OutputDirectory"
        }
        [IO.Directory]::Delete($OutputDirectory, $false)
    }
    Move-ZirconWindowsPath -Source $PublicationDirectory -Destination $OutputDirectory -ApprovedRoot $PublicationParent
}

function Invoke-RenderExtractProfilingInputBuild {
    param(
        [Parameter(Mandatory)]
        [string]$OutputDirectory
    )

    $resolvedOutputDirectory = Assert-RenderExtractProfilingInputDirectory -Path $OutputDirectory
    $publicationParent = [IO.Path]::GetDirectoryName($resolvedOutputDirectory)
    $publicationLeaf = [IO.Path]::GetFileName($resolvedOutputDirectory)
    if ([string]::IsNullOrWhiteSpace($publicationParent) -or
        [string]::IsNullOrWhiteSpace($publicationLeaf)) {
        throw "Render-extract profiling publication target must name a child directory: $resolvedOutputDirectory"
    }
    $publicationDirectory = Join-ZirconWindowsPath `
        -Path $publicationParent `
        -ChildPath ($publicationLeaf + '.partial-' + [guid]::NewGuid().ToString('N'))
    if ([IO.Directory]::Exists($publicationDirectory) -or [IO.File]::Exists($publicationDirectory)) {
        throw "Render-extract profiling publication staging directory already exists: $publicationDirectory"
    }

    [IO.Directory]::CreateDirectory($publicationDirectory) | Out-Null
    try {
        $buildSet = New-MvpProductBuildSet `
            -RepositoryRoot $repoRoot `
            -BuildSetRoot (Join-ZirconWindowsPath -Path $publicationDirectory -ChildPath 'build-set')
        $validator = Join-Path $buildSet.snapshot_root '.codex\skills\zircon-dev\scripts\validate-matrix.ps1'
        if (-not [IO.File]::Exists($validator)) {
            throw "BuildSet is missing the versioned Cargo validator: $validator"
        }

        foreach ($request in @(Get-RenderExtractProfilingBuildRequests)) {
            Assert-MvpProductBuildSet -ManifestPath $buildSet.manifest_path | Out-Null
            $validatorArguments = Get-RenderExtractProfilingValidatorArguments `
                -Validator $validator `
                -RepositoryRoot $buildSet.snapshot_root `
                -OutputDirectory $publicationDirectory `
                -Request $request
            & powershell.exe @validatorArguments
            if ($LASTEXITCODE -ne 0) {
                throw "Managed profiling build failed for $($request.logical_id)."
            }
            Assert-MvpProductBuildSet -ManifestPath $buildSet.manifest_path | Out-Null

            $artifactPath = Join-ZirconWindowsPath `
                -Path (Join-ZirconWindowsPath -Path $publicationDirectory -ChildPath $request.Product) `
                -ChildPath $request.ArtifactName
            if (-not [IO.File]::Exists($artifactPath)) {
                throw "Managed profiling build did not publish $($request.logical_id): $artifactPath"
            }
        }

        Assert-MvpProductBuildSet -ManifestPath $buildSet.manifest_path | Out-Null
        $stagedManifestPath = Join-ZirconWindowsPath `
            -Path $publicationDirectory `
            -ChildPath 'render-extract-profiling-inputs.json'
        $manifest = Write-RenderExtractProfilingInputManifest `
            -Path $stagedManifestPath `
            -BuildSet $buildSet `
            -ArtifactOutputDirectory $publicationDirectory `
            -PublishedArtifactOutputDirectory $resolvedOutputDirectory
        Assert-MvpProductBuildSet -ManifestPath $buildSet.manifest_path | Out-Null
        Publish-RenderExtractProfilingInputRoot `
            -PublicationDirectory $publicationDirectory `
            -OutputDirectory $resolvedOutputDirectory `
            -PublicationParent $publicationParent

        $manifestPath = Join-ZirconWindowsPath `
            -Path $resolvedOutputDirectory `
            -ChildPath 'render-extract-profiling-inputs.json'
        $manifestDisplayPath = (Resolve-ZirconWindowsPath -Path $manifestPath).DisplayPath
        Write-Host "Render-extract profiling input manifest: $manifestDisplayPath"
        return $manifest
    }
    catch {
        $failure = $_
        if ([IO.Directory]::Exists($publicationDirectory)) {
            $abortPath = Join-ZirconWindowsPath `
                -Path $publicationDirectory `
                -ChildPath 'render-extract-profiling-inputs-aborted.json'
            try {
                Write-RenderExtractProfilingInputAbortReceipt `
                    -Path $abortPath `
                    -OutputDirectory (Resolve-ZirconWindowsPath -Path $resolvedOutputDirectory).DisplayPath `
                    -FailureMessage $failure.Exception.Message
            }
            catch {
                Write-Warning "Could not write render-extract profiling input abort receipt: $($_.Exception.Message)"
            }
        }
        throw $failure
    }
}

if ($env:RENDER_EXTRACT_PROFILING_INPUTS_TEST_MODE -ne '1') {
    Invoke-RenderExtractProfilingInputBuild -OutputDirectory $ArtifactOutputDirectory
}
