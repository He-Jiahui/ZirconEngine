$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$resolverModule = Join-Path $repoRoot 'tools\WindowsPathResolver.psm1'

Import-Module $resolverModule -Force

Describe 'Windows path resolver' {
    It 'rejects drive-relative paths before normalizing their per-drive working directory' {
        $rejected = $false
        try {
            Resolve-ZirconWindowsPath -Path 'C:ambiguous-project-root'
        }
        catch {
            $rejected = $_.Exception.Message -match 'drive-rooted'
        }
        if (-not $rejected) {
            throw 'Drive-relative path was accepted or did not report the stable drive-rooted requirement.'
        }
    }

    It 'hides a verbatim prefix when rejecting a drive-relative path' {
        $message = $null
        try {
            Resolve-ZirconWindowsPath -Path '\\?\C:ambiguous-project-root'
        }
        catch {
            $message = $_.Exception.Message
        }

        $message | Should Be "Windows paths must be drive-rooted, not drive-relative: 'C:ambiguous-project-root'."
    }

    It 'resolves an uncreated child through its existing junction ancestor' {
        $targetDirectory = Join-Path $TestDrive 'target'
        $junctionDirectory = Join-Path $TestDrive 'junction'
        [System.IO.Directory]::CreateDirectory($targetDirectory) | Out-Null
        New-Item -ItemType Junction -Path $junctionDirectory -Target $targetDirectory | Out-Null

        $resolved = Resolve-ZirconWindowsPath -Path (Join-Path $junctionDirectory 'product-inputs')

        $resolved.OperationalPath | Should Match '^\\\\\?\\'
        ($resolved.PSObject.Properties.Name -contains 'ResolvedPath') | Should Be $false
        ($resolved.PSObject.Properties.Name -contains 'ResolvedExistingPath') | Should Be $false
        $resolved.DisplayPath | Should Be (Join-Path $targetDirectory 'product-inputs')
    }

    It 'normalizes an uncreated dotdot tail only after resolving its junction ancestor' {
        $targetDirectory = Join-Path $TestDrive 'dotdot-target'
        $junctionDirectory = Join-Path $TestDrive 'dotdot-junction'
        [System.IO.Directory]::CreateDirectory($targetDirectory) | Out-Null
        New-Item -ItemType Junction -Path $junctionDirectory -Target $targetDirectory | Out-Null
        $requestedPath = $junctionDirectory + '\uncreated-parent\..\product-inputs'

        $resolved = Resolve-ZirconWindowsPath -Path $requestedPath

        $resolved.DisplayPath | Should Be (Join-Path $targetDirectory 'product-inputs')
        $resolved.OperationalPath.Contains('\..\') | Should Be $false
    }

    It 'resolves an uncreated child through a temporary SUBST drive' {
        $targetDirectory = Join-Path $TestDrive 'subst-target'
        [System.IO.Directory]::CreateDirectory($targetDirectory) | Out-Null
        $usedDriveNames = @(Get-PSDrive -PSProvider FileSystem | ForEach-Object { $_.Name.ToUpperInvariant() })
        $driveName = @('X', 'Y', 'Z') | Where-Object { $_ -notin $usedDriveNames } | Select-Object -First 1
        if ([string]::IsNullOrWhiteSpace($driveName)) {
            throw 'No free drive letter is available for the SUBST resolver test.'
        }

        $driveRoot = "$driveName`:"
        try {
            & subst.exe $driveRoot $targetDirectory
            if ($LASTEXITCODE -ne 0) {
                throw "Could not create temporary SUBST drive $driveRoot."
            }

            $resolved = Resolve-ZirconWindowsPath -Path (Join-Path $driveRoot 'product-inputs')
        }
        finally {
            & subst.exe $driveRoot /D
        }

        $resolved.OperationalPath | Should Match '^\\\\\?\\'
        $resolved.DisplayPath | Should Be (Join-Path $targetDirectory 'product-inputs')
    }

    It 'uses one physical identity for hard-linked file aliases' {
        $source = Join-Path $TestDrive 'runtime.dll'
        $hardLink = Join-Path $TestDrive 'editor-runtime.dll'
        [IO.File]::WriteAllText($source, 'profile-artifact')
        New-Item -ItemType HardLink -Path $hardLink -Target $source | Out-Null

        (Get-ZirconWindowsFileIdentity -Path $source) | Should Be (Get-ZirconWindowsFileIdentity -Path $hardLink)
    }
}
