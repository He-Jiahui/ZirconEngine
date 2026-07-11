# Local Session Coordinator Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a user-level local service that coordinates every ZirconEngine Session in the shared `main` workspace, preserves business intermediate versions outside Git, prevents blind file overwrites, governs plan/failure/Cargo concurrency, and creates a normal Git commit only after an explicit finalize request.

**Architecture:** A Python standard-library daemon owns SQLite WAL state, content-addressed snapshots, leases, delayed patches, Failure graph, Cargo lanes, baselines, cleanup, and Git finalize transactions. PowerShell provides the Windows client and scheduled-task lifecycle; repository-local skills route Sessions through the service while retaining Markdown plan artifacts as durable truth.

**Tech Stack:** Python 3 standard library (`sqlite3`, `http.server`, `hashlib`, `zlib`, `subprocess`, `unittest`), PowerShell 5+/7, SQLite WAL, Git CLI, Cargo, Windows Task Scheduler, Markdown/YAML workflow skills.

**Approved design:** `docs/superpowers/specs/2026-07-11-local-session-coordinator-design.md`

**Repository constraints:** Execute from the existing `main` checkout. Do not create a branch, worktree, hidden checkpoint commit, stash-based version store, or repo-local Cargo target. Workflow infrastructure is versioned normally; business Session intermediate versions remain service-managed until an explicit user-requested finalize.

---

## File Structure

### Service core

- Create `tools/session_coordinator/__init__.py`: package version and public entrypoint metadata only.
- Create `tools/session_coordinator/__main__.py`: dispatch `serve` and CLI commands.
- Create `tools/session_coordinator/config.py`: repo discovery, local state path, target-root allowlist, TTLs and retention defaults.
- Create `tools/session_coordinator/models.py`: enums and immutable command/result records shared by service modules.
- Create `tools/session_coordinator/database.py`: SQLite connection, WAL pragmas and transaction helper.
- Create `tools/session_coordinator/migrations.py`: idempotent schema versions and startup migration runner.
- Create `tools/session_coordinator/server.py`: authenticated loopback HTTP service, single-instance lock, runtime descriptor and health endpoint.
- Create `tools/session_coordinator/client.py`: token-aware local HTTP client with structured errors.
- Create `tools/session_coordinator/cli.py`: human/JSON command surface and exit-code contract.

### Coordination domains

- Create `tools/session_coordinator/sessions.py`: Session registration, heartbeat, enum transitions, stale/archive lifecycle.
- Create `tools/session_coordinator/baselines.py`: HEAD/index/worktree epoch, file hashes, attribution and degraded/reconcile behavior.
- Create `tools/session_coordinator/snapshots.py`: SHA-256/zlib object store, manifests, preview restore and retention references.
- Create `tools/session_coordinator/leases.py`: normalized path claims, atomic multi-file acquisition, TTL and renewal.
- Create `tools/session_coordinator/patches.py`: Git-compatible patch preflight, FIFO delay, hash recheck, apply and `needs_rebase` artifacts.
- Create `tools/session_coordinator/plans.py`: recursive `docs/plans` plus legacy `.codex/plans` scan, numbered-owner resolution and protected-file rules.
- Create `tools/session_coordinator/failures.py`: artifact ingestion, lifecycle graph, priority, cycle/duplicate detection and transactional fixed return.
- Create `tools/session_coordinator/cargo_jobs.py`: lane allocation, allowed target paths, PID/heartbeat tracking and cleanup eligibility.
- Create `tools/session_coordinator/cleanup.py`: Session archive, object GC, log rotation and safe target cleanup planning.
- Create `tools/session_coordinator/git_finalize.py`: explicit finalize request, path ownership audit, index mutex, validation and commit rollback.
- Create `tools/session_coordinator/workspace_copy.py`: stable validation copy manifest, command execution and verified deletion.
- Create `tools/session_coordinator/watch.py`: periodic file/Git/process observation and external-drift events.

### Windows and repository integration

