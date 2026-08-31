function Test-ZirconWorkbenchPointerTransactionCounterGate {
    param(
        [string]$ProfileDir,
        [string]$ScenarioName
    )

    if ($ScenarioName.Trim().ToLowerInvariant() -ne "idle_hover") {
        return $true
    }
    $timelinePath = Join-Path $ProfileDir "timeline.zrtrace.json"
    if (-not (Test-Path $timelinePath)) {
        Write-Warning "Workbench pointer transaction gate requires a timeline artifact."
        return $false
    }

    $snapshot = Get-Content -Path $timelinePath -Raw | ConvertFrom-Json
    $transactionCount = Get-UiCounterTotal -Counters @($snapshot.counters) `
        -Names @("ui.workbench.pointer.transaction_count")
    $coalescedRefreshCount = Get-UiCounterTotal -Counters @($snapshot.counters) `
        -Names @("ui.workbench.pointer.activation_coalesced_refresh_count")
    $feedbackDeferredCount = Get-UiCounterTotal -Counters @($snapshot.counters) `
        -Names @("ui.workbench.pointer.feedback_deferred_count")
    $publicationCount = $coalescedRefreshCount + $feedbackDeferredCount

    Write-Host ("- workbench_pointer_transaction transactions={0} publications={1} feedback_deferred={2} activation_coalesced={3}" -f `
            $transactionCount, $publicationCount, $feedbackDeferredCount, $coalescedRefreshCount)
    if ($transactionCount -le 0 -or
        $publicationCount -le 0 -or
        $publicationCount -gt $transactionCount) {
        Write-Warning "Workbench pointer feedback exceeded one publication per transaction or was not exercised."
        return $false
    }
    return $true
}
