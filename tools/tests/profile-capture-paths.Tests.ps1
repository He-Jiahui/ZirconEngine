# 采集路径解析只接受批准根下的单级子名；隔离夹具验证祖先 junction 和路径分隔拒绝，防止 profile 产物逃出预定根。
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
. (Join-Path $repoRoot 'tools\analysis\profiling\shared\profile-capture-paths.ps1')

Describe 'Zircon profile capture paths' {
    It 'resolves plain child components beneath the approved root' {
        $profileRoot = Join-Path $TestDrive 'profile-root'
        [IO.Directory]::CreateDirectory($profileRoot) | Out-Null

        $resolved = Resolve-ZirconProfileContainedPath `
            -Root $profileRoot `
            -PathSegments @('profile-projects', 'startup-measured-01', 'ProfileCaptureProject')

        $resolved | Should Be (Join-Path $profileRoot 'profile-projects\startup-measured-01\ProfileCaptureProject')
    }

    It 'rejects non-leaf child components before path composition' {
        $profileRoot = Join-Path $TestDrive 'profile-root'
        [IO.Directory]::CreateDirectory($profileRoot) | Out-Null

        { Resolve-ZirconProfileContainedPath -Root $profileRoot -PathSegments @('..') } |
            Should Throw "Profile path component must be a plain child name: '..'."
        { Resolve-ZirconProfileContainedPath -Root $profileRoot -PathSegments @('nested\child') } |
            Should Throw "Profile path component must be a plain child name: 'nested\child'."
        { Resolve-ZirconProfileContainedPath -Root $profileRoot -PathSegments @('CON') } |
            Should Throw "Profile path component must not be a reserved Windows device name: 'CON'."
    }

    It 'rejects a profile child whose existing ancestor is a junction' {
        $profileRoot = Join-Path $TestDrive 'profile-root'
        $outsideRoot = Join-Path $TestDrive 'outside-root'
        $junction = Join-Path $profileRoot 'profile-projects'
        [IO.Directory]::CreateDirectory($profileRoot) | Out-Null
        [IO.Directory]::CreateDirectory($outsideRoot) | Out-Null
        New-Item -ItemType Junction -Path $junction -Target $outsideRoot | Out-Null
        ([bool]((Get-Item -LiteralPath $junction -Force).Attributes -band [IO.FileAttributes]::ReparsePoint)) |
            Should Be $true

        {
            Resolve-ZirconProfileContainedPath `
                -Root $profileRoot `
                -PathSegments @('profile-projects', 'startup-measured-01')
        } | Should Throw "Profile capture path contains a reparse point: $junction"
    }
}

