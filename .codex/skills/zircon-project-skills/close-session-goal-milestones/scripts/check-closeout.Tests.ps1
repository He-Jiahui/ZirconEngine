[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$checker = Join-Path $PSScriptRoot "check-closeout.ps1"
$seed = Join-Path $PSScriptRoot "seed-closeout-test-state.py"
$projectRoot = (Resolve-Path (Join-Path $PSScriptRoot "../../../../../")).Path
$coordinator = Join-Path $projectRoot "tools/zircon-session.ps1"
$env:ZIRCON_CLOSEOUT_TEST_FIXTURE = "1"
$env:PYTHONPATH = $projectRoot

function Write-JsonFile {
    param([string]$Path, [object]$Value)
    $Value | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $Path -Encoding UTF8
}

function New-CloseoutFixture {
    param(
        [ValidateSet("Milestone", "Goal")]
        [string]$Mode = "Milestone",
        [switch]$IncompleteGoal,
        [switch]$IncompleteTesting,
        [switch]$CompletedTestingStatus,
        [switch]$PendingTableOnly,
        [switch]$BlockedTableOnly,
        [switch]$MilestoneSlicePending,
        [switch]$ChildPlanPending,
        [switch]$OmitOwnedUntracked,
        [switch]$OmitUntrackedCategory,
        [switch]$UntrackedOnlyClassification,
        [switch]$CompletedSession,
        [switch]$OpenFailure,
        [switch]$InvalidFailureMarkdown,
        [switch]$ForeignInvalidFailureMarkdown,
        [switch]$UnleasedPath,
        [switch]$DeleteOwnedCode,
        [switch]$DivergeAfterStage,
        [switch]$StageForeign,
        [switch]$AddUnownedManifestPath,
        [switch]$UseNonMain,
        [string]$CommitMessage = "【feature】feat(runtime): complete M2 milestone",
        [switch]$StageWebhook,
        [switch]$StageMaintenanceToken,
        [switch]$StageCredential,
        [switch]$StageWeComKey,
        [switch]$AddMissingManifestPath
    )

    $root = Join-Path ([IO.Path]::GetTempPath()) ("zircon-closeout-" + [guid]::NewGuid().ToString("N"))
    New-Item -ItemType Directory -Path $root -Force | Out-Null
    & git -C $root init -q
    & git -C $root config user.email "closeout@example.invalid"
    & git -C $root config user.name "Closeout Test"
    & git -C $root config core.autocrlf false
    & git -C $root branch -M main

    foreach ($relative in @(
        "src/feature.py",
        "docs/feature.md",
        "docs/foreign-staged.md",
        "docs/plans/feature/02-feature.md",
        "docs/plans/feature/02/2026-07-11-m2.md"
    )) {
        $absolute = Join-Path $root $relative
        New-Item -ItemType Directory -Path (Split-Path -Parent $absolute) -Force | Out-Null
        Set-Content -LiteralPath $absolute -Value "baseline" -Encoding UTF8
    }
    $baselinePaths = @(
        "src/feature.py", "docs/feature.md", "docs/foreign-staged.md",
        "docs/plans/feature/02-feature.md", "docs/plans/feature/02/2026-07-11-m2.md"
    )
    if ($ChildPlanPending) {
        $childPlan = Join-Path $root "docs/plans/feature/02/03-child.md"
        New-Item -ItemType Directory -Path (Split-Path -Parent $childPlan) -Force | Out-Null
        Set-Content -LiteralPath $childPlan -Value "| M3 | M3-T acceptance | 验收中 |" -Encoding UTF8
        $baselinePaths += "docs/plans/feature/02/03-child.md"
    }
    & git -C $root add -- $baselinePaths
    & git -C $root commit -q -m "test: baseline"
    & python -B $seed --repo-root $root --action init
    if ($LASTEXITCODE -ne 0) { throw "Failed to initialize coordinator fixture" }
    & python -B $seed --repo-root $root --action lease `
        --path "src/feature.py" --path "docs/feature.md" `
        --path "docs/plans/feature/02/2026-07-11-m2.md" `
        --path "tests/test_feature.py" --path "tools/check.ps1"
    if ($LASTEXITCODE -ne 0) { throw "Failed to lease coordinator fixture paths" }

    if ($DeleteOwnedCode) {
        Remove-Item -LiteralPath (Join-Path $root "src/feature.py")
    } else {
        Set-Content -LiteralPath (Join-Path $root "src/feature.py") -Value "feature" -Encoding UTF8
    }
    Set-Content -LiteralPath (Join-Path $root "docs/feature.md") -Value "evidence" -Encoding UTF8
    New-Item -ItemType Directory -Path (Join-Path $root "tests") -Force | Out-Null
    New-Item -ItemType Directory -Path (Join-Path $root "tools") -Force | Out-Null
    Set-Content -LiteralPath (Join-Path $root "tests/test_feature.py") -Value "test" -Encoding UTF8
    Set-Content -LiteralPath (Join-Path $root "tools/check.ps1") -Value "check" -Encoding UTF8
    Set-Content -LiteralPath (Join-Path $root "src/foreign.py") -Value "foreign" -Encoding UTF8

    $manifestTests = if ($OmitOwnedUntracked -or $UntrackedOnlyClassification) { @() } else { @("tests/test_feature.py") }
    $manifestUntracked = if ($OmitOwnedUntracked -or $OmitUntrackedCategory) {
        @("tools/check.ps1")
    } else {
        @("tests/test_feature.py", "tools/check.ps1")
    }
    $manifestDocs = @("docs/feature.md", "docs/plans/feature/02/2026-07-11-m2.md")
    if ($AddUnownedManifestPath) { $manifestDocs += "docs/foreign-staged.md" }
    $manifest = [ordered]@{
        session_id = "session-m2"
        mode = $Mode
        milestone_id = "M2"
        owned_dirty_paths = @(
            "src/feature.py",
            "docs/feature.md",
            "docs/plans/feature/02/2026-07-11-m2.md",
            "tests/test_feature.py",
            "tools/check.ps1"
        )
        categories = [ordered]@{
            code = @("src/feature.py")
            docs = $manifestDocs
            tests = $manifestTests
            scripts = @("tools/check.ps1")
            untracked = $manifestUntracked
        }
    }
    if ($AddMissingManifestPath) {
        $manifest.categories.code += "src/missing.py"
    }
    $manifestPath = Join-Path $root "closeout-manifest.json"
    Write-JsonFile -Path $manifestPath -Value $manifest

    $planPath = Join-Path $root "docs/plans/feature/02/2026-07-11-m2.md"
    New-Item -ItemType Directory -Path (Split-Path -Parent $planPath) -Force | Out-Null
    $testingStatus = if ($IncompleteTesting) { "待验收" } elseif ($CompletedTestingStatus) { "完成" } else { "通过" }
    $remaining = if ($IncompleteGoal -and -not $PendingTableOnly -and -not $BlockedTableOnly) { "- [ ] M3 remaining`n" } else { "" }
    $pendingRow = if ($PendingTableOnly) {
        "| M3 | M3.1 implementation | 进行中 |`n"
    } elseif ($BlockedTableOnly) {
        "| M3 | M3.1 implementation | blocked |`n"
    } elseif ($MilestoneSlicePending) {
        "| M2 | M2.1 implementation | 进行中 |`n"
    } else { "" }
    $plan = @"
# Fixture plan

- [x] M2 implementation
$remaining
| 里程碑 | 切片 | 状态 |
|---|---|---|
| M2 | M2-T acceptance | $testingStatus |
$pendingRow
"@
    Set-Content -LiteralPath $planPath -Value $plan -Encoding UTF8

    if ($StageWebhook) {
        if ($DeleteOwnedCode) { throw "Secret fixture cannot also delete the code file" }
        $hostParts = @("qyapi", "weixin", "qq", "com")
        $webhook = "https://" + ($hostParts -join ".") + "/cgi-bin/webhook/send?" + "key=fake"
        Set-Content -LiteralPath (Join-Path $root "src/feature.py") -Value $webhook -Encoding UTF8
    }
    if ($StageMaintenanceToken) {
        if ($DeleteOwnedCode) { throw "Secret fixture cannot also delete the code file" }
        $name = @("ZIRCON", "COORDINATOR", "MAINTENANCE", "TOKEN") -join "_"
        Set-Content -LiteralPath (Join-Path $root "src/feature.py") -Value ($name + "=fake") -Encoding UTF8
    }
    if ($StageCredential) {
        if ($DeleteOwnedCode) { throw "Secret fixture cannot also delete the code file" }
        $name = @("client", "secret") -join "_"
        Set-Content -LiteralPath (Join-Path $root "src/feature.py") -Value ('{"' + $name + '":"fake"}') -Encoding UTF8
    }
    if ($StageWeComKey) {
        if ($DeleteOwnedCode) { throw "Secret fixture cannot also delete the code file" }
        Set-Content -LiteralPath (Join-Path $root "src/feature.py") -Value "WECOM_WEBHOOK_KEY=fake" -Encoding UTF8
    }

    $attributedPaths = @(
        "src/feature.py",
        "docs/feature.md",
        "docs/plans/feature/02/2026-07-11-m2.md",
        "tests/test_feature.py",
        "tools/check.ps1"
    )
    $attributeArguments = @("-B", $seed, "--repo-root", $root, "--action", "attribute")
    foreach ($path in $attributedPaths) { $attributeArguments += @("--path", $path) }
    & python @attributeArguments
    if ($LASTEXITCODE -ne 0) { throw "Failed to attribute owned fixture paths" }

    $stage = @("docs/feature.md", "docs/plans/feature/02/2026-07-11-m2.md", "tools/check.ps1")
    if ($DeleteOwnedCode) {
        & git -C $root add -u -- src/feature.py
    } else {
        $stage += "src/feature.py"
    }
    if (-not $OmitOwnedUntracked) { $stage += "tests/test_feature.py" }
    & git -C $root add -- $stage
    if ($DivergeAfterStage) {
        if ($DeleteOwnedCode) { throw "Divergence fixture cannot also delete the code file" }
        Set-Content -LiteralPath (Join-Path $root "src/feature.py") -Value "newer worktree content" -Encoding UTF8
        & python -B $seed --repo-root $root --action attribute --path "src/feature.py"
        if ($LASTEXITCODE -ne 0) { throw "Failed to update divergent attribution" }
    }
    if ($StageForeign -or $AddUnownedManifestPath) {
        Set-Content -LiteralPath (Join-Path $root "docs/foreign-staged.md") -Value "foreign staged" -Encoding UTF8
        & git -C $root add docs/foreign-staged.md
    }
    if ($UseNonMain) {
        & git -C $root branch -M other
    }
    if ($CompletedSession) {
        & python -B $seed --repo-root $root --action status --status completed
        if ($LASTEXITCODE -ne 0) { throw "Failed to complete Session fixture" }
    }
    if ($OpenFailure) {
        & python -B $seed --repo-root $root --action failure
        if ($LASTEXITCODE -ne 0) { throw "Failed to seed open Failure fixture" }
    }
    if ($InvalidFailureMarkdown) {
        $failurePath = Join-Path $root "docs/plans/feature/02/failure-2026-07-11-invalid.md"
        New-Item -ItemType Directory -Path (Split-Path -Parent $failurePath) -Force | Out-Null
        Set-Content -LiteralPath $failurePath -Value "invalid handoff" -Encoding UTF8
    }
    if ($ForeignInvalidFailureMarkdown) {
        $failurePath = Join-Path $root "docs/plans/foreign/09/failure-2026-07-11-invalid.md"
        New-Item -ItemType Directory -Path (Split-Path -Parent $failurePath) -Force | Out-Null
        Set-Content -LiteralPath $failurePath -Value "invalid foreign handoff" -Encoding UTF8
    }
    if ($UnleasedPath) {
        & python -B $seed --repo-root $root --action release --path "src/feature.py"
        if ($LASTEXITCODE -ne 0) { throw "Failed to release fixture lease" }
    }
    & $coordinator start -RepoRoot $root -Json *> $null
    if ($LASTEXITCODE -ne 0) { throw "Failed to start real coordinator fixture" }

    return [pscustomobject]@{
        Root = $root
        Mode = $Mode
        Manifest = $manifestPath
        CommitMessage = $CommitMessage
    }
}

function Invoke-CloseoutCheck {
    param([object]$Fixture)
    $output = & $checker `
        -RepoRoot $Fixture.Root `
        -Mode $Fixture.Mode `
        -SessionId "session-m2" `
        -CommitMessage $Fixture.CommitMessage `
        -ManifestPath $Fixture.Manifest
    return [pscustomobject]@{
        ExitCode = $LASTEXITCODE
        Json = (($output -join "`n") | ConvertFrom-Json)
    }
}

