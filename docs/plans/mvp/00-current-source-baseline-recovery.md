---
related_code:
  - tools/zircon-session.ps1
  - tools/session_coordinator
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1
  - zircon_runtime/src/asset/migration
  - zircon_runtime/src/asset/reference_resolver.rs
related_tests:
  - tools/tests/session-coordinator-smoke.Tests.ps1
  - tools/session_coordinator/tests
  - zircon_runtime/src/asset/tests/migration/project_commandlet/resolver_index.rs
  - zircon_runtime/src/asset/tests/migration/project_commandlet/source_boundary.rs
plan_sources:
  - docs/plans/mvp/index.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/04/failure-2026-07-23-asset-migration-indexed-resolver-generation.md
status: in_progress
gate: baseline
last_refined: 2026-08-01
---

# 当前源码与验证基线恢复 Implementation Plan

- fixed 已修复：[validation ticket deletion manifest](00/fixed-2026-08-05-validation-ticket-deletion-manifest.md)；删除墓碑协议、目录/悬空链接重现拒绝与最终复审均已收敛，source-bound managed batch `30/30` GREEN；Runtime15 可恢复硬删除 manifest 票据。

> **For agentic workers:** REQUIRED SUB-SKILL: 使用 `subagent-driven-development`（推荐）或 `executing-plans` 按里程碑执行；开始前使用 `cross-session-coordination`，测试阶段使用 `zircon-dev-validation` 和 `support-first-regression-testing`。

**Goal:** 恢复机器可读的受管验证入口，完成 Runtime 04 migration resolver index 的单一 generation 接线，并使 `zircon_runtime`、`zircon_editor`、`zircon_app` 当前源码重新通过包级编译检查。

**Architecture:** 先修验证控制面，再修最早的 Runtime 04 公共编译原因，最后从 Runtime 向 Editor/App 逐层收敛编译。禁止在上层 call site 增加临时 fallback，也不把声明 `mod resolver_index` 当成 migration 架构已完成。

**Tech Stack:** PowerShell、Python coordinator、Rust、Cargo、Windows managed target pool。

---

## 1. 当前证据

- 2026-07-24 的 `super::resolver_index` unresolved import 已过时：`migration/mod.rs` 当前声明并 re-export `resolver_index`，`MigrationResolver` 只持有 `AssetRegistryIndex` 与 `MigrationResolverIndex`，resolver 内无 root walk、`PathBuf`/`fs` probe 或 `persisted_source_path_for_locator` fallback。
- coordinator JSON stdout 污染已修复：`tools/zircon-session.ps1` 的两处 `Coordinator ready.` 都受 `if (-not $Json)` 门控；smoke test 覆盖 cold register、daemon reuse、JSON `ConvertFrom-Json` 与 human readiness。validator 已能提交并释放 managed jobs。
- 当前最低编译基线已变化：2026-08-01 managed `zircon_runtime --no-default-features --lib` focused job `c630da233fc440559b1788eafebddf9a` 在 lib-test harness 有 16 个 owner-bound 编译错误；default-feature job `367cc00c08394ae4b4705d2fe3c6f44f` 有 53 个。M0.3 因此仍未完成，但不再由 JSON 协议或 resolver module registration 阻断。
- 当前 checkout 包含大量用户和其他 Session 变化。本计划不清理、不还原、不重排这些变化，只通过 coordinator scope、source fingerprint 和逐层验证建立可复现基线。

## 2. 入口条件

- [ ] 当前 Session 已注册到本子计划，已查询最近四小时协调状态。
- [ ] 已查询 Runtime 04、Tooling 和受影响上层 plan 的 open failure。
- [ ] 已领取将修改的 tooling、migration source/test 文件 lease；未领取的用户变化保持原样。
- [ ] 已记录当前 commit、`Cargo.lock` 指纹、工作区路径级状态计数和 validator 脚本指纹，作为本里程碑 source-bound evidence。

## 3. 非目标

