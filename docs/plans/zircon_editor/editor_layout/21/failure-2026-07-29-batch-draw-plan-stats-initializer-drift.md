---
handoff_kind: failure
status: open
created_at: 2026-07-29
summary_slug: batch-draw-plan-stats-initializer-drift
origin_plan: docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
origin_workflow_node: M2
fixing_plan: docs/plans/zircon_editor/editor_layout/21-gpu-submission-and-draw-pipeline.md
origin_child_dir: docs/plans/zircon_editor/editor/01
fixing_child_dir: docs/plans/zircon_editor/editor_layout/21
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/batching.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/batching/tests/scale_and_cache.rs
tests:
  - cargo +1.94.1 test -p zr_rhi_wgpu --lib --locked -- ui_surface --test-threads=1
  - cargo +1.94.1 test -p zircon_editor --lib --locked -- core::gateway:: --test-threads=1
---

# Layout21: BatchDrawPlanStats initializer drift

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md`
- 来源执行切片：M2 runtime-frame-demand hard cut 的 `gateway::` current-source compile gate
- 来源执行者：`editor01-runtime-frame-demand-hardcut-r1-20260729`
- 来源受管运行：job `640dc354cc38475daa1bd25e7217baf6` / run `bb226267623f4322839092a6f7365c15`
- 修复责任计划：`docs/plans/zircon_editor/editor_layout/21-gpu-submission-and-draw-pipeline.md`
- 交接原因：UI batch plan 的构造、缓存命中和统计口径属于 Layout21 的 GPU 提交边界；Editor01 只消费运行时编译结果，不能在 gateway 层补字段或改变统计语义。

## 失败现象与复现证据

2026-07-29 11:05 CST，source-manifest fingerprint `d3510f7e2cd5a4ade996a4887d77103aee3b1c710e29b2714427666e9503eed5` 的受管 focused gate 自然终态为 `exit 101`，尚未进入 `gateway::` 测试。Rust 编译 `zircon_runtime` 时报告：

```text
error[E0063]: missing fields `batch_plan_build_count` and `batch_plan_cache_hit_count`
in initializer of `BatchDrawPlanStats`
  --> zircon_runtime/src/rhi_wgpu/ui_surface/batching.rs:150:16
