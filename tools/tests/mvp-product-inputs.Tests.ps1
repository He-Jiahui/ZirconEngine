$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$builder = Join-Path $repoRoot 'tools\mvp\Build-MvpProductInputs.ps1'
$resolverModule = Join-Path $repoRoot 'tools\WindowsPathResolver.psm1'
$originalTestMode = $env:MVP_PRODUCT_INPUTS_TEST_MODE

Import-Module $resolverModule -Force

try {
    $env:MVP_PRODUCT_INPUTS_TEST_MODE = '1'
    . $builder
}
finally {
    $env:MVP_PRODUCT_INPUTS_TEST_MODE = $originalTestMode
}

Describe 'MVP product input build plan' {
    It 'keeps client and editor-host artifacts in separate feature-scoped directories' {
        $requests = @(Get-MvpProductBuildRequests)
        $runtimeExecutable = $requests | Where-Object { $_.ArtifactName -eq 'zircon_runtime.exe' }
        $runtimeLibrary = $requests | Where-Object {
            $_.ArtifactName -eq 'zircon_runtime.dll' -and $_.OutputGroup -eq 'runtime'
        }
        $editorExecutable = $requests | Where-Object { $_.ArtifactName -eq 'zircon_editor.exe' }
        $editorLibrary = $requests | Where-Object {
            $_.ArtifactName -eq 'zircon_runtime.dll' -and $_.OutputGroup -eq 'editor'
        }

        $requests.Count | Should Be 4
        $runtimeExecutable.Package | Should Be 'zircon_app'
        $runtimeExecutable.Bin | Should Be 'zircon_runtime'
        $runtimeExecutable.Features | Should Be 'target-client,platform-winit,input-gamepad,gamepad-gilrs'
        $runtimeExecutable.OutputGroup | Should Be 'runtime'
        $runtimeLibrary.Package | Should Be 'zircon_runtime'
        $runtimeLibrary.Features | Should Be 'target-client,platform-winit,input-gamepad,gamepad-gilrs'
        $editorExecutable.Package | Should Be 'zircon_app'
        $editorExecutable.Bin | Should Be 'zircon_editor'
        $editorExecutable.Features | Should Be 'target-editor-host'
        $editorExecutable.OutputGroup | Should Be 'editor'
        $editorLibrary.Package | Should Be 'zircon_runtime'
        $editorLibrary.Features | Should Be 'target-editor-host'
    }

    It 'rejects coordinator-managed drive roots before creating product inputs' {
        $messages = @('D:\', 'E:\', 'F:\') | ForEach-Object {
            try {
                Assert-MvpProductInputDirectory -Path (Join-Path $_ 'ZirconBuilds\mvp-product-inputs')
                $null
            }
            catch {
                $_.Exception.Message
            }
        }

        $messages.Count | Should Be 3
        $messages | ForEach-Object {
            $_ | Should Match 'outside coordinator-governed D/E/F roots'
        }
    }

    It 'rejects drive-relative product input paths before resolving their per-drive working directory' {
        $rejected = $false
        try {
            Assert-MvpProductInputDirectory -Path 'C:ambiguous-product-inputs'
        }
        catch {
            $rejected = $_.Exception.Message -match 'drive-rooted'
        }

        $rejected | Should Be $true
    }

    It 'resolves a junction even when its lexical path starts on C drive' {
        $targetDirectory = Join-Path $TestDrive 'reparse-target'
        $junctionDirectory = Join-Path $TestDrive 'reparse-link'
        [System.IO.Directory]::CreateDirectory($targetDirectory) | Out-Null
        New-Item -ItemType Junction -Path $junctionDirectory -Target $targetDirectory | Out-Null
        $requestedPath = Join-Path $junctionDirectory 'product-inputs'
        $resolvedPath = Assert-MvpProductInputDirectory -Path $requestedPath
        $resolution = Resolve-ZirconWindowsPath -Path $requestedPath

        $resolvedPath | Should Be $resolution.OperationalPath
        $resolution.DisplayPath | Should Be (Join-Path $targetDirectory 'product-inputs')
    }
}
