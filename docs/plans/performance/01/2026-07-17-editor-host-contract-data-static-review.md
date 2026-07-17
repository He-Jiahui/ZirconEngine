---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/data/**/*.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/presentation/snapshot.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/**/*.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/viewport_surfaces.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01
  - docs/plans/zircon_editor/editor_ui/05
  - docs/plans/zircon_editor/editor_ui/08
reference_sources:
  - dev/bevy/crates/bevy_ui/src/ui_node.rs
  - dev/bevy/crates/bevy_ui/src/widget/button.rs
  - dev/bevy/crates/bevy_ui/src/widget/image.rs
  - dev/bevy/crates/bevy_ui/src/widget/text.rs
  - dev/slint/internal/core/items/image.rs
  - dev/slint/internal/core/items/input_items.rs
  - dev/slint/internal/core/items/text.rs
tests:
  - world-space host row/source-id borrowing source guard
  - existing world-space submission order/extent/hit tests
  - current-source Windows Cargo and 1/100/10k clone-byte/cache trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor host-contract data逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`zircon_editor/src/ui/retained_host/host_contract/data`当前共 **77** 个Rust文件、**2,241** 行，已逐文件阅读 **77/77**：root/host component 12/12、interaction 7/7、pane 11/11、template node 11/11、UI asset 27/27、viewport/welcome/world-space 9/9。当前源动态测试与clone-byte/cache证据未完成，因此目录留在`pending.md`。

## 已直接优化

PERF-MVP-148的局部修复让world-space submission直接遍历borrowed `ModelRc` rows，floating window同样借用；node converter只复制最终submission字段。同一pane surface id用`Cow<str>`在九类pane node集合间复用，不再每次clone。排序、world extent fallback、surface/node/control tie-break与viewport hit语义保持不变。

## P0热点

- PERF-MVP-147：`host_presentation_from_state`每次从state深clone完整`HostWindowPresentationData`，再覆盖menu/overflow/pane interaction/text focus/viewport image并应用hover。pointer move、scroll、keyboard target discovery、present、viewport surface sync与单字段toggle均调用`get_host_presentation()`；读一个字段也复制全部dock/floating pane、template model和可能的RGBA Vec。
- PERF-MVP-148：`TemplatePaneNodeData`是约160字段的最大形状DTO。普通text/button也携带sample grid、timeline、heatmap、collection、world-space、drag/drop、ripple、image和全部action字段；`PaneData`同时嵌入Welcome/Viewport/Hierarchy/Inspector/Console/Asset/UI Asset/Animation等所有payload。278处默认node构造与任何整行clone都按最大形状付出初始化、内存带宽和cache footprint。

## 参考约束与责任边界

Slint按`ImageItem`、`SimpleText`、`TouchArea`等item-specific结构只存该类型属性；Bevy保留common `Node`布局组件，并以`Button` marker、`ImageNode`、`Text`等稀疏组件组合能力。Zircon应保留一个紧凑common header/frame/identity/state，再按component family持有typed payload；`PaneData`改为active-pane tagged/shared payload，不能在paint或pointer consumer旁增加第二份cache。

EditorUI05负责compiled property/item family与typed payload generation；EditorUI08负责immutable presentation/active-pane generation，EditorUI01负责从同一generation消费hit/control index。PERF-MVP-147/148只有在1/100/10k node与1k pointer storm的full-snapshot clone、RGBA copied bytes、node size、allocation、cache miss和p95均有当前源证据，且paint/input/route/serialization parity通过后才可关闭。
