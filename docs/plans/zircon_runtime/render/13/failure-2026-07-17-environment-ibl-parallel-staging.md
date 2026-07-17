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

Open state: `待 Render13/TX-M3 修复`; Shader06 仅保留测量与上游验收责任。