- Create `tools/zircon-session.ps1`: stable client wrapper, hidden auto-start and pass-through JSON mode.
- Create `tools/install-session-coordinator-task.ps1`: install/update/remove/query the user-level scheduled task with dry-run.
- Modify `tools/cleanup-stale-targets.ps1`: replace fuzzy drive-root deletion with service cleanup planning/execution.
- Modify `.codex/skills/zircon-dev/scripts/validate-matrix.ps1`: allocate and release a service-managed Cargo lane for every target selection mode.
- Modify `.codex/skills/zircon-dev/scripts/validate-matrix.Tests.ps1`: assert service lane selection, allowlist rejection and release semantics.
- Modify `.codex/skills/zircon-dev/references/cargo-target-disk-policy.md`: document only the managed drive-root `targets` policy.
- Modify `.codex/skills/zircon-dev/references/main-branch-development-policy.md`: require service baseline/finalize discipline without branches or worktrees.
- Modify `.codex/skills/zircon-project-skills/cross-session-coordination/SKILL.md`: make service registration, claims, heartbeat and closeout the normal flow.
- Modify `.codex/skills/zircon-project-skills/cross-session-coordination/references/session-note-template.md`: retain a compact compatibility view with enum status and service IDs.
- Modify `.codex/skills/zircon-project-skills/cross-session-coordination/scripts/Get-RecentCoordinationContext.ps1`: query service first and recursively scan `docs/plans` when offline.
- Modify `.codex/skills/zircon-project-skills/handle-plan-failure-handoffs/SKILL.md`: connect artifact lifecycle to graph commands while keeping Markdown canonical.
- Modify `.codex/skills/zircon-project-skills/handle-plan-failure-handoffs/scripts/validate_plan_failure_handoffs.py`: expose reusable parsed artifact records for import/audit.
- Modify `.codex/skills/zircon-project-skills/write-plan-output-records/SKILL.md`: require coordinator owner routing and protect global plan files.
- Modify `.codex/skills/zircon-project-skills/milestone-first-workflow-policy.md`: route Session startup, writes, Cargo validation and finalization through the service.
- Modify `.codex/skills/zircon-project-skills/SKILL.md`: expose the coordinated workflow in the project skill index.
- Modify `.codex/skills/project-skills-index/catalog-existing-skills/current-project-skills.md`: refresh the shallow catalog after workflow changes.
- Modify `.gitignore`: explicitly document local coordinator state while preserving normal tracking of repository-local skills.

### Tests and documentation

- Create `tools/session_coordinator/tests/`: focused `unittest` modules and temporary-repository fixtures for every service domain.
- Create `tools/tests/session-coordinator-smoke.Tests.ps1`: client, scheduled-task dry-run, cleanup dry-run and offline fallback checks.
- Create `docs/cli-and-tooling/local-session-coordinator.md`: operator commands, state model, recovery and troubleshooting.
- Create `tests/acceptance/local-session-coordinator.md`: end-to-end acceptance evidence and destructive-test exclusions.

## Milestone M1: Service Kernel, Schema, and Enum Session Lifecycle

**Goal:** Start one authenticated local daemon, persist versioned state transactionally, and replace free-form Session status with a validated enum lifecycle.

**In-scope behaviors:** Repository discovery; `main` verification; local runtime descriptor; token authentication; SQLite WAL; schema migration; register/show/list/heartbeat/transition; invalid-transition rejection; single instance; structured JSON errors; hidden on-demand start.

**Dependencies:** Python 3 available; approved design; current main-branch policy.

**Implementation slices:**

- [ ] **M1.1 Define configuration and models:** add exact enums from the design (`registered`, `active`, `waiting_lease`, `resolving_failure`, `waiting_validation`, `finalizing`, `completed`, `stale`, `archived`, `cancelled`), typed IDs, TTL defaults, repo-root normalization, `.codex/state/session-coordinator` layout and drive-root target configuration.
- [ ] **M1.2 Build database and migrations:** create `schema_version`, `sessions`, `events`, `runtime_locks` and configuration tables; enable WAL, foreign keys and busy timeout; make every migration idempotent and covered by upgrade/rollback tests.
- [ ] **M1.3 Build daemon and client protocol:** implement single-instance startup, atomic `runtime.json`, random token, `127.0.0.1` binding, `/health`, authenticated command dispatch, graceful shutdown and stale-runtime recovery.
- [ ] **M1.4 Implement Session lifecycle:** register from `CODEX_THREAD_ID` or generated UUID, persist base metadata, enforce transition table, heartbeat timestamps, reason text and read-only archived records.
- [ ] **M1.5 Add PowerShell entrypoint:** implement `status`, `start`, `stop`, `session register/list/show/heartbeat/set-status`, hidden auto-start and `-Json` without duplicating service rules in PowerShell.
- [ ] **M1.6 Add focused unit tests:** cover first startup, second-instance rejection, bad token, schema re-open, every legal transition, illegal transitions, heartbeat update, stale runtime descriptor and non-main read-only diagnostics.

**Lightweight checks:**

