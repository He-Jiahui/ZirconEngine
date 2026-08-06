$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$sourceScript = Join-Path $repoRoot 'tools\build-editor.ps1'

function Invoke-EditorBuildFixture {
    param(
        [Parameter(Mandatory = $true)]
        [string]$ScriptPath,

        [Parameter(Mandatory = $true)]
        [string]$OutputDirectory
    )

    $output = @(& powershell.exe `
        -NoProfile `
        -ExecutionPolicy Bypass `
        -File $ScriptPath `
        -OutputDirectory $OutputDirectory `
        -SkipSmokeTest 2>&1)

    return @{
        ExitCode = $LASTEXITCODE
        Output = $output
    }
}

Describe 'Editor build bundle script' {
    BeforeEach {
        $fixtureRoot = Join-Path $TestDrive ([guid]::NewGuid().ToString('N'))
        $fixtureTools = Join-Path $fixtureRoot 'tools'
        $fixtureValidatorDirectory = Join-Path $fixtureRoot '.codex\skills\zircon-dev\scripts'
        $fixtureAssets = Join-Path $fixtureRoot 'zircon_runtime\assets\fonts'
        $fixtureScript = Join-Path $fixtureTools 'build-editor.ps1'
        $fixtureValidator = Join-Path $fixtureValidatorDirectory 'validate-matrix.ps1'
        $callLog = Join-Path $fixtureRoot 'validator-calls.log'

        [System.IO.Directory]::CreateDirectory($fixtureTools) | Out-Null
        [System.IO.Directory]::CreateDirectory($fixtureValidatorDirectory) | Out-Null
        [System.IO.Directory]::CreateDirectory($fixtureAssets) | Out-Null
        Copy-Item -LiteralPath $sourceScript -Destination $fixtureScript
        Set-Content -LiteralPath (Join-Path $fixtureAssets 'fixture.txt') -Value 'asset fixture'

        @'
[CmdletBinding()]
param(
    [string]$RepoRoot,
    [string]$Package,
    [string]$Bin,
    [switch]$NoDefaultFeatures,
    [string]$Features,
    [switch]$SkipTest,
    [string]$ArtifactOutputDirectory,
    [string[]]$PublishArtifact
)

$record = '{0}|{1}|{2}|{3}|{4}|{5}' -f `
    $Package, $Bin, $NoDefaultFeatures.IsPresent, $Features, $SkipTest.IsPresent, ($PublishArtifact -join ',')
[System.IO.File]::AppendAllText($env:BUILD_EDITOR_TEST_LOG, $record + [Environment]::NewLine)

if ($env:BUILD_EDITOR_TEST_FAIL_PACKAGE -eq $Package) {
    exit 17
}

[System.IO.Directory]::CreateDirectory($ArtifactOutputDirectory) | Out-Null
foreach ($artifact in $PublishArtifact) {
    [System.IO.File]::WriteAllBytes(
        (Join-Path $ArtifactOutputDirectory $artifact),
        [System.Text.Encoding]::UTF8.GetBytes("fixture:$artifact"))
}

exit 0
'@ | Set-Content -LiteralPath $fixtureValidator -Encoding UTF8

        $env:BUILD_EDITOR_TEST_LOG = $callLog
        $env:BUILD_EDITOR_TEST_FAIL_PACKAGE = $null
    }

    AfterEach {
        $env:BUILD_EDITOR_TEST_LOG = $null
        $env:BUILD_EDITOR_TEST_FAIL_PACKAGE = $null
    }

    It 'publishes the editor, runtime DLL, and assets only after both builds succeed' {
        $bundle = Join-Path $fixtureRoot 'editor-bundle'

        $result = Invoke-EditorBuildFixture -ScriptPath $fixtureScript -OutputDirectory $bundle

        $result.ExitCode | Should Be 0
        Test-Path -LiteralPath (Join-Path $bundle 'zircon_editor.exe') | Should Be $true
        Test-Path -LiteralPath (Join-Path $bundle 'zircon_runtime.dll') | Should Be $true
        Test-Path -LiteralPath (Join-Path $bundle 'assets\fonts\fixture.txt') | Should Be $true

        $calls = @(Get-Content -LiteralPath $callLog)
        $calls.Count | Should Be 2
        $calls[0] | Should Be 'zircon_app|zircon_editor|True|target-editor-host|True|zircon_editor.exe'
        $calls[1] | Should Be 'zircon_runtime||True|target-editor-host|True|zircon_runtime.dll'
        @(Get-ChildItem -LiteralPath $fixtureRoot -Filter 'editor-bundle.partial-*').Count | Should Be 0
    }

    It 'removes its partial bundle when the runtime build fails' {
        $bundle = Join-Path $fixtureRoot 'editor-bundle'
        $env:BUILD_EDITOR_TEST_FAIL_PACKAGE = 'zircon_runtime'

        $result = Invoke-EditorBuildFixture -ScriptPath $fixtureScript -OutputDirectory $bundle

        $result.ExitCode | Should Be 1
        Test-Path -LiteralPath $bundle | Should Be $false
        @(Get-ChildItem -LiteralPath $fixtureRoot -Filter 'editor-bundle.partial-*').Count | Should Be 0
    }

    It 'does not overwrite an existing output directory' {
        $bundle = Join-Path $fixtureRoot 'editor-bundle'
        [System.IO.Directory]::CreateDirectory($bundle) | Out-Null
        Set-Content -LiteralPath (Join-Path $bundle 'sentinel.txt') -Value 'keep me'

        $result = Invoke-EditorBuildFixture -ScriptPath $fixtureScript -OutputDirectory $bundle

        $result.ExitCode | Should Be 1
        (Get-Content -LiteralPath (Join-Path $bundle 'sentinel.txt')) | Should Be 'keep me'
        Test-Path -LiteralPath $callLog | Should Be $false
    }
}
