Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$bootstrapPath = Join-Path $repoRoot '.codex\sessions\tooling15-integrated-bootstrap.ps1'
$source = Get-Content -LiteralPath $bootstrapPath -Raw

Describe 'Tooling15 integrated bootstrap artifact lifecycle' {
    It 'acquires a Coordinator fixture before creating its runtime tree' {
        $source | Should Match 'MvpTestFixturePaths\.psm1'
        $source | Should Match 'New-MvpTestFixtureRoot\s+-Prefix\s+"tooling15-wave\$\{wave\}-runtime"'
        $source | Should Not Match 'Join-Path\s+''D:\\ZirconBuilds''\s+\("tooling15-wave'
    }

    It 'removes the runtime tree and releases its fixture lease on every exit path' {
        $source | Should Match '(?s)finally\s*\{.*Remove-MvpTestFixtureRoot\s+-Path\s+\$runtimeRoot'
    }

    It 'preserves the pinned runner exit code across successful cleanup' {
        $source | Should Match '\$runnerExitCode\s*=\s*\$LASTEXITCODE'
        $source | Should Match 'exit\s+\$runnerExitCode'
        $source | Should Not Match 'exit\s+\$LASTEXITCODE'
    }
}
