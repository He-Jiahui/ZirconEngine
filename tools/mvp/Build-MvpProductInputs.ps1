[CmdletBinding()]
param(
    [string]$ArtifactOutputDirectory,
    [string]$SourceAdditionsManifestPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$pathResolverRepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
Import-Module (Join-Path $PSScriptRoot "MvpProductInputManifest.psm1") -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot "MvpProductProfileRegistry.psm1") -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot "MvpArtifactStoragePolicy.psm1") -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot "MvpBuildSet.psm1") -Force -ErrorAction Stop
# MvpProductInputManifest imports this module for its own scope. Import it last here so its
# forced module reload also leaves the resolver command visible to this build script's functions.
Import-Module (Join-Path $pathResolverRepoRoot "tools\maintenance\WindowsPathResolver.psm1") -Force -ErrorAction Stop
if ([string]::IsNullOrWhiteSpace($ArtifactOutputDirectory)) {
    $ArtifactOutputDirectory = New-MvpArtifactStoragePath -NamespaceId 'mvp-product-inputs'
}

function Get-MvpProductBuildRequests {
    param([AllowNull()]$RegistrySnapshot)

    # The client and editor-host cdylibs share a file name, so feature provenance requires separate groups.
    return @(Get-MvpProductInputSpecifications -RegistrySnapshot $RegistrySnapshot | ForEach-Object {
            [pscustomobject]@{
                logical_id = $_.logical_id
                Package = $_.package
                Bin = $_.bin
                Features = $_.features
                OutputGroup = $_.output_group
                ArtifactName = $_.artifact_name
            }
        })
}

function Invoke-MvpProductStagingCoordinator {
    param([Parameter(Mandatory)][string[]]$Arguments)

    $coordinator = Join-Path $pathResolverRepoRoot 'tools\dev\zircon-session.ps1'
    $output = @(& $coordinator -Json artifact @Arguments)
    if ($LASTEXITCODE -ne 0) {
        throw "Coordinator product staging command failed with exit code ${LASTEXITCODE}: $($output -join ' ')"
    }
    $json = $output -join "`n"
    if ([string]::IsNullOrWhiteSpace($json)) {
        throw 'Coordinator product staging command returned no JSON result.'
    }
    try {
        return $json | ConvertFrom-Json -ErrorAction Stop
    }
    catch {
        throw "Coordinator product staging command returned invalid JSON: $($_.Exception.Message)"
    }
}

function Assert-MvpProductStagingLease {
    param(
        [Parameter(Mandatory)]$Response,
        [Parameter(Mandatory)][string]$LeaseId,
        [Parameter(Mandatory)][string]$ExpectedStatus
    )

    if (
        $null -eq $Response.lease -or
        [string]$Response.lease.leaseId -ne $LeaseId -or
        [string]$Response.lease.status -ne $ExpectedStatus -or
        [int]$Response.lease.ownerPid -ne $PID
    ) {
        throw "Coordinator product staging returned an invalid '$ExpectedStatus' lease for $LeaseId."
    }
}

function Assert-MvpProductInputDirectory {
    param(
        [Parameter(Mandatory)]
        [string]$Path
    )

    try {
        $resolution = Resolve-MvpArtifactStoragePath -Path $Path -NamespaceId 'mvp-product-inputs'
    }
    catch {
        throw "-ArtifactOutputDirectory MVP product input artifact output must resolve under an approved storage policy root: $Path. $($_.Exception.Message)"
    }
    $resolvedPath = $resolution.operation_path

    if ([System.IO.Directory]::Exists($resolvedPath) -and
        [System.IO.Directory]::EnumerateFileSystemEntries($resolvedPath).GetEnumerator().MoveNext()) {
        throw "-ArtifactOutputDirectory must be empty to preserve product evidence: $resolvedPath"
    }

    return $resolvedPath
}

