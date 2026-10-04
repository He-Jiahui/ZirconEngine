---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: environment-ibl-parallel-staging
origin_plan: docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
fixing_plan: docs/plans/zircon_runtime/render/13-texture-pipeline.md
origin_child_dir: docs/plans/zircon_runtime/shader/06
fixing_child_dir: docs/plans/zircon_runtime/render/13
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/asset/importer/environment_ibl.rs
  - zircon_runtime/src/asset/artifact/ibl_source_cubemap_staging.rs
tests:
  - serial-versus-parallel source cubemap staging equivalence
  - cache-hit staging does not recompute PMREM source artifacts
---

# Render13：环境 IBL source cubemap staging 缺少受控并行执行器

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md`
- 来源执行者：`shader06-current-source-closeout-audit-20260716`
- 来源执行切片：current-source PBR viewer 首次/二次环境 IBL bake 响应评估
- 修复责任计划：`docs/plans/zircon_runtime/render/13-texture-pipeline.md`
- 交接原因：equirect → source cubemap/PMREM 的资产导入和 staging API 属于 Texture Pipeline TX-M3；Shader06 只消费已导入的资产，不能在 viewer 层复制或绕过导入调度。

## 失败现象与复现证据

- `zircon_runtime/src/asset/importer/environment_ibl.rs:108` 对 2:1 HDR 的 staging 走 serial `SourceCubemapMipChain::from_equirect_with_pmrem_layout(...)`（约 lines 156–162）。
- 框架已有经过测试的 `from_equirect_with_parallel_executor(...)`，但该 staging API 没有接收 `ParallelSliceExecutor`，导致 512-face PMREM bake 只被隐藏在单一后台线程，而不是利用受管 slice 并行。
- Shader06 的历史 current-source viewer 首次 Ready 约 127 秒；该数字是性能诊断，不是通过降低 bake 分辨率或跳过产物检查可以掩盖的门禁。

## 最低共享层根因

环境 IBL importer 的 staging boundary 没有表达 runtime-task-owned `ParallelSliceExecutor`，因此底层已有并行算法在该资产路径不可达；缓存语义和并行调度也没有由同一 API 共同约束。

## 架构修复验收

- Render13/TX-M3 为 environment IBL staging 提供受 runtime task 管理的 parallel executor 输入，并让 source cubemap/PMREM construction 使用它。
- 相同输入下 serial 与 parallel 产物必须字节等价；缓存命中不得重算 source cubemap 或 PMREM artifact。
- 添加聚焦 serial-vs-parallel 与 cache-hit/no-recompute 测试，并通过受管验证/审查/commit 返回该 handoff。
- Shader06 在 fixed return 后测量当前源码 first/second bake 响应；不得改 importer 路径、降低分辨率或声明未测的性能收益。

## 禁止临时方案

- 不得在 Shader06 viewer 中复制 cubemap/PMREM staging 或添加上层线程池旁路。
- 不得通过减少面分辨率、跳过 PMREM、接受近似字节结果或禁用缓存检查来缩短时间。
- 不得创建未受 coordinator 记录的 build target 或直接 Cargo 验证。

## 修复结果与回传

Open state: `实现已完成，待受管验证与独立审查`; Shader06 仅保留测量与上游验收责任。

### 当前实现状态（2026-07-17）

- `ProjectManager` 将运行时拥有的可选 `TaskPool` 传入 environment IBL staging；没有运行时任务所有者的直接工具路径保留串行基线。
- source cubemap 构建继续通过注入的 `ParallelSliceExecutor` 处理 equirectangular 基础投影和 source mip；equirect 与 captured-face 的并行入口都会将每个 PMREM mip 的六个独立 cube face 任务交给同一执行器，结果统一按固定 face-major 顺序回写。
- staging 在构建前检查完整的 `.zcube`/`.zribl` 当前缓存对。缓存命中会直接返回 `Reused`，不会调用 source mip 或 PMREM 的并行任务。
- 已有聚焦契约覆盖 serial/parallel 字节等价、PMREM 每 mip 调度和 cache-hit 零重算；尚未运行当前源码的受管 Cargo gate，故本 handoff 仍为 `open`，不得作为 Shader06 首次/二次 bake 性能结论。
- 独立静态复审已闭合为 `Critical 0 / Important 0 / Minor 0`：已复核 equirect sampler 的 `Fn + Send + Sync` 边界、PMREM 直接调度契约、captured-face 并行 PMREM 路径和缓存短路；该结论不替代待 FIFO 的受管 Cargo 验证。
- 查看器性能接线补充（2026-07-17）：parallel staging 新增 caller-decoded RGBA32F 入口，viewer 将同一份 HDR 像素用于曝光/尺寸和 equirect → source/PMREM staging，消除重复完整 decode；仍以原始 `AssetImportContext` bytes/settings 生成 request 与 cache key。现有 serial-versus-parallel staging contract 已改为通过该入口写 parallel bundle，仍需 fresh managed Cargo 证明当前源码。

### 2026-09-19 rolling validation retry and external blocker

- The existing primary Session `failure-roll-01a084c8-render13-ibl-staging-r2` was
  resumed after its stale heartbeat and reclaimed the exact four-path scope (the
  two implementation files, focused contract test, and this canonical record).
  The previous supervisor-level failures remain explicitly non-reusable:
  `f4e1e5abcda645669f5645b058480fe9` exited `1` after a health timeout with no
  test-level receipt, while the later exact-filter job
  `16476ed30ac342729a24621640ac99b6` also exited `1`; the only prior exact-filter
  green evidence is job `564bcb1aef4a44abab6883abec2d2c38` from baseline 608.
- A corrected direct structured Cargo request
  `failure-roll-01a084c8-render13-ibl-staging-20260919-r5` was admitted to policy
  evaluation with the exact command
  `cargo +1.94.1 test -p zircon_runtime --locked --test runtime_environment_ibl_source_import_staging_contract hdr_equirect_parallel_staging_matches_serial_bundle_and_reuses_cache -- --nocapture --test-threads=1`,
  but admission rejected it before ticket creation because external worktree
  `E:\Git\zr_vm` is dirty (`validation_ticket_external_worktree_dirty`). No Cargo
  process started and no dynamic acceptance is claimed. The lifecycle remains
  open and waits for the external owner to restore a clean revision before a
  non-duplicated managed retry.

### 2026-09-21 current-source successor and static evidence

- Stale-retention archived r2 without any validation ticket, so successor Session
  `failure-roll-01a084c8-render13-ibl-staging-r3` reclaimed and attributed only the two
  implementation files, the focused integration test, and this record. Preflight snapshot
  `3708` binds `environment_ibl.rs`
  (`d82d6d65dca2f3a0cbba52bc28dae491a95f0879781e87aacf61beb0a3618099`),
  `ibl_source_cubemap_staging.rs`
  (`5b04fd4101ae8ea2a252fc8ef17361ddbe8bac3617de863e54ff3707284a97a4`), and the contract test
  (`5b9cacef97a5f9bcea386dffe906c2ae0e7b161f2a7ad0a70561e4a39dea7c3e`). No production or test
  source changed in this successor.
- The source-bound static preflight is GREEN with marker
  `RENDER13_ENVIRONMENT_IBL_PARALLEL_STAGING_OWNED_STATIC_PASS`: it checks all parallel build
  entry points, the pre-build complete-bundle cache short circuit, serial/parallel byte equality,
  the positive executor call proof, the zero-call cache-hit proof, and pinned Rust 1.94.1
  `rustfmt --check`.
- The first managed static request (`...-20260921-static-r1`) was rejected before ticket creation
  with `validation_copy_overlay_not_owned`; the successor then attributed the exact current hashes.
  Corrected static-only ticket `befa8b6eccef47be88e6542316277240`
  (`...-20260921-static-r2`) passed with exit code `0` (job
  `f1a381c2d4824f17be5db711208ea909`, run `befa8b6eccef47be88e6542316277240`) and emitted the exact
  marker `RENDER13_ENVIRONMENT_IBL_PARALLEL_STAGING_OWNED_STATIC_PASS`. Its immutable manifest is
  `4b0d95730e507b12cd1d59d66995ad5c698dea65435d9fa41453dfa3e4fc82ef`. It remains explicitly
  `fullCoverage: false` / `staticParseOnly: true` and is not dynamic acceptance.
- The dynamic exact-filter Cargo gate remains blocked rather than resubmitted: the external
  `E:\Git\zr_vm` worktree still contains 157 foreign changes. Shader06 first/second bake
  measurement, fixed return, closeout, commit, and notification all remain pending.
- Independent current-source review completed `Critical 0 / Important 0 / Moderate 0`. It
  confirmed the runtime-owned executor reaches equirect projection, source mip, PMREM, and IEM;
  the complete bundle probe returns before all build work; the test compares actual `.zcube` and
  `.zribl` bytes from independent roots and proves a warm cache hit makes zero executor calls; and
  the caller-decoded path retains request/cache identity from original context bytes and settings.
  The reviewer made no edits and did not run Cargo.
