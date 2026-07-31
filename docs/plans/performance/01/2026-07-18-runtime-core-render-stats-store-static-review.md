---
related_code:
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store
  - zircon_runtime/src/runtime_diagnostics/collect.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
tests:
  - thirty leaf Rust files reviewed
  - two source-level RED to GREEN performance guards added
  - rustfmt and scoped diff checks passed
  - current-source Cargo, metric counters and product traces pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime render stats store逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`zircon_runtime/src/core/runtime/diagnostics/render_stats_store/**`当前Rust文件30/30（25个生产文件、5个test module/leaf，约6.3k行），并复核分发根`render_stats_store.rs`及调用方`runtime_diagnostics/collect.rs`。范围覆盖capability/history/graph/产品render families、advanced provider、后处理readback、virtual geometry/hybrid GI/particle/volumetric fog及product camera/visibility/HZB/light/mesh/GPU scene/sprite/UI。

## PERF-MVP-324：固定541条左右的全量series批次

静态展开确认一次`record_render_stats_diagnostics`约写入541条series（源码直接调用下限519；advanced provider/light family/dispatch循环展开后约541），其中capability 27条在adapter生命周期内通常不变，graph约84、mesh queue 56、virtual geometry 54、camera 48。编辑器只在Runtime Diagnostics或Performance Timeline可见时采集是正确的可达性门，但每次可见刷新仍先全量写series，再对整个DiagnosticStore和profile ring深snapshot；同一render frame多次刷新也没有generation cache。测试只检查少量值/标签，且共享`assert_series`每次断言都重新深snapshot，不能提供批次预算。

前一轮已让四类helper统一走`record_static`，本轮继续找到5个遗漏：effect-stack四条和light-grid average仍直接走通用owned metadata路径；调用根的render/physics/animation五条静态series也同样绕过快路。两组RED→GREEN守卫已把这10条全部切到static metadata复用，并用全叶扫描确认`render_stats_store/**`不再存在`store.record(`。值、unit、tag与history语义保持不变。

## 剩余根因与计划

`record_static`仍对每条series做BTreeMap O(log S)查询、metadata集合比较、summary更新和最多64项ring push；541条全批成本随可见刷新率而非数据变化量增长。直接用`submitted_frames`跳过整批并不安全，因为异步GPU readback/provider completion可能在同一submitted frame更新。Runtime07/Render17应在RenderStats内提供明确的snapshot generation与domain generations（capability、graph、readback、product families），DiagnosticStore预注册dense series token并接受packed batch；consumer按generation只追加变化domain，editor缓存同generation的owned snapshot，异步完成必须推进对应generation。

## 验收要求

对diagnostic panes hidden/visible、render 0/30/60/120 Hz、UI refresh 30/60/120 Hz、series 1/541/10k、history 1/64/1k记录record calls、BTreeMap probes/comparisons、metadata alloc、ring pushes、snapshot clone bytes、lock hold/wait、pane build p95和RSS：hidden写入/快照=0；同一完整generation整批写入≤1；capability每adapter generation≤1；静态metadata后续alloc=0；异步readback generation不丢更新；snapshot/pane只消费delta或缓存。current-source Cargo、F2/F4产品trace与RenderDoc/CPU对应帧证据通过前，本目录留在`pending.md`。
