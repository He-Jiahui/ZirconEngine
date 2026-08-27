---
handoff_kind: fixed
status: fixed
failure_scope: local
created_at: 2026-08-27
summary_slug: integrated-bootstrap-artifact-lease
origin_plan: docs/plans/optimize/zircon_tooling/15-mvp-build-staging-product-process-acceptance-evidence-resource-baseline-control-plane-review.md
fixing_plan: docs/plans/optimize/zircon_tooling/15-mvp-build-staging-product-process-acceptance-evidence-resource-baseline-control-plane-review.md
origin_child_dir: docs/plans/optimize/zircon_tooling/15
fixing_child_dir: docs/plans/optimize/zircon_tooling/15
plan_link_mode: child_record_only
related_code:
  - .codex/sessions/tooling15-integrated-bootstrap.ps1
  - tools/mvp/MvpTestFixturePaths.psm1
  - tools/tests/tooling15-integrated-bootstrap.Tests.ps1
tests:
  - powershell -NoProfile -Command "Invoke-Pester -Script tools/tests/tooling15-integrated-bootstrap.Tests.ps1 -PassThru"
  - python -u -B -m unittest tools.session_coordinator.tests.test_artifact_governance.ArtifactGovernanceTests.test_fixture_release_requires_removal_and_does_not_exempt_recreation tools.session_coordinator.tests.test_artifact_governance.ArtifactGovernanceTests.test_require_clean_recovers_missing_artifact_reservations_online tools.session_coordinator.tests.test_artifact_governance.ArtifactGovernanceTests.test_require_clean_omits_recovered_reservations_from_rejection tools.session_coordinator.tests.test_artifact_governance.ArtifactGovernanceTests.test_require_clean_preserves_existing_artifact_reservation -v
resolved_at: 2026-08-27
---

# Tooling15 integrated bootstrap artifact lease

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_tooling/15-mvp-build-staging-product-process-acceptance-evidence-resource-baseline-control-plane-review.md`
- 来源执行切片：Tooling15 pinned PowerShell/Pester integrated validation bootstrap
- 修复责任计划：`docs/plans/optimize/zircon_tooling/15-mvp-build-staging-product-process-acceptance-evidence-resource-baseline-control-plane-review.md`
- 交接原因：同一计划拥有 bootstrap 与其验证环境；该缺口不属于被 artifact audit 阻塞的 UI12、Runtime 或 Coordinator product owners。

## 失败现象与复现证据

三个存活的 `tooling15-integrated-bootstrap.ps1` 进程 PID `13436`、`30860`、`33976`
分别直接创建 `D:\ZirconBuilds\tooling15-wave159-runtime-20260827-123718`、
`tooling15-wave160-runtime-20260827-125552`、`tooling15-wave161-runtime-20260827-131815`。
bootstrap 在创建这些目录前没有取得 coordinator artifact fixture lease。官方 artifact audit 因此
正确报告三者为 unmanaged；cleanup 对 wave159/160 反复得到 Windows error 32，因为创建者仍然
存活并持有其下载/模块树。所有 managed Cargo admission 随之 fail-closed。

## 最低共享层根因

仓库已有 `MvpTestFixturePaths.psm1`，它在物理创建前调用 `artifact fixture-acquire`，并要求物理
删除后再 `fixture-release`。tracked integrated bootstrap 绕过了该唯一生命周期 owner，自行拼接
`D:\ZirconBuilds` 路径，导致活跃临时环境在 durable governance 中没有身份。

## 架构修复验收

- bootstrap 必须在创建 runtime、PowerShell、Pester 或 evidence 路径前取得 coordinator-issued fixture。
- runtime tree 必须完全位于返回的 fixture path 内；不得保留自定义 `D:\ZirconBuilds\tooling15-wave*` 根。
- pinned runner 的真实 exit code 必须跨 cleanup 保留；所有成功、失败路径都必须删除 runtime tree，
  然后由同一 owner PID release fixture lease。
- focused Pester 必须固定 acquire-before-create、finally cleanup/release 与 exit-code preservation。
- 本修复只约束未来 bootstrap。不得终止当前三个 live producer，也不得手工删除其目录或 reservation。

## 禁止临时方案

- 不得按 `tooling15-wave*` 前缀永久豁免 artifact governance。
- 不得让 cleanup 删除活跃 producer 的目录，不得忽略 error 32，也不得绕过 managed Cargo preflight。
- 不得复制 fixture registry、直接写 Coordinator SQLite，或在 release 前遗留 runtime/evidence bytes。

## 修复结果与回传

- 根因：tracked bootstrap bypassed `MvpTestFixturePaths.psm1` and directly created a custom
  `D:\ZirconBuilds\tooling15-wave*` tree without a durable artifact fixture identity.
- 架构修复：the bootstrap now obtains its only runtime root from `New-MvpTestFixtureRoot`; every
  exit path reaches `finally`, where `Remove-MvpTestFixtureRoot` deletes the tree before releasing
  the coordinator lease. The pinned runner exit code is stored before cleanup, and a primary
  exception remains authoritative if cleanup also fails.
- 验证：valid production RED was `3 failed / 0 passed`; focused Windows PowerShell Pester 3.4
  GREEN is `3 passed / 0 failed`. The deterministic artifact-governance contract is `4/4` GREEN,
  including release/recreation and the four stale `mvp-test-fixtures-{11376,29760,10976,16996}`
  cleanup reservations. A prior real `tools/tests/mvp-test-fixture-paths.Tests.ps1` run exited `0`.
  Its post-rollover repeat observed a valid guardian race: after release/recreation, durable events
  recorded `artifact.unmanaged_delete_started` and `artifact.unmanaged_deleted` before audit returned,
  so the recreated path was governed rather than permanently exempt.
- 回传：source repair was sealed by coordinator maintenance commit
  `64942164497096a82cbb4a721405d9ffe367bccf`. The predecessor wave159-163
  producers exited naturally and their custom roots disappeared without manual deletion. Production
  waves 164-168 obtained active fixture leases; schema 68 successor
  `7d256d8279624daf963f8599d4d290b8` is healthy, the four stale reservation rows are absent,
  and both official audit plus the final independent scan report `unmanaged: []`. The separately
  observed full fixture probe race depends on uncommitted mixed `MvpTestFixturePaths.psm1` and
  `MvpArtifactStoragePolicy.psm1` bytes, so it was not absorbed into this source repair or its commit.
- Required-lane defense: `tooling15-integrated-bootstrap.Tests.ps1` is registered in both the Windows
  workflow and the pinned integrated runner. The combined local batch is `94/94` GREEN and the exact
  integrated contract count is `377+3`. Wave168 was submitted through the fixture-aware bootstrap;
  its process and logs remain intentionally unread pending the coordinator finalizer.
