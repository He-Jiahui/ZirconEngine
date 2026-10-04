. (Join-Path $PSScriptRoot "validation-stages.ps1")

Describe "Validation stage selection" {
    It "compiles a focused library test only through cargo test" {
        $stages = Get-ValidationStagePlan -LibTests
        $stages.Build | Should Be $false
        $stages.Test | Should Be $true
        $stages.Check | Should Be $true
    }
    It "compiles a focused integration test only through cargo test" {
        (Get-ValidationStagePlan -TestTarget scene_contract).Build | Should Be $false
    }
    It "retains the explicit product build gate" {
        (Get-ValidationStagePlan -LibTests -BuildBeforeTest).Build | Should Be $true
    }
    It "retains product builds for artifact publishing" {
        (Get-ValidationStagePlan -LibTests -PublishArtifactCount 1).Build | Should Be $true
    }
    It "retains the general workspace build and test gates" {
        $stages = Get-ValidationStagePlan
        $stages.Build | Should Be $true
        $stages.Test | Should Be $true
    }
    It "retains build-only and no-stage invocations" {
        (Get-ValidationStagePlan -SkipTest).Build | Should Be $true
        $stages = Get-ValidationStagePlan -SkipBuild -SkipTest
        $stages.Build | Should Be $false
        $stages.Test | Should Be $false
        $stages.Check | Should Be $false
    }
    It "runs only cargo check for a preflight" {
        $stages = Get-ValidationStagePlan -CheckOnly
        $stages.Build | Should Be $false
        $stages.Test | Should Be $false
        $stages.Check | Should Be $true
    }
    It "rejects contradictory stages" {
        foreach ($parameters in @(
            @{ SkipBuild = $true; BuildBeforeTest = $true },
            @{ CheckOnly = $true; SkipBuild = $true },
            @{ CheckOnly = $true; PublishArtifactCount = 1 }
        )) {
            $failure = $null
            try { Get-ValidationStagePlan @parameters | Out-Null } catch { $failure = $_ }
            $failure | Should Not BeNullOrEmpty
        }
    }
}

Describe "Validation prerequisite failure" {
    It "returns the build failure without invoking cargo test" {
        $previousMode = $env:VALIDATE_MATRIX_TEST_MODE
        $env:VALIDATE_MATRIX_TEST_MODE = "1"
        try {
            . (Join-Path $PSScriptRoot "validate-matrix.ps1") -DryRun `
                -Package zircon_runtime -LibTests -BuildBeforeTest
            Mock Invoke-Cargo { $global:LASTEXITCODE = 101 }
            $result = Invoke-ValidateMatrixMain
            $result | Should Be 1
            Assert-MockCalled Invoke-Cargo -Times 1 -Exactly
            $script:Results.Count | Should Be 1
            $script:Results[0].ExitCode | Should Be 101
        } finally {
            $env:VALIDATE_MATRIX_TEST_MODE = $previousMode
        }
    }
}
