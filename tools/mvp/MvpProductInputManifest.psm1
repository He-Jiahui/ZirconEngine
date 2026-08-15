Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$script:MvpProductInputSchemaVersion = 1
$script:MvpGitHashObjectBatchArgumentBudget = 24576
$script:MvpProductInputSpecifications = @(
    [ordered]@{
        logical_id = 'runtime-executable'
        package = 'zircon_app'
        bin = 'zircon_runtime'
        features = 'target-client,platform-winit,input-gamepad,gamepad-gilrs'
        output_group = 'runtime'
        artifact_name = 'zircon_runtime.exe'
    },
    [ordered]@{
        logical_id = 'runtime-library/runtime'
        package = 'zircon_runtime'
        bin = $null
        features = 'target-client,platform-winit,input-gamepad,gamepad-gilrs'
        output_group = 'runtime'
        artifact_name = 'zircon_runtime.dll'
    },
    [ordered]@{
        logical_id = 'editor-executable'
        package = 'zircon_app'
        bin = 'zircon_editor'
        features = 'target-editor-host'
        output_group = 'editor'
        artifact_name = 'zircon_editor.exe'
    },
    [ordered]@{
        logical_id = 'runtime-library/editor'
        package = 'zircon_runtime'
        bin = $null
        features = 'target-editor-host'
        output_group = 'editor'
        artifact_name = 'zircon_runtime.dll'
    }
)

$moduleRepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
Import-Module (Join-Path $moduleRepoRoot 'tools\WindowsPathResolver.psm1') -Force -ErrorAction Stop

function Get-MvpProductInputSpecifications {
    return @($script:MvpProductInputSpecifications | ForEach-Object { [pscustomobject]$_ })
}

function Get-MvpProductInputFileSha256 {
    param([Parameter(Mandatory)][string]$Path)

    $stream = [IO.File]::OpenRead($Path)
    $hasher = [Security.Cryptography.SHA256]::Create()
    try {
        return -join ($hasher.ComputeHash($stream) | ForEach-Object { $_.ToString('X2') })
    }
    finally {
        $hasher.Dispose()
        $stream.Dispose()
    }
}

function Get-MvpProductInputBytesSha256 {
    param([Parameter(Mandatory)][byte[]]$Bytes)

    $hasher = [Security.Cryptography.SHA256]::Create()
    try {
        return -join ($hasher.ComputeHash($Bytes) | ForEach-Object { $_.ToString('X2') })
    }
    finally {
        $hasher.Dispose()
    }
}

function Invoke-MvpSourceGit {
    param(
        [Parameter(Mandatory)][string]$GitPath,
        [Parameter(Mandatory)][string]$WorkingDirectory,
        [Parameter(Mandatory)][string[]]$Arguments
    )

    $startInfo = [Diagnostics.ProcessStartInfo]::new()
    $startInfo.FileName = $GitPath
    $startInfo.WorkingDirectory = $WorkingDirectory
    $startInfo.Arguments = $Arguments -join ' '
    $startInfo.UseShellExecute = $false
    $startInfo.CreateNoWindow = $true
    $startInfo.RedirectStandardOutput = $true
    $startInfo.RedirectStandardError = $true
    $process = [Diagnostics.Process]::new()
    $process.StartInfo = $startInfo
    $stdoutStream = [IO.MemoryStream]::new()
    try {
        if (-not $process.Start()) {
            throw "Could not start git source fingerprint command '$($Arguments -join ' ')'."
        }
        $stdoutTask = $process.StandardOutput.BaseStream.CopyToAsync($stdoutStream)
        $stderrTask = $process.StandardError.ReadToEndAsync()
        $process.WaitForExit()
        [Threading.Tasks.Task]::WaitAll(@($stdoutTask, $stderrTask))
        if ($process.ExitCode -ne 0) {
            $detail = $stderrTask.Result.Trim()
            if ([string]::IsNullOrWhiteSpace($detail)) {
                $detail = 'no stderr output'
            }
            throw "Git source fingerprint command '$($Arguments -join ' ')' failed with exit code $($process.ExitCode): $detail"
        }
        Write-Output -NoEnumerate ([byte[]]$stdoutStream.ToArray())
    }
    finally {
        $stdoutStream.Dispose()
        $process.Dispose()
    }
}

