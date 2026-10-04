$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$scriptPath = Join-Path $repoRoot 'tools\dev\dev-fast-build.ps1'
$aliasesPath = Join-Path $repoRoot 'tools\dev\dev-fast-aliases.ps1'

Describe 'Dev fast build managed output policy' {
    It 'delegates target allocation to the coordinator validator' {
        $source = Get-Content -Raw -Encoding UTF8 $scriptPath

        $source | Should Match 'validate-matrix\.ps1'
        $source | Should Match 'coordinator compatibility pools'
        $source | Should Not Match '& cargo'
    }

    It 'forwards explicit link and storage modes through the common entry' {
        $source = Get-Content -Raw -Encoding UTF8 $scriptPath

        foreach ($name in @('LinkMode', 'Linker', 'StorageMode', 'RuntimeProductDll', 'LibTests', 'TestTarget', 'TestFilter')) {
            $source | Should Match ([regex]::Escape($name))
        }
        $source | Should Match 'NoDefaultFeatures = \$true'
        $source | Should Match "'test' \{"
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
