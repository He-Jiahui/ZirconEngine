---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/**/*.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes/**/*.rs
  - tools/ui-profile-capture.ps1
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/bevy/crates/bevy_render/src/view/window/screenshot.rs
tests:
  - profiling artifact generation and route consistency tests
  - capture-disabled 1000-present counter trace pending
  - one-shot artifact worker and 1/100/10000-control scale tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor profiling artifacts/hit routes逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`profiling_artifacts.rs` + `profiling_artifacts/**`共 **35** 个Rust文件、**1,252** 行；`profiling_hit_routes.rs` + `profiling_hit_routes/**`共 **18** 个、**526** 行。合计 **53/53** 个文件、**1,778** 行已逐文件阅读，并核对直接采集脚本与Bevy screenshot实现。当前源Cargo、采集自扰动计数和规模trace未完成，因此仍留在`pending.md`。

## 已有正确边界

Geometry schema保留窗口大小、layout、splitter、tab、activity rail、viewport toolbar、template control和center/outside hit一致性样本，采集脚本据此驱动真实交互并检查route。类型化schema与产品route共享语义，避免脚本用硬编码坐标冒充交互。问题不在证据目标，而在采集触发频率、执行线程和重复投影算法。

## 热点与计划

- PERF-MVP-168：`export_present_artifacts`挂在每次成功present后。关闭采集仍每帧调用`std::env::var`；开启后每帧重复读取输出/session/screenshot/force配置、`create_dir_all`、重建完整`UiProfileGeometry`、pretty JSON序列化并同步覆盖同一文件。截图开启时还在present线程software paint完整frame、PNG encode并同步覆盖。交互场景持续数秒，因而WPR/CPU样本包含采集器自己的全树投影、编码和磁盘I/O。`is_forced_softbuffer_screenshot_run`在capture已启用分支内要求`!profile_capture_enabled()`，该条件恒false且再次读环境。修复必须是startup immutable配置、显式一次性或stable-generation request，以及有界worker异步编码/写盘；正常present不应触碰环境或artifact路径。Bevy以短生命周期`Screenshot`组件显式请求、异步完成事件和完成后despawn，提供了请求有界而非per-frame覆盖的最低参考。
- PERF-MVP-169：geometry先创建多个完整集合，再clone汇总成第二份`clickable_frames`。每个frame创建center/outside-left/outside-bottom三个样本并重复clone id/kind/surface；每个样本独立进入route。Activity/tab route线性扫rows并格式化expected id，template/toolbar route重复检查document、left/right/bottom/floating panes；surface-frame收集还逐pointer node调用hit-test证明top hit。采集规模会出现重复全树扫描与字符串分配。修复应携带稳定control/route identity，直接迭代已有集合并复用generation-owned hit/control index；每个样本最多一次实际hit-test，仍保留独立route parity判定。

## 动态验收

关闭采集运行1,000次present，记录profile env read、geometry build、software reference paint、JSON/PNG encode和file write，必须全部为0。发出一次capture request时geometry/json/png各恰好1次，UI/present线程encode/write为0，worker queue容量、drop/retry、完成generation和age可见。以1/100/10,000 controls运行geometry+hit consistency，记录visited nodes、hit tests、allocated/string bytes；总量必须近线性，clickable整表clone为0，每sample hit-test不超过1，并保持center/outside、popup z-order、clip、surface和typed route等价。
