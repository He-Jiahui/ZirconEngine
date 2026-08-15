$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$stager = Join-Path $repoRoot 'tools\mvp\Stage-MvpProducts.ps1'

$stagerSource = Get-Content -LiteralPath $stager -Raw -Encoding UTF8
if ($stagerSource -notmatch '\[string\]\$ProductInputManifest') {
    throw 'MVP staging must require the source-bound ProductInputManifest input.'
}
if ($stagerSource -match '(?m)^\s*\[string\]\$EditorRuntimeLibrary\s*,?$') {
    throw 'MVP staging must not accept an editor runtime library outside ProductInputManifest.'
}
if ($stagerSource -notmatch "'runtime-library/editor'") {
    throw 'MVP staging must resolve the editor runtime library from the canonical logical product input.'
}

Write-Host 'MVP staging source-bound editor runtime library contract passed'