- 不关闭 Runtime 04 下与 MVP 无直接关系的 importer、watcher、virtual geometry 或大规模性能 failure。
- 不在本阶段构建或运行最终产品二进制；产品 build/start 属于 F0。
- 不强制当前共享 checkout 变为 clean，也不删除未跟踪文件。
- 不通过忽略 `ready` 后所有文本的宽松解析来掩盖 machine-readable 协议污染。

## 4. Owner 边界

| 范围 | 权威 owner | 本计划职责 |
|---|---|---|
| coordinator CLI/stdout 协议 | `tools/session_coordinator`、`tools/zircon-session.ps1` | 让 `-Json` 成为单一 JSON document；补回归测试 |
| validator 解析与 target policy | `.codex/skills/zircon-dev` | 严格消费机器输出并给出 actionable error |
| migration inventory/resolver | Runtime 04 | 完成 index 的生命周期接线和 focused contract |
| Editor/App 编译 follow-up | 对应最低 owner | 只修阻断当前编译的共享根因；跨 owner 走 failure handoff |

## 5. M0.1 受管验证输出协议

### 目标

保持 `zircon-session.ps1 ... -Json` stdout 为单一 JSON document，并用 current-source tooling tests/managed dry-run 防止 readiness 文本回流；该实现已落地，剩余是本基线的最新受管验收记录。

### 实现切片

- [x] `tools/tests/session-coordinator-smoke.Tests.ps1` 覆盖 cold register、daemon reuse，并直接 `ConvertFrom-Json` 解析 stdout。
- [x] 非 JSON 模式继续断言 `Coordinator ready.`，普通 CLI 反馈未被吞掉。
- [x] `tools/zircon-session.ps1` 仅在 `-not $Json` 时输出 readiness；JSON starting/response 各保持单一 document 语义。
- [x] `validate-matrix.ps1` 继续严格 `ConvertFrom-Json`；不得恢复宽松截断或忽略前导文本。
- [x] validation ticket `source_manifest` 以 JSON `null` 封存归属删除路径；queue claim、copy overlay 与 materialized-copy 的路径重现统一判为 `snapshot_stale`。
- [ ] 重跑 validator dry-run 回归，确认异常路径也释放 lane，不遗留 starting/active job，并把 exact count 记入本阶段 outcome。

### 测试阶段：M0.1 Tooling Contract Gate

- [ ] 运行涉及 `zircon-session` JSON 和 validator dry-run 的 Pester/Python focused suites。
- [ ] 从普通 PowerShell 运行 validator dry-run，确认 stdout 解析、compatibility 描述、批准 target root 和 release 生命周期。
- [ ] 在 daemon 已存在与冷启动各执行一次相同 dry-run。
- [ ] 失败时只修 control-plane 最低原因，重新运行 focused tooling batch；不进入 Runtime 编译。

### 退出证据

- [ ] 两种 daemon 状态下 machine-readable stdout 均为单一 JSON document。
- [ ] validator dry-run exit code 为 0，coordinator 中无遗留 active job/lease。
- [ ] 普通非 JSON CLI 仍有可读反馈。

## 6. M0.2 Runtime 04 migration resolver generation

### 目标

一次安全 inventory scan 发布 immutable `MigrationResolverIndex`，migration resolver 只做 pure lookup；逻辑 locator、root-relative path、physical identity 和 compound sidecar binding 在同一 generation 内一致。

### 实现切片