function Assert-ErrorCode {
    param([object]$Result, [string]$Code)
    $codes = @($Result.Json.errors | ForEach-Object { [string]$_.code })
    ($codes -contains $Code) | Should Be $true
}

function Remove-CloseoutFixture {
    param([object]$Fixture)
    if ($null -eq $Fixture -or -not (Test-Path -LiteralPath $Fixture.Root)) { return }
    & $coordinator stop -RepoRoot $Fixture.Root -Json *> $null
    Remove-Item -LiteralPath $Fixture.Root -Recurse -Force
}

Describe "Session Goal milestone closeout checker" {
    AfterEach {
        if ($null -ne $script:fixture -and (Test-Path -LiteralPath $script:fixture.Root)) {
            Remove-CloseoutFixture $script:fixture
        }
        $script:fixture = $null
    }

    It "accepts an isolated accepted milestone" {
        $script:fixture = New-CloseoutFixture -Mode Milestone
        $beforeHead = (& git -C $script:fixture.Root rev-parse HEAD).Trim()
        $beforeStatus = (& git -C $script:fixture.Root status --porcelain=v1) -join "`n"
        $beforeIndex = (& git -C $script:fixture.Root diff --cached --binary) -join "`n"
        $result = Invoke-CloseoutCheck $script:fixture

        $result.ExitCode | Should Be 0
        $result.Json.status | Should Be "ok"
        $result.Json.keep_session_active | Should Be $true
        ((& git -C $script:fixture.Root rev-parse HEAD).Trim()) | Should Be $beforeHead
        ((& git -C $script:fixture.Root status --porcelain=v1) -join "`n") | Should Be $beforeStatus
        ((& git -C $script:fixture.Root diff --cached --binary) -join "`n") | Should Be $beforeIndex
    }

    It "accepts a terminal Goal" {
        $script:fixture = New-CloseoutFixture -Mode Goal
        $result = Invoke-CloseoutCheck $script:fixture

        $result.ExitCode | Should Be 0
        $result.Json.complete_goal | Should Be $true
    }

    It "accepts an attributed staged deletion" {
        $script:fixture = New-CloseoutFixture -Mode Milestone -DeleteOwnedCode
        $result = Invoke-CloseoutCheck $script:fixture

        $result.ExitCode | Should Be 0
        $result.Json.status | Should Be "ok"
    }

    It "rejects a missing manifest path" {
        $script:fixture = New-CloseoutFixture -AddMissingManifestPath
        $result = Invoke-CloseoutCheck $script:fixture

        $result.ExitCode | Should Not Be 0
        Assert-ErrorCode $result "manifest_path_missing"
    }

    It "rejects a staged path owned by another Session" {
        $script:fixture = New-CloseoutFixture -StageForeign
        $result = Invoke-CloseoutCheck $script:fixture

        $result.ExitCode | Should Not Be 0
        Assert-ErrorCode $result "staged_scope_mismatch"
    }

    It "rejects an omitted owned untracked test" {
        $script:fixture = New-CloseoutFixture -OmitOwnedUntracked
        $result = Invoke-CloseoutCheck $script:fixture

        $result.ExitCode | Should Not Be 0
        Assert-ErrorCode $result "owned_path_omitted"
    }

    It "rejects a staged addition omitted from the untracked category" {
        $script:fixture = New-CloseoutFixture -OmitUntrackedCategory
        $result = Invoke-CloseoutCheck $script:fixture

        $result.ExitCode | Should Not Be 0
        Assert-ErrorCode $result "staged_addition_not_untracked"
    }

    It "rejects staged content older than the current attributed worktree hash" {
        $script:fixture = New-CloseoutFixture -DivergeAfterStage
        $result = Invoke-CloseoutCheck $script:fixture

        $result.ExitCode | Should Not Be 0
        Assert-ErrorCode $result "staged_content_not_attributed"
    }

    It "rejects a non-main checkout" {
        $script:fixture = New-CloseoutFixture -UseNonMain
        $result = Invoke-CloseoutCheck $script:fixture

        $result.ExitCode | Should Not Be 0
        Assert-ErrorCode $result "not_on_main"
    }

    It "rejects Session-tag and checkpoint commit subjects" {
        foreach ($message in @(
            "feat(m2): close [zircon-session:abc]",
            "chore: checkpoint M2",
            "feat(m2): close`n`n[zircon-session:abc]"
        )) {
            $script:fixture = New-CloseoutFixture -CommitMessage $message
            $result = Invoke-CloseoutCheck $script:fixture
            $result.ExitCode | Should Not Be 0
            Assert-ErrorCode $result "invalid_commit_message"
            Remove-CloseoutFixture $script:fixture
            $script:fixture = $null
        }
    }

    It "rejects missing or mismatched plan-module commit prefixes" {
        foreach ($message in @(
            "feat(runtime): complete M2 milestone",
            "【runtime】feat(runtime): complete M2 milestone"
        )) {
            $script:fixture = New-CloseoutFixture -CommitMessage $message
            $result = Invoke-CloseoutCheck $script:fixture
            $result.ExitCode | Should Not Be 0
            Assert-ErrorCode $result "invalid_commit_module"
            Remove-CloseoutFixture $script:fixture
            $script:fixture = $null
        }
    }

    It "rejects staged webhook, maintenance capability, and credential material" {
        foreach ($switchName in @("StageWebhook", "StageMaintenanceToken", "StageCredential", "StageWeComKey")) {
            $arguments = @{}
            $arguments[$switchName] = $true
            $script:fixture = New-CloseoutFixture @arguments
            $result = Invoke-CloseoutCheck $script:fixture
            $result.ExitCode | Should Not Be 0
            Assert-ErrorCode $result "sensitive_staged_content"
            Remove-CloseoutFixture $script:fixture
            $script:fixture = $null
        }
    }

    It "rejects terminal Goal closeout without aggregate completion" {
        $script:fixture = New-CloseoutFixture -Mode Goal -IncompleteGoal
        $result = Invoke-CloseoutCheck $script:fixture

        $result.ExitCode | Should Not Be 0
        Assert-ErrorCode $result "goal_incomplete"
    }

    It "rejects a milestone whose testing stage is not accepted" {
        $script:fixture = New-CloseoutFixture -Mode Milestone -IncompleteTesting
        $result = Invoke-CloseoutCheck $script:fixture

        $result.ExitCode | Should Not Be 0
        Assert-ErrorCode $result "testing_stage_incomplete"
    }

    It "rejects a milestone with a pending implementation slice" {
        $script:fixture = New-CloseoutFixture -Mode Milestone -MilestoneSlicePending
        $result = Invoke-CloseoutCheck $script:fixture

        $result.ExitCode | Should Not Be 0
        Assert-ErrorCode $result "milestone_incomplete"
    }

    It "accepts the standard Chinese completed testing-stage status" {
        $script:fixture = New-CloseoutFixture -Mode Milestone -CompletedTestingStatus
        $result = Invoke-CloseoutCheck $script:fixture

        $result.ExitCode | Should Be 0
    }

    It "rejects terminal Goal closeout with an in-progress table row" {
        $script:fixture = New-CloseoutFixture -Mode Goal -IncompleteGoal -PendingTableOnly
        $result = Invoke-CloseoutCheck $script:fixture

        $result.ExitCode | Should Not Be 0
        Assert-ErrorCode $result "goal_incomplete"
    }

    It "rejects terminal Goal closeout with a blocked table row" {
        $script:fixture = New-CloseoutFixture -Mode Goal -IncompleteGoal -BlockedTableOnly
        $result = Invoke-CloseoutCheck $script:fixture

        $result.ExitCode | Should Not Be 0
        Assert-ErrorCode $result "goal_incomplete"
    }

    It "ignores historical pending text when current staged Goal evidence is complete" {
        $script:fixture = New-CloseoutFixture -Mode Goal -ChildPlanPending
        $result = Invoke-CloseoutCheck $script:fixture

        $result.ExitCode | Should Be 0
        $result.Json.complete_goal | Should Be $true
    }

    It "rejects a completed Session reopening another closeout" {
        $script:fixture = New-CloseoutFixture -Mode Goal -CompletedSession
        $result = Invoke-CloseoutCheck $script:fixture

        $result.ExitCode | Should Not Be 0
        Assert-ErrorCode $result "session_not_closeable"
    }

    It "rejects closeout while the registered plan has an open Failure" {
        $script:fixture = New-CloseoutFixture -Mode Milestone -OpenFailure
        $result = Invoke-CloseoutCheck $script:fixture

        $result.ExitCode | Should Not Be 0
        Assert-ErrorCode $result "open_failure_remaining"
    }

    It "rejects canonical Failure Markdown diagnostics not yet imported into SQLite" {
        $script:fixture = New-CloseoutFixture -Mode Milestone -InvalidFailureMarkdown
        $result = Invoke-CloseoutCheck $script:fixture

        $result.ExitCode | Should Not Be 0
        Assert-ErrorCode $result "failure_graph_invalid"
    }

    It "reports but does not block an unrelated plan Failure diagnostic" {
        $script:fixture = New-CloseoutFixture -Mode Milestone -ForeignInvalidFailureMarkdown
        $result = Invoke-CloseoutCheck $script:fixture

        $result.ExitCode | Should Be 0
        @($result.Json.foreign_failure_diagnostics).Count | Should BeGreaterThan 0
    }

    It "rejects a manifest path without a live owned lease" {
        $script:fixture = New-CloseoutFixture -Mode Milestone -UnleasedPath
        $result = Invoke-CloseoutCheck $script:fixture

        $result.ExitCode | Should Not Be 0
        Assert-ErrorCode $result "manifest_path_not_leased"
    }

    It "rejects closeout when the real coordinator is offline" {
        $script:fixture = New-CloseoutFixture -Mode Milestone
        & $coordinator stop -RepoRoot $script:fixture.Root -Json *> $null
        $result = Invoke-CloseoutCheck $script:fixture

        $result.ExitCode | Should Not Be 0
        Assert-ErrorCode $result "checker_failure"
    }

    It "requires every untracked file to have a content category" {
        $script:fixture = New-CloseoutFixture -Mode Milestone -UntrackedOnlyClassification
        $result = Invoke-CloseoutCheck $script:fixture

        $result.ExitCode | Should Not Be 0
        Assert-ErrorCode $result "untracked_content_category_missing"
    }

    It "rejects a manifest path without current-hash Session ownership" {
        $script:fixture = New-CloseoutFixture -Mode Milestone -AddUnownedManifestPath
        $result = Invoke-CloseoutCheck $script:fixture

        $result.ExitCode | Should Not Be 0
        Assert-ErrorCode $result "manifest_path_not_owned"
    }
}
