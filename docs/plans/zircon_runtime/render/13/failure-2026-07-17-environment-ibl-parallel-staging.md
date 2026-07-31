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