- [x] 以 [`../zircon_runtime/runtime/04/failure-2026-07-23-asset-migration-indexed-resolver-generation.md`](../zircon_runtime/runtime/04/failure-2026-07-23-asset-migration-indexed-resolver-generation.md) 为唯一 failure owner，重新盘点 `run.rs`、`scan.rs`、`sidecar.rs`、`resolver.rs` 和 `resolver_index.rs` 当前调用图。
- [x] `migration/mod.rs` 已注册并在 crate 内 re-export `resolver_index`；不存在孤立 `mod` 声明。
- [x] 让 scan owner 从已经 canonicalized、拒绝 symlink/reparse 的 regular-file inventory 构建 `MigrationSourceProjection`；禁止 index 自己再次访问文件系统。
- [x] 让 sidecar 解析发布经过验证的 compound binding；重复 registry identity、重复 locator、ambiguous root 和 missing source 返回现有 typed error/issue kind。
- [x] `MigrationResolver::new` 当前只接收 `AssetRegistryIndex` 与 `MigrationResolverIndex`；per-reference roots/FS fallback 已删除。
- [x] 扩充 `resolver_index.rs` 测试：唯一 source、compound zmeta hint、duplicate registry 优先级、ambiguous source、missing source、跨 root 相同相对路径、link/reparse 拒绝和 index generation 一致性。
- [x] 添加源码 guard，阻止 migration resolver 恢复 filesystem fallback 或第二次 root scan。

M0.2 current-source handoffs:

- fixed 已修复：[deferred-lighting-cache-test-hard-cut](00/fixed-2026-07-28-deferred-lighting-cache-test-hard-cut.md)
- Runtime 04: [`asset migration streaming transaction journal`](../zircon_runtime/runtime/04/failure-2026-07-22-asset-migration-streaming-transaction-journal.md).

M0.2 continues from the lowest passing test target while these owners complete their declared upward validation.

### 测试阶段：M0.2 Runtime 04 Resolver Gate

- [ ] 在 Windows coordinator lane 中先运行 `zircon_runtime` package check。
- [ ] 运行 migration resolver/index/source-boundary focused tests，并包含所有新增 typed failure 分支。
- [ ] 运行 Runtime 04 asset migration parent batch，确认 scan、sidecar、transaction 和 idempotent migration 未回归。
- [ ] 若上层失败，先回到 index/inventory generation；不得修改 Editor/App 适配失败的 resolver 语义。

### 退出证据

- [ ] `zircon_runtime` 当前源码编译通过。
- [x] migration resolver 不包含 filesystem lookup/fallback，所有查询来自单一 index generation。
- [ ] open failure 已完成 upward validation 并按 canonical failure/fixed 流程返回。

## 7. M0.3 Runtime → Editor → App 编译收敛

### 目标

在同一 source fingerprint 上逐层证明三个根包可检查，为 F0 产品构建提供稳定输入。

### 实现切片

- [x] 生成 validation manifest，按 `zircon_runtime`、`zircon_editor`、`zircon_app` 顺序列出 feature/profile、受影响 contract 和已知 open failure。
- [ ] 每次只处理当前最早编译错误；先判断是否属于本计划已修改边界、现有 owner 变化或新的跨计划 failure。
- [ ] 属于其他 owner 的最低原因写入该 owner 的 numbered failure 目录，并继续不依赖该失败的检查；不得在上层添加兼容 alias 或 feature bypass。
- [ ] 每次根因修复后重新运行最低包 focused check，再向上运行依赖包。
- [ ] 完成 `git diff --check`、触及 Rust 文件的 formatter check 和 source guards。

### M0.3 Validation Manifest

| 顺序 | 包 / profile | 受管 compile gate | 受影响 contract | 已知 open failure |
|---|---|---|---|---|
| 1 | `zircon_runtime` / default | `validate-matrix.ps1 -Package zircon_runtime -SkipTest` | Runtime04 single-inventory indexed resolver；Runtime15 receipt-test hard cut | [Runtime04 indexed resolver generation](../zircon_runtime/runtime/04/failure-2026-07-23-asset-migration-indexed-resolver-generation.md)；[Runtime15 plan-status receipt compile debt](../zircon_runtime/runtime/15/failure-2026-08-02-plan-status-receipt-test-compile-debt.md) |
| 2 | `zircon_editor` / default | `validate-matrix.ps1 -Package zircon_editor -SkipTest` | Runtime facade、RHI neutral presenter、Editor host compile boundary | [Frameworks01 RHI WGPU presenter/backend contract test owner](../zircon_runtime/frameworks/01/failure-2026-08-02-rhi-wgpu-presenter-and-backend-contract-test-owner.md) |
| 3 | `zircon_app` / default | `validate-matrix.ps1 -Package zircon_app -SkipTest` | Runtime/Editor entry composition 与 app startup compile boundary | fresh gate 前没有可复用的 current-source lowest failure；若出现诊断，按最低 owner 新建或刷新 canonical handoff |

