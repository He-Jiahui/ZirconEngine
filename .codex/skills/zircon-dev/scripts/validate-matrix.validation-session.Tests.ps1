$script:ValidateMatrixValidationSessionScript = Join-Path $PSScriptRoot "validate-matrix.ps1"
$script:ValidateMatrixValidationSessionRepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..\..")).Path
$script:OriginalValidateMatrixValidationSessionTestMode = $env:VALIDATE_MATRIX_TEST_MODE
$script:OriginalValidateMatrixValidationSessionThreadId = $env:CODEX_THREAD_ID

$env:VALIDATE_MATRIX_TEST_MODE = "1"
. $script:ValidateMatrixValidationSessionScript -DryRun -SkipBuild -SkipTest
$env:VALIDATE_MATRIX_TEST_MODE = $script:OriginalValidateMatrixValidationSessionTestMode

Describe "Validate matrix session ownership" {
    It "uses a stable validation child session instead of the primary thread session" {
        try {
            $env:CODEX_THREAD_ID = "primary-session-id"

            Resolve-ValidationSessionId -RepoRoot $script:ValidateMatrixValidationSessionRepoRoot |
                Should Be "validate-matrix:primary-session-id"
        }
        finally {
            $env:CODEX_THREAD_ID = $script:OriginalValidateMatrixValidationSessionThreadId
        }
    }

    It "keeps the operational validation scope off the primary owner" {
        $source = Get-Content -Raw -Encoding UTF8 $script:ValidateMatrixValidationSessionScript

        $source | Should Match 'function Resolve-ValidationSessionId'
        $source | Should Match '\$ownerId = Resolve-ValidationSessionId -RepoRoot \$RepoRoot'
        $source | Should Match '"--write-scope", "Cargo validation"'
    }

    It "uses the validation child session for every coordinator cargo command" {
        try {
            $env:CODEX_THREAD_ID = "primary-session-id"
            $script:ValidationSessionCoordinatorCalls = [System.Collections.Generic.List[object]]::new()

            Mock Invoke-SessionCoordinatorJson {
                param([string]$RepoRoot, [string[]]$Arguments)

                $script:ValidationSessionCoordinatorCalls.Add([pscustomobject]@{
                    Arguments = @($Arguments)
                })

                if ($Arguments[0] -eq "session") {
                    return [pscustomobject]@{
                        session = [pscustomobject]@{
                            session_id = "validate-matrix:primary-session-id"
                            status = "active"
                        }
                    }
                }
                if ($Arguments[0] -eq "cargo" -and $Arguments[1] -eq "acquire") {
                    return [pscustomobject]@{
                        job = [pscustomobject]@{
                            job_id = "validation-job-id"
                            target_dir = "E:\\cargo-targets\\validation-session-test"
                            dry_run = $false
                        }
                    }
                }

                return [pscustomobject]@{}
            }

            $target = Resolve-CoordinatorCargoTarget `
                -RepoRoot $script:ValidateMatrixValidationSessionRepoRoot `
                -LaneKind "test" `
                -WorkspaceManifest "Cargo.toml" `
                -PrecomputedCompatibilityJson '{"platform":"windows","toolchain":"fixture","target_architecture":"x86_64-pc-windows-msvc","workspace":"Cargo.toml","build_config":"{}"}'
            Start-CoordinatorCargoTarget `
                -RepoRoot $script:ValidateMatrixValidationSessionRepoRoot `
                -ResolvedTarget $target
            Complete-CoordinatorCargoTarget `
                -RepoRoot $script:ValidateMatrixValidationSessionRepoRoot `
                -ResolvedTarget $target `
                -ExitCode 0 `
                -StartAttempted

            $expectedOwner = "validate-matrix:primary-session-id"
            $script:ValidationSessionCoordinatorCalls.Count | Should Be 5
            $commandSequence = @(
                "session register"
                "cargo acquire"
                "cargo start"
                "cargo finish"
                "cargo release"
            )
            for ($index = 0; $index -lt $commandSequence.Count; $index++) {
                $script:ValidationSessionCoordinatorCalls[$index].Arguments[0..1] -join " " |
                    Should Be $commandSequence[$index]
            }
            foreach ($call in $script:ValidationSessionCoordinatorCalls) {
                $ownerIndex = [Array]::IndexOf($call.Arguments, "--session-id")

                $ownerIndex | Should BeGreaterThan -1
                $call.Arguments[$ownerIndex + 1] | Should Be $expectedOwner
            }
        }
        finally {
            $env:CODEX_THREAD_ID = $script:OriginalValidateMatrixValidationSessionThreadId
        }
    }
}
