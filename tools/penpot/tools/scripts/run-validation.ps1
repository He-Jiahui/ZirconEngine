[CmdletBinding()]
param(
    [string]$ArtifactRoot,
    [string]$CacheSource,
    [switch]$TestsOnly,
    [switch]$BuildOnly,
    [switch]$LintOnly,
    [switch]$Offline
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
if (@(@($TestsOnly, $BuildOnly, $LintOnly) | Where-Object { $_ }).Count -gt 1) {
    throw 'Select at most one of TestsOnly, BuildOnly and LintOnly.'
}
$repository = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\..\..\..'))
if (-not $ArtifactRoot) {
    $ArtifactRoot = Join-Path 'E:\cargo-targets\zircon-local\penpot' ('validation-' + [guid]::NewGuid().ToString('N'))
}
$python = Join-Path $repository '.jenkins\runtime\python\python.exe'
if (-not (Test-Path -LiteralPath $python)) {
    $python = (Get-Command python -ErrorAction Stop).Source
}
$stageArguments = @('-B', '-X', 'utf8', (Join-Path $PSScriptRoot 'stage-workspace.py'), '--artifact-root', $ArtifactRoot)
if ($CacheSource) { $stageArguments += @('--cache-source', $CacheSource) }
$stageOutput = @(& $python @stageArguments)
if ($LASTEXITCODE -ne 0) { throw 'Penpot input staging failed.' }
$stage = ($stageOutput -join "`n") | ConvertFrom-Json
$workspace = Join-Path $stage.workspace 'tools\penpot'
$temporary = Join-Path $stage.artifactRoot 'tmp'
[IO.Directory]::CreateDirectory($temporary) | Out-Null
$env:TEMP = $temporary
$env:TMP = $temporary
$env:TMPDIR = $temporary
$env:npm_config_cache = Join-Path $stage.artifactRoot 'npm-cache'
$env:COREPACK_HOME = Join-Path $stage.artifactRoot 'corepack'
$env:PNPM_HOME = Join-Path $stage.artifactRoot 'pnpm-home'
$env:npm_config_store_dir = Join-Path $stage.artifactRoot 'pnpm-store'
Add-Content -LiteralPath (Join-Path $workspace '.npmrc') -Value ('store-dir=' + $env:npm_config_store_dir.Replace('\', '/')) -Encoding utf8
$env:XDG_CACHE_HOME = Join-Path $stage.artifactRoot 'cache'
$env:XDG_DATA_HOME = Join-Path $stage.artifactRoot 'data'
$env:XDG_STATE_HOME = Join-Path $stage.artifactRoot 'state'
$env:PLAYWRIGHT_BROWSERS_PATH = Join-Path $stage.artifactRoot 'playwright'
$env:PUPPETEER_CACHE_DIR = Join-Path $stage.artifactRoot 'puppeteer'
$env:PUPPETEER_SKIP_DOWNLOAD = 'true'
$env:ZUI_LAYOUT_REPO_ROOT = $stage.repository
$env:ZUI_PLUGIN_ARTIFACT_ROOT = $stage.artifactRoot
$env:ZUI_PLUGIN_DIST_ROOT = Join-Path $stage.artifactRoot 'zircon-zui-plugin\dist'
$env:ZUI_PLUGIN_BUNDLE_PATH = Join-Path $stage.artifactRoot 'zircon-zui-plugin\bundle\assets\plugin.js'
[IO.Directory]::CreateDirectory($env:ZUI_PLUGIN_DIST_ROOT) | Out-Null
$env:PYTHONDONTWRITEBYTECODE = '1'
$pnpm = (Get-Command pnpm -ErrorAction Stop).Source
$node = (Get-Command node -ErrorAction Stop).Source
$commands = [Collections.Generic.List[object]]::new()

function Invoke-PnpmGate {
    param([string]$Name, [string[]]$Arguments)
    $log = Join-Path $stage.artifactRoot ($Name + '.log')
    Write-Host "Running $Name in $workspace"
    & $pnpm --dir $workspace @Arguments *> $log
    $code = $LASTEXITCODE
    $commands.Add([ordered]@{ name = $Name; arguments = $Arguments; exitCode = $code; log = $log })
    Get-Content -LiteralPath $log -Tail 35
    if ($code -ne 0) { throw "$Name failed with exit code $code. Log: $log" }
}

function Invoke-NodeGate {
    param([string]$Name, [string]$Directory, [string[]]$Arguments)
    $log = Join-Path $stage.artifactRoot ($Name + '.log')
    Write-Host "Running $Name in $Directory"
    Push-Location $Directory
    try { & $node @Arguments *> $log; $code = $LASTEXITCODE }
    finally { Pop-Location }
    $commands.Add([ordered]@{ name = $Name; arguments = $Arguments; exitCode = $code; log = $log })
    Get-Content -LiteralPath $log -Tail 35
    if ($code -ne 0) { throw "$Name failed with exit code $code. Log: $log" }
}

try {
    $install = @('install', '--frozen-lockfile', '--store-dir', (Join-Path $stage.artifactRoot 'pnpm-store'))
    if ($Offline) { $install += '--offline' }
    Invoke-PnpmGate 'install' $install
    Invoke-PnpmGate 'workspace' @('list', '--recursive', '--depth', '-1')
    $app = Join-Path $workspace 'apps\zircon-zui-plugin'
    if (-not $LintOnly) {
        Invoke-NodeGate 'bundle' $workspace @('tools/scripts/build-plugin.mjs', '--plugin=zircon-zui-plugin')
    }
    if (-not $BuildOnly -and -not $LintOnly) {
        Invoke-NodeGate 'test' $app @('../../node_modules/vitest/vitest.mjs', 'run')
        Invoke-NodeGate 'host' $app @('../../node_modules/tsx/dist/cli.mjs', 'tools/penpot-runtime-host-contract.ts')
    }
    if (-not $TestsOnly -and -not $BuildOnly) {
        Invoke-NodeGate 'typecheck' $workspace @('node_modules/typescript/bin/tsc', '-b', 'apps/zircon-zui-plugin/tsconfig.json', '--pretty', 'false')
        Invoke-NodeGate 'lint' $app @('../../node_modules/eslint/bin/eslint.js', '.')
    }
    if (-not $TestsOnly -and -not $LintOnly) {
        Invoke-NodeGate 'build' $workspace @('node_modules/@angular/cli/bin/ng.js', 'build', 'zircon-zui-plugin', '--output-path', $env:ZUI_PLUGIN_DIST_ROOT, '--no-progress')
        $bundle = $env:ZUI_PLUGIN_BUNDLE_PATH
        $distributionBundle = Join-Path $env:ZUI_PLUGIN_DIST_ROOT 'assets\plugin.js'
        [IO.Directory]::CreateDirectory((Split-Path -Parent $distributionBundle)) | Out-Null
        Copy-Item -LiteralPath $bundle -Destination $distributionBundle
        if ((Get-FileHash -LiteralPath $bundle).Hash -ne (Get-FileHash -LiteralPath $distributionBundle).Hash) {
            throw 'Distribution bundle differs from the esbuild output.'
        }
    }
    $status = 'passed'
}
catch {
    $status = 'failed'
    throw
}
finally {
    $receipt = [ordered]@{ status = $status; repository = $stage.repository; workspace = $workspace; commands = @($commands.ToArray()) }
    $receipt | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $stage.artifactRoot 'validation.json') -Encoding utf8
    Write-Host "Validation receipt: $(Join-Path $stage.artifactRoot 'validation.json')"
}
