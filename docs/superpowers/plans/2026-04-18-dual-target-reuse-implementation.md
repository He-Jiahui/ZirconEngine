# Dual Target Reuse Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 `validate-matrix.ps1` 增加双 slot Cargo target 自动复用逻辑，并同步更新 `zircon-dev` skill 文档，避免默认验证产物无限增长。

**Architecture:** 在验证脚本里新增可测试的 target 分配与 lease 管理函数，把默认 `cargo build/test` 统一收敛到 `target/codex-shared-a|b` 两个共享 slot。文档层同步声明默认规则、手工覆盖入口和人工命令约束。

**Tech Stack:** PowerShell 7, Pester 3.4, Cargo, repo-local Codex skills markdown

---

## Target File Structure

```text
.codex/skills/zircon-dev/scripts/
  validate-matrix.ps1
  validate-matrix.Tests.ps1

.codex/skills/zircon-dev/
  SKILL.md
  validation/SKILL.md
  validation/manual-commands.md
```

## Validation Baseline

- `Invoke-Pester '.codex/skills/zircon-dev/scripts/validate-matrix.Tests.ps1'`
- `pwsh -File '.codex/skills/zircon-dev/scripts/validate-matrix.ps1' -DryRun`

预期：先通过 red tests 证明当前脚本没有双 slot 自动分配能力，再通过最小实现转绿，并在 dry-run 输出里看到 `--target-dir`.

### Task 1: Add Red Tests For Dual Slot Allocation

**Files:**
- Create: `.codex/skills/zircon-dev/scripts/validate-matrix.Tests.ps1`
- Modify: `.codex/skills/zircon-dev/scripts/validate-matrix.ps1`

- [ ] **Step 1: Write the failing test file**

```powershell
$env:VALIDATE_MATRIX_TEST_MODE = "1"
. "$PSScriptRoot/validate-matrix.ps1"

Describe "Resolve-SharedCargoTarget" {
    It "reuses the current owner slot before claiming another slot" {
        $leases = @(
            @{ slot_name = "a"; owner_id = "thread-1"; target_dir = "target/codex-shared-a"; last_seen_utc = "2026-04-18T00:00:00Z"; repo_root = "E:/Git/ZirconEngine" },
            @{ slot_name = "b"; owner_id = "thread-2"; target_dir = "target/codex-shared-b"; last_seen_utc = "2026-04-18T00:00:00Z"; repo_root = "E:/Git/ZirconEngine" }
        )

        $result = Resolve-SharedCargoTarget -RepoRoot "E:/Git/ZirconEngine" -OwnerId "thread-1" -Leases $leases -NowUtc ([datetime]"2026-04-18T01:00:00Z")

        $result.SlotName | Should Be "a"
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```powershell
Invoke-Pester '.codex/skills/zircon-dev/scripts/validate-matrix.Tests.ps1'
```

Expected: FAIL because `Resolve-SharedCargoTarget` and test-mode-safe script loading do not exist yet.

- [ ] **Step 3: Expand the red suite to lock the whole behavior surface**

```powershell
It "claims slot b when slot a is occupied by another active owner" {
    $leases = @(
        @{ slot_name = "a"; owner_id = "thread-2"; target_dir = "target/codex-shared-a"; last_seen_utc = "2026-04-18T01:00:00Z"; repo_root = "E:/Git/ZirconEngine" },
        @{ slot_name = "b"; owner_id = $null; target_dir = "target/codex-shared-b"; last_seen_utc = $null; repo_root = "E:/Git/ZirconEngine" }
    )

    (Resolve-SharedCargoTarget -RepoRoot "E:/Git/ZirconEngine" -OwnerId "thread-1" -Leases $leases -NowUtc ([datetime]"2026-04-18T02:00:00Z")).SlotName | Should Be "b"
}