function Write-MvpProductInputManifest {
    param(
        [Parameter(Mandatory)]
        [string]$Path,
        [Parameter(Mandatory)]
        $Summary
    )

    $bytes = [Text.UTF8Encoding]::new($false).GetBytes(
        ($Summary | ConvertTo-Json -Depth 4)
    )
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
            throw "Refusing to overwrite existing MVP product input manifest: $Path"
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

function Write-MvpProductInputAbortReceipt {
    param(
        [Parameter(Mandatory)]
        [string]$Path,
        [Parameter(Mandatory)]
        [ValidatePattern('^mvp-product-inputs-[A-Za-z0-9][A-Za-z0-9._-]*$')]
        [string]$ArtifactOutputName,
        [Parameter(Mandatory)]
        [ValidatePattern('^[a-z0-9][a-z0-9._-]{0,127}$')]
        [string]$FailureKind,
        [Parameter(Mandatory)]
        [AllowEmptyString()]
        [string]$FailureMessage
    )

    $failureMessagePrefixLength = [Math]::Min($FailureMessage.Length, 4096)
    $failureMessagePrefix = $FailureMessage.Substring(0, $failureMessagePrefixLength)
    $failureMessagePrefixBytes = [Text.UTF8Encoding]::new($false).GetBytes($failureMessagePrefix)
    Write-MvpProductInputManifest -Path $Path -Summary ([ordered]@{
            schema_version                  = 1
            receipt_kind                    = 'zircon.mvp-product-input-abort'
            failed_at_utc                   = [DateTime]::UtcNow.ToString('o')
            artifact_output_name            = $ArtifactOutputName
            failure_kind                    = $FailureKind
            failure_message_length          = [Int64]$FailureMessage.Length
            failure_message_prefix_length   = [Int64]$failureMessagePrefixLength
            failure_message_prefix_sha256   = Get-MvpProductInputBytesSha256 -Bytes $failureMessagePrefixBytes
            failure_message_truncated       = $FailureMessage.Length -gt $failureMessagePrefixLength
        })
}

function Publish-MvpProductInputAbortReceipt {
    param(
        [Parameter(Mandatory)]
        [string]$PublicationParent,
        [Parameter(Mandatory)]
        [ValidatePattern('^mvp-product-inputs-[A-Za-z0-9][A-Za-z0-9._-]*$')]
        [string]$PublicationLeaf,
        [Parameter(Mandatory)]
        [ValidatePattern('^[a-z0-9][a-z0-9._-]{0,127}$')]
        [string]$FailureKind,
        [Parameter(Mandatory)]
        [AllowEmptyString()]
        [string]$FailureMessage
    )

    $abortLeaf = $PublicationLeaf + '.aborted.json'
    $abortPath = Join-ZirconWindowsPath -Path $PublicationParent -ChildPath $abortLeaf
    if ([IO.File]::Exists($abortPath) -or [IO.Directory]::Exists($abortPath)) {
        throw "MVP product input abort receipt already exists: $abortPath"
    }
    $stagedAbortPath = Join-ZirconWindowsPath `
        -Path $PublicationParent `
        -ChildPath ($abortLeaf + '.partial-' + [guid]::NewGuid().ToString('N'))
    Write-MvpProductInputAbortReceipt `
        -Path $stagedAbortPath `
        -ArtifactOutputName $PublicationLeaf `
        -FailureKind $FailureKind `
        -FailureMessage $FailureMessage
    Move-ZirconWindowsPath `
        -Source $stagedAbortPath `
        -Destination $abortPath `
        -ApprovedRoot $PublicationParent | Out-Null
    return $abortPath
}

function Get-MvpProductInputFailureKind {
    param([Parameter(Mandatory)][Exception]$Exception)

    if ($Exception -is [OperationCanceledException]) {
        return 'cancelled'
    }
    if ($Exception -is [TimeoutException]) {
        return 'timeout'
    }
    if ($Exception -is [UnauthorizedAccessException]) {
        return 'access_denied'
    }
    if ($Exception -is [IO.IOException]) {
        return 'io_failure'
    }
    return 'build_failed'
}

function Publish-MvpProductInputPublicationRoot {
    param(
        [Parameter(Mandatory)]
        [string]$PublicationDirectory,
        [Parameter(Mandatory)]
        [string]$OutputDirectory,
        [Parameter(Mandatory)]
        [string]$PublicationParent,
        [Microsoft.Win32.SafeHandles.SafeFileHandle]$SourceLease
    )

    if ([IO.File]::Exists($OutputDirectory)) {
        throw "MVP product input publication target is a file: $OutputDirectory"
    }
    if ([IO.Directory]::Exists($OutputDirectory)) {
        if ($null -ne $SourceLease) {
            throw "MVP product input publication target appeared after staging was acquired: $OutputDirectory"
        }
        if ([IO.Directory]::EnumerateFileSystemEntries($OutputDirectory).GetEnumerator().MoveNext()) {
            throw "MVP product input publication target must remain empty until publication: $OutputDirectory"
        }
        # The caller proved this exact target empty before staging; remove it so the move creates
        # the completed output root in one operation rather than exposing partial group directories.
        [IO.Directory]::Delete($OutputDirectory, $false)
    }
    if ($null -ne $SourceLease) {
        Move-ZirconWindowsLeasedPathWithinRoot `
            -SourceLease $SourceLease `
            -Destination $OutputDirectory `
            -ApprovedRoot $PublicationParent | Out-Null
    }
    else {
        Move-ZirconWindowsPath -Source $PublicationDirectory -Destination $OutputDirectory -ApprovedRoot $PublicationParent | Out-Null
    }
}

