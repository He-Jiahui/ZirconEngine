$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$stager = Join-Path $repoRoot 'tools\mvp\Stage-MvpProducts.ps1'

$rejected = $false
try {
    & $stager `
        -RuntimeExecutable 'runtime.exe' `
        -EditorExecutable 'editor.exe' `
        -RuntimeLibrary 'runtime.dll' `
        -EditorRuntimeLibrary '' `
        -TemplateRoot 'templates' `
        -EngineAssetRoot 'assets' `
        -NoLaunch `
        -AllowUnsafeStagingRoot | Out-Null
}
catch {
    $rejected = $_.Exception.Message -match 'EditorRuntimeLibrary'
}

if (-not $rejected) {
    throw 'An empty EditorRuntimeLibrary must be rejected before staging resolves other inputs.'
}

Write-Host 'MVP staging editor runtime library contract passed'