```

原始 stderr：`.codex/state/session-coordinator/cargo-runs/640dc354cc38475daa1bd25e7217baf6/bb226267623f4322839092a6f7365c15/stderr.log`。该运行还同时暴露 Render17 图执行导出问题，已另行路由，不应混入本工单。

## 最低共享层根因

`BatchDrawPlanStats` 已把 batch-plan build/cache-hit 指标纳入共享统计契约，但 `batch_draw_plan` 的直接构造初始化器没有同步迁移。`CompiledUiBatchPlanCache::resolve` 已在其结果上区分 build 与 cache hit；direct builder 的统计初始化和 resolve 层的指标合成缺少单一、完整的语义边界，导致 Rust 的结构体完整性检查在任何依赖 `zircon_runtime` 的上层门禁前失败。

## 架构修复验收

- Layout21 为 direct `BatchDrawPlan` 构造与 `CompiledUiBatchPlanCache::resolve` 定义一致的 build/cache-hit 统计归属，所有 `BatchDrawPlanStats` 初始化器完整表达该契约。
- 覆盖 cache miss、稳定 generation cache hit、damage/无 generation bypass 的计数测试，证明计数不被重复累计且不依赖调用方填补字段。
- `cargo test -p zircon_runtime --lib ui_surface --locked` 的受管 current-source 验证通过；随后来源命令能够编译越过本 E0063 后才可继续判定 Editor01 gateway 结果。
- 记录 immutable manifest、独立 review 和 fixed return；本 failure 在上述证据齐备前保持 `open`。

## 禁止临时方案

- 不得在 Editor01、Cargo 包装器或任一上层调用点以结构体默认值、条件编译或忽略统计的方式掩盖此错误。
- 不得删除 build/cache-hit 字段来恢复旧统计形状，也不得把 cache 指标伪装为 draw-call 指标。
- 不得将本次 `exit 101` 当作 gateway 失败或把 source-polluted 后续运行作为 Layout21 验收。

## 修复结果与回传

Open。该工单由 Layout21 接收后，需在 batch-plan owner 范围内修复、执行受管 current-source 验证并经独立审查，再以 `fixed-*` lifecycle record 回传来源计划。来源的 Editor01 网关切片保持冻结，等待外部编译链恢复后创建新的 source-bound reservation，绝不复用 job `640dc354cc38475daa1bd25e7217baf6`。

2026-07-29 12:50 CST，Layout21 已通过 failure-priority reservation
`6c56e3b131884a2788f0419544d9be78` 运行当前源 focused lib gate：job
`daf0b16f577d49d0aa8dc747d972f702` / run
`d8da72004daf439085e99efac3999c9a` 自然释放为 `exit 101`、live PIDs `[]`。
该运行未再报告 `BatchDrawPlanStats`、`batching.rs` 或 E0063，但不构成验收：完整
Runtime 输入指纹从启动前 `3126939cdea97bea6c293a2dc1e70247f22a6c159bc5b70f1aa1e79cd581779a`
变化为终态 `bcce51a05a74f42f56acc34753eb8680224b67946065954194c58a676487ca60`。
终态八个编译错误全部属于已经登记的下层责任：Render17
`scene-viewport-surface-projection-drift` 两项，以及 Runtime11 operation bounded-service
六项。原始日志保留在
`.codex/state/session-coordinator/cargo-runs/daf0b16f577d49d0aa8dc747d972f702/d8da72004daf439085e99efac3999c9a/`。
在两个下层 lifecycle fixed return 且 Runtime 输入重新冻结前，不创建或复用 Layout21
验收作业。

### 2026-08-13 current-source 前向修复状态

- `implemented_static / validation_pending`：`BatchDrawPlanStats` 的 direct builder
  初始化器已完整声明 `batch_plan_build_count=0`、`batch_plan_cache_hit_count=0`；构建/命中
  归属集中在 `CompiledUiBatchPlanCache::resolve`，cache miss 与无 generation bypass 返回
  `1/0`，稳定 generation 命中返回 `0/1`，调用方只投影该次 resolve 的统计，不重复累计。
- `scale_and_cache.rs` 当前覆盖稳定 generation 命中、target-only surface resize 保持 projection
  时复用、projection size 或 generation 变化时重建、versioned damage 复用完整 projection，以及
  unversioned damage 强制重建；direct
  builder 测试固定内部 build/hit 统计为零。全仓静态检索确认所有该字段构造点均完整。
- 本轮没有执行 Cargo，也没有新的 source-bound managed terminal evidence；因此旧 source-raced
  运行仍只作历史诊断，本 failure 保持 `status: open`，不声称来源 Editor01 gateway、向上验收
  或 `fixed-*` return 已通过。

## 产出记录与时间

| 时间 | 范围 | 状态 | 完成项与后续门禁 |
| --- | --- | --- | --- |
| 2026-07-29 11:05 CST | Layout21 batch-plan cache statistics | failure open | 已从 Editor01 受管 job `640dc354cc38475daa1bd25e7217baf6` 的终态日志提取 E0063，并确认根因位于 `BatchDrawPlanStats` 构造/缓存统计契约。等待 Layout21 在 owned scope 完成语义修复、受管验证、独立复审和 fixed return。 |
| 2026-07-29 12:50 CST | Layout21 current-source focused lib diagnostic | diagnostic RED / source-raced | failure-priority job `daf0b16f577d49d0aa8dc747d972f702` / run `d8da72004daf439085e99efac3999c9a` 自然释放 `exit 101`、无存活 PID；本 owner E0063 已消失，但 pre/post 全输入指纹不一致，八项终态错误均已路由至既有 Render17 surface 与 Runtime11 operation lifecycle。本运行不得验收、不得复用。 |
| 2026-08-13 | Layout21 current-source statistics audit | `open / implemented_static / validation_pending` | direct builder 初始化器完整声明 build/hit 为 0；cache resolve 独占每次 miss/hit/bypass 计数，focused source tests 覆盖稳定 generation、尺寸/代际变化和 damage 路径。仅有静态源码证据，无 managed terminal evidence，不声称 fixed return。 |

## 2026-09-11 rolling repair admission

- Stable fixing Session `failure-roll-01a084c8-editorlayout21-batch-draw-plan-stats` owns this
  record and the two batch-plan source/test files. Snapshot `3405` freezes the exact current
  bytes at HEAD `c37155ba304740b3762b20585f77fb53a6da47fb`.
- Request `batch-draw-plan-stats-20260911-r1` submitted the focused `cargo test -p
  zircon_runtime --lib ui_surface --locked` command without a caller-owned `--jobs` override.
- Coordinator admission rejected the request with `validation_ticket_external_worktree_dirty` for
  external repository `E:\Git\zr_vm`, before a validation ticket or Cargo process was created. No
  dynamic `ui_surface` result, Editor01 gateway result, performance result, or fixed return is
  claimed; the external worktree was not modified and this failure remains open.

## 2026-09-19 rolling successor formal source binding

- Successor Session `failure-roll-01a084c8-editorlayout21-batch-draw-plan-stats-r2` reclaimed the
  archived exact-path ownership through coordinator transfer fingerprint
  `81d8ed223d0a253ed4be1856c576f976485fec5dda5456a1e056329f20b86058` at baseline epoch
  `611`; no source bytes were changed during attribution.
- Formal non-Cargo source-contract ticket `71199447949a4394a563194190b04b93` was admitted
  from request `failure-roll-01a084c8-editorlayout21-batch-draw-plan-stats-20260919-r1` and is
  currently `queued`. Its sealed source-manifest hash is
  `17a31a3ab11dda21efaac7bd531bb10ed315a07fbbea9942227f14651af8744c`:

  | path | SHA-256 |
  | --- | --- |
  | `docs/plans/zircon_editor/editor_layout/21/failure-2026-07-29-batch-draw-plan-stats-initializer-drift.md` | `cf076276195d6480e35de50e8dc5a8c33af08fddfeb6f3a83333ee9d81b7f598` |
  | `zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/batching.rs` | `8fae2b2e1fb7ed782fa7c91725b647ca43cb267f7b7b20b8351b59e49619ab2a` |
  | `zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/batching/tests/scale_and_cache.rs` | `5f907728352d4f18e21c2e85cc02d4c19ceb01685f35b259c278c241ec0c9912` |

- The ticket executes a Windows PowerShell/rustfmt source-contract parse covering complete
  build/cache-hit fields, cache ownership and miss/hit/generation/damage regression anchors.
  It explicitly defers the focused `ui_surface` Cargo gate, Editor01 gateway command,
  independent C/I/M review, canonical fixed return and closeout. The prior source-raced run
  and external dirty-worktree admission are retained as diagnostics only.
- Failure remains `open`; no fixed return, commit, or notification is claimed.

### Formal source-contract ticket terminal result

- Ticket `71199447949a4394a563194190b04b93` completed `passed` at
  `2026-09-19T04:55:03.234763Z` (exit code 0) with immutable output
  `LAYOUT21_BATCH_DRAW_PLAN_STATS_SOURCE_CONTRACT_PARSE_PASS`. Sealed manifest hash:
  `17a31a3ab11dda21efaac7bd531bb10ed315a07fbbea9942227f14651af8744c`.
- This static result does not replace the focused `ui_surface` Cargo gate, Editor01 gateway
  acceptance, independent C/I/M review, fixed return or closeout; the prior external dirty
  worktree blocker remains recorded.

## 2026-09-21 rolling review repair

- Successor Session `failure-roll-01a084c8-editorlayout21-batch-draw-plan-stats-r3` froze
  pre-review snapshot `3702`. The production owner retained SHA-256
  `8fae2b2e1fb7ed782fa7c91725b647ca43cb267f7b7b20b8351b59e49619ab2a`; the prior static
  ticket therefore still described the production implementation, but not the review repair.
- Independent review initially reported `Critical 0 / Important 1 / Moderate 0`: this record
  mislabeled a target-only surface resize as a projection-size change and claimed a
  generation-change rebuild regression that did not exist. A source-bound static RED exited `1`
  because both exact regression test names were absent.
- `scale_and_cache.rs` now separately proves that changing only the explicit generation and
  changing only the projection size each rebuild the cached plan, report per-call counters
  `1/0`, and replace the plan `Arc`. The existing target-only resize test continues to prove
  reuse when the projection coordinate space is preserved. The exact static guard is GREEN,
  pinned Rust 1.94.1 `rustfmt --check` is GREEN, and the scoped diff whitespace gate is GREEN.
- Final independent review of the repaired source and record is
  `Critical 0 / Important 0 / Moderate 0`. Focused managed `ui_surface` Cargo and the Editor01
  upward gateway gate remain required; no dynamic pass, fixed return, closeout, commit, or
  notification is claimed here.
- The first corrected-source validation request (`...-20260921-r2`) was rejected before ticket
  creation with `validation_copy_overlay_not_owned` because five direct consumers are owned by
  other active work. Their ownership was not transferred and the rejected request is not
  acceptance evidence.
- Static-only ticket `021b9c52507c4217a9d5abf53e2b4de0` (`...-20260921-r3`) was admitted against manifest
  `20cb996bbb51bc52d0ea77eee2d520231babd555deedf29da7ba43c12d5ad7e3`. Its immutable overlay is
  limited to `batching.rs`
  (`8fae2b2e1fb7ed782fa7c91725b647ca43cb267f7b7b20b8351b59e49619ab2a`) and
  `scale_and_cache.rs`
  (`4b3fc73153e70ccb5031aca8c32f036cfb43a6abdadddf4d92d65aa342dda543`). It failed before contract
  evaluation because its command tried to read undeclared read-only consumer paths that are not
  materialized in the two-file validation copy (`FileNotFoundError`, exit code `1`, job
  `0c3e7ab21bff471e82173ac00805f592`). This is retained as a command/manifest mismatch, not a
  product failure or reusable acceptance result.
- Corrected successor ticket `d635bb0694ba446fa57d70bd0938e4cd` (`...-20260921-r4`) is queued with
  the same immutable manifest and checks only the two owned overlay files, with expected marker
  `LAYOUT21_BATCH_DRAW_PLAN_STATS_REVIEW_REPAIR_OWNED_STATIC_PASS`. The five direct consumers remain
  separately checked read-only in the live checkout and are explicitly deferred from the ticket.
  The successor remains `fullCoverage: false` / `staticParseOnly: true`; queued is not passed and
  cannot replace the focused managed Cargo or upward Editor01 acceptance gates.

## 2026-09-27 current acceptance commands and terminal receipt reconciliation

- Successor Session `failure-roll-01a0df1a-layout21-batch-stats-r1` formally received only
  this record and its two source/test paths from the archived r3 owner at baseline epoch
  `628`, HEAD `bc02eefafead65dbf5050482110e8175250a5e77`, through transfer fingerprint
  `05e3461089072e65106d9b3a10d973728cb4f1bb79d62ef1955a5f4f8a4303ad`.
  Production and regression source bytes are unchanged. Existing validation tickets keep
  their original Session ownership.
- The active `tests` field now targets `zr_rhi_wgpu`, the current package declaring
  `ui_surface::batching::tests::scale_and_cache`. Testing `zircon_runtime` alone does not
  execute a dependency package's unit tests. The original Runtime command above is retained
  as historical reproduction evidence; the required owner regression is the current
  `zr_rhi_wgpu --lib ui_surface` command. Editor01's consumer gate now uses the current
  `core::gateway::` module path. Both commands remain Windows managed, pinned to Rust
  `1.94.1`, `--locked`, and without a caller-owned jobs or target directory override.
  Their actual test output must prove nonzero target execution before acceptance.
- The formerly queued corrected static ticket `d635bb0694ba446fa57d70bd0938e4cd`
  reached `passed` at `2026-09-21T13:19:10.018127+00:00`, exit `0`, job
  `28231430320b40beb559bd80d03e711b`, with marker
  `LAYOUT21_BATCH_DRAW_PLAN_STATS_REVIEW_REPAIR_OWNED_STATIC_PASS`. Its sealed manifest
  `20cb996bbb51bc52d0ea77eee2d520231babd555deedf29da7ba43c12d5ad7e3` matches the
  current two source hashes below. The earlier queued description records the submission
  time, not the terminal result. The durable reconciliation is
  `.codex/tmp/failure-roll-01a0df1a-layout21-existing-ticket-reconcile.json`.

  | Source path | SHA-256 |
  | --- | --- |
  | `zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/batching.rs` | `8fae2b2e1fb7ed782fa7c91725b647ca43cb267f7b7b20b8351b59e49619ab2a` |
  | `zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/batching/tests/scale_and_cache.rs` | `4b3fc73153e70ccb5031aca8c32f036cfb43a6abdadddf4d92d65aa342dda543` |

- Reuse is limited to the identical two-file static parse and rustfmt result. It proves no
  Rust compilation, dynamic test, direct consumer acceptance, product result, canonical
  fixed return or closeout. The corrected record is outside that old ticket's manifest.
  The five consumer paths previously rejected as foreign overlays remain with their
  actual owners; normal inventory/archive must bind the complete compile input.
- Managed Cargo remains pending on external `zr_vm` capture readiness and the existing
  network decision. No rejected validation request is repeated by this correction.
  All eventual compilation products and compiler caches must physically remain beneath
  drive-root `D:\cargo-targets`, `E:\cargo-targets` or `F:\cargo-targets`.
  Failure stays `open`; no dynamic pass, fixed return, commit, or notification is claimed.
