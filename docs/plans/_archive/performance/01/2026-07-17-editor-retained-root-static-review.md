---
related_code:
  - zircon_editor/src/ui/retained_host
  - zircon_editor/src/ui/retained_host/app/workspace_docking/drawer_resize/movement.rs
  - zircon_editor/src/ui/host/layout_commands.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08/failure-2026-07-17-layout-metadata-full-recompute-and-sync-io.md
reference_sources:
  - dev/bevy/crates/bevy_ui/src/layout/ui_surface.rs
  - dev/slint/internal/core/model/repeater.rs
tests:
  - floating_window_projection_indexes_native_hosts_once
  - floating_window_projection_preserves_first_native_host_for_duplicate_window_ids
  - model_rc_takes_unique_vec_model_without_cloning_rows
  - model_rc_clones_rows_when_the_source_vec_model_is_shared
  - existing floating-window/drawer/notification/popup/profiling tests
  - current-source Windows zircon_editor focused Cargo pending
  - 1/100/1000 floating windows and 10k-row clone-count trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor retained host root 逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`zircon_editor/src/ui/retained_host` 根目录当前共 **19** 个 Rust 文件，已逐文件阅读 **19/19**。其中 `app.rs`、`scroll_surface_host.rs` 与 preview-action root的结论也被更细模块证据引用；本记录补齐 root wiring、primitive、event effects、drawer resize、floating projection、popup metrics、notifications、run config与 profiling counters 的整体处置。

当前源码 Cargo、floating-window scale、drawer-resize transaction计数和 large-pane clone-byte trace仍未完成，因此继续保留在 `pending.md`。

## 已直接优化

- PERF-MVP-130：projection 旧实现对每个 floating window先找 native host，再调用 helper重新扫描同一 host slice，并用 `BTreeMap`保存只按 id查询的 frames。现一次建立 borrowed host-id index，按窗口均摊 O(1) lookup；frames改为 `HashMap`。使用 `entry.or_insert`保持 duplicate host id 的 first-match语义。
- PERF-MVP-132：`ModelRc::from(Rc<VecModel<T>>)` 旧实现无条件 clone完整 `Vec<T>`。现以 `Rc::try_unwrap`接管唯一 source；只有 source确实共享时 clone。生产调用普遍在转换点新建 Rc，因此 steady host projection不再复制整批 rows。

源码 RED→GREEN、`rustfmt` 与 `git diff --check` 已过，动态测试尚在协调器 FIFO 中。

## 已移交热点

- PERF-MVP-131：left/right drawer resize group分别 dispatch top/bottom extent command。`EditorUiHost::apply_layout_command` 对每个 command都执行 session lock、layout apply、legacy drawer sync与完整 `recompute_session_metadata`；一次交互成为两次全量事务，并存在半提交失败边界。已补充 EditorUI08 layout failure，要求 atomic typed batch/delta。
- `Image::to_rgba8` 与 viewport frame import仍复制整张 RGBA；这是同步 readback/CPU texture边界 PERF-MVP-023/120 的既有证据，不在 primitive里用第二份缓存掩盖。
- notification history已明确截断为64；单条 toast/history encoding有短期 String/Vec分配，但事件频率低于 pointer/frame热路，当前作为可接受风险，后续只有 storm trace证明成瓶颈才升级。
- `UiPerfCounter` 名称是静态字符串，非 profiling build为空实现；没有发现每 counter格式化或动态分配。

## 待动态验收

协调器下运行 floating projection、primitive、drawer resize、notification与 retained performance tests；记录1/100/1000 floating windows的 host visited count/p95，10k host rows的 clone count/bytes，以及left/right resize的 apply、metadata recompute和publish次数。所有 route/frame/tree-id、row order/data、drawer extent和failure atomicity回归通过前，不进入 `review.md`。