It "reclaims stale slots" {
    $leases = @(
        @{ slot_name = "a"; owner_id = "old-thread"; target_dir = "target/codex-shared-a"; last_seen_utc = "2026-04-17T00:00:00Z"; repo_root = "E:/Git/ZirconEngine" },
        @{ slot_name = "b"; owner_id = "other-thread"; target_dir = "target/codex-shared-b"; last_seen_utc = "2026-04-18T01:30:00Z"; repo_root = "E:/Git/ZirconEngine" }
    )

    (Resolve-SharedCargoTarget -RepoRoot "E:/Git/ZirconEngine" -OwnerId "thread-1" -Leases $leases -NowUtc ([datetime]"2026-04-18T14:00:00Z")).SlotName | Should Be "a"
}

It "throws when both slots are occupied by other active owners" {
    $leases = @(
        @{ slot_name = "a"; owner_id = "thread-2"; target_dir = "target/codex-shared-a"; last_seen_utc = "2026-04-18T01:00:00Z"; repo_root = "E:/Git/ZirconEngine" },
        @{ slot_name = "b"; owner_id = "thread-3"; target_dir = "target/codex-shared-b"; last_seen_utc = "2026-04-18T01:30:00Z"; repo_root = "E:/Git/ZirconEngine" }
    )

    { Resolve-SharedCargoTarget -RepoRoot "E:/Git/ZirconEngine" -OwnerId "thread-1" -Leases $leases -NowUtc ([datetime]"2026-04-18T02:00:00Z") } | Should Throw
}