function Add-MvpTrackedSourceContentHashBatch {
    param(
        [Parameter(Mandatory)][string]$GitPath,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][Text.Encoding]$Encoding,
        [Parameter(Mandatory)][Collections.Generic.List[string]]$Paths,
        [Parameter(Mandatory)][AllowEmptyCollection()][Collections.Generic.List[object]]$Records
    )

    $arguments = [Collections.Generic.List[string]]::new()
    $arguments.AddRange([string[]]@('hash-object', '--no-filters', '--'))
    foreach ($relativePath in $Paths) {
        if ($relativePath.Contains('"')) {
            throw "Tracked source input '$relativePath' contains a double quote and cannot be supplied to git hash-object safely."
        }
        # ProcessStartInfo parses this command-line string directly; Windows file names cannot
        # contain double quotes, so quoting preserves spaces without a shell interpretation.
        $arguments.Add(('"{0}"' -f $relativePath))
    }
    [byte[]]$hashOutputBytes = Invoke-MvpSourceGit `
        -GitPath $GitPath `
        -WorkingDirectory $RepositoryRoot `
        -Arguments $arguments.ToArray()
    $hashes = @($Encoding.GetString($hashOutputBytes).Split("`n") | ForEach-Object { $_.Trim() } | Where-Object {
            -not [string]::IsNullOrWhiteSpace($_)
        })
    if ($hashes.Count -ne $Paths.Count) {
        throw "Git source fingerprint hash-object output count ($($hashes.Count)) does not match tracked source inputs ($($Paths.Count))."
    }
    for ($index = 0; $index -lt $Paths.Count; $index++) {
        $hash = $hashes[$index]
        if ($hash -notmatch '^[0-9a-fA-F]+$') {
            throw "Git source fingerprint hash-object returned an invalid object id for '$($Paths[$index])'."
        }
        $Records.Add([pscustomobject]@{
                relative_path = $Paths[$index]
                object_hash = $hash.ToUpperInvariant()
            })
    }
}

