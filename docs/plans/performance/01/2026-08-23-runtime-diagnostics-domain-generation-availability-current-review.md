---
related_code:
  - zircon_runtime/src/runtime_diagnostics
  - zircon_runtime/src/core/framework/render/framework.rs
  - zircon_runtime/src/graphics/runtime/render_framework/query_virtual_geometry_debug_snapshot
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Nanite/NaniteVisualize.cpp
tests:
  - current 4 of 4 facade Rust files and 6 tests reviewed
  - M0 static availability contract 3 of 3 passed
  - related WGPU Rust source behavior test added but not executed
  - focused rustfmt plus 1.94.1 and scoped diff check passed
  - current-source Cargo and F2/F4 traces blocked
doc_type: implementation-evidence
status: m0_static_complete_dynamic_blocked
---

# Runtime diagnostics domain generation/availability当前复审（2026-08-23）

## 范围与调用频率

`zircon_runtime/src/runtime_diagnostics/**`当前 **4/4** 个Rust文件、**389** 行、**13,504 B**、**6** 条测试已逐文件复读，manifest SHA256为`8489a9a89183769bc997c9d5d1dbbfea84563320d1fe01876342768706276f12`。四文件当前均干净，本轮不需要规避重叠源码改动。

端到端调用图已确认：Dev dynamic session默认每1 s调用`collect_runtime_diagnostic_current_store`，其他profile关闭周期日志；编辑器只在Runtime Diagnostics/Performance Timeline的publication target要求payload时采集；完整`profiling::snapshot()`只在显式诊断/devtools/profile-control路径生成。旧报告中“周期深clone history”已收敛为`DiagnosticStoreCurrentSnapshot`，不应继续当作当前根因。

## M0：boolean availability不得深clone debug payload

`collect_render_diagnostics` 只需要`virtual_geometry_debug_available: bool`，却调用owned `query_virtual_geometry_debug_snapshot()`后`is_some()`。WGPU实现在framework state锁内clone `Arc<RenderVirtualGeometryDebugSnapshot>`，锁外又`as_deref().cloned()`深拷贝完整payload。该payload当前含30多个`Vec`与多个嵌套snapshot；成本为`O(payload rows + bytes)`，却仅产生1 bit结果。

M0在`RenderFramework`添加`query_virtual_geometry_debug_snapshot_available()`默认方法，用旧owned query保留外部/测试backend兼容；WGPU override只在短state lock内查`Option<Arc<_>>::is_some()`，facade调用新contract。WGPU boolean query的payload Arc clone由1降为0、深clone由1降为0，复杂度由`O(payload)`降为`O(1)`；owned snapshot API和其调用方保留。

Unreal依据为`dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Nanite/NaniteVisualize.cpp:669-723`：先用`VisualizationData.IsActive()`和`GetActiveModeID()`的轻量状态判定是否生成/读取visualization，不为availability复制完整Nanite debug产物。Zircon只继承“先标量门控，详情按需”的数据流，不复制UE API。

## 结构剩余工作

- 每次domain collection仍重新resolve render/physics/animation manager，owned `query_stats()`仍深拷贝宽`RenderStats`，并将render stats展开为约541条series。这不是facade局部cache能正确解决的；PERF-MVP-324/418应由render owner发布generation-owned sealed diagnostics `Arc`，Runtime07以domain generation做if-newer/delta collection，同generation build≤1。
- `collect_runtime_diagnostics` 无法表达consumer只要render summary、store current或profile；最终API需要显式domain mask/subscription和summary/detail分层，hidden domains query/write/clone均为0。
- physics status和animation settings相对小，但仍应消费同一domain generation，不单独构造第二套缓存owner。

## 验收

M0先以source contract与focused Rust行为门确认facade不回退owned VG query、WGPU availability无`clone`/`.cloned()`。最终对hidden/render-only/physics-only/full、0/30/60/120 Hz、same/changed generation、VG payload 0/1k/100k/1M rows记录manager resolves、stats/VG queries、series writes、Arc/deep clone bytes、lock hold、alloc、p95/p99与RSS；hidden=0、same generation domain build≤1、availability payload clone bytes=0。current Cargo、F2/F4 WPR/Tracy与RenderDoc可见debug parity未通过前留在`pending.md`。

## 2026-08-23 M0实施证据

- `RenderFramework`新增兼容默认boolean query，WGPU提供短锁`Option<Arc<_>>::is_some()` override，`runtime_diagnostics::collect_render_diagnostics`已切换到boolean contract；owned snapshot API与旧backend保持可用。
- `tools/tests/test_runtime_diagnostics_availability_performance_contract.py`在实现前2 failures+1 error RED，实现后3/3 GREEN；42行、1,717 B，SHA256 `3ed82b9f1c587e243368a5afc127c19d628a4e547d43cb10084d325b6c668555`。相关WGPU源文件添加`virtual_geometry_availability_query_does_not_clone_the_payload`行为守卫，但current Cargo不可执行，该Rust测试 **未运行**。
- focused `rustfmt +1.94.1 --edition 2021 --check`与scoped `git diff --check`通过。facade实现后仍为4文件/389行/6 tests，13,506 B，manifest SHA256 `fa82886386258609994dae96f98d1e2fcce7bafd560efd256d076eb89fe8de14`。WGPU boolean path静态payload Arc clone `1 -> 0`、deep clone `1 -> 0`、复杂度`O(payload) -> O(1)`；未有wall-clock/allocator/功耗实测，不声称动态验收。