It "bypasses shared slot selection when TargetDir is provided" {
    (Resolve-EffectiveTargetDir -RepoRoot "E:/Git/ZirconEngine" -ManualTargetDir "target/manual-check").SelectionMode | Should Be "manual"
}
```

- [ ] **Step 4: Re-run the tests to keep the suite red**

Run:

```powershell
Invoke-Pester '.codex/skills/zircon-dev/scripts/validate-matrix.Tests.ps1'
```

Expected: FAIL with missing functions or wrong selection behavior.

### Task 2: Implement Minimal Dual Slot Allocation In The Script

**Files:**
- Modify: `.codex/skills/zircon-dev/scripts/validate-matrix.ps1`

- [ ] **Step 1: Add test-safe bootstrap guard**

```powershell
if ($env:VALIDATE_MATRIX_TEST_MODE -ne "1") {
    Invoke-ValidateMatrixMain
}
```

- [ ] **Step 2: Add `-TargetDir` and target resolution helpers**

```powershell
param(
    [string]$RepoRoot,
    [string]$Package,
    [string]$TargetDir,
    [switch]$SkipBuild,
    [switch]$SkipTest,
    [switch]$NoLocked,
    [switch]$VerboseOutput,
    [switch]$DryRun
)
```

```powershell
function Resolve-OwnerId { param([string]$RepoRoot) }
function Resolve-SharedCargoTarget { param([string]$RepoRoot, [string]$OwnerId, [object[]]$Leases, [datetime]$NowUtc) }
function Resolve-EffectiveTargetDir { param([string]$RepoRoot, [string]$ManualTargetDir) }
```

- [ ] **Step 3: Add lease IO and lock helpers**

```powershell
function Get-TargetLeaseDirectory { param([string]$RepoRoot) }
function Read-TargetLease { param([string]$LeasePath) }
function Write-TargetLease { param([string]$LeasePath, [hashtable]$Lease) }
function Test-TargetLeaseStale { param([object]$Lease, [string]$RepoRoot, [datetime]$NowUtc) }
function Use-TargetLeaseLock { param([string]$LeaseRoot, [scriptblock]$Action) }
```

- [ ] **Step 4: Thread selected target through cargo args**

```powershell
$ResolvedTarget = Resolve-EffectiveTargetDir -RepoRoot $ResolvedRepoRoot -ManualTargetDir $TargetDir
...
$args.Add("--target-dir") | Out-Null
$args.Add($ResolvedTarget.TargetDir) | Out-Null
```

- [ ] **Step 5: Print the selected target source**

```powershell
Write-Host ("Target dir: {0} ({1})" -f $ResolvedTarget.TargetDir, $ResolvedTarget.Reason)
```

- [ ] **Step 6: Run the test suite to verify it passes**

Run:

```powershell
Invoke-Pester '.codex/skills/zircon-dev/scripts/validate-matrix.Tests.ps1'
```

Expected: PASS

### Task 3: Verify Dry-Run Behavior Against The Real Script Entry

**Files:**
- Modify: `.codex/skills/zircon-dev/scripts/validate-matrix.ps1`

- [ ] **Step 1: Run dry-run with automatic slot selection**

Run:

```powershell
pwsh -File '.codex/skills/zircon-dev/scripts/validate-matrix.ps1' -DryRun
```

Expected: output includes `Target dir:` plus a `cargo build ... --target-dir ...` and `cargo test ... --target-dir ...` command.

- [ ] **Step 2: Run dry-run with manual override**

Run:

```powershell
pwsh -File '.codex/skills/zircon-dev/scripts/validate-matrix.ps1' -DryRun -TargetDir 'target/manual-check'
```

Expected: output uses `target/manual-check` and reports manual selection instead of shared slot allocation.

- [ ] **Step 3: Fix any mismatch without weakening the tests**

```powershell
# Keep tests authoritative; only adjust script output or path resolution.
```

### Task 4: Update The Skill Documentation

**Files:**
- Modify: `.codex/skills/zircon-dev/SKILL.md`
- Modify: `.codex/skills/zircon-dev/validation/SKILL.md`
- Modify: `.codex/skills/zircon-dev/validation/manual-commands.md`

- [ ] **Step 1: Update the root skill with the high-level default**

```markdown
- Use `validation/SKILL.md` and `scripts/validate-matrix.ps1` as the default Cargo validation path; the script automatically reuses one of two shared target directories unless `-TargetDir` overrides it.
```

- [ ] **Step 2: Update `validation/SKILL.md` with the operator-facing rule**

```markdown
- By default the validator auto-selects one of `target/codex-shared-a` or `target/codex-shared-b` and reuses it per active session.
- If both shared slots are occupied by other active sessions, pass `-TargetDir` explicitly instead of inventing a third default slot.
```

- [ ] **Step 3: Update `manual-commands.md` with the manual command policy**

```markdown
- Prefer `--target-dir target/codex-shared-a` or `--target-dir target/codex-shared-b` for ad-hoc local commands.
- Do not keep minting new `target/<name>` directories for routine validation loops.
```

- [ ] **Step 4: Re-read the edited docs for internal consistency**

Run:

```powershell
Get-Content '.codex/skills/zircon-dev/SKILL.md'
Get-Content '.codex/skills/zircon-dev/validation/SKILL.md'
Get-Content '.codex/skills/zircon-dev/validation/manual-commands.md'
```

Expected: the three files describe the same two-slot default and the same `-TargetDir` override path.

### Task 5: Final Verification

**Files:**
- Modify: `.codex/skills/zircon-dev/scripts/validate-matrix.ps1`
- Modify: `.codex/skills/zircon-dev/scripts/validate-matrix.Tests.ps1`
- Modify: `.codex/skills/zircon-dev/SKILL.md`
- Modify: `.codex/skills/zircon-dev/validation/SKILL.md`
- Modify: `.codex/skills/zircon-dev/validation/manual-commands.md`

- [ ] **Step 1: Run the test suite**

```powershell
Invoke-Pester '.codex/skills/zircon-dev/scripts/validate-matrix.Tests.ps1'
```

- [ ] **Step 2: Run the validator in dry-run mode**

```powershell
pwsh -File '.codex/skills/zircon-dev/scripts/validate-matrix.ps1' -DryRun
```

- [ ] **Step 3: Summarize remaining risks**

```text
- Non-Codex shells fall back to a coarse owner id and may still need explicit -TargetDir for strict isolation.
- Lease TTL is intentionally conservative and may need tuning after real usage.
```