- `python -m compileall -q tools/session_coordinator`; expect exit `0`.
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools/zircon-session.ps1 status -Json`; before service startup expect a structured `offline` response, not a stack trace.

### Testing stage M1-T: Kernel acceptance

Run:

```powershell
python -m unittest tools.session_coordinator.tests.test_database tools.session_coordinator.tests.test_server tools.session_coordinator.tests.test_sessions -v
powershell -NoProfile -ExecutionPolicy Bypass -File tools/tests/session-coordinator-smoke.Tests.ps1 -KernelOnly
git diff --check -- tools/session_coordinator tools/zircon-session.ps1
```

Expected: all unit tests pass; smoke test starts exactly one daemon in a temporary repo, authenticates, rejects an invalid transition, shuts down cleanly and leaves no live test process. Debug failures at the database/protocol layer before proceeding.

**Exit evidence:** passing named suites; `runtime.json` contains only local runtime metadata; no database/state files are Git-visible; status values outside the enum are impossible through the service API.

## Milestone M2: Baseline Epochs, Snapshots, File Leases, and Delayed Patches

**Goal:** Make shared-main writes attributable and recoverable, and prevent two Sessions from silently overwriting the same file.

**In-scope behaviors:** Initial hash manifest; baseline epochs; healthy/degraded/reconcile; SHA-256/zlib objects; snapshot manifests; normalized exclusive leases; atomic multi-path claims; heartbeat renewal; delayed FIFO patches; base-hash validation; `needs_rebase`; lease-violation events; preview-only restore.

**Dependencies:** M1 kernel and Session identity are green.

**Implementation slices:**

- [ ] **M2.1 Implement baseline epochs:** record HEAD, index tree and Git-visible file hashes; bind Session registration to an epoch; open a new epoch after HEAD changes; classify service-attributed and external changes.
- [ ] **M2.2 Implement object storage:** write deduplicated compressed objects atomically, store manifests in SQLite, verify object hash on read, and refuse restore outside the repository or without a matching lease/current hash.
- [ ] **M2.3 Implement path leases:** normalize case and separators, reject paths outside the repo, sort and acquire multi-file claims in one transaction, support re-entry/renewal, and reclaim only after TTL plus grace.
- [ ] **M2.4 Implement delayed patch queue:** accept a patch file and explicit target list, run `git apply --check`, save patch/base hashes, queue on conflict, apply after release only when hashes still match, otherwise emit `needs_rebase` with base/current/patch references.
- [ ] **M2.5 Implement watcher and reconciliation:** detect unregistered file changes and HEAD/index changes, snapshot evidence, mark baseline `degraded`, and provide `baseline diff`, `attribute`, `accept`, and lease-protected `restore-preview` commands.
- [ ] **M2.6 Add concurrency tests:** use separate client processes against one temp repo to prove atomic claims, FIFO order, lease expiry, no blind apply after foreign edits, object deduplication, degraded finalize guard stub and restart recovery.

**Lightweight checks:**

- `python -m compileall -q tools/session_coordinator` after each slice.
- `python -m unittest tools.session_coordinator.tests.test_paths tools.session_coordinator.tests.test_models -v` for pure path/model changes only.

### Testing stage M2-T: Write-conflict acceptance

Run:

```powershell
python -m unittest tools.session_coordinator.tests.test_baselines tools.session_coordinator.tests.test_snapshots tools.session_coordinator.tests.test_leases tools.session_coordinator.tests.test_patches tools.session_coordinator.tests.test_watch -v
python -m unittest tools.session_coordinator.tests.test_concurrent_writers -v
powershell -NoProfile -ExecutionPolicy Bypass -File tools/tests/session-coordinator-smoke.Tests.ps1 -LeaseAndPatch
```

Expected: two-writer fixture produces one lease owner and one queued patch; release applies the queued patch only for an unchanged base; a foreign edit produces `needs_rebase` and preserves both contents in objects; watcher drift changes the baseline to `degraded` without deleting files. Debug the lowest layer in order: path normalization, transaction/lease, hash/snapshot, patch application, watcher.

**Exit evidence:** deterministic concurrency test repeated 20 times without an overwrite; restart preserves queue and ownership; restore remains preview until lease and hash checks pass.

## Milestone M3: Plan Ownership Guards and Failure Graph Governance

**Goal:** Route every execution record to its numbered child plan, make global plan hotspots read-only, and lift failure/fixed artifacts into an auditable dependency graph without replacing Markdown truth.

**In-scope behaviors:** Recursive `docs/plans` scan; read-only `.codex/plans` compatibility scan; numbered plan resolution; protected global paths; failure/fixed import; duplicate/self/cycle/depth checks; priority; fixer-first Session startup; transactional fixed return with relative link/status summary.

**Dependencies:** M2 path safety, snapshots and transaction events are green; existing handoff validator remains authoritative for artifact schema.

**Implementation slices:**

- [ ] **M3.1 Extract reusable handoff parser:** refactor `validate_plan_failure_handoffs.py` so the CLI behavior and current 14 validator cases stay unchanged while coordinator import receives structured artifact records.
- [ ] **M3.2 Implement plan scanners and owner routing:** recursively scan `docs/plans`, scan `.codex/plans` as legacy, map plan definitions to `{id}/`, reject ambiguous ownership, and return the allowed output directory to Session registration.
- [ ] **M3.3 Enforce protected plan paths:** deny ordinary Session writes to any `index.md`, `engine-code-*.md` and numbered plan-definition Markdown; allow only the registered child directory; expose explicit maintenance mode as a separate authenticated command.
- [ ] **M3.4 Build Failure graph:** persist lifecycle nodes/edges from Markdown, detect duplicate lifecycle keys, self-edges, cycles, excessive depth, wrong placement and unresolved return states; order open failures for each fixing plan.
- [ ] **M3.5 Implement fixed return transaction:** after architectural acceptance and upward validation are recorded, move/rename the canonical artifact into the origin child directory, update origin references, write a concise fixer summary with relative link, update graph state, and roll back on any filesystem error.
- [ ] **M3.6 Integrate project skills:** update cross-session, failure-handoff, plan-output and milestone policies so startup queries open failures, source work continues, fixer work is prioritized, and all records use the coordinator-derived child owner.
- [ ] **M3.7 Add fixtures and real-tree read-only audit:** cover protected paths, ambiguous plans, all four current open failures, cycle/duplicate fixtures, successful return and injected rollback; audit the real `docs/plans` tree without modifying it.

**Lightweight checks:**

- `python .codex/skills/zircon-project-skills/handle-plan-failure-handoffs/scripts/test_validate_plan_failure_handoffs.py`; expect the existing suite to remain green after parser extraction.
- Coordinator plan/failure commands must support `--dry-run --json` and produce no Git diff during real-tree audit.

### Testing stage M3-T: Plan and graph acceptance

Run:

```powershell
python .codex/skills/zircon-project-skills/handle-plan-failure-handoffs/scripts/test_validate_plan_failure_handoffs.py
python .codex/skills/zircon-project-skills/handle-plan-failure-handoffs/scripts/validate_plan_failure_handoffs.py --repo-root E:/Git/ZirconEngine
python -m unittest tools.session_coordinator.tests.test_plans tools.session_coordinator.tests.test_failures -v
powershell -NoProfile -ExecutionPolicy Bypass -File tools/zircon-session.ps1 plan audit -RepoRoot E:/Git/ZirconEngine -Json
git diff --check -- tools/session_coordinator .codex/skills/zircon-project-skills
```

Expected: validator unit suite passes; real handoff validator reports no new coordinator-caused errors; protected-path fixtures are denied; graph fixture reports exact cycle members; successful fixed return leaves one canonical `fixed-*` in origin and one valid relative summary in fixer. Correct parser/schema issues before graph logic, then graph logic before skill text.

**Exit evidence:** recursive scan visibly includes `docs/plans`; global plan writes fail closed; current real artifacts import with stable graph IDs; return rollback leaves neither duplicate nor lost artifact.

## Milestone M4: Managed Cargo Lanes, Safe Cleanup, and User-Level Autostart

**Goal:** Put all supported Cargo validation behind service leases rooted only under drive-level `targets`, and replace unsafe fuzzy cleanup with lease/PID-aware maintenance.

**In-scope behaviors:** Target allowlist; drive selection; lane kinds; every `validate-matrix.ps1` target mode; PID/heartbeat; live process audit; disk threshold; cleanup plan/apply; no live-lane deletion; scheduled task install/update/remove/dry-run; legacy-task handoff.

**Dependencies:** M1 service lifecycle; M2 leases/path safety; M3 skill integration.

**Implementation slices:**

- [ ] **M4.1 Implement target allowlist and lane records:** accept only `D:\targets\zircon-engine`, `E:\targets\zircon-engine`, `F:\targets\zircon-engine` roots that exist/configure successfully; allocate unique `lanes/{lane-id}` paths and reject repo-local or arbitrary explicit targets.
- [ ] **M4.2 Implement Cargo job lifecycle:** add acquire/start/heartbeat/finish/release, lane kinds (`check`, `test`, `workspace`, `gpu`), PID command-line evidence, orphan detection and queueing for incompatible writers.
- [ ] **M4.3 Integrate `validate-matrix.ps1`:** replace repo-local JSON slots; normalize explicit `-TargetDir` and inherited `CARGO_TARGET_DIR` through the service; always pass the granted target; release in `finally`; update dry-run output and tests.
- [ ] **M4.4 Implement cleanup planning:** compute eligibility from lane state, process liveness, last activity, retention and free disk; require resolved allowlisted paths; separate read-only `cleanup plan` from explicit/scheduled `cleanup apply`.
- [ ] **M4.5 Replace stale-target script behavior:** make `tools/cleanup-stale-targets.ps1` call the service and never enumerate fuzzy root names or directly remove an unknown target; retain `-WhatIf` and useful offline diagnostics.
- [ ] **M4.6 Install user-level scheduled task:** add hidden at-logon daemon start plus 15-minute maintenance trigger, dry-run XML/command output, idempotent update and uninstall; only disable the old hourly task after new health checks succeed.
- [ ] **M4.7 Add fake-Cargo and cleanup tests:** simulate long-running jobs, abrupt exit, explicit/env target bypass, active-PID cleanup denial, stale lane cleanup, symlink/junction escape and unavailable drive.

**Lightweight checks:**

- `powershell -NoProfile -ExecutionPolicy Bypass -File .codex/skills/zircon-dev/scripts/validate-matrix.ps1 -RepoRoot E:/Git/ZirconEngine -SkipBuild -SkipTest -DryRun`; expect a managed allowlisted target path.
- Scheduled-task and cleanup checks use `-WhatIf`/dry-run only before M4-T.

### Testing stage M4-T: Cargo and maintenance acceptance

Run:

```powershell
python -m unittest tools.session_coordinator.tests.test_cargo_jobs tools.session_coordinator.tests.test_cleanup -v
powershell -NoProfile -ExecutionPolicy Bypass -Command '$result = Invoke-Pester .codex/skills/zircon-dev/scripts/validate-matrix.Tests.ps1 -PassThru; if ($result.FailedCount -gt 0) { exit 1 }'
powershell -NoProfile -ExecutionPolicy Bypass -File tools/tests/session-coordinator-smoke.Tests.ps1 -CargoAndCleanup
powershell -NoProfile -ExecutionPolicy Bypass -File tools/install-session-coordinator-task.ps1 -Action Install -DryRun
powershell -NoProfile -ExecutionPolicy Bypass -File tools/cleanup-stale-targets.ps1 -WhatIf
```

Expected: all target selection modes return an allowlisted lane; arbitrary and repo-local targets fail before Cargo starts; cleanup refuses active PID/lease and junction escape; stale fixture lane is removed only inside its test root; task install dry-run is hidden/user-level/idempotent. Debug allowlist/path checks before job lifecycle, then cleanup, then scheduled-task integration.

**Exit evidence:** no validator code path can choose `target/codex-shared-*`; no cleanup test deletes outside its generated test root; legacy fuzzy drive scan is absent.

## Milestone M5: Explicit Git Finalize and Stable Validation Copies

**Goal:** Convert only an explicitly approved, fully attributed Session result into a normal Git commit, and provide isolated stable build evidence without branches or worktrees.

**In-scope behaviors:** Finalize request; baseline/queue/lease guards; owned-path calculation; index mutex; staged-scope audit; semantic message; validation hooks; commit SHA record; index rollback; workflow-maintenance mutex; stable file copy; managed target; safe deletion.

**Dependencies:** M2 ownership/baseline; M3 plan validation; M4 Cargo lanes and path-safe cleanup.

**Implementation slices:**

- [ ] **M5.1 Implement finalize request model:** ensure `completed` alone never commits; require an explicit `finalize --commit` command tied to the current user request, record message/paths/validation profile and move Session through `finalizing` transactionally.
- [ ] **M5.2 Implement finalize guards:** reject degraded baseline, unattributed files, foreign leases, queued/`needs_rebase` patches, invalid plan output, open required Failure acceptance and active Git mutex.
- [ ] **M5.3 Implement scoped Git transaction:** snapshot index state, stage only service-owned paths, compare staged names to the approved set, run configured checks, create an ordinary semantic commit without Session tags, record SHA and open a new epoch; restore index on every failure without reverting worktree content.
- [ ] **M5.4 Support normal workflow-maintenance commits:** expose the same index mutex and staged-scope audit to repository skill/tooling maintenance, but do not store those changes as business Session intermediate commits.
- [ ] **M5.5 Implement validation copies:** materialize a manifest-selected source tree under `{target-root}\verify\{job-id}\source`, exclude `.git`, other Session changes and build output, run commands with adjacent managed target, record evidence and delete only after resolved-root validation.
- [ ] **M5.6 Add temporary-repository Git tests:** prove completed-without-finalize creates no commit, explicit finalize commits only owned files, foreign staged paths abort, hook failure restores index, no `[zircon-session:*]` appears, concurrent finalize serializes, and validation copy never contains `.git`.

**Lightweight checks:**

- All Git mutation tests run only in generated temporary repositories.
- Real-repo commands are limited to `finalize preview --json` and `validation-copy plan --json` until M5-T acceptance is complete.

### Testing stage M5-T: Git boundary acceptance

Run:

```powershell
python -m unittest tools.session_coordinator.tests.test_git_finalize tools.session_coordinator.tests.test_workspace_copy -v
python -m unittest tools.session_coordinator.tests.test_finalize_concurrency -v
powershell -NoProfile -ExecutionPolicy Bypass -File tools/tests/session-coordinator-smoke.Tests.ps1 -FinalizeInTempRepo
git log --all --format=%s --fixed-strings --grep="[zircon-session:"
```

Expected: temp-repo finalize tests pass; completion never commits implicitly; staged-scope violation produces no commit and restores the prior index; validation copies are deleted inside their job root; grep produces no new special-tag commits. Debug ownership calculation before index mutation, then validation hooks, commit, rollback and copy cleanup.

**Exit evidence:** one explicit test finalize yields exactly one normal commit containing only approved files; all negative cases yield zero commits and preserved worktree content.

## Milestone M6: Legacy Migration, Archival, Documentation, and Full Rollout

**Goal:** Adopt the service safely in the current repository, migrate uncontrolled Session state without data loss, and make the coordinated workflow the documented default.

**In-scope behaviors:** Existing Session note import; status mapping; active/stale/archive decision; `docs/plans` and legacy plan inventory; Failure graph import; old Cargo lease diagnosis; scheduled-task cutover; report; retention/GC; operator docs; recovery drill; real-repo non-destructive acceptance.

**Dependencies:** M1-M5 fully accepted.

**Implementation slices:**

- [ ] **M6.1 Implement idempotent legacy importer:** parse existing `.codex/sessions` notes, map known status strings to enums, preserve unknown text in `status_reason`, import plan links and timestamps, and never delete source files during import.
- [ ] **M6.2 Classify and archive Session roots:** keep notes with live process/recent heartbeat/active reference as active, mark expired notes stale, archive only after the 24-hour/no-reference rule, and emit a before/after manifest with hashes.
- [ ] **M6.3 Import plans, failures and Cargo diagnostics:** recursively index both plan roots, build the real Failure graph, report legacy `.codex/plans`, identify old repo-local Cargo leases/targets and leave them untouched until the cleanup plan is reviewed.
- [ ] **M6.4 Perform scheduled-task cutover:** install the new task, verify two consecutive health/maintenance ticks, then disable the old direct cleanup task; retain rollback instructions and do not delete the old task definition immediately.
- [ ] **M6.5 Complete workflow and operator docs:** document startup, Session commands, claim/patch/rebase, plan owner routing, Failure priority/return, Cargo lanes, explicit finalize, baseline recovery, archive restore and emergency offline mode.
- [ ] **M6.6 Run recovery and retention drills:** restart during a queued patch, restart during an intent event, simulate stale Session/archive, run object GC with live references, simulate active Cargo cleanup and restore an archived snapshot preview.
- [ ] **M6.7 Run real-repository rollout audit:** start the service against `E:/Git/ZirconEngine`, import in report-only mode, compare counts/hashes, resolve only deterministic status mappings, install tasks after dry-run review, and record acceptance without creating a business Git commit.

**Lightweight checks:**

- `python -m compileall -q tools/session_coordinator`.
- Every migration command supports `--dry-run --report <path>` and produces the same report on a second unchanged run.

### Testing stage M6-T: Full system acceptance

Run:

```powershell
python -m unittest discover -s tools/session_coordinator/tests -p "test_*.py" -v
powershell -NoProfile -ExecutionPolicy Bypass -File tools/tests/session-coordinator-smoke.Tests.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File .codex/skills/zircon-dev/scripts/validate-matrix.Tests.ps1
python .codex/skills/zircon-project-skills/handle-plan-failure-handoffs/scripts/test_validate_plan_failure_handoffs.py
python .codex/skills/zircon-project-skills/handle-plan-failure-handoffs/scripts/validate_plan_failure_handoffs.py --repo-root E:/Git/ZirconEngine
python .codex/skills/zircon-project-skills/write-plan-output-records/scripts/audit_plan_output_records.py --repo-root E:/Git/ZirconEngine
powershell -NoProfile -ExecutionPolicy Bypass -File tools/zircon-session.ps1 audit all -RepoRoot E:/Git/ZirconEngine -Json
git diff --check -- tools .codex/skills docs/cli-and-tooling tests/acceptance
```

Expected: all coordinator and integration suites pass; existing handoff validator remains green; plan-output audit introduces no new findings; real audit reports `main`, healthy or explicitly reconciled baseline, recursive `docs/plans` coverage, enum-only Sessions, allowlisted Cargo roots and no unsafe cleanup candidate. For any upper-layer failure, repair the lowest shared layer first and rerun upward.

**Exit evidence:** repeatable migration report; archived notes retain hashes and summaries; new scheduled task is healthy; old direct cleanup is disabled only after cutover; operator doc and acceptance record contain exact commands and recovery evidence.

## Milestone Promotion Rules

- A milestone advances only after its named testing stage passes and its numbered child-plan output record contains one row per completed slice plus the testing-stage result.
- Any baseline, lease, path-safety, rollback or cleanup failure is a lower-layer stop for higher milestones; do not compensate in CLI/skill text.
- Cross-plan failures use the existing `failure-*` lifecycle and do not stop unrelated slices. A fixing Session must close the lowest shared architecture cause before returning `fixed-*`.
- Implementation Sessions write evidence to their registered numbered child-plan directory. They do not update `docs/plans/**/index.md`, `engine-code-*.md` or numbered plan definitions.
- Do not run destructive migration, target cleanup, real Git finalize or scheduled-task cutover until the relevant earlier testing stages have passed.
- Do not create a Git commit merely because a business Session enters `completed`; require explicit user-requested finalize.

## 状态与产出记录

执行时逐切片填写；完成一个切片更新一行，不许批量补记。具体执行证据必须写到协调服务为该 Session 推导的编号子计划目录；本表只保留里程碑导航，不回写全局计划热点。

| 里程碑 | 切片 | 状态 | 完成日期 | 证据（命令输出 / 文件 / 测试名） |
|---|---|---|---|---|
| M1 | M1.1 Define configuration and models | 完成（待 M1-T） | 2026-07-11 | `tools/session_coordinator/config.py`、`models.py`：固定 10 个 Session 状态、迁移表、UTC/错误/记录 DTO、本地状态与受管 target 根配置。 |
| M1 | M1.2 Build database and migrations | 完成（待 M1-T） | 2026-07-11 | `database.py`、`migrations.py`：SQLite WAL、外键、busy timeout、显式事务、v1 内核表与 v2 基线/对象/租约/patch 表。 |
| M1 | M1.3 Build daemon and client protocol | 完成（待 M1-T） | 2026-07-11 | `server.py`、`client.py`：单实例锁、原子 runtime descriptor、随机 token、认证回环 HTTP、health/command、stale/offline 错误与非 main 只读模式。 |
| M1 | M1.4 Implement Session lifecycle | 完成（待 M1-T） | 2026-07-11 | `sessions.py`：注册/幂等刷新、HEAD/epoch 绑定、heartbeat、合法迁移、stale、事件日志；自由文本状态被拒绝。 |
| M1 | M1.5 Add PowerShell entrypoint | 完成（待 M1-T） | 2026-07-11 | `cli.py`、`__main__.py`、`tools/zircon-session.ps1`：status/start/stop、Session 子命令、JSON/退出码、隐藏按需启动。 |
| M1 | M1.6 Add focused unit tests | 完成（待 M1-T） | 2026-07-11 | `tests/test_database.py`、`test_sessions.py`、`test_server.py`；RED 证据：首次运行因缺少 `tools.session_coordinator.config` 按预期失败。 |
| M1 | M1-T Kernel acceptance | 通过 | 2026-07-11 | 8/8 Python tests 通过（启用 ResourceWarning=error）；`session-coordinator-smoke.Tests.ps1 -KernelOnly` PASS；scoped `git diff --check` exit 0。 |
| M2 | M2.1 Implement baseline epochs | 完成（待 M2-T） | 2026-07-11 | `baselines.py`：HEAD/index/manifest epoch、SHA-256、healthy/degraded、diff/scan、Session 归属、显式 accept 与 HEAD 变更刷新。 |
| M2 | M2.2 Implement object storage | 完成（待 M2-T） | 2026-07-11 | `snapshots.py`：SHA-256/zlib 内容寻址对象、原子写入、读时校验、snapshot manifest 与不落盘的 restore preview。 |
| M2 | M2.3 Implement path leases | 完成（待 M2-T） | 2026-07-11 | `leases.py`：仓库边界/保护路径、Windows casefold key、排序后的原子多文件 claim、重入/heartbeat/release、TTL+grace 回收。 |
| M2 | M2.4 Implement delayed patch queue | 完成（待 M2-T） | 2026-07-11 | `patches.py`：显式 target/base hash/object、FIFO queued/applying/applied/needs_rebase、`git apply --check`、前后快照、归属记录与无覆盖释放。 |
| M2 | M2.5 Implement watcher and reconciliation | 完成（待 M2-T） | 2026-07-11 | `watch.py` 及 server/CLI 命令：baseline init/status/diff/scan/attribute/accept、lease、snapshot preview、patch queue/status/process；外部编辑只降级并保留内容。 |
| M2 | M2.6 Add concurrency tests | 完成（待 M2-T） | 2026-07-11 | 新增 baseline/snapshot/lease/patch/watch 与 20 轮双线程 claim tests；RED 证据：首次运行因缺少 `tools.session_coordinator.baselines` 按预期失败。 |
| M2 | M2-T Write-conflict acceptance | 通过 | 2026-07-11 | `compileall` exit 0；完整 Python suite 21/21（含 20 轮竞争、后台 watcher、非 main 只读、单实例、即时 apply 竞态）通过；Kernel 与 LeaseAndPatch 两个 PowerShell smoke PASS；scoped diff check exit 0。 |
| M3 | M3.1 Extract reusable handoff parser | 完成（待 M3-T） | 2026-07-11 | validator 导出 `HandoffRecord`/`parse_handoff_records` 与稳定 lifecycle key；新增结构化导入测试，原 14 项加新用例共 15/15 通过。 |
| M3 | M3.2 Implement plan scanners and owner routing | 完成（待 M3-T） | 2026-07-11 | `plans.py`：递归正式 `docs/plans`、legacy `.codex/plans`、编号定义/子目录解析、显式 owner decision；3 项测试通过。 |
| M3 | M3.3 Enforce protected plan paths | 完成（待 M3-T） | 2026-07-11 | 普通 Session 仅可写注册编号子目录；`index.md`、`engine-code-*`、编号定义拒绝；maintenance 显式放行但仍受仓库 realpath 边界。 |
| M3 | M3.4 Build Failure graph | 完成（待 M3-T） | 2026-07-11 | schema v3 与 `failures.py`：Markdown 索引、稳定 lifecycle、open priority、schema diagnostics、重复/自边/环/深度治理；图测试通过。 |
| M3 | M3.5 Implement fixed return transaction | 完成（待 M3-T） | 2026-07-11 | 临时仓库验证 failure→fixed 移动、frontmatter/result 重写、双计划相对链接/状态摘要、validator 复验；第二次原子写入注入失败时完整回滚。 |
| M3 | M3.6 Integrate project skills | 完成（待 M3-T） | 2026-07-11 | 更新 cross-session、failure、plan-output、milestone policy、父索引/skill catalog 与递归 context reader；服务优先、Markdown canonical、离线兼容边界明确。 |
| M3 | M3.7 Add fixtures and real-tree read-only audit | 完成（待 M3-T） | 2026-07-11 | 临时 plan/failure fixture 覆盖 protected/legacy/cycle/depth/return/rollback；真实树只读导入 130 个正式计划、126 个 legacy 文档、6 个 handoff 节点与 28 项并发 schema diagnostic，未改业务 artifact。 |
| M3 | M3-T Plan and graph acceptance | 通过 | 2026-07-11 | `compileall` exit 0；handoff validator tests 15/15；coordinator suite 30/30；Kernel/LeaseAndPatch smoke PASS；skill quick validation PASS；scoped diff check exit 0。 |
| M4 | M4.1 Implement target allowlist and lane records | 完成（待 M4-T） | 2026-07-11 | schema v4-v7 与 `cargo_jobs.py`：Cargo jobs、cleanup reservations/persisted plans；仅 `lanes` 直接子目录，case/separator identity、overlap、junction/symlink/repo-local 统一拒绝。 |
| M4 | M4.2 Implement Cargo job lifecycle | 完成（待 M4-T） | 2026-07-11 | check/test/workspace/gpu lane，owner-checked leased/running/succeeded/failed/released/orphaned 状态，PID/真实进程命令/exit/heartbeat 审计，running 与 pre-start leased 失联回收，显式 lane 安全复用。 |
| M4 | M4.3 Integrate `validate-matrix.ps1` | 完成（待 M4-T） | 2026-07-11 | 删除 repo-local JSON 双槽；CLI/env/默认目标统一经服务校验；acquire 后立即进入外层 finally，pre-start/Cargo 缺失/正常/异常路径均释放。 |
| M4 | M4.4 Implement cleanup planning | 完成（待 M4-T） | 2026-07-11 | 持久化、单次、30 分钟有效 `plan_id` 锁定候选/retention；apply 只可缩减且拒绝 untracked；短事务 reservation、事务外 rmtree、重启恢复。 |
| M4 | M4.5 Replace stale-target script behavior | 完成（待 M4-T） | 2026-07-11 | `cleanup-stale-targets.ps1` 只调用服务，不模糊枚举盘符、不直接删除；默认 preview，`-Apply` 与 `-WhatIf` 边界保留。 |
| M4 | M4.6 Install user-level scheduled task | 完成（待 M4-T） | 2026-07-11 | 新增 repo-hash 隔离的 at-logon 隐藏 daemon 与 15 分钟 maintenance 两任务，Install/Update/Query/Remove 幂等入口和 DryRun 输出；验收阶段未实际安装。 |
| M4 | M4.7 Add fake-Cargo and cleanup tests | 完成（待 M4-T） | 2026-07-11 | Cargo/cleanup smoke 27/27：junction/symlink、unavailable root、nested overlap、case identity、显式复用、foreign owner、pre-start orphan、persisted plan、untracked拒绝、reservation/acquire 并发与事务外删除。 |
| M4 | M4-T Cargo and maintenance acceptance | 通过 | 2026-07-11 | Python full 57/57；Windows PowerShell 5 Pester 68/68；CargoAndCleanup smoke 27/27；installer DryRun/cleanup plan/compile/diff/secret scan exit 0；最终 reviewer 无 Critical/Important。 |
