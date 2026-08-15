Set-StrictMode -Version Latest

$windowsPathResolverModule = Join-Path $PSScriptRoot '..\WindowsPathResolver.psm1'
Import-Module $windowsPathResolverModule -Force -ErrorAction Stop

$script:ZirconSessionScript = Join-Path $PSScriptRoot '..\zircon-session.ps1'
$script:MvpTestFixtureLeases = @{}

function Invoke-MvpFixtureCoordinator {
    param(
        [Parameter(Mandatory)]
        [string[]]$Arguments
    )

    $output = @(& $script:ZirconSessionScript @Arguments -Json)
    if ($LASTEXITCODE -ne 0) {
        throw "Coordinator fixture command failed with exit code ${LASTEXITCODE}: $($output -join ' ')"
    }
    $json = $output -join "`n"
    if ([string]::IsNullOrWhiteSpace($json)) {
        throw 'Coordinator fixture command returned no JSON result.'
    }
    try {
        return $json | ConvertFrom-Json -ErrorAction Stop
    }
    catch {
        throw "Coordinator fixture command returned invalid JSON: $($_.Exception.Message)"
    }
}

function New-MvpTestFixtureRoot {
    param(
        [Parameter(Mandatory)]
        [ValidatePattern('^[A-Za-z0-9][A-Za-z0-9._-]*$')]
        [string]$Prefix
    )

    $fixtureParentName = "mvp-test-fixtures-$PID"
    $leaseId = $null
    $fixtureDisplayPath = $null
    try {
        $response = Invoke-MvpFixtureCoordinator -Arguments @(
            'artifact',
            'fixture-acquire',
            '--prefix',
            $Prefix,
            '--owner-pid',
            [string]$PID
        )
        $lease = $response.lease
        if ($null -eq $lease) {
            throw 'Coordinator fixture acquire response omitted its lease.'
        }
        $leaseId = [string]$lease.leaseId
        $fixtureDisplayPath = [string]$lease.path
        if ($leaseId -notmatch '^[0-9a-f]{32}$') {
            throw "Coordinator fixture acquire returned an invalid lease ID: $leaseId"
        }
        if ([int]$lease.ownerPid -ne $PID -or [string]$lease.status -ne 'active') {
            throw 'Coordinator fixture acquire returned a lease for another owner or lifecycle state.'
        }
        $fixtureDisplayPattern = '^[D-F]:\\ZirconBuilds\\' +
            [regex]::Escape($fixtureParentName) + '\\' +
            [regex]::Escape($Prefix) + '-' + [regex]::Escape($leaseId) + '$'
        if ($fixtureDisplayPath -notmatch $fixtureDisplayPattern) {
            throw "Coordinator fixture path is outside the approved process root: $fixtureDisplayPath"
        }

        $script:MvpTestFixtureLeases[$fixtureDisplayPath] = $leaseId
        $fixtureParentDisplayPath = Split-Path -Parent $fixtureDisplayPath
        $artifactRoot = Split-Path -Parent $fixtureParentDisplayPath
        $artifactRootResolution = Resolve-ZirconWindowsPath -Path $artifactRoot
        if (-not $artifactRootResolution.DisplayPath.Equals($artifactRoot, [StringComparison]::OrdinalIgnoreCase)) {
            throw "approved fixture root resolves outside its physical root: $($artifactRootResolution.DisplayPath)"
        }
        $fixtureParent = Join-ZirconWindowsPath `
            -Path $artifactRootResolution.OperationalPath `
            -ChildPath $fixtureParentName
        [IO.Directory]::CreateDirectory($fixtureParent) | Out-Null
        $fixtureParentResolution = Resolve-ZirconWindowsPath -Path $fixtureParent
        if (-not $fixtureParentResolution.DisplayPath.Equals($fixtureParentDisplayPath, [StringComparison]::OrdinalIgnoreCase)) {
            throw "fixture parent resolves outside the Coordinator-issued physical root: $($fixtureParentResolution.DisplayPath)"
        }
        $fixtureRoot = Join-ZirconWindowsPath `
            -Path $fixtureParentResolution.OperationalPath `
            -ChildPath ("$Prefix-$leaseId")
        [IO.Directory]::CreateDirectory($fixtureRoot) | Out-Null
        $fixtureResolution = Resolve-ZirconWindowsPath -Path $fixtureRoot
        if (-not $fixtureResolution.DisplayPath.Equals($fixtureDisplayPath, [StringComparison]::OrdinalIgnoreCase)) {
            throw "resolved fixture root differs from the Coordinator-issued path: $($fixtureResolution.DisplayPath)"
        }
        return $fixtureResolution.DisplayPath
    }
    catch {
        $primaryMessage = $_.Exception.Message
        if ($null -ne $fixtureDisplayPath -and $script:MvpTestFixtureLeases.ContainsKey($fixtureDisplayPath)) {
            try {
                Remove-MvpTestFixtureRoot -Path $fixtureDisplayPath
            }
            catch {
                throw "Could not create the Coordinator-managed MVP fixture root: $primaryMessage; cleanup also failed: $($_.Exception.Message)"
            }
        }
        throw "Could not create the Coordinator-managed MVP fixture root: $primaryMessage"
    }
}

