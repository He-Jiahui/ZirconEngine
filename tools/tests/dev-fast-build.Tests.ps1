$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$scriptPath = Join-Path $repoRoot 'tools\dev\dev-fast-build.ps1'
$aliasesPath = Join-Path $repoRoot 'tools\dev\dev-fast-aliases.ps1'

Describe 'Dev fast build Agent-local output policy' {
    It 'uses the independent physical-output-checked wrapper' {
        $source = Get-Content -Raw -Encoding UTF8 $scriptPath

        $source | Should Match 'local-cargo\.ps1'
        $source | Should Match 'Resolve-AgentPython'
        $source | Should Not Match 'tools[\\/]jenkins-management[\\/]entry\.py'
        $source | Should Not Match '\.jenkins[\\/]runtime[\\/]python'
        $source | Should Not Match '& cargo'
    }

    It 'preserves selectors and rejects unsupported modes instead of dropping them' {
        $source = Get-Content -Raw -Encoding UTF8 $scriptPath

        foreach ($name in @('LinkMode', 'Linker', 'StorageMode', 'RuntimeProductDll', 'LibTests', 'TestTarget', 'TestFilter')) {
            $source | Should Match ([regex]::Escape($name))
        }
        $source | Should Match "'--no-default-features'"
        $source | Should Match 'runtime-feature-presets\.toml'
        $source | Should Match 'no supported direct executor'
        $source | Should Match 'Select exactly one'
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