三个 gate 必须由 coordinator 以同一 current-source manifest 顺序执行；任一 compile input 变化都会废弃该批次，不允许用旧 ticket、feature bypass 或上层 alias 继续。

### 测试阶段：M0.3 Current-Source Compile Gate

- [ ] 使用 `validate-matrix.ps1 -Package zircon_runtime -SkipTest` 完成 Runtime build/check batch。
- [ ] 使用 `validate-matrix.ps1 -Package zircon_editor -SkipTest` 完成 Editor build/check batch。
- [ ] 使用 `validate-matrix.ps1 -Package zircon_app -SkipTest` 完成 App build/check batch。
- [ ] 三个批次必须共享兼容的 Windows coordinator pool 和相同 source fingerprint；出现源码变化后重新生成 manifest。
- [ ] 测试结束确认无遗留 Cargo/rustc 进程和 active coordinator job。

### 退出证据

- [ ] Tooling、Runtime、Editor、App 三层均在 current source 上通过声明的 compile gate。
- [ ] 所有未解决问题已有明确 fixing owner 和 failure 链接，不存在“临时忽略后继续”的未知阻断。
- [ ] F0 可以从相同源码指纹开始产品 profile build。

## 8. 阶段退出清单

- [ ] M0.1、M0.2、M0.3 全部通过各自测试阶段。
- [ ] validator 的 JSON 协议有行为测试，而不只是手工确认。
- [ ] Runtime 04 resolver index open failure 完成架构接线和 upward validation。
- [ ] `zircon_runtime`、`zircon_editor`、`zircon_app` 当前源码包级编译均为绿色。
- [ ] 没有删除、回退或覆盖进入本 Session 前的用户变化。
- [ ] 只写一条本阶段 accepted outcome，不记录每次编译尝试。

## 状态与产出记录

每个里程碑测试通过后记录一次；实现切片不单独写入产出记录。

| 里程碑 | 范围 | 状态 | 完成日期 | 验证批次 / 残余风险 |
|---|---|---|---|---|

## Code Review 同步结论 (2026-07-30，2026-08-01 落实)

- JSON readiness 污染、resolver module registration 与 resolver FS fallback 三条旧 blocker 已同步到正文和 checklist，不再作为待实施工作。
- 2026-08-03 Runtime04 review repair 已将 linked current/retired sidecar 决策收回单一 inventory generation，并把 source-presence lookup 收敛为排序后的 O(logN) 查询；1/1k/100k references、1/4 roots 的双向顺序回归已落地，独立复审为 Critical/Important/Minor=`0/0/0`。
- 2026-08-03 M0.1 validation-ticket 删除墓碑协议已落地；focused Python batch 16/16 通过，首轮独立复审 `C0/I2/M1` 的 copy-race 分类、真实 baseline deletion 覆盖与 manifest 输入覆盖均已修复，最终独立复审 `C0/I0/M0`。manifest `d7e44074...` / ticket `00dfbdb27a9e4e85935a5f85a7c4f462` 已受理但无 terminal receipt。
- 计划状态保持 `in_progress`：M0.2 implementation 已完成，manifest `a587940b...` 的 package-check 与 focused tickets 已受理但尚无 terminal receipt；M0.3 的 Runtime→Editor→App current-source compile gate 尚未 GREEN。
- 下游不得继续引用 2026-07-24 的旧 blocker，但也不得仅凭静态修正解除 `blocked_by_00`；解除条件仍是 M0.1 focused、M0.2 resolver batch 与 M0.3 三包受管编译在同一 source fingerprint 下通过并写入 accepted outcome。
