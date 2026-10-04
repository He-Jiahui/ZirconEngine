---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/routing.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/routing
  - zircon_editor/src/ui/retained_host/primitives.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/bevy/crates/bevy_ui/src/layout/ui_surface.rs
  - dev/slint/internal/core/input.rs
  - dev/slint/internal/core/string.rs
tests:
  - native chrome/pane/asset/toolbar route tests
  - current-source Windows Cargo pending
  - 1/100/10000 route clone/allocation/visited-node scale tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor native pointer routing逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`native_pointer/routing.rs` + `routing/**`共 **48/48** 个Rust文件、**1,166** 行已逐文件阅读，并回查`ModelRc`与`SharedString`基础类型。当前源Cargo、route规模trace和字符串所有权迁移未完成，因此仍留在`pending.md`。

## 已有正确边界

Routing保持resize→document tabs→rails→drawer tabs→host page→floating header的root chrome顺序；floating pane按reverse row保留topmost z，local pane按document/left/right/bottom顺序；tab close优先body，viewport toolbar优先body，popup/route的最终行为由上层dispatch决定。坐标转换、half-open hit containment和viewport toolbar runtime surface hit保持单一函数。

## 热点与计划

- PERF-MVP-173：floating chrome/pane、activity rail、document/drawer/page tabs和asset content panel都用`row_data`遍历。`ModelRc::row_data`会clone整行，因此每个未命中candidate都深拷贝DTO；floating window和asset panel含完整pane/node数据。Asset route为1-2个已知control id重复扫描全部nodes并返回owned约160字段node。局部应全部改用`iter/get`借用，panel finder返回引用。结构上由EditorUI01 generation owner发布chrome/pane spatial/control index，统一维护floating z、tab/rail/panel/toolbar identity；event route使用typed enum/borrowed stable id，避免为`document`、`left`、`activity`等静态值分配String。
- PERF-MVP-174：`primitives.rs`把`SharedString`定义为普通`String`。Editor共有1,192个`SharedString`引用点，其中retained host 816个；所以presentation、wide node、row、route和interaction的每次clone都按字节深拷贝。这个事实解释了为何局部`row_data`、hover、keyboard和snapshot clone远重于类型名暗示。EditorUI08必须把immutable id/label/path/action硬切到真正COW或`Arc<str>`共享类型，editable text/format builder明确保留`String`，并测试mutation ownership与thread/lifetime。Slint的`SharedString`基于`SharedVector<u8>`、clone只复制引用、mutation时copy-on-write，是直接参考。

不能用全局永久intern表隐藏内存增长，也不能把所有文本都改为immutable从而让编辑输入每键重新分配。PERF-MVP-167的局部text edit仍允许一次owned buffer copy；PERF-MVP-174针对长期DTO/route共享值。

## 动态验收

对1/100/10,000 floating windows、tabs、rail buttons和asset nodes运行move/scroll/click route，记录candidate visited、DTO clone、string copied bytes、heap allocations和p95。局部borrow修复后candidate DTO clone必须为0；最终index后visited与无关tree size解耦，steady route静态字符串allocation为0。10k-node presentation/route snapshot的shared text clone copied bytes/heap alloc为0且O(1)引用复制；serde/hash/order/borrow/path、Unicode、duplicate control、floating z和全部route/pixel行为等价。
