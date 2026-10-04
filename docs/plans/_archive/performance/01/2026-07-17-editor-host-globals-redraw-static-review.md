---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/globals.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals
  - zircon_editor/src/ui/retained_host/host_contract/diagnostics.rs
  - zircon_editor/src/ui/retained_host/host_contract/diagnostics
  - zircon_editor/src/ui/retained_host/host_contract/diagnostics_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/redraw.rs
  - zircon_editor/src/ui/retained_host/host_contract/redraw
  - zircon_editor/src/ui/retained_host/host_contract/redraw_tests.rs
  - zircon_editor/src/ui/retained_host/ui/apply_presentation.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/slint/internal/core/partial_renderer.rs
tests:
  - globals pane interaction setter tests
  - host refresh diagnostics tests
  - redraw request merge tests
  - current-source Windows Cargo pending
  - multi-region damage product trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor host globals/diagnostics/redraw逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`globals.rs` + `globals/**`共 **16** 个Rust文件、**600** 行；`diagnostics.rs` + `diagnostics/**` + 相邻测试共 **9** 个、**186** 行；`redraw.rs` + `redraw/**` + 相邻测试共 **7** 个、**334** 行。合计 **32/32** 个文件、**1,120** 行已逐文件阅读。当前源Cargo、空投影计数和多damage产品trace未完成，因此仍留在`pending.md`。

## 已有正确边界

Callback调用先clone轻量`Rc<dyn Fn>`再释放`RefCell` borrow，允许callback重入；redraw请求会在native redraw前合并，并区分paint-only region和需要frame update的region/full。Performance scenario使用thread-local `Cell`，非profiling构建的counter调用为空。Refresh diagnostics只保存固定计数和两个`Instant`；其重复overlay格式化已归PERF-MVP-149。

## 热点与直接修复

- PERF-MVP-162：`apply_presentation`原先调用22个空`PaneSurfaceHostContext` setter，tail又把mesh path送入第23个空sink。14个调用仍会先遍历并转换recent project、project overview、activity/browser folders、items、selection、references和used-by模型，随后立即drop；八个字符串字段也被无效move。本轮已删除23个调用、14类转换函数、asset-data空setter模块与无效welcome/mesh方法，并增加source guard；`ShellPresentation`和最终host scene保持唯一真实投影。Rustfmt已过，当前源Cargo待验收。
- PERF-MVP-163：`HostRedrawRequest::Region`只持一个`FrameRect`；每次merge都调用`union_frame`生成外接矩形。两个分离小脏区会重绘/上传它们之间的全部未变像素。Slint `DirtyRegion`固定保存3个矩形，先做containment，容量满后选择面积增长最小的pair合并；Zircon应采用相同的bounded/no-heap原则并记录升Full原因。

## 其余观察

`HostContractState`是UI线程上的单一`Rc<RefCell>`；callback getter clone与小型drag/resize state clone没有静态证据表明是当前主瓶颈。`set_viewport_image`接收poll得到的owned image并转成host payload，像素resource owner由PERF-MVP-150负责。空setter之外不在本模块建立局部cache或并发状态权威。

## 动态验收

Presentation trace记录23类setter call、14类转换build/row visit/allocated bytes，修复后均为0。对1080p/4K两个或三个远距20x20 hover/cursor/diagnostics damage记录requested rects、retained rects、merge area growth、Full promotion、painted pixels和GPU upload bytes；面积应接近rect总和而不是bounding box。1k storm保持固定容量、无heap、单次merge有界，并保持frame-update/scenario、clip、z-order和像素等价。
