---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test
  - zircon_editor/src/ui/retained_host/host_contract/template_popup_layout
  - zircon_editor/src/ui/retained_host/host_contract/template_geometry
  - zircon_editor/src/ui/retained_host/host_contract/template_component_family
  - zircon_editor/src/ui/retained_host/host_contract/frame_geometry
  - zircon_editor/src/ui/retained_host/host_contract/template_input_semantics
  - zircon_editor/src/ui/retained_host/host_contract/template_activation_semantics
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01
  - docs/plans/zircon_editor/editor_ui/08
reference_sources:
  - dev/bevy/crates/bevy_ui/src/focus.rs
  - dev/slint/internal/core/input.rs
tests:
  - existing template surface and popup row hit tests
  - ModelRc borrowed-row clone probe and uniform popup row boundary regression
  - current-source Windows zircon_editor focused Cargo pending
  - 1k pointer moves over 1/100/10k nodes/options clone and visited-count trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor template surface hit 逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`zircon_editor/src/ui/retained_host/host_contract/surface_hit_test` 当前共 **16** 个 Rust 文件、**923** 行，已逐文件阅读 **16/16**；直接支撑的`template_popup_layout` 5/5、`template_geometry` 2/2、`template_component_family` 6/6、`frame_geometry` 4/4、`template_input_semantics` 2/2、`template_activation_semantics` 4/4也已读完。本证据覆盖 **39** 文件、**1,659** 行。首次广域Cargo使用编译期间旧source snapshot，不能作为当前源码验收；当前源聚焦测试仍在FIFO等待，目录留在 `pending.md`。

## 已直接优化

PERF-MVP-141已把template surface的dispatchability pass和tree build pass改为借用 `ModelRc` rows，避免每个pane generation两次完整`TemplatePaneNodeData` clone。PERF-MVP-146的局部修复进一步让bounds、popup z扫描与base node lookup借用rows，删除`SurfaceFramePointerHit`未使用的owned control/frame，并按uniform row的Y坐标O(1)定位candidate。共享inclusive边界为保持旧语义最多检查相邻2行，与总行数无关。Tree仍只包含dispatchable nodes，node id到row的`+2`映射、clip、state和input policy保持不变。

## 待优化热点

PERF-MVP-146剩余根因：Workbench native pointer入口每次仍扫描bounds并调用`template_nodes_surface_frame`新建、填充、`rebuild`完整surface；无popup时仍需反向扫描全部nodes确认open state。局部clone与popup逐行扫描已删除，但event-time surface transaction和open-popup discovery没有generation authority。

局部修复先删除这些clone和无用DTO；EditorUI08/01随后让presentation generation发布持久hit surface与open-popup z stack，稳定pointer move既不build/rebuild surface也不做全node scan。验收必须覆盖clip、z-order、disabled/separator阻断underlay、dropdown向上翻转、bounds clamp、projected dropdown-popup frame和action/value route等价。