function Publish-MvpProductInputManifest {
    param(
        [Parameter(Mandatory)]
        [string]$Path,
        [Parameter(Mandatory)]
        $Summary,
        [Parameter(Mandatory)]
        $BuildSet
    )

    Write-MvpProductInputManifest -Path $Path -Summary $Summary
    Assert-MvpProductBuildSet -ManifestPath $BuildSet.manifest_path | Out-Null
}

function Invoke-MvpProductInputBuild {
    param(
        [Parameter(Mandatory)]
        [string]$OutputDirectory
    )

    $repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
    $resolvedOutputDirectory = Assert-MvpProductInputDirectory -Path $OutputDirectory
    $publicationParent = [IO.Path]::GetDirectoryName($resolvedOutputDirectory)
    $publicationLeaf = [IO.Path]::GetFileName($resolvedOutputDirectory)
    if ([string]::IsNullOrWhiteSpace($publicationParent) -or [string]::IsNullOrWhiteSpace($publicationLeaf)) {
        throw "MVP product input publication target must name a child directory: $resolvedOutputDirectory"
    }
    $publicationDirectory = $null
    $productStagingLeaseId = $null
    $productStagingStatus = $null
    $stagingCreated = $false
    $rootLease = $null
    $stagingLease = $null
    $cleanupRootLease = $null
    # This retained field is only for the current Stage reader. Build identity is the BuildSet
    # below; no product build or publication re-reads the mutable checkout.
    $publicationCompleted = $false
    try {
        if ([IO.Directory]::Exists($resolvedOutputDirectory) -or [IO.File]::Exists($resolvedOutputDirectory)) {
            throw "MVP product input publication target must not already exist: $resolvedOutputDirectory"
        }
        $outputDisplayPath = (Resolve-ZirconWindowsPath -Path $resolvedOutputDirectory).DisplayPath
        $acquireResponse = Invoke-MvpProductStagingCoordinator -Arguments @(
            'staging-acquire'
            '--purpose', 'build-product-inputs'
            '--final-path', $outputDisplayPath
            '--owner-pid', [string]$PID
        )
        if ($null -eq $acquireResponse.lease) {
            throw 'Coordinator product staging acquire response omitted its lease.'
        }
        $productStagingLeaseId = [string]$acquireResponse.lease.leaseId
        if ($productStagingLeaseId -notmatch '^[0-9a-f]{32}$') {
            throw "Coordinator product staging acquire returned an invalid lease ID: $productStagingLeaseId"
        }
        Assert-MvpProductStagingLease -Response $acquireResponse -LeaseId $productStagingLeaseId -ExpectedStatus 'active'
        $productStagingStatus = 'active'
        if (
            [string]$acquireResponse.lease.purpose -ne 'build-product-inputs' -or
            -not [string]::Equals(
                [string]$acquireResponse.lease.finalPath,
                $outputDisplayPath,
                [StringComparison]::OrdinalIgnoreCase)
        ) {
            throw 'Coordinator product staging lease is not bound to this product output.'
        }
        $stagingResolution = Resolve-ZirconWindowsPath -Path ([string]$acquireResponse.lease.stagingPath)
        $publicationDirectory = $stagingResolution.OperationalPath.TrimEnd('\')
        $expectedStagingPath = Join-ZirconWindowsPath `
            -Path $publicationParent `
            -ChildPath "mvp-product-inputs-build-product-inputs-$productStagingLeaseId"
        if (-not [string]::Equals(
                $publicationDirectory,
                $expectedStagingPath,
                [StringComparison]::OrdinalIgnoreCase)) {
            throw "Coordinator product staging path is outside the approved product root: $($stagingResolution.DisplayPath)"
        }
        $rootLease = Open-ZirconWindowsDirectoryLease `
            -Path $publicationParent `
            -ExpectedOperationalPath $publicationParent
        if ([IO.Directory]::Exists($publicationDirectory) -or [IO.File]::Exists($publicationDirectory)) {
            throw "MVP product input publication staging directory already exists: $publicationDirectory"
        }
        [System.IO.Directory]::CreateDirectory($publicationDirectory) | Out-Null
        $stagingCreated = $true
        $stagingLease = Open-ZirconWindowsDirectoryLease `
            -Path $publicationDirectory `
            -ExpectedOperationalPath $publicationDirectory `
            -ForMove `
            -DenyWrite `
            -NoFollow
        $buildSet = New-MvpProductBuildSet `
            -RepositoryRoot $repoRoot `
            -BuildSetRoot (Join-ZirconWindowsPath -Path $publicationDirectory -ChildPath 'build-set') `
            -SourceAdditionsManifestPath $SourceAdditionsManifestPath
        $productProfileRegistrySnapshot = Get-MvpProductProfileRegistrySnapshot `
            -RegistryPath (Join-ZirconWindowsPath -Path $buildSet.snapshot_root -ChildPath 'tools\mvp\mvp-product-profile-registry.json')
        $validator = (Resolve-ZirconWindowsPath `
            -Path (Join-ZirconWindowsPath `
                -Path $buildSet.snapshot_root `
                -ChildPath ".codex\skills\zircon-dev\scripts\validate-matrix.ps1")).DisplayPath
        if (-not [System.IO.File]::Exists($validator)) {
            throw "BuildSet is missing the versioned Cargo validator: $validator"
        }
        $published = [System.Collections.Generic.List[object]]::new()

        foreach ($request in (Get-MvpProductBuildRequests -RegistrySnapshot $productProfileRegistrySnapshot)) {
            Assert-MvpProductBuildSet -ManifestPath $buildSet.manifest_path | Out-Null
            $stagedGroupDirectory = Join-ZirconWindowsPath -Path $publicationDirectory -ChildPath $request.OutputGroup
            [System.IO.Directory]::CreateDirectory($stagedGroupDirectory) | Out-Null
            $validatorArguments = @(
            "-NoProfile",
            "-ExecutionPolicy", "Bypass",
            "-File", $validator,
            "-RepoRoot", $buildSet.snapshot_root,
            "-Package", $request.Package,
            "-NoDefaultFeatures",
            "-Features", $request.Features,
            "-SkipTest",
            "-MvpProductInputArtifactOutput",
            "-ArtifactOutputDirectory", $stagedGroupDirectory,
            "-PublishArtifact", $request.ArtifactName
            )
            if (-not [string]::IsNullOrWhiteSpace($request.Bin)) {
                $validatorArguments += @("-Bin", $request.Bin)
            }

            & powershell.exe @validatorArguments
            if ($LASTEXITCODE -ne 0) {
                throw "Managed product build failed for $($request.Package) $($request.ArtifactName)."
            }
            Assert-MvpProductBuildSet -ManifestPath $buildSet.manifest_path | Out-Null

            $stagedArtifactPath = Join-ZirconWindowsPath -Path $stagedGroupDirectory -ChildPath $request.ArtifactName
            if (-not [System.IO.File]::Exists($stagedArtifactPath)) {
                throw "Managed product build did not publish the declared artifact: $stagedArtifactPath"
            }
            $publishedArtifactPath = Join-ZirconWindowsPath `
                -Path (Join-ZirconWindowsPath -Path $resolvedOutputDirectory -ChildPath $request.OutputGroup) `
                -ChildPath $request.ArtifactName
            $artifactDisplayPath = (Resolve-ZirconWindowsPath -Path $publishedArtifactPath).DisplayPath
            $published.Add([pscustomobject]@{
                LogicalId     = $request.logical_id
                Package      = $request.Package
                Bin          = $request.Bin
                Features     = $request.Features
                OutputGroup  = $request.OutputGroup
                ArtifactName = $request.ArtifactName
                Path         = $artifactDisplayPath
                Bytes        = [System.IO.FileInfo]::new($stagedArtifactPath).Length
                Sha256       = Get-MvpProductInputFileSha256 -Path $stagedArtifactPath
            }) | Out-Null
        }

        Assert-MvpProductBuildSet -ManifestPath $buildSet.manifest_path | Out-Null

        $summary = [pscustomobject]@{
        schema_version           = 2
        generated_at_utc         = [DateTime]::UtcNow.ToString("o")
        source_fingerprint       = $buildSet.build_set_id
        product_profile_registry = $productProfileRegistrySnapshot.receipt
        build_set                = [ordered]@{
            build_set_id = $buildSet.build_set_id
            git_revision = $buildSet.git_revision
            dirty_overlay_sha256 = $buildSet.dirty_overlay_sha256
            manifest_relative_path = 'build-set/build-set.json'
        }
        artifact_output_directory = (Resolve-ZirconWindowsPath -Path $resolvedOutputDirectory).DisplayPath
        artifacts                = @($published)
        }
        $stagedSummaryPath = Join-ZirconWindowsPath -Path $publicationDirectory -ChildPath "mvp-product-inputs.json"
        Publish-MvpProductInputManifest `
            -Path $stagedSummaryPath `
            -Summary $summary `
            -BuildSet $buildSet
        $publishingResponse = Invoke-MvpProductStagingCoordinator -Arguments @(
            'staging-begin-publish'
            '--lease-id', $productStagingLeaseId
            '--owner-pid', [string]$PID
        )
        Assert-MvpProductStagingLease -Response $publishingResponse -LeaseId $productStagingLeaseId -ExpectedStatus 'publishing'
        $productStagingStatus = 'publishing'
        Publish-MvpProductInputPublicationRoot `
            -PublicationDirectory $publicationDirectory `
            -OutputDirectory $resolvedOutputDirectory `
            -PublicationParent $publicationParent `
            -SourceLease $stagingLease
        $stagingCreated = $false
        $publishedResponse = Invoke-MvpProductStagingCoordinator -Arguments @(
            'staging-complete-publish'
            '--lease-id', $productStagingLeaseId
            '--owner-pid', [string]$PID
        )
        Assert-MvpProductStagingLease -Response $publishedResponse -LeaseId $productStagingLeaseId -ExpectedStatus 'published'
        $productStagingStatus = 'published'
        $publicationCompleted = $true
        $summaryPath = Join-ZirconWindowsPath -Path $resolvedOutputDirectory -ChildPath "mvp-product-inputs.json"
        Write-Host "MVP product input manifest: $((Resolve-ZirconWindowsPath -Path $summaryPath).DisplayPath)"
        return $summary
    }
    catch {
        $failure = $_
        if ($stagingCreated -and $null -ne $stagingLease) {
            try {
                $rootLease.Dispose()
                $rootLease = $null
                $cleanupRootLease = Open-ZirconWindowsDirectoryLease `
                    -Path $publicationParent `
                    -ExpectedOperationalPath $publicationParent `
                    -DenyWrite `
                    -NoFollow
                Remove-ZirconWindowsLeasedDirectoryTree -Lease $stagingLease
                $stagingLease.Dispose()
                $stagingLease = $null
            }
            catch {
                Write-Warning "Could not remove the leased product staging directory: $($_.Exception.Message)"
            }
        }
        if (
            $null -ne $productStagingLeaseId -and
            $productStagingStatus -in @('active', 'publishing') -and
            ($null -eq $publicationDirectory -or
                (-not [IO.Directory]::Exists($publicationDirectory) -and
                 -not [IO.File]::Exists($publicationDirectory))) -and
            -not [IO.Directory]::Exists($resolvedOutputDirectory) -and
            -not [IO.File]::Exists($resolvedOutputDirectory)
        ) {
            try {
                $releaseResponse = Invoke-MvpProductStagingCoordinator -Arguments @(
                    'staging-release'
                    '--lease-id', $productStagingLeaseId
                    '--owner-pid', [string]$PID
                )
                Assert-MvpProductStagingLease -Response $releaseResponse -LeaseId $productStagingLeaseId -ExpectedStatus 'released'
                $productStagingStatus = 'released'
            }
            catch {
                Write-Warning "Could not release the product staging lease: $($_.Exception.Message)"
            }
        }
        if ($null -ne $cleanupRootLease) {
            $cleanupRootLease.Dispose()
            $cleanupRootLease = $null
        }
        if (-not $publicationCompleted -and -not [IO.Directory]::Exists($resolvedOutputDirectory)) {
            try {
                Publish-MvpProductInputAbortReceipt `
                    -PublicationParent $publicationParent `
                    -PublicationLeaf $publicationLeaf `
                    -FailureKind (Get-MvpProductInputFailureKind -Exception $failure.Exception) `
                    -FailureMessage $failure.Exception.Message
            }
            catch {
                Write-Warning "Could not write MVP product input abort receipt: $($_.Exception.Message)"
            }
        }
        throw $failure
    }
    finally {
        if ($null -ne $cleanupRootLease) { $cleanupRootLease.Dispose() }
        if ($null -ne $stagingLease) { $stagingLease.Dispose() }
        if ($null -ne $rootLease) { $rootLease.Dispose() }
    }
}

if ($env:MVP_PRODUCT_INPUTS_TEST_MODE -ne "1") {
    Invoke-MvpProductInputBuild -OutputDirectory $ArtifactOutputDirectory
}
