$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$scriptPath = Join-Path $repoRoot 'tools\dev-fast-build.ps1'
$aliasesPath = Join-Path $repoRoot 'tools\dev-fast-aliases.ps1'

Describe 'Dev fast build managed output policy' {
    It 'resolves the shared target through the physical Windows path resolver' {
        $source = Get-Content -Raw -Encoding UTF8 $scriptPath

        $source | Should Match 'WindowsPathResolver\.psm1'
        $source | Should Match 'function Resolve-AllowedCargoTargetPath'
        $source | Should Match 'Resolve-ZirconWindowsPath -Path \$Path'
        $source | Should Match 'must physically resolve below D:\\cargo-targets, E:\\cargo-targets, or F:\\cargo-targets'
    }

    It 'binds all Cargo compiler cache directories to the shared target and restores the caller environment' {
        $source = Get-Content -Raw -Encoding UTF8 $scriptPath

        $source | Should Match 'function Push-ManagedFastBuildEnvironment'
        $source | Should Match 'function Pop-ManagedFastBuildEnvironment'
        foreach ($name in @('CARGO_TARGET_DIR', 'CARGO_HOME', 'SCCACHE_DIR', 'TEMP', 'TMP', 'TMPDIR')) {
            $source | Should Match [regex]::Escape($name)
        }
        $source | Should Match 'cargo-home'
        $source | Should Match 'sccache'
        $source | Should Match 'temporary'
        $source | Should Match '\[System\.IO\.Directory\]::CreateDirectory\(\$resolution\.OperationalPath\)'
        $source | Should Match 'Pop-ManagedFastBuildEnvironment -Lease \$buildEnvironmentLease'
        $source | Should Match 'SetEnvironmentVariable\(''RUSTC_WRAPPER'', \$previousRustcWrapper, ''Process''\)'
    }
}

Describe 'Dev fast build sccache alias policy' {
    It 'finds the binary installed in the managed Cargo home after the build environment is restored' {
        $source = Get-Content -Raw -Encoding UTF8 $aliasesPath

        $source | Should Match 'function zr-sccache-status'
        $source | Should Match '\[string\]\$SharedTargetRoot'
        $source | Should Match 'cargo-home\\bin'
        $source | Should Match 'sccache\.exe'
    }
}