function Get-MvpTrackedSourceContentHashes {
    param(
        [Parameter(Mandatory)][string]$GitPath,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][byte[]]$TrackedPathBytes
    )

    # `powershell.exe` must not emit an UTF-8 preamble into git's line-oriented
    # --stdin-paths protocol; the preamble would become part of the first path.
    $encoding = [Text.UTF8Encoding]::new($false)
    $trackedPaths = @($encoding.GetString($TrackedPathBytes).Split([char]0) | Where-Object {
            $_.Length -gt 0
        } | Sort-Object -Unique)
    $existingPaths = [System.Collections.Generic.List[string]]::new()
    foreach ($relativePath in $trackedPaths) {
        if ($relativePath.Contains("`n") -or $relativePath.Contains("`r")) {
            throw "Tracked source input '$relativePath' contains a newline and cannot be supplied to git hash-object safely."
        }
        $path = (Resolve-ZirconWindowsPath `
                -Path (Join-ZirconWindowsPath -Path $RepositoryRoot -ChildPath $relativePath)).OperationalPath
        if ([IO.File]::Exists($path)) {
            $existingPaths.Add($relativePath)
        }
        elseif ([IO.Directory]::Exists($path)) {
            throw "Tracked source input '$relativePath' is a directory; source fingerprinting requires a file identity."
        }
    }
    if ($existingPaths.Count -eq 0) {
        return @()
    }

    $records = [System.Collections.Generic.List[object]]::new()
    $batchPaths = [System.Collections.Generic.List[string]]::new()
    $batchArgumentLength = 'hash-object --no-filters --'.Length
    foreach ($relativePath in $existingPaths) {
        $argumentLength = $relativePath.Length + 3
        if ($argumentLength -gt $script:MvpGitHashObjectBatchArgumentBudget) {
            throw "Tracked source input '$relativePath' exceeds the Git hash-object command-line budget."
        }
        if ($batchPaths.Count -gt 0 -and
            ($batchArgumentLength + $argumentLength) -gt $script:MvpGitHashObjectBatchArgumentBudget) {
            Add-MvpTrackedSourceContentHashBatch `
                -GitPath $GitPath `
                -RepositoryRoot $RepositoryRoot `
                -Encoding $encoding `
                -Paths $batchPaths `
                -Records $records
            $batchPaths.Clear()
            $batchArgumentLength = 'hash-object --no-filters --'.Length
        }
        $batchPaths.Add($relativePath)
        $batchArgumentLength += $argumentLength
    }
    if ($batchPaths.Count -gt 0) {
        Add-MvpTrackedSourceContentHashBatch `
            -GitPath $GitPath `
            -RepositoryRoot $RepositoryRoot `
            -Encoding $encoding `
            -Paths $batchPaths `
            -Records $records
    }
    return @($records)
}

function Add-MvpFingerprintSegment {
    param(
        [Parameter(Mandatory)][IO.Stream]$Stream,
        [Parameter(Mandatory)][AllowEmptyCollection()][byte[]]$Bytes
    )

    [byte[]]$length = [BitConverter]::GetBytes([Int64]$Bytes.LongLength)
    $Stream.Write($length, 0, $length.Length)
    $Stream.Write($Bytes, 0, $Bytes.Length)
}

function Get-MvpSourceFingerprint {
    param([string]$RepositoryRoot = $moduleRepoRoot)

    $repoRoot = (Resolve-ZirconWindowsPath -Path $RepositoryRoot).OperationalPath
    $git = Get-Command git -ErrorAction SilentlyContinue
    if ($null -eq $git) {
        throw 'Could not resolve git for the MVP source fingerprint.'
    }

    [byte[]]$commitBytes = Invoke-MvpSourceGit `
            -GitPath $git.Source `
            -WorkingDirectory $repoRoot `
            -Arguments @('rev-parse', 'HEAD')
    $commit = [Text.Encoding]::ASCII.GetString($commitBytes).Trim()
    if ([string]::IsNullOrWhiteSpace($commit)) {
        throw 'Could not resolve the current source commit for the MVP source fingerprint.'
    }
    # Raw diff carries every tracked path, status, and mode without materializing a full binary
    # patch. The corresponding working-tree bytes are hashed below with git hash-object.
    [byte[]]$trackedDiffBytes = Invoke-MvpSourceGit `
        -GitPath $git.Source `
        -WorkingDirectory $repoRoot `
        -Arguments @('diff', '--no-ext-diff', '--raw', '--no-abbrev', '-z', 'HEAD')
    [byte[]]$trackedPathBytes = Invoke-MvpSourceGit `
        -GitPath $git.Source `
        -WorkingDirectory $repoRoot `
        -Arguments @('diff', '--no-ext-diff', '--name-only', '-z', 'HEAD')
    $trackedContentHashes = Get-MvpTrackedSourceContentHashes `
        -GitPath $git.Source `
        -RepositoryRoot $repoRoot `
        -TrackedPathBytes $trackedPathBytes
    [byte[]]$untrackedOutputBytes = Invoke-MvpSourceGit `
        -GitPath $git.Source `
        -WorkingDirectory $repoRoot `
        -Arguments @('ls-files', '-z', '--others', '--exclude-standard')
    $untrackedEncoding = [Text.UTF8Encoding]::new($false)
    $untrackedPaths = @($untrackedEncoding.GetString($untrackedOutputBytes).Split([char[]]@([char]0)) | Where-Object {
            $_.Length -gt 0
        })

    $material = [IO.MemoryStream]::new()
    try {
        $encoding = [Text.UTF8Encoding]::new($false)
        Add-MvpFingerprintSegment -Stream $material -Bytes $encoding.GetBytes('zircon-mvp-source-fingerprint-v3')
        Add-MvpFingerprintSegment -Stream $material -Bytes $encoding.GetBytes($commit)
        Add-MvpFingerprintSegment -Stream $material -Bytes $trackedDiffBytes
        foreach ($tracked in $trackedContentHashes) {
            Add-MvpFingerprintSegment -Stream $material -Bytes $encoding.GetBytes($tracked.relative_path)
            Add-MvpFingerprintSegment -Stream $material -Bytes $encoding.GetBytes($tracked.object_hash)
        }
        Add-MvpFingerprintSegment -Stream $material -Bytes $untrackedOutputBytes
        foreach ($relativePath in ($untrackedPaths | Sort-Object)) {
            $path = (Resolve-ZirconWindowsPath `
                -Path (Join-ZirconWindowsPath -Path $repoRoot -ChildPath $relativePath)).OperationalPath
            if (-not [IO.File]::Exists($path)) {
                throw "Untracked source input '$relativePath' does not exist or is not a file."
            }
            Add-MvpFingerprintSegment -Stream $material -Bytes $encoding.GetBytes($relativePath)
            Add-MvpFingerprintSegment -Stream $material -Bytes $encoding.GetBytes((Get-MvpProductInputFileSha256 -Path $path))
        }
        return Get-MvpProductInputBytesSha256 -Bytes $material.ToArray()
    }
    finally {
        $material.Dispose()
    }
}

function Get-MvpProductInputManifestProperty {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Label
    )

    $property = $Value.PSObject.Properties[$Name]
    if ($null -eq $property) {
        throw "$Label is missing required property '$Name'."
    }
    return $property.Value
}

function Resolve-MvpProductInputManifest {
    param([Parameter(Mandatory)][string]$Path)

    $manifestPath = (Resolve-ZirconWindowsPath -Path $Path).OperationalPath
    if (-not [IO.File]::Exists($manifestPath)) {
        throw "ProductInputManifest '$Path' does not exist or is not a file."
    }
    try {
        # Stage copies must be compared with the exact bytes parsed here, not a later re-read.
        $manifestBytes = [IO.File]::ReadAllBytes($manifestPath)
        $manifest = ([Text.UTF8Encoding]::new($false)).GetString($manifestBytes) | ConvertFrom-Json
    }
    catch {
        throw "ProductInputManifest '$Path' is not valid JSON: $($_.Exception.Message)"
    }
    if ($null -eq $manifest -or $manifest -is [Array]) {
        throw "ProductInputManifest '$Path' must contain one JSON object."
    }

    $schemaVersion = Get-MvpProductInputManifestProperty -Value $manifest -Name 'schema_version' -Label 'ProductInputManifest'
    if ([int]$schemaVersion -ne $script:MvpProductInputSchemaVersion) {
        throw "ProductInputManifest '$Path' has unsupported schema_version '$schemaVersion'."
    }
    $sourceFingerprint = [string](Get-MvpProductInputManifestProperty -Value $manifest -Name 'source_fingerprint' -Label 'ProductInputManifest')
    if ($sourceFingerprint -notmatch '^[0-9A-F]{64}$') {
        throw "ProductInputManifest '$Path' must contain an uppercase SHA-256 source_fingerprint."
    }
    $artifacts = @(Get-MvpProductInputManifestProperty -Value $manifest -Name 'artifacts' -Label 'ProductInputManifest')
    if ($artifacts.Count -ne $script:MvpProductInputSpecifications.Count) {
        throw "ProductInputManifest '$Path' must contain exactly $($script:MvpProductInputSpecifications.Count) product artifacts."
    }

    $resolvedArtifacts = [ordered]@{}
    foreach ($specification in $script:MvpProductInputSpecifications) {
        $matches = @($artifacts | Where-Object {
            ([string](Get-MvpProductInputManifestProperty -Value $_ -Name 'LogicalId' -Label 'Product artifact')) -eq $specification.logical_id
        })
        if ($matches.Count -ne 1) {
            throw "ProductInputManifest '$Path' must contain exactly one '$($specification.logical_id)' artifact."
        }
        $artifact = $matches[0]
        foreach ($propertyName in @('Package', 'Bin', 'Features', 'OutputGroup', 'ArtifactName')) {
            $actual = Get-MvpProductInputManifestProperty -Value $artifact -Name $propertyName -Label "Product artifact '$($specification.logical_id)'"
            $expectedName = switch ($propertyName) {
                'Package' { 'package' }
                'Bin' { 'bin' }
                'Features' { 'features' }
                'OutputGroup' { 'output_group' }
                'ArtifactName' { 'artifact_name' }
            }
            $expected = $specification[$expectedName]
            if ($null -eq $expected) {
                if ($null -ne $actual -and -not [string]::IsNullOrWhiteSpace([string]$actual)) {
                    throw "Product artifact '$($specification.logical_id)' has unexpected $propertyName '$actual'."
                }
            }
            elseif ([string]$actual -ne [string]$expected) {
                throw "Product artifact '$($specification.logical_id)' has $propertyName '$actual'; expected '$expected'."
            }
        }

        $artifactPath = [string](Get-MvpProductInputManifestProperty -Value $artifact -Name 'Path' -Label "Product artifact '$($specification.logical_id)'")
        $artifactResolution = Resolve-ZirconWindowsPath -Path $artifactPath
        if (-not [IO.File]::Exists($artifactResolution.OperationalPath)) {
            throw "Product artifact '$($specification.logical_id)' does not exist: $artifactPath"
        }
        $expectedBytes = [Int64](Get-MvpProductInputManifestProperty -Value $artifact -Name 'Bytes' -Label "Product artifact '$($specification.logical_id)'")
        $actualBytes = [IO.FileInfo]::new($artifactResolution.OperationalPath).Length
        if ($expectedBytes -ne $actualBytes) {
            throw "Product artifact '$($specification.logical_id)' byte length differs from ProductInputManifest."
        }
        $expectedHash = [string](Get-MvpProductInputManifestProperty -Value $artifact -Name 'Sha256' -Label "Product artifact '$($specification.logical_id)'")
        if ($expectedHash -notmatch '^[0-9A-F]{64}$') {
            throw "Product artifact '$($specification.logical_id)' must contain an uppercase SHA-256."
        }
        $actualHash = Get-MvpProductInputFileSha256 -Path $artifactResolution.OperationalPath
        if ($expectedHash -ne $actualHash) {
            throw "Product artifact '$($specification.logical_id)' SHA-256 differs from ProductInputManifest."
        }
        $resolvedArtifacts[$specification.logical_id] = [ordered]@{
            operation_path = $artifactResolution.OperationalPath
            bytes = $actualBytes
            sha256 = $actualHash
        }
    }

    return [ordered]@{
        operation_path = $manifestPath
        bytes = [Int64]$manifestBytes.LongLength
        sha256 = Get-MvpProductInputBytesSha256 -Bytes $manifestBytes
        source_fingerprint = $sourceFingerprint
        artifacts = $resolvedArtifacts
    }
}

Export-ModuleMember -Function @(
    'Get-MvpProductInputSpecifications',
    'Get-MvpProductInputFileSha256',
    'Get-MvpSourceFingerprint',
    'Resolve-MvpProductInputManifest'
)
