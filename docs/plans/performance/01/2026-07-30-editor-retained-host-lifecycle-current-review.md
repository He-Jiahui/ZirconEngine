---
related_code:
  - zircon_editor/src/ui/retained_host/app/host_lifecycle
  - zircon_editor/src/ui/retained_host/app/native_windows/store.rs
  - zircon_editor/src/ui/retained_host/app/workbench_notifications.rs
  - zircon_editor/src/ui/retained_host/event_bridge.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_editor/editor/04-pie-and-simulation.md
  - docs/plans/zircon_editor/editor/12-plugin-management.md
  - docs/plans/zircon_editor/editor/15-build-export-and-publishing.md
tests:
  - inline tests: 7
  - rustfmt check: blocked by pre-existing current-source formatting drift in externally modified files
  - scoped whitespace check: passed
  - current-source managed Windows Cargo pending
  - WPR/Tracy F4 invalidation-storm trace pending
  - RenderDoc frame/presentation parity capture pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor retained-host lifecycle当前源码复核（2026-07-30）

## 范围

`zircon_editor/src/ui/retained_host/app/host_lifecycle/**`排除已单列验收的`startup.rs`与`startup/**`后，当前源 **32/32** 个Rust文件、**1,548** 行、**7** 条`#[test]`已逐文件阅读；path+raw-content SHA-256为`4e38ee86b056dd6b7ab76c28f083e4cf748482ba66d9b9a91b6effc0fd0ec4fc`。其中18个文件含外部未提交内容，本轮只读纳入current-source审查，未修改Rust。

| 模块 | 文件 | 行 | 测试 | 当前边界 |
|---|---:|---:|---:|---|
| dispatch/invalidation | 6/6 | 221 | 0 | event side effects、dirty mask与paint-only/slow-path判定 |
| native presenters | 6/6 | 240 | 0 | target收集、pane payload与per-window presentation apply |
| pane payloads | 3/3 | 220 | 1 | editor/runtime/plugin pane投影与可见pane裁剪 |
| recompute | 14/14 | 582 | 2 | chrome/model/geometry/template/floating/viewport/pointer重算 |
| render/shell/tick | 3/3 | 285 | 4 | frame tick、shell metrics与viewport submit |

## 发现

- **正向边界**：纯`PAINT_ONLY` invalidation已走fast path，不重建chrome/model/pane payload；相同status/progress会在置脏前比较；无新viewport图像时`Ok(false)`保留render dirty并请求后续帧，未伪装成成功提交。
- **PERF-MVP-103仍是current-source P0反馈环**：每个成功viewport submit的`Ok(true)`分支无条件调用`mark_presentation_dirty()`，唯一理由是pane payload已经收集后RenderStats才更新。源码测试还明确断言该调用存在。连续viewport渲染因此可形成render success -> presentation dirty -> 下一tick完整slow recompute，即使Runtime Diagnostics不可见。不能只删调用：必须先发布独立render-stats generation/counter，并让可见诊断或显式capture按需读取，保持post-submit diagnostics语义。
- **PERF-MVP-106仍是全域slow path**：任意非paint-only dirty会串行构造layout descriptors、完整chrome、command context/lock下的Workbench model、geometry、root/workbench template bridges、floating projection、viewport surfaces、全部pane payload、main presentation、native presenters和全部pointer surfaces。viewport resize在同一recompute中dispatch resize后又构造一次chrome/model；前面得到的geometry/template frame仍来自第一次构造。
- floating window projection在每个slow path遍历所有窗口并调用`sync_native_window_projection_bounds`，随后读取全部native hosts并重建bundle；没有按layout/window generation跳过unchanged bounds写入。
- 非空native target会在main presentation已经准备Module/Plugin、Build/Export和component-showcase payload后，再由`prepare_native_window_pane_payloads`重复准备这些payload。`NativeWindowPresenterStore::sync_targets`每次构造target id集合和stale id列表，并对每个target无条件apply完整presentation，没有per-window applied-generation fast path。该重复同时补强 **PERF-MVP-106/107**，不新建任务号。
- `collect_host_lifecycle_pane_payloads`每个slow path无条件读取preset names与全部plugin Template V2 pane snapshots；内建UI asset/animation payload已按可见kind裁剪，这是正向边界。plugin demand继续归 **PERF-MVP-595**，Build/Export与Module/Plugin shared generation归 **PERF-MVP-107**。
- `sync_pending_play_decisions`先调用`active_activity_window_template_document_is`，该gate当前会构造完整chrome；`tick()`调用一次，任意dispatch side-effect结束后又调用一次。因此交互帧可能在既有每tick完整pending projection之外再支付一次template gate与decision sync，补强 **PERF-MVP-105/596**。
- 本模块没有可在保持等价前提下直接提交的孤立小修：stats反馈、domain generation、main/native payload共享、per-window apply cursor和pending decision identity需要由EditorUI08统一持有，局部缓存会产生第二authority或stale window/pane状态。

## 参考与目标

- Godot `dev/godot/scene/main/canvas_item.cpp:143-180,540-551`用`pending_update`把重复`queue_redraw`合并为一个deferred callback，并直到draw完成才清标记以阻止递归更新。Zircon应保留自己的dirty-domain语义，但render统计发布不得反向制造结构dirty递归。
- Bevy `dev/bevy/crates/bevy_winit/src/winit_config.rs:90-107,121-133`只让配置的事件或deadline唤醒Reactive模式；`state.rs:705-732`把一次`redraw_requested`广播给窗口后立即清除。Zircon的Editor viewport可以连续render，但presentation rebuild、native presenter apply与diagnostic projection必须由各自generation需求驱动，而不是跟随每次submit。

EditorUI08建立`layout/chrome/model/pane/presentation/window/render-stats`依赖DAG：每frame capture一次dirty generations，按topological order每domain最多build一次；viewport resize先提交尺寸generation再统一build；main/native共享同一pane payload artifact，native store记录per-window applied generation；render stats与structural presentation分代。Editor04的pending decision projection只在generation改变且surface需要时应用，dispatch不再重复执行stable gate。Editor12/15仅发布immutable plugin/build/export generation，不在native presenter中建立第二catalog/cache。

## 动态验收

按windows/panes `0/1/4/16`、controls/nodes `1/1K/10K`、invalidations `idle/paint-only/presentation/layout/resize/render-success`、storm `1/1K/100K`、tick `30/60/120Hz`运行stable、single-change、同帧多domain、continuous viewport和diagnostics hidden/visible/capture矩阵。记录每domain build、chrome/model/template layout、pane payload、plugin callback、preset snapshot、main/native presentation、per-window apply、bounds write、render submit/stats、snapshot/clone bytes、UI thread wall、lock wait/hold、RSS和F4 p50/p95。

验收要求：idle与paint-only的structural build=0；正常render success的presentation dirty/rebuild=0；同帧每dirty domain build不超过1，resize的chrome/model build=1；main/native共享payload且每generation build不超过1；unchanged window apply/bounds write=0；hidden/stable plugin/build/export/diagnostic payload=0；stable decision gate/snapshot=0。managed Cargo、规模counter、independent review、WPR/Tracy与F4产品trace完成前保留在`pending.md`，RenderDoc只用于确认frame数量、viewport内容和最终present parity，不进入`review.md`。
