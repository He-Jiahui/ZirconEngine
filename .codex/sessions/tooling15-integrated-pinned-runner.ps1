$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
Set-Location 'E:\Git\ZirconEngine'

try {
    if ($env:MVP_VALIDATION_WAVE -notmatch '^\d+$') {
        throw 'Tooling15 validation wave must be numeric.'
    }
    $wave = $env:MVP_VALIDATION_WAVE
    if ($PSVersionTable.PSVersion.ToString() -ne $env:MVP_POWERSHELL_VERSION) {
        throw "Wave$wave PowerShell version mismatch: expected=$env:MVP_POWERSHELL_VERSION actual=$($PSVersionTable.PSVersion)."
    }
    Import-Module (Join-Path $env:MVP_PESTER_MODULE_ROOT "Pester\$env:MVP_PESTER_VERSION\Pester.psd1") -Force -ErrorAction Stop
    if ((Get-Module Pester).Version.ToString() -ne $env:MVP_PESTER_VERSION) {
        throw "Wave$wave Pester version mismatch: expected=$env:MVP_PESTER_VERSION actual=$((Get-Module Pester).Version)."
    }

    Write-Host "TOOLING15_WAVE${wave}_RUNTIME pwsh=$($PSVersionTable.PSVersion) pester=$((Get-Module Pester).Version)"
    $suites = @(
        '.\tools\tests\mvp-acceptance-staging-projection.Tests.ps1',
        '.\tools\tests\mvp-acceptance-staging-tree-manifest.Tests.ps1',
        '.\tools\tests\mvp-acceptance-snapshot-admission.Tests.ps1',
        '.\tools\tests\mvp-staging-tree-manifest-device-path.Tests.ps1',
        '.\tools\tests\mvp-build-set.Tests.ps1',
        '.\tools\tests\mvp-build-summary-evidence.Tests.ps1',
        '.\tools\tests\mvp-artifact-storage-policy.Tests.ps1',
        '.\tools\tests\mvp-project-copy-policy.Tests.ps1',
        '.\tools\tests\mvp-product-inputs.Tests.ps1',
        '.\tools\tests\render-extract-profiling-inputs.Tests.ps1',
        '.\tools\tests\render-extract-scale-project.Tests.ps1',
        '.\tools\tests\render-extract-baseline-capture.Tests.ps1',
        '.\tools\tests\render-extract-machine-evidence.Tests.ps1',
        '.\tools\tests\render-extract-baseline-report.Tests.ps1',
        '.\tools\tests\resource-management-baseline-plan.Tests.ps1',
        '.\tools\tests\resource-management-scale-project.Tests.ps1',
        '.\tools\tests\resource-management-baseline-report.Tests.ps1',
        '.\tools\tests\resource-management-comparison.Tests.ps1',
        '.\tools\tests\render-extract-process-lifecycle.Tests.ps1',
        '.\tools\tests\mvp-process-environment-policy.Tests.ps1',
        '.\tools\tests\mvp-process-output-capture.Tests.ps1',
        '.\tools\tests\mvp-process-lifecycle-journal.Tests.ps1',
        '.\tools\tests\mvp-process-liveness-probe.Tests.ps1',
        '.\tools\tests\mvp-staging-cancellation-request.Tests.ps1',
        '.\tools\tests\mvp-staging-terminal-receipt.Tests.ps1',
        '.\tools\tests\mvp-automation-scenario-spec.Tests.ps1',
        '.\tools\tests\mvp-scenario-registry.Tests.ps1',
        '.\tools\tests\mvp-process-qualification-context.Tests.ps1',
        '.\tools\tests\mvp-stage-job-authority.Tests.ps1',
        '.\tools\tests\mvp-run-artifact-budget.Tests.ps1',
        '.\tools\tests\staged-process-log-summary.Tests.ps1',
        '.\tools\tests\staged-process-supervisor.Tests.ps1'
    )
    $controlPlaneResult = Invoke-Pester `
        -Script $suites `
        -PassThru `
        -OutputFormat NUnitXml `
        -OutputFile (Join-Path $env:MVP_EVIDENCE_ROOT "tooling15-wave${wave}-control-plane-nunit.xml")
    if ($controlPlaneResult.FailedCount -ne 0 -or $controlPlaneResult.TotalCount -ne 361) {
        throw "Tooling15 pinned integrated contracts failed: total=$($controlPlaneResult.TotalCount), passed=$($controlPlaneResult.PassedCount), failed=$($controlPlaneResult.FailedCount)."
    }
    Write-Host "TOOLING15_INTEGRATED_PESTER total=$($controlPlaneResult.TotalCount) passed=$($controlPlaneResult.PassedCount) failed=$($controlPlaneResult.FailedCount)"

    $scriptContractResult = Invoke-Pester `
        -Script '.\tools\tests\mvp-required-script-contracts.Tests.ps1' `
        -PassThru `
        -OutputFormat NUnitXml `
        -OutputFile (Join-Path $env:MVP_EVIDENCE_ROOT "tooling15-wave${wave}-required-scripts-nunit.xml")
    if ($scriptContractResult.FailedCount -ne 0 -or $scriptContractResult.TotalCount -ne 3) {
        throw "Tooling15 required script contracts failed: total=$($scriptContractResult.TotalCount), passed=$($scriptContractResult.PassedCount), failed=$($scriptContractResult.FailedCount)."
    }
    Write-Host "TOOLING15_REQUIRED_SCRIPT_PESTER total=$($scriptContractResult.TotalCount) passed=$($scriptContractResult.PassedCount) failed=$($scriptContractResult.FailedCount)"
    Write-Host "TOOLING15_INTEGRATED_WAVE${wave}_PINNED_VALIDATION_PASSED"
}
catch {
    Write-Error $_
    exit 1
}