function New-ZirconProfilePathOwnedRoot {
    $parent = Resolve-ZirconProfileOutputRoot -RepoRoot $PSScriptRoot -Path 'D:\cargo-targets\zircon-local\evidence\profile-output-contract-fixtures'
    [IO.Directory]::CreateDirectory($parent) | Out-Null
    $root = Resolve-ZirconProfileContainedPath -Root $parent `
        -PathSegments @('path-' + [guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $root -ErrorAction Stop | Out-Null
    return $root
}

function Remove-ZirconProfilePathOwnedRoot {
    param([Parameter(Mandatory = $true)][string]$Root)

    $parent = Resolve-ZirconProfileOutputRoot -RepoRoot $PSScriptRoot -Path 'D:\cargo-targets\zircon-local\evidence\profile-output-contract-fixtures'
    $resolvedRoot = [IO.Path]::GetFullPath($Root).TrimEnd([char[]]"\/")
    if (-not [IO.Path]::GetDirectoryName($resolvedRoot).Equals($parent, [StringComparison]::OrdinalIgnoreCase) -or
        -not [IO.Path]::GetFileName($resolvedRoot).StartsWith('path-', [StringComparison]::Ordinal)) {
        throw "Refusing fixture cleanup outside its owned direct child: $Root"
    }
    Assert-ZirconProfilePathContainsNoReparsePoint -Root $parent -Candidate $resolvedRoot
    if (Test-Path -LiteralPath $resolvedRoot) {
        Remove-Item -LiteralPath $resolvedRoot -Recurse -Force
    }
}

Describe 'Zircon profile physical output roots' {
    It 'admits only approved roots on available physical drives without creating output' {
        $available = 0
        foreach ($drive in @('D', 'E', 'F')) {
            if (-not (Test-Path -LiteralPath "${drive}:\" -PathType Container)) {
                continue
            }
            $available++
            $root = "${drive}:\cargo-targets\zircon-local"
            Resolve-ZirconProfileOutputRoot -RepoRoot $repoRoot -Path $root | Should Be $root
            $candidate = $root + '\profiles\a-physical-output.json'
            Resolve-ZirconProfileOutputRoot -RepoRoot $repoRoot -Path $candidate | Should Be $candidate
        }
        $available | Should BeGreaterThan 0
    }

    It 'rejects aliases and every external or lookalike output root' {
        $expectedMessages = @{
            'D:\cargo-targets\zircon-local\..\escape' = "Profile path component must be a plain child name: '..'."
            'D:\cargo-targets\zircon-local\profiles\.' = "Profile path component must be a plain child name: '.'."
            'D:\cargo-targets\zircon-local\profiles.\trace.json' = "Profile path component must be a literal Windows child name: 'profiles.'."
            'D:\cargo-targets\zircon-local\profiles \trace.json' = "Profile path component must be a literal Windows child name: 'profiles '."
            'D:\cargo-targets\zircon-local\profiles\trace.json:stream' = "Profile path component must be a literal Windows child name: 'trace.json:stream'."
            'D:\cargo-targets\zircon-local\profiles\NUL.json' = "Profile path component must not be a reserved Windows device name: 'NUL.json'."
        }
        foreach ($candidate in @(
                'C:\cargo-targets\zircon-local\profiles',
                'E:\zircon-profiles', 'E:\Git\ZirconEngine\target\profiles',
                'target\profiles', 'D:relative', '\cargo-targets\zircon-local',
                '\\server\share\cargo-targets\zircon-local',
                '\\?\D:\cargo-targets\zircon-local\profiles',
                '\\.\D:\cargo-targets\zircon-local\profiles',
                'D:\targets\profiles', 'E:\ZirconBuilds\profiles',
                'F:\nested\cargo-targets\zircon-local\profiles',
                'D:\cargo-targets-other\zircon-local\profiles',
                'D:\cargo-targets\zircon-local-other\profiles',
                'D:\cargo-targets\zircon-local\..\escape',
                'D:\cargo-targets\zircon-local\profiles\.',
                'D:\cargo-targets\zircon-local\profiles.\trace.json',
                'D:\cargo-targets\zircon-local\profiles \trace.json',
                'D:\cargo-targets\zircon-local\profiles\trace.json:stream',
                'D:\cargo-targets\zircon-local\profiles\NUL.json')) {
            $expectedMessage = 'Profile output root must be an absolute literal path beneath physical D/E/F:\cargo-targets\zircon-local.'
            if ($expectedMessages.ContainsKey($candidate)) {
                $expectedMessage = $expectedMessages[$candidate]
            }
            { Resolve-ZirconProfileOutputRoot -RepoRoot $repoRoot -Path $candidate } |
                Should Throw $expectedMessage
        }
    }

    It 'rejects an approved-looking root below an existing junction ancestor' {
        $fixture = New-ZirconProfilePathOwnedRoot
        $junction = Join-Path $fixture 'ancestor'
        $createdJunction = $false
        try {
            $target = Join-Path $fixture 'real-directory'
            [IO.Directory]::CreateDirectory((Join-Path $target 'profile-root')) | Out-Null
            New-Item -ItemType Junction -Path $junction -Target $target -ErrorAction Stop | Out-Null
            $createdJunction = $true
            $boundary = Join-Path $junction 'profile-root'
            { Assert-ZirconProfilePathContainsNoReparsePoint -Root $boundary -Candidate (Join-Path $boundary 'trace.json') } |
                Should Throw 'contains a reparse point'
            # Its resolved destination is also inside the approved physical D: root.
            { Resolve-ZirconProfileOutputRoot -RepoRoot $repoRoot -Path (Join-Path $junction 'trace.json') } |
                Should Throw 'contains a reparse point'
        }
        finally {
            if ($createdJunction) {
                [IO.Directory]::Delete($junction)
            }
            Remove-ZirconProfilePathOwnedRoot -Root $fixture
        }
    }
}

Describe 'Zircon profile alias identity rejection' {
    It 'rejects a provider drive whose root is an external alias' {
        Mock Get-PSDrive { return [pscustomobject]@{ Root = 'C:\'; DisplayRoot = '' } }
        { Resolve-ZirconProfileOutputRoot -RepoRoot $repoRoot -Path 'D:\cargo-targets\zircon-local\profiles' } |
            Should Throw 'provider drive aliases are not allowed'
    }
}

Describe 'Zircon profile physical file alias rejection' {
    It 'rejects a physical file with another hard link before allowing an output overwrite' {
        $fixture = New-ZirconProfilePathOwnedRoot
        try {
            $target = Join-Path $fixture 'existing-artifact.json'
            $alias = Join-Path $fixture 'artifact-alias.json'
            [IO.File]::WriteAllText($target, 'fixture-owned data')
            New-Item -ItemType HardLink -Path $alias -Target $target -ErrorAction Stop | Out-Null
            { Resolve-ZirconProfileOutputRoot -RepoRoot $repoRoot -Path $alias } |
                Should Throw 'file aliases are not allowed'
            [IO.File]::ReadAllText($target) | Should Be 'fixture-owned data'
            { Resolve-ZirconProfileContainedPath -Root $fixture -PathSegments @('artifact-alias.json') } |
                Should Throw 'file aliases are not allowed'
        }
        finally {
            Remove-ZirconProfilePathOwnedRoot -Root $fixture
        }
    }
}