function Remove-MvpTestFixtureRoot {
    param(
        [Parameter(Mandatory)]
        [string]$Path
    )

    $resolution = Resolve-ZirconWindowsPath -Path $Path
    $operationPath = $resolution.OperationalPath
    $displayPath = $resolution.DisplayPath
    $fixtureParentName = "mvp-test-fixtures-$PID"
    $fixtureDisplayPattern = '^[D-F]:\\ZirconBuilds\\' + [regex]::Escape($fixtureParentName) + '\\[A-Za-z0-9][A-Za-z0-9._-]*-[0-9a-f]{32}$'
    if ($displayPath -notmatch $fixtureDisplayPattern) {
        throw "Refusing to remove fixture outside the approved MVP fixture root: $displayPath"
    }
    $leaseId = if ($script:MvpTestFixtureLeases.ContainsKey($displayPath)) {
        [string]$script:MvpTestFixtureLeases[$displayPath]
    }
    else {
        $null
    }
    $fixtureParent = [IO.Directory]::GetParent($operationPath.TrimEnd([IO.Path]::DirectorySeparatorChar))
    if ($null -eq $fixtureParent) {
        throw "Refusing to remove fixture without a process-scoped parent: $displayPath"
    }
    $fixtureParentResolution = Resolve-ZirconWindowsPath -Path $fixtureParent.FullName
    $fixtureParentDisplayPattern = '^[D-F]:\\ZirconBuilds\\' + [regex]::Escape($fixtureParentName) + '$'
    if ($fixtureParentResolution.DisplayPath -notmatch $fixtureParentDisplayPattern) {
        throw "Refusing to remove fixture parent outside the approved MVP fixture root: $($fixtureParentResolution.DisplayPath)"
    }
    if ([IO.Directory]::Exists($fixtureParentResolution.OperationalPath)) {
        $parentAttributes = [IO.File]::GetAttributes($fixtureParentResolution.OperationalPath)
        if ([bool]($parentAttributes -band [IO.FileAttributes]::ReparsePoint)) {
            throw "Refusing to remove a reparse-point fixture parent: $($fixtureParentResolution.DisplayPath)"
        }
    }

    if ([IO.Directory]::Exists($operationPath)) {
        $attributes = [IO.File]::GetAttributes($operationPath)
        if ([bool]($attributes -band [IO.FileAttributes]::ReparsePoint)) {
            throw "Refusing to remove a reparse-point fixture root: $displayPath"
        }
        Remove-MvpTestFixtureDirectory -Path $operationPath
    }

    if ([IO.Directory]::Exists($fixtureParentResolution.OperationalPath)) {
        $remainingEntries = [IO.Directory]::EnumerateFileSystemEntries($fixtureParentResolution.OperationalPath).GetEnumerator()
        try {
            if (-not $remainingEntries.MoveNext()) {
                [IO.Directory]::Delete($fixtureParentResolution.OperationalPath, $false)
            }
        }
        finally {
            $remainingEntries.Dispose()
        }
    }

    if ($null -ne $leaseId) {
        $response = Invoke-MvpFixtureCoordinator -Arguments @(
            'artifact',
            'fixture-release',
            '--lease-id',
            $leaseId,
            '--owner-pid',
            [string]$PID
        )
        if (
            $null -eq $response.lease -or
            [string]$response.lease.leaseId -ne $leaseId -or
            [string]$response.lease.status -ne 'released'
        ) {
            throw "Coordinator fixture release returned an invalid lifecycle result for lease $leaseId."
        }
        $script:MvpTestFixtureLeases.Remove($displayPath)
    }
}

function Remove-MvpTestFixtureDirectory {
    param(
        [Parameter(Mandatory)]
        [string]$Path
    )

    $directoryAttributes = [IO.File]::GetAttributes($Path)
    if ([bool]($directoryAttributes -band [IO.FileAttributes]::ReparsePoint)) {
        $displayPath = (Resolve-ZirconWindowsPath -Path $Path).DisplayPath
        throw "Refusing to recurse into a reparse-point fixture directory: $displayPath"
    }
    foreach ($childPath in [IO.Directory]::EnumerateFileSystemEntries($Path)) {
        $childAttributes = [IO.File]::GetAttributes($childPath)
        if ([bool]($childAttributes -band [IO.FileAttributes]::Directory)) {
            if ([bool]($childAttributes -band [IO.FileAttributes]::ReparsePoint)) {
                # A junction is a leaf entry for fixture cleanup; never recurse into its target.
                [IO.Directory]::Delete($childPath, $false)
            }
            else {
                Remove-MvpTestFixtureDirectory -Path $childPath
            }
        }
        else {
            [IO.File]::Delete($childPath)
        }
    }
    [IO.Directory]::Delete($Path, $false)
}

Export-ModuleMember -Function @('New-MvpTestFixtureRoot', 'Remove-MvpTestFixtureRoot')
