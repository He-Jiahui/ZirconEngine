$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
. (Join-Path $repoRoot 'tools\profile-capture-paths.ps1')

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
